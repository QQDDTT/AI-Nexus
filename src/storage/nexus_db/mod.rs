pub mod collection;
pub mod wal;

use anyhow::Result;
use collection::Collection;
use dashmap::DashMap;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use wal::{WalOp, WriteAheadLog};

pub struct NexusDb {
    _db_dir: PathBuf,
    collections: DashMap<String, Arc<Collection>>,
    wal: WriteAheadLog,
}

impl NexusDb {
    pub fn new<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let db_dir = dir.as_ref().to_path_buf();
        std::fs::create_dir_all(&db_dir)?;

        let wal_path = db_dir.join("nexus_wal.bin");
        let ops = WriteAheadLog::read_all_ops(&wal_path).unwrap_or_default();
        let wal = WriteAheadLog::new(&wal_path)?;

        let db = Self {
            _db_dir: db_dir,
            collections: DashMap::new(),
            wal,
        };

        db.replay_ops(ops);

        Ok(db)
    }

    fn replay_ops(&self, ops: Vec<WalOp>) {
        for op in ops {
            match op {
                WalOp::Insert { collection, key, payload_json } => {
                    if let Ok(value) = serde_json::from_str::<Value>(&payload_json) {
                        self.collection(&collection).insert(key, value);
                    }
                }
                WalOp::Delete { collection, key } => {
                    self.collection(&collection).delete(&key);
                }
            }
        }
    }

    pub fn collection(&self, name: &str) -> Arc<Collection> {
        self.collections
            .entry(name.to_string())
            .or_insert_with(|| Arc::new(Collection::new(name.to_string())))
            .clone()
    }

    pub fn insert(&self, collection: &str, key: &str, value: Value) -> Result<()> {
        let payload_json = serde_json::to_string(&value)?;
        self.wal.append(&WalOp::Insert {
            collection: collection.to_string(),
            key: key.to_string(),
            payload_json,
        })?;
        self.collection(collection).insert(key.to_string(), value);
        Ok(())
    }

    pub fn delete(&self, collection: &str, key: &str) -> Result<()> {
        self.wal.append(&WalOp::Delete {
            collection: collection.to_string(),
            key: key.to_string(),
        })?;
        self.collection(collection).delete(key);
        Ok(())
    }
}
