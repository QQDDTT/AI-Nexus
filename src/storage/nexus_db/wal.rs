use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::sync::RwLock;

/// Mutation Operations recorded in WAL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalOp {
    Insert {
        collection: String,
        key: String,
        payload_json: String,
    },
    Delete {
        collection: String,
        key: String,
    },
}

pub struct WriteAheadLog {
    file: RwLock<File>,
}

impl WriteAheadLog {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(path.as_ref())
            .context("Failed to open WAL file")?;

        Ok(Self {
            file: RwLock::new(file),
        })
    }

    pub fn append(&self, op: &WalOp) -> Result<()> {
        let mut encoded = postcard::to_allocvec(op)?;
        // Simple length-prefixed framing (u32 LE)
        let len = encoded.len() as u32;
        let mut frame = len.to_le_bytes().to_vec();
        frame.append(&mut encoded);

        let mut file = self.file.write().unwrap();
        file.write_all(&frame)?;
        file.sync_data()?;
        Ok(())
    }

    pub fn read_all_ops<P: AsRef<Path>>(path: P) -> Result<Vec<WalOp>> {
        if !path.as_ref().exists() {
            return Ok(vec![]);
        }

        let mut file = File::open(path)?;
        let mut ops = Vec::new();

        loop {
            let mut len_buf = [0u8; 4];
            match file.read_exact(&mut len_buf) {
                Ok(_) => {
                    let len = u32::from_le_bytes(len_buf) as usize;
                    let mut payload = vec![0u8; len];
                    file.read_exact(&mut payload)?;
                    if let Ok(op) = postcard::from_bytes(&payload) {
                        ops.push(op);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(ops)
    }
}
