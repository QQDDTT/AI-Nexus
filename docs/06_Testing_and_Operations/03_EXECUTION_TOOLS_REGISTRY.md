# 11.0 执行工具册 (Execution Tools Registry)

## 架构边界说明
在 AI-Nexus 系统中，我们明确区分了 **“技能 (Skills)”** 与 **“执行工具 (Execution Tools)”**：

1. **技能 (Skills，位于 `skills/` 目录)**：
   - **本质**：Agent 的“认知外挂”与标准作业程序 (SOP)。
   - **形态**：纯文本的 Markdown (`SKILL.md`)。
   - **职责**：不包含任何可执行代码，只包含系统的意图、逻辑推理指导以及工具使用的前提条件。它告诉 LLM “遇到什么情况，应该做什么判断，最终调用什么工具”。

2. **执行工具 (Execution Tools，原 Native Skills)**：
   - **本质**：Agent 的“手和脚”。
   - **形态**：由 Rust 编写，编译在核心底座中，或通过 Wasm 沙箱按需加载。
   - **职责**：无状态、确定性的能力载体。它们作为全局公共资源池，可以被任何 Skill 通过指令（Prompt）调度和调用。

---

## 当前系统公共执行工具列表

以下是底层已注册并可供全量 Skill 重复调用的基础级执行工具：

### 1. 网页内容检索 (`web_search`)
* **核心职责**：调用外部 HTTP 网络接口，为大模型提供获取实时网络数据、发起 API 调用的能力。
* **参数 (Schema)**：
  ```json
  {
      "domain": "<string, 目标域名或 IP 地址，如 'api.github.com' 或 '192.168.1.10'>",
      "endpoint": "<string, 请求路径，如 '/search/repositories'>",
      "method": "<string, HTTP 方法，如 'GET', 'POST', 'PUT', 'DELETE'>",
      "params": "<object, URL 查询参数 (Query parameters)>",
      "headers": "<object, HTTP 请求头 (Request headers)，如 {'Authorization': 'Bearer ...'}>",
      "body": "<object|string, 请求体 (Request body)，对于 POST/PUT 请求必需>"
  }
  ```
* **返回值 (反馈能力)**：具备读取与解析完整 Response 的能力，返回结构如下：
  ```json
  {
      "status_code": 200,
      "headers": { "content-type": "application/json" },
      "body": "<解析后的具体数据 (对象或字符串)>",
      "error": "<网络请求失败时的异常原因>"
  }
  ```

### 2. 沙盒文件生成 (`file_generate`)
* **核心职责**：将生成的代码或文本内容落盘为物理文件。
* **安全机制**：路径受到严格的沙盒限制（限定于 `./data/generated/` 目录下），防止跨目录攻击 (`../` 拦截)。此工具默认开启**人类审批 (Human Approval)** 防线。
* **参数 (Schema)**：
  ```json
  {
      "filename": "<string, 相对文件名，如 'output.json'>",
      "content": "<string, 写入的源内容>",
      "format": "<string, 文件格式标识，如 'json', 'markdown', 'rust', 'plaintext'>",
      "encoding": "<string, 文件编码格式，默认 'utf-8'>"
  }
  ```
* **返回值 (审核与校验能力)**：具备对落盘文件进行后置审核与状态获取的能力，返回结构如下：
  ```json
  {
      "success": true,
      "file_path": "./data/generated/output.json",
      "size_bytes": 1024,
      "checksum": "<sha256哈希值>",
      "audit_status": "Passed"
  }
  ```

### 3. Wasm 动态沙盒执行器 (`dynamic_wasm_sandbox`)
* **核心职责**：提供一个 Wasmtime 运行时环境，能够安全隔离地加载 `.wasm` 字节码并执行其中的 `_start` 或特定函数。
* **应用场景**：当 Agent 动态生成了一段复杂的数据处理代码并编译成 Wasm 后，通过此工具在内存沙盒中验证其运行结果，而不会崩溃宿主系统。
* **参数 (Schema)**：
  ```json
  {
      "script_path": "<string, Wasm 脚本/模块的物理路径或资源地址，如 './data/generated/my_logic.wasm'>",
      "function_name": "<string, 要调用的目标函数名，默认 '_start'>",
      "args": "<array, 传递给 Wasm 函数的执行参数列表，如 ['--input', 'data.json']>",
      "env_vars": "<object, 注入到执行沙盒中的环境变量>"
  }
  ```
* **返回值 (结果判断与异常处理能力)**：具备精准捕获执行结果、拦截标准输出以及处理底层异常陷阱 (Traps) 的能力，返回结构如下：
  ```json
  {
      "exit_code": 0,
      "stdout": "<执行期间的标准输出日志>",
      "stderr": "<执行期间的标准错误日志>",
      "execution_time_ms": 45,
      "result": "<Wasm 函数实际返回的数据/指针>",
      "trap_error": "<发生内存越界等 Wasm Trap 时的异常详细堆栈>"
  }
  ```
---

## Skill 如何声明与调用执行工具？

由于执行工具是**全局共用资源**，为了让 Agent 在读取特定 Skill 时知道自己拥有哪些“手脚”，我们建议在 `skills/<skill_name>/SKILL.md` 中增加**固定的资源声明段落**：

### 推荐的 SKILL.md 编写范式

在你的 `SKILL.md` 中加入 `## 可用执行工具` (Available Tools) 模块：

```markdown
---
name: research_and_report
description: 利用网络搜索收集资料，并输出为本地报告文件。
---

# Research and Report Skill

## 1. 可用执行工具 (Shared Execution Tools)
在执行本技能时，你（Agent）有权调用以下全局执行工具：
- `web_search`: 用于查找相关的百科背景知识。
- `file_generate`: 用于将最终的总结报告保存到本地。

## 2. 工具调用规范
当你决定使用工具时，请严格输出以下 JSON 格式的指令块，等待宿主拦截并执行：

\`\`\`json
{
  "action": "call_tool",
  "tool_name": "web_search",
  "arguments": {
    "query": "Rust (programming language)"
  }
}
\`\`\`

## 3. 业务流转逻辑
1. 先根据用户的问题，使用 `web_search` 收集 2~3 个维度的信息。
2. 整合信息并润色，最终使用 `file_generate` 将报告保存为 `report.md`。
```

通过这种声明式的方法，底层工具的变动无需修改每个 Skill，只需要在基础层添加新工具，然后通过 Prompt 赋能给对应的 Skill 即可。
