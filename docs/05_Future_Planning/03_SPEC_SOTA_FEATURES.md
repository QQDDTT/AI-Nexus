# 终极前沿特性规范 (SOTA Features Specification)

本规范代表了 AI-Nexus 作为下一代 Agent OS 的顶层追求。在这里，我们将彻底超越传统的文本问答机器人，引入多模态、实时流式交互、人类审批在环（HITL）以及高度可扩展的插件系统。

## 1. 多模态输入抽象 (Multimodal Attachments)

为了充分释放 Gemini 的多模态能力，系统的 `Channel` 不再仅仅传递字符串，而是支持富媒体附件。

### 1.1 数据结构演进
在 `AgentContext` 或底层的 `ChatMessage` 结构中，文本不再是唯一的载体：
```rust
pub enum MessageContent {
    Text(String),
    Image { mime_type: String, data: Vec<u8> },
    Audio { mime_type: String, data: Vec<u8> },
    Document { mime_type: String, data: Vec<u8> },
}

pub struct ChatMessage {
    pub role: String,
    pub contents: Vec<MessageContent>, // 允许一条消息混合文本和多张图片
}
```

### 1.2 处理链路
* **网关层 (Channel)**：自动将用户上传的文件（如 Telegram 的图片或 Web 的 PDF）下载并转为二进制流。
* **模型层 (Gemini)**：利用官方 SDK 的 `Part` 对象，将这些多模态数据原封不动地发给大模型进行视觉或语音推理。

---

## 2. 人类在环审批机制 (Human-in-the-Loop, HITL)

安全不仅依靠沙箱，更需要人类的最后把关。

### 2.1 高危技能阻断
我们为 `Skill` 引入一个新的属性或分类标志 `requires_human_approval: bool`。
当 OS 发现即将执行的技能属于高危操作时，状态机将触发 `HITL` 流程：
1. **[Thinking -> WaitingForHuman]**：Agent 陷入冻结（挂起）状态，释放内存，仅将审批流状态持久化到 `Storage` 中。
2. **通知人类**：通过特定 Channel 推送类似卡片的消息：“Agent 申请执行 `DROP DATABASE nexus`，是否允许？[Approve] / [Reject]”。
3. **唤醒 (Resume)**：收到回调后，OS 恢复该 Agent 的状态机。若是 Reject，则以系统 Error 身份塞回给大模型让其放弃。

---

## 3. 流式输出与实时网络架构 (Streaming & SSE)

天下武功，唯快不破。大模型长文本的“打字机效果”是现代 Agent 的标配。

### 3.1 架构实现
* **模型侧 (Gemini Module)**：废弃单次阻塞调用的 API，全面采用 `streamGenerateContent` 接口。返回一个 Rust `Stream` (通过 `tokio` 异步流机制)。
* **总线侧 (OS Module)**：OS 在 `[Replying]` 阶段，不再等待完整字符串，而是边收边发。
* **接入侧 (Channel)**：实现 `Server-Sent Events (SSE)` 或 `WebSocket` 接口，确保前端 UI 可以一个个字地接收渲染，彻底消灭用户的“等待空白期”。

---

## 4. 中间件与插件拦截机制 (Middleware Hooks)

为了保证核心代码的绝对纯洁，所有非核心的脏活（如：敏感词过滤、链路追踪打点、全局日志抓取）都通过中间件解耦。

### 4.1 事件总线钩子 (Event Hooks)
在 OS 的核心流转链路前后，暴露标准的注入点：
*   `OnMessageReceived` (消息入站前拦截，如触发风控)
*   `OnSkillExecuting` (技能执行前置钩子)
*   `OnSkillExecuted` (技能执行后置钩子)
*   `OnResponseStreaming` (流式返回前的文字篡改/脱敏)

开发者可以通过编写独立的 Rust 闭包或注册 `Middleware Trait`，像玩乐高一样给系统无缝挂载各种监控和拦截器，实现高度的开放生态。
