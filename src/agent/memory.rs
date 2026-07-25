use crate::core::interfaces::{ChatMessage, MemoryStore, MessageContent, EmbeddingProvider};
use crate::core::interfaces::{GraphNode, GraphStorage, NodeLabel};
use crate::utils::errors::AiNexusError;
use async_trait::async_trait;
use std::sync::Arc;

/// 支持 GraphRAG 联想记忆的 Agent 记忆引擎
pub struct InMemoryStore {
    /// 短期记忆：当前会话滑动窗口
    short_term: Vec<ChatMessage>,
    /// 图谱存储引擎（用于长期记忆写入与联想扩散）
    graph_store: Arc<dyn GraphStorage>,
    /// 向量存储引擎（与图谱节点协同锚定）
    vector_store: Arc<dyn crate::core::interfaces::VectorStore>,
    /// Embedding 转换提供者
    embedding_provider: Arc<dyn EmbeddingProvider>,
}

impl InMemoryStore {
    pub fn new(
        graph_store: Arc<dyn GraphStorage>,
        vector_store: Arc<dyn crate::core::interfaces::VectorStore>,
        embedding_provider: Arc<dyn EmbeddingProvider>,
    ) -> Self {
        Self {
            short_term: Vec::new(),
            graph_store,
            vector_store,
            embedding_provider,
        }
    }
}

#[async_trait]
impl MemoryStore for InMemoryStore {
    fn push_short_term(&mut self, msg: ChatMessage) {
        self.short_term.push(msg);
    }

    fn get_folded_context(&self, max_tokens: usize) -> Vec<ChatMessage> {
        // 简化的折叠逻辑：从新到老累加文本长度，若超出阈值则截断更老的对话
        // 假设 1 token ≈ 4 字符
        let max_chars = max_tokens * 4;
        let mut current_chars = 0;
        let mut folded = Vec::new();

        for msg in self.short_term.iter().rev() {
            let mut msg_chars = 0;
            for content in &msg.contents {
                if let MessageContent::Text(text) = content {
                    msg_chars += text.len();
                }
            }

            if current_chars + msg_chars > max_chars && !folded.is_empty() {
                break;
            }

            current_chars += msg_chars;
            folded.push(msg.clone());
        }

        folded.into_iter().rev().collect()
    }

    /// 将关键事实写入 GraphDB 节点，并同步向量锚点到 VectorStore
    async fn save_long_term(
        &self,
        entity_id: &str,
        entity_type: &str,
        content: &str,
    ) -> Result<(), AiNexusError> {
        // 将 content 序列化为 properties（简单使用 UTF-8 字节）
        let properties = content.as_bytes().to_vec();

        // 请求 API 获取真实向量
        let embedding = self.embedding_provider.generate_embedding(content).await?;

        // 1. 写入图谱节点
        let node = GraphNode {
            id: entity_id.to_string(),
            label: NodeLabel::Memory,
            properties: properties.clone(),
            embedding: embedding.clone(),
        };

        self.graph_store
            .upsert_node(node)
            .await
            .map_err(|e| AiNexusError::General(format!("图谱节点写入失败: {}", e)))?;

        tracing::info!(
            "长期记忆已写入图谱: entity_id={}, type={}, content_len={}",
            entity_id,
            entity_type,
            content.len()
        );

        // 2. 同步向量锚点
        self.vector_store
            .upsert("memories", entity_id, embedding.clone(), properties)
            .await
            .map_err(|e| AiNexusError::General(format!("向量锚点写入失败: {}", e)))?;

        Ok(())
    }

    /// 联想记忆召回：从种子实体出发通过激活扩散返回关联记忆的内容摘要
    async fn recall_associative(
        &self,
        seed_entity_ids: &[String],
        top_k: usize,
        depth: usize,
    ) -> Result<Vec<String>, AiNexusError> {
        let activated = self
            .graph_store
            .spreading_activation(seed_entity_ids, top_k, depth)
            .await
            .map_err(|e| AiNexusError::General(format!("激活扩散查询失败: {}", e)))?;

        // 将激活的节点的 properties（UTF-8 内容）提取为字符串摘要
        let summaries: Vec<String> = activated
            .into_iter()
            .filter_map(|activated_node| {
                String::from_utf8(activated_node.node.properties).ok()
            })
            .collect();

        tracing::debug!(
            "联想记忆召回完成: seeds={:?}, top_k={}, depth={}, results={}",
            seed_entity_ids,
            top_k,
            depth,
            summaries.len()
        );

        Ok(summaries)
    }
}
