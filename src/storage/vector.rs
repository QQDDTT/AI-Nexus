use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use crate::core::interfaces::SearchResult;
use crate::utils::errors::AiNexusError;

#[derive(Serialize, Deserialize, Clone)]
pub struct VectorData {
    pub id: String,
    pub vector: Vec<f32>,
    pub payload: Vec<u8>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
struct VectorStoreState {
    collections: HashMap<String, Vec<VectorData>>,
}


pub struct HnswVectorStore {
    file_path: PathBuf,
    state: Arc<RwLock<VectorStoreState>>,
}

impl HnswVectorStore {
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Self> {
        let file_path = path.into();
        
        let state = if file_path.exists() {
            let bytes = std::fs::read(&file_path)
                .with_context(|| format!("Failed to read vector store file at {:?}", file_path))?;
            match postcard::from_bytes(&bytes) {
                Ok(s) => {
                    info!("Loaded HNSW Vector Store from disk ({} bytes)", bytes.len());
                    s
                },
                Err(e) => {
                    error!("Failed to deserialize vector store: {}. Creating a new one.", e);
                    VectorStoreState::default()
                }
            }
        } else {
            VectorStoreState::default()
        };

        Ok(Self {
            file_path,
            state: Arc::new(RwLock::new(state)),
        })
    }

    async fn flush_to_disk(&self) {
        let state = self.state.read().await.clone();
        let file_path = self.file_path.clone();
        
        tokio::spawn(async move {
            match postcard::to_stdvec(&state) {
                Ok(bytes) => {
                    if let Err(e) = tokio::fs::write(&file_path, bytes).await {
                        error!("Failed to flush vector store to disk: {}", e);
                    }
                }
                Err(e) => {
                    error!("Failed to serialize vector store: {}", e);
                }
            }
        });
    }
}

#[async_trait]
impl crate::core::interfaces::VectorStore for HnswVectorStore {
    async fn upsert(&self, collection: &str, id: &str, vector: Vec<f32>, payload: Vec<u8>) -> std::result::Result<(), AiNexusError> {
        let mut state = self.state.write().await;
        let col = state.collections.entry(collection.to_string()).or_insert_with(Vec::new);
        
        // If it already exists, replace it
        if let Some(existing) = col.iter_mut().find(|v| v.id == id) {
            existing.vector = vector.to_vec();
            existing.payload = payload;
        } else {
            col.push(VectorData {
                id: id.to_string(),
                vector: vector.clone(),
                payload,
            });
        }
        
        // Drop lock before flushing to avoid holding it during task spawn
        drop(state);
        
        self.flush_to_disk().await;
        Ok(())
    }

    async fn search(&self, collection: &str, query_vector: &[f32], limit: usize) -> std::result::Result<Vec<SearchResult>, AiNexusError> {
        let state = self.state.read().await;
        
        let col = match state.collections.get(collection) {
            Some(c) => c,
            None => return Ok(Vec::new()),
        };

        let mut results = Vec::with_capacity(col.len());
        
        let query_norm = query_vector.iter().map(|v| v * v).sum::<f32>().sqrt();
        if query_norm == 0.0 {
            return Ok(Vec::new()); // Avoid division by zero
        }

        for data in col {
            if data.vector.len() != query_vector.len() {
                continue;
            }
            
            let dot_product: f32 = data.vector.iter().zip(query_vector.iter()).map(|(a, b)| a * b).sum();
            let data_norm = data.vector.iter().map(|v| v * v).sum::<f32>().sqrt();
            
            if data_norm == 0.0 {
                continue;
            }
            
            let score = dot_product / (query_norm * data_norm);
            results.push(SearchResult {
                id: data.id.clone(),
                score,
                payload: data.payload.clone(),
            });
        }

        // Sort by score descending
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        results.truncate(limit);
        Ok(results)
    }
}
