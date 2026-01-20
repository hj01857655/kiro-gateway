---
inclusion: fileMatch
fileMatchPattern: "**/converter*.{rs,ts,tsx}"
---

# API 格式转换规范

## 概述

KiroGate 需要在 OpenAI/Anthropic 格式和 Kiro 格式之间进行双向转换。

---

## 基础请求转换

### OpenAI → Kiro

- `messages[role=system]` → `additionalContext.systemPrompt`
- `messages[role=user]` 最后一条 → `currentMessage.userInputMessage.content`
- 之前的消息 → `history`
- `tools` → `userInputMessageContext.tools`（toolSpec 格式）
- `model` → `userInputMessage.modelId`（无需 qdev:: 前缀）

### Anthropic → Kiro

- `system` → `additionalContext.systemPrompt`
- `messages` 处理同 OpenAI
- `tools` → input_schema 转 inputSchema.json

---

## 基础响应转换

### Kiro → OpenAI

```
assistantResponseEvent → choices[0].delta.content
toolUseEvent → choices[0].delta.tool_calls
结束 → finish_reason: stop/tool_calls/length
最后 → data: [DONE]
```

### Kiro → Anthropic

```
assistantResponseEvent → content_block_delta (text_delta)
toolUseEvent → content_block_start + input_json_delta + content_block_stop
reasoningContentEvent → thinking block 完整生命周期
结束 → message_delta + message_stop
```

---

## 工具调用转换

### 工具定义转换

```javascript
function convertTools(tools, format) {
  if (!tools || tools.length === 0) return []
  
  return tools.map(tool => {
    // OpenAI 格式
    if (format === 'openai' && tool.type === 'function') {
      return {
        toolSpec: {
          name: tool.function.name,
          description: tool.function.description || '',
          inputSchema: {
            json: tool.function.parameters || { type: 'object', properties: {} }
          }
        }
      }
    }
    
    // Anthropic 格式
    if (format === 'anthropic') {
      return {
        toolSpec: {
          name: tool.name,
          description: tool.description || '',
          inputSchema: {
            json: tool.input_schema || { type: 'object', properties: {} }
          }
        }
      }
    }
    
    return null
  }).filter(Boolean)
}
```

### 工具调用响应转换

**Kiro → OpenAI**:
```javascript
{
  "choices": [{
    "delta": {
      "tool_calls": [{
        "id": toolUseId,
        "type": "function",
        "function": {
          "name": name,
          "arguments": JSON.stringify(input)
        }
      }]
    }
  }]
}
```

**Kiro → Anthropic**:
```javascript
[
  { event: 'content_block_start', data: { type: 'tool_use', id, name, input: {} } },
  { event: 'content_block_delta', data: { type: 'input_json_delta', partial_json } },
  { event: 'content_block_stop', data: { type: 'content_block_stop' } }
]
```

---

## 图片/多模态转换

⚠️ **重要**：图片放在 `userInputMessage.images` 数组，不是 `content` 里！

```javascript
// Kiro 格式
{
  userInputMessage: {
    content: ["这张图片是什么？"],
    images: [{
      image: {
        format: "png",
        source: { bytes: "base64..." }
      }
    }]
  }
}
```

---

## 相邻消息合并

Kiro API 不接受连续相同 role 的消息，必须合并：

```javascript
function mergeAdjacentMessages(messages) {
  const merged = []
  let pendingToolResults = []
  
  for (const msg of messages) {
    // tool 消息收集为 tool_results
    if (msg.role === 'tool') {
      pendingToolResults.push({
        type: 'tool_result',
        tool_use_id: msg.tool_call_id,
        content: msg.content || '(empty result)'
      })
      continue
    }
    
    // 处理待处理的 tool_results
    if (pendingToolResults.length > 0) {
      merged.push({ role: 'user', content: pendingToolResults })
      pendingToolResults = []
    }
    
    // 合并相同 role 的消息
    const last = merged[merged.length - 1]
    if (last && msg.role === last.role) {
      const lastText = extractText(last.content)
      const currText = extractText(msg.content)
      last.content = `${lastText}\n${currText}`
      
      // 合并 tool_calls（重要！）
      if (msg.role === 'assistant' && msg.tool_calls) {
        last.tool_calls = [...(last.tool_calls || []), ...msg.tool_calls]
      }
    } else {
      merged.push({ ...msg })
    }
  }
  
  return merged
}
```

---

## Kiro SSE 事件类型

### 主要事件

| 事件类型 | 用途 | 转换目标 |
|---------|------|---------|
| `assistantResponseEvent` | 文本内容 | content/text_delta |
| `codeEvent` | 代码块 | Markdown 代码块 |
| `toolUseEvent` | 工具调用 | tool_calls/tool_use |
| `reasoningContentEvent` | 推理内容 | reasoning_content/thinking |
| `metadataEvent` | Token 统计 | usage 字段 |
| `invalidStateEvent` | 错误 | 错误响应 |

### Token 统计

从 `metadataEvent.tokenUsage` 获取：

```javascript
function getUsage(tokenUsage) {
  const outputTokens = tokenUsage.outputTokens || 0
  const totalTokens = tokenUsage.totalTokens || 0
  const inputTokens = totalTokens - outputTokens  // 需要计算
  
  return {
    prompt_tokens: inputTokens,
    completion_tokens: outputTokens,
    total_tokens: totalTokens
  }
}
```

### finish_reason 映射

```javascript
// OpenAI
const FINISH_REASON_MAP = {
  'end_turn': 'stop',
  'tool_use': 'tool_calls',
  'max_tokens': 'length',
  'context_length_exceeded': 'length'
}

// Anthropic
const FINISH_REASON_MAP = {
  'end_turn': 'end_turn',
  'tool_use': 'tool_use',
  'max_tokens': 'max_tokens',
  'context_length_exceeded': 'max_tokens'
}
```

---

## 注意事项

1. **相邻同 role 消息必须合并** - 否则 Kiro 会报错
2. **图片放 images 数组** - 不是 content 里
3. **长 tool description** - 移到 system prompt
4. **Kiro 不支持的参数** - temperature/top_p/max_tokens 直接忽略
5. **inputTokens 需要计算** - totalTokens - outputTokens
6. **tool_calls 要合并** - 合并 assistant 消息时保留所有 tool_calls
7. **codeEvent 转 Markdown** - 包装成代码块格式
