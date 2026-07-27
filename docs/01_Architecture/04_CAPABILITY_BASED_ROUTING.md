# 基于算力 Profile 的动态任务路由 (Capability Profile-Based Routing)

## 1. 架构理念
在多 Agent 的复杂生态系统中，直接将特定 Agent 绑定到具体的物理大模型（如 `gemini-2.5-pro` 或 `gpt-4o`）会导致系统极度脆弱。
当模型 API 密钥耗尽、服务商宕机、或出现性价比更高的新模型时，硬编码的绑定关系将带来巨大的维护灾难。

AI-Nexus 采用了**算力 Profile 模式 (Capability Profile Mode)** 的路由架构：
将“业务/推理任务的需求 (Inference Task Type)”与“底层的计算资源”进行彻底解耦，通过算力 Profile 聚合多任务类型关联、容灾节点链与高级分流规则，实现统一的高可用调度。

---

## 2. 核心角色与数据结构

### 2.1 算力 Profile 结构 (`CapabilityProfile`)
策略节点由单一规则升级为 `CapabilityProfile` 结构，单个 Profile 可同时绑定多个任务类型，并包含高级模型调度规则：

```json
{
  "High-Reasoning-Profile": {
    "name": "深度智力与代码算力组",
    "description": "适用于高逻辑、复杂代码分析与多步骤推理任务",
    "task_types": ["Tier-1-Logic", "Code-Generation"],
    "primary": "claude-3-5-sonnet",
    "failover": ["gemini-2.5-pro", "gpt-4o"],
    "routing_rules": {
      "context_overflow_model": "gemini-1.5-pro",
      "max_token_threshold": 32768,
      "timeout_ms": 10000
    }
  },
  "General-Balanced-Profile": {
    "name": "通用对话与极速响应组",
    "description": "适用于日常对话、低延迟回复与轻量结构化提取",
    "task_types": ["Tier-2-Balanced", "Tier-3-Fast", "Structured-Output"],
    "primary": "gemini-2.5-flash",
    "failover": ["llama-3-8b"],
    "routing_rules": {
      "context_overflow_model": "gemini-1.5-flash",
      "max_token_threshold": 16384,
      "timeout_ms": 5000
    }
  }
}
```

### 2.2 推理任务类型 (`InferenceTaskType`)
系统内置并扩展了标准推理任务分类：
- `Tier-1-Logic` / `Logic`: 深度逻辑推理与复杂解析
- `Code-Generation` / `Code`: 代码编写、重构与 Review
- `Tier-2-Balanced` / `Balanced`: 常规通用对话
- `Tier-3-Fast` / `Fast`: 极速低延迟响应
- `Multimodal-Vision` / `Vision`: 多模态视觉图像处理
- `Structured-Output` / `Structured`: 强类型 JSON 与模式提取

---

## 3. 模型调度算法与错误处理规则

### 3.1 Profile 匹配与长文本分流逻辑
当 Agent 或 API 发起推理请求时：
1. **多任务 Profile 匹配**：`ModelRouter` 遍历路由表中的各个 `CapabilityProfile`，优先查找 `task_types` 数组包含当前任务类型的 Profile（或 key 相符）。
2. **长文本 Token 溢出评估**：若传入预估 Token 长度 `estimated_tokens` 且配置了 `routing_rules.max_token_threshold`：
   - 当 `estimated_tokens > max_token_threshold` 且配置了 `context_overflow_model` 时，系统自动将主模型切换重定向至该长文本专属模型（如 `gemini-1.5-pro`），并标记 `is_context_overflow = true`。

### 3.2 严格无静默保底 (No Silent Fallback)
为了防止隐式软失败掩盖策略缺失错误，系统**去除了静默保底降级逻辑**：
- 当在传入配置和默认配置中均无法找到与目标任务相吻合的 Profile 或 Profile 配置无效时，`route_task` 将明确返回 `Err(ModelRouterError::NoMatchingRoute)`。
- 在 REST API 处拦截该异常并响应 `HTTP 400 Bad Request`，以便开发运维及时定位缺失的算力组。

---

## 4. 服务商凭证池 (Providers Registry)

系统通过 `NexusDb` 的 `providers` 集合动态统一管理 API Key。

### 4.1 数据结构
包含 `id`, `name`, `api_key`, `base_url` 4 个核心属性。向前端展示时自动掩码脱敏（如 `sk-****`）。

### 4.2 密钥加载流
当 `ModelRouter` 完成模型路由后，底层 `GeminiClient` 动态查询 `providers` 集合提取生效中的凭证并进行请求组装，彻底解耦环境变量。
