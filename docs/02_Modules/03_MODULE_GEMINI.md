# Gemini 模块设计规范 (Gemini Core & Model Router Module Specification)

`src/gemini` 模块是系统唯一的思考中枢（大脑）。它的主要任务是代理并抽象与 Google Gemini 官方 API 的所有网络交互，并通过 `Model Router`（模型调度算法）为 Agent 节省额度开销，并保证“瘦上下文（Thin Context）”。

## 1. 核心职责 (Core Responsibilities)

1.  **Model Router (模型路由)**：根据任务上下文的长度和类型，在多个不同版本的 Gemini 模型（例如 gemini-1.5-flash, gemini-1.5-pro, gemini-1.5-flash-8b）之间进行自动降级或升级路由。
2.  **API 客户端封装 (HTTP/gRPC Client)**：基于 `reqwest` 构建强类型的异步网络请求层，处理官方 API 的重试、认证与断路器机制。
3.  **Prompt 组装与修剪 (Prompt Engineering)**：将 `os` 模块传来的 `AgentContext`（含系统人设和历史记忆），动态拼装并进行精准截断，确保严格遵循瘦上下文约束。
4.  **结构化输出解析 (Structured Output)**：强制模型输出强类型的 JSON 意图（Function Calling / Tool Use），供后续分发给 `Skill Router`。

## 2. 关键设计与架构 (Key Designs)

### 2.1 Model Router 混合调度策略
`Model Router` 在收到推理请求后，遵循基于能力标签 (Capability Tiers) 的动态映射：

*   **T0 - 意图分类/超简单闲聊**：首选 `gemini-1.5-flash-8b`。返回极速，开销极低。
*   **T1 - 常见逻辑/多工具调度**：当发现涉及了技能搜索，首选 `gemini-1.5-flash`。
*   **T2 - 复杂系统级任务**：仅当 `Meta Agent` 需要调用五大 `Meta Skill` 执行编写或验证系统技能库等高度复杂的高级任务时，才路由至 `gemini-1.5-pro`。

### 2.2 Token 计数器与“瘦上下文”修剪 (Context Trimming)

为了避免无效的 Token 爆炸，在正式发起网络请求前，模块必须经过 `ContextTrimmer`：
1.  **静态计费**：调用本地计算器（如基于 `tiktoken` 或官方 API 提供的 CountTokens 接口预估）估算当前 `AgentContext` 的长度。
2.  **遗忘算法**：如果超过预设的 `local_model_token_limit` (例如 4096)，优先抛弃最古老的对话历史 (FIFO)。
3.  **核心保留**：System Prompt 和 `SkillRegistry` 召回的 Top-K 工具 Schema 属于绝对只读核心，**不参与遗忘**。

## 3. 核心接口与数据结构

```rust
pub mod router {
    /// 模型选择策略
    pub enum RoutingStrategy {
        Tier1Logic,
        Tier2Balanced,
        Tier3Fast,
        MultimodalVision,
    }

    /// 核心的模型路由器
    pub struct ModelRouter {
        // 配置表与配额追踪器
    }

    impl ModelRouter {
        pub async fn select_best_model(&self, strategy: RoutingStrategy, estimated_tokens: usize) -> String {
            // 根据实时 Quota 和策略，返回最应该使用的模型名称
        }
    }
}

pub mod client {
    use crate::core::protocol::AcpPayload;
    
    /// Gemini API 客户端
    pub struct GeminiClient {
        api_key: String,
        base_url: String,
        http_client: reqwest::Client,
    }

    impl GeminiClient {
        /// 执行带有 Function Calling 的生成请求
        pub async fn generate_content(
            &self, 
            provider_id: &str, 
            model: &str, 
            system_instruction: &str, 
            messages: &[Message], 
            tools: &[serde_json::Value]
        ) -> Result<GenerateResponse, anyhow::Error> { /* ... */ }
    }
}
```

## 4. 容错与重试机制 (Resilience)

1.  **指数退避 (Exponential Backoff)**：遇到 HTTP 429 (Too Many Requests) 或 503 时，API Client 自动采用指数退避算法进行至多 3 次重试。
2.  **降级熔断**：当 `gemini-1.5-flash` 持续抛出错误时，Model Router 临时将流量切分至上一代模型作为平滑过渡。
