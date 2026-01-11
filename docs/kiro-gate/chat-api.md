# Kiro 对话 API 完整指南

## 概述

Kiro 的对话接口基于 AWS CodeWhisperer Streaming Service，支持 **Bearer Token 认证**。

**端点**: 
- 普通: `https://codewhisperer.us-east-1.amazonaws.com`
- 流式: `https://amazoncodewhispererstreamingservice.us-east-1.amazonaws.com`

**认证**: `Authorization: Bearer {accessToken}`

**必需请求头**（Kiro 源码确认）:
- `Authorization: Bearer {accessToken}` - 认证 Token
- `x-amzn-kiro-agent-mode: vibe` - Agent 模式（默认值）
- `x-amz-user-agent: KiroIDE-{版本}-{机器ID}` - 用户代理标识
- `amz-sdk-invocation-id: {UUID}` - 请求唯一标识
- `amz-sdk-request: attempt=1; max=3` - 重试信息

---

## 完整调用流程

```
┌─────────────────────────────────────────────────────────────────┐
│                      1. Token 管理                               │
├─────────────────────────────────────────────────────────────────┤
│  - 检查 accessToken 是否过期（1小时有效期）                       │
│  - 过期则用 refreshToken 刷新                                    │
│  - 刷新失败则需要重新登录                                        │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   2. 发送对话请求                                │
├─────────────────────────────────────────────────────────────────┤
│  POST /generateAssistantResponse                                │
│  - 构建 conversationState                                       │
│  - 包含 tools 定义（如果需要工具调用）                           │
│  - 包含 history（多轮对话）                                      │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   3. 处理 SSE 流式响应                           │
├─────────────────────────────────────────────────────────────────┤
│  事件类型:                                                       │
│  - messageMetadataEvent: 消息元数据                              │
│  - assistantResponseEvent: AI 文本响应                           │
│  - codeEvent: 代码块                                            │
│  - toolUseEvent: 工具调用请求 ← 需要处理                         │
│  - contextUsageEvent: 上下文使用量 ← 需要监控                    │
│  - followupPromptEvent: 后续提示                                │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   4. 工具调用循环                                │
├─────────────────────────────────────────────────────────────────┤
│  收到 toolUseEvent 时:                                          │
│  1. 执行工具（readFile、executePwsh 等）                         │
│  2. 构建 toolResult                                             │
│  3. 发送新请求，history 包含工具调用和结果                       │
│  4. 重复直到没有 toolUseEvent                                   │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   5. 上下文压缩检查                              │
├─────────────────────────────────────────────────────────────────┤
│  contextUsagePercentage >= 80% 时:                              │
│  - 触发自动总结（summarization）                                 │
│  - 压缩历史消息                                                  │
│  - 保留关键上下文                                                │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│                   6. 错误处理                                    │
├─────────────────────────────────────────────────────────────────┤
│  - 401: Token 过期 → 刷新 Token 重试                            │
│  - 429: 限流 → 指数退避重试                                     │
│  - 500: 服务错误 → 重试 3 次                                    │
│  - ServiceQuotaExceeded: 配额用尽 → 提示用户                    │
└─────────────────────────────────────────────────────────────────┘
```

---

## 1. Token 管理

### 1.1 Token 结构

```javascript
// Kiro Token 结构
{
  accessToken: "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",  // JWT，1小时有效
  refreshToken: "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...", // 用于刷新
  expiresIn: 3600,  // 秒
  profileArn: ""    // 企业版用
}
```

### 1.2 Token 过期检测

```javascript
// 源码位置: @smithy/core/dist-es/util-identity-and-auth/tokenIdentity.js

const EXPIRATION_MS = 5 * 60 * 1000  // 提前 5 分钟判定过期

function isIdentityExpired(identity) {
  if (!identity.expiration) return false
  const now = Date.now()
  const expireTime = identity.expiration.getTime()
  return now >= expireTime - EXPIRATION_MS
}

// 自动刷新逻辑
memoizeIdentityProvider = (provider, isExpired, requiresRefresh) => {
  let resolved
  return async (options) => {
    if (!resolved || options?.forceRefresh || isExpired(resolved)) {
      resolved = await provider(options)
    }
    return resolved
  }
}
```

### 1.3 刷新 Token

```javascript
// 端点: POST https://prod.us-east-1.auth.desktop.kiro.dev/refreshToken

async function refreshAccessToken(refreshToken) {
  const response = await fetch(
    'https://prod.us-east-1.auth.desktop.kiro.dev/refreshToken',
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refreshToken })
    }
  )
  
  if (!response.ok) {
    if (response.status === 401) {
      throw new Error('RefreshToken 已过期，需要重新登录')
    }
    throw new Error(`刷新失败: ${response.status}`)
  }
  
  return await response.json()
  // { accessToken: "...", expiresIn: 3600 }
}
```

---

## 2. 请求结构

### 2.1 基础请求

```javascript
const request = {
  conversationState: {
    conversationId: crypto.randomUUID(),  // 会话 ID
    chatTriggerType: 'MANUAL',            // MANUAL | AUTO
    currentMessage: {
      userInputMessage: {
        content: ['用户消息'],
        userIntent: 'CODE_GENERATION',
        userInputMessageContext: {
          tools: [],           // 工具定义
          editorState: {},     // 编辑器状态
          additionalContext: {} // 额外上下文
        }
      }
    },
    history: []  // 历史消息
  },
  profileArn: ''
}
```

### 2.2 userIntent 枚举

```javascript
// 源码位置: packages/@amzn/codewhisperer-streaming/dist-cjs/models/models_0.js

const UserIntent = {
  CODE_GENERATION: 'CODE_GENERATION',
  EXPLAIN_CODE_SELECTION: 'EXPLAIN_CODE_SELECTION',
  SUGGEST_ALTERNATE_IMPLEMENTATION: 'SUGGEST_ALTERNATE_IMPLEMENTATION',
  APPLY_COMMON_BEST_PRACTICES: 'APPLY_COMMON_BEST_PRACTICES',
  IMPROVE_CODE: 'IMPROVE_CODE',
  CITE_SOURCES: 'CITE_SOURCES',
  FIX_CODE: 'FIX_CODE',
  GENERATE_UNIT_TESTS: 'GENERATE_UNIT_TESTS'
}
```

### 2.3 origin 枚举（Kiro 源码确认）

```javascript
// 源码位置: packages/@amzn/codewhisperer-streaming/dist-cjs/models/models_0.js
// 用于 ListAvailableModels、getUsageLimits 等接口的 origin 参数

const Origin = {
  AI_EDITOR: 'AI_EDITOR',           // Kiro 主要使用
  CHATBOT: 'CHATBOT',
  CLI: 'CLI',
  CONSOLE: 'CONSOLE',
  DOCUMENTATION: 'DOCUMENTATION',
  GITLAB: 'GITLAB',
  IDE: 'IDE',
  INLINE_CHAT: 'INLINE_CHAT',
  KIRO_CLI: 'KIRO_CLI',
  MARKETING: 'MARKETING',
  MD: 'MD',
  MD_CE: 'MD_CE',
  MD_IDE: 'MD_IDE',
  MOBILE: 'MOBILE',
  OPENSEARCH_DASHBOARD: 'OPENSEARCH_DASHBOARD',
  Q_DEV_BEXT: 'Q_DEV_BEXT',
  SAGE_MAKER: 'SAGE_MAKER',
  SERVICE_INTERNAL: 'SERVICE_INTERNAL',
  SM_AI_STUDIO_IDE: 'SM_AI_STUDIO_IDE',
  UNIFIED_SEARCH: 'UNIFIED_SEARCH',
  UNKNOWN: 'UNKNOWN'
}

// KiroGate 建议使用 AI_EDITOR
```
```

### 2.4 工具定义

```javascript
const tools = [
  {
    toolSpec: {
      name: 'readFile',
      description: '读取文件内容',
      inputSchema: {
        json: {
          type: 'object',
          properties: {
            path: { type: 'string', description: '文件路径' }
          },
          required: ['path']
        }
      }
    }
  },
  {
    toolSpec: {
      name: 'executePwsh',
      description: '执行 PowerShell 命令',
      inputSchema: {
        json: {
          type: 'object',
          properties: {
            command: { type: 'string' },
            path: { type: 'string' }
          },
          required: ['command']
        }
      }
    }
  }
]
```

---

## 3. SSE 响应处理

### 3.1 事件类型

```javascript
// 源码位置: packages/@amzn/codewhisperer-streaming/dist-cjs/models/models_0.js

const ChatResponseStream = {
  messageMetadataEvent: {},      // 消息元数据
  assistantResponseEvent: {},    // AI 文本响应
  codeEvent: {},                 // 代码块
  toolUseEvent: {},              // 工具调用请求
  toolResultEvent: {},           // 工具调用结果
  contextUsageEvent: {},         // 上下文使用量
  followupPromptEvent: {},       // 后续提示
  intentsEvent: {},              // 意图检测
  citationEvent: {},             // 引用信息
  reasoningContentEvent: {},     // 推理内容
  interactionComponentsEvent: {} // 交互组件
}
```

### 3.2 解析 SSE 流

```javascript
async function* parseSSEStream(response) {
  const reader = response.body.getReader()
  const decoder = new TextDecoder()
  let buffer = ''
  
  while (true) {
    const { done, value } = await reader.read()
    if (done) break
    
    buffer += decoder.decode(value, { stream: true })
    const lines = buffer.split('\n')
    buffer = lines.pop() || ''
    
    for (const line of lines) {
      if (line.startsWith('data: ')) {
        try {
          const data = JSON.parse(line.slice(6))
          yield data
        } catch (e) {
          // 忽略解析错误
        }
      }
    }
  }
}

// 使用示例
for await (const event of parseSSEStream(response)) {
  if (event.assistantResponseEvent) {
    process.stdout.write(event.assistantResponseEvent.content)
  }
  if (event.toolUseEvent) {
    // 处理工具调用
  }
  if (event.contextUsageEvent) {
    // 监控上下文使用量
  }
}
```

### 3.3 contextUsageEvent 结构

```javascript
// 上下文使用量事件
{
  contextUsagePercentage: 45.5,    // 使用百分比
  totalTokens: 12000,              // 总 Token 数
  outputTokens: 500,               // 输出 Token
  uncachedInputTokens: 8000,       // 未缓存输入
  cacheReadInputTokens: 3000,      // 缓存读取
  cacheWriteInputTokens: 500       // 缓存写入
}
```

---

## 4. 工具调用循环

### 4.1 工具调用流程

```
用户消息 → AI 响应 → toolUseEvent → 执行工具 → toolResult → AI 继续响应 → ...
```

### 4.2 toolUseEvent 结构

```javascript
{
  toolUseId: 'uuid-xxx',
  name: 'readFile',
  input: {
    path: 'src/main.js'
  }
}
```

### 4.3 构建 toolResult

```javascript
function buildToolResult(toolUseId, result, status = 'success') {
  return {
    toolResult: {
      toolUseId,
      status,  // 'success' | 'error'
      content: [
        {
          text: typeof result === 'string' ? result : JSON.stringify(result)
        }
      ]
    }
  }
}
```

### 4.4 完整工具调用循环

```javascript
async function chatWithTools(message, tools, conversationId) {
  let history = []
  let currentMessage = message
  
  while (true) {
    // 发送请求
    const response = await sendChatRequest({
      conversationId,
      message: currentMessage,
      tools,
      history
    })
    
    // 收集响应
    let assistantContent = ''
    let toolUses = []
    let contextUsage = null
    
    for await (const event of parseSSEStream(response)) {
      if (event.assistantResponseEvent) {
        assistantContent += event.assistantResponseEvent.content
      }
      if (event.toolUseEvent) {
        toolUses.push(event.toolUseEvent)
      }
      if (event.contextUsageEvent) {
        contextUsage = event.contextUsageEvent
      }
    }
    
    // 检查上下文使用量
    if (contextUsage?.contextUsagePercentage >= 80) {
      console.warn('上下文使用量超过 80%，需要压缩')
      // 触发总结压缩
    }
    
    // 没有工具调用，结束循环
    if (toolUses.length === 0) {
      return { content: assistantContent, history }
    }
    
    // 执行工具调用
    const toolResults = []
    for (const toolUse of toolUses) {
      const result = await executeToolCall(toolUse)
      toolResults.push(buildToolResult(toolUse.toolUseId, result))
    }
    
    // 更新历史
    history.push(
      { userInputMessage: { content: [currentMessage], userIntent: 'CODE_GENERATION' } },
      { assistantResponseMessage: { content: assistantContent, toolUse: toolUses } }
    )
    
    // 下一轮请求带上工具结果
    currentMessage = toolResults
  }
}
```

### 4.5 工具调用失败检测

```javascript
// 源码位置: dist/extension.js 第 671436 行

// Kiro 会检测工具调用循环失败
const TOOL_FAIL_THRESHOLD = 3      // 普通工具失败阈值
const MCP_TOOL_FAIL_THRESHOLD = 5  // MCP 工具失败阈值

function checkFailingToolCallLoop(context, toolType = 'normal', threshold = TOOL_FAIL_THRESHOLD) {
  const recentToolCalls = context.messages
    .filter(m => m.toolUse)
    .slice(-threshold)
  
  // 检查是否连续失败
  const allFailed = recentToolCalls.every(call => 
    call.toolResult?.status === 'error'
  )
  
  if (allFailed && recentToolCalls.length >= threshold) {
    return {
      detected: true,
      reason: `工具 ${toolType} 连续失败 ${threshold} 次`
    }
  }
  
  return null
}

// 检查重复工具调用
function checkRepetitiveToolUse(context) {
  const recentCalls = context.messages
    .filter(m => m.toolUse)
    .slice(-5)
    .map(m => JSON.stringify({ name: m.toolUse.name, input: m.toolUse.input }))
  
  // 检查是否有重复
  const unique = new Set(recentCalls)
  if (unique.size === 1 && recentCalls.length >= 3) {
    return {
      detected: true,
      reason: '检测到重复的工具调用'
    }
  }
  
  return null
}
```

---

## 5. 上下文压缩（自动总结）

### 5.1 触发条件

```javascript
// 源码位置: dist/extension.js

// 上下文使用量超过 80% 时触发自动总结
const SUMMARIZATION_THRESHOLD = 80

function shouldTriggerSummarization(contextUsagePercentage) {
  return contextUsagePercentage >= SUMMARIZATION_THRESHOLD
}
```

### 5.2 总结逻辑

```javascript
// 源码位置: dist/extension.js 第 161732 行

function summarize(message) {
  if (Array.isArray(message)) {
    return `${stripImages(message).substring(0, 100)}...`
  }
  return message.substring(0, 100) + '...'
}

// 压缩历史消息
function compressHistory(chatHistory, modelName, maxTokens) {
  let totalTokens = countTokens(chatHistory, modelName)
  let i = 0
  
  // 从最早的消息开始压缩
  while (totalTokens > maxTokens && i < chatHistory.length) {
    const message = chatHistory[i]
    totalTokens -= countTokens(message.content, modelName)
    totalTokens += countTokens(summarize(message.content), modelName)
    message.content = summarize(message.content)
    i++
  }
  
  return chatHistory
}
```

### 5.3 手动触发总结

```javascript
// 发送 /compact 命令或调用总结接口
async function triggerSummarization(conversationId, history) {
  // 构建总结请求
  const summaryRequest = {
    conversationState: {
      conversationId,
      chatTriggerType: 'AUTO',
      currentMessage: {
        userInputMessage: {
          content: ['请总结以上对话的关键信息'],
          userIntent: 'CODE_GENERATION'
        }
      },
      history
    },
    profileArn: ''
  }
  
  // 发送请求获取总结
  const response = await sendChatRequest(summaryRequest)
  
  // 用总结替换历史
  return [{
    assistantResponseMessage: {
      content: `[会话总结] ${await collectResponse(response)}`
    }
  }]
}
```

---

## 6. 错误处理

### 6.1 错误类型

```javascript
// 源码位置: packages/@amzn/codewhisperer-streaming/dist-cjs/models/models_0.js

const ErrorTypes = {
  AccessDeniedException: {
    status: 403,
    reason: 'ACCESS_DENIED',
    action: '检查 Token 权限'
  },
  ThrottlingException: {
    status: 429,
    reason: 'THROTTLING',
    action: '指数退避重试'
  },
  ServiceQuotaExceededException: {
    status: 429,
    reason: 'QUOTA_EXCEEDED',
    action: '配额用尽，等待重置或升级'
  },
  ValidationException: {
    status: 400,
    reason: 'VALIDATION_ERROR',
    action: '检查请求参数'
  },
  InternalServerException: {
    status: 500,
    reason: 'INTERNAL_ERROR',
    action: '重试 3 次'
  },
  ServiceUnavailableException: {
    status: 503,
    reason: 'SERVICE_UNAVAILABLE',
    action: '稍后重试'
  },
  ExpiredTokenException: {
    status: 401,
    reason: 'TOKEN_EXPIRED',
    action: '刷新 Token'
  }
}
```

### 6.2 重试策略

```javascript
// 源码位置: @aws-sdk/middleware-retry

const THROTTLING_RETRY_DELAY_BASE = 500   // 限流重试基础延迟 500ms
const DEFAULT_RETRY_DELAY_BASE = 100      // 默认重试基础延迟 100ms
const MAXIMUM_RETRY_DELAY = 20000         // 最大延迟 20s
const INITIAL_RETRY_TOKENS = 500          // 初始重试配额

// 指数退避
function exponentialBackoff(delayBase, attempts) {
  return Math.floor(
    Math.min(MAXIMUM_RETRY_DELAY, Math.random() * 2 ** attempts * delayBase)
  )
}

// 判断是否应该重试
function shouldRetry(error, attempts, maxAttempts = 3) {
  if (attempts >= maxAttempts) return false
  
  const retryableErrors = [
    'ThrottlingException',
    'TooManyRequestsException',
    'InternalServerException',
    'ServiceUnavailableException',
    'TransientError'
  ]
  
  return retryableErrors.includes(error.name)
}
```

### 6.3 完整错误处理

```javascript
async function chatWithRetry(message, options = {}) {
  const maxRetries = options.maxRetries || 3
  let lastError
  
  for (let attempt = 0; attempt < maxRetries; attempt++) {
    try {
      return await sendChatRequest(message, options)
    } catch (error) {
      lastError = error
      
      // Token 过期
      if (error.name === 'ExpiredTokenException' || error.status === 401) {
        console.log('Token 过期，正在刷新...')
        options.accessToken = await refreshAccessToken(options.refreshToken)
        continue
      }
      
      // 限流
      if (error.name === 'ThrottlingException' || error.status === 429) {
        const delay = exponentialBackoff(THROTTLING_RETRY_DELAY_BASE, attempt)
        console.log(`限流，等待 ${delay}ms 后重试...`)
        await sleep(delay)
        continue
      }
      
      // 配额用尽
      if (error.name === 'ServiceQuotaExceededException') {
        throw new Error('配额已用尽，请等待重置或升级套餐')
      }
      
      // 服务错误
      if (error.status >= 500) {
        const delay = exponentialBackoff(DEFAULT_RETRY_DELAY_BASE, attempt)
        console.log(`服务错误，等待 ${delay}ms 后重试...`)
        await sleep(delay)
        continue
      }
      
      // 其他错误不重试
      throw error
    }
  }
  
  throw lastError
}
```

---

## 7. 完整客户端实现

### 7.1 KiroChatClient 类

```javascript
class KiroChatClient {
  constructor(options) {
    this.accessToken = options.accessToken
    this.refreshToken = options.refreshToken
    this.endpoint = options.endpoint || 'https://codewhisperer.us-east-1.amazonaws.com'
    this.maxRetries = options.maxRetries || 3
    this.tools = options.tools || []
    
    // Token 过期时间（提前 5 分钟）
    this.tokenExpireTime = Date.now() + (options.expiresIn || 3600) * 1000 - 5 * 60 * 1000
  }
  
  // 检查并刷新 Token
  async ensureValidToken() {
    if (Date.now() >= this.tokenExpireTime) {
      console.log('Token 即将过期，正在刷新...')
      const result = await this.refreshAccessToken()
      this.accessToken = result.accessToken
      this.tokenExpireTime = Date.now() + result.expiresIn * 1000 - 5 * 60 * 1000
    }
  }
  
  async refreshAccessToken() {
    const response = await fetch(
      'https://prod.us-east-1.auth.desktop.kiro.dev/refreshToken',
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ refreshToken: this.refreshToken })
      }
    )
    
    if (!response.ok) {
      throw new Error('Token 刷新失败，需要重新登录')
    }
    
    return await response.json()
  }
  
  // 发送请求
  async sendRequest(conversationState) {
    await this.ensureValidToken()
    
    const response = await fetch(`${this.endpoint}/generateAssistantResponse`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.accessToken}`
      },
      body: JSON.stringify({
        conversationState,
        profileArn: ''
      })
    })
    
    if (!response.ok) {
      const error = new Error(`请求失败: ${response.status}`)
      error.status = response.status
      error.name = response.headers.get('x-amzn-errortype')?.split(':')[0]
      throw error
    }
    
    return response
  }
  
  // 解析 SSE 流
  async *parseStream(response) {
    const reader = response.body.getReader()
    const decoder = new TextDecoder()
    let buffer = ''
    
    while (true) {
      const { done, value } = await reader.read()
      if (done) break
      
      buffer += decoder.decode(value, { stream: true })
      const lines = buffer.split('\n')
      buffer = lines.pop() || ''
      
      for (const line of lines) {
        if (line.startsWith('data: ')) {
          try {
            yield JSON.parse(line.slice(6))
          } catch (e) {}
        }
      }
    }
  }
```

  // 执行工具调用
  async executeToolCall(toolUse) {
    const { name, input } = toolUse
    
    switch (name) {
      case 'readFile':
        return await fs.readFile(input.path, 'utf-8')
      case 'executePwsh':
        return await execCommand(input.command, input.path)
      case 'listDirectory':
        return await fs.readdir(input.path)
      // ... 其他工具
      default:
        return { error: `未知工具: ${name}` }
    }
  }
  
  // 主对话方法
  async chat(message, options = {}) {
    const conversationId = options.conversationId || crypto.randomUUID()
    let history = options.history || []
    let contextUsagePercentage = 0
    
    for (let retry = 0; retry < this.maxRetries; retry++) {
      try {
        // 构建请求
        const conversationState = {
          conversationId,
          chatTriggerType: 'MANUAL',
          currentMessage: {
            userInputMessage: {
              content: Array.isArray(message) ? message : [message],
              userIntent: options.userIntent || 'CODE_GENERATION',
              userInputMessageContext: {
                tools: this.tools.map(t => ({ toolSpec: t }))
              }
            }
          },
          history
        }
        
        // 发送请求
        const response = await this.sendRequest(conversationState)
        
        // 收集响应
        let content = ''
        let toolUses = []
        
        for await (const event of this.parseStream(response)) {
          // 文本响应
          if (event.assistantResponseEvent) {
            content += event.assistantResponseEvent.content
            if (options.onContent) {
              options.onContent(event.assistantResponseEvent.content)
            }
          }
          
          // 工具调用
          if (event.toolUseEvent) {
            toolUses.push(event.toolUseEvent)
          }
          
          // 上下文使用量
          if (event.contextUsageEvent) {
            contextUsagePercentage = event.contextUsageEvent.contextUsagePercentage
            if (options.onContextUsage) {
              options.onContextUsage(event.contextUsageEvent)
            }
          }
        }
        
        // 检查是否需要压缩
        if (contextUsagePercentage >= 80) {
          console.warn(`上下文使用量: ${contextUsagePercentage}%，建议压缩`)
          if (options.autoCompress) {
            history = await this.compressHistory(history)
          }
        }
        
        // 没有工具调用，返回结果
        if (toolUses.length === 0) {
          return {
            conversationId,
            content,
            history,
            contextUsagePercentage
          }
        }
        
        // 执行工具调用
        const toolResults = []
        for (const toolUse of toolUses) {
          if (options.onToolUse) {
            options.onToolUse(toolUse)
          }
          
          try {
            const result = await this.executeToolCall(toolUse)
            toolResults.push({
              toolResult: {
                toolUseId: toolUse.toolUseId,
                status: 'success',
                content: [{ text: String(result) }]
              }
            })
          } catch (e) {
            toolResults.push({
              toolResult: {
                toolUseId: toolUse.toolUseId,
                status: 'error',
                content: [{ text: e.message }]
              }
            })
          }
        }
        
        // 更新历史，继续对话
        history.push(
          { userInputMessage: { content: [message], userIntent: 'CODE_GENERATION' } },
          { assistantResponseMessage: { content, toolUse: toolUses } }
        )
        message = toolResults
        
      } catch (error) {
        // 错误处理
        if (error.status === 401) {
          await this.ensureValidToken()
          continue
        }
        if (error.status === 429) {
          const delay = Math.min(20000, 500 * Math.pow(2, retry))
          await new Promise(r => setTimeout(r, delay))
          continue
        }
        throw error
      }
    }
    
    throw new Error('超过最大重试次数')
  }
  
  // 压缩历史
  async compressHistory(history) {
    // 保留最近 5 条，其余压缩
    if (history.length <= 5) return history
    
    const toCompress = history.slice(0, -5)
    const toKeep = history.slice(-5)
    
    // 生成摘要
    const summary = toCompress.map(m => {
      const content = m.userInputMessage?.content || m.assistantResponseMessage?.content
      return String(content).substring(0, 100)
    }).join('\n')
    
    return [
      { assistantResponseMessage: { content: `[历史摘要]\n${summary}` } },
      ...toKeep
    ]
  }
}
```

### 7.2 使用示例

```javascript
// 初始化客户端
const client = new KiroChatClient({
  accessToken: 'eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...',
  refreshToken: 'eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...',
  expiresIn: 3600,
  tools: [
    {
      name: 'readFile',
      description: '读取文件内容',
      inputSchema: {
        json: {
          type: 'object',
          properties: { path: { type: 'string' } },
          required: ['path']
        }
      }
    }
  ]
})

// 简单对话
const result = await client.chat('帮我写一个 Hello World')
console.log(result.content)

// 带回调的对话
const result2 = await client.chat('读取 package.json 并解释', {
  onContent: (text) => process.stdout.write(text),
  onToolUse: (tool) => console.log(`\n调用工具: ${tool.name}`),
  onContextUsage: (usage) => console.log(`上下文: ${usage.contextUsagePercentage}%`),
  autoCompress: true
})

// 多轮对话
let conversationId = null
let history = []

const r1 = await client.chat('创建一个 Express 服务器', { conversationId, history })
conversationId = r1.conversationId
history = r1.history

const r2 = await client.chat('添加一个 /api/users 路由', { conversationId, history })
history = r2.history

console.log(r2.content)
```

---

## 8. 注意事项

### 8.1 Token 管理
- accessToken 有效期 1 小时
- 提前 5 分钟刷新
- refreshToken 过期需要重新登录

### 8.2 工具调用
- 工具调用可能循环多次
- 检测失败循环（连续 3 次失败）
- 检测重复调用

### 8.3 上下文管理
- 监控 contextUsagePercentage
- 超过 80% 触发压缩
- 保留关键上下文

### 8.4 错误处理
- 401: 刷新 Token
- 429: 指数退避重试
- 500: 重试 3 次
- 配额用尽: 提示用户

---

## 相关文档

- [Kiro 认证服务](./kiro-endpoints.md) - Token 获取和刷新
- [CodeWhisperer 数据结构](./codewhisperer/data-structures.md) - 完整数据结构
- [Agent Chat 系统](../systems/agent/chat.md) - Kiro 内部对话架构
- [LangGraph 状态图](../internals/langgraph.md) - 对话流程状态管理
