# 企业级架构特性规范 (Enterprise Architecture Specification)

本规范用于填补 AI-Nexus 走向多用户商业化、多智能体协作、以及完整 GraphRAG 链路的最后三块底层拼图：IAM 身份管理、Agent Swarm 协同通信、以及向量化与知识图谱引擎策略。

## 1. 身份与访问管理模块 (IAM & Quota Control)

`src/iam` 模块是整个系统的“海关”。任何从 `Channel` 进来的外部消息，在唤醒对应的 `AgentContext` 前，必须通过 IAM 模块的严格审计。

### 1.1 核心职责
*   **身份鉴权 (AuthN)**：校验 Channel 传递的 Token 或 Webhook 签名的合法性。
*   **配额阻断 (Quota AuthZ)**：实时拦截欠费用户，防止大模型 API 被恶意刷量 (DDoS/CC)。

### 1.2 拦截链路
OS 模块在 `[Awake]` 阶段的最初始操作，会调用 IAM 网关：
```rust
pub mod iam {
    /// 检查用户合法性及余额
    pub async fn verify_and_deduct_quota(user_id: &str, estimated_tokens: u32) -> Result<(), IamError> {
        // 1. 从本地二进制快照或 GraphDB / HNSW 索引读取用户订阅套餐
        // 2. 如果超限，返回 Err(IamError::QuotaExceeded)
        // OS 捕获此错误后，直接向 Channel 返回“余额不足，请充值”，不再唤醒大模型。
    }
}
```

---

## 2. 多智能体群组协作 (Multi-Agent Swarm)

单打独斗的 Agent 处理日常对话很高效，但遇到复杂的系统级任务（如用户要求编写一个极其复杂的长篇代码项目），单线程的 Context 会捉襟见肘。

### 2.1 Swarm 网络/内部通道 机制
我们在 `06_OS_MODULE` 的 Event Bus 中加入特殊的 `Swarm Message` 路由。
*   **请求外包 (Delegation)**：普通 Agent 在执行某项技能时遇到瓶颈，可以通过生成一个特定的内部意图（如 `@MetaAgent: help me write a new skill`），将一段 Prompt 打包发送给内部的 `Meta Agent`。
*   **挂起等待**：此时普通 Agent 的状态机进入 `Sleeping (Wait for Swarm)` 状态。
*   **回调唤醒**：Meta Agent 完成复杂的编译与测试后，将结果作为一个新的 Event 投递回总线，OS 接收后唤醒原普通 Agent 继续执行。

此机制彻底打通了系统内部不同 Persona 之间的壁垒，形成了“产品经理 Agent”提出需求，“程序员 Agent (Meta Agent)”写代码的工厂流水线。

---

## 3. 向量化引擎 (Embedding Pipeline)

为了支撑 `GraphRAG` 知识图谱检索与多跳推理记忆，系统必须具备高效稳定的文本向量化能力与实体抽取能力。

### 3.1 混合架构策略 (Hybrid Embedding)

为了在“计算成本”和“语义精度”之间取得平衡，我们不在 `Gemini API` 上死磕，而是采用混合模式：

1.  **本地 Rust 引擎 (首选)**：
    *   **选型**：使用 Rust 本地的 `candle` 框架或 `ort` (ONNX Runtime)，加载极轻量级的开源模型，例如 `all-MiniLM-L6-v2` (产出 384 维向量)。
    *   **适用场景**：高频、短文本的处理。例如用户说的一句话“我叫Nick”，或者系统检索 `Skill Schema` 的短描述。这部分完全本地计算，0 延迟，0 计费。
2.  **云端大模型引擎 (备选)**：
    *   **选型**：Gemini 官方的 `text-embedding-004` (产出 768 维高精度向量)。
    *   **适用场景**：对于几千字的超长文章、复杂的大段代码，或者需要极高语义理解深度的任务。

### 3.2 接口抽象
```rust
pub mod embedding {
    #[async_trait]
    pub trait Embedder: Send + Sync {
        /// 将文本转换为向量
        async fn embed_text(&self, text: &str) -> Result<Vec<f32>, anyhow::Error>;
    }
    
    /// 具体实现 1：LocalOllama / CandleEmbedder
    /// 具体实现 2：GeminiApiEmbedder
}
```
