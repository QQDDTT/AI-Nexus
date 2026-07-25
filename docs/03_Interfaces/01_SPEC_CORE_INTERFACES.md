# Core Interfaces Specification (底层基础契约规范)

为了保证 AI-Nexus 中各个子系统（Nexus OS, Model Router, Gemini Core, Skill Engine 等）的绝对解耦与独立演进，系统强制所有跨模块通信与资源调用都必须依赖定义在 `src/core` 中的底层 Trait 与枚举契约。

本文档详细定义了这些核心契约的抽象骨架。

## 1. 跨进程通信：ACP 通信协议 (Agent Control Protocol)

ACP 协议是 AI-Nexus 内部运作的二进制血脉，推荐底层通过 `tokio::mpsc` 等内存 Channel 配合 `bincode` 序列化传输。

### 1.1 路由标识 (Component)
系统中的任何节点都必须使用此枚举作为身份寻址：

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Component {
    NexusOS,
    ModelRouter,
    GeminiCore,
    SkillRouter,
    /// 包含具体技能的 ID，用于沙箱精准寻址
    SkillEngine(String), 
}

// -----------------------------------------------------------------------------
// 核心领域实体模型 (Core Domain Models)
// -----------------------------------------------------------------------------

/// 大模型服务商凭证定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderDef {
    pub id: String,
    pub name: String,
    pub api_key: String,
    pub base_url: Option<String>,
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 静态人格与模版设定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaDef {
    pub id: String,
    pub name: String,
    pub base_prompt: String,
    pub allowed_skills: Vec<String>,
    pub tone: Option<String>,
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 动态智能体实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDef {
    pub id: String,
    pub name: String,
    pub persona_id: String,
    pub capability_requirement: String,
    pub status: String,
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 物理接入渠道网关
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayDef {
    pub id: String,
    pub gateway_type: String,
    pub bound_persona_id: String,
    pub status: String,
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}
```

### 1.2 业务载荷 (AcpPayload)
所有的具体业务逻辑被封装进强类型的负载：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AcpPayload {
    /// 向上请求算力：携带系统提示词和用户输入
    InferenceRequest { 
        prompt: String, 
        capability_requirement: String // 取代原先的 target_model，用于交给算力中心动态路由
    },
    /// 向下发起动作（总线层路由信封）：OS → Skill Router 的路由消息，
    /// 使用 skill_name 做初步寻址。Skill Router 确认技能存在后，
    /// 将其展开为精确的 ExecutionDsl::Skill { skill_id(UUID), ... } 下发沙箱执行。
    /// 注意：ActionTrigger 是总线层概念，ExecutionDsl 是执行层概念，二者是层次关系。
    ActionTrigger { 
        skill_name: String, 
        parameters: serde_json::Value 
    },
    /// 异步结果回传（与执行层 ExecutionOutcome 对齐）
    ActionResult { 
        /// 执行是否成功
        success: bool,
        /// HTTP 风格状态码（200 = 成功，4xx/5xx = 失败）
        status_code: u16, 
        /// 技能执行结果（对齐 SKILL.md output Schema）
        data: serde_json::Value,
        /// 执行失败原因，success 为 false 时必填
        error: Option<String>,
    },
    /// 多智能体任务外包 (Swarm Delegation)
    SwarmDelegation {
        target_agent: String,
        task_description: String,
    },
    /// 流式输出推送块 (Streaming SSE Chunk)
    StreamingChunk {
        chunk_data: String,
        is_final: bool,
    },
    /// 物理探活心跳
    Ping,
}
```

### 1.3 通信信封 (AcpMessage)
完整的协议帧结构，包含必要的分布式追踪元数据：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpMessage {
    /// 全局唯一的请求链路追踪 ID (UUID)
    pub trace_id: String,       
    /// 发送方
    pub source: Component,        
    /// 接收方
    pub target: Component,        
    /// 发生时间的时间戳 (Unix epoch ms)
    pub timestamp: u64,           
    /// 协议具体内容
    pub payload: AcpPayload,      
}
```

---

### 1.4 执行层指令与回执 (Execution Layer)

执行层直接面向沙箱，负责将总线层的人类可读动作转化为精确的物理执行指令，并返回结构化的回执。

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionDsl {
    Skill {
        /// 技能唯一标识符 (UUID，防重名冲突)
        skill_id: String,
        /// 技能名称
        title: String,
        /// 技能入参 (对齐声明的 input 数据字典)
        input: serde_json::Value,
        /// 沙箱最大运行超时限制 (毫秒)
        timeout_ms: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOutcome {
    /// 物理执行是否成功
    pub success: bool,
    /// HTTP 风格状态码（200 = 成功，4xx/5xx = 失败），以便无损转入 ActionResult
    pub status_code: u16,
    /// 技能输出结果（统一为 JSON 结构）
    pub data: serde_json::Value,
    /// 错误原因描述 (若 success 为 false)
    pub error: Option<String>,
}
```

## 2. 动态调度：知识图谱与技能检索契约

为避免上下文臃肿，大模型在推理前必须通过基于 GraphRAG 的多跳联想网络“临时加载”相关的实体知识与技能定义。

### 2.1 文本向量化器 (EmbeddingProvider)
```rust
use async_trait::async_trait;

#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// 将一整段文本转换为密集向量 (例如通过 Gemini Embedding)
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, crate::utils::errors::AiNexusError>;
}
```

### 2.2 混合图谱与向量树检索库 (VectorStore / GraphStore)

> [!NOTE]
> `VectorStore` 为异步 Trait，内部应使用 `Arc<RwLock<...>>` 封装可变状态，
> 禁止在 `&mut self` 设计下对外暴露，避免高并发环境下的锁争用。

```rust
/// 向量检索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    pub payload: Vec<u8>,
}

#[async_trait]
pub trait VectorStore: Send + Sync {
    /// 插入或更新一条数据及其对应的向量（内部可安全并发）
    async fn upsert(&self, collection: &str, id: &str, vector: Vec<f32>, payload: Vec<u8>) -> Result<(), crate::utils::errors::AiNexusError>;
    
    /// 基于给定的 query_vector，返回最相似的 top_k 个结果
    async fn search(&self, collection: &str, query_vector: &[f32], top_k: usize) -> Result<Vec<SearchResult>, crate::utils::errors::AiNexusError>;
}
```

### 2.3 高级接口：技能管家 (SkillRegistry)
聚合底层向量库，对外提供业务级接口。

```rust
#[async_trait]
pub trait SkillRegistry: Send + Sync {
    /// 注册一个技能，内部完成 schema 到 vector 的转换并存入 VectorStore
    async fn register_skill(&mut self, skill: Box<dyn Skill>) -> Result<(), crate::utils::errors::AiNexusError>;
    
    /// 给定用户当前的意图（模糊文本），动态召回最应该挂载给大模型的 Top-K 个技能
    async fn retrieve_relevant_skills(&self, intent: &str, limit: usize) -> Result<Vec<Box<dyn Skill>>, crate::utils::errors::AiNexusError>;
}
```

---

## 3. 物理执行：技能生命周期与标准契约 (Skill Interface)

在成熟的 Agent 平台中，技能不能只是简单的函数调用，必须具备完整的“定义 -> 校验 -> 执行 -> 审核”流水线。无论是内置的 Rust Native 代码，还是大模型临时生成的沙箱脚本，都必须遵循此生命周期 `Trait`。

```rust
#[async_trait]
pub trait Skill: Send + Sync {
    /// 1. 获取该技能的全局唯一英文标识 (例如 "search_google")
    fn name(&self) -> &str;
    
    /// 2. 完整信息组装 (Prompt Injection)
    /// 返回 OpenAPI / JSON Schema 格式的接口描述及详细 Prompt 指导。
    /// 包含技能的用途、参数列表、必填项约束等，大模型将完全依靠此描述来推断如何使用该技能。
    fn schema(&self) -> serde_json::Value;
    
    /// [新增] 是否为高危敏感技能，需要 OS 触发 HITL 流程阻塞等待人工审批
    fn requires_human_approval(&self) -> bool {
        false
    }
    
    /// 3. 执行前校验与修复 (Pre-validation & Repair)
    /// 拦截大模型生成的参数，在物理执行前进行安全与逻辑校验。
    /// 允许对缺失的非关键参数进行算法级修复 (Fallback) 并返回安全的参数结构。
    /// 若参数极度危险或格式严重错误导致无法修复，则抛出 Error，系统会自动将 Error 抛回给大脑要求其修正。
    fn validate_and_repair(&self, params: serde_json::Value) -> Result<serde_json::Value, crate::utils::errors::AiNexusError> {
        // 默认实现：不干预，直接透传大模型生成的参数
        Ok(params)
    }
    
    /// 4. 实际的物理执行核心 (Execution)
    /// 接收安全校验后的 JSON 参数，在沙箱中进行网络请求、文件操作等动作，
    /// 返回对齐 SKILL.md 中声明的 output Schema 的 JSON 结果。
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, crate::utils::errors::AiNexusError>;
    
    /// 5. 执行后置审核与复核 (Post-verification)
    /// 动作执行完毕后，检查执行结果是否达到了预期状态。
    /// 如果判定未达预期（即便 execute 没有崩溃），可返回 Error 触发系统重试或通知大脑重新规划路径。
    fn verify_result(&self, params: &serde_json::Value, result: &serde_json::Value) -> Result<(), crate::utils::errors::AiNexusError> {
        // 默认实现：无条件信任 execute 的返回值
        Ok(())
    }
}
```

---

## 4. 外部桥梁：渠道与会话契约 (Channel & Agent)

将各色各样的外部通信平台统一收口。

### 4.1 会话上下文与多模态结构 (Context & Messages)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    Text(String),
    Image { mime_type: String, data: Vec<u8> },
    Audio { mime_type: String, data: Vec<u8> },
    Document { mime_type: String, data: Vec<u8> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub contents: Vec<MessageContent>,
}

#[derive(Debug, Clone)]
pub struct AgentContext {
    /// 绑定的输入输出通道名称 (继承自 Gateway)
    pub channel_name: String,
    /// 用户在该渠道上的唯一 ID
    pub user_id: String,
    /// 该上下文挂载的智能体实例快照
    pub agent_def: AgentDef,
    /// 继承自 Persona 的系统提示词快照
    pub persona_prompt: String,
    /// 临时记忆，最近 N 轮的对话历史 (支持多模态附件)
    pub short_term_memory: Vec<ChatMessage>, 
    /// 身份特权标识，用于标明该 Context 是否归属于拥有全量 Meta Skill 权限的 Meta Agent
    pub is_meta_agent: bool,
}
```

### 4.2 渠道网关 (Channel)
Channel 负责搬运和序列化。

```rust
#[async_trait]
pub trait Channel: Send + Sync {
    /// 获取当前渠道的名字，如 "Telegram" 或 "LocalTerminal"
    fn channel_name(&self) -> &str;
    
    /// 从该渠道阻塞或异步接收下一条原始消息，并转化为多模态结构
    async fn receive_input(&self) -> Result<Vec<MessageContent>, crate::utils::errors::AiNexusError>;
    
    /// 将系统处理完的结果一次性推流回该渠道
    async fn send_reply(&self, target_user: &str, contents: Vec<MessageContent>) -> Result<(), crate::utils::errors::AiNexusError>;
    
    /// [新增] 将系统正在生成的打字机数据流式推送回该渠道 (SSE / WebSocket)
    async fn send_stream_chunk(&self, target_user: &str, chunk: &str, is_final: bool) -> Result<(), crate::utils::errors::AiNexusError> {
        Err(anyhow::anyhow!("Streaming not supported on this channel"))
    }
}
```

---
*注：关于 API 速率限制 (Rate Limiting) 和 大模型 Token 计费等逻辑，属于业务层的非功能性需求，将实现在 `Model Router` 模块的拦截器中，不污染此处核心的 `core` 契约协议。*

---

## 5. 协议层次关系说明 (Protocol Hierarchy)

> [!NOTE]
> **`AcpPayload` vs `ExecutionDsl` 的层次关系**
>
> 这两个类型在文档中并存，但属于**不同的架构层次**，不是互相替代：
>
> | 类型 | 所属层 | 职责 | 寻址方式 |
> |------|--------|------|----------|
> | `AcpPayload::ActionTrigger` | **总线层** (OS → Skill Router) | 路由信封，传递意图 | `skill_name`（人类可读名称） |
> | `ExecutionDsl::Skill` | **执行层** (Skill Router → ACS Sandbox) | 精确执行指令 | `skill_id`（UUID，防重名冲突） |
>
> **完整流转路径**：
> `GeminiCore 决策` → `AcpPayload::ActionTrigger(skill_name)` → `Skill Router 查询注册表` → `ExecutionDsl::Skill(UUID, timeout)` → `ACS 沙箱执行` → `AcpPayload::ActionResult`
