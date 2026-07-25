# OS 模块设计规范 (Nexus OS Module Specification)

`src/os` 模块是整个 AI-Nexus 的中枢神经系统（系统总线）。它不仅是外部 `Channel` 数据进入系统的第一道防线，更是所有内部子模块（Gemini Core, Skill Engine）之间进行消息路由与生命周期管理的协调者。

## 1. 核心职责 (Core Responsibilities)

1.  **渠道网关 (Channel Gateway)**：监听并统一解析来自各大平台（Telegram, LINE 等）的外部输入事件，识别进线网关配置 (`GatewayDef`)。
2.  **身份与权限校验 (RBAC & Auth)**：拦截非法请求。为合法请求通过 Gateway 绑定的 `bound_persona_id`，去提取 `PersonaDef` 并以此实例化动态的 `AgentContext`（心智分身设定）。
3.  **ACP 消息总线 (Message Bus)**：基于 `tokio::mpsc` 建立高并发的内部通信通道，负责在 `Model Router` 和 `Skill Router` 之间流转 `AcpMessage`。
4.  **会话状态机 (Session State Machine)**：追踪每一次对话的生命周期，处理超时、中断和重试机制。

## 2. 关键设计与架构 (Key Designs)

### 2.1 高并发事件循环 (Event Loop)

为保证极低的延迟，`os` 模块采用 Actor 模型思想。主控进程 (Nexus OS Daemon) 启动后，会孵化出多个基于 `tokio::spawn` 的轻量级异步任务：

*   **Listener Worker**：专门负责轮询/监听 Webhook 或 Long Polling 的网络请求。
*   **Context Builder**：依据请求来源对应的 `GatewayDef` 找出绑定的 `PersonaDef`，再结合用户的权限等级 (Permission Level) 和最近的记忆记录 (Short-term Memory)，生成完整的强类型 `AgentContext` (其内部包含 `AgentDef`)。
*   **Dispatcher Worker**：处理核心流转逻辑，通过内部通道将上下文包装为 `InferenceRequest` 转发给 `Model Router`。

### 2.2 状态机演进图 (State Machine Transition)

每一次用户对话在 `os` 中都经历如下标准生命周期：

<div style="font-family: sans-serif; text-align: center; color: #333; line-height: 1.5; padding: 20px; border: 1px solid #ddd; border-radius: 8px; background-color: #f9f9f9; max-width: 700px; margin: 0 auto;">
    <!-- Stage 1 -->
    <div style="display: flex; justify-content: center; margin-bottom: 15px;">
        <div style="padding: 10px 20px; background-color: #e5e7eb; border: 1px solid #9ca3af; border-radius: 6px; width: 40%;">
            <strong>💤 Idle (闲置)</strong><br>
            <span style="font-size: 0.8em;">监听信道</span>
        </div>
    </div>
    <div style="font-size: 0.9em; color: #555;">⬇️ 收到输入 ⬇️</div>
    <!-- Stage 2 -->
    <div style="display: flex; justify-content: center; gap: 15px; margin: 15px 0;">
        <div style="padding: 10px; background-color: #fca5a5; border: 1px solid #ef4444; border-radius: 6px; width: 30%;">
            <strong>❌ Rejected</strong><br>
            <span style="font-size: 0.8em;">权限不足/账号冻结</span>
        </div>
        <div style="padding: 10px; background-color: #fef08a; border: 1px solid #facc15; border-radius: 6px; width: 40%;">
            <strong>🔐 Authenticating</strong><br>
            <span style="font-size: 0.8em;">身份与权限校验</span>
        </div>
    </div>
    <div style="display: flex; justify-content: center; gap: 15px; font-size: 0.9em; color: #555;">
        <div style="width: 30%;"></div>
        <div style="width: 40%;">⬇️ 权限通过 ⬇️</div>
    </div>
    <!-- Stage 3 -->
    <div style="display: flex; justify-content: center; margin: 15px 0;">
        <div style="padding: 10px 20px; background-color: #bfdbfe; border: 1px solid #60a5fa; border-radius: 6px; width: 50%;">
            <strong>📦 Assembling</strong><br>
            <span style="font-size: 0.8em;">查 Gateway -> 取 Persona -> 生成 AgentContext</span>
        </div>
    </div>
    <div style="font-size: 0.9em; color: #555;">⬇️ 分配算力 (路由至 Gemini) ⬇️</div>
    <!-- Stage 4 -->
    <div style="display: flex; justify-content: space-between; gap: 10px; margin: 15px 0;">
        <div style="padding: 10px; background-color: #bbf7d0; border: 1px solid #4ade80; border-radius: 6px; flex: 1; display: flex; flex-direction: column; justify-content: center;">
            <strong>🧠 Thinking</strong><br>
            <span style="font-size: 0.8em;">产生意图 / 决定回复</span>
        </div>
        <!-- ReAct Loop Indicator -->
        <div style="display: flex; flex-direction: column; justify-content: center; font-size: 1.2em;">
            ➡️<br>
            ⬅️
        </div>
        <div style="padding: 10px; background-color: #fbcfe8; border: 1px solid #f472b6; border-radius: 6px; flex: 1; display: flex; flex-direction: column; justify-content: center;">
            <strong>⚙️ Acting</strong><br>
            <span style="font-size: 0.8em;">沙箱执行技能<br>(可多次循环回 Thinking)</span>
        </div>
        <div style="padding: 10px; background-color: #fed7aa; border: 1px solid #fb923c; border-radius: 6px; flex: 1;">
            <strong>⏱️ Timeout</strong><br>
            <span style="font-size: 0.8em;">响应超时 / 执行挂死</span>
        </div>
    </div>
    <div style="font-size: 0.9em; color: #555;">⬇️ 最终决策完毕 (Final Answer) ⬇️</div>
    <!-- Stage 5 -->
    <div style="display: flex; justify-content: center; margin: 15px 0;">
        <div style="padding: 10px 20px; background-color: #ddd6fe; border: 1px solid #a78bfa; border-radius: 6px; width: 60%;">
            <strong>💬 Replying</strong><br>
            <span style="font-size: 0.8em;">发送回复 (或兜底失败消息) 并更新记忆</span>
        </div>
    </div>
    <div style="font-size: 0.9em; color: #555;">⤴️ 回归 Idle 状态 ⤴️</div>
</div>

### 2.3 核心启动流程 (Startup Sequence)

`OS` 模块作为 AI-Nexus 的中枢，负责在进程启动时协调各个子系统的有序拉起。标准的 `main.rs` 初始化过程如下：

1. **环境与日志准备**：初始化安全脱敏的追踪日志 (`tracing-subscriber`)。
2. **配置加载**：通过 `utils::config` 解析 `ainexus.yaml` 并热加载。
3. **OS 存储初始化 (Storage Init)**：调用 `os::init_os_storage()`，由系统初始化并挂载底层的 Block-L/Block-S 原生二进制文件引擎。我们采用**单体多模块架构**，存储层与 OS 层共用同一物理进程，彻底摒弃 SQLite 等传统关系型数据库，实现基于 `postcard` 的高性能日志追加操作。
4. **引擎实例化**：并行拉起身份矩阵 (`AgentInstance`)、大模型路由池 (`ModelRouter`) 以及技能沙盒 (`WasmSandbox`)。
5. **总线连接与网关启动**：创建 `NexusBus`，孵化 `SessionManager` 和垃圾回收守护进程 (`Reaper`)，最后启动外部 Channel 网关 (如 Telegram) 监听和本地的前后端 API 服务器。

## 3. 核心接口与数据结构

```rust
pub mod bus {
    use crate::core::protocol::AcpMessage;
    use tokio::sync::mpsc;

    /// 内部核心事件总线
    pub struct NexusBus {
        sender: mpsc::Sender<AcpMessage>,
        receiver: mpsc::Receiver<AcpMessage>,
    }

    impl NexusBus {
        /// 初始化总线，设定通道容量防止背压 (Backpressure)
        pub fn new(capacity: usize) -> Self { /* ... */ }
        
        /// 分发消息至对应的 Component
        pub async fn route(&self, msg: AcpMessage) -> Result<(), anyhow::Error> { /* ... */ }
    }
}

pub mod session {
    /// 追踪一次对话完整生命周期的管理器
    pub struct SessionManager {
        // ... 维护活跃会话的内存哈希表
    }
}
```

## 4. 安全防护策略

1.  **限流阀 (Rate Limiter)**：在 `Context Builder` 阶段引入令牌桶算法 (Token Bucket)，针对单独的 `user_id` 和 `channel_name` 限制每分钟的发起次数，防止恶意滥用耗尽 Gemini API 额度。
2.  **硬超时中断 (Hard Timeout)**：对处于 `Thinking` 或 `Acting` 状态超过阈值 (如 30 秒) 的会话强行注入 Cancel 信号，并释放对应的 `AgentContext` 资源。
