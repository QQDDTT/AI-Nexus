# LocalTerminal 渠道使用规范与设计 (LocalTerminal Channel)

在 AI-Nexus 接入复杂的 Telegram 或 LINE 之前，`LocalTerminal` 是我们最核心的本地开发与调试渠道。它允许开发者直接在命令行中与系统的 Agent 进行对话，免去了配置网络 Webhook、代理和 API Key 的繁琐过程。

## 1. 定位与应用场景

`LocalTerminal` 是实现了 `core::interfaces::Channel` 的具体结构体，存在于系统的边缘接入层。

*   **极速调试**：在开发新 Skill 或调试 OS 状态机流转时，直接在终端打字交互。
*   **安全隔离**：在本地终端运行的实例，天然不暴露公网，是最安全的沙箱测试环境。
*   **多模态模拟**：虽然终端是纯文本的，但可以通过特定指令（如 `@attach file.png`）在本地模拟多模态附件的上传。

## 2. 核心架构设计

### 2.1 结构体定义
在代码中，`LocalTerminal` 将是一个持有标准输入输出流的对象：

```rust
pub struct LocalTerminalChannel {
    user_id: String,
}

#[async_trait]
impl Channel for LocalTerminalChannel {
    fn channel_name(&self) -> &str {
        "LocalTerminal"
    }
    
    // ... 实现 receive_input 和 send_reply 等接口
}
```

### 2.2 交互流转 (Event Loop)
1.  系统启动后，实例化 `LocalTerminalChannel` 并注册进 OS 总线。
2.  主线程进入死循环，等待 `stdin` (标准输入)。
3.  用户在终端敲击回车后，Channel 读取字符串，封装为 `MessageContent::Text` 发送给 OS。
4.  OS 处理完毕，调用 Channel 的 `send_reply`，在终端打印输出。

---

## 3. 开发者使用指南

当后续我们完成了代码编写并运行 `cargo run` 后，终端将呈现如下交互模式：

### 3.1 基础对话
启动系统后，终端会出现提示符：
```bash
[AI-Nexus OS] System initialized.
[LocalTerminal] Enter your message (type /exit to quit):
> 帮我写一个 python 的 helloworld
```

### 3.2 模拟多模态附件 (SOTA 特性测试)
由于我们在 `14` 规范中加入了多模态特性，终端可以通过斜杠命令或特定前缀模拟文件上传：
```bash
> /attach ./test_image.png 看看这张图片里有什么？
```
*原理解析*：`LocalTerminalChannel` 在解析到 `/attach` 时，会在本地读取 `test_image.png` 的二进制流，并组装成 `MessageContent::Image` 提交给 OS。

### 3.3 模拟高危审批 (HITL 测试)
当调试触发了需要人类审批的高危动作时，终端会高亮提示并挂起：
```bash
[SYSTEM ALERT]: Agent is attempting to execute high-risk skill 'database_drop'.
[SYSTEM ALERT]: Do you approve? (y/N)
> n
[SYSTEM ALERT]: Execution rejected. Returning error to Agent...
```

---

## 4. 与 OS 总线的桥接计划

在即将进行的 `Phase 2` 开发中，我们将首先把 `LocalTerminal` 实例化，然后把它作为驱动力，跑通 `src/os` 中的 **[Awake] -> [Routing] -> [Executing] -> [Aggregating]** 完整闭环，从而彻底验证我们核心架构的代码可用性。
