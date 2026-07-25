# Utils 模块设计规范 (Infrastructure & Utils Module Specification)

`src/utils` 模块是 AI-Nexus 系统的纯技术基建底座。它不包含任何特定的业务逻辑，但它被所有核心模块（OS, Gemini, Skill）所依赖。为保障开发效率和系统稳定性，该模块提供了全局一致的错误处理、日志聚合与配置解析机制。

## 1. 核心职责 (Core Responsibilities)

1.  **全局错误体系 (Error Handling)**：构建扁平且含义明确的自定义错误枚举，方便业务层进行 Pattern Matching 和快速排障。
2.  **安全日志与链路追踪 (Secure Tracing)**：提供基于 `tracing` 库的高性能无锁日志流，并且强制对打印到控制台或文件的隐私数据进行“自动脱敏”。
3.  **配置管理器 (Config Parser)**：安全解析环境中的 `.env`、`yaml` 配置文件，并提供具有热更新能力的单例配置访问。

## 2. 关键设计与架构 (Key Designs)

### 2.1 复合日志体系 (Composite Logging System)

AI-Nexus 拥有复杂的并发协同逻辑，为了在排障与 Dashboard 可视化时互不干扰，系统的日志被严格划分为**系统日志 (System Logs)** 和 **Agent 日志 (Agent Logs)** 两个维度。同时，作为强隐私系统，所有日志流出前必须经过自动脱敏层。

#### 2.1.1 系统日志 (System Logs)
*   **定位**：侧重于基础设施、OS 内核层、资源分配与底层异常。供管理员或 DevOps 排查故障和性能瓶颈。
*   **内容特征**：包含网络请求耗时、数据库连接池状态、沙箱 Cgroup 隔离状态、配置热重载提示、全局严重 Error 等。
*   **输出流向**：通常采用结构化 JSON 格式写入文件或 stdout，以便对接 Loki/Vector 等传统日志收集平台。

#### 2.1.2 Agent 日志 (Agent Logs)
*   **定位**：侧重于“心智流”与高层业务逻辑。记录 Agent 的思考链路、Task 分发过程、Meta Agent 的代码生成行为及 Skill 的调用细节。
*   **内容特征**：必须强绑定上下文（`Agent_ID`、`Task_ID`、`Session_ID`）。涵盖模型的推理轨迹 (Thought Process)、工具调用出入参 (Tool Call) 以及多 Agent 间的通信握手 (Message Passing)。
*   **输出流向**：利用 `tracing` 库的 `Span` 机制追踪上下文树。除本地存储外，还会通过 WebSocket 流式投递至 Dashboard，用于渲染直观的工作流拓扑。

#### 2.1.3 安全脱敏管道 (Data Masking Pipeline)
无论是系统日志还是 Agent 日志，在最终格式化输出前，都会途经统一的安全拦截层：
*   **正则自动掩码**：匹配并替换诸如 `Telegram ID: [0-9]+`、`Bearer [a-zA-Z0-9]+` 等私人凭据，中间替换为 `****`。
*   **显式标签拦截**：遇到业务代码中标记为 `[SECURE]` 的数据结构，在 `Debug` 或 `Trace` 时仅输出其哈希摘要或长度，绝不打印明文。

### 2.2 统一的错误收口 (Unified Errors)

项目中严格区分“系统级不可恢复错误”与“业务级可预期错误”。

*   使用 `thiserror` 定义 `AiNexusError` 核心枚举，包含如下大类：
    *   `ApiQuotaExceeded` (模型额度耗尽，触发 Router 降级)
    *   `SkillExecutionFailed` (技能物理执行失败)
    *   `AgentContextCorrupted` (记忆断档)
*   对于最外层的总线分发代码或不需精确判断处理分支的通用代码，统一退化为 `anyhow::Result`。

## 3. 核心接口与数据结构

```rust
pub mod errors {
    use thiserror::Error;

    /// 贯穿全系统的核心业务错误集
    #[derive(Error, Debug)]
    pub enum AiNexusError {
        #[error("Model API quota exceeded, downgrading route")]
        ApiQuotaExceeded,
        
        #[error("WASM Sandbox execution failed for '{skill_name}': {reason}")]
        SkillExecutionFailed {
            skill_name: String,
            reason: String,
        },
        
        #[error("Agent context corrupted or memory lost")]
        AgentContextCorrupted,
    }
}

pub mod logger {
    /// 初始化带脱敏功能的链路追踪体系
    pub fn init_secure_tracing(log_level: &str) {
        // ... 配置 tracing_subscriber
        // ... 植入 Regex Masking Layer 过滤敏感关键词
    }
}

pub mod config {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct AppConfig {
        pub core: CoreConfig,
        pub models: ModelQuotas,
    }

    /// 提供全局单例的只读配置
    pub fn global_config() -> &'static AppConfig { /* ... */ }
}
```

## 4. 最佳实践 (Best Practices)

1.  **绝不使用 `println!`**：整个项目中禁止出现标准库的 `println!` 或 `eprintln!`。所有输出必须走 `tracing::info!` 或 `tracing::error!` 宏，以保证格式的一致性和脱敏过滤层的生效。
2.  **错误附带上下文**：在函数逐层抛出 Error 的时候，必须使用 `anyhow::Context` 追加当前步骤的上下文描述，避免底层抛出一个冰冷的 `IO Error` 导致上层无法追踪这是发生在读数据库还是读配置文件时。

---

## 5. 配置解析模块详细设计 (Config Module Details)

AI-Nexus 作为追求极致性能和灵活度的操作系统，其配置模块（Config Module）必须保证高度的**安全性**、**确定性**与**可扩展性**。
本模块设计全面承接并落实 `03_DATA_AND_CONFIG.md` 中提出的 **“读写绝缘法则” (Data Insulation Law)**，将系统配置严格拆分为两个边界清晰的维度：

1. **业务规则与策略 (`ainexus.yaml`)**
2. **环境与密钥池 (`.env`)**

### 5.1 配置文件详细设计 (Schema & Design)

#### 5.1.1 业务规则与策略 (`ainexus.yaml`)

**定位**：系统所有非涉密的基础规则与资源分配配额。随源码一并进行版本控制。
**解析格式**：YAML 格式。
**不可变性**：在运行期是只读的，禁止运行时服务（Inference/Evolution 等）直接修改此文件。

**Schema 核心字段设计**：

```yaml
system:
  rpc_timeout_ms: 5000       # ACP 控制总线最大超时等待 (ms)
  max_retries: 3             # 内部服务故障的最大重试次数

compute:
  max_inference_threads: 8   # 推理引擎允许启动的最大并行线程
  max_execution_workers: 4   # 技能执行器最大 Worker 数量

models:
  routing:
    local_model_token_limit: 4096    # 本地小模型的最大 Token 限额
    gemini_api_token_limit: 128000   # Gemini API 最大上下文大小
  global_assets:
    # global_gemini_model: "gemini-1.5-flash"  # Migrated to NexusDb Routing Table
    

```

#### 5.1.2 环境与密钥池 (`.env`)

**定位**：存放与底层应用强相关的系统级敏感认证信息（如 Gemini API Key）以及底层设施的绝对路径挂载点。**严格禁止进入版本控制库（.gitignore 必须包含此项）**。绝对禁止在 `.env` 中存放任何动态的 Channel 渠道信息（如 `telegram_bot_name`, `is_active`），这些必须由 Dashboard 在运行时存入 BlockStore。
**解析格式**：DotEnv (Key-Value) 格式。

**核心字段集**：
```env
# Secrets / Keys
    # Note: API Keys are stored dynamically in NexusDb Providers
    AINEXUS_ADMIN_PASSWORD=...
    
    # Mounts / Endpoints
AURA_CACHE_PATH=/var/lib/ainexus/cache
RUST_LOG=info,ainexus_kernel=debug
```

### 5.2 配置加载与解析机制 (Config Loader)

#### 5.2.1 依赖与技术栈选型

*   **反序列化库**: `serde` 和 `serde_yaml`、`serde_json` 提供强类型映射。
*   **分层加载引擎**: 推荐使用 `config-rs` 库（或手动封装合并逻辑），以支持多层级的合并策略（Defaults -> `.env` -> `.yaml` -> 环境变量覆盖）。

#### 5.2.2 加载优先级 (Priority & Overrides)

系统启动时，由 `ainexus-init` 或 Kernel 发起的配置加载必须遵循以下严格的覆盖规则（低优先级会被高优先级覆盖）：

1. **Hardcoded Defaults** (Rust 结构体中的 `Default` trait 实现)。
2. **`ainexus.yaml`** (业务基线配置)。
3. **`.env`** (注入敏感数据与环境挂载点)。
4. **系统环境变量 (OS Env Vars)** (拥有最高优先级，便于 Docker/K8s 容器化编排时动态覆盖，例如传入 `AINEXUS_COMPUTE_MAX_INFERENCE_THREADS=16`)。

#### 5.2.3 强类型绑定与单例共享

解析完成后，所有的配置项将被聚合到 `AiNexusConfig` 核心结构体中。该结构体在 `prepare` 阶段固化为 `Arc<AiNexusConfig>`，通过依赖注入（DI）传递给各个子系统和维面引擎，保证内存共享。

```rust
pub struct AiNexusConfig {
    pub system: SystemConfig,
    pub compute: ComputeConfig,
    pub models: ModelConfig,
    pub secrets: SecretsConfig, // 内部单独映射
}
```

### 5.3 动态热重载设计 (Hot Reloading)

对于 `ainexus.yaml`，系统设计为支持**不停机热重载**，避免打断当前进行的长效推理任务。

#### 5.3.1 触发机制
1. **基于操作系统的文件监控 (File Watcher)**：使用 Rust 的 `notify` 库监控配置目录的变化。
2. **SIGHUP 信号 (Unix Signal)**：通过接收操作系统的 `SIGHUP` 信号触发重载（工业标准方案）。

#### 5.3.2 热加载原子更新 (Atomic Swap)
配置被更新读取后，由于配置结构体已被 `Arc` 包装，我们采用 `arc-swap` 或 `RwLock<Arc<AiNexusConfig>>` 进行无锁或低延迟的原子替换。
新建立的子协程或新接入的请求直接读取新的 `Arc` 引用，旧有运行中协程继续持有旧配置的安全副本直至生命周期结束，保证运行时安全。

> [!IMPORTANT]
> **`.env` 的热重载限制**
>
> `.env` 文件通常在进程启动时由操作系统环境变量注入，在运行时强行热重载 `.env` 可能引发不可预知的基础设施状态冲突。如果管理员修改了 `.env`（如修改了全局 API Key 或数据库路径），必须发送 `SIGHUP` 触发**受控重启（Graceful Reload）**，而不能执行无感知热更新。

### 5.4 多租户隔离安全 (Tenant Isolation)

如 `03_DATA_AND_CONFIG.md` 规定，上述配置文件**仅仅用于存放系统的全局或兜底配置**。探索者（最终用户）通过系统生成的独立算力 Key、定制化模型选项，**严禁写回** `ainexus_secrets.json` 或 `ainexus.yaml`。

*   **隔离策略**：所有探索者的凭证属于**动态演进数据**，由系统接管，存储到 `.bin` 持久化块中。
*   **获取路径**：当用户的 Sandboxed Skill 运行时，配置加载器会在获取特定模型凭证时，优先使用传入的 `User Tag / Context` 去底座查询。若查询命中，则无视 Global Config，强制使用该用户的隔离凭证。
