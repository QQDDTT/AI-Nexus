# Agent 模块设计规范 (Agent & Memory Module Specification)

`src/agent` 模块是 AI-Nexus 系统的“灵魂容器”。在我们的架构中，Channel（如 Telegram）仅仅是物理通道，真正具有心智、记忆、权限的实体是 **Agent（心智分身）**。

## 1. 核心职责 (Core Responsibilities)

1.  **心智设定 (Persona Management)**：根据外部传入的 `user_id` 或特定配置，加载并动态组装大模型的底层 System Prompt（例如：冷酷模式、专业助理模式）。
2.  **长短期记忆 (Memory Management)**：
    *   **短期记忆 (Short-term Memory)**：维护当前会话的上下文窗口（滑动窗口机制），并在到达上限时进行无损截断或总结。
    *   **长期记忆 (Long-term Memory)**：将用户的关键信息提取并持久化，基于 GraphRAG 构建跨会话的联想记忆网络，实现类人脑的情景记忆与语义记忆。
3.  **长期目标追踪 (Goal Tracking)**：赋予 Agent 在没有用户输入时，自发执行后台任务或定时唤醒的能力（例如后台监控股票并主动推送）。

## 2. 关键设计与架构 (Key Designs)

### 2.1 记忆折叠算法 (Memory Folding)
为贯彻“瘦上下文 (Thin Context)”理念，Agent 的记忆不能无脑堆砌。
*   **摘要机制**：当单轮会话长度突破 4000 Token，Agent 模块会在后台触发一个廉价模型（如 Gemini Flash Lite），将历史对话总结为几句核心要点，替换掉原始的冗长记录。
*   **实体抽取**：对话过程中，Agent 模块会随时嗅探诸如“我讨厌吃香菜”、“我明天上午 9 点要开会”等实体信息，存入外挂的长期数据库中。

### 2.2 多重分身架构 (Multi-Agent Multiplexing)
`Nexus OS` 每接收到一个新的 Channel 请求，都会在内存中映射出一个对应的 `AgentContext`。
*   这意味着系统支持**多租户/多分身**。同一个物理主机上，Telegram 进来的消息由“二次元性格的 Agent A”处理，而从 CLI 进来的消息由“严肃程序员 Agent B”处理，互不干扰。

### 2.3 顶级分身：Meta Agent (全局造物主)
系统内部除了常规的用户映射分身，还存在唯一一个脱离外部 Channel 独立存在的超级实体——**Meta Agent**。
*   **唯一特权**：Meta Agent 是全系统唯一被授权挂载并使用五大 `Meta Skill` 的实体（包含描述规则、读取、新建/修改、验证技能代码、以及生成和装配新 Agent 分身）。
*   **进化职责**：它不直接服务于普通人类日常聊天。它的主要目标是根据系统的需要，自动调用 Meta Skills 编写、审查并向系统中发布新的能力模块（Skill）或装配全新的任务分身（Agent），是系统实现自我迭代与扩展的核心驱动力。

### 2.4 技能挂载与动态编排 (Skill Mounting & Dynamic Orchestration)
为了确保 Agent 具备实际的物理执行力而非纯粹的对话引擎，系统强制规定：**每个 Agent 实例至少包含一个可执行 Skill**。
*   **动作能力基础**：Agent 的能力边界由其白名单内的技能决定。
*   **工作流自适应编排**：在执行多步任务的过程中，Agent 可以根据当前的上下文和上一个技能的执行结果，自主决策并动态编排多个 Skill 的调用顺序与逻辑关联，从而形成灵活的工作流（Workflow）以达成最终目标。

## 3. 核心接口与数据结构

```rust
pub mod persona {
    use crate::core::interfaces::PersonaDef;

    /// Persona 现在已经归纳为核心领域模型 PersonaDef
    /// 包含 base_prompt, allowed_skills (白名单) 等静态配置。
}

pub mod memory {
    use crate::core::protocol::ChatMessage;

    /// 记忆容器抽象
    pub trait MemoryStore: Send + Sync {
        /// 将新对话压入短期滑动窗口
        /// 注意：ChatMessage 包含 Vec<MessageContent>，支持文本、图片、音频等多模态附件的混合存储
        fn push_short_term(&mut self, msg: ChatMessage);
        
        /// 获取截断、折叠后的瘦上下文，用于提交给大模型
        fn get_folded_context(&self, max_tokens: usize) -> Vec<ChatMessage>;
        
        /// 将重要信息持久化到向量记忆库
        async fn save_long_term(&self, key_fact: &str) -> Result<(), crate::utils::errors::AiNexusError>;
    }
}

pub mod instance {
    use crate::core::interfaces::AgentDef;
    use super::memory::MemoryStore;

    /// 内存中存活的 Agent 实例实体
    pub struct AgentInstance {
        pub agent_def: AgentDef, // 包含 agent_id, persona_id, capability_requirement
        pub owner_id: String,    // 归属的用户 ID
        pub memory: Box<dyn MemoryStore>,
    }
    
    impl AgentInstance {
        /// 根据能力要求与 Model Router 动态解析模型，匹配失败抛出 ModelRouterError
        pub fn resolve_target_model(
            &self,
            routing_map: Option<&serde_json::Value>,
        ) -> Result<String, crate::gemini::router::ModelRouterError>;

        /// 根据当前记忆和设定，生成最终给 Gemini Core 的标准请求
        pub async fn build_inference_request(&self, user_input: &str, max_tokens: usize) -> GenerateRequest;
    }
}
```

## 4. 与 OS 及 Gemini 模块的交互关系

*   **上游 (OS)**：`os` 模块收到信息后，只负责唤醒或拉取对应的 `AgentInstance`。
*   **中游 (Agent 本身)**：`AgentInstance` 把记忆、人设、用户输入打包在一起。
*   **下游 (Gemini)**：`AgentInstance` 将打包好的干净数据通过 ACP 协议发送给 `gemini` 模块进行纯粹的智能推理。
