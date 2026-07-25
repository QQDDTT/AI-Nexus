use anyhow::{Context, Result};
use async_trait::async_trait;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::core::interfaces::{NodeLabel, GraphNode, GraphEdge, ActivatedNode};
use crate::utils::errors::AiNexusError;
use tracing::{error, info};

// ─────────────────────────────────────────────
// 持久化状态（postcard 序列化）
// ─────────────────────────────────────────────

#[derive(Serialize, Deserialize, Default, Clone)]
struct GraphState {
    /// 所有节点 id -> GraphNode
    nodes: HashMap<String, GraphNode>,
    /// 邻接表：from_id -> Vec<(to_id, GraphEdge)>
    edges: HashMap<String, Vec<(String, GraphEdge)>>,
}

// ─────────────────────────────────────────────
// PetGraphStore 实现
// ─────────────────────────────────────────────

/// 基于 petgraph 的内存图 + postcard 磁盘持久化的知识图谱引擎
pub struct PetGraphStore {
    file_path: PathBuf,
    state: Arc<RwLock<GraphState>>,
}

impl PetGraphStore {
    /// 从磁盘加载或初始化一个新的图存储
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let file_path = path.into();

        let state = if file_path.exists() {
            let bytes = std::fs::read(&file_path)
                .with_context(|| format!("无法读取图存储文件: {:?}", file_path))?;
            match postcard::from_bytes::<GraphState>(&bytes) {
                Ok(s) => {
                    info!(
                        "已从磁盘加载知识图谱 ({} 节点, {} 个邻接关系, {} bytes)",
                        s.nodes.len(),
                        s.edges.len(),
                        bytes.len()
                    );
                    s
                }
                Err(e) => {
                    error!("图存储反序列化失败: {}。将创建全新图谱。", e);
                    GraphState::default()
                }
            }
        } else {
            GraphState::default()
        };

        Ok(Self {
            file_path,
            state: Arc::new(RwLock::new(state)),
        })
    }

    /// 异步后台持久化到磁盘
    async fn flush_to_disk(&self) {
        let state = self.state.read().await.clone();
        let file_path = self.file_path.clone();

        tokio::spawn(async move {
            match postcard::to_stdvec(&state) {
                Ok(bytes) => {
                    if let Err(e) = tokio::fs::write(&file_path, bytes).await {
                        error!("图存储持久化失败: {}", e);
                    }
                }
                Err(e) => {
                    error!("图存储序列化失败: {}", e);
                }
            }
        });
    }

    /// 将内部邻接表构建为 petgraph DiGraph，用于图算法（激活扩散等）
    fn build_digraph(state: &GraphState) -> (DiGraph<String, f32>, HashMap<String, NodeIndex>) {
        let mut graph = DiGraph::new();
        let mut index_map: HashMap<String, NodeIndex> = HashMap::new();

        // 添加所有节点
        for id in state.nodes.keys() {
            let idx = graph.add_node(id.clone());
            index_map.insert(id.clone(), idx);
        }

        // 添加所有边
        for (from_id, adj) in &state.edges {
            if let Some(&from_idx) = index_map.get(from_id) {
                for (to_id, edge) in adj {
                    if let Some(&to_idx) = index_map.get(to_id) {
                        graph.add_edge(from_idx, to_idx, edge.weight);
                    }
                }
            }
        }

        (graph, index_map)
    }
}

#[async_trait]
#[async_trait]
impl crate::core::interfaces::GraphStorage for PetGraphStore {
    async fn upsert_node(&self, node: GraphNode) -> std::result::Result<(), AiNexusError> {
        let mut state = self.state.write().await;
        state.nodes.insert(node.id.clone(), node);
        drop(state);
        self.flush_to_disk().await;
        Ok(())
    }

    async fn upsert_edge(&self, from_id: &str, to_id: &str, edge: GraphEdge) -> std::result::Result<(), AiNexusError> {
        let mut state = self.state.write().await;
        if !state.nodes.contains_key(from_id) {
            return Err(AiNexusError::General(format!("图谱插入边失败：源节点 '{}' 不存在", from_id)));
        }
        if !state.nodes.contains_key(to_id) {
            return Err(AiNexusError::General(format!("图谱插入边失败：目标节点 '{}' 不存在", to_id)));
        }

        let adj = state.edges.entry(from_id.to_string()).or_insert_with(Vec::new);

        // 幂等：若同 from->to 的边已存在则替换
        if let Some(existing) = adj.iter_mut().find(|(tid, _)| tid == to_id) {
            existing.1 = edge;
        } else {
            adj.push((to_id.to_string(), edge));
        }

        drop(state);
        self.flush_to_disk().await;
        Ok(())
    }

    async fn spreading_activation(&self, seed_ids: &[String], top_k: usize, depth: usize) -> std::result::Result<Vec<ActivatedNode>, AiNexusError> {
        let state = self.state.read().await;
        let (graph, index_map) = Self::build_digraph(&state);

        // 衰减系数（每跳乘以该值）
        const DECAY: f32 = 0.6;

        // activation_map: NodeIndex -> 激活强度
        let mut activation_map: HashMap<NodeIndex, f32> = HashMap::new();

        // BFS 队列: (NodeIndex, 当前激活值, 剩余跳数)
        let mut queue: VecDeque<(NodeIndex, f32, usize)> = VecDeque::new();
        let mut visited: HashSet<NodeIndex> = HashSet::new();

        // 种子节点初始激活值 = 1.0
        for seed_id in seed_ids {
            if let Some(&seed_idx) = index_map.get(seed_id) {
                queue.push_back((seed_idx, 1.0, depth));
                activation_map.insert(seed_idx, 1.0);
                visited.insert(seed_idx);
            }
        }

        while let Some((current_idx, current_activation, remaining_depth)) = queue.pop_front() {
            if remaining_depth == 0 {
                continue;
            }

            // 沿出边扩散
            for edge_ref in graph.edges(current_idx) {
                let neighbor_idx = edge_ref.target();
                let edge_weight = *edge_ref.weight();
                let propagated = current_activation * edge_weight * DECAY;

                // 累加激活（允许多路径叠加）
                let entry = activation_map.entry(neighbor_idx).or_insert(0.0);
                *entry += propagated;

                if !visited.contains(&neighbor_idx) {
                    visited.insert(neighbor_idx);
                    queue.push_back((neighbor_idx, propagated, remaining_depth - 1));
                }
            }
        }

        // 排除种子节点自身，按激活值降序排列
        let seed_indices: HashSet<NodeIndex> = seed_ids
            .iter()
            .filter_map(|id| index_map.get(id))
            .copied()
            .collect();

        let mut results: Vec<ActivatedNode> = activation_map
            .into_iter()
            .filter(|(idx, _)| !seed_indices.contains(idx))
            .filter_map(|(idx, activation)| {
                let node_id = graph[idx].clone();
                state.nodes.get(&node_id).map(|node| ActivatedNode {
                    node: node.clone(),
                    activation,
                })
            })
            .collect();

        results.sort_by(|a, b| {
            b.activation
                .partial_cmp(&a.activation)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(top_k);

        Ok(results)
    }

    async fn neighbors(&self, node_id: &str) -> std::result::Result<Vec<GraphNode>, AiNexusError> {
        let state = self.state.read().await;
        let neighbors = state
            .edges
            .get(node_id)
            .map(|adj| {
                adj.iter()
                    .filter_map(|(to_id, _)| state.nodes.get(to_id).cloned())
                    .collect()
            })
            .unwrap_or_default();
        Ok(neighbors)
    }

    async fn get_nodes_by_label(&self, label: &NodeLabel) -> std::result::Result<Vec<GraphNode>, AiNexusError> {
        let state = self.state.read().await;
        let nodes = state
            .nodes
            .values()
            .filter(|n| &n.label == label)
            .cloned()
            .collect();
        Ok(nodes)
    }
}
