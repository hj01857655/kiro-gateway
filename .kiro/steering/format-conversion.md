# 格式转换规范

## 请求转换

### OpenAI → Kiro

- `messages[role=system]` → `additionalContext.systemPrompt`
- `messages[role=user]` 最后一条 → `currentMessage.userInputMessage.content`
- 之前的消息 → `history`
- `tools` → `userInputMessageContext.tools`（toolSpec 格式）
- `model` → `userInputMessage.modelId`（qdev:: 前缀）

### Anthropic → Kiro

- `system` → `additionalContext.systemPrompt`
- `messages` 处理同 OpenAI
- `tools` → input_schema 转 inputSchema.json

## 响应转换

### Kiro → OpenAI

```
assistantResponseEvent → choices[0].delta.content
toolUseEvent → choices[0].delta.tool_calls
结束 → finish_reason: stop/tool_calls
最后 → data: [DONE]
```

### Kiro → Anthropic

```
assistantResponseEvent → content_block_delta (text_delta)
toolUseEvent → content_block_start + input_json_delta + content_block_stop
reasoningContentEvent → thinking block 完整生命周期
结束 → message_delta + message_stop
```

## 注意事项

- 相邻同 role 消息必须合并
- 图片放 `userInputMessage.images`，不是 content
- 长 tool description 移到 system prompt
- Kiro 不支持 temperature/top_p/max_tokens，直接忽略
