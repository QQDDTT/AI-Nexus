use crate::core::interfaces::ChatMessage;
use crate::storage::{BlockStore, SessionBlock};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::error;

/// Agent 上下文快照与恢复
pub struct StateSnapshot;

impl StateSnapshot {
    /// 异步将当前会话状态封包并抛给基座
    pub async fn save_context_state(
        user_id: &str,
        channel_name: &str,
        memory: Vec<ChatMessage>,
        session_store: Arc<BlockStore>,
    ) {
        let session_id = user_id.to_string();
        let source = channel_name.to_string();
        
        // Serialize short-term memory
        let memory_payload = match postcard::to_stdvec(&memory) {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to serialize short_term_memory for snapshot: {}", e);
                return;
            }
        };

        let last_heartbeat = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let block = SessionBlock {
            session_id,
            source,
            model: crate::core::config::get_config().models.global_gemini_model.clone(), // Extracted from config
            tokens: 0, // This could be updated dynamically
            status: "Done".to_string(),
            last_heartbeat,
            memory_payload,
        };

        tokio::spawn(async move {
            if let Err(e) = session_store.append_record(&block) {
                error!("Failed to append SessionBlock snapshot: {}", e);
            }
        });
    }
}
