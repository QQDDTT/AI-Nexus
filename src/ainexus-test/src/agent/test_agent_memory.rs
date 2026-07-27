use ai_nexus::agent::memory::InMemoryStore;
use ai_nexus::core::interfaces::{ChatMessage, MemoryStore, MessageContent, EmbeddingProvider, GraphStorage, VectorStore};
use ai_nexus::storage::graph::PetGraphStore;
use ai_nexus::storage::vector::HnswVectorStore;
use ai_nexus::utils::errors::AiNexusError;
use async_trait::async_trait;
use std::sync::Arc;
use tempfile::tempdir;

pub struct MockEmbeddingProvider;

#[async_trait]
impl EmbeddingProvider for MockEmbeddingProvider {
    async fn generate_embedding(&self, _text: &str) -> Result<Vec<f32>, AiNexusError> {
        Ok(vec![0.0; 768])
    }
}

#[tokio::test]
async fn test_short_term_folding() {
    let dir = tempdir().unwrap();
    let graph_store = Arc::new(PetGraphStore::new(dir.path().join("graph.bin")).unwrap()) as Arc<dyn GraphStorage>;
    let vector_store = Arc::new(HnswVectorStore::new(dir.path().join("vec.bin")).unwrap()) as Arc<dyn VectorStore>;
    let mut store = InMemoryStore::new(graph_store, vector_store, Arc::new(MockEmbeddingProvider));
    
    // 假设 1 token = 4 字符
    // Max 10 tokens = 40 chars
    
    store.push_short_term(ChatMessage {
        role: "user".to_string(),
        contents: vec![MessageContent::Text("1234567890".to_string())], // 10 chars
    });
    
    store.push_short_term(ChatMessage {
        role: "model".to_string(),
        contents: vec![MessageContent::Text("1234567890".to_string())], // 10 chars
    });
    
    store.push_short_term(ChatMessage {
        role: "user".to_string(),
        contents: vec![MessageContent::Text("1234567890".to_string())], // 10 chars
    });
    
    // 目前总计 30 chars，如果 max_tokens 是 10 (40 chars)，应该全部保留
    let folded = store.get_folded_context(10);
    assert_eq!(folded.len(), 3);
    
    // 再塞入 20 字符
    store.push_short_term(ChatMessage {
        role: "model".to_string(),
        contents: vec![MessageContent::Text("12345678901234567890".to_string())], // 20 chars
    });
    
    // 目前总共 50 chars。获取 max_tokens=10 (40 chars)。
    // 从后往前：
    // 最后一条 20 chars (剩余 20)
    // 倒数第二条 10 chars (剩余 10)
    // 倒数第三条 10 chars (剩余 0)
    // 第一条会被截断
    let folded_truncated = store.get_folded_context(10);
    assert_eq!(folded_truncated.len(), 3);
    assert_eq!(folded_truncated[0].role, "model"); // 这是原来的第二条
}
