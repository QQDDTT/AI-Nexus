# AI-Nexus Architecture Documentation

本目录包含了 AI-Nexus 系统的所有核心架构设计与规范文档。为了便于阅读、维护与企业级扩展，文档已按照领域驱动设计 (Domain-Driven) 分解为以下 6 个逻辑子模块：

## [📁 01. 顶层架构与核心生命周期](./01_Architecture)
系统底层的世界观、最高指导原则与全局架构。
- `01_CORE_FOUNDATION.md` - 核心定位与系统基础
- `02_ARCHITECTURE_SKILLS.md` - 执行层与沙箱双子架构
- `03_DATA_AND_CONFIG.md` - 数据治理与环境配置法则
- `04_CAPABILITY_BASED_ROUTING.md` - 混合模型路由与算力分发
- `05_LIFECYCLE_AND_GARBAGE_COLLECTION.md` - 实例生命周期与垃圾回收

## [📁 02. 核心子系统实现规范](./02_Modules)
深入探讨各业务子系统（对应 `src/` 下的具体模块）的实现细节。
- `01_MODULE_OS.md` - OS 调度模块与会话状态机
- `02_MODULE_AGENT.md` - 活体 Agent 与记忆管理模块
- `03_MODULE_GEMINI.md` - 大模型核心通信模块
- `04_MODULE_SKILL.md` - 技能引擎与执行模块
- `05_MODULE_STORAGE.md` - 底层存储与单体多模块架构
- `06_MODULE_UTILS.md` - 公共工具与错误处理
- `07_MODULE_DASHBOARD.md` - 前端仪表盘与运维面板

## [📁 03. 接口契约与网关通道](./03_Interfaces)
定义模块之间的底层 Trait 契约以及对外暴露的网络通道。
- `01_SPEC_CORE_INTERFACES.md` - 底层 ACP 通信协议与核心领域模型
- `02_SPEC_DASHBOARD_API.md` - Dashboard 后台 API 规范
- `03_ECOSYSTEM_INTERFACES.md` - 外部生态与大模型能力对接
- `04_LOCAL_TERMINAL_CHANNEL.md` - CLI 本地开发调试终端通道

## [📁 04. 高级分身与特权系统](./04_Advanced_Agents)
剥离普通工作流，专门存放系统的“总线调度级”和“造物主级”分身设定。
- `01_SPEC_MAIN_AGENT.md` - 全局总管家分身
- `02_SPEC_META_AGENT_AND_SKILL.md` - 负责自动造物的 Meta Agent 与特权技能

## [📁 05. 前瞻特性与企业级演进](./05_Future_Planning)
规划系统未来的进化蓝图，包含高阶架构迭代设想。
- `01_SPEC_ADVANCED_FEATURES.md` - 高阶与待探索特性
- `02_SPEC_ENTERPRISE_ARCHITECTURE.md` - 企业级商业架构
- `03_SPEC_SOTA_FEATURES.md` - 行业前沿特性融合计划

## [📁 06. 测试、进度与运维参考](./06_Testing_and_Operations)
系统开发进度、外部模型参数资料以及端到端测试方案。
- `01_TEST_STRATEGY_E2E.md` - 端到端全链路测试策略
- `02_TEST_CHECKLIST_DASHBOARD.md` - Dashboard 前后端联调 Checklist
- `03_EXECUTION_TOOLS_REGISTRY.md` - 工具执行注册表
- `04_BASE_PROGRESS.md` - 基座开发进度与代办
- `05_GOOGLE_AI_STUDIO_LIMITS.md` - Google AI Studio 速率与模型规格限制
