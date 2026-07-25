# 高级系统特性规范 (Advanced OS Features Specification)

本规范用于补全 AI-Nexus 作为“工业级 Agent OS”的最后三块核心进阶拼图：动态提示词模板、自我纠错反思循环、以及后台异步任务调度。

## 1. 动态模板引擎 (Prompt Template Engine)

在 `Agent` 和 `Gemini` 模块交互之间，系统不仅要发送死板的 System Prompt，还必须注入实时环境上下文。

### 1.1 架构设计
*   **选型**：引入 Rust 的 `Askama` 或 `Tera` 作为渲染引擎。
*   **注入变量 (Context Variables)**：
    *   `{{ current_time }}`：当前主机的精确时间。
    *   `{{ available_skills }}`：该 Agent 当前权限下可调用的 Skill Schema 集合。
    *   `{{ memory_highlights }}`：通过 GraphRAG 社区摘要与联想记忆网络（激活扩散模型）从知识库中跨层级召回的上下文实体与多跳逻辑。
    *   `{{ persona }}`：该分身设定的性格基调（如“傲娇”、“专业”）。

### 1.2 处理流程
每次 OS 唤醒 Agent 准备推理前，`AgentContext` 会首先通过模板引擎将上述变量编译为最终的纯文本 System Prompt，然后再丢入 `gemini` 模块计算。

---

## 2. 自我纠错与反思机制 (Self-Reflection Loop)

在大模型调用 `Skill` 进行 `Acting` 时，难免会出现参数幻觉或沙箱崩溃。系统**禁止**直接向用户抛出异常，而是必须进行“自我反思”。

### 2.1 故障回传注入 (Error Injection)
当 `skill` 模块的 WebAssembly 沙箱执行报错，或原生工具返回 `Result::Err` 时：
1.  OS 不会结束当前状态机，而是将错误堆栈（Stack Trace）或标准错误输出（stderr）包装为系统级对话。
2.  格式示例：`[System Error]: 技能执行失败。错误原因：缺少必填参数 'file_path'。请修正后重试。`
3.  OS 强行将该条 Error 消息推入 `AgentContext` 的短期记忆中。

### 2.2 ReAct 闭环重试
由于状态机中存在从 `Acting -> Thinking` 的回流箭头，Gemini 在下一轮推理时会“看到”刚才自己的错误，从而自动生成修正后的工具调用指令，直到调用成功或达到最大重试次数上限（如 3 次）才会彻底放弃并告知用户。

---

## 3. 任务调度模块 (Scheduler Module)

为了让 AI-Nexus 突破“一问一答”的聊天机器人局限，我们新增 `src/scheduler` 模块，赋予 Agent 守护进程 (Daemon) 的能力。

### 3.1 核心职责
*   **Cron 定时任务**：支持类似 `0 * * * *` 的时间轮，让 Agent 能够“每天早上 8 点主动发新闻”。
*   **Detached 后台长挂起任务**：当用户要求“分析这 10 个 G 的日志并在完成后通知我”，Agent 可以在沙箱后台派生出常驻进程。

### 3.2 架构实现
```rust
pub mod scheduler {
    use tokio::time::Interval;

    /// 全局任务调度中心
    pub struct JobScheduler {
        // ...
    }

    impl JobScheduler {
        /// 注册一个基于 Cron 表达式的唤醒任务
        pub async fn schedule_cron(&self, cron_expr: &str, action_intent: &str, agent_id: &str) -> Result<(), anyhow::Error> {
            // 当时间到达时，构造一个虚拟的 UserInput，注入到 OS 的 Event Bus 中
        }
        
        /// 将一个耗时极长的 Skill 转移到后台执行
        pub fn spawn_detached_job(&self, skill_id: &str, params: serde_json::Value) -> JobHandle {
            // 在独立的 tokio worker 中执行
        }
    }
}
```

### 3.3 与 OS 总线的桥接
当时间触发器 (Cron) 激活时，`scheduler` 模块会伪装成一个虚拟的 `Channel`，向 `os` 的核心总线发送一条“唤醒消息”。此后，整个状态流转将完全复用常规对话的逻辑。
