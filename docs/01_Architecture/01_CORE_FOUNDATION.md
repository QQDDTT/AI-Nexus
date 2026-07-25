# Core Foundation (核心愿景与系统基础)

# AI-Nexus 核心设计与愿景 (Base Design)

## 1. 核心愿景 (Core Vision)

AI-Nexus 的目标是打造一个全面适配 Gemini API 的专属个人数字助手。
不同于传统的基于本地模型推理的平台（如旧版 Aura），AI-Nexus：
- **摒弃高昂本地性能开销**：完全依赖云端 Gemini 大模型的强劲算力，不占用本地宿主机显存。
- **专注于“技能 (Skill)”的极致编排**：不再局限于死板的知识库检索，而是打造一套动态调用、编排和调度的技能路由系统。
- **个人专属化 (Personalized)**：旨在服务于个人的定制化需求，作为深度集成各项私人工具的随身智囊。

## 2. 技术路线与选型 (Tech Stack)

- **核心语言**：Rust (edition 2021)
- **并发与网络**：`tokio` (异步运行环境) + `reqwest` (网络通信)
- **序列化**：`serde` + `serde_json`
- **运行环境**：完全在 DevContainer 中进行开发与编译，确保环境隔离。

## 3. 三元多元化架构 (Tri-Tier Diversified Architecture)

为了实现系统的高内聚低耦合，以及应对未来复杂的业务场景，AI-Nexus 采用了经典的“设计优先”三元架构理念：

### 3.1 Tier 1: 网关多元化 (Gateway / Channel Diversity)
* **通道层职责**：作为系统的感官神经末梢，它不处理任何复杂的业务流转。
* **特性**：支持多平台原生接入（Telegram, Slack, Web等），支持多用户完全隔离，完美适配聊天流交互界面。每一个接入点就是一个独立的“物理通道”。

### 3.2 Tier 2: 智能代理多元化 (Agent Diversity)
* **代理层职责**：作为系统的心智分身（🧠 大脑层），承接来自 Tier 1 的指令。
* **特性**：系统内允许多个 Agent 并存。每个 Agent 都拥有独立的人设（Persona）、记忆上下文、专属的大模型算力路由规则（Model Router），以及其被授权访问的特定技能池。这是实现“心智”多元化的核心中枢。

### 3.3 Tier 3: 技能多元化 (Skill Diversity)
* **技能层职责**：作为系统的手脚（🛠️ 行动层）。
* **特性**：涵盖无限扩展的原子化能力（如联网自动化、代码执行、数据库调度等），可按需动态装载或拔插给指定的 Agent，为 Agent 提供与物理世界交互的手段。

### 3.4 架构流转图 (HTML Diagram)

<div style="font-family: sans-serif; text-align: center; color: #333; line-height: 1.5; padding: 20px; border: 1px solid #ddd; border-radius: 8px; background-color: #f9f9f9; max-width: 700px; margin: 0 auto;">
    <!-- Channel Level -->
    <div style="display: flex; justify-content: center; gap: 20px; margin-bottom: 20px;">
        <div style="padding: 10px 20px; background-color: #e0f2fe; border: 1px solid #7dd3fc; border-radius: 6px; flex: 1;">Channel (Telegram)</div>
        <div style="padding: 10px 20px; background-color: #e0f2fe; border: 1px solid #7dd3fc; border-radius: 6px; flex: 1;">Channel (LINE / Web)</div>
    </div>
    <div style="margin-bottom: 10px; font-size: 0.9em; color: #555;">⬇️ 映射与权限校验 (Nexus OS) ⬇️</div>
    <!-- Agent Level -->
    <div style="background-color: #fef08a; border: 2px solid #facc15; border-radius: 8px; padding: 20px; margin-bottom: 20px;">
        <h3 style="margin-top: 0; color: #854d0e;">🤖 Agent (心智分身)</h3>
        <p style="font-size: 0.9em; color: #854d0e; margin-bottom: 15px;">拥有独立人设、记忆和权限的虚拟数字实体。它接收指令，进行意图理解与规划。</p>
        <div style="padding: 15px; background-color: #fce7f3; border: 1px solid #f9a8d4; border-radius: 6px; display: inline-block;">
            <strong>🧠 Gemini Core</strong><br>
            (Agent 的推理大脑与状态机)
        </div>
    </div>
    <div style="display: flex; justify-content: space-around; margin-bottom: 10px; font-size: 0.9em; color: #555;">
        <div style="width: 45%;">↙️ 算力请求 ↙️</div>
        <div style="width: 45%;">↘️ 动作指令 ↘️</div>
    </div>
    <!-- Router Level -->
    <div style="display: flex; justify-content: space-between; gap: 15px; margin-bottom: 20px;">
        <div style="flex: 1; padding: 15px; background-color: #dcfce7; border: 1px solid #86efac; border-radius: 6px;">
            <strong>🔀 Model Router</strong><br>
            <span style="font-size: 0.8em;">(向上调度：根据配额挑选最佳云端模型)</span>
        </div>
        <div style="flex: 1; padding: 15px; background-color: #dcfce7; border: 1px solid #86efac; border-radius: 6px;">
            <strong>⚙️ Skill Router</strong><br>
            <span style="font-size: 0.8em;">(向下调度：分发动作指令至物理引擎)</span>
        </div>
    </div>
    <div style="display: flex; justify-content: space-between; gap: 15px; margin-bottom: 10px; font-size: 0.9em; color: #555;">
        <div style="flex: 1;">⬇️</div>
        <div style="flex: 1;">⬇️</div>
    </div>
    <!-- Foundation Level -->
    <div style="display: flex; justify-content: space-between; gap: 15px;">
        <div style="flex: 1; padding: 15px; background-color: #f3f4f6; border: 1px dashed #9ca3af; border-radius: 6px;">
            <strong>☁️ Cloud Models</strong><br>
            <span style="font-size: 0.8em;">(gemini-1.5-flash / gemini-1.5-pro 等)</span>
        </div>
        <div style="flex: 1; padding: 15px; background-color: #f3f4f6; border: 1px dashed #9ca3af; border-radius: 6px;">
            <strong>🛠️ Skill Engine</strong><br>
            <span style="font-size: 0.8em;">(隔离沙箱中执行具体的代码/工具)</span>
        </div>
    </div>
</div>

### 3.3 核心组件说明

系统运行在中枢总线的调度下，呈现“漏斗状双路由”架构：

1. **Gateway (渠道网关)**：物理层的监听入口。接收不同 Channel（Telegram, Lark）的用户输入，并将其绑定到特定的默认 Persona。
2. **Persona (人格配置)**：静态模板，定义系统提示词和可使用的 Skill 白名单。
3. **Agent (智能体实例)**：动态实例。系统基于 Persona 实例化 Agent，它携带上下文记忆与算力标签 (Capability Requirement)。
4. **Model Router (算力中心)**：拦截 Agent 发出的推理请求，读取算力标签（如 `Tier-1`），并前往 `Providers Registry` 动态提取脱敏的 API Key，完成底层云端模型的调度。
5. **Skill Engine (技能引擎)**：Agent 与外部环境交互的物理沙箱手臂。执行权限受限于生成该 Agent 时挂载的 Persona 约束。

## 4. 下一步演进 (Next Steps)

- 明确各组件之间的消息传递协议。
- 设计并测试第一版与 Gemini API 的联调。
- 构建基础的技能生命周期 (Skill Lifecycle)。


---


# 项目结构规范 (Base Structure)

为了确保 AI-Nexus 的高效开发与维护，所有的代码实现必须遵循以下模块划分：

## 核心目录 (Core Directories)

```text
AI-Nexus/
├── .devcontainer/             # 隔离的开发环境配置
├── docs/                      # 核心设计、架构与进度跟踪文档
│   ├── 01_CORE_FOUNDATION.md       # 核心愿景、架构哲学、领域设计
│   ├── 02_ARCHITECTURE_SKILLS.md   # 技能引擎与双子解耦架构
│   ├── 03_DATA_AND_CONFIG.md       # 数据治理与环境配置规范
│   ├── 04_ECOSYSTEM_INTERFACES.md  # 外部接入与生态延展
│   ├── 05_SPEC_CORE_INTERFACES.md  # 底层基础契约（ACP, Trait 定义）
│   ├── 06_SPEC_META_AGENT_AND_SKILL.md  # Meta Agent 与元技能规范
│   ├── 07_SPEC_MAIN_AGENT.md       # Main Agent 调度规范
│   ├── 08_LIFECYCLE_AND_GARBAGE_COLLECTION.md
│   ├── 10.x_MODULE_SPEC_*.md       # 各模块详细设计规范
│   ├── 12~15_SPEC_*.md             # 高级特性、企业架构等补充规范
│   └── 20_BASE_PROGRESS.md         # 项目进度
├── src/
│   ├── main.rs                # 应用程序入口
│   ├── agent/                 # 心智分身设定与长短期记忆管理
│   ├── core/                  # 基础协议、Trait 和错误定义
│   ├── gemini/                # Gemini API 通信与意图理解层
│   ├── iam/                   # 身份验证与配额计费管理
│   ├── os/                    # 调度中枢与生命周期管理
│   ├── scheduler/             # 后台任务调度与定时触发器
│   ├── skill/                 # 核心的技能编排与沙箱执行引擎
│   ├── storage/               # 数据持久化、向量库与状态恢复
│   ├── test/                  # 集成测试（E2E 测试）
│   └── utils/                 # 通用工具函数库
├── tests/                     # 联合集成测试（joint_e2e.rs 等）
├── frontend/                  # Dashboard 前端（React/Vite）
├── Cargo.toml                 # 依赖配置
└── README.md                  # 项目概览
```

## 模块隔离原则
1. **职责单一**：禁止跨层级进行反向依赖调用。
2. **面向接口**：子模块间的数据流转应依赖 `Trait` 或明确定义的枚举 (Enum)，而不是直接耦合实现细节。


---


# AI-Nexus 领域设计 (Domain Design)

本篇文档定义了 AI-Nexus 的核心领域模型、架构理念以及状态流转机制。其核心灵感来源于最前沿的 Agent IDE（如 Antigravity）的配额与调度机制。

## 1. 核心架构哲学 (Architecture Philosophy)

传统大模型应用倾向于“胖上下文（Fat Context）”——即在一次请求中塞入百万级 Token 强行让大模型理解全部信息。
**AI-Nexus 完全摒弃这一模式。** 

我们采用 **“瘦上下文 + 强技能网络 (Thin Context + Strong Skill Network)”** 架构，并融合了**多模态与异步事件驱动 (Multimodal & Event-Driven)** 哲学：
- **极致的克制原则 (Principle of Restraint)**：相较于旧版 Aura 追求“无限知识库”和“臃肿的全局技能库”，AI-Nexus 坚持“克制”。知识库仅保留高价值的中短期核心记忆，拒绝无脑的数据堆砌；初始内置技能库保持极简，仅保留必备原子工具。当面临边缘需求时，依赖 `Meta Skill` 动态生成“用完即走”的临时逻辑。保持系统绝对轻盈、高响应率与高度可控。
- **瘦上下文 (Thin Context)**：中枢调度模型 (Nexus OS) 的上下文窗口将被严格控制在极小范围（例如 16K Token），以确保其具有极高的响应帧率 (高 RPM) 和极低的推理延迟，降低无效 Token 损耗 (低 TPM)。
- **强技能网络 (Strong Skill Network)**：知识检索、网络搜索、文件读写等一切厚重任务，均作为外挂技能 (Skills) 存在，由调度器根据短小的意图进行调用。
- **多模态与后台异步流转 (Multimodal & Asynchronous)**：打破传统的文本聊天框桎梏，系统原生支持图片、音频等多模态附件输入。同时，OS 总线具备跨越时间的调度能力（Cron/Detached Jobs），使 Agent 成为全天候值守的自主实体。

---

## 2. 四大核心管理底座 (Four Core Foundations)

整个 AI-Nexus 系统的职责，本质上就是对以下四大核心资源进行精准的调度与权限收口：

### 2.1 资源一：多模型矩阵 (Multi-Model Pool)
系统需要管理几十种可用的 Gemini API 模型（如 Flash, Pro, Lite, Live API 等）。
- **调度策略**：绝不是简单地写死一个模型，而是必须根据**当前任务的复杂度需求**、**API 剩余可用额度 (Quota)** 以及**当前所需的 Skill 的特性**，动态挑选最合适的模型去执行。
- *例如*：高频意图分类使用 `gemini-1.5-flash-8b`；需要编写复杂新技能时，自动切换为 `gemini-1.5-pro`。

### 2.2 资源二：动态技能树 (Dynamic Skills)
系统摒弃预加载上百个死板功能的做法，转而管理一个动态的技能生命周期。
- **常规技能**：如 `Web Search`, `Knowledge Base` 等极少数用作地基的原子技能。
- **Meta Skill (元技能)**：用于管理和进化其他技能的核心特权技能。严格限定为以下 **5 项**：
  1. 描述技能规则的知识增强型技能。
  2. 读取技能文件代码与配置的技能。
  3. 新建或修改技能文件的技能。
  4. 验证与评价其他技能执行能力的技能。
  5. 生成与装配全新 Agent 分身的技能（详见 [06_SPEC_META_AGENT_AND_SKILL.md](../04_Advanced_Agents/02_SPEC_META_AGENT_AND_SKILL.md)）。

### 2.3 资源三：多渠道通信与权限隔离 (Channels)
系统将接入各种不同的前端界面或聊天软件（如 Telegram, Web UI, Discord 等），我们将这些接入点抽象为 **Channel**。
- **多模态富媒体 (Multimodal)**：Channel 不仅仅传递纯文本，还会将文件、截图等封装为多模态附件传入。
- **IAM 身份鉴权 (AuthN & AuthZ)**：通过独立的 IAM 模块，实现对欠费用户的硬拦截和 API 防刷。
- **Meta Agent (全局造物主)**：系统的最高特权实体。系统内部存在一个全局唯一的超级心智（Agent），**全系统唯有这个 Meta Agent 有权限调用五大 `Meta Skill`**。通过将特权与内部 Agent 强绑定，我们在架构上彻底废除了“特殊外部信道 (Meta Channel)”的设计，从而实现了更安全的物理隔离。

### 2.4 资源四：时空连贯性 (Time & Space Continuum)
使 Agent 脱离“即时问答”的局限，具备在时间和空间上延续的能力：
- **存储与记忆引擎 (Storage & Vector DB)**：通过本地二进制块、轻量图数据库与 HNSW 的分层存储，不仅实现 L1热数据/L2温数据/L3冷语义向量的跨会话记忆，更引入 **GraphRAG 与联想记忆引擎 (Associative Memory)**，赋予大模型类人的多跳推理与拓扑结构认知能力。
- **异步任务调度器 (Scheduler)**：赋予 Agent 在无外力触发时的主动执行能力，支持基于 Cron 的定时拉起和沙箱任务的长挂起守护。

---

## 3. 技能状态流转 (State Transition)

当用户发起请求时，AI-Nexus 的状态机遵循以下标准周期：

1. **[Awake & Auth]** 唤醒与鉴权：经过 IAM 拦截器校验余额及合法性后，组装当前 Channel 的多模态意图与极少量 RAG 记忆进入 16K 核心上下文。
2. **[Routing & Delegation]** 路由与外包：根据任务复杂度挑选 **Models**。如果任务极其复杂，可以通过内部通道 (MPSC Channel) 将子任务外包 (Delegation) 给 Meta Agent 或其他专职 Agent 处理。
3. **[Executing & HITL]** 执行与审查：校验当前权限后，在本地物理沙箱中执行工具。如果是高危操作，触发 HITL (人类在环) 将进程挂起等待人工审批；如果执行报错，则启动 ReAct 自我纠错循环，将报错信息注入上下文让大模型重新反思。
4. **[Aggregating & Streaming]** 聚合与流式输出：将执行结果附加到对话末尾，利用 SSE/WebSocket 机制向外部 Channel 实时流式推送最终回复，消灭等待空白期。
5. **[Sleeping & Distilling]** 休眠与精炼：返回完毕，在后台将临时记忆转入长效 Storage，通过 WAL 快照落盘持久化。

