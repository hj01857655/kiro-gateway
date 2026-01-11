# KiroGate 实现指南

## 一句话说明

KiroGate 是一个代理服务器，把 OpenAI/Anthropic 格式的请求转成 Kiro 格式，调用 Kiro API，再把响应转回去。

---

## 整体流程图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              KiroGate 完整流程                               │
└─────────────────────────────────────────────────────────────────────────────┘

  Claude Code / Cursor / Continue
           │
           │ POST /v1/messages (Anthropic) 或 /v1/chat/completions (OpenAI)
           ▼
  ┌─────────────────┐
  │  1. 接收请求     │  ← 判断是 OpenAI 还是 Anthropic 格式
  └────────┬────────┘
           │
           ▼
  ┌─────────────────┐
  │  2. 验证 API Key │  ← 你自己设的 KiroGate 密钥，不是 Kiro 的
  └────────┬────────┘
           │
           ▼
  ┌─────────────────┐
  │  3. 选择账号     │  ← 从账号池选一个 Token 有效的
  └────────┬────────┘
           │
           ▼
  ┌─────────────────┐     ┌─────────────────┐
  │  4. 检查 Token   │────→│  Token 过期？    │
  └────────┬────────┘     └────────┬────────┘
           │                       │ 是
           │ 否                    ▼
           │              ┌─────────────────┐
           │              │  刷新 Token      │
           │              │  Social: Kiro Auth
           │              │  IDC: AWS OIDC   │
           │              └────────┬────────┘
           │                       │
           ◄───────────────────────┘
           │
           ▼
  ┌─────────────────┐
  │  5. 转换请求格式  │  ← OpenAI/Anthropic → Kiro
  └────────┬────────┘
           │
           ▼
  ┌─────────────────┐
  │  6. 调用 Kiro API │  ← POST generateAssistantResponse
  └────────┬────────┘
           │
           ▼ (SSE 流)
  ┌─────────────────┐
  │  7. 转换响应格式  │  ← Kiro SSE → OpenAI/Anthropic SSE
  └────────┬────────┘
           │
           ▼
  Claude Code / Cursor / Continue
```

---

## 核心步骤详解

### 步骤 5: 请求格式转换

```
┌─────────────────────────────────────────────────────────────────┐
│                        请求转换流程                              │
└─────────────────────────────────────────────────────────────────┘

  OpenAI 请求                          Anthropic 请求
  {                                    {
    "model": "gpt-4",                    "model": "claude-3-5-sonnet",
    "messages": [                        "system": "你是助手",
      {"role": "system", "content":...}  "messages": [
      {"role": "user", "content":...}      {"role": "user", "content":...}
    ],                                   ],
    "tools": [...]                       "tools": [...]
  }                                    }
           │                                    │
           └──────────────┬─────────────────────┘
                          ▼
                 ┌─────────────────┐
                 │   提取各部分     │
                 │                 │
                 │ • system prompt │
                 │ • messages      │
                 │ • tools         │
                 │ • model         │  ← 模型选择
                 └────────┬────────┘
                          ▼
                 ┌─────────────────┐
                 │   组装 Kiro 格式 │
                 └────────┬────────┘
                          ▼
  {
    "conversationState": {
      "conversationId": "uuid",
      "currentMessage": {
        "userInputMessage": {
          "content": ["用户消息"],
          "modelId": "qdev::claude-sonnet-4.5",  ← 模型 ID
          "userIntent": "CODE_GENERATION",
          "userInputMessageContext": {
            "tools": [...],                    ← 工具定义
            "additionalContext": {
              "systemPrompt": "你是助手"       ← system prompt
            }
          }
        }
      },
      "history": [...]                         ← 历史消息
    },
    "profileArn": ""                           ← IDC 账号才有值
  }
```

### 模型选择

**可用模型（2026-01-11 实测）**：

- `auto` - 自动选择（默认，1.0x 费率）
- `claude-haiku-4.5` - 最便宜（0.4x 费率）
- `claude-sonnet-4` - 常规使用（1.3x 费率）
- `claude-sonnet-4.5` - 最新 Sonnet（1.3x 费率）
- `claude-opus-4.5` - 最强（2.2x 费率）
- `claude-sonnet-4.5-1m` - 1M 上下文（客户端硬编码）

**modelId 格式**：`qdev::模型ID`

```javascript
// 模型映射示例
const MODEL_MAP = {
  // OpenAI 模型名 → Kiro modelId
  'gpt-4': 'qdev::auto',
  'gpt-4-turbo': 'qdev::claude-sonnet-4.5',
  'gpt-4o': 'qdev::claude-sonnet-4.5',
  
  // Anthropic 模型名 → Kiro modelId
  'claude-3-5-sonnet-20241022': 'qdev::claude-sonnet-4.5',
  'claude-3-opus-20240229': 'qdev::claude-opus-4.5',
  'claude-3-haiku-20240307': 'qdev::claude-haiku-4.5',
  
  // 直接使用 Kiro 模型名
  'claude-sonnet-4.5': 'qdev::claude-sonnet-4.5',
  'claude-opus-4.5': 'qdev::claude-opus-4.5',
  'auto': 'qdev::auto'
}

function getKiroModelId(requestModel) {
  return MODEL_MAP[requestModel] || 'qdev::auto'
}
```

**获取可用模型列表**（可选）：

```bash
curl -X GET "https://codewhisperer.us-east-1.amazonaws.com/ListAvailableModels?origin=AI_EDITOR" \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN"
```

### 步骤 7: 响应格式转换

```
┌─────────────────────────────────────────────────────────────────┐
│                        响应转换流程                              │
└─────────────────────────────────────────────────────────────────┘

  Kiro SSE 事件流
       │
       ▼
  ┌─────────────────────────────────────────────────────────────┐
  │ event: messageMetadataEvent                                 │
  │ data: {"messageId": "xxx", ...}                             │
  │                                                             │
  │ event: assistantResponseEvent        ──→  文本内容          │
  │ data: {"content": "Hello"}                                  │
  │                                                             │
  │ event: assistantResponseEvent        ──→  文本内容          │
  │ data: {"content": " World"}                                 │
  │                                                             │
  │ event: toolUseEvent                  ──→  工具调用          │
  │ data: {"toolUseId": "x", "name": "read_file", "input": {}}  │
  │                                                             │
  │ event: contextUsageEvent             ──→  token 统计        │
  │ data: {"inputTokens": 100, "outputTokens": 50}              │
  └─────────────────────────────────────────────────────────────┘
       │
       ▼ 转换
  ┌─────────────────────────────────────────────────────────────┐
  │                      OpenAI 格式                             │
  │                                                             │
  │ data: {"choices":[{"delta":{"content":"Hello"}}]}           │
  │ data: {"choices":[{"delta":{"content":" World"}}]}          │
  │ data: {"choices":[{"delta":{"tool_calls":[...]}}]}          │
  │ data: {"choices":[{"finish_reason":"stop"}]}                │
  │ data: [DONE]                                                │
  └─────────────────────────────────────────────────────────────┘
       │
       ▼ 或者
  ┌─────────────────────────────────────────────────────────────┐
  │                     Anthropic 格式                           │
  │                                                             │
  │ event: message_start                                        │
  │ data: {"type":"message_start",...}                          │
  │                                                             │
  │ event: content_block_delta                                  │
  │ data: {"delta":{"text":"Hello"}}                            │
  │                                                             │
  │ event: content_block_delta                                  │
  │ data: {"delta":{"text":" World"}}                           │
  │                                                             │
  │ event: message_stop                                         │
  │ data: {"type":"message_stop"}                               │
  └─────────────────────────────────────────────────────────────┘
```

---

## 最简实现代码

### 1. 项目结构

```
kirogate/
├── src/
│   ├── index.js          # 入口，启动 HTTP 服务
│   ├── routes/
│   │   ├── openai.js     # /v1/chat/completions
│   │   └── anthropic.js  # /v1/messages
│   ├── converter/
│   │   ├── request.js    # 请求格式转换
│   │   └── response.js   # 响应格式转换
│   ├── kiro/
│   │   ├── client.js     # 调用 Kiro API
│   │   └── auth.js       # Token 刷新
│   └── account/
│       └── manager.js    # 账号池管理
├── config.json           # 账号配置
└── package.json
```

### 2. 入口文件 (index.js)

```javascript
const express = require('express')
const { handleOpenAI } = require('./routes/openai')
const { handleAnthropic } = require('./routes/anthropic')

const app = express()
app.use(express.json())

// OpenAI 兼容接口
app.post('/v1/chat/completions', handleOpenAI)

// Anthropic 兼容接口
app.post('/v1/messages', handleAnthropic)

app.listen(8080, () => {
  console.log('KiroGate running on http://localhost:8080')
})
```

### 3. 请求处理 (routes/openai.js)

```javascript
const { convertRequest } = require('../converter/request')
const { convertResponse } = require('../converter/response')
const { callKiroAPI } = require('../kiro/client')
const { getAccount } = require('../account/manager')

async function handleOpenAI(req, res) {
  // 1. 验证 API Key
  const apiKey = req.headers.authorization?.replace('Bearer ', '')
  if (apiKey !== process.env.API_KEY) {
    return res.status(401).json({ error: { message: 'Invalid API key' } })
  }

  // 2. 获取可用账号
  const account = await getAccount()
  if (!account) {
    return res.status(503).json({ error: { message: 'No available account' } })
  }

  // 3. 转换请求格式
  const kiroRequest = convertRequest(req.body, 'openai')

  // 4. 设置 SSE 响应头
  res.setHeader('Content-Type', 'text/event-stream')
  res.setHeader('Cache-Control', 'no-cache')
  res.setHeader('Connection', 'keep-alive')

  // 5. 调用 Kiro API 并转换响应
  try {
    const stream = await callKiroAPI(kiroRequest, account)
    
    for await (const event of convertResponse(stream, 'openai')) {
      res.write(`data: ${JSON.stringify(event)}\n\n`)
    }
    
    res.write('data: [DONE]\n\n')
    res.end()
  } catch (error) {
    res.write(`data: ${JSON.stringify({ error: { message: error.message } })}\n\n`)
    res.end()
  }
}

module.exports = { handleOpenAI }
```

### 4. 请求转换 (converter/request.js)

```javascript
function convertRequest(request, format) {
  // 提取 system prompt
  let systemPrompt = ''
  let messages = request.messages || []
  
  if (format === 'openai') {
    const systemMsg = messages.find(m => m.role === 'system')
    systemPrompt = systemMsg?.content || ''
    messages = messages.filter(m => m.role !== 'system')
  } else if (format === 'anthropic') {
    systemPrompt = request.system || ''
  }

  // 分离历史和当前消息
  const lastUserIndex = messages.findLastIndex(m => m.role === 'user')
  const history = messages.slice(0, lastUserIndex)
  const currentContent = messages[lastUserIndex]?.content || ''

  // 转换工具定义
  const tools = convertTools(request.tools, format)

  // 获取模型 ID
  const modelId = getKiroModelId(request.model)

  // 组装 Kiro 格式
  return {
    conversationState: {
      conversationId: crypto.randomUUID(),
      chatTriggerType: 'MANUAL',
      currentMessage: {
        userInputMessage: {
          content: Array.isArray(currentContent) 
            ? currentContent.map(c => c.text || c) 
            : [currentContent],
          modelId,  // ← 模型选择
          userIntent: 'CODE_GENERATION',
          userInputMessageContext: {
            tools,
            additionalContext: systemPrompt ? { systemPrompt } : {}
          }
        }
      },
      history: convertHistory(history, format)
    },
    profileArn: ''
  }
}

// 模型映射
const MODEL_MAP = {
  // OpenAI → Kiro
  'gpt-4': 'qdev::auto',
  'gpt-4-turbo': 'qdev::claude-sonnet-4.5',
  'gpt-4o': 'qdev::claude-sonnet-4.5',
  // Anthropic → Kiro
  'claude-3-5-sonnet-20241022': 'qdev::claude-sonnet-4.5',
  'claude-3-opus-20240229': 'qdev::claude-opus-4.5',
  'claude-3-haiku-20240307': 'qdev::claude-haiku-4.5',
  // 直接使用
  'claude-sonnet-4.5': 'qdev::claude-sonnet-4.5',
  'claude-opus-4.5': 'qdev::claude-opus-4.5',
  'auto': 'qdev::auto'
}

function getKiroModelId(requestModel) {
  return MODEL_MAP[requestModel] || 'qdev::auto'
}

function convertTools(tools, format) {
  if (!tools) return []
  
  return tools.map(tool => {
    if (format === 'openai' && tool.type === 'function') {
      return {
        toolSpec: {
          name: tool.function.name,
          description: tool.function.description || '',
          inputSchema: { json: tool.function.parameters || {} }
        }
      }
    }
    if (format === 'anthropic') {
      return {
        toolSpec: {
          name: tool.name,
          description: tool.description || '',
          inputSchema: { json: tool.input_schema || {} }
        }
      }
    }
  }).filter(Boolean)
}

function convertHistory(messages, format) {
  // 转换历史消息...
  return messages.map(msg => {
    if (msg.role === 'user') {
      return {
        userInputMessage: {
          content: [typeof msg.content === 'string' ? msg.content : msg.content[0]?.text],
          userIntent: 'CODE_GENERATION'
        }
      }
    }
    if (msg.role === 'assistant') {
      return {
        assistantResponseMessage: {
          content: typeof msg.content === 'string' ? msg.content : ''
        }
      }
    }
    // tool 结果...
  }).filter(Boolean)
}

module.exports = { convertRequest }
```

### 5. 调用 Kiro API (kiro/client.js)

```javascript
const KIRO_API = 'https://kiro.amazon.dev/v1/generateAssistantResponse'

// 限流错误码（AWS SDK 标准）
const THROTTLING_ERRORS = [
  'ThrottlingException', 'TooManyRequestsException', 'RequestThrottledException',
  'LimitExceededException', 'ProvisionedThroughputExceededException', 'SlowDown'
]

// 临时错误码
const TRANSIENT_ERRORS = ['TimeoutError', 'RequestTimeout', 'InternalServerException']

// 可重试的 HTTP 状态码
const RETRYABLE_STATUS = [429, 500, 502, 503, 504]

function isRetryable(error, status) {
  if (error.name === 'ExpiredTokenException') return true  // Token 过期可重试
  if (THROTTLING_ERRORS.includes(error.name)) return true
  if (TRANSIENT_ERRORS.includes(error.name)) return true
  if (RETRYABLE_STATUS.includes(status)) return true
  return false
}

// AWS SDK 标准延迟算法
function getRetryDelay(attempt, isThrottling) {
  const base = isThrottling ? 500 : 100  // 限流 500ms，普通 100ms
  const delay = Math.random() * Math.pow(2, attempt) * base
  return Math.min(Math.floor(delay), 20000)  // 最大 20 秒
}

async function callKiroAPI(request, account, options = {}) {
  const maxRetries = options.maxRetries || 3
  let lastError
  
  // 生成请求标识（Kiro 源码使用 UUID）
  const invocationId = crypto.randomUUID()
  
  for (let attempt = 0; attempt <= maxRetries; attempt++) {
    try {
      const response = await fetch(KIRO_API, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${account.accessToken}`,
          // AWS SDK 标准头
          'amz-sdk-invocation-id': invocationId,
          'amz-sdk-request': `attempt=${attempt + 1}; max=${maxRetries + 1}`,
          // Kiro 特定头
          'x-amzn-kiro-agent-mode': 'vibe',
          // User-Agent（格式：KiroIDE-版本-机器ID）
          'x-amz-user-agent': `KiroIDE-${account.kiroVersion || '0.1.25'}-${account.machineId || 'unknown'}`
        },
        body: JSON.stringify(request)
      })

      if (!response.ok) {
        const error = await response.json().catch(() => ({}))
        error.name = error.__type || error.name || 'UnknownError'
        
        if (isRetryable(error, response.status) && attempt < maxRetries) {
          // Token 过期需要刷新
          if (error.name === 'ExpiredTokenException' && options.refreshToken) {
            await options.refreshToken(account)
          }
          
          const delay = getRetryDelay(attempt, THROTTLING_ERRORS.includes(error.name))
          await new Promise(r => setTimeout(r, delay))
          continue
        }
        
        throw error
      }

      return response.body  // 返回 SSE 流
      
    } catch (error) {
      lastError = error
      
      if (isRetryable(error, 0) && attempt < maxRetries) {
        const delay = getRetryDelay(attempt, false)
        await new Promise(r => setTimeout(r, delay))
        continue
      }
      
      throw error
    }
  }
  
  throw lastError
}

module.exports = { callKiroAPI }
```

### 6. 响应转换 (converter/response.js)

⚠️ **重要**：Kiro API 返回的是 **AWS Event Stream 二进制格式**（`application/vnd.amazon.eventstream`），不是普通的 SSE 文本流！

#### AWS Event Stream 格式

Kiro 使用 AWS Smithy SDK 处理事件流，涉及以下包：
- `@smithy/eventstream-codec` - 二进制编解码
- `@smithy/eventstream-serde` - 序列化/反序列化
- `SmithyMessageDecoderStream` - 消息解码流

#### 解析实现

```javascript
const { EventStreamCodec } = require('@smithy/eventstream-codec')
const { fromUtf8, toUtf8 } = require('@smithy/util-utf8')

// 创建解码器（Kiro 源码方式）
const codec = new EventStreamCodec(toUtf8, fromUtf8)

async function* parseEventStream(responseBody) {
  const reader = responseBody.getReader()
  let buffer = new Uint8Array(0)
  
  while (true) {
    const { done, value } = await reader.read()
    if (done) break
    
    // 拼接 buffer
    buffer = concatArrays(buffer, value)
    
    // 尝试解析完整消息
    while (buffer.length >= 16) {  // AWS Event Stream 最小消息长度
      // 读取消息总长度（前 4 字节，大端序）
      const totalLength = new DataView(buffer.buffer, buffer.byteOffset).getUint32(0, false)
      
      if (buffer.length < totalLength) break  // 数据不完整，等待更多数据
      
      // 提取完整消息
      const messageBytes = buffer.slice(0, totalLength)
      buffer = buffer.slice(totalLength)
      
      try {
        // 使用 EventStreamCodec 解码
        const decoded = codec.decode(messageBytes)
        const payload = JSON.parse(toUtf8(decoded.body))
        yield payload
      } catch (e) {
        console.error('解析事件失败:', e)
      }
    }
  }
}

function concatArrays(a, b) {
  const result = new Uint8Array(a.length + b.length)
  result.set(a)
  result.set(b, a.length)
  return result
}
```

#### 完整响应转换

```javascript
const { EventStreamCodec } = require('@smithy/eventstream-codec')
const { fromUtf8, toUtf8 } = require('@smithy/util-utf8')

async function* convertResponse(stream, format) {
  let hasToolUse = false
  
  // 解析 AWS Event Stream
  for await (const event of parseEventStream(stream)) {
    const converted = convertEvent(event, format)
    if (converted) {
      if (event.toolUseEvent) hasToolUse = true
      yield converted
    }
  }

  // 发送结束事件
  yield createEndEvent(format, hasToolUse)
}

function convertEvent(event, format) {
  // 文本内容
  if (event.assistantResponseEvent) {
    const text = event.assistantResponseEvent.content
    if (format === 'openai') {
      return {
        choices: [{ index: 0, delta: { content: text }, finish_reason: null }]
      }
    }
    // anthropic...
  }

  // 工具调用
  if (event.toolUseEvent) {
    const { toolUseId, name, input } = event.toolUseEvent
    if (format === 'openai') {
      return {
        choices: [{
          index: 0,
          delta: {
            tool_calls: [{
              index: 0,
              id: toolUseId,
              type: 'function',
              function: { name, arguments: JSON.stringify(input) }
            }]
          },
          finish_reason: null
        }]
      }
    }
    // anthropic...
  }

  return null
}

function createEndEvent(format, hasToolUse) {
  const finishReason = hasToolUse ? 'tool_calls' : 'stop'
  if (format === 'openai') {
    return {
      choices: [{ index: 0, delta: {}, finish_reason: finishReason }]
    }
  }
  // anthropic...
}

module.exports = { convertResponse }
```

### 7. Token 刷新 (kiro/auth.js)

```javascript
// Social 账号刷新
async function refreshSocialToken(refreshToken) {
  const response = await fetch(
    'https://prod.us-east-1.auth.desktop.kiro.dev/refreshToken',
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refreshToken })
    }
  )
  return response.json()  // { accessToken, expiresAt }
}

// IDC 账号刷新
async function refreshIDCToken(refreshToken, clientId, clientSecret, region) {
  const response = await fetch(
    `https://oidc.${region}.amazonaws.com/token`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({
        grant_type: 'refresh_token',
        refresh_token: refreshToken,
        client_id: clientId,
        client_secret: clientSecret
      })
    }
  )
  return response.json()  // { access_token, expires_in }
}

module.exports = { refreshSocialToken, refreshIDCToken }
```

---

## 配置文件示例

```json
{
  "apiKey": "your-kirogate-api-key",
  "accounts": [
    {
      "id": "social-1",
      "type": "social",
      "accessToken": "eyJ...",
      "refreshToken": "eyJ...",
      "expiresAt": 1704067200000,
      "enabled": true
    },
    {
      "id": "idc-1",
      "type": "idc",
      "accessToken": "eyJ...",
      "refreshToken": "eyJ...",
      "clientId": "xxx",
      "clientSecret": "xxx",
      "region": "us-east-1",
      "profileArn": "arn:aws:...",
      "expiresAt": 1704067200000,
      "enabled": true
    }
  ]
}
```

---

## Claude Code 配置

```bash
# 设置环境变量
export ANTHROPIC_BASE_URL=http://localhost:8080
export ANTHROPIC_API_KEY=your-kirogate-api-key

# 启动 Claude Code
claude
```

---

## 关键点总结

1. **两个接口**: `/v1/chat/completions` (OpenAI) 和 `/v1/messages` (Anthropic)
2. **请求转换**: 提取 system/messages/tools/model → 组装 Kiro 格式
3. **模型选择**: 通过 `userInputMessage.modelId` 传递，格式为 `qdev::模型ID`
4. **响应转换**: Kiro SSE 事件 → OpenAI/Anthropic SSE 格式
5. **Token 管理**: 过期前自动刷新，Social 和 IDC 刷新端点不同
6. **工具调用**: Claude Code 会传工具，必须正确转换 tool_calls
7. **错误重试**: AWS SDK 标准算法，限流 500ms 基础延迟，普通 100ms，指数退避 + 随机抖动

---

## 附录：请求头详解

### 必需请求头

```javascript
const headers = {
  // 认证
  'Authorization': `Bearer ${accessToken}`,
  'Content-Type': 'application/json',
  
  // AWS SDK 标准头（Kiro 源码确认）
  'amz-sdk-invocation-id': crypto.randomUUID(),  // 请求唯一标识
  'amz-sdk-request': 'attempt=1; max=3',         // 重试信息
  
  // Kiro 特定头（Kiro 源码确认）
  'x-amzn-kiro-agent-mode': 'vibe',              // Agent 模式，默认值
  
  // User-Agent（Kiro 源码格式）
  'x-amz-user-agent': `KiroIDE-${version}-${machineId}`
}
```

### User-Agent 格式

Kiro 源码中的 User-Agent 格式：
```
KiroIDE-{版本号}-{机器ID}
```

示例：`KiroIDE-0.1.25-abc123def456...`

### Kiro 版本号获取

```javascript
const { execSync } = require('child_process')
const fs = require('fs')
const path = require('path')

function getKiroVersion() {
  if (process.platform === 'darwin') {
    // macOS: 从 Info.plist 读取
    const plistPaths = [
      '/Applications/Kiro.app/Contents/Info.plist',
      path.join(process.env.HOME, 'Applications/Kiro.app/Contents/Info.plist')
    ]
    
    for (const plistPath of plistPaths) {
      try {
        const output = execSync(`defaults read "${plistPath}" CFBundleShortVersionString`).toString().trim()
        if (output) return output
      } catch {}
    }
  } else if (process.platform === 'win32') {
    // Windows: 从安装目录的 package.json 读取
    const packagePath = path.join(process.env.LOCALAPPDATA, 'Programs/Kiro/resources/app/package.json')
    try {
      const pkg = JSON.parse(fs.readFileSync(packagePath, 'utf-8'))
      if (pkg.version) return pkg.version
    } catch {}
  }
  
  // 默认版本号
  return '0.1.25'
}
```

### Machine ID 生成

Kiro 使用系统硬件 ID 的 SHA256 哈希：

```javascript
const crypto = require('crypto')
const { execSync } = require('child_process')

function getMachineId() {
  let rawId = ''
  
  if (process.platform === 'darwin') {
    // macOS: IOPlatformUUID
    const output = execSync('ioreg -rd1 -c "IOPlatformExpertDevice"').toString()
    const match = output.match(/IOPlatformUUID.*"(.+)"/)
    rawId = match ? match[1].toLowerCase() : ''
  } else if (process.platform === 'linux') {
    // Linux: /etc/machine-id
    rawId = require('fs').readFileSync('/etc/machine-id', 'utf-8').trim()
  } else if (process.platform === 'win32') {
    // Windows: wmic csproduct get UUID
    const output = execSync('wmic csproduct get UUID').toString()
    rawId = output.split('\n')[1]?.trim().toLowerCase() || ''
  }
  
  // SHA256 哈希
  return crypto.createHash('sha256').update(rawId).digest('hex')
}
```

### 凭证文件位置

Kiro 凭证存储在：
```
~/.aws/sso/cache/kiro-auth-token.json
```

IDC 账号的 clientId/clientSecret 存储在：
```
~/.aws/sso/cache/{clientIdHash}.json
```

