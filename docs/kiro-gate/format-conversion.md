# 格式转换

## 概述

KiroGate 需要在三种 API 格式之间转换：

- **OpenAI 格式** - `/v1/chat/completions`
- **Anthropic 格式** - `/v1/messages`
- **Kiro 格式** - `/generateAssistantResponse`

---

## OpenAI → Kiro

### 请求转换

```javascript
// OpenAI 请求
{
  "model": "gpt-4",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant"},
    {"role": "user", "content": "Hello"},
    {"role": "assistant", "content": "Hi there!"},
    {"role": "user", "content": "How are you?"}
  ],
  "stream": true,
  "temperature": 0.7,
  "max_tokens": 4096
}

// 转换为 Kiro 请求
{
  "conversationState": {
    "conversationId": "uuid-xxx",
    "chatTriggerType": "MANUAL",
    "currentMessage": {
      "userInputMessage": {
        "content": ["How are you?"],  // 最后一条 user 消息
        "userIntent": "CODE_GENERATION",
        "userInputMessageContext": {
          "additionalContext": {
            "systemPrompt": "You are a helpful assistant"  // system 消息
          }
        }
      }
    },
    "history": [
      // 之前的消息对
      {
        "userInputMessage": {
          "content": ["Hello"],
          "userIntent": "CODE_GENERATION"
        }
      },
      {
        "assistantResponseMessage": {
          "content": "Hi there!"
        }
      }
    ]
  },
  "profileArn": ""
}
```

### 转换函数

```javascript
function openaiToKiro(openaiRequest) {
  const { messages, model } = openaiRequest
  
  // 提取 system 消息
  const systemMessage = messages.find(m => m.role === 'system')?.content || ''
  
  // 过滤出 user/assistant 消息
  const chatMessages = messages.filter(m => m.role !== 'system')
  
  // 最后一条 user 消息作为 currentMessage
  const lastUserIndex = chatMessages.findLastIndex(m => m.role === 'user')
  const currentMessage = chatMessages[lastUserIndex]?.content || ''
  
  // 之前的消息作为 history
  const historyMessages = chatMessages.slice(0, lastUserIndex)
  const history = []
  
  for (let i = 0; i < historyMessages.length; i += 2) {
    const userMsg = historyMessages[i]
    const assistantMsg = historyMessages[i + 1]
    
    if (userMsg?.role === 'user') {
      history.push({
        userInputMessage: {
          content: [userMsg.content],
          userIntent: 'CODE_GENERATION'
        }
      })
    }
    
    if (assistantMsg?.role === 'assistant') {
      history.push({
        assistantResponseMessage: {
          content: assistantMsg.content
        }
      })
    }
  }
  
  return {
    conversationState: {
      conversationId: crypto.randomUUID(),
      chatTriggerType: 'MANUAL',
      currentMessage: {
        userInputMessage: {
          content: [currentMessage],
          userIntent: 'CODE_GENERATION',
          userInputMessageContext: {
            additionalContext: systemMessage ? { systemPrompt: systemMessage } : {}
          }
        }
      },
      history
    },
    profileArn: ''
  }
}
```

### 响应转换

```javascript
// Kiro SSE 事件
{ "assistantResponseEvent": { "content": "Hello" } }
{ "assistantResponseEvent": { "content": " there" } }
{ "assistantResponseEvent": { "content": "!" } }

// 转换为 OpenAI SSE 格式
data: {"id":"chatcmpl-xxx","object":"chat.completion.chunk","created":1704067200,"model":"kiro","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

data: {"id":"chatcmpl-xxx","object":"chat.completion.chunk","created":1704067200,"model":"kiro","choices":[{"index":0,"delta":{"content":" there"},"finish_reason":null}]}

data: {"id":"chatcmpl-xxx","object":"chat.completion.chunk","created":1704067200,"model":"kiro","choices":[{"index":0,"delta":{"content":"!"},"finish_reason":null}]}

data: {"id":"chatcmpl-xxx","object":"chat.completion.chunk","created":1704067200,"model":"kiro","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
```

### 响应转换函数

```javascript
function kiroEventToOpenAI(kiroEvent, requestId) {
  const timestamp = Math.floor(Date.now() / 1000)
  
  // 文本响应
  if (kiroEvent.assistantResponseEvent) {
    return {
      id: `chatcmpl-${requestId}`,
      object: 'chat.completion.chunk',
      created: timestamp,
      model: 'kiro',
      choices: [{
        index: 0,
        delta: { content: kiroEvent.assistantResponseEvent.content },
        finish_reason: null
      }]
    }
  }
  
  // 结束事件
  if (kiroEvent.endOfStream) {
    return {
      id: `chatcmpl-${requestId}`,
      object: 'chat.completion.chunk',
      created: timestamp,
      model: 'kiro',
      choices: [{
        index: 0,
        delta: {},
        finish_reason: 'stop'
      }]
    }
  }
  
  return null
}
```

---

## Anthropic → Kiro

### 请求转换

```javascript
// Anthropic 请求
{
  "model": "claude-3-5-sonnet-20241022",
  "max_tokens": 4096,
  "system": "You are a helpful assistant",
  "messages": [
    {"role": "user", "content": "Hello"},
    {"role": "assistant", "content": "Hi there!"},
    {"role": "user", "content": "How are you?"}
  ],
  "stream": true
}

// 转换为 Kiro 请求（与 OpenAI 类似）
{
  "conversationState": {
    "conversationId": "uuid-xxx",
    "chatTriggerType": "MANUAL",
    "currentMessage": {
      "userInputMessage": {
        "content": ["How are you?"],
        "userIntent": "CODE_GENERATION",
        "userInputMessageContext": {
          "additionalContext": {
            "systemPrompt": "You are a helpful assistant"
          }
        }
      }
    },
    "history": [...]
  },
  "profileArn": ""
}
```

### 转换函数

```javascript
function anthropicToKiro(anthropicRequest) {
  const { messages, system } = anthropicRequest
  
  // Anthropic 的 system 是单独字段
  const systemPrompt = system || ''
  
  // 最后一条 user 消息
  const lastUserIndex = messages.findLastIndex(m => m.role === 'user')
  const currentMessage = messages[lastUserIndex]?.content || ''
  
  // 构建 history
  const historyMessages = messages.slice(0, lastUserIndex)
  const history = []
  
  for (let i = 0; i < historyMessages.length; i += 2) {
    const userMsg = historyMessages[i]
    const assistantMsg = historyMessages[i + 1]
    
    if (userMsg?.role === 'user') {
      history.push({
        userInputMessage: {
          content: [extractContent(userMsg.content)],
          userIntent: 'CODE_GENERATION'
        }
      })
    }
    
    if (assistantMsg?.role === 'assistant') {
      history.push({
        assistantResponseMessage: {
          content: extractContent(assistantMsg.content)
        }
      })
    }
  }
  
  return {
    conversationState: {
      conversationId: crypto.randomUUID(),
      chatTriggerType: 'MANUAL',
      currentMessage: {
        userInputMessage: {
          content: [extractContent(currentMessage)],
          userIntent: 'CODE_GENERATION',
          userInputMessageContext: {
            additionalContext: systemPrompt ? { systemPrompt } : {}
          }
        }
      },
      history
    },
    profileArn: ''
  }
}

// Anthropic content 可能是字符串或数组
function extractContent(content) {
  if (typeof content === 'string') return content
  if (Array.isArray(content)) {
    return content
      .filter(c => c.type === 'text')
      .map(c => c.text)
      .join('')
  }
  return ''
}
```

### 响应转换

```javascript
// Kiro SSE 事件
{ "assistantResponseEvent": { "content": "Hello" } }

// 转换为 Anthropic SSE 格式
event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}

// 结束
event: message_stop
data: {"type":"message_stop"}
```

### 响应转换函数

```javascript
function kiroEventToAnthropic(kiroEvent, requestId) {
  // 文本响应
  if (kiroEvent.assistantResponseEvent) {
    return {
      event: 'content_block_delta',
      data: {
        type: 'content_block_delta',
        index: 0,
        delta: {
          type: 'text_delta',
          text: kiroEvent.assistantResponseEvent.content
        }
      }
    }
  }
  
  // 结束事件
  if (kiroEvent.endOfStream) {
    return {
      event: 'message_stop',
      data: { type: 'message_stop' }
    }
  }
  
  return null
}
```

---

## 完整转换示例

```javascript
class FormatConverter {
  // OpenAI → Kiro
  static openaiToKiro(request) {
    // ... 上面的实现
  }
  
  // Anthropic → Kiro
  static anthropicToKiro(request) {
    // ... 上面的实现
  }
  
  // Kiro → OpenAI (流式)
  static kiroToOpenaiStream(kiroEvent, requestId) {
    // ... 上面的实现
  }
  
  // Kiro → Anthropic (流式)
  static kiroToAnthropicStream(kiroEvent, requestId) {
    // ... 上面的实现
  }
  
  // 检测请求格式
  static detectFormat(request, path) {
    if (path.includes('/v1/chat/completions')) return 'openai'
    if (path.includes('/v1/messages')) return 'anthropic'
    return 'unknown'
  }
}
```

---

## 注意事项

### Kiro API 请求结构（源码确认）

Kiro 的 `generateAssistantResponse` 接口只接受两个顶层字段：

```javascript
// 源码: se_GenerateAssistantResponseCommand
body = JSON.stringify({
  "conversationState": { ... },
  "profileArn": ""
});
```

**这意味着 OpenAI/Anthropic 的以下参数没有地方放，直接忽略即可**：

**OpenAI 参数**（[官方文档](https://platform.openai.com/docs/api-reference/chat)）：
- `temperature` - 采样温度，范围 0-2
- `top_p` - 核采样参数
- `max_tokens` / `max_completion_tokens` - 最大输出长度
- `frequency_penalty` - 频率惩罚，范围 -2.0 到 2.0
- `presence_penalty` - 存在惩罚，范围 -2.0 到 2.0
- `logprobs` - 是否返回日志概率
- `logit_bias` - 词元偏置映射
- `n` - 生成多个响应
- `stream_options` - 流式选项（含 `include_usage`）
- `parallel_tool_calls` - 并行工具调用
- `metadata` - 请求元数据（最多 16 个键值对）
- `reasoning_effort` - 推理努力程度
- `prediction` - 预测输出配置

**Anthropic 参数**（[官方文档](https://platform.claude.com/docs/en/api/messages)）：
- `temperature` - 采样温度
- `top_p` - 核采样参数
- `top_k` - 只从前 K 个最可能的 token 中采样
- `max_tokens` - 最大输出长度（必填）
- `stop_sequences` - 停止序列数组
- `metadata` - 请求元数据（含 `user_id`）

### 需要转换的功能

- `tools` / `functions` - 需要格式转换（见工具调用章节）
- `vision` (图片输入) - Kiro 支持，但格式不同（见高级转换章节）
- `system` - OpenAI 放在 messages 里，Anthropic 是单独字段，Kiro 放在 `additionalContext.systemPrompt`
- `stream` - 两边都有，Kiro 默认流式

### 消息格式差异

- OpenAI: `messages` 数组，`role` 可以是 `system/user/assistant/tool`
- Anthropic: `messages` 数组 + 单独的 `system` 字段
- Kiro: `currentMessage` + `history` 分离，`system` 在 `additionalContext` 中

### 流式响应差异

- OpenAI: `data: {json}\n\n` 格式，最后 `data: [DONE]`
- Anthropic: `event: xxx\ndata: {json}\n\n` 格式
- Kiro: 类似 OpenAI 的 `data: {json}\n\n` 格式

### Anthropic Beta 功能

Anthropic 有一些 Beta 功能需要特殊请求头：

```javascript
// Anthropic 缓存功能
headers['anthropic-beta'] = 'prompt-caching-2024-07-31'

// Anthropic 文件 API
headers['anthropic-beta'] = 'files-api-2025-04-14'
```

KiroGate 不需要处理这些，因为 Kiro 有自己的缓存机制。

### Anthropic 新工具类型

Anthropic 支持一些特殊工具类型，KiroGate 需要识别但可能无法完全转换：

- `server_tool_use` - 服务端工具调用
- `mcp_tool_use` - MCP 工具调用
- `web_search_tool_result` - 网页搜索结果

这些类型在 Kiro 中没有直接对应，建议转换为普通 `tool_use`。
