# Kiro 消息清理规范

## 概述

Kiro IDE 和 Kiro Account Manager 都实现了完整的消息清理逻辑（`sanitizeConversation`），确保发送给 Kiro API 的消息符合格式要求。

**参考来源**：
- Kiro IDE: `extension.js` 的 `sanitizeConversation` 函数
- Kiro Account Manager: [commit 624a3c4](https://github.com/chaogei/Kiro-account-manager/commit/624a3c4e5263dd6f153ab5e441cda05a9a038e48)

---

## Kiro API 请求格式

Kiro API 使用**原生 history 字段**来传递历史消息，而不是将历史嵌入到 currentMessage 中。

**请求结构**（来自 Kiro IDE 源码 extension.js:679288）：

```typescript
{
  history: serializedMessages.slice(0, -1),  // 除最后一条外的所有消息
  currentMessage: serializedMessages.at(-1),  // 最后一条消息
  chatTriggerType: "MANUAL",
  // ... 其他字段
}
```

**关键点**：
- `history` 字段包含除最后一条外的所有消息
- `currentMessage` 字段包含最后一条消息（必须是 user 消息）
- 消息清理（sanitizeConversation）在构建 history 之前执行
- 清理后的消息列表会被拆分为 history 和 currentMessage

---

## 标准占位消息

Kiro 使用标准的占位消息来确保消息格式正确：

```typescript
// 用户消息占位符
const HELLO_MESSAGE = {
  userInputMessage: {
    content: "Hello",
    origin: "AI_EDITOR"
  }
}

const CONTINUE_MESSAGE = {
  userInputMessage: {
    content: "Continue",
    origin: "AI_EDITOR"
  }
}

// 助手消息占位符
const UNDERSTOOD_MESSAGE = {
  assistantResponseMessage: {
    content: "understood"
  }
}

// 工具调用失败消息
function FAILED_TOOL_USE_MESSAGE(toolUseIds: string[]) {
  return {
    userInputMessage: {
      content: "",
      origin: "AI_EDITOR",
      userInputMessageContext: {
        toolResults: toolUseIds.map(toolUseId => ({
          toolUseId,
          content: [{ text: "Tool execution failed" }],
          status: "error"
        }))
      }
    }
  }
}
```

---

## 消息清理流程

`sanitizeConversation` 函数按以下顺序执行清理：

```typescript
function sanitizeConversation(messages) {
  let sanitized = [...messages]
  
  // 1. 确保以 user 消息开始
  sanitized = ensureStartsWithUserMessage(sanitized)
  
  // 2. 移除空的 user 消息
  sanitized = removeEmptyUserMessages(sanitized)
  
  // 3. 重新排序 tool result 消息
  sanitized = reorderToolResultMessages(sanitized)
  
  // 4. 确保工具调用有对应结果
  sanitized = ensureValidToolUsesAndResults(sanitized)
  
  // 5. 确保消息交替（user → assistant → user → assistant）
  sanitized = ensureAlternatingMessages(sanitized)
  
  // 6. 确保以 user 消息结束
  sanitized = ensureEndsWithUserMessage(sanitized)
  
  return sanitized
}
```

---

## 各步骤详解

### 1. ensureStartsWithUserMessage

确保消息列表以 user 消息开始。

```typescript
function ensureStartsWithUserMessage(messages) {
  if (isUserInputMessage(messages[0])) {
    return messages
  }
  return [HELLO_MESSAGE, ...messages]
}
```

**作用**：如果第一条消息不是 user 消息，在开头插入 `HELLO_MESSAGE`。

---

### 2. removeEmptyUserMessages

移除空的 user 消息（除了第一条）。

```typescript
function removeEmptyUserMessages(messages) {
  if (messages.length <= 1) {
    return messages
  }
  
  const firstUserMessageIndex = messages.findIndex(isUserInputMessage)
  
  return messages.filter((message, index) => {
    // 保留所有 assistant 消息
    if (isAssistantResponseMessage(message)) return true
    
    // 保留第一条 user 消息
    if (isUserInputMessage(message) && index === firstUserMessageIndex) return true
    
    // 检查 user 消息是否有内容或 tool results
    if (isUserInputMessage(message)) {
      const hasContent = message.userInputMessage?.content?.trim() !== ''
      const hasToolResults = !!message.userInputMessage?.userInputMessageContext?.toolResults?.length
      return hasContent || hasToolResults
    }
    
    return true
  })
}
```

**作用**：移除空的 user 消息，但保留：
- 第一条 user 消息（即使为空）
- 有内容的 user 消息
- 有 tool results 的 user 消息

---

### 3. reorderToolResultMessages

重新排序 tool result 消息，确保它们紧跟在对应的 tool use 之后。

```typescript
function reorderToolResultMessages(messages) {
  const toolUseIndices = []
  const toolResultIndices = new Map()
  
  // 找出所有 tool use 和 tool result 的位置
  for (let i = 0; i < messages.length; i++) {
    const message = messages[i]
    
    if (isAssistantResponseMessage(message) && hasToolUses(message)) {
      toolUseIndices.push(i)
    }
    
    if (isUserInputMessage(message) && hasToolResults(message)) {
      const toolResults = message.userInputMessage.userInputMessageContext.toolResults
      for (const result of toolResults) {
        toolResultIndices.set(result.toolUseId, i)
      }
    }
  }
  
  // 重新排序（具体实现略）
  // ...
}
```

**作用**：确保 tool result 消息紧跟在对应的 tool use 消息之后。

---

### 4. ensureValidToolUsesAndResults

确保每个 tool use 都有对应的 tool result。

```typescript
function ensureValidToolUsesAndResults(messages) {
  const result = []
  
  for (let i = 0; i < messages.length; i++) {
    const message = messages[i]
    result.push(message)
    
    // 检查 assistant 消息是否有 tool uses
    if (isAssistantResponseMessage(message) && hasToolUses(message)) {
      const nextMessage = i + 1 < messages.length ? messages[i + 1] : null
      
      // 情况1：没有下一条消息，或下一条不是 user 消息，或没有 tool results
      if (!nextMessage || !isUserInputMessage(nextMessage) || !hasToolResults(nextMessage)) {
        const toolUses = message.assistantResponseMessage.toolUses
        const toolUseIds = toolUses.map((tu, idx) => tu.toolUseId ?? `toolUse_${idx + 1}`)
        result.push(FAILED_TOOL_USE_MESSAGE(toolUseIds))
      }
      // 情况2：tool results 不匹配
      else if (!hasMatchingToolResults(
        message.assistantResponseMessage.toolUses,
        nextMessage.userInputMessage.userInputMessageContext.toolResults
      )) {
        // 检查是否有其他地方匹配
        const hasMatchingToolUseElsewhere = messages.some((msg, j) => 
          j !== i && 
          isAssistantResponseMessage(msg) && 
          hasToolUses(msg) && 
          hasMatchingToolResults(
            msg.assistantResponseMessage.toolUses,
            nextMessage.userInputMessage.userInputMessageContext.toolResults
          )
        )
        
        // 如果没有其他地方匹配，添加失败消息
        if (!hasMatchingToolUseElsewhere) {
          const toolUses = message.assistantResponseMessage.toolUses
          const toolUseIds = toolUses.map((tu, idx) => tu.toolUseId ?? `toolUse_${idx + 1}`)
          result.push(FAILED_TOOL_USE_MESSAGE(toolUseIds))
        }
      }
    }
  }
  
  return result
}
```

**作用**：
- 检查每个 tool use 是否有对应的 tool result
- 如果没有，自动添加失败消息（status: "error"）
- 验证 tool use 和 tool result 的 ID 是否匹配

**工具结果匹配检查**：

```typescript
function hasMatchingToolResults(toolUses, toolResults) {
  if (!toolUses || !toolUses.length) return true
  if (!toolResults || !toolResults.length) return false
  
  // 检查所有 tool use 都有对应的 result
  const allToolUsesHaveResults = toolUses.every(
    toolUse => toolResults.some(result => result.toolUseId === toolUse.toolUseId)
  )
  
  // 检查所有 tool result 都有对应的 use
  const allToolResultsHaveUses = toolResults.every(
    result => toolUses.some(toolUse => result.toolUseId === toolUse.toolUseId)
  )
  
  return allToolUsesHaveResults && allToolResultsHaveUses
}
```

---

### 5. ensureAlternatingMessages

确保消息交替出现（user → assistant → user → assistant）。

```typescript
function ensureAlternatingMessages(messages) {
  if (messages.length <= 1) return messages
  
  const result = [messages[0]]
  
  for (let i = 1; i < messages.length; i++) {
    const prevMessage = result[result.length - 1]
    const currentMessage = messages[i]
    
    // 两条连续的 user 消息 → 插入 UNDERSTOOD_MESSAGE
    if (isUserInputMessage(prevMessage) && isUserInputMessage(currentMessage)) {
      result.push(UNDERSTOOD_MESSAGE)
    }
    // 两条连续的 assistant 消息 → 插入 CONTINUE_MESSAGE
    else if (isAssistantResponseMessage(prevMessage) && isAssistantResponseMessage(currentMessage)) {
      result.push(CONTINUE_MESSAGE)
    }
    
    result.push(currentMessage)
  }
  
  return result
}
```

**作用**：在连续的同角色消息之间插入占位消息。

---

### 6. ensureEndsWithUserMessage

确保消息列表以 user 消息结束。

```typescript
function ensureEndsWithUserMessage(messages) {
  if (messages.length === 0) {
    return [HELLO_MESSAGE]
  }
  
  if (isUserInputMessage(messages[messages.length - 1])) {
    return messages
  }
  
  return [...messages, CONTINUE_MESSAGE]
}
```

**作用**：如果最后一条消息不是 user 消息，在末尾添加 `CONTINUE_MESSAGE`。

---

## 辅助函数

### 类型检查

```typescript
function isUserInputMessage(message) {
  return message != null && 
         'userInputMessage' in message && 
         message.userInputMessage != null
}

function isAssistantResponseMessage(message) {
  return message != null && 
         'assistantResponseMessage' in message && 
         message.assistantResponseMessage != null
}

function hasToolResults(message) {
  return !!message.userInputMessage?.userInputMessageContext?.toolResults?.length
}

function hasToolUses(message) {
  return !!message.assistantResponseMessage?.toolUses?.length
}
```

---

## 我们的实现状态

### ✅ 已完成的功能（v0.3.8）

**基础功能**：
- `trim_message_history` - 截断历史消息
- `merge_adjacent_messages` - 合并相邻同角色消息
- 确保以 user 消息开始和结束

**消息清理（sanitizeConversation）**：
- ✅ `ensure_starts_with_user_message` - 确保以 user 消息开始
- ✅ `remove_empty_user_messages` - 移除空的 user 消息（保留第一条和有内容/tool results 的）
- ✅ `reorder_tool_result_messages` - 重新排序 tool result 消息，确保紧跟在 tool use 之后
- ✅ `ensure_valid_tool_uses_and_results` - 工具调用验证，自动添加失败消息
- ✅ `ensure_alternating_messages` - 确保消息交替（user → assistant → user → assistant）
- ✅ `ensure_ends_with_user_message` - 确保以 user 消息结束
- ✅ `has_matching_tool_results` - 工具结果匹配检查
- ✅ 标准占位消息 - Hello/Continue/understood

**原生 History 支持**：
- ✅ 使用 Kiro API 原生的 `history` 字段传递历史消息
- ✅ `history` 包含除最后一条外的所有消息
- ✅ `currentMessage` 包含最后一条消息（必须是 user 消息）
- ✅ 消息清理在构建 history 之前执行

**实现位置**：
- 文件：`src-tauri/src/converter.rs`
- 函数：`sanitize_conversation()` - 消息清理
- 函数：`reorder_tool_result_messages()` - 工具结果重排序
- 函数：`build_kiro_payload()` - 构建请求，拆分 history 和 currentMessage
- 调用位置：`build_kiro_payload()` 中，在 `merge_adjacent_messages()` 之后

### ⚠️ 完整实现

所有 Kiro IDE 的消息清理功能已全部实现，与官方实现完全一致。

---

## 注意事项

1. **消息清理是必须的** - Kiro API 对消息格式要求严格
2. **顺序很重要** - 清理步骤的顺序不能随意调整
3. **占位消息是标准的** - 使用 Kiro 官方的占位消息格式
4. **工具调用必须有结果** - 没有结果会导致 API 错误
5. **消息必须交替** - 不能有连续的同角色消息

---

## 参考资料

- Kiro IDE 源码: `C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js`
- Kiro Account Manager: https://github.com/chaogei/Kiro-account-manager/commit/624a3c4
- 当前实现: `src-tauri/src/converter.rs`
