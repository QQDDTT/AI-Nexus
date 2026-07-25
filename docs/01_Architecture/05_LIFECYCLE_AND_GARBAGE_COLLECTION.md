# 08 - 项目生命周期与垃圾回收机制

本文档详细描述了 AI-Nexus 平台在运行过程中的生命周期管理及相应的垃圾回收（Garbage Collection, GC）机制，以确保系统在长期运行下的稳定性、内存安全及资源的高效利用。

## 1. 生命周期管理 (Lifecycle Management)

### 1.1 内存生命周期 (Memory Lifecycle)
内存的生命周期严格遵循“按需分配、及时释放”的原则，主要划分为以下阶段：
- **初始化与分配**：在会话建立或请求到达时，系统为 Context（上下文）、缓存对象（如图片、文档等二进制数据）分配内存池。
- **活跃使用期**：伴随着 Agent 的推理、大模型路由调用以及数据在流水线中的传递，内存被持续访问。为了避免频繁分配导致碎片化，高频访问的数据结构采用对象池 (Object Pool) 模式。
- **降级与冻结**：当会话处于空闲状态（Idle）超过特定阈值，相关内存对象（如非活跃对话的历史记录）会被序列化并下沉至磁盘或持久化缓存层，从而释放宝贵的物理内存。
- **释放**：会话彻底结束或触发系统资源告警时，强制进行反初始化，相关内存将被移交至底层运行时的 GC 管控区。

### 1.2 进程生命周期 (Process Lifecycle)
整个 Nexus 系统的进程管理采用主从架构（Master-Worker）：
- **启动与预热 (Startup & Warm-up)**：主进程 (Master) 负责加载核心配置、初始化网关监听端口及拉起监控探针。随后按需 Fork 出工作进程 (Worker) 进行连接预热。
- **健康运行 (Healthy Running)**：Worker 进程处理实际的路由分发和 Agent 调度。主循环通过心跳 (Heartbeat) 和内部 Channel (mpsc) 监控所有模块的状态。
- **平滑重启 (Graceful Reload)**：当检测到配置热更新或某个 Worker 进程出现内存泄漏预兆时，主进程会拦截新请求并路由给新的 Worker，旧 Worker 在处理完当前排队的请求后自行退出。
- **终止 (Termination)**：进程收到 SIGTERM 信号后，首先切断网关输入，完成现有队列的 Token 结算与日志刷盘，最后释放所有网络 Socket 句柄后退出。

### 1.3 交互生命周期 (Interaction Lifecycle)
用户或客户端与系统的一次交互完整流转如下：
- **接入 (Connect)**：客户端通过渠道网关（如 Telegram、Web Widget）建立连接，生成全局唯一的 `TraceID`。
- **推理与多路复用 (Inference & Multiplexing)**：用户输入被解析，路由中心根据负载均衡及 Token 预算，分配给合适的 Agent/大语言模型进行处理。
- **流式返回 (Streaming Output)**：采用 SSE/WebSocket 方式，将生成的 Token 实时回传给客户端。
- **结算与归档 (Settlement & Archiving)**：交互结束，Token Ledger 完成扣费，日志和会话状态被异步投递至消息队列等待持久化。

---

## 2. 垃圾回收机制 (Garbage Collection)

为了防止系统随时间推移产生资源耗尽，系统设计了多层次的异步 GC 策略。

### 2.1 后台进程的垃圾回收 (Background Process GC)
后台进程（如下载任务、长效工具调用、异步日志刷盘等）如果管理不善，极易成为“僵尸进程”。
- **超时强杀 (Timeout Reaper)**：系统后台运行一个守护线程（Reaper Task），定期扫描所有后台任务的注册表。若发现执行时间超过系统设定的 `max_execution_time` 且未返回心跳的进程，将被发送 SIGKILL 强制回收。
- **孤儿资源清理**：当某个 Worker 进程异常崩溃（Crash）时，它所持有的临时文件、锁以及未完成的异步句柄，将由主进程的 Watchdog 机制在下一个周期内统一清理。
- **连接池收缩**：BlockStore 文件句柄和上游 API 的连接池会根据当前的 QPS 自动伸缩。空闲超过规定时间的连接将被自动关闭并回收。

### 2.2 Agent 记忆的垃圾回收 (Agent Memory GC)
Agent 在与用户多轮对话中会积累庞大的上下文和记忆库，这是导致 OOM（内存溢出）的核心隐患之一。
- **滑动窗口清理 (Sliding Window)**：Agent 的短期工作记忆 (Working Memory) 采用 Token 计数或轮数限制。超出窗口大小的早期记忆会被自动裁剪（Eviction），确保发送给大模型的 Prompt 始终在最大上下文限制内。
- **记忆摘要化 (Memory Summarization)**：对于被移出滑动窗口的记忆，系统不会直接丢弃，而是触发一个后台的轻量级 Summarizer 模型，将其压缩为一段高度概括的摘要，保留核心意图后存储为长期记忆（Long-term Memory）。
- **过期记忆淘汰 (TTL-based Pruning)**：存储在向量数据库或 KV 存储中的长期记忆，均带有 TTL（Time To Live）标签。对于用户超过 30 天未激活或已被判定为“无价值”的临时上下文，系统将在夜间低峰期（Maintenance Window）执行批量的 Delete 操作，释放存储空间。
