use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::utils::errors::AiNexusError;

/// 定义了 AI-Nexus 内部所有的核心架构组件
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Component {
    /// NexusOS 主循环或事件总线
    NexusOS,
    /// 模型路由网关，负责将请求分发给不同的大模型供应商
    ModelRouter,
    /// Gemini 中心枢纽
    GeminiCore,
    /// 技能路由器，负责解析意图并分发给具体技能
    SkillRouter,
    /// 具体的技能执行引擎实例（包含技能名称）
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

fn default_gateway_status() -> String {
    "Idle".to_string()
}

/// 物理接入渠道网关
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayDef {
    pub id: String,
    #[serde(alias = "type", default)]
    pub gateway_type: Option<String>,
    #[serde(alias = "bound_persona", default)]
    pub bound_persona_id: String,
    #[serde(default = "default_gateway_status")]
    pub status: String,
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Agent 通信协议 (ACP) 的载荷定义，涵盖了所有节点间交互的事件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AcpPayload {
    /// 发起大模型推理请求
    InferenceRequest { 
        prompt: String, 
        target_model: String 
    },
    /// 触发具体的某个原子技能
    ActionTrigger { 
        skill_name: String, 
        parameters: serde_json::Value 
    },
    /// 技能或推断执行的结果回调
    ActionResult { 
        success: bool,
        status_code: u16, 
        data: serde_json::Value,
        error: Option<String>,
    },
    /// Swarm 集群内不同 Agent 之间的任务委托
    SwarmDelegation {
        target_agent: String,
        task_description: String,
    },
    /// 大模型流式输出的数据块
    StreamingChunk {
        chunk_data: String,
        is_final: bool,
    },
    /// 用于组件之间的探活与心跳检测
    Ping,
}

/// 标准的 Agent 通信协议 (ACP) 消息体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcpMessage {
    /// 全局唯一追踪 ID，用于跨组件的分布式追踪
    pub trace_id: String,       
    /// 消息发送方组件
    pub source: Component,        
    /// 消息接收方组件
    pub target: Component,        
    /// 消息产生的时间戳（Unix epoch）
    pub timestamp: u64,           
    /// 具体的消息数据载荷
    pub payload: AcpPayload,      
}

/// 领域特定语言 (DSL)，用于定义和持久化待执行的工作流或技能指令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionDsl {
    /// 定义一次需要被沙箱执行的技能任务
    Skill {
        /// 技能的唯一标识符
        skill_id: String,
        /// 技能任务的可读标题
        title: String,
        /// 传递给技能执行的 JSON 参数
        input: serde_json::Value,
        /// 技能执行的超时时间 (毫秒)
        timeout_ms: u64,
    },
}

/// 定义技能或工作流执行后的统一结果出口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOutcome {
    /// 任务是否成功执行
    pub success: bool,
    /// 状态码，类似于 HTTP status code (如 200 表示成功)
    pub status_code: u16,
    /// 执行成功后返回的 JSON 结果数据
    pub data: serde_json::Value,
    /// 如果失败，包含的错误信息
    pub error: Option<String>,
}

/// 提供文本转向量 (Embedding) 能力的抽象接口
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// 将自然语言文本转化为高维向量数组
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>, AiNexusError>;
}

/// 向量数据库查询返回的单条搜索结果
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// 对应存储记录的唯一标识符
    pub id: String,
    /// 余弦相似度或相关性打分
    pub score: f32,
    /// 附带存储的序列化载荷 (如 JSON 字节流)
    pub payload: Vec<u8>,
}

/// GraphRAG 混合检索基座抽象：HNSW 语义相似度 + 图谱拓扑关联协同工作
/// 向量侧负责语义接近性搜索；图谱侧（GraphStorage in storage::graph）负责多跳联想扩散。

// ─────────────────────────────────────────────
// 知识图谱模型
// ─────────────────────────────────────────────

/// 知识图谱节点类型
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeLabel {
    Skill,
    Concept,
    Entity,
    Memory,
    Community, // GraphRAG 社区摘要节点
}

/// 图谱节点数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// 全局唯一业务 ID（如 skill_id、entity_name 等）
    pub id: String,
    /// 节点类型标签
    pub label: NodeLabel,
    /// postcard 序列化的附加属性（由调用方决定内容）
    pub properties: Vec<u8>,
    /// 对应的语义向量锚点（与 VectorStore 共享 ID 联动）
    pub embedding: Vec<f32>,
}

/// 图谱有向边数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// 关系类型，如 "relates_to", "triggers", "belongs_to", "contradicts"
    pub relation: String,
    /// 边权重（激活扩散传播时使用，0.0~1.0）
    pub weight: f32,
}

/// 激活扩散查询结果
#[derive(Debug, Clone)]
pub struct ActivatedNode {
    pub node: GraphNode,
    /// 激活强度（越近的节点越高）
    pub activation: f32,
}

/// 知识图谱存储接口
/// 负责实体关系网络的持久化、多跳推理与激活扩散联想记忆
#[async_trait]
pub trait GraphStorage: Send + Sync {
    /// 插入或更新一个实体节点（幂等）
    async fn upsert_node(&self, node: GraphNode) -> Result<(), AiNexusError>;

    /// 在两个节点之间插入或更新有向边（幂等）
    async fn upsert_edge(&self, from_id: &str, to_id: &str, edge: GraphEdge) -> Result<(), AiNexusError>;

    /// 激活扩散查询 (Spreading Activation)：
    /// 从 seed_ids 出发，沿图边扩散 depth 层，返回激活值最高的 top_k 个节点。
    /// 激活值随跳数和边权重衰减：activation *= weight * decay_factor
    async fn spreading_activation(
        &self,
        seed_ids: &[String],
        top_k: usize,
        depth: usize,
    ) -> Result<Vec<ActivatedNode>, AiNexusError>;

    /// 获取直接相邻节点（一跳邻居）
    async fn neighbors(&self, node_id: &str) -> Result<Vec<GraphNode>, AiNexusError>;

    /// 按标签类型获取所有节点
    async fn get_nodes_by_label(&self, label: &NodeLabel) -> Result<Vec<GraphNode>, AiNexusError>;
}

#[async_trait]
pub trait VectorStore: Send + Sync {
    /// 插入或更新指定集合中的向量及其关联负载（同时应在 GraphStorage 中同步实体节点锚点）
    async fn upsert(&self, collection: &str, id: &str, vector: Vec<f32>, payload: Vec<u8>) -> Result<(), AiNexusError>;
    /// 在指定集合中通过目标向量查询最相似的 top_k 条记录（可作为激活扩散的种子节点 ID 来源）
    async fn search(&self, collection: &str, query_vector: &[f32], top_k: usize) -> Result<Vec<SearchResult>, AiNexusError>;
}

/// AI-Nexus 的原子能力单元抽象。所有的 WASM 沙箱技能或其他原生技能都应实现该接口
#[async_trait]
pub trait Skill: Send + Sync {
    /// 获取技能的全局唯一名称
    fn name(&self) -> &str;
    /// 获取该技能支持的 JSON Schema 参数定义（用于注入给大模型做 Function Calling）
    fn schema(&self) -> serde_json::Value;
    /// 标识该技能（如删库、转账等）执行前是否需要人工拦截审批
    fn requires_human_approval(&self) -> bool { false }
    /// 对大模型产生的参数进行验证和修复
    fn validate_and_repair(&self, params: serde_json::Value) -> Result<serde_json::Value, AiNexusError> { Ok(params) }
    /// 核心逻辑执行入口，由 Agent 唤起
    async fn execute(&self, params: serde_json::Value) -> Result<serde_json::Value, AiNexusError>;
    /// 验证执行结果是否符合预期，防止技能崩溃或脏数据
    fn verify_result(&self, _params: &serde_json::Value, _result: &serde_json::Value) -> Result<(), AiNexusError> { Ok(()) }
}

/// 管理和检索所有已加载技能的注册中心抽象
#[async_trait]
pub trait SkillRegistry: Send + Sync {
    /// 将一个技能实例注册到全局或上下文作用域内
    async fn register_skill(&self, skill: Box<dyn Skill>) -> Result<(), AiNexusError>;
    /// 根据用户的自然语言意图，通过语义检索选出最匹配的几个技能候选
    async fn retrieve_relevant_skills(&self, intent: &str, limit: usize) -> Result<Vec<Box<dyn Skill>>, AiNexusError>;
    /// 根据名称获取特定的技能实例
    async fn get_skill(&self, name: &str) -> Option<Box<dyn Skill>>;
    /// 获取当前注册的所有技能
    async fn list_all_skills(&self) -> Vec<Box<dyn Skill>>;
}

/// 多模态消息载荷定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageContent {
    /// 纯文本输入
    Text(String),
    /// 图片文件 (带有 MIME 类型与二进制数据)
    Image { mime_type: String, data: Vec<u8> },
    /// 音频文件 (带有 MIME 类型与二进制数据)
    Audio { mime_type: String, data: Vec<u8> },
    /// 文档文件 (如 PDF，带有 MIME 类型与二进制数据)
    Document { mime_type: String, data: Vec<u8> },
}

/// 用户或 Agent 发送的聊天消息包装器，用于记忆上下文
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// 角色 (如 "user", "model", "system")
    pub role: String,
    /// 包含的多模态内容列表
    pub contents: Vec<MessageContent>,
}

/// Agent 运行时的全生命周期上下文容器
#[derive(Debug, Clone)]
pub struct AgentContext {
    /// 绑定的输入输出通道名称 (继承自 Gateway)
    pub channel_name: String,
    /// 正在交互的用户标识
    pub user_id: String,
    /// 该上下文挂载的智能体实例快照
    pub agent_def: AgentDef,
    /// 继承自 Persona 的系统提示词快照
    pub persona_prompt: String,
    /// 滑动窗口式的短期对话记忆列表
    pub short_term_memory: Vec<ChatMessage>, 
    /// 标识该 Agent 是否是具备代码生成或环境操作最高权限的元生命 (Meta Agent)
    pub is_meta_agent: bool,
}

/// 负责将系统接入外部终端、聊天软件或 API 网关的 IO 通道抽象
#[async_trait]
pub trait Channel: Send + Sync {
    /// 获取通道的名称
    fn channel_name(&self) -> &str;
    /// 异步挂起等待，直到接收到用户的多模态输入
    async fn receive_input(&self) -> Result<Vec<MessageContent>, AiNexusError>;
    /// 向指定用户发送最终的回复
    async fn send_reply(&self, target_user: &str, contents: Vec<MessageContent>) -> Result<(), AiNexusError>;
    /// 支持流式输出时的 Chunk 投递方法
    async fn send_stream_chunk(&self, _target_user: &str, _chunk: &str, _is_final: bool) -> Result<(), AiNexusError> {
        Err(AiNexusError::General("Streaming not supported on this channel".to_string()))
    }
}

/// Agent 的人格面具与能力限制设定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    /// 基础角色设定提示词
    pub base_prompt: String,
    /// 该人设允许调用的安全技能白名单，留空表示全能
    pub allowed_skills: Vec<String>,
    /// 对话风格 (如: 严肃、傲娇、专业)
    pub tone: String,
}

/// 会话短期记忆的存储引擎抽象
#[async_trait]
pub trait MemoryStore: Send + Sync {
    /// 将新的消息压入短期记忆队列
    fn push_short_term(&mut self, msg: ChatMessage);
    /// 获取根据 Token 上限折叠/修剪后的对话上下文
    fn get_folded_context(&self, max_tokens: usize) -> Vec<ChatMessage>;
    /// 将关键事实或归档知识沉淀为长期记忆（写入 GraphDB 节点并同步向量锚点）
    async fn save_long_term(&self, entity_id: &str, entity_type: &str, content: &str) -> Result<(), AiNexusError>;
    /// 联想记忆召回：从 seed_entity_ids 出发，通过激活扩散 (depth 层) 返回 top_k 个关联记忆节点的内容摘要
    async fn recall_associative(
        &self,
        seed_entity_ids: &[String],
        top_k: usize,
        depth: usize,
    ) -> Result<Vec<String>, AiNexusError>;
}
/// 定时任务与事件触发器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerDef {
    pub id: String,
    #[serde(alias = "type")]
    pub trigger_type: String,
    pub source: String,
    pub status: String,
    #[serde(alias = "targetAgent")]
    pub target_agent: String,
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 会话上下文与统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDef {
    pub session_id: String,
    pub source: String,
    pub model: String,
    pub tokens: usize,
    pub status: String,
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// 仪表盘与系统全局设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingsDef {
    pub db_path: Option<String>,
    pub session_timeout_ms: Option<u64>,
    pub log_masking: Option<bool>,
    pub admin_username: Option<String>,
    pub admin_email: Option<String>,
    pub avatar_base64: Option<String>,
    pub theme: Option<String>,
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}
