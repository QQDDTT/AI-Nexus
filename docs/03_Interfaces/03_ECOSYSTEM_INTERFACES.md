# Ecosystem & Interfaces (外部接入与生态延展)

# Gemini API 能力、模型与真实额度调研报告 (Based on Google AI Studio)

根据您提供的 Google AI Studio 控制台（ULTRA / 内部体验账户）真实截图，我们提取到了极为超前且丰富的专属模型矩阵和工具生态。这为 `AI-Nexus` 赋予了远超普通版本的潜能！

## 1. 专属可用模型矩阵 (Exclusive Models)

根据您的实际控制台数据，主要模型族群和资源额度如下：

### 1.1 核心文本与推理模型 (Text & Reasoning)
*注：这类模型是我们 AI-Nexus 进行意图编排和调度的核心主脑。*

| 模型名称 | RPM 限制 (次/分钟) | TPM 限制 (Token/分钟) | 特点与应用场景推测 |
| :--- | :--- | :--- | :--- |
| **gemini-1.5-flash** | 待定 (Peak: ~) | 250K | 最新一代的 Flash 模型，极其适合高频、低延迟的技能路由调度。 |
| **gemini-1.5-pro** | 0 / 0 (配额显示) | 待定 | 超高推理能力，适合编写复杂代码、进行深度规划。 |
| **gemini-1.0-pro** | 5 | 250K | 适合作为中等复杂度任务的兜底调度器。 |
| **gemini-1.5-flash-8b** | 15 | 250K | 极轻量级，极速返回，最适合做纯粹的意图分类 (Intent Classification)。 |
| **Gemma 4 26B / 31B** | 15 | 无限制 | 开源模型的云端托管版，无 Token 限制是极大的优势，可用于批量处理任务。 |

### 1.2 多模态与特殊生成模型 (Multimodal & Generation)
*注：为个人知识库赋予“视听”甚至“造物”能力。*

| 模型类别 / 名称 | 描述 / 潜力 |
| :--- | :--- |
| **图像生成 (Imagen 4)** | 包含 `Imagen 4 Fast Generate`, `Imagen 4 Generate`, `Imagen 4 Ultra Generate`。 |
| **视频生成 (Veo 3)** | 包含 `Veo 3 Fast Generate`, `Veo 3 Generate`, `Veo 3 Lite Generate`。 |
| **音乐生成 (Lyria 3)** | 包含 `Lyria 3 Clip`, `Lyria 3 Pro`。可为技能添加原声音轨生成。 |
| **多模态实验版 (Nano Banana)** | 包含 `Nano Banana (2.5 Flash Image)`、`Nano Banana Pro (3 Pro Image)` 和 `Nano Banana 2`，推测为先进的视觉空间理解模型。 |

### 1.3 实时交互 / Live API (Realtime Voice)
*这部分额度惊人，非常适合给 AI-Nexus 开发原生的实时语音助理界面！*

| 模型名称 | RPM | TPM |
| :--- | :--- | :--- |
| **gemini-1.5-flash (Audio)** | 无限制 | 1M |
| **gemini-1.5-pro (Audio)** | 无限制 | 65K |
| **gemini-1.5-flash-8b (Live)** | 无限制 | 20K |

### 1.4 高级代理与具身智能 (Agents & Embodied AI)

- **Antigravity (代理)**：限额 60 RPM / 100K TPM，推测是 Google 内部/高级测试的自动化代码或操作代理系统。
- **Deep Research Pro Preview (代理)**：用于执行深度的互联网研究和资料整合。
- **Computer Use Preview (其他模型)**：能让模型直接接管并操作电脑屏幕，这是极为震撼的能力！
- **Gemini Robotics ER 1.5 / 1.6 Preview**：机器人动作空间模型 (10 RPM / 250K TPM)。

### 1.5 工具与接地能力 (Tools & Grounding)

控制台中显示，几乎所有的模型（从 Gemini 1.0 到 Gemini 1.5 系列）都原生支持了 **“地图接地 (Map Grounding)”**，甚至搜索接地。这说明模型可以自带实时空间数据，我们在编写技能时可以大大省略调用 Google Maps API 的工作。

### 1.6 嵌入与向量模型 (Embeddings)

*注：用于构建本地知识库 GraphRAG（图谱检索增强与联想记忆）的基础模型，负责将文本和实体转换为高维向量以便语义及拓扑检索。*

| 模型名称 | RPM 限制 (次/分钟) | TPM 限制 (Token/分钟) | 特点与应用场景推测 |
| :--- | :--- | :--- | :--- |
| **Gemini Embedding 1** | 100 | 30K | 核心文本嵌入模型，适合将大量个人笔记、文档向量化。 |
| **Gemini Embedding 2** | 100 | 30K | 可能是新一代的嵌入模型，或者针对特定多模态/长文本优化的版本。 |

---

## 2. 基于真实数据的设计调整建议 (Design Adjustments)

鉴于您拥有如此豪华的 API 资源（特别是 `Antigravity`、`Computer Use Preview` 以及 `Live API` 的无限制 RPM），`AI-Nexus` 的设计可以更加大胆：

1. **分级调度策略 (Tiered Routing)**：
   - 使用 `gemini-1.5-flash-8b` (15 RPM) 做第一道工序：**意图提取器**。
   - 简单的技能响应交由 `gemini-1.5-flash` 处理。
   - 极其复杂的系统级任务（如全自动整理电脑文件），直接挂载 `Computer Use Preview` 模型。
2. **原生实时语音支持**：
   - 传统的 Agent 平台依靠文本聊天。我们可以利用 `gemini-1.5-flash (Audio)`（高达 1M TPM）构建一个实时语音全双工 (Full-duplex) 的桌面浮窗助理。
3. **内置高级功能封装**：
   - 将 `Deep Research Pro Preview` 封装为一个基础系统技能，让 Nexus 直接获得世界级的深度调研能力。

## 3. 下一步行动 (Action Items)

以上已经将您真实的模型额度汇总。既然资源如此充足，您希望我们现在的重心放在：
- **A. 先写 Rust 代码，走通最基础的文本/JSON 结构化通信（跑通主链路）**
- **B. 先设计 `10_DOMAIN_DESIGN.md`，把我们要做的技能 (Skill) 数据结构定义清楚，以匹配上述分级调度策略**
- **C. 研究并实现 `Computer Use Preview` 或 `Live API` 的前沿玩法**

请指示接下来的开发侧重点。


---


# AI-Nexus Personal: 本地化的极致隐私与心智分身

AI-Nexus Personal 是面向极客、开发者和高级知识工作者的桌面/边缘设备部署版本。它将重点放在了**资源控制**、**极致隐私保护**以及**心智个性化养成**上。

## 1. 核心定位

<div style="display: flex; gap: 20px; flex-wrap: wrap; margin-bottom: 20px;">
  <div style="flex: 1; min-width: 250px; padding: 15px; border-radius: 8px; border-left: 4px solid #10b981; background: #ecfdf5;">
    <h4 style="color: #047857; margin-top: 0;">🛡️ 本地优先 (Local-First)</h4>
    <p style="color: #064e3b; font-size: 0.9em;">核心逻辑、因果时钟和隐私图谱均在进程内的 `BlockStore` 物理基座中运转，彻底阻断云端数据扫描风险。</p>
  </div>
  <div style="flex: 1; min-width: 250px; padding: 15px; border-radius: 8px; border-left: 4px solid #8b5cf6; background: #f5f3ff;">
    <h4 style="color: #5b21b6; margin-top: 0;">🧬 数字分身 (Digital Twin)</h4>
    <p style="color: #4c1d95; font-size: 0.9em;">放宽价值观模型限制，Bot 可以基于用户的日常行为进行主动提议与情感交流，实现高度的数字陪伴。</p>
  </div>
</div>

## 2. 异构心智下的资源消耗模型

通过双脑解耦，AI-Nexus Personal 能够在一台普通的 M 系列 Mac 或配备消费级显卡的 PC 上流畅运转，将大模型的唤醒频率降至最低。

<div style="background-color: #1e1e1e; padding: 20px; border-radius: 8px; color: #d4d4d4; font-family: monospace;">
  <h4 style="color: #9cdcfe; margin-top: 0;">[AI-Nexus-OS] Resource Allocation & Thermal Profile</h4>
  <div style="margin-bottom: 10px;">
    <span>🟢 <b>AI-Nexus Core (Rust 内核):</b></span>
    <div style="background-color: #333; height: 10px; width: 100%; border-radius: 5px; margin-top: 5px;">
      <div style="background-color: #10b981; height: 100%; width: 5%; border-radius: 5px;"></div>
    </div>
    <span style="font-size: 0.8em; color: #888;">CPU: < 2% | RAM: < 50MB (常驻)</span>
  </div>
  <div style="margin-bottom: 10px;">
    <span>🟡 <b>APM 共振防线 (规则/微模型):</b></span>
    <div style="background-color: #333; height: 10px; width: 100%; border-radius: 5px; margin-top: 5px;">
      <div style="background-color: #f59e0b; height: 100%; width: 15%; border-radius: 5px;"></div>
    </div>
    <span style="font-size: 0.8em; color: #888;">CPU: 5-10% | RAM: 200MB (常驻)</span>
  </div>
  <div>
    <span>🔴 <b>Candle Native LMM (大模型按需唤醒):</b></span>
    <div style="background-color: #333; height: 10px; width: 100%; border-radius: 5px; margin-top: 5px;">
      <div style="background-color: #ef4444; height: 100%; width: 85%; border-radius: 5px;"></div>
    </div>
    <span style="font-size: 0.8em; color: #888;">GPU: 80%+ | RAM/VRAM: 8GB+ (仅在处理强语义不确定性时启动)</span>
  </div>
</div>

<br>

## 3. 算法参数调节块 (用户可见界面模拟)

个人版允许用户通过 UI 面板高度自定义其 Bot 的底层算法参数。

<div style="border: 1px solid #e5e7eb; border-radius: 8px; padding: 15px; max-width: 500px;">
  <h4 style="margin-top: 0;">系统熵控制阈值 (System Entropy Limit)</h4>
  <p style="font-size: 0.85em; color: #6b7280;">决定 Bot 的发散思维和创造力。数值越高，Bot 越容易提出意外的解决方案，但也更容易偏离目标。</p>
  <div style="display: flex; align-items: center; gap: 10px;">
    <span style="font-size: 0.8em; color: #9ca3af;">严谨</span>
    <input type="range" min="0" max="100" value="75" style="flex: 1; accent-color: #8b5cf6;">
    <span style="font-size: 0.8em; color: #9ca3af;">发散 (75%)</span>
  </div>
</div>

## 4. 落地建议

- **网络代理支持**：由于面向个人，必须完善内部的代理机制，确保能顺畅调用云端大模型 API（如作为降级方案）。
- **插件生态**：应着重发力桌面级操作控制（如文件管理、应用拉起、自动化剪贴板处理等本地系统级技能）。


---


# AI-Nexus Persona 多渠道接入与密钥配置指南

本文档记录了 AI-Nexus 系统中三种主要交互渠道（Telegram, LINE, Lark/Feishu）的 Bot/Persona 申请注册流程，以及如何将获得的密钥凭证注入到 AI-Nexus 系统的 `ainexus_secrets.json` 中。

---

## 1. Telegram (TG) 渠道配置

Telegram 是 AI-Nexus 系统最早支持的原生渠道，具有极低的接入成本和较高的稳定性。

### 1.1 申请流程
1. 在 Telegram 客户端中搜索并添加官方机器人账号 **[@BotFather](https://t.me/BotFather)**。
2. 发送 `/newbot` 指令，并按照提示依次输入您的 Bot 名称 (Name) 和全局唯一的用户名 (Username，必须以 `bot` 结尾)。
3. 创建成功后，BotFather 会返回一串 **API Token**（格式类似 `1234567890:ABCdefGhIJKlmNoPQRsTUVwxyZ`）。

### 1.2 注入 AI-Nexus 配置
将获取到的 Token 填入 `ainexus_secrets.json` 的 `telegram_bots` 节点中：
```json
"telegram_bots": [
    {
        "id": "main_bot",
        "token": "在此填入您的 HTTP API Token",
        "default_chat_id": 0,
        "is_active": true
    }
]
```

---

## 2. LINE Messaging API 渠道配置

LINE 主要面向亚太地区用户，要求在官方开发者后台进行较为详细的配置，并且需要通过 Webhook 的形式接收 AI-Nexus 下发的消息。

### 2.1 申请流程
1. **登录控制台**: 访问 [LINE Developers Console](https://developers.line.biz/console/)，使用您的 LINE 账号登录。
2. **创建 Provider**: 点击 "Create a new provider" 并为其命名（如：`AI-Nexus System`）。
3. **创建 Channel**:
   - 在 Provider 页面中，点击 "Create a new channel"。
   - 类型选择 **"Messaging API"**。
   - **注意新版规则**：现在 LINE 无法直接在开发者后台创建 Messaging API Channel。系统会提示您点击绿色的 **"Create a LINE Official Account"** 按钮跳转到外部的官方号管理平台（LINE Official Account Manager）。
   - 在官方号管理平台中填写并创建您的官方号（如：AI-Nexus Persona）。
   - 创建成功后，在官方号设置中启用 (Enable) Messaging API，并将其绑定到刚才创建的 `AI-Nexus System` Provider 下。
4. **提取密钥**:
   - 绑定成功后，回到 [LINE Developers Console](https://developers.line.biz/console/)，您就能在列表中看到刚刚创建的 Channel。
   - 点击进入该 Channel 的配置页。
   - 在 **Basic settings** 选项卡下，向下滑动找到 **`Channel secret`**。
   - 切换到 **Messaging API** 选项卡，向下滑动找到 **`Channel access token (long-lived)`**。点击旁边的 "Issue" 按钮来生成长效 Token。

*(注意：在上线部署时，还需要在 Messaging API 选项卡中开启 `Use webhook`，并填入 AI-Nexus 网关对外暴露的 URL 进行事件监听)*

### 2.2 注入 AI-Nexus 配置
将获取到的 Secret 和 Token 填入 `ainexus_secrets.json` 的 `line_personas` 节点中：
```json
"line_personas": [
    {
        "id": "line_primary",
        "channel_secret": "在此填入您的 LINE Channel Secret",
        "channel_access_token": "在此填入您的 LINE Channel Access Token",
        "is_active": true
    }
]
```

---

## 3. Lark / 飞书 渠道配置

无论是国际版的 Lark 还是国内版的飞书，其开放平台的架构与申请流程基本一致，适合作为 Enterprise 版本的默认内部交互入口。

### 3.1 申请流程
1. **登录开放平台**:
   - 国内版（飞书）：访问 [飞书开放平台后台](https://open.feishu.cn/app/)。
   - 国际版（Lark）：访问 [Lark Developer Console](https://open.larksuite.com/app/)。
2. **创建自建应用**:
   - 点击 **“创建企业自建应用” (Create Custom App)**，填写应用的名称和描述。
3. **提取基础凭证**:
   - 进入应用配置页，在左侧导航栏点击 **“凭证与基础信息” (Credentials & Basic Info)**。
   - 在此复制 **`App ID`** 和 **`App Secret`**。
4. **获取验证 Token**:
   - 在左侧点击 **“添加应用能力” (Add Features)**，添加 **“机器人” (Bot)** 功能。
   - 接着点击左侧的 **“事件订阅” (Event Subscriptions)**。在配置页上方，您能找到用于验证回调安全性的 **`Verification Token`**。

### 3.2 注入 AI-Nexus 配置
将获取到的 ID 和 Token 填入 `ainexus_secrets.json` 的 `lark_personas` 节点中：
```json
"lark_personas": [
    {
        "id": "lark_primary",
        "app_id": "在此填入您的 Lark App ID",
        "app_secret": "在此填入您的 Lark App Secret",
        "verification_token": "在此填入您的 Lark Verification Token",
        "is_active": true
    }
]
```

---

## 4. 全局初始化提醒

> [!IMPORTANT]
> 凭证注入说明
> 上述 `ainexus_secrets.json` 只是 AI-Nexus 系统的配置**种子文件**。在系统生命周期的 Genesis 初始化阶段，AI-Nexus-Init 会读取该种子文件的内容，并将其**解析并序列化为二进制 Block 块**，追加写入至 **Block-L 长期记忆区**（而非作为裸露的 JSON 文件存在）中持久化保存。后续系统运行期间所有的增删改查以及动态加载，均指向底层的二进制存储记录，从而确保严格的物理读写隔离原则。


---


