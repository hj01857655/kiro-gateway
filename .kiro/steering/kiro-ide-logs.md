# Kiro IDE 日志分析指南

## 版本信息
- Kiro IDE 版本：v0.8.140
- 分析日期：2026-01-20
- 日志来源：`C:\Users\12925\AppData\Roaming\Kiro\logs\`

---

## 日志文件概览

| 日志文件 | 大小 | 行数 | 重要性 | 用途 |
|---------|------|------|--------|------|
| q-client.log | 4.5 MB | 266 | ⭐⭐⭐⭐⭐ | API 请求/响应 |
| Kiro Logs.log | 4.78 MB | 27,441 | ⭐⭐⭐⭐ | 运行状态 |
| KiroLLMLogs.log | 2.62 MB | 1,826 | ⭐⭐⭐⭐⭐ | LLM 提示词 |
| Kiro - MCP Logs.log | 0.03 MB | 388 | ⭐⭐⭐ | MCP 工具调用 |

---

## 日志位置

### 主日志目录
```
C:\Users\{用户名}\AppData\Roaming\Kiro\logs\
```

### 日志目录结构
```
logs/
├── {日期时间戳}/              # 例如：20260118T144803
│   ├── window1/               # 窗口1的日志
│   │   └── exthost/          # 扩展宿主日志
│   │       ├── kiro.kiroAgent/  # Kiro Agent 日志目录
│   │       │   ├── q-client.log        # Q Client 日志（最重要！）
│   │       │   ├── Kiro Logs.log       # Kiro 主日志
│   │       │   ├── KiroLLMLogs.log     # LLM 相关日志
│   │       │   └── Kiro - MCP Logs.log # MCP 日志
│   │       ├── exthost.log
│   │       ├── extHostTelemetry.log
│   │       └── output_logging_*.log
│   ├── window2/               # 窗口2的日志（如果有多个窗口）
│   ├── main.log
│   ├── telemetry.log
│   └── ...
```

## 关键日志文件

### 1. q-client.log ⭐⭐⭐⭐⭐
**最重要的日志文件**，记录了完整的 Kiro API 请求和响应。

**位置**：
```
C:\Users\{用户名}\AppData\Roaming\Kiro\logs\{时间戳}\window{N}\exthost\kiro.kiroAgent\q-client.log
```

**基本信息**：
- **大小**：4.5 MB
- **行数**：266 行
- **平均每行**：约 17 KB（每行是一个完整的 JSON 对象）
- **重要性**：⭐⭐⭐⭐⭐（最重要）

**内容结构**：

每一行都是一个完整的 JSON 对象，包含：

```json
{
  "clientName": "CodeWhispererStreamingClient",
  "commandName": "GenerateAssistantResponseCommand",
  "input": {
    "conversationState": {
      "conversationId": "uuid",
      "agentContinuationId": "uuid",
      "agentTaskType": "vibe",
      "currentMessage": { ... },
      "history": [ ... ],
      "chatTriggerType": "MANUAL"
    }
  },
  "output": {
    "conversationId": "",
    "generateAssistantResponseResponse": "STREAMING_CONTENT"
  },
  "metadata": {
    "httpStatusCode": 200,
    "requestId": "uuid",
    "attempts": 1,
    "totalRetryDelay": 0
  }
}
```

**关键发现**：

1. **敏感信息脱敏** - 所有敏感信息都被标记为 `***SensitiveInformation***`：
   - 用户输入内容
   - 工具名称和参数
   - 工具结果
   - 文件内容

2. **完整的请求结构**：
   - `conversationId` - 会话 ID
   - `agentContinuationId` - Agent 连续 ID
   - `agentTaskType` - 任务类型（`vibe` 或 `spectask`）
   - `currentMessage` - 当前消息
   - `history` - 历史消息数组
   - `chatTriggerType` - 触发类型（`MANUAL`）

3. **工具调用流程**：
   ```
   用户消息 → Assistant 返回 toolUses → 用户提交 toolResults → Assistant 继续响应
   ```

4. **配额查询**：
   ```json
   {
     "clientName": "CodeWhispererRuntimeClient",
     "commandName": "GetUsageLimitsCommand",
     "input": {
       "origin": "AI_EDITOR",
       "resourceType": "AGENTIC_REQUEST"
     },
     "output": {
       "usageBreakdownList": [{
         "currentUsage": 0,
         "usageLimit": 50,
         "freeTrialInfo": {
           "currentUsage": 326,
           "usageLimit": 500,
           "freeTrialStatus": "ACTIVE"
         }
       }]
     }
   }
   ```

5. **simple-task 模型** - 发现了一个特殊模型：
   - 不带工具定义（`userInputMessageContext: {}`）
   - 用于简单的文本生成任务
   - 不调用工具（`toolUses: []`）

**用途**：
- ✅ 验证 API 请求格式
- ✅ 调试 API 错误
- ✅ 学习官方实现
- ✅ 对比自己的实现

**查看命令**：
```powershell
# 查看最新的 q-client.log
Get-ChildItem "$env:APPDATA\Kiro\logs" -Recurse -Filter "q-client.log" | 
  Sort-Object LastWriteTime -Descending | 
  Select-Object -First 1 | 
  ForEach-Object { Get-Content $_.FullName -Tail 50 }
```

### 2. Kiro Logs.log
**主日志文件**，记录 Kiro IDE 的运行日志。

**大小**：通常较大（几 MB）

**内容**：
- Kiro IDE 的运行状态
- 错误和警告信息
- 功能调用记录

### 3. KiroLLMLogs.log
**LLM 相关日志**，记录与大语言模型交互的日志。

**内容**：
- LLM 请求和响应
- Token 使用情况
- 模型选择信息

### 4. Kiro - MCP Logs.log
**MCP 日志**，记录 Model Context Protocol 相关的日志。

**内容**：
- MCP 服务器连接状态
- MCP 工具调用记录
- MCP 错误信息

## 查找日志文件

### 查找最新的日志目录
```powershell
Get-ChildItem "$env:APPDATA\Kiro\logs" | 
  Sort-Object LastWriteTime -Descending | 
  Select-Object -First 1
```

### 查找所有 q-client.log 文件
```powershell
Get-ChildItem "$env:APPDATA\Kiro\logs" -Recurse -Filter "q-client.log" | 
  Select-Object FullName, @{Name="Size(KB)";Expression={[math]::Round($_.Length/1KB,2)}}
```

### 查找有内容的日志文件（大于 1KB）
```powershell
Get-ChildItem "$env:APPDATA\Kiro\logs" -Recurse -Filter "*.log" | 
  Where-Object { $_.Length -gt 1000 } | 
  Sort-Object Length -Descending | 
  Select-Object -First 10 FullName, @{Name="Size(KB)";Expression={[math]::Round($_.Length/1KB,2)}}
```

## 日志分析技巧

### 1. 查看最新的 API 请求
```powershell
$logFile = Get-ChildItem "$env:APPDATA\Kiro\logs" -Recurse -Filter "q-client.log" | 
  Sort-Object LastWriteTime -Descending | 
  Select-Object -First 1

Get-Content $logFile.FullName -Tail 100
```

### 2. 搜索特定关键词
```powershell
$logFile = Get-ChildItem "$env:APPDATA\Kiro\logs" -Recurse -Filter "q-client.log" | 
  Sort-Object LastWriteTime -Descending | 
  Select-Object -First 1

Select-String -Path $logFile.FullName -Pattern "conversationId" -Context 0,5
```

### 3. 提取 JSON 结构
q-client.log 中的每一行都是一个完整的 JSON 对象，可以用 PowerShell 解析：

```powershell
$logFile = Get-ChildItem "$env:APPDATA\Kiro\logs" -Recurse -Filter "q-client.log" | 
  Sort-Object LastWriteTime -Descending | 
  Select-Object -First 1

$lastLine = Get-Content $logFile.FullName -Tail 1
$json = $lastLine | ConvertFrom-Json
$json | ConvertTo-Json -Depth 10
```

## 从日志中学习

### 可以学到的内容

1. **请求格式**：
   - 完整的 `conversationState` 结构
   - `currentMessage` 和 `history` 的格式
   - 工具调用的格式（`toolUses` 和 `toolResults`）

2. **响应格式**：
   - API 返回的数据结构
   - 流式响应的处理方式
   - 错误响应的格式

3. **消息流转**：
   - 用户消息 → 助手消息 → 工具调用 → 工具结果 → 助手消息
   - 消息如何在 `history` 中累积
   - `conversationId` 如何保持会话连续性

4. **配额管理**：
   - `GetUsageLimitsCommand` 的请求和响应
   - 配额信息的结构
   - 免费试用和订阅信息

### 与源码对比

- **日志优势**：可以看到实际运行时的数据，更直观
- **源码优势**：可以看到实现逻辑和边界条件处理
- **结合使用**：先看日志了解格式，再看源码理解实现

## 注意事项

1. **敏感信息**：
   - 日志中的敏感信息会被标记为 `***SensitiveInformation***`
   - 包括：用户输入、文件内容、工具名称、工具参数等
   - 不影响学习请求和响应的结构

2. **日志大小**：
   - q-client.log 可能很大（几 MB）
   - 建议使用 `-Tail` 参数只查看最后几行
   - 或者使用 `Select-String` 搜索特定内容

3. **日志时效性**：
   - 日志按会话时间戳分目录存储
   - 最新的日志在最新的时间戳目录中
   - 关闭 Kiro IDE 后会创建新的日志目录

4. **多窗口**：
   - 如果打开多个 Kiro IDE 窗口，会有 window1、window2 等
   - 每个窗口有独立的日志目录

## 实际应用

### 在 kiro-gateway 开发中的应用

1. **验证请求格式**：
   - 对比 kiro-gateway 构建的请求和 Kiro IDE 实际发送的请求
   - 确保格式完全一致

2. **调试问题**：
   - 当 API 返回错误时，查看 Kiro IDE 的日志
   - 看看官方是如何处理的

3. **学习新功能**：
   - 当 Kiro IDE 更新后，查看日志了解新的 API 变化
   - 例如：新的工具、新的字段、新的响应格式

4. **性能优化**：
   - 查看请求的大小和响应时间
   - 优化 kiro-gateway 的性能

## 日志文件详细分析

### 1. q-client.log ⭐⭐⭐⭐⭐

**最重要的日志**，记录完整的 Kiro API 请求和响应。

**内容示例**：
- 完整的 `conversationState` 结构（包含 conversationId、agentContinuationId、agentTaskType）
- 工具调用和工具结果的完整流程
- 配额查询（GetUsageLimitsCommand）
- 流式响应（STREAMING_CONTENT）

**用途**：
- 验证请求格式是否正确
- 调试 API 错误
- 学习官方实现

### 2. Kiro Logs.log

**主日志文件**，记录 Kiro IDE 的运行状态。

**内容示例**：
```
2026-01-19 03:26:52.134 [warning] [SteeringController] AGENTS.md file is not in a workspace folder
2026-01-19 03:26:52.160 [info] [ChatFile] Wrote chat file
2026-01-19 03:26:52.631 [error] [ProfileStorage] Error reading profile
2026-01-19 03:26:56.618 [info] [AgentIterator] Synchronizing results
2026-01-19 03:26:56.622 [info] [Steering] Populating steering for execution
2026-01-19 03:27:47.487 [info] [Execution] Completed with abort
```

**关键信息**：
- SteeringController - Steering 文件管理
- ChatFile - 聊天文件保存
- ProfileStorage - 配置文件读取
- AgentIterator - Agent 执行流程
- Execution - 执行状态（完成/中止）

**用途**：
- 调试 Steering 文件问题
- 查看 Agent 执行流程
- 排查配置文件错误

### 3. KiroLLMLogs.log

**LLM 提示词日志**，记录发送给 LLM 的完整提示词。

**内容示例**：
- 完整的系统提示词（System Prompt）
- 用户消息（User Message）
- AI 响应（AI Message）
- 工具调用（Tool Calls）

**关键发现**：
- 系统提示词包含完整的 Kiro IDE 功能说明
- 包含 Steering 规则、Hooks 规则、MCP 规则等
- 可以看到 AI 的思考过程和工具调用

**用途**：
- 理解 Kiro IDE 的提示词工程
- 学习如何构建 AI Agent 系统
- 调试 AI 响应问题

### 4. Kiro - MCP Logs.log

**MCP 日志**，记录 Model Context Protocol 相关的日志。

**内容示例**：
```
2026-01-19 01:51:34.454 [info] [chrome-devtools] MCP Tool Call
  Tool: list_network_requests
  Arguments: {"resourceTypes":["script"],"pageSize":50}
  Consent Mechanism: auto

2026-01-19 01:52:28.833 [info] [chrome-devtools] MCP Tool Call
  Tool: get_network_request
  Arguments: {"reqid":130}
  Consent Mechanism: auto

2026-01-19 01:52:51.465 [info] [chrome-devtools] MCP Tool Call
  Tool: navigate_page
  Arguments: {"type":"url","url":"https://..."}
  Consent Mechanism: auto
```

**关键信息**：
- MCP 工具调用记录（Tool Call）
- 工具参数（Arguments）
- 同意机制（Consent Mechanism: auto/manual）
- Chrome DevTools MCP 的使用

**用途**：
- 调试 MCP 工具调用
- 查看 MCP 服务器状态
- 学习 MCP 工具的使用方式

### 5. 5-Kiro - LLM PromptCompletion.log

**LLM 完成日志**，记录 LLM 的完整响应。

**大小**：875 KB

**用途**：
- 查看 LLM 的完整响应
- 调试流式响应问题
- 分析 Token 使用情况

---

## 日志分析总结

**最有价值的日志**：
1. **q-client.log** - API 请求/响应，必看
2. **KiroLLMLogs.log** - 提示词工程，学习用
3. **Kiro - MCP Logs.log** - MCP 工具调用，调试用
4. **Kiro Logs.log** - 运行状态，排查问题用

**日志查看技巧**：
- 使用 `-Tail` 参数只看最后几行
- 使用 `Select-String` 搜索关键词
- 使用 `Sort-Object LastWriteTime` 找最新日志
- 大文件（如 Kiro Logs.log）慎用 `Get-Content -Raw`

**常见问题排查**：
- API 错误 → 查 q-client.log
- Steering 不生效 → 查 Kiro Logs.log
- MCP 工具失败 → 查 Kiro - MCP Logs.log
- AI 响应异常 → 查 KiroLLMLogs.log

---

## 关键发现总结

### 1. API 请求结构（q-client.log）

**完整的 conversationState**：
```json
{
  "conversationId": "uuid",
  "agentContinuationId": "uuid",  // ✅ 已实现
  "agentTaskType": "vibe",        // ✅ 已实现
  "currentMessage": { ... },
  "history": [ ... ],
  "chatTriggerType": "MANUAL"
}
```

**工具调用流程**：
1. 用户消息 + 工具定义（约 46 个工具）
2. Assistant 返回 toolUses
3. 用户提交 toolResults（content 为空字符串 `""`）
4. Assistant 继续响应

**配额计算**：
- 基础配额：0 / 50
- 试用配额：326 / 500
- 总使用率：326 / 550 = 59.3%

### 2. 系统提示词（KiroLLMLogs.log）

**核心组件**：
- Identity（身份）
- Capabilities（能力）
- Response Style（响应风格）
- Rules（规则）
- Key Kiro Features（关键功能）
  - Autonomy Modes
  - Chat Context
  - Spec
  - Hooks
  - Steering
  - MCP
  - Internet Access

**Steering 规则**：
- 全局规则（Global）
- 工作区规则（Workspace）
- 自动包含 / 条件包含 / 手动引用

### 3. 运行状态（Kiro Logs.log）

**关键模块**：
- SteeringController - Steering 文件管理
- ChatFile - 聊天文件保存
- ProfileStorage - 配置文件读取
- AgentIterator - Agent 执行流程
- Execution - 执行状态

**常见问题**：
- Steering 文件不在工作区
- 配置文件读取失败
- 执行中止

### 4. MCP 工具（Kiro - MCP Logs.log）

**Chrome DevTools MCP**：
- 网络请求监控
- 页面导航
- 脚本执行
- 页面截图

**同意机制**：
- 所有工具调用自动批准

---

## 实际应用

### 1. 验证 kiro-gateway 实现

**对比清单**：
- ✅ conversationId
- ✅ agentContinuationId
- ✅ agentTaskType
- ✅ currentMessage
- ✅ history
- ✅ chatTriggerType
- ✅ 工具调用流程
- ✅ 配额查询

**结论**：kiro-gateway 实现与 Kiro IDE 完全一致（100%）

### 2. 学习提示词工程

**关键要素**：
- 清晰的身份定义
- 详细的能力说明
- 规范的响应风格
- 完整的规则系统
- 模块化的功能组织

### 3. 调试问题

**问题类型** → **查看日志**：
- API 错误 → q-client.log
- Steering 不生效 → Kiro Logs.log
- AI 响应异常 → KiroLLMLogs.log
- MCP 工具失败 → Kiro - MCP Logs.log

### 4. 优化实现

**优化方向**：
- 参考官方的消息清理逻辑
- 学习工具调用的最佳实践
- 优化配额查询和计算
- 改进错误处理

---

## 相关文档

- Kiro IDE 源码分析：`.kiro/steering/kiro-ide-source.md`
- Kiro API 规范：`docs/kiro-gate/kiro-api.md`
- 消息清理规范：`.kiro/steering/message-sanitization.md`
- Kiro API 请求结构：`.kiro/steering/kiro-api-request-structure.md`
- Kiro API 完整流程：`.kiro/steering/kiro-api-flow-analysis.md`
- 会话管理源码分析：`.kiro/steering/kiro-session-management-source-analysis.md`
