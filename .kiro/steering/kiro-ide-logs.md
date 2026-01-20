# Kiro IDE 日志分析指南

## 版本信息
- Kiro IDE 版本：v0.8.140
- 创建日期：2026-01-20

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

**内容**：
- 完整的 API 请求结构（JSON 格式）
- 完整的 API 响应结构
- 工具调用（toolUses）和工具结果（toolResults）
- 会话状态（conversationState）
- 历史消息（history）
- 配额信息（GetUsageLimitsCommand）

**特点**：
- 敏感信息被标记为 `***SensitiveInformation***`
- 可以看到实际的请求格式，比源码更直观
- 包含完整的消息流转过程

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

## 相关文档

- Kiro IDE 源码分析：`.kiro/steering/kiro-ide-source.md`
- Kiro API 规范：`docs/kiro-gate/kiro-api.md`
- 消息清理规范：`.kiro/steering/message-sanitization.md`
