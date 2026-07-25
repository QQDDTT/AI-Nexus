//! # 存储层二进制数据块模型
//!
//! 本模块定义的结构体（`SessionBlock`, `LedgerBlock`, `NodeStatusBlock`）
//! 是专为**二进制直接落盘**优化的存储层内部表示，
//! 与 `core::interfaces::SessionDef` 等 API 层的 DTO 分属不同分层，
//! **有意独立**：DTO 面向 REST 序列化，Block 面向 postcard 二进制持久化。
use serde::{Deserialize, Serialize};

/// 会话状态 (SessionBlock)
/// 用于持久化短期会话以及展示追踪，使用二进制直接落盘。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionBlock {
    pub session_id: String,
    pub source: String,
    pub model: String,
    pub tokens: u32,
    pub status: String, // Processing, Waiting, Done
    pub last_heartbeat: u64,
    pub memory_payload: Vec<u8>,
}

/// 计费与审计流水 (LedgerBlock)
/// 高频次追加的纯二进制账本。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerBlock {
    pub timestamp: u64,
    pub user_id: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub est_cost_usd: f64,
}

/// 网关与路由节点 (NodeStatusBlock)
/// 用于网关探活与权重策略。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatusBlock {
    pub node_id: String,
    pub node_type: String, // Gateway / ModelRouter
    pub health_status: String,
    pub metrics_payload: Vec<u8>, // 包含延迟、分流权重等打包数据
}
