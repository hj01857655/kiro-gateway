# 高级格式转换

## 概述

除了基础的文本对话，KiroGate 还需要处理：

1. **工具调用** - Claude Code 会传工具定义和调用
2. **System Prompt** - 不同格式的系统提示词
3. **图片/多模态** - 图片输入格式转换
4. **参数映射** - max_tokens 等参数

---

## 1. 工具调用转换

Claude Code 会传递工具定义，让 AI 决定调用哪个工具。

### Anthropic 工具格式 → Kiro

```javascript
// Anthropic 请求
{
  "model": "claude-3-5-sonnet",
  "max_tokens": 4096,
  "tools": [
    {
      "name": "read_file",
      "description": "读取文件内容",
      "input_schema": {
        "type": "object",
        "properties": {
          "path": { "type": "string", "description": "文件路径" }
        },
        "required": ["path"]
      }
    },
    {
      "name": "write_file",
      "description": "写入文件",
      "input_schema": {
        "type": "object",
        "properties": {
          "path": { "type": "string" },
          "content": { "type": "string" }
        },
        "required": ["path", "content"]
      }
    }
  ],
  "messages": [...]
}

// 转换为 Kiro 格式
{
  "conversationState": {
    "currentMessage": {
      "userInputMessage": {
        "content": [...],
        "userIntent": "CODE_GENERATION",
        "userInputMessageContext": {
          "tools": [
            {
              "toolSpec": {
                "name": "read_file",
                "description": "读取文件内容",
                "inputSchema": {
                  "json": {
                    "type": "object",
                    "properties": {
                      "path": { "type": "string", "description": "文件路径" }
                    },
                    "required": ["path"]
                  }
                }
              }
            },
            {
              "toolSpec": {
                "name": "write_file",
                "description": "写入文件",
                "inputSchema": {
                  "json": {
                    "type": "object",
                    "properties": {
                      "path": { "type": "string" },
                      "content": { "type": "string" }
                    },
                    "required": ["path", "content"]
                  }
                }
              }
            }
          ]
        }
      }
    }
  }
}
```

### OpenAI 工具格式 → Kiro

```javascript
// OpenAI 请求
{
  "model": "gpt-4",
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "read_file",
        "description": "读取文件内容",
        "parameters": {
          "type": "object",
          "properties": {
            "path": { "type": "string" }
          },
          "required": ["path"]
        }
      }
    }
  ],
  "messages": [...]
}

// 转换为 Kiro 格式（同上）
```

### 转换函数

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

---

## 2. 工具调用响应转换

当 AI 决定调用工具时，Kiro 返回 `toolUseEvent`。

### Kiro toolUseEvent → Anthropic

```javascript
// Kiro 响应
{
  "toolUseEvent": {
    "toolUseId": "tool-123",
    "name": "read_file",
    "input": {
      "path": "src/main.js"
    }
  }
}

// 转换为 Anthropic 格式
{
  "type": "content_block_start",
  "index": 1,
  "content_block": {
    "type": "tool_use",
    "id": "tool-123",
    "name": "read_file",
    "input": {}
  }
}
// 然后
{
  "type": "content_block_delta",
  "index": 1,
  "delta": {
    "type": "input_json_delta",
    "partial_json": "{\"path\":\"src/main.js\"}"
  }
}
```

### Kiro toolUseEvent → OpenAI

```javascript
// Kiro 响应
{
  "toolUseEvent": {
    "toolUseId": "tool-123",
    "name": "read_file",
    "input": {
      "path": "src/main.js"
    }
  }
}

// 转换为 OpenAI 格式
{
  "choices": [{
    "index": 0,
    "delta": {
      "tool_calls": [{
        "index": 0,
        "id": "tool-123",
        "type": "function",
        "function": {
          "name": "read_file",
          "arguments": "{\"path\":\"src/main.js\"}"
        }
      }]
    },
    "finish_reason": null
  }]
}
```

### 转换函数

```javascript
function convertToolUseEvent(event, format) {
  const { toolUseId, name, input } = event.toolUseEvent
  
  if (format === 'anthropic') {
    return [
      {
        event: 'content_block_start',
        data: {
          type: 'content_block_start',
          index: 1,
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
          index: 1,
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
          index: 1
        }
      }
    ]
  }
  
  if (format === 'openai') {
    return {
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
}
```

---

## 3. 工具结果转换

客户端执行工具后，需要把结果发回来。

### Anthropic tool_result → Kiro

```javascript
// Anthropic 请求（带工具结果）
{
  "messages": [
    { "role": "user", "content": "读取 main.js" },
    {
      "role": "assistant",
      "content": [
        { "type": "text", "text": "我来读取文件" },
        {
          "type": "tool_use",
          "id": "tool-123",
          "name": "read_file",
          "input": { "path": "src/main.js" }
        }
      ]
    },
    {
      "role": "user",
      "content": [
        {
          "type": "tool_result",
          "tool_use_id": "tool-123",
          "content": "console.log('hello')"
        }
      ]
    }
  ]
}

// 转换为 Kiro history
{
  "history": [
    {
      "userInputMessage": {
        "content": ["读取 main.js"],
        "userIntent": "CODE_GENERATION"
      }
    },
    {
      "assistantResponseMessage": {
        "content": "我来读取文件",
        "toolUse": [{
          "toolUseId": "tool-123",
          "name": "read_file",
          "input": { "path": "src/main.js" }
        }]
      }
    },
    {
      "toolResult": {
        "toolUseId": "tool-123",
        "status": "success",
        "content": [{ "text": "console.log('hello')" }]
      }
    }
  ]
}
```

### OpenAI tool_calls → Kiro

```javascript
// OpenAI 请求（带工具结果）
{
  "messages": [
    { "role": "user", "content": "读取 main.js" },
    {
      "role": "assistant",
      "content": null,
      "tool_calls": [{
        "id": "tool-123",
        "type": "function",
        "function": {
          "name": "read_file",
          "arguments": "{\"path\":\"src/main.js\"}"
        }
      }]
    },
    {
      "role": "tool",
      "tool_call_id": "tool-123",
      "content": "console.log('hello')"
    }
  ]
}

// 转换为 Kiro history（同上）
```

---

## 4. System Prompt 转换

### OpenAI → Kiro

```javascript
// OpenAI 请求
{
  "messages": [
    { "role": "system", "content": "你是一个代码助手" },
    { "role": "user", "content": "写个函数" }
  ]
}

// 转换
function extractSystemPrompt(messages, format) {
  if (format === 'openai') {
    const systemMsg = messages.find(m => m.role === 'system')
    return systemMsg?.content || ''
  }
  return ''
}

// Kiro 格式
{
  "conversationState": {
    "currentMessage": {
      "userInputMessage": {
        "content": ["写个函数"],
        "userInputMessageContext": {
          "additionalContext": {
            "systemPrompt": "你是一个代码助手"
          }
        }
      }
    }
  }
}
```

### Anthropic → Kiro

```javascript
// Anthropic 请求
{
  "system": "你是一个代码助手",
  "messages": [
    { "role": "user", "content": "写个函数" }
  ]
}

// 转换
function extractSystemPrompt(request, format) {
  if (format === 'anthropic') {
    return request.system || ''
  }
  return ''
}

// Kiro 格式（同上）
```

---

## 5. 图片/多模态转换

> ⚠️ **重要**：图片放在 `userInputMessage.images` 数组，不是 `content` 里！

### Kiro 源码

```javascript
// Kiro 源码中的图片处理
{
  userInputMessage: {
    content: [...],
    images: this.chatMessageToImageBlocks(messages2.at(-1))
  }
}

// 图片格式
{
  image: {
    format: "jpeg",  // 或 "png"
    source: {
      bytes: Buffer.from(part.imageUrl.url.split(",")[1], "base64")
    }
  }
}
```

### Anthropic 图片格式 → Kiro

```javascript
// Anthropic 请求
{
  "messages": [{
    "role": "user",
    "content": [
      { "type": "text", "text": "这张图片是什么？" },
      {
        "type": "image",
        "source": {
          "type": "base64",
          "media_type": "image/png",
          "data": "iVBORw0KGgo..."
        }
      }
    ]
  }]
}

// 转换为 Kiro 格式
{
  "conversationState": {
    "currentMessage": {
      "userInputMessage": {
        "content": ["这张图片是什么？"],
        "images": [
          {
            "image": {
              "format": "png",
              "source": {
                "bytes": "iVBORw0KGgo..."
              }
            }
          }
        ]
      }
    }
  }
}
```

### OpenAI 图片格式 → Kiro

```javascript
// OpenAI 请求
{
  "messages": [{
    "role": "user",
    "content": [
      { "type": "text", "text": "这张图片是什么？" },
      {
        "type": "image_url",
        "image_url": {
          "url": "data:image/png;base64,iVBORw0KGgo..."
        }
      }
    ]
  }]
}

// 转换为 Kiro 格式（同上，图片放 images 数组）
```

### 转换函数

```javascript
function convertImages(messages, format) {
  const lastUserMsg = messages.filter(m => m.role === 'user').at(-1)
  if (!lastUserMsg) return []
  
  const content = lastUserMsg.content
  if (typeof content === 'string') return []
  
  const images = []
  
  for (const part of content) {
    // OpenAI 格式
    if (part.type === 'image_url') {
      const url = part.image_url.url
      const match = url.match(/^data:image\/(\w+);base64,(.+)$/)
      if (match) {
        images.push({
          image: {
            format: match[1],
            source: { bytes: match[2] }
          }
        })
      }
    }
    
    // Anthropic 格式
    if (part.type === 'image') {
      images.push({
        image: {
          format: part.source.media_type.split('/')[1],
          source: { bytes: part.source.data }
        }
      })
    }
  }
  
  return images
}

// 使用
const kiroPayload = {
  conversationState: {
    currentMessage: {
      userInputMessage: {
        content: extractTextContent(messages),
        images: convertImages(messages, format),  // 图片单独放这里
        userIntent: 'CODE_GENERATION'
      }
    }
  }
}
```

---

## 6. 参数映射

### max_tokens

```javascript
// OpenAI/Anthropic 都有 max_tokens
// Kiro 对应字段（如果有的话）

function mapParameters(request, format) {
  const params = {}
  
  // max_tokens - Kiro 可能不支持，忽略或记录
  if (request.max_tokens) {
    // Kiro 似乎没有对应字段，忽略
    console.log(`max_tokens=${request.max_tokens} 被忽略`)
  }
  
  // temperature - Kiro 不支持
  if (request.temperature) {
    console.log(`temperature=${request.temperature} 被忽略`)
  }
  
  // stream - 始终使用流式
  params.stream = true
  
  return params
}
```

### 不支持的参数

以下参数 Kiro 不支持，会被忽略：
- `temperature`
- `top_p`
- `top_k`
- `presence_penalty`
- `frequency_penalty`
- `stop`
- `seed`

---

## 7. 完整消息转换示例

```javascript
class MessageConverter {
  constructor(format) {
    this.format = format  // 'openai' | 'anthropic'
  }
  
  // 转换完整请求
  convert(request) {
    const systemPrompt = this.extractSystemPrompt(request)
    const messages = this.extractMessages(request)
    const tools = this.convertTools(request.tools)
    
    // 分离 history 和 currentMessage
    const { history, currentMessage } = this.buildHistory(messages)
    
    return {
      conversationState: {
        conversationId: crypto.randomUUID(),
        chatTriggerType: 'MANUAL',
        currentMessage: {
          userInputMessage: {
            content: currentMessage,
            userIntent: 'CODE_GENERATION',
            userInputMessageContext: {
              tools,
              additionalContext: systemPrompt ? { systemPrompt } : {}
            }
          }
        },
        history
      },
      profileArn: ''
    }
  }
  
  extractSystemPrompt(request) {
    if (this.format === 'anthropic') {
      return request.system || ''
    }
    if (this.format === 'openai') {
      const msg = request.messages?.find(m => m.role === 'system')
      return msg?.content || ''
    }
    return ''
  }
  
  extractMessages(request) {
    let messages = request.messages || []
    
    // OpenAI: 过滤掉 system
    if (this.format === 'openai') {
      messages = messages.filter(m => m.role !== 'system')
    }
    
    return messages
  }
  
  convertTools(tools) {
    if (!tools) return []
    return convertTools(tools, this.format)
  }
  
  buildHistory(messages) {
    // 找到最后一条 user 消息
    const lastUserIndex = messages.findLastIndex(m => m.role === 'user')
    
    const historyMessages = messages.slice(0, lastUserIndex)
    const currentMessage = this.convertContent(messages[lastUserIndex]?.content)
    
    const history = []
    
    for (const msg of historyMessages) {
      if (msg.role === 'user') {
        history.push({
          userInputMessage: {
            content: this.convertContent(msg.content),
            userIntent: 'CODE_GENERATION'
          }
        })
      } else if (msg.role === 'assistant') {
        const { text, toolUse } = this.parseAssistantMessage(msg)
        history.push({
          assistantResponseMessage: {
            content: text,
            toolUse
          }
        })
      } else if (msg.role === 'tool') {
        history.push({
          toolResult: {
            toolUseId: msg.tool_call_id,
            status: 'success',
            content: [{ text: msg.content }]
          }
        })
      }
    }
    
    return { history, currentMessage }
  }
  
  convertContent(content) {
    if (typeof content === 'string') {
      return [content]
    }
    
    if (Array.isArray(content)) {
      return content.map(item => {
        if (item.type === 'text') return item.text
        if (item.type === 'image' || item.type === 'image_url') {
          return convertImageContent(item, this.format)
        }
        if (item.type === 'tool_result') {
          return item.content
        }
        return item
      }).filter(Boolean)
    }
    
    return [String(content)]
  }
  
  parseAssistantMessage(msg) {
    let text = ''
    let toolUse = []
    
    if (typeof msg.content === 'string') {
      text = msg.content
    } else if (Array.isArray(msg.content)) {
      for (const item of msg.content) {
        if (item.type === 'text') text += item.text
        if (item.type === 'tool_use') {
          toolUse.push({
            toolUseId: item.id,
            name: item.name,
            input: item.input
          })
        }
      }
    }
    
    // OpenAI tool_calls
    if (msg.tool_calls) {
      for (const call of msg.tool_calls) {
        toolUse.push({
          toolUseId: call.id,
          name: call.function.name,
          input: JSON.parse(call.function.arguments || '{}')
        })
      }
    }
    
    return { text, toolUse }
  }
}
```

---

## 8. 相邻消息合并

Kiro API 不接受连续相同 role 的消息，必须合并。

### 问题场景

```javascript
// 这种情况会报错
{
  "messages": [
    { "role": "user", "content": "你好" },
    { "role": "user", "content": "帮我写代码" },  // 连续两个 user
    { "role": "assistant", "content": "好的" }
  ]
}
```

### Kiro 源码实现

Kiro 使用 LangChain 的 `mergeMessageRuns` 函数：

```javascript
// LangChain 实现
function _mergeMessageRuns(messages) {
  if (!messages.length) return []
  
  const merged = []
  for (const msg of messages) {
    const curr = msg
    const last = merged.pop()
    
    if (!last) {
      merged.push(curr)
    } else if (curr.getType() === "tool" || curr.getType() !== last.getType()) {
      // tool 消息不合并，不同类型不合并
      merged.push(last, curr)
    } else {
      // 相同类型：合并内容，用换行符连接
      const mergedContent = `${last.content}\n${curr.content}`
      merged.push({ ...last, content: mergedContent })
    }
  }
  return merged
}
```

### KiroGate 实现

```javascript
function mergeAdjacentMessages(messages) {
  if (!messages.length) return []
  
  const merged = []
  let pendingToolResults = []
  
  for (const msg of messages) {
    // 1. tool 消息特殊处理：收集为 tool_results
    if (msg.role === 'tool') {
      pendingToolResults.push({
        type: 'tool_result',
        tool_use_id: msg.tool_call_id,
        content: msg.content || '(empty result)'
      })
      continue
    }
    
    // 2. 如果有待处理的 tool_results，创建 user 消息
    if (pendingToolResults.length > 0) {
      merged.push({
        role: 'user',
        content: pendingToolResults
      })
      pendingToolResults = []
    }
    
    // 3. 合并相同 role 的消息
    const last = merged[merged.length - 1]
    if (last && msg.role === last.role) {
      // 合并文本内容
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
  
  // 处理末尾的 tool_results
  if (pendingToolResults.length > 0) {
    merged.push({
      role: 'user',
      content: pendingToolResults
    })
  }
  
  return merged
}

function extractText(content) {
  if (typeof content === 'string') return content
  if (Array.isArray(content)) {
    return content
      .filter(c => typeof c === 'string' || c.type === 'text')
      .map(c => typeof c === 'string' ? c : c.text)
      .join('\n')
  }
  return String(content)
}
```

### 关键区别

**Kiro 源码**：
- tool 消息保持独立，不合并
- 只合并文本内容

**KiroGate 实现**：
- tool 消息转成 user 消息的 `tool_results` 数组
- 合并 assistant 的 `tool_calls`（重要！否则会丢失工具调用）

### 为什么要合并 tool_calls

```javascript
// 如果不合并 tool_calls，这种情况会出错：
{
  "messages": [
    { "role": "assistant", "content": "我来读取文件", "tool_calls": [{ id: "1", ... }] },
    { "role": "assistant", "content": "再读取另一个", "tool_calls": [{ id: "2", ... }] },
    { "role": "user", "content": [
      { "type": "tool_result", "tool_use_id": "1", ... },
      { "type": "tool_result", "tool_use_id": "2", ... }
    ]}
  ]
}

// 合并后：
{
  "messages": [
    { 
      "role": "assistant", 
      "content": "我来读取文件\n再读取另一个", 
      "tool_calls": [{ id: "1", ... }, { id: "2", ... }]  // 两个都保留
    },
    { "role": "user", "content": [...] }
  ]
}
```

---

## 9. Tool Description 长度处理

Kiro API 对 tool description 有长度限制，超长的需要特殊处理。

### 问题

Claude Code 的工具（如 `bash`、`computer`）description 很长，可能超过 Kiro 限制。

### 解决方案

把超长的 description 移到 system prompt，tool 里只留引用。

```javascript
const TOOL_DESCRIPTION_MAX_LENGTH = 4000  // 根据实际限制调整

function processToolsWithLongDescriptions(tools, systemPrompt) {
  if (!tools || tools.length === 0) {
    return { tools: null, systemPrompt }
  }
  
  const processedTools = []
  const toolDocs = []
  
  for (const tool of tools) {
    const description = tool.function?.description || tool.description || ''
    
    if (description.length <= TOOL_DESCRIPTION_MAX_LENGTH) {
      // 短 description，保持不变
      processedTools.push(tool)
    } else {
      // 长 description，移到 system prompt
      const toolName = tool.function?.name || tool.name
      
      // 添加到文档
      toolDocs.push(`## Tool: ${toolName}\n\n${description}`)
      
      // 创建简化版 tool
      processedTools.push({
        ...tool,
        function: {
          ...tool.function,
          description: `[Full documentation in system prompt under '## Tool: ${toolName}']`
        }
      })
    }
  }
  
  // 拼接 tool 文档到 system prompt
  if (toolDocs.length > 0) {
    const toolDocSection = `
---
# Tool Documentation
The following tools have detailed documentation that couldn't fit in the tool definition.

${toolDocs.join('\n\n---\n\n')}
`
    systemPrompt = systemPrompt 
      ? `${systemPrompt}${toolDocSection}`
      : toolDocSection.trim()
  }
  
  return {
    tools: processedTools.length > 0 ? processedTools : null,
    systemPrompt
  }
}
```

### 使用示例

```javascript
// 转换请求时
function convertRequest(request, format) {
  let systemPrompt = extractSystemPrompt(request, format)
  let tools = request.tools
  
  // 处理长 description
  const processed = processToolsWithLongDescriptions(tools, systemPrompt)
  tools = processed.tools
  systemPrompt = processed.systemPrompt
  
  // 继续转换...
  return buildKiroPayload(request, tools, systemPrompt)
}
```

### 效果

```javascript
// 原始 tool（description 很长）
{
  "name": "bash",
  "description": "Execute a bash command in the terminal...(5000+ 字符)"
}

// 处理后的 tool
{
  "name": "bash",
  "description": "[Full documentation in system prompt under '## Tool: bash']"
}

// system prompt 末尾
// ---
// # Tool Documentation
// ## Tool: bash
// Execute a bash command in the terminal...(完整内容)
```

---

## 注意事项

1. **工具调用是关键** - Claude Code 依赖工具调用来读写文件、执行命令
2. **tool_use_id 必须匹配** - 工具结果的 ID 必须和调用的 ID 一致
3. **图片格式** - 注意 media_type 和 format 的转换
4. **参数忽略** - Kiro 不支持的参数直接忽略，不要报错
5. **消息合并** - 必须合并相邻同 role 消息，否则 Kiro 会报错
6. **tool_calls 合并** - 合并 assistant 消息时要保留所有 tool_calls
7. **长 description** - 超长的 tool description 移到 system prompt

---

## 10. Kiro 特有上下文字段

Kiro API 支持一些特有的上下文字段，KiroGate 可以选择性使用。

### userInputMessageContext 完整结构

```javascript
{
  "userInputMessageContext": {
    // 工具定义
    "tools": [...],
    
    // 编辑器状态（可选）
    "editorState": {
      "document": {
        "documentId": "file:///path/to/file.js",
        "relativeFilePath": "src/file.js",
        "programmingLanguage": { "languageName": "javascript" },
        "text": "文件内容..."
      },
      "cursorState": {
        "range": {
          "start": { "line": 10, "character": 0 },
          "end": { "line": 10, "character": 0 }
        }
      }
    },
    
    // Shell 状态（可选）
    "shellState": {
      "shellName": "powershell",
      "shellHistory": [
        { "command": "npm install", "output": "..." }
      ]
    },
    
    // 环境状态（可选）
    "envState": {
      "operatingSystem": "Windows",
      "currentWorkingDirectory": "E:/project"
    },
    
    // 工作区状态（可选）
    "workspaceState": {
      "workspaceFolders": [
        { "uri": "file:///E:/project", "name": "project" }
      ]
    },
    
    // 相关文档（可选）
    "relevantDocuments": [
      {
        "documentId": "file:///path/to/related.js",
        "relativeFilePath": "src/related.js",
        "text": "相关文件内容..."
      }
    ],
    
    // 额外上下文
    "additionalContext": {
      "systemPrompt": "你是一个代码助手"
    }
  }
}
```

### KiroGate 建议

对于 KiroGate 代理，大部分上下文字段可以忽略：

```javascript
// 最简配置
{
  "userInputMessageContext": {
    "tools": convertedTools,
    "additionalContext": {
      "systemPrompt": systemPrompt
    }
  }
}

// 如果需要提供文件上下文
{
  "userInputMessageContext": {
    "tools": convertedTools,
    "relevantDocuments": [
      {
        "documentId": "context-1",
        "relativeFilePath": "context.txt",
        "text": "用户提供的上下文内容"
      }
    ],
    "additionalContext": {
      "systemPrompt": systemPrompt
    }
  }
}
```

### 何时使用这些字段

- `editorState` - 如果客户端提供了当前编辑文件信息
- `shellState` - 如果需要提供终端历史
- `envState` - 如果需要告诉 AI 运行环境
- `relevantDocuments` - 如果需要提供额外的文件上下文
- `workspaceState` - 通常不需要
