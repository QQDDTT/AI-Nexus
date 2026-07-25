use crate::core::interfaces::{Skill, SkillRegistry, EmbeddingProvider};
use crate::core::interfaces::{GraphEdge, GraphNode, GraphStorage, NodeLabel};
use crate::utils::errors::AiNexusError;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// GraphRAG 技能注册中心
///
/// 技能注册时同时写入：
/// 1. 内存技能表（用于直接调用）
/// 2. GraphStorage 节点（用于拓扑关联与联想激活扩散召回）
/// 3. VectorStore 锚点（用于语义相似度初步召回作为种子）
pub struct GraphSkillRegistry {
    /// 内存技能表：skill_name -> Skill
    skills: Arc<RwLock<HashMap<String, Arc<dyn Skill>>>>,
    /// 图谱存储引擎
    graph_store: Arc<dyn GraphStorage>,
    /// 向量存储引擎
    vector_store: Arc<dyn crate::core::interfaces::VectorStore>,
    /// Embedding 转换提供者
    embedding_provider: Arc<dyn EmbeddingProvider>,
}

impl GraphSkillRegistry {
    pub fn new(
        graph_store: Arc<dyn GraphStorage>,
        vector_store: Arc<dyn crate::core::interfaces::VectorStore>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
    ) -> Self {
        Self {
            skills: Arc::new(RwLock::new(HashMap::new())),
            graph_store,
            vector_store,
            embedding_provider,
        }
    }

    /// 在两个技能节点之间建立拓扑关联边（建立技能之间的"联想"路径）
    pub async fn link_skills(
        &self,
        from_skill: &str,
        to_skill: &str,
        relation: &str,
        weight: f32,
    ) -> Result<(), AiNexusError> {
        self.graph_store
            .upsert_edge(
                from_skill,
                to_skill,
                GraphEdge {
                    relation: relation.to_string(),
                    weight,
                },
            )
            .await
            .map_err(|e| AiNexusError::General(format!("技能关联边写入失败: {}", e)))?;
        Ok(())
    }

    /// 通过语义相似度（HNSW）找到种子技能，再通过图谱激活扩散扩展召回
    /// 这是 GraphRAG 混合召回的核心流程
    pub async fn graphrag_recall(
        &self,
        query_embedding: &[f32],
        top_k: usize,
        depth: usize,
    ) -> Result<Vec<Arc<dyn Skill>>, AiNexusError> {
        // Step 1: 向量相似度初步召回（获取种子节点 ID）
        let vector_results = self
            .vector_store
            .search("skills", query_embedding, top_k)
            .await
            .map_err(|e| AiNexusError::General(format!("向量召回失败: {}", e)))?;

        let seed_ids: Vec<String> = vector_results.iter().map(|r| r.id.clone()).collect();

        if seed_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Step 2: 以种子节点为出发点，在图谱上做激活扩散（联想扩展）
        let activated = self
            .graph_store
            .spreading_activation(&seed_ids, top_k * 2, depth)
            .await
            .map_err(|e| AiNexusError::General(format!("激活扩散失败: {}", e)))?;

        // Step 3: 合并种子集合 + 激活节点，去重后加载技能实例
        let skills = self.skills.read().await;
        let mut result_ids: Vec<String> = seed_ids.clone();
        for activated_node in activated {
            if !result_ids.contains(&activated_node.node.id) {
                result_ids.push(activated_node.node.id);
            }
        }

        let matched: Vec<Arc<dyn Skill>> = result_ids
            .iter()
            .take(top_k)
            .filter_map(|id| skills.get(id).cloned())
            .collect();

        info!(
            "GraphRAG 技能召回完成: 向量种子={}, 图谱扩展后={}, 最终返回={}",
            seed_ids.len(),
            result_ids.len(),
            matched.len()
        );

        Ok(matched)
    }
}

#[async_trait]
impl SkillRegistry for GraphSkillRegistry {
    /// 注册技能：写入内存表 + GraphStorage 节点 + VectorStore 向量锚点
    async fn register_skill(&self, skill: Box<dyn Skill>) -> Result<(), AiNexusError> {
        let skill_name = skill.name().to_string();
        let schema_json = serde_json::to_string(&skill.schema())
            .unwrap_or_default();

        // 请求 API 获取真实向量（使用 schema_json 作为 embedding 的文本）
        let embedding = self.embedding_provider.generate_embedding(&schema_json).await?;

        // 1. 写入图谱节点
        let node = GraphNode {
            id: skill_name.clone(),
            label: NodeLabel::Skill,
            properties: schema_json.as_bytes().to_vec(),
            embedding: embedding.clone(),
        };

        self.graph_store
            .upsert_node(node)
            .await
            .map_err(|e| AiNexusError::General(format!("技能图谱节点写入失败: {}", e)))?;

        // 2. 写入向量锚点
        self.vector_store
            .upsert("skills", &skill_name, embedding.clone(), schema_json.into_bytes())
            .await
            .map_err(|e| AiNexusError::General(format!("技能向量锚点写入失败: {}", e)))?;

        // 3. 写入内存技能表
        let mut skills = self.skills.write().await;
        skills.insert(skill_name.clone(), Arc::from(skill));
        drop(skills);

        // 4. GraphRAG 自动拓扑建链 (Spreading Edge Construction)
        // 寻找与当前技能最相似的 top_3 技能，建立 relates_to 边
        if let Ok(matched_skills) = self.graphrag_recall(&embedding, 3, 1).await {
            for neighbor in matched_skills {
                let neighbor_name = neighbor.name();
                if neighbor_name != skill_name {
                    // 建立双向无向边拓扑
                    let _ = self.link_skills(&skill_name, neighbor_name, "relates_to", 0.8).await;
                    let _ = self.link_skills(neighbor_name, &skill_name, "relates_to", 0.8).await;
                }
            }
        }

        info!("技能已注册至 GraphRAG 图谱并完成拓扑建链: skill_name={}", skill_name);

        Ok(())
    }

    /// 基于用户意图的技能召回：真实调用 Embedding 后，走 GraphRAG 混合召回流程
    async fn retrieve_relevant_skills(
        &self,
        intent: &str,
        limit: usize,
    ) -> Result<Vec<Box<dyn Skill>>, AiNexusError> {
        let embedding = self.embedding_provider.generate_embedding(intent).await?;
        
        let matched = self.graphrag_recall(&embedding, limit, 2).await?;
        
        let result: Vec<Box<dyn Skill>> = matched
            .into_iter()
            .map(|s| Box::new(SkillProxy(s.clone())) as Box<dyn Skill>)
            .collect();
            
        Ok(result)
    }

    async fn get_skill(&self, name: &str) -> Option<Box<dyn Skill>> {
        let skills = self.skills.read().await;
        skills.get(name).map(|s| Box::new(SkillProxy(s.clone())) as Box<dyn Skill>)
    }

    async fn list_all_skills(&self) -> Vec<Box<dyn Skill>> {
        let skills = self.skills.read().await;
        skills.values().map(|s| Box::new(SkillProxy(s.clone())) as Box<dyn Skill>).collect()
    }
}

/// 将 Arc<dyn Skill> 包装为 Box<dyn Skill> 的代理层
struct SkillProxy(Arc<dyn Skill>);

#[async_trait]
impl Skill for SkillProxy {
    fn name(&self) -> &str { self.0.name() }
    fn schema(&self) -> serde_json::Value { self.0.schema() }
    fn requires_human_approval(&self) -> bool { self.0.requires_human_approval() }
    fn validate_and_repair(&self, params: serde_json::Value) -> Result<serde_json::Value, AiNexusError> {
        self.0.validate_and_repair(params)
    }
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, AiNexusError> {
        self.0.execute(params).await
    }
    fn verify_result(&self, params: &serde_json::Value, result: &serde_json::Value) -> Result<(), AiNexusError> {
        self.0.verify_result(params, result)
    }
}
