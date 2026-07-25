# Main Agent 设计规范 (Main Agent & Control Specification)

在多智能体 (Multi-Agent) 协同的架构中，为了避免指令混乱和资源争抢，我们需要引入一个具备调度职责的中枢节点，即 **Main Agent（主控/总管家分身）**。

与作为“造物主”的 `Meta Agent` 相比，`Main Agent` 并不负责创造或修改底层代码，它是直接面向用户侧复杂需求的第一级承接者和任务路由中心。

## 1. 核心职责与定位

1.  **用户意图的统一入口**：当系统接入一个复杂的长期目标或宏观诉求时，首先由 Main Agent 承接，避免具体专业任务直接污染普通聊天分身的上下文。
2.  **全盘状态感知**：Main Agent 拥有透视当前系统内所有应用层分身（Sub-Agents）的权限。
3.  **任务拆解与分发 (Task Dispatching)**：它是系统的路由枢纽，负责将宏大的用户目标拆解为子任务，并委派给拥有对应能力的专业 Agent。

## 2. 主控类技能 (Control-Type Skills)

`Main Agent` 拥有专门的“主控类” `Meta/Control Skill` 白名单，这使得它区别于只专注于特定领域的底层执行分身：

### 2.1 查看与检索 Agent 列表 (Agent Registry Inspection)
*   **功能描述**：允许 Main Agent 实时读取当前系统中注册的所有 Agent 实例。
*   **获取信息**：能够获取每个 Agent 的唯一标识 (ID)、职责设定 (Persona)、当前挂载的技能列表 (Skill Whitelist) 以及当前的工作状态（空闲/忙碌/休眠/执行中）。
*   **意义**：通过知晓“手下有哪些兵”，Main Agent 才能做出准确的派发决策。

### 2.2 任务派发与委派 (Task Delegation)
*   **功能描述**：向指定的下游 Agent 异步或同步发送 `Task Payload`。
*   **工作流**：将用户的自然语言需求转换为结构化的指令（包含上下文、目标标准、截止要求等），唤醒目标 Agent 进入工作流。

### 2.3 状态监控与结果回收 (Status Monitoring & Aggregation)
*   **功能描述**：订阅下游 Agent 的任务执行进度事件，或者主动轮询健康状态。
*   **异常处理**：当下游 Agent 陷入死循环或报错时，Main Agent 负责拦截异常，选择重新派发任务或直接向用户汇报失败。
*   **汇总反馈**：收集多个子任务的结果，合并提炼后，形成连贯一致的回复输出给外部 Channel。

## 3. 多层级联动机制 (Multi-Tier Coordination)

Main Agent 在系统中的流转闭环如下：

1.  **感知与拆解**：用户发送请求：“帮我监控服务器状态并写一份周报，同时开发一个新的数据展示面板”。
2.  **检索与匹配**：Main Agent 动用主控技能查看 Persona 列表。发现系统中存在适合运维的 `Ops_Persona` 和适合数据分析的 `Data_Persona`。
3.  **发现缺失能力 (与 Meta Agent 的协同)**：Main Agent 发现列表中没有擅长开发面板的专门前端 Persona。此时，Main Agent 可以向上级或者通过特定事件机制，**触发 Meta Agent 介入**。
    *   *Meta Agent 启动，生成一个新的 `Frontend_Persona` 并分配 `Tier-1-Logic` 算力标签。*
4.  **下发与追踪**：新 Persona 就绪后，Main Agent 实例化出对应的 Agent，将“监控”、“报告”、“开发”三个子任务分别委派给它们执行。
5.  **验收闭环**：全部子 Agent 提交结果后，Main Agent 整合汇报给用户。
