# Gemini 模块设计规范 (Gemini Core & Model Router Module Specification)

`src/gemini` 模块是系统唯一的思考中枢。它的主要任务是代理并抽象与大模型 API 的网络交互，并通过基于**算力 Profile** 的 `Model Router` 模型调度算法为 Agent 选择最优模型与容灾链路，同时保证“瘦上下文 (Thin Context)”。

## 1. 核心职责 (Core Responsibilities)

1. **Model Router (模型路由)**：根据推理任务类型 (`InferenceTaskType`)、上下文长度预估 (`estimated_tokens`) 与算力 Profile 映射表，在不同模型之间动态路由，并支持长文本溢出重定向 (`context_overflow_model`)。
2. **API 客户端封装 (HTTP Client)**：基于 `reqwest` 构建强类型的异步网络请求层，处理重试、超时与动态凭证读取。
3. **记忆上下文折叠 (Context Folding)**：配合 Agent Memory 将历史记忆根据 Token 约束进行轻量化折叠与剪裁。
4. **结构化输出与 Function Calling**：解析模型输出的 Tool Call 意图，配合 `SkillPipeline` 执行。

---

## 2. 核心架构与数据结构

```rust
pub mod router {
    use serde::{Deserialize, Serialize};

    /// 推理任务类型枚举
    #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub enum InferenceTaskType {
        Logic,
        CodeGeneration,
        Balanced,
        Fast,
        Vision,
        StructuredOutput,
        Custom(String),
    }

    /// 高级调度规则
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct RoutingRules {
        pub context_overflow_model: Option<String>,
        pub max_token_threshold: Option<usize>,
        pub timeout_ms: Option<u64>,
    }

    /// 算力 Profile 结构（关联多任务类型）
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct CapabilityProfile {
        pub name: String,
        pub description: Option<String>,
        pub task_types: Vec<String>,
        pub primary: String,
        pub failover: Vec<String>,
        pub routing_rules: Option<RoutingRules>,
    }

    /// 强类型路由异常
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum ModelRouterError {
        NoMatchingRoute { task_type: String },
        InvalidProfile { profile_key: String, reason: String },
    }

    /// 路由解析结果
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub struct RouteResult {
        pub profile_key: String,
        pub primary: String,
        pub failover: Vec<String>,
        pub routing_rules: Option<RoutingRules>,
        pub is_context_overflow: bool,
    }

    pub struct ModelRouter { ... }

    impl ModelRouter {
        /// 严格路由解析：无静默保底，失败抛出 ModelRouterError
        pub fn route_task(
            &self,
            task: &InferenceTaskType,
            estimated_tokens: Option<usize>,
            config: Option<&serde_json::Value>,
        ) -> Result<RouteResult, ModelRouterError>;

        pub fn select_best_model(
            &self,
            task: &InferenceTaskType,
            estimated_tokens: Option<usize>,
            config: Option<&serde_json::Value>,
        ) -> Result<String, ModelRouterError>;
    }
}

pub mod client {
    pub struct GeminiClient {
        api_key: String,
        base_url: String,
        http_client: reqwest::Client,
    }

    impl GeminiClient {
        pub fn new(api_key: String) -> Self;
        pub async fn generate_content(
            &self,
            model: &str,
            request: &GenerateRequest,
        ) -> Result<GenerateResponse, AiNexusError>;
    }
}
```

---

## 3. 容错与重试机制 (Resilience)

1. **指数退避重试**：遇到 HTTP 429 或网络瞬态错误时，客户端自动尝试最多 3 次退避重试。
2. **严谨无保底**：当路由表无法解构出目标 `task_type` 对应的算力 Profile 时，抛出 `ModelRouterError::NoMatchingRoute` 供上层或者 REST API 捕获，拒绝硬编码隐式误用模型。
