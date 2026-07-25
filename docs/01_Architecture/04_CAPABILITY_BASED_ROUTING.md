# 基于能力标签的混合路由 (Capability-Based Hybrid Routing)

## 1. 架构理念
在多 Agent 的复杂生态系统中，直接将特定 Agent 绑定到具体的物理大模型（如 `gemini-2.5-pro` 或 `gpt-4o`）会导致系统极度脆弱。
当模型 API 密钥耗尽、服务商宕机、或出现性价比更高的新模型时，硬编码的绑定关系将带来巨大的维护灾难。

**混合路由架构** 的核心在于**解耦**：
将“业务对算力的需求”与“实际的算力提供方”解耦，通过中间的“能力标签 (Capabilities)”进行撮合。

## 2. 角色分工

### 2.1 Agent 层 (消费者)
Agent 的定义 (在 Agent Factory 中配置) 不再包含 `model_name` 字段。
取而代之的是 `capability_requirement`（能力需求标签）。
常见的标签定义：
- `Tier-1-Logic` (最高级别的逻辑推理与代码生成能力)
- `Tier-2-Balanced` (均衡的成本与性能，适用于常规任务)
- `Tier-3-Fast` (极速响应，适用于闲聊或简单的文本提取)
- `Multimodal-Vision` (多模态视觉处理能力)

### 2.2 算力中心 / Model Router (调度者)
全局统一定义“能力标签”到“物理模型”的映射规则池。
- **Primary / Failover 路由**: `Tier-1-Logic` 的 Primary 可以是 `gemini-2.5-pro`，Failover 可以是 `gpt-4o`。
- **额度与成本控制**: 监控全局 Token 开销，当 `Tier-1` 额度耗尽时，Model Router 有权全局降级路由，而无需修改任何底层 Agent 的配置。

## 3. 契约定义 (API Data Structures)

### 3.1 Agent Payload
```json
{
  "id": "coder_agent_01",
  "name": "Senior Coder",
  "persona": { ... },
  "capability_requirement": "Tier-1-Logic" // 替代原有的 model: "gemini-2.5-pro"
}
```

### 3.2 Model Router 映射配置 (Routing Table)
```json
{
  "Tier-1-Logic": {
    "primary": "gemini-2.5-pro",
    "failover": ["gpt-4o", "claude-3-5-sonnet"]
  },
  "Tier-3-Fast": {
    "primary": "gemini-2.5-flash",
    "failover": ["llama-3-8b"]
  }
}
```

## 4. 优势
- **容灾能力**: 极速切换宕机节点。
- **统一计费**: 在算力中心层面统一限流与成本审计。
- **职责分离**: 业务侧专心定义 Agent 性格与提示词，运维侧专心调配算力资源。

## 5. 服务商凭证池 (Providers Registry)
为了彻底解耦底层环境变量，系统采用数据库动态管理 API Key 的机制。

### 5.1 数据结构
提供商包含四个字段：`id`, `name`, `api_key`, `base_url`。
后端存储时，`api_key` 为敏感数据。当向前端提供时，后端执行脱敏操作（如 `sk-****`）。

### 5.2 API Key 读取逻辑
当 Model Router 在路由到具体模型时，会实时查询凭证池中对应的 Provider，并使用明文 API Key 构造 HTTP 客户端。
