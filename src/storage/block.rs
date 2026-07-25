use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use anyhow::{Context, Result};
use memmap2::Mmap;

/// 本地二进制块存储引擎 (基于 postcard)
/// 强制 Append-Only 写入，支持零拷贝读取
pub struct BlockStore {
    _ledger_path: PathBuf,
    ledger_file: RwLock<File>,
}

impl BlockStore {
    /// 初始化并在本地 data/blocks 创建二进制文件
    pub fn new(data_dir: &str, file_name: &str) -> Result<Self> {
        let dir_path = Path::new(data_dir);
        if !dir_path.exists() {
            std::fs::create_dir_all(dir_path).context("Failed to create data directory")?;
        }

        let ledger_path = dir_path.join(file_name);
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .read(true) // We need read access for mmap
            .open(&ledger_path)
            .context("Failed to open or create block file")?;

        Ok(Self {
            _ledger_path: ledger_path,
            ledger_file: RwLock::new(file),
        })
    }

    /// 保存任意实现了 Serialize 的数据结构，封包为纯二进制并追加写入
    /// 格式: [4 bytes length (u32)][postcard binary data]
    pub fn append_record<T: serde::Serialize>(&self, record: &T) -> Result<()> {
        let serialized = postcard::to_stdvec(record).context("Failed to serialize record")?;
        let len = serialized.len() as u32;
        
        let mut file = self.ledger_file.write().unwrap();
        // 写入长度前缀
        file.write_all(&len.to_le_bytes()).context("Failed to write length prefix")?;
        // 写入数据包
        file.write_all(&serialized).context("Failed to write block data")?;
        file.flush().context("Failed to flush block to disk")?;
        
        Ok(())
    }

    /// 读取文件中所有的二进制块并反序列化为 T 的列表
    /// 使用 mmap 实现零拷贝映射
    pub fn read_all_records<T: serde::de::DeserializeOwned>(&self) -> Result<Vec<T>> {
        let file = self.ledger_file.read().unwrap();
        let metadata = file.metadata()?;
        if metadata.len() == 0 {
            return Ok(Vec::new());
        }

        // 使用 mmap 将文件映射到内存，避免大量 I/O 拷贝
        let mmap = unsafe { Mmap::map(&*file) }.context("Failed to mmap block file")?;
        let mut offset = 0;
        let mut records = Vec::new();

        while offset < mmap.len() {
            if offset + 4 > mmap.len() {
                // Incomplete record
                break;
            }

            // 读取 4 字节长度前缀
            let len_bytes = [mmap[offset], mmap[offset + 1], mmap[offset + 2], mmap[offset + 3]];
            let len = u32::from_le_bytes(len_bytes) as usize;
            offset += 4;

            if offset + len > mmap.len() {
                // Incomplete record payload
                break;
            }

            let payload = &mmap[offset..offset + len];
            let record: T = postcard::from_bytes(payload).context("Failed to deserialize block")?;
            records.push(record);

            offset += len;
        }

        Ok(records)
    }
}
