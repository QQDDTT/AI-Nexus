# Architecture & Skills (核心架构：执行与技能引擎)

本技术论证文档详细阐述在 AI-Nexus 高性能操作系统级 AI 代理框架中，物理执行平面的重塑。为了实现最高级别的性能与安全隔离，我们彻底废弃了任何跨进程 RPC、Python 运行时以及外部 Docker 容器的方案，转而采用了**单体多模块（Monolithic Multi-Module）结合原生 WebAssembly (`wasmtime`) 沙箱**的新型架构。

---

## 1. 架构演进背景与核心痛点 (Problem Statement)

AI-Nexus OS 原有的物理执行平面曾考虑过引入 Python 沙箱或独立 Webhook 进程。然而，这些架构在特定阶段暴露出三大工业级痛点：

1. **契约库编译膨胀与难以拔插**：每次在现实场景中增加或变更一种物理动作（如增加一个特定平台的 API 访问或传感器控制），都必须修改核心 `ainexus-core` 的 `ExecutionDsl` 枚举契约，导致整个系统重新编译，破坏了操作系统底座的稳定性和动态插拔灵活性。
2. **微服务链路冗余与高延迟**：将网络请求或脚本运行隔离至独立进程，极大增加了 UDS/IPC 的通信开销（定长帧封装、反序列化、双向握手），且导致网络动作与本地计算动作割裂，给全系统带来了不可忽视的延迟（10ms ~ 50ms）。
3. **安全隔离机制过重**：使用 `Docker` 或 `CLONE_NEWUSER` 容器级隔离不仅极大拖慢了启动速度，而且在某些严格限制权限的主机上极易产生兼容性问题。

为了达成 **“宿主机的绝对安全防御”**、**“技能的无限动态插拔”** 与 **“10微秒级极速响应”**，AI-Nexus 废除旧有架构，重构为统一在单一 Rust 进程内的「单体多模块 `Wasm` 隔离引擎」。

---

## 2. 技能化引擎系统定义 (Skill Engine Definition)

新设计将所有物理动作（包括网络访问、数据清洗、异构系统控制等）统一抽象为“技能（Skills）”。为保障执行态的零信任安全，执行平面在同一个 OS 进程内划分为逻辑解耦的两个模块：**`Meta Agent 生成器`**（负责通过大模型生成、编译并持久化技能资产）和 **`Skill Sandbox`**（专注于已部署 Wasm 技能的只读加载与受控执行）。

### 2.1 技能实体规范 (Skill Specification)

每个技能作为系统的资产，存储于 BlockStore 的特定目录下，包含两个核心概念：

1. **说明文件 (`SKILL.md` 或等效 JSON)**：
   明确记录技能的元数据与数据字典（Schema），包含：
   - `id`: 全局唯一标识符 (UUID v4)
   - `title`: 技能名称
   - `input`: 输入参数字典 Schema (含名称、类型、是否必填)
   - `output`: 输出数据字典 Schema
2. **二进制产物 (`skill.wasm`)**：
   - 技能的具体执行逻辑不再使用明文 Python 脚本，而是编译后的 `.wasm` 二进制格式。
   - 保证极速冷启动（亚毫秒级）和严格的内存隔离。

### 2.2 ACP 协议契约的统一对齐 (ACP Contract Refactoring)

为配合“技能化（Skills-based）”物理运行平面的确立，并且消除历史遗留的协议不一致（如 `ExecutionDsl` 与 `ActionTrigger` 的混淆），现统一执行层的协议契约：

1. **执行指令定义 (`ExecutionDsl::Skill`)**：
   在底层的执行面，路由信封（总线层的 `ActionTrigger`）将被解析和投影为精确的 `ExecutionDsl`，它是发往沙箱的最终指令：
   ```rust
   pub enum ExecutionDsl {
       Skill {
           /// 技能唯一标识符 (UUID，防重名冲突)
           skill_id: String,
           /// 技能名称
           title: String,
           /// 技能入参 (对齐声明的 input 数据字典)
           input: serde_json::Value,
           /// 沙箱最大运行超时限制 (毫秒)
           timeout_ms: u64,
       },
   }
   ```
   > 注：总线层的 `AcpPayload::ActionTrigger` 关注人类可读的 `skill_name`；执行层的 `ExecutionDsl::Skill` 关注防冲突的 `skill_id (UUID)`。

2. **执行回执定义 (`ExecutionOutcome` / `ActionResult`)**：
   执行结束或遇到沙箱拦截时，执行子模块将返回统一的 Outcome，随后封装入 `AcpPayload::ActionResult`，并通过 BlockStore 持久化到 Ledger 中：
   ```rust
   pub struct ExecutionOutcome {
       /// 物理执行是否成功
       pub success: bool,
       /// 技能输出结果（统一为 JSON 结构）
       pub data: serde_json::Value,
       /// 错误原因描述 (若 success 为 false)
       pub error: Option<String>,
   }
   ```

### 2.3 单体多模块协作架构 (Core Components)

新版执行子系统通过 `wasmtime` 实现进程内绝对安全的沙箱闭环：

```mermaid
graph TD
    A[大模型 / Meta Agent] -->|1. 生成技能指令| B[OS 事件总线]
    B -->|2. 调用编译器编译| C[Wasm 二进制产物]
    C -->|3. 部署并写入| D[BlockStore 技能清单]
    
    E[普通 Agent 推理] -->|4. 下发执行请求| F[Skill Router]
    F -->|5. 映射为 ExecutionDsl| G[Skill Sandbox (wasmtime)]
    D -->|6. 内存映射只读加载| G
    G -->|7. 绝对沙箱隔离运行| H[物理反馈回写 BlockStore]
```

---

## 3. 安全防护与隔离边界 (Security & Isolation Boundary)

利用 `wasmtime`，原本复杂的内核级 Cgroups 限制和命名空间隔离（如 `CLONE_NEWUSER`）被极简的进程内虚拟机完全替代：

1. **零内存越界（Memory Sandboxing）**：
   Wasm 模块运行在独立的线性内存中，完全无法读取宿主机的指针或内存，彻底杜绝了越权访问。
2. **资源与指令消耗限制（Fuel Consumption）**：
   通过 `wasmtime` 的 Fuel (燃料) 机制，可以对大模型生成的死循环代码进行精确控制。燃料耗尽后沙箱立即熔断。
3. **能力白名单（WASI Capabilities）**：
   默认情况下，Wasm 模块处于 "Deny-by-default" 状态。除非在 `sandbox.rs` 中显式向其实例注入特定的 WASI 模块（如网络访问白名单、特定文件夹的只读访问），否则它连获取当前系统时间的能力都没有。这比 Seccomp-BPF 更加安全且无兼容性负担。

---

## 4. 对称架构版本对齐

- **状态**: 单体多模块 Wasm 原生沙箱架构 (v12.0.0)
- **最后更新**: 2026-07-20
