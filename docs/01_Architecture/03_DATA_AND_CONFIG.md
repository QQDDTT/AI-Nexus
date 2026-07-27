# Data & Configuration (数据治理与环境配置)

# 16. 全局数据管理规则 (GLOBAL_DATA_MANAGEMENT_RULE)

## 1. 核心宪章：读写绝缘法则 (Data Insulation Law)

在追求极致物理性能和微秒级调度的 AI-Nexus 操作系统中，数据持久化存储是系统的心脏。为了彻底杜绝运行时的文件锁竞争、序列化开销与数据不一致性，AI-Nexus 确立了不可逾越的**“读写绝缘法则”**。

本法则严苛划定了两种文件格式（`.json` / `.yaml` 与 `.bin`）的物理隔离与操作权限。

### 1.1 JSON/YAML：静态特权数据 (Initialization & Admin Only)
所有人类可读的结构化文本格式（包含 `.json`, `.yaml`, `.toml` 等）在系统中**仅拥有以下唯一合法身份**：
1. **系统初始化种子 (Genesis Seeds)**：在 `ainexus-init`（项目初始化工具）被唤醒时，一次性读取并导入底层数据库。
2. **人工干预快照 (Admin Snapshots)**：管理员通过外部工具手动导入、导出与备份使用。

**绝对红线 (The Absolute Redline)**：
- **禁止运行时修改**：AI-Nexus 系统的任何运行时自动化操作（推理、进化、部署、交互），**绝对禁止**向任何 `.json` 文件写入或追加数据。
- **测试清单只读与沙箱隔离**：测试子系统（`ainexus-test`）在消费声明式测试清单（如 `ainexus_telegram_e2e_checklist.json` 等）时，**绝对禁止**任何测试用例或逻辑运行时修改这些用例 JSON 本身。所有由于测试运行产生的临时状态和数据，必须被局限在 ACS 沙箱隔离的 `tmpfs` 虚拟文件系统内，且测试生命周期结束时随用随销，严禁向宿主机物理磁盘写入任何脏数据或残留文件。
- 自动化组件（如 `ainexus-developer`, `ainexus-evolution`）必须将其持久化需求卸载给物理基座。

### 1.2 Binary (Postcard/GraphDB & HNSW)：动态演进数据 (Runtime Operations)
AI-Nexus 自动化运行过程中的一切状态演变（技能上下线、心智进化、记忆沉淀），必须且只能以**纯二进制**的形式，通过进程内的 `BlockStore` 引擎进行操作。
1. **Block-L/I/E**：使用 `postcard` 纯二进制无模式序列化引擎进行数据封包，落盘为不可变的 `.bin` 块。
2. **语义图谱与向量索引 (GraphDB & HNSW)**：以高维稠密向量结合轻量级关系图谱（实体与边）的方式持久化至系统的内存映射存储中，支持基于 GraphRAG 的多跳联想记忆网络。

---

## 2. 核心数据实体管理规范

根据上述法则，系统内各大数据实体的流转规范如下：

### 2.1 技能库清单 (Skills Registry)
- **静态定义**：存放在 `data/blocks/longterm/skills.json`（或 `src/ainexus-init/data/seeds/skills.json`），仅作为 AI-Nexus-Init 初始化种子。
- **动态部署**：当进行技能热部署时，系统将调用进程内的 `BlockStore` 引擎，将技能的元数据打包成二进制 `DataBlock` 存入 `Block-L`，并更新 GraphRAG 图谱与 HNSW 混合检索树。运行时读取仅通过内存进行，绝不读取 JSON。

### 2.2 价值观与人格 (Personas & Values)
- **静态定义**：初始世界观由 `personas.json` 与 `prompts.yaml` 决定，由 `ainexus-init` 在 Genesis 时加载进内存模型。
- **动态进化**：`ainexus-evolution` 在执行长期的变分自由能 (VFE) 回收、评价与价值观漂移时，所有的参数修改必须固化为二进制块状增量更新（Delta Record），运行时系统自动将 Delta 应用至基础结构，绝对不直接回写或覆盖原始 JSON。

### 2.3 用户与鉴权 (Users & Secrets)
- **静态定义**：存放在 `users.json` 和 `.env` 中，仅在启动阶段由 Init 或 OS Kernel 一次性加载至安全内存域。
- **动态行为**：如发生临时封禁、鉴权 Token 过期等事件，其状态修改将作为二进制日志落盘，并在重启时通过日志重放 (Log Replay) 与基础配置叠加生效。如需永久删除用户，必须由管理员从 JSON 移除后重启系统或发送特定的系统重置指令。

### 2.4 知识库与公理 (Knowledge Base)
- **静态定义**：外部语料存放在 `genesis_axioms.json` 及其他原始语料库中。
- **动态蒸馏**：AI-Nexus 的抽象学习工具会将海量原始语料蒸馏为事实碎片，并通过本进程的 `BlockStore` 持久化进入 `Block-L` 二进制网络，不再操作任何 JSON。

---

## 3. 设计目的 (Design Philosophy)

1. **零拷贝反序列化**：`.bin` 文件使得 `BlockStore` 可以运用 `mmap` 进行零拷贝反序列化，实现 10 微秒级的加载速度，远超 JSON 的字符串解析与分配开销。
2. **防止脏写与锁竞争**：在并发微服务下，多进程写 JSON 需要复杂的文件锁（File Lock），极易导致死锁或数据截断。追加写不可变的 `.bin` Block 则天然支持无锁高并发。
3. **数据一致性 (Immutability)**：所有运行时演化都是追加写入（Append-only），结合系统的垃圾回收 (GC) 机制，可以轻松实现历史溯源与因果防篡改。

---
- **状态**: 全局读写绝缘数据管理确立版 (v11.5.0)
- **最后更新**: 2026-06-04


---


# AI-Nexus 系统配置与环境隔离规范 (Config Specification)

## 1. 设计原则 (Design Principles)

为确保 AI-Nexus 系统的可移植性、安全性与规则透明度，系统配置严格拆分为 **底层环境配置 (.env)** 与 **规则策略配置 (YAML)** 两大类，禁止交叉混用。

**特别注意**：由于 AI-Nexus 的 Dashboard 平台提供了高度自由的 Channel（渠道，如 Telegram/Slack）配置与多租户管理能力，所有运行时的渠道名称 (name)、鉴权状态等**动态上下文严禁写入任何静态配置文件**，全部交由底座数据库管理。

- **`.env` (环境与基础设施配置)**：**仅保存**系统的基础设施路径/网络端点数据（Paths/Endpoints）。严禁在其中配置任何业务逻辑参数。*注：所有大模型 API Keys 均统一通过 NexusDb 的 `providers` 动态凭证池进行在线管理与加密脱敏，不再通过全局环境变量硬编码。*

---

## 2. `.env` 环境配置规范

`.env` 是系统启动时的基础设施可选配置。

### 2.1 允许存放的数据类型
1. **安全与本地环境 (Security & Local Env)**：如 `AINEXUS_ADMIN_PASSWORD` (管理员密码) 和 `DB_ENCRYPTION_KEY` (数据库加密主密钥)。
2. **路径数据 (Path Data)**：如日志存放路径、`BLOCKSTORE_PATH` 等底层设施的挂载点。

### 2.2 绝对禁止存放的数据
- **任何 Channel 渠道信息**：如 `telegram_bot_name`, `is_active`。这些信息必须在 Dashboard 动态配置，保存在底座数据库中。
- **性能与业务规则**：如 `max_threads`, `timeout_ms`，这些应放在 YAML 中。

---

## 3. `ainexus.yaml` 规则与全局资产配置规范

`ainexus.yaml` 将作为系统各维面的全局静态行为指南。

### 3.1 核心配置域 (Configuration Domains)

1. **计算性能参数 (Performance Parameters)**
   - `max_inference_threads`: 推理允许的最大并发线程数。
   - `max_execution_workers`: 技能执行的最大并发 Worker 数。
   - `rpc_timeout_ms`: 通信最大等待时间。
2. **路由配置**
   - 已迁移至基于能力标签 (Capability Tiers) 的动态路由，此文件不再硬编码全局模型。
3. **网络通信 (Network & Endpoints)**
   - `server_port`: 核心服务暴露的监听端口。
   - `dashboard_port`: 面板服务的监听端口。
   - `metrics_endpoint`: 监控指标暴露路由。

### 3.2 示例结构

```yaml
system:
  rpc_timeout_ms: 5000
  server_port: 8000
  dashboard_port: 3000
  metrics_endpoint: "/metrics"

compute:
  max_inference_threads: 8
  max_execution_workers: 4

models:
  # 注意：模型名称在代码和配置中一律使用 API 格式（短横线+小写），如 gemini-1.5-flash
  # 路由策略配置已迁移至数据库动态管理

  local_model_token_limit: 4096


```

---

## 4. 探索者私有推理资产 (BlockStore 二进制账本)

在多用户协同机制下，探索者在编写技能脚本（在受控沙箱内运行，并通过 `antigravity SDK` 申请推理算力）时所专用的 Gemini Key 与 Model 资源，将由底座的 **BlockStore 二进制引擎** 直接存储和管理。

- **底层打标隔离**：这些私有资产严禁出现在 `ainexus.yaml` 或 `.env` 中，而是直接写入到二进制的私有资产数据块中，并在追加写入时强制添加 **用户标记 (User Identifier / User Tag)**。
- **隔离代理逻辑**：当沙箱代码调用 SDK 时，引擎提取执行事件链的 User Tag，检索其在 BlockStore 绑定且 active 的 Key，若无专属 Key 则拒绝执行。

---

## 5. 维面 Config Loader 实现

在系统 `prepare` 阶段实现配置加载：

1. **依赖引入**: 使用 `serde`, `serde_yaml` 解析 YAML。
2. **加载顺序**: 
   - 优先读取环境变量 (`.env`)，获取密钥与挂载点。
   - 读取工作目录的 `ainexus.yaml` 配置，获取性能参数与网络端点。
   - 结合基座的动态 Channel 配置，完成系统初始化。
3. **单例模式/全局传递**: 解析后的配置应在 `prepare` 阶段初始化，并通过 `Arc<AiNexusConfig>` 注入到维面实例中。

---
**状态**: 多用户协同与私有资产隔离版 (v12.0.0)
**Last Updated**: 2026-07-20

> [!NOTE]
> **模型命名规范**：所有代码、配置文件中的模型名称一律使用 **API 格式**（短横线 + 小写），
> 如 `gemini-1.5-flash`、`gemini-1.5-pro`。文档叙述中可使用展示格式（空格 + 首字母大写），如 `Gemini 1.5 Flash`。


---


# AI-Nexus 系统集成测试设计 (13_TEST_DESIGN)

本文件定义了 AI-Nexus 系统的全量测试规划与执行规范。测试旨在确保系统在 ACP 消息流转、算法精确度、资源消耗及安全性方面达到工业级稳态。

---

## 1. 测试总则 (Testing Principles)

*   **集中化管理**: **所有测试脚本必须存放于 `src/ainexus-test` 目录下**。禁止在其他子项目（如 `ainexus-kernel`, `ainexus-data` 等）的 `tests/` 文件夹或源代码中混入集成测试脚本，以保持核心代码的纯粹性与测试逻辑的独立性。
*   **自动化驱动**: 优先使用 `cargo test` 配合自定义仿真环境进行自动化验证。
*   **因果追踪**: 所有的联合测试必须验证 `TraceID` 的完整生命周期。

---

## 2. 测试规划 (Test Categories)

### 2.1 单元测试 (Unit Testing)
*   **目标**: 覆盖所有子项目（Crates）的核心函数。
*   **要求**:
    *   验证每一个数学公式（如 VFE, Boltzmann 采样）的数值边界。
    *   验证所有数据转换逻辑（Serialization/Deserialization）。
    *   确保函数在异常输入下能抛出正确的 `AiNexusError` 而非崩溃。
*   **存放位置**: `src/ainexus-test/src/unit/`

### 2.2 联合测试 (Joint/Integration Testing)
*   **目标**: 覆盖所有 **ACP (AI-Nexus Control Protocol)** 传播环节。
*   **要求**:
    *   **协议解析**: 确保所有 `AiNexusFacet` 子类（交互层、推理层、执行层、进化层等）能够正确解析接收到的 `AiNexusMessage`。
    *   **协议生成**: 验证各层级产出的 `AiNexusMessage` 符合 ACP 2.0 规范，且 `target_facet` 路由正确。
    *   **因果链闭环**: 仿真 S0-S4 全生命周期流转，验证信号在各维面间的有序传递。
*   **存放位置**: `src/ainexus-test/src/joint/` 或 `src/ainexus-test/src/kernel_tests.rs`

### 2.3 性能测试 (Performance & Resource Testing)
*   **目标**: 参考 [18_RESOURCE_EVALUATION.md](18_RESOURCE_EVALUATION.md)，确保系统资源消耗在预期范围内。
*   **要求**:
    *   **内存基准**: 验证内核及各层级进程在常驻态下的内存占用。
    *   **资源优化验证**: 
        *   人为触发 `distill()`（蒸馏）操作，验证内存回收效率。
        *   触发“熵爆炸”预警逻辑，验证系统是否能自动执行紧急剪枝（Purge）。
    *   **并发压力**: 模拟高频 ACP 消息冲击，观测 CPU 抖动与内存泄漏情况。
*   **存放位置**: `src/ainexus-test/src/perf/`

### 2.4 系统测试 (System/Manual Testing)
*   **目标**: 针对无法自动化的 UI 交互与复杂业务场景进行手动验证。
*   **要求**:
    *   **Dashboard 体验**: 验证 631 视觉原则下的图谱渲染流畅度。
    *   **多端网关测试**: 手动在 Telegram/飞书客户端发送指令，观察系统反馈。
    *   **极端故障演练**: 手动关停 `ainexus-cache` 或断开内部 Channel 连接，验证系统的自愈能力与因果时钟的重对齐逻辑。

---

## 3. 测试环境规范 (Environment)

*   **Mock 机制**: 在 `ainexus-test` 中提供统一的 `MockProvider`，用于模拟真实的推理模型（Gemini）与物理技能（Wasm）。
*   **确定性回放**: 支持通过固定随机数种子重现特定演化路径。

---

## 4. 自动化流水线 (CI Integration)

*   每次提交 PR 必须通过 `ainexus-test` 下的全量测试集。
*   测试失败将直接阻塞 `dev` 分支的合并。

---
**状态**: 测试规划正式版 (v4.0.0)
**最后更新**: 2026-05-15


---


