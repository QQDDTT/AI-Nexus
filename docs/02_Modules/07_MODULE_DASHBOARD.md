# Dashboard 模块设计规范 (Management Console Specification)

`Dashboard` 模块是 AI-Nexus 系统的可视化管理控制台，为系统管理员或开发者提供对多智能体体系（Multi-Agent System）的全局透视与控制能力。

虽然系统的大部分用户交互通过 Telegram、CLI 等 Channel 进行，但为了监控系统状态、管理技能资产以及调试智能体的执行流，一个拥有丰富构建能力的可视化 Dashboard 是必不可少的。

## 1. 核心职责 (Core Responsibilities)

1.  **全局状态监控 (System Observability)**：实时展示系统资源占用、OS 模块运行状态、Gemini API 调用统计与延迟。
2.  **Agent 可视化管理 (Agent Fleet Management)**：
    *   查看所有存活/休眠的 Agent 列表及其 Persona 设定。
    *   实时监控 Main Agent 的任务分发状态与各个 Sub-Agent 的工作流流转。
3.  **Skill 资产控制台 (Skill Asset Console)**：
    *   提供可视化的技能库 (Skill Base) 管理界面，涵盖 Rust 原生技能与 Markdown 规范技能。
    *   **全智能编排 (Full-AI Orchestration)**：审核 Meta Agent 自动生成的新代码。在将动态生成的技能应用到生产环境前，提供人工 Review 与一键编译执行的控制台。
    *   **半智能协同编辑 (Semi-AI Collaborative Editing)**：在可视化的代码与规范编辑器中，深度挂载 `meta_skill` 的系统级认知。允许人类开发者通过直接输入自然语言指令（Prompt），召唤 AI 助手对选中的技能进行代码修改、重构或说明文档续写，形成“人机结对编程”的半智能演进工作流。
4.  **记忆与记忆库透视 (Memory & RAG Inspection)**：
    *   可视化查看 Agent 的短期记忆滑动窗口状态。
    *   提供 Zettelkasten 双向链接视窗与全景 3D 节点星图，管理底层的 GraphRAG 与 HNSW 混合记忆库，支持手动修正或删除错误抽取的实体信息。

## 2. 架构与技术选型预想

为了保证系统轻量且高效，Dashboard 的构建建议采用现代 Web 技术栈，并与 Rust 后端通过标准接口通信。

### 2.1 前端 (Frontend)
*   **构建工具**：Vite (提供极速的冷启动与热更新)
*   **核心框架**：React 或 Vue3 (按需选择)
*   **样式与 UI**：采用现代化的设计风格 (如 Glassmorphism、暗黑模式)，确保界面直观、充满极客感且信息密度合理。避免使用过于原始的默认样式，提供 Premium 的视觉体验。
*   **通信协议**：WebSocket (用于实时 Agent 状态推送与日志流) + RESTful/GraphQL API (用于配置修改与静态资源拉取)。

### 2.2 后端接入点 (Backend Integration)
*   在 `Nexus OS` 中暴露一组专用的 Admin API (通常挂载在一个独立的内部端口，如 `127.0.0.1:8080`)。
*   API 必须具备权限校验机制，防止普通用户通过外部网络直接访问控制台接口。

## 3. 关键交互视图设计 (Key Views)

### 3.1 概览大盘 (Overview)
*   核心指标：当前在线 Agent 数量、今日大模型 Token 消耗、系统拦截的恶意沙箱越权次数等。
*   拓扑图：实时展示 Main Agent 与底层执行 Agent 之间的树状/网状通信拓扑。

### 3.2 造物主审核中心 (Creator Review Center)
*   专为 `Meta Agent` 服务的代码审查流。
*   当 Meta Agent 在沙箱中完成了一个新 Skill 的编写和跑通测试后，在 Dashboard 会生成一个待办工单。
*   管理员可以查看该 Skill 的源码 Diff、沙箱测试的执行日志，并一键批准 (Approve) 或拒绝 (Reject)。

### 3.3 沙箱终端观测 (Sandbox Terminal Observer)
*   提供一个类似于 Web Terminal 的组件，允许开发者实时“附着 (Attach)”到某个正在执行可执行技能的隔离沙箱中，查看其 stdout/stderr 输出，用于深度 Debugging。
