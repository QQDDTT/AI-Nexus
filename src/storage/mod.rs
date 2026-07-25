use crate::core::interfaces::GraphStorage;
pub mod block;
pub mod graph;
pub mod models;
pub mod nexus_db;
pub mod vector;
pub mod snapshot;

pub use block::*;
pub use graph::*;
pub use models::*;
pub use nexus_db::*;
pub use vector::*;

use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;

/// 全局统一存储门面，贯彻读写绝缘法则
pub struct Storage {
    // L1: 热层 (In-Memory)
    pub sessions: Arc<DashMap<String, SessionBlock>>,

    // L2: 温层 (Binary BlockStore)
    pub session_store: Arc<BlockStore>,
    pub ledger_store: Arc<BlockStore>,

    // L3: 冷层 (GraphRAG & Vector)
    pub vector_store: Arc<dyn crate::core::interfaces::VectorStore>,
    pub graph_store: Arc<dyn GraphStorage>,

    // L4: 关联层 (Nexus Database)
    pub nexus_db: Arc<NexusDb>,
}

impl Storage {
    pub fn new(data_dir: &str) -> Result<Self> {
        let ledger_store = Arc::new(BlockStore::new(data_dir, "ledger.bin")?);
        let session_store = Arc::new(BlockStore::new(data_dir, "session.bin")?);

        let vector_store_path = std::path::Path::new(data_dir).join("vector_store.bin");
        let vector_store = Arc::new(HnswVectorStore::new(vector_store_path)?);

        let graph_store_path = std::path::Path::new(data_dir).join("graph_store.bin");
        let graph_store = Arc::new(PetGraphStore::new(graph_store_path)?);

        let nexus_db = Arc::new(NexusDb::new(data_dir)?);

        let sessions = DashMap::new();
        // Zero-Copy Replay / Resurrection
        if let Ok(history) = session_store.read_all_records::<SessionBlock>() {
            for block in history {
                // The latest append for a session will overwrite older ones
                sessions.insert(block.session_id.clone(), block);
            }
            tracing::info!("Resurrected {} sessions from binary snapshot.", sessions.len());
        }

        Ok(Self {
            sessions: Arc::new(sessions),
            session_store,
            ledger_store,
            vector_store,
            graph_store,
            nexus_db,
        })
    }
}
