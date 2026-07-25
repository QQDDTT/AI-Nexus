# 10.8 仪表盘画面分析与通讯需求规范

基于对当前系统 `frontend/src/pages/` 目录下所有页面的分析，本文档总结了每个页面的 UI 内容，并将所需的前后端通讯接口需求整理为表格，为后续全面打通数据提供参考。

## 1. 平台概览 (Dashboard.tsx)
**画面内容**：
- **KPI 指标卡片**：当前活跃会话数、今日消耗 Total Tokens、系统心跳 API 健康度。
- **图表展示**：近 7 天 Token 消耗趋势模拟图。
- **实时节点列表**：展示目前接入的各个网关（如 Telegram Bot, Slack App, Web Widget）及其在线状态。

## 2. 渠道网关 (Gateways.tsx)
**画面内容**：
- **外部节点列表**：列出 Telegram、Slack、WhatsApp 等服务网关。
- **指标统计**：每个网关在过去 24 小时的**请求总数**以及**平均延迟 (ms)**。
- **控制操作**：配置网关、启/停网关连接（如开启/关闭轮询）、新增网关按钮。

## 3. 算力中心 (ModelRouter.tsx)
**画面内容**：
- **主路由策略 (Primary)**：当前采用的路由策略（如成本优先），及各模型（Gemini 1.5 Pro, Claude 3）分担的流量权重百分比和进度条可视化。
- **备用节点 (Failover)**：备用提供商（OpenAI, Groq）的接入情况及 Standby 状态。
- **控制操作**：调整权重、添加提供商、启停节点。

## 4. 活动会话 (Sessions.tsx)
**画面内容**：
- **表格呈现**：实时显示正在进行的 Agent 对话上下文，包含 `Session ID`、`User / Source` (请求源)、被路由到的 `Model`、已消耗的 `Tokens`、当前所处 `Status` (Processing, Waiting 等)。
- **控制操作**：刷新列表、筛选、查看详细对话历史 (Eye)、强制终止会话 (XCircle)。

## 5. 核心配置 (Settings.tsx)
**画面内容**：
- **系统与存储**：BlockStore 挂载目录 (data_dir)、全局会话超时时间 (ms)、启动本地日志脱敏开关。
- **侧边栏导航**：支持切换至“安全&鉴权”、“个性化(UI)”。
- **控制操作**：保存设置。

## 6. 账单明细 (TokenLedger.tsx)
**画面内容**：
- **历史明细表**：详细的单次请求成本追踪，包括 `Time`, `User ID`, `Model`, 分离的 `Input Tokens` 和 `Output Tokens`，以及最终的 `Est. Cost` (预估美元花费)。
- **控制操作**：导出为 CSV 格式。

---

## 7. 通讯接口需求统计总表

| 模块名称 | HTTP Method | API 路由 | 描述与用途 | 状态 |
| :--- | :--- | :--- | :--- | :--- |
| **平台概览** | `GET` | `/api/dashboard/stats` | 获取活跃会话、Tokens 概况、API 健康度及网关状态列表 | 已实现 (Mock Token) |
| **平台概览** | `GET` | `/api/dashboard/token-trend` | 获取近 7 天 Token 消耗趋势数据（用于图表渲染） | 待实现 |
| **渠道网关** | `GET` | `/api/gateways` | 拉取详细的网关列表及统计指标（状态、请求数、延迟） | 待实现 |
| **渠道网关** | `POST` | `/api/gateways/{id}/toggle` | 启停特定的网关进程（连接/断开） | 待实现 |
| **渠道网关** | `POST/PUT`| `/api/gateways` | 新增、修改网关配置 | 待实现 |
| **算力中心** | `GET` | `/api/models/routing` | 获取目前的基于能力标签的映射表 | 已实现 |
| **算力中心** | `PUT` | `/api/models/routing` | 更新映射表主路由与流量分配权重 | 已实现 |
| **算力中心** | `GET` | `/api/models/providers` | 获取数据库中所有的服务商凭证列表（脱敏） | 已实现 |
| **算力中心** | `POST` | `/api/models/providers` | 新增一个服务商凭证池 | 已实现 |
| **算力中心** | `DELETE` | `/api/models/providers/:id` | 从凭证池吊销指定的凭证 | 已实现 |
| **活动会话** | `GET` | `/api/sessions` | 分页或全量拉取当前内存中的活跃 Session 列表及状态枚举 | 待实现 |
| **活动会话** | `GET` | `/api/sessions/{id}` | 查看具体 Session 的 trace 日志及对话上下文详情 | 待实现 |
| **活动会话** | `DELETE`| `/api/sessions/{id}` | 强制回收或终止指定的对话会话 | 待实现 |
| **核心配置** | `GET` | `/api/settings` | 加载后台全局配置 (YAML/JSON)、存储设置及热力学参数 | 待实现 |
| **核心配置** | `PUT` | `/api/settings` | 保存系统配置，触发热重载 (Hot Reload) 或持久化保存 | 待实现 |
| **账单明细** | `GET` | `/api/ledger` | 分页获取来自 BlockStore 的追加写入历史消费流水和预估成本 | 待实现 |
| **账单明细** | `GET` | `/api/ledger/export` | 请求生成并下载包含账单明细的 CSV 文件流 | 待实现 |

> [!IMPORTANT]
> 此表格归纳了全平台彻底动态化所需的全部后台接口规范。前端使用 `fetch` 向代理 `/api` 发起请求即可与后端 Rust Axum Server 进行对接。
