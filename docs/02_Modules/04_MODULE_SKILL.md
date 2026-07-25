# Skill 模块设计规范 (Skill Engine & Router Module Specification)

`src/skill` 模块是**知识库 (Knowledge Base)**与**技能库 (Skill Base)**整合的产物。它不仅赋予大模型执行物理动作的“双手”，还为其提供业务逻辑和经验法则的“行业大脑”。此模块必须提供高并发的执行能力，保障 Wasm 代码沙箱的绝对安全，并统一维护系统的知识与动态技能资产。

## 1. 核心职责 (Core Responsibilities)

1.  **技能全生命周期管理**：严格履行双轨生命周期机制。
    *   针对 **可执行技能**：履行在 `05` 契约文档中定义的 `定义 -> 校验 -> 执行 (Wasm 沙箱运行) -> 审核` 四阶段流水线。
    *   针对 **知识增强技能**：适配为 `定义 -> 校验 (向量化检查) -> 召回 (上下文挂载) -> 反馈 (归因评估)` 流水线，实现对静态知识资产的精细化治理。
2.  **物理沙箱与绝对隔离**：基于 `wasmtime` 构建原生 Wasm 隔离执行环境。通过 WebAssembly 的线性内存模型与 WASI 严格权限控制（Deny-by-default），提供微秒级极速启动与绝对安全。
3.  **GraphRAG 混合检索引擎**：管理所有静态和动态技能的 Metadata 向量与图谱拓扑边。为大脑提供基于联想记忆的多跳召回，限制每次传给大模型的上下文体积并确保上下文包含全局关联视野。
4.  **无状态函数模型**：技能必须被编译为纯粹的 `.wasm` 模块，所有持久化状态必须通过 `BlockStore` 引擎进行读写，沙箱内部严禁随意读写宿主文件系统。

## 2. 技能的类型与实体规范 (Skill Types & Specification)

在 AI-Nexus 的架构中，Skill 被划分为两大类，统一在二进制 `BlockStore` 及 `skills/` 逻辑目录下管理。

### 2.1 知识增强技能 (Knowledge-Enhanced Skills)
本质上是对“知识库”的封装。这类技能不涉及对宿主环境的物理状态变更（无副作用），主要用于提供领域知识、代码规范、操作SOP或历史经验。
*   **触发方式**：通过 GraphRAG 与联想记忆机制 (Associative Memory) 顺着拓扑关系被动扩散召回，或者作为上下文被大模型主动挂载。
*   **内容实体**：主要由 `SKILL.md` 与附属的 Markdown/JSON 参考资产组成。

### 2.2 可执行技能 (Executable Skills)
本质上是对“技能库”的封装。这类技能包含明确的副作用，负责调用 API、修改文件或执行计算。
*   **触发方式**：由大模型主动生成 `ActionTrigger` 调用指令。
*   **内容实体**：标准的 WebAssembly 模块（`.wasm`）。大模型动态生成的 Rust 源码将被编译为无状态的 Wasm 二进制。对外暴露统一标准的 `_start` 或特定 `execute` 导出函数。

## 3. 关键设计与架构

### 3.1 单体 Wasm 沙箱架构 (Monolithic Wasm Sandbox)

摒弃了沉重的 Python 进程与容器隔离，全面拥抱单进程架构：
*   **微秒级冷启动**：相较于拉起 Docker 或新建 Python 进程，实例化一个 Wasm 模块仅需几微秒至几毫秒，开销可忽略不计，彻底消除 UDS/IPC 通信延迟。
*   **Fuel (燃料) 限额控制**：通过 `wasmtime` 引擎的指令燃料机制（Fuel Consumption），强制锁定执行上限。无论是大模型不小心生成的死循环还是恶意运算，一旦燃料耗尽即刻终止，完美取代了粗糙的 Cgroups CPU 限额。
*   **WASI Deny-by-default 权限**：所有外部资源（文件、网络、时钟）必须由宿主环境（AI-Nexus OS）在实例化沙箱前通过 Capability-based 模型显式注入。未授权的调用直接在指令层面返回错误。

### 3.2 GraphRAG 混合检索引擎

为实现毫秒级的知识与技能联合召回，不依赖沉重的外部关系型数据库：
*   系统启动时，调用 Embedding 接口，将技能描述转换为高维向量，并抽取实体挂载至网络拓扑节点。
*   利用纯 Rust 实现的图数据库引擎结合轻量级 HNSW 库构建内存检索图，通过余弦距离和激活扩散模型比对意图，快速、且带有**联想逻辑**地召回最相关的“知识增强技能”与“可执行技能”。

## 4. 核心接口与数据结构

```rust
pub mod sandbox {
    use wasmtime::{Engine, Module, Store, Linker};
    
    /// 基于 wasmtime 的极速沙箱
    pub struct WasmSandbox {
        engine: Engine,
    }

    impl WasmSandbox {
        /// 初始化并配置 Fuel 与 WASI 限制
        pub fn new() -> Self { /* ... */ }
        
        /// 执行技能，注入极其严格的 Capabilities
        pub async fn execute_wasm(&self, wasm_bytes: &[u8], params: serde_json::Value) -> Result<Vec<u8>, SandboxError> {
            // ... 实例化模块，注入限额的燃料，执行导出函数并捕获结果
        }
    }
}

pub mod pipeline {
    /// 技能执行的流水线统筹器
    pub struct SkillPipeline;

    impl SkillPipeline {
        pub async fn run_skill(skill_id: &str, raw_params: serde_json::Value) -> Result<Vec<u8>, anyhow::Error> {
            // 1. 拦截校验与依赖审计
            // 2. 调度 Wasm Sandbox 执行 (依赖 wasmtime 自动阻断)
            // 3. 执行结果后置清洗与反序列化
            Ok(vec![])
        }
    }
}
```

## 5. 容错与断崖策略

1.  **指令级硬性熔断 (Fuel Exhaustion)**：当 Wasm 模块执行的指令数量超出预设 Fuel 限额时，`wasmtime` 会抛出 `Trap::OutOfFuel`，执行被立即且安全地终止。
2.  **审核失败阻断**：当 `pipeline` 判定输出结果不符合预先注册的规范 Schema 时，流转管道直接抛出异常并阻断动作，从而迫使 `Nexus OS` 捕获异常现场并要求大模型重新生成修复方案。

