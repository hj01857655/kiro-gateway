# Kiro SSE 事件类型

## 概述

Kiro API 返回的 SSE 流包含多种事件类型，KiroGate 需要将它们转换为 OpenAI/Anthropic 格式。

---

## 事件类型一览

```
Kiro SSE 事件                    转换目标
─────────────────────────────────────────────────────
messageMetadataEvent      →    (内部使用，不输出)
metadataEvent             →    usage 字段（精确 token 数）
assistantResponseEvent    →    文本内容
codeEvent                 →    文本内容（代码块）
toolUseEvent              →    工具调用 (tool_calls/tool_use)
reasoningContentEvent     →    推理内容 (reasoning_content/thinking)
dryRunSucceedEvent        →    (干运行成功，不输出)
supplementaryWebLinksEvent →   (忽略或附加到内容)
citationEvent             →    (忽略或附加到内容)
followupPromptEvent       →    (忽略)
contextUsageEvent         →    usage 字段（只有百分比，需估算）
invalidStateEvent         →    错误响应
endOfStream               →    finish_reason: stop/tool_calls/length
```

---

## 详细事件结构

### 1. messageMetadataEvent

消息元数据，包含消息 ID 和时间戳。

```json
{
  "messageMetadataEvent": {
    "conversationId": "uuid-xxx",
    "messageId": "msg-xxx",
    "timestamp": "2026-01-11T12:00:00Z"
  }
}
```

**处理方式**: 提取 `messageId` 用于响应的 `id` 字段，不输出到流。

---

### 2. assistantResponseEvent

AI 文本响应，最常见的事件。

```json
{
  "assistantResponseEvent": {
    "content": "这是一段文本"
  }
}
```

**转换为 OpenAI**:
```json
{
  "id": "chatcmpl-xxx",
  "object": "chat.completion.chunk",
  "created": 1704960000,
  "model": "kiro",
  "choices": [{
    "index": 0,
    "delta": { "content": "这是一段文本" },
    "finish_reason": null
  }]
}
```

**转换为 Anthropic**:
```
event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"这是一段文本"}}
```

---

### 3. codeEvent

代码块事件，包含语言和代码内容。

```json
{
  "codeEvent": {
    "language": "javascript",
    "content": "console.log('hello')"
  }
}
```

**处理方式**: 转换为 Markdown 代码块格式，作为文本内容输出。

```javascript
function handleCodeEvent(event) {
  const { language, content } = event.codeEvent
  const markdown = `\`\`\`${language || ''}\n${content}\n\`\`\``
  return { content: markdown }
}
```

---

### 4. supplementaryWebLinksEvent

引用的网页链接。

```json
{
  "supplementaryWebLinksEvent": {
    "links": [
      {
        "title": "MDN Web Docs",
        "url": "https://developer.mozilla.org/..."
      }
    ]
  }
}
```

**处理方式**: 可选，附加到响应末尾或忽略。

---

### 5. citationEvent

引用信息。

```json
{
  "citationEvent": {
    "citations": [
      {
        "source": "file.js",
        "line": 42,
        "content": "..."
      }
    ]
  }
}
```

**处理方式**: 可选，附加到响应末尾或忽略。

---

### 6. followupPromptEvent

后续提示建议。

```json
{
  "followupPromptEvent": {
    "prompts": [
      "你想了解更多关于...",
      "需要我解释..."
    ]
  }
}
```

**处理方式**: 忽略，不输出到流。

---

### 7. metadataEvent

Token 使用量统计（精确数据）。

```json
{
  "metadataEvent": {
    "tokenUsage": {
      "outputTokens": 500,
      "totalTokens": 8500,
      "uncachedInputTokens": 6000,
      "cacheReadInputTokens": 2000,
      "cacheWriteInputTokens": 0,
      "contextUsagePercentage": 4.25
    }
  }
}
```

**字段说明**：
- `outputTokens` - 输出 token 数
- `totalTokens` - 总 token 数
- `uncachedInputTokens` - 未缓存的输入 token
- `cacheReadInputTokens` - 从缓存读取的输入 token
- `cacheWriteInputTokens` - 写入缓存的输入 token
- `contextUsagePercentage` - 上下文使用百分比

⚠️ **注意**：没有直接的 `inputTokens` 字段，需要计算：`inputTokens = totalTokens - outputTokens`

**处理方式**: 提取 token 信息，用于最终响应的 `usage` 字段。

---

### 8. contextUsageEvent

上下文使用百分比（独立事件）。

```json
{
  "contextUsageEvent": {
    "contextUsagePercentage": 45.5
  }
}
```

⚠️ **注意**：`contextUsageEvent` 只包含百分比，**不包含** token 数量！

### Token 计算方式

**从 metadataEvent.tokenUsage 计算（推荐）**：

```javascript
function getUsageFromMetadata(tokenUsage) {
  const outputTokens = tokenUsage.outputTokens || 0
  const totalTokens = tokenUsage.totalTokens || 0
  // inputTokens 需要计算
  const inputTokens = totalTokens - outputTokens
  
  return {
    prompt_tokens: inputTokens,
    completion_tokens: outputTokens,
    total_tokens: totalTokens
  }
}
```

**转换为 OpenAI** (最终响应):
```json
{
  "usage": {
    "prompt_tokens": 8000,
    "completion_tokens": 500,
    "total_tokens": 8500
  }
}
```

**转换为 Anthropic** (最终响应):
```json
{
  "usage": {
    "input_tokens": 8000,
    "output_tokens": 500
  }
}
```

---

### 9. reasoningContentEvent

推理/思考内容事件（Extended Thinking）。

```json
{
  "reasoningContentEvent": {
    "text": "让我分析一下这个问题...",
    "signature": "xxx",
    "redactedContent": null
  }
}
```

**字段说明**：
- `text` - 推理文本内容
- `signature` - 签名（用于验证）
- `redactedContent` - 被编辑的内容（base64）

**转换为 OpenAI**（reasoning_content 字段）:
```json
{
  "choices": [{
    "delta": {
      "reasoning_content": "让我分析一下这个问题..."
    }
  }]
}
```

**转换为 Anthropic**（thinking block 完整流程）:

Anthropic 的 thinking block 需要完整的生命周期事件：

```javascript
// 1. content_block_start - 开始 thinking block
{
  "type": "content_block_start",
  "index": 0,  // thinking block 通常是第一个
  "content_block": {
    "type": "thinking",
    "thinking": "",
    "signature": "sig_xxx"  // 占位签名
  }
}

// 2. content_block_delta - 流式输出 thinking 内容（多次）
{
  "type": "content_block_delta",
  "index": 0,
  "delta": {
    "type": "thinking_delta",
    "thinking": "让我分析一下这个问题..."
  }
}

// 3. content_block_stop - 结束 thinking block
{
  "type": "content_block_stop",
  "index": 0
}

// 4. 然后才是 text block（index: 1）
{
  "type": "content_block_start",
  "index": 1,
  "content_block": {
    "type": "text",
    "text": ""
  }
}
```

**转换函数**：

```javascript
class ThinkingBlockHandler {
  constructor() {
    this.thinkingStarted = false
    this.thinkingIndex = null
    this.textIndex = null
    this.currentIndex = 0
  }
  
  handleReasoningContent(event) {
    const events = []
    const text = event.reasoningContentEvent.text
    
    // 首次收到 thinking 内容，发送 content_block_start
    if (!this.thinkingStarted) {
      this.thinkingIndex = this.currentIndex
      events.push({
        event: 'content_block_start',
        data: {
          type: 'content_block_start',
          index: this.thinkingIndex,
          content_block: {
            type: 'thinking',
            thinking: '',
            signature: `sig_${crypto.randomUUID().replace(/-/g, '').slice(0, 32)}`
          }
        }
      })
      this.thinkingStarted = true
    }
    
    // 发送 thinking delta
    if (text) {
      events.push({
        event: 'content_block_delta',
        data: {
          type: 'content_block_delta',
          index: this.thinkingIndex,
          delta: {
            type: 'thinking_delta',
            thinking: text
          }
        }
      })
    }
    
    return events
  }
  
  // 收到普通 content 时，先关闭 thinking block
  handleContent(event) {
    const events = []
    
    // 关闭 thinking block
    if (this.thinkingStarted && this.thinkingIndex !== null) {
      events.push({
        event: 'content_block_stop',
        data: {
          type: 'content_block_stop',
          index: this.thinkingIndex
        }
      })
      this.thinkingStarted = false
      this.currentIndex++
    }
    
    // 开始 text block
    if (this.textIndex === null) {
      this.textIndex = this.currentIndex
      events.push({
        event: 'content_block_start',
        data: {
          type: 'content_block_start',
          index: this.textIndex,
          content_block: {
            type: 'text',
            text: ''
          }
        }
      })
    }
    
    // 发送 text delta
    events.push({
      event: 'content_block_delta',
      data: {
        type: 'content_block_delta',
        index: this.textIndex,
        delta: {
          type: 'text_delta',
          text: event.assistantResponseEvent.content
        }
      }
    })
    
    return events
  }
}
```

⚠️ **注意**：这是 Kiro 原生支持的推理事件，不需要通过注入标签模拟。

---

### 10. dryRunSucceedEvent

干运行成功事件，当请求设置 `dryRun: true` 时返回。

```json
{
  "dryRunSucceedEvent": {}
}
```

**处理方式**: 表示请求验证通过，Token 有效，配额正常。用于健康检查或预检请求。

```javascript
if (kiroEvent.dryRunSucceedEvent !== undefined) {
  // 干运行成功，不需要输出内容
  return { success: true, message: '验证通过' }
}
```

---

### 11. invalidStateEvent

错误状态事件。

```json
{
  "invalidStateEvent": {
    "reason": "CONTEXT_LENGTH_EXCEEDED",
    "message": "上下文长度超出限制"
  }
}
```

**处理方式**: 转换为错误响应，中断流。

---

### 12. toolUseEvent

工具调用事件，AI 决定调用工具时返回。

```json
{
  "toolUseEvent": {
    "toolUseId": "tool-123",
    "name": "read_file",
    "input": { "path": "src/main.js" }
  }
}
```

**处理方式**: 转换为对应格式的工具调用，详见 [高级格式转换](./advanced-conversion.md)。

---

### 13. endOfStream / 流结束

流结束标记（可能是空事件或特定标记）。

**转换为 OpenAI**:
```json
{
  "choices": [{
    "index": 0,
    "delta": {},
    "finish_reason": "stop"
  }]
}
```
然后发送 `data: [DONE]`

**转换为 Anthropic**:
```
event: message_stop
data: {"type":"message_stop"}
```

---

## finish_reason 映射

不同结束原因需要正确映射：

### Kiro → OpenAI

```javascript
const FINISH_REASON_MAP_OPENAI = {
  // 正常结束
  'end_turn': 'stop',
  'stop_sequence': 'stop',
  
  // 工具调用
  'tool_use': 'tool_calls',
  
  // 长度限制
  'max_tokens': 'length',
  'context_length_exceeded': 'length',
  
  // 内容过滤
  'content_filtered': 'content_filter'
}
```

### Kiro → Anthropic

```javascript
const FINISH_REASON_MAP_ANTHROPIC = {
  // 正常结束
  'end_turn': 'end_turn',
  'stop_sequence': 'stop_sequence',
  
  // 工具调用
  'tool_use': 'tool_use',
  
  // 长度限制
  'max_tokens': 'max_tokens',
  'context_length_exceeded': 'max_tokens'
}
```

### 判断逻辑

```javascript
function getFinishReason(kiroEvent, hasToolUse) {
  // 如果有工具调用，finish_reason 是 tool_use/tool_calls
  if (hasToolUse) {
    return this.format === 'openai' ? 'tool_calls' : 'tool_use'
  }
  
  // 如果有 invalidStateEvent 且是长度问题
  if (kiroEvent.invalidStateEvent?.reason === 'CONTEXT_LENGTH_EXCEEDED') {
    return this.format === 'openai' ? 'length' : 'max_tokens'
  }
  
  // 默认正常结束
  return this.format === 'openai' ? 'stop' : 'end_turn'
}
```

---

## 完整转换实现

```javascript
class SSEConverter {
  constructor(format = 'openai') {
    this.format = format  // 'openai' | 'anthropic'
    this.messageId = null
    this.tokenUsage = null        // 从 metadataEvent 获取
    this.contentIndex = 0
    this.hasToolUse = false
  }

  // 处理单个 Kiro 事件
  convert(kiroEvent) {
    // messageMetadataEvent
    if (kiroEvent.messageMetadataEvent) {
      this.messageId = kiroEvent.messageMetadataEvent.messageId
      return null  // 不输出
    }

    // metadataEvent - 包含完整 token 信息
    if (kiroEvent.metadataEvent?.tokenUsage) {
      this.tokenUsage = kiroEvent.metadataEvent.tokenUsage
      return null
    }

    // assistantResponseEvent
    if (kiroEvent.assistantResponseEvent) {
      return this.convertText(kiroEvent.assistantResponseEvent.content)
    }

    // codeEvent
    if (kiroEvent.codeEvent) {
      const { language, content } = kiroEvent.codeEvent
      const markdown = `\`\`\`${language || ''}\n${content}\n\`\`\``
      return this.convertText(markdown)
    }

    // toolUseEvent
    if (kiroEvent.toolUseEvent) {
      this.hasToolUse = true
      return this.convertToolUse(kiroEvent.toolUseEvent)
    }

    // contextUsageEvent - 只有百分比，忽略（用 metadataEvent）
    if (kiroEvent.contextUsageEvent) {
      return null
    }

    // invalidStateEvent
    if (kiroEvent.invalidStateEvent) {
      return this.convertError(kiroEvent.invalidStateEvent)
    }

    // 其他事件忽略
    return null
  }

  // 转换文本内容
  convertText(text) {
    if (this.format === 'openai') {
      return {
        id: `chatcmpl-${this.messageId || Date.now()}`,
        object: 'chat.completion.chunk',
        created: Math.floor(Date.now() / 1000),
        model: 'kiro',
        choices: [{
          index: 0,
          delta: { content: text },
          finish_reason: null
        }]
      }
    }

    if (this.format === 'anthropic') {
      return {
        event: 'content_block_delta',
        data: {
          type: 'content_block_delta',
          index: this.contentIndex,
          delta: { type: 'text_delta', text }
        }
      }
    }
  }

  // 转换工具调用
  convertToolUse(toolUse) {
    const { toolUseId, name, input } = toolUse
    
    if (this.format === 'openai') {
      return {
        id: `chatcmpl-${this.messageId || Date.now()}`,
        object: 'chat.completion.chunk',
        created: Math.floor(Date.now() / 1000),
        model: 'kiro',
        choices: [{
          index: 0,
          delta: {
            tool_calls: [{
              index: 0,
              id: toolUseId,
              type: 'function',
              function: {
                name: name,
                arguments: JSON.stringify(input)
              }
            }]
          },
          finish_reason: null
        }]
      }
    }

    if (this.format === 'anthropic') {
      // Anthropic 需要多个事件
      return [
        {
          event: 'content_block_start',
          data: {
            type: 'content_block_start',
            index: ++this.contentIndex,
            content_block: {
              type: 'tool_use',
              id: toolUseId,
              name: name,
              input: {}
            }
          }
        },
        {
          event: 'content_block_delta',
          data: {
            type: 'content_block_delta',
            index: this.contentIndex,
            delta: {
              type: 'input_json_delta',
              partial_json: JSON.stringify(input)
            }
          }
        },
        {
          event: 'content_block_stop',
          data: {
            type: 'content_block_stop',
            index: this.contentIndex
          }
        }
      ]
    }
  }

  // 获取 finish_reason
  getFinishReason(kiroEvent) {
    // 工具调用
    if (this.hasToolUse) {
      return this.format === 'openai' ? 'tool_calls' : 'tool_use'
    }
    
    // 长度限制
    if (kiroEvent?.invalidStateEvent?.reason === 'CONTEXT_LENGTH_EXCEEDED') {
      return this.format === 'openai' ? 'length' : 'max_tokens'
    }
    
    // 正常结束
    return this.format === 'openai' ? 'stop' : 'end_turn'
  }

  // 获取 usage 信息
  getUsage() {
    if (!this.tokenUsage) return null
    
    const outputTokens = this.tokenUsage.outputTokens || 0
    const totalTokens = this.tokenUsage.totalTokens || 0
    // inputTokens 需要计算：totalTokens - outputTokens
    const inputTokens = totalTokens - outputTokens
    
    return {
      prompt_tokens: inputTokens,
      completion_tokens: outputTokens,
      total_tokens: totalTokens
    }
  }

  // 转换结束事件
  convertEnd(lastEvent) {
    const finishReason = this.getFinishReason(lastEvent)
    const usage = this.getUsage()
    
    if (this.format === 'openai') {
      return {
        id: `chatcmpl-${this.messageId || Date.now()}`,
        object: 'chat.completion.chunk',
        created: Math.floor(Date.now() / 1000),
        model: 'kiro',
        choices: [{
          index: 0,
          delta: {},
          finish_reason: finishReason
        }],
        usage: usage || undefined
      }
    }

    if (this.format === 'anthropic') {
      return [
        {
          event: 'message_delta',
          data: {
            type: 'message_delta',
            delta: { stop_reason: finishReason },
            usage: usage ? {
              output_tokens: usage.completion_tokens
            } : undefined
          }
        },
        {
          event: 'message_stop',
          data: { type: 'message_stop' }
        }
      ]
    }
  }

  // 转换错误
  convertError(error) {
    if (this.format === 'openai') {
      return {
        error: {
          message: error.message || error.reason,
          type: error.reason || 'server_error',
          code: error.reason
        }
      }
    }

    if (this.format === 'anthropic') {
      return {
        event: 'error',
        data: {
          type: 'error',
          error: {
            type: error.reason || 'api_error',
            message: error.message || error.reason
          }
        }
      }
    }
  }
}
```

---

## SSE 输出格式

### OpenAI 格式

```
data: {"id":"chatcmpl-xxx","object":"chat.completion.chunk","created":1704960000,"model":"kiro","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}

data: {"id":"chatcmpl-xxx","object":"chat.completion.chunk","created":1704960000,"model":"kiro","choices":[{"index":0,"delta":{"content":" World"},"finish_reason":null}]}

data: {"id":"chatcmpl-xxx","object":"chat.completion.chunk","created":1704960000,"model":"kiro","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}

data: [DONE]
```

### Anthropic 格式

```
event: message_start
data: {"type":"message_start","message":{"id":"msg-xxx","type":"message","role":"assistant","content":[],"model":"claude-3-5-sonnet","stop_reason":null,"usage":{"input_tokens":0,"output_tokens":0}}}

event: content_block_start
data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":" World"}}

event: content_block_stop
data: {"type":"content_block_stop","index":0}

event: message_delta
data: {"type":"message_delta","delta":{"stop_reason":"end_turn"},"usage":{"output_tokens":10}}

event: message_stop
data: {"type":"message_stop"}
```

---

## 注意事项

1. **codeEvent 处理** - 需要转成 Markdown 代码块
2. **usage 统计** - 从 `metadataEvent.tokenUsage` 获取，`inputTokens = totalTokens - outputTokens`
3. **contextUsageEvent 只有百分比** - 不包含 token 数量，用 `metadataEvent` 获取精确数据
4. **错误处理** - invalidStateEvent 需要中断流并返回错误
5. **Anthropic 格式更复杂** - 需要 message_start、content_block_start 等额外事件
