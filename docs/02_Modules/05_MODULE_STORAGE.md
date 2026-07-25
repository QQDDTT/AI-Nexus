# Storage 模块设计规范 (Storage & Database Module Specification)

`src/storage` 模块是 AI-Nexus 避免“失忆”的核心底座。大语言模型应用由于极度依赖“上下文历史”，一旦发生进程崩溃重启，内存数据丢失将造成毁灭性的用户体验。为贯彻全局“读写绝缘法则”，我们将采用完全独立的**原生的 BlockStore 二进制存储**方案。

## 1. 核心职责 (Core Responsibilities)

1.  **数据持久化与状态快照 (State Persistence)**：定期将内存中的 `AgentContext`（短期记忆与状态机）通过纯二进制序列化落盘。在系统重启时，做到微秒级状态恢复 (Resurrection)。
2.  **图谱与向量记忆库 (Graph & Vector Database)**：管理长期记忆 (Long-term Memory) 和动态挂载的 Skill Schema，基于 GraphRAG 拓扑与 HNSW 内存映射实现极速的实体联想与相似度搜索。
3.  **计费与审计流水 (Billing & Audit Ledger)**：高频记录全系统的 API Token 消耗日志、Meta Agent 操作日志，以追加写入 (Append-only) 的方式记录。

## 2. 关键设计与架构 (Key Designs)

### 2.1 分层存储策略 (Tiered Storage)

不同于传统的嵌入式关系型数据库，我们彻底拥抱读写分离、冷热隔离，以及完全独立的二进制基座架构：

*   **L1 内存热层 (In-Memory Cache)**：
    *   **选型**：使用 Rust 本地的 `moka` 或 `dashmap`。
    *   **存储内容**：正在活跃对话的 `AgentContext`。极高频的读写，绝不走网络和磁盘 I/O。
*   **L2 持久化温层 (Binary Block Storage)**：
    *   **选型**：在同进程内挂载的自研底层追加写引擎 (`BlockStore`)，结合 `postcard` 纯二进制无模式序列化直接操作本地文件。
    *   **存储内容**：用户的静态权限配置、已结束的对话历史 (归档)、Token 消耗账单流水 (Block-L/I/E)。
*   **L3 语义冷层 (GraphRAG & HNSW Vector Store)**：
    *   **选型**：基座自带的高维稠密向量持久化检索树 (HNSW 内存映射) 结合轻量级图数据库引擎 (GraphDB)。
    *   **存储内容**：经过实体抽取 (Entity Extraction) 与 Embedding 转化的实体知识网络 (Knowledge Graph)、经过社区检测算法生成的宏观摘要，以及动态挂载的 Skill Schema 向量。支持跨多文档的多跳推理与激活扩散模型 (Spreading Activation) 的联想记忆机制。

### 2.2 优雅崩溃与复活机制 (Resurrection Mechanism)

为了应对服务突发崩溃：
1.  **二进制增量日志 (Delta Record)**：`OS` 模块每次完成对 `AgentContext` 短期记忆的更新后，`storage` 会以 `postcard` 序列化方式，在后台向自研基座发送一条异步二进制追加更新。
2.  **零拷贝回放 (Zero-Copy Replay)**：当 `Nexus OS` 进程重新启动时，通过 `mmap` 零拷贝技术直接映射底层 `.bin` 文件，将未失效的 Context 反序列化回内存，实现真正的微秒级热启动。

## 3. 核心接口与数据结构

```rust
pub mod database {
    use crate::core::protocol::DataBlock;

    /// 本地二进制块存储引擎 (基于 postcard)
    pub struct BlockStore {
        ledger_path: std::path::PathBuf,
        ledger_file: std::sync::Arc<std::sync::RwLock<std::fs::File>>,
    }

    impl BlockStore {
        /// 初始化并在本地 data/blocks 创建二进制文件
        pub fn new(data_dir: &str) -> Result<Self, anyhow::Error> { /* ... */ }
        
        /// 保存计费流水，封包为纯二进制并追加写入 (Append-only)
        pub fn append_ledger(&self, record: &LedgerRecord) -> Result<(), anyhow::Error> { /* ... */ }
    }
}

pub mod vector {
    /// 长期记忆与技能向量持久化接口
    pub trait VectorStorage: Send + Sync {
        /// 向 HNSW 树插入高维 f32 向量
        async fn upsert_embedding(&self, collection: &str, id: &str, vector: &[f32], payload: Vec<u8>) -> Result<(), anyhow::Error>;
        
        /// 语义相似度搜索
        async fn similarity_search(&self, collection: &str, query: &[f32], limit: usize) -> Result<Vec<SearchResult>, anyhow::Error>;
    }
}

pub mod snapshot {
    /// Agent 上下文快照与恢复
    pub struct StateSnapshot;

    impl StateSnapshot {
        /// 异步将当前会话状态封包并抛给基座
        pub async fn save_context_state(ctx: &crate::core::protocol::AgentContext) -> Result<(), anyhow::Error> { /* ... */ }
    }
}
```

## 4. 最佳实践与约束

1.  **绝不阻塞主线程**：所有对 `L2` 和 `L3` 层基座的写入操作，必须通过 `tokio::spawn` 扔到后台异步执行，或放入队列中由专属的 DB Worker 批量消费写入，绝对不允许让大模型的请求因此卡顿等待。
2.  **避免脏写与锁竞争**：遵守“读写绝缘法则”，所有的运行时演化必须是追加写入（Append-only），结合基座系统的垃圾回收 (GC) 机制，实现历史溯源与因果防篡改。

## 5. 二进制数据模型 (BlockStore Binary Schemas)

由于摒弃了关系型数据库的 SQL Schema，所有持久化实体由 Rust 原生 `struct` 配合 `serde` 直接映射为底层二进制：

### 5.1 会话状态 (SessionBlock)
用于持久化短期会话以及展示追踪，使用二进制直接落盘。
- `session_id`: 追踪 UUID
- `source`: 渠道来源
- `capability_requirement`: 算力能力要求标签 (如 Tier-1-Logic)
- `tokens`: 消耗的 Token 计数
- `status`: 当前状态 (Processing, Waiting)
- `last_heartbeat`: 最后心跳时间戳

### 5.2 计费与审计流水 (LedgerBlock)
高频次追加的纯二进制账本。
- `timestamp`: 调用发生的时间戳
- `user_id`: 调用方身份标识
- `model`: 实际调用的具体模型
- `input_tokens` / `output_tokens`: 消耗 Token
- `est_cost_usd`: 预估美金账单

### 5.3 网关与路由节点 (NodeStatusBlock)
替代原有 `gateways` 和 `model_routing` 表，用于网关探活与权重策略。
- `node_id`: 节点或网关的唯一标识
- `node_type`: 节点类型 (Gateway / ModelRouter)
- `health_status`: 运行状态
- `metrics`: 包含延迟、过去24小时请求量、分流权重等打包数据
