# AI-Nexus 仪表盘全矩阵深度测试清单 (Enterprise QA Matrix)

本文档是针对 `AI-Nexus` 前端与 API 交互的**企业级自动化与手动测试基准 (Test Standard)**。包含了从像素级 UI、微动效、无障碍访问 (A11y) 到后端状态机、并发安全、性能分析的全面测试矩阵。

---

## 1. UI 渲染与排版一致性 (Pixel-perfect UI & Layout)

### 1.1 全局布局 (Global Layout)
- [ ] T-UI-001: 验证主色调 `--primary-color` (如 `#3b82f6`) 是否准确应用于所有主要按钮、进度条高亮区域。
- [ ] T-UI-002: 验证辅助色 `--accent-color` (如 `#8b5cf6`) 和 `--secondary-color` (如 `#10b981`) 在对应 KPI 边框、图表的高亮是否一致。
- [ ] T-UI-003: 验证背景色使用暗黑/毛玻璃风格 (`rgba(255,255,255,0.05)`)，在不同层级容器间是否有足够的对比度。
- [ ] T-UI-004: 验证系统默认字体堆栈（如 `Inter`, `Roboto`, `sans-serif`），字母间距 (letter-spacing) 和行高 (line-height) 是否符合可读性规范。
- [ ] T-UI-005: 验证所有 Panel 和 Card 拥有统一的圆角 `borderRadius: 8px` 及柔和的边框 `1px solid var(--surface-border)`。

### 1.2 响应式与跨端适配 (Responsive & Cross-Device)
- [ ] T-RS-001: 浏览器宽度缩放至 `1920x1080`（全高清），侧边栏占据固定宽度（如 `240px`），主内容区充分舒展，图表不拉伸失真。
- [ ] T-RS-002: 浏览器宽度缩放至 `1280x720`（常见笔电），`kpi-grid` 必须正常使用 `grid-template-columns: repeat(auto-fit, minmax(240px, 1fr))` 自动收缩。
- [ ] T-RS-003: 浏览器宽度缩放至 `768x1024`（平板竖屏），Dashboard 下方的双栏布局 (`2fr 1fr`) 必须折叠为上下单栏布局 (`1fr`)。
- [ ] T-RS-004: 浏览器宽度缩放至 `375x667`（移动端），侧边栏是否被隐藏（Hamburger Menu）或者被移动到屏幕底部导航。
- [ ] T-RS-005: 移动端下，所有表格必须允许横向滑动 (overflow-x: auto) 或将列隐藏/折叠，绝对不能引发整个页面的横向滚动条。
- [ ] T-RS-006: 验证页面是否声明了正确的 `<meta name="viewport" content="width=device-width, initial-scale=1">`。

---

## 2. 无障碍与键盘交互 (A11y & Keyboard Navigation)

### 2.1 键盘导航 (Focus & Tab Index)
- [ ] T-A11Y-001: 不使用鼠标，仅按 `Tab` 键，焦点必须依次经过侧边栏链接、主区主要按钮、表单输入框。
- [ ] T-A11Y-002: 验证元素被 `Focus` 时，必须带有明显的聚焦环 (如 `outline: 2px solid var(--primary-color)`)。
- [ ] T-A11Y-003: 验证按下 `Enter` 键能够触发拥有 `Focus` 的按钮（如 "保存设置"、"刷新"）。
- [ ] T-A11Y-004: 验证按下 `Space` 键能够触发 `Checkbox`、`Toggle` 开关（网关开关、脱敏开关）。

### 2.2 屏幕阅读器语义 (ARIA & Semantic HTML)
- [ ] T-A11Y-005: 验证 Header 必须使用 `<h1>` 或 `<h2>` 标签，不能仅仅是用 `span` 加粗。
- [ ] T-A11Y-006: 验证所有交互性 Icon（如“X”删除按钮），必须带有 `aria-label="Delete Session"`。
- [ ] T-A11Y-007: 验证网关切换按钮，当前状态若是 Idle，必须具有 `aria-checked="false"` 和 `role="switch"`。
- [ ] T-A11Y-008: 验证色彩对比度：所有的浅灰色说明文字 `var(--text-secondary)` 与深色背景的对比度需符合 WCAG 2.1 AA 级标准 (>= 4.5:1)。

---

## 3. 前端交互动效与微状态机 (Micro-interactions)

### 3.1 悬停与点击反馈 (Hover & Active)
- [ ] T-MI-001: 鼠标进入 `Dashboard` KPI 卡片，监听 `mouseenter` 事件，必须在 0.3s 内触发 `transform: scale(1.2) rotate(5deg)`，动画必须无卡顿 (Hardware Acceleration)。
- [ ] T-MI-002: 鼠标快速滑过多个 KPI 卡片，`mouseleave` 触发时必须防抖或平滑归位，不可发生动画冲突抖动。
- [ ] T-MI-003: 悬停于表格内的 Action Button (Eye, XCircle, Download)，按钮背景变亮，鼠标指针变更为 `cursor: pointer`。
- [ ] T-MI-004: 左键按住 (Active) Button，按钮元素应缩放 (`transform: scale(0.95)`) 以模拟物理按压感。

### 3.2 加载动画与骨架屏 (Loading States)
- [ ] T-MI-005: API 请求发出且耗时 > 300ms 时，列表/数据区域必须展示 Spinner 或骨架屏，禁止显示空白导致跳屏 (Layout Shift)。
- [ ] T-MI-006: Sessions 页面的 `刷新` 按钮在加载期间，其内部图标必须带有持续的 `spin 1s linear infinite` 动画。
- [ ] T-MI-007: Settings 页面的保存按钮在 `Promise.pending` 期间应显示 `Loading...`，并 `disabled` 按钮防止重复点击。

---

## 4. 核心端对端功能 (End-to-End Business Flows)

### 4.1 仪表盘数据一致性 (Dashboard Flow)
- [ ] T-FLOW-001: `GET /api/dashboard/stats` 返回的 `active_sessions` 数字，必须与 `GET /api/sessions` 列表中的数据条目总数保持最终一致。
- [ ] T-FLOW-002: Token 消耗趋势图表中渲染的最高条形图 (Bar)，其高度百分比必须被严格计算为 `100%`。
- [ ] T-FLOW-003: Token 趋势柱状图的 CSS 高度过渡动画 (`transition: height 0.5s`) 在数据加载完成后必须顺利执行，不是瞬间出现。
- [ ] T-FLOW-004: 仪表盘上的“网关实时节点”简报，其数据状态必须与独立网关页 (`/gateways`) 完全一致。

### 4.2 会话管理流转 (Session Management Flow)
- [ ] T-FLOW-005: 识别返回的 Session Source：包含 "Telegram" 渲染为蓝标，"Slack" 为红标，否则为绿标。如果 source 内容出现意外字符（如 `Null`, `Undefined`, `[]`），必须平稳降级到默认绿标，不能触发 JS `Cannot read property 'includes' of null` 错误崩溃。
- [ ] T-FLOW-006: 会话终止请求：点击 XCircle -> `DELETE /api/sessions/{id}` -> 后端成功移除状态管理器内容 -> 前端 `.then()` 中再次发起 `fetchSessions()` -> 渲染最新列表。
- [ ] T-FLOW-007: 验证 URI 编码安全：如果 Session ID 包含特殊字符（如 `#sess_9a8b7c` 或 `+/?`），`encodeURIComponent(id)` 必须被正确调用以防止 400 Bad Request。

### 4.3 网关配置控制 (Gateway Control Flow)
- [ ] T-FLOW-008: 发起网关切换请求时，前端必须先使用**乐观更新 (Optimistic Update)**：先在 UI 将绿标变为红标。
- [ ] T-FLOW-009: 如果 `POST /api/gateways/{id}/toggle` 返回非 2xx 响应，前端的 `catch` 逻辑必须包含**状态回滚 (Rollback)**，将红标变回绿标，并弹出错误 Toast。
- [ ] T-FLOW-010: 验证在另一个浏览器标签页打开网关页，当该标签页刷新时，应正确反映刚刚在第一个标签页切换的网关状态。

### 4.4 系统配置读写 (Settings Pipeline)
- [ ] T-FLOW-011: 验证 `SettingsDTO` 的 `session_timeout_ms` 字段从后端传来时是 `number` 类型。
- [ ] T-FLOW-012: 在前端输入框输入字母，利用 `parseInt(e.target.value) || 0` 必须将非法字符拦截并 fallback 为 `0`。
- [ ] T-FLOW-013: 验证 `log_masking` 的前端开关切换，`PUT /api/settings` Payload 中必须输出严格的 Boolean 类型 (`true` / `false`)，不可是字符串 `"true"`。
- [ ] T-FLOW-014: 提交 Settings 时，打开 Chrome DevTools，验证 Content-Type 必须是 `application/json`。

---

## 5. 异常、边界与破坏性测试 (Error, Boundary & Chaos)

### 5.1 数据类型污染与边界保护 (Data Fuzzing & Boundaries)
- [ ] T-BD-001: 修改 `api.rs` 的 ledger 返回，故意在 `input_tokens` 传入极大整数 (`9999999999999999999`)，前端的 `toLocaleString()` 是否能正确转换，且不引发溢出截断。
- [ ] T-BD-002: 修改 `api.rs`，让 `est_cost_usd` 返回带 10 位小数的浮点数，验证前端 `.toFixed(3)` 是否正确四舍五入。
- [ ] T-BD-003: 修改后端让某列表接口返回空数据 `[]`，验证前端页面是否显示友好的 "No data available" 等空状态，而非仅仅是空白或表头。
- [ ] T-BD-004: 修改后端让某接口不返回 `gateways` 这个 key 而是返回 undefined，验证解构渲染时的 `stats.gateways.map` 抛出异常被 ErrorBoundary 拦截，不会导致白屏 (White Screen of Death)。

### 5.2 网络断网与高延迟 (Network Emulation)
- [ ] T-NT-001: 使用 DevTools -> Network 开启 `Slow 3G` (高延迟)，进入 Sessions 页面。验证 Loading 动画是否持续稳定显示。
- [ ] T-NT-002: 在 `Slow 3G` 期间，迅速双击 "刷新" 按钮，验证请求不被堆叠重复发送（锁机制）。
- [ ] T-NT-003: 使用 DevTools -> Network 开启 `Offline` (无网络)。进入任何页面，确认 `fetch().catch()` 能够捕获 `TypeError: Failed to fetch`。
- [ ] T-NT-004: 网络恢复后，点击“刷新”按钮应能重新连接成功并渲染页面。

### 5.3 后端并发与锁竞争 (Race Conditions)
- [ ] T-RC-001: 编写外部压测脚本，对 `POST /api/gateways/Web Widget/toggle` 以 100 QPS 持续请求。同时在浏览器前端刷新页面。
- [ ] T-RC-002: 观察后端终端日志，由于使用了 `Arc<RwLock>`，确认不会发生读写锁死锁 (Deadlock) 或线程恐慌 (Panic)。
- [ ] T-RC-003: 在压测期间，验证前端界面的状态切换是否表现出跳变，最终状态是否与后端的真实状态严格一致。

---

## 6. 安全性与注入防护 (Security & Vulnerabilities)

### 6.1 XSS 反射与存储型注入 (XSS Defenses)
- [ ] T-SC-001: 修改后端的 mock 数据，在 `DashboardStats.active_sessions_trend` 中插入 Payload: `<img src="x" onerror="alert('XSS')">`。
- [ ] T-SC-002: 前端加载该数据时，由于 React 默认的 JSX 文本转义机制，该字符串应当被作为纯文本渲染输出到屏幕，而不是被当做 DOM 注入。
- [ ] T-SC-003: 检查所有组件中是否存在非法的 `dangerouslySetInnerHTML` 调用。如果存在，必须经过 `DOMPurify` 过滤。

### 6.2 敏感数据保护 (Sensitive Data)
- [ ] T-SC-004: 验证 `GET /api/settings` 和其他网络请求中，不会意外泄露服务器上的实际环境变量、API Key 等不该由前端获取的信息。

---

## 7. 性能与加载指标 (Performance & Core Web Vitals)

### 7.1 LCP 与 FCP
- [ ] T-PF-001: FCP (First Contentful Paint)：关闭网络缓存重新加载页面，页面主体背景与 Header 框架需在 500ms 内出现。
- [ ] T-PF-002: LCP (Largest Contentful Paint)：页面最大的元素（如 Token 柱状图或主数据卡片）完全加载并在屏幕内渲染的时间需控制在 1.2s 内。

### 7.2 内存泄漏检测 (Memory Leaks)
- [ ] T-PF-003: 在前端不断点击左侧菜单在所有路由之间高频来回切换 50 遍。
- [ ] T-PF-004: 打开 Chrome Task Manager 或 Memory Profiler，验证页面的 JS Heap Size 不会呈无限线性增长，旧页面的 React DOM 组件能被正确卸载并回收垃圾 (Garbage Collection)。
- [ ] T-PF-005: 确认 `useEffect` 中的事件绑定（如 `Dashboard.tsx` 中绑定到 KPI 卡片上的 `mouseenter` 和 `mouseleave` 监听器）在其 `return` 函数中被正确解绑。如果解绑失败将导致游离 DOM 节点 (Detached DOM Elements)。
