# Kiro API 规范

## 端点

`POST https://codewhisperer.us-east-1.amazonaws.com/generateAssistantResponse`

## 必需请求头

```
Authorization: Bearer {accessToken}
x-amzn-kiro-agent-mode: vibe
x-amz-user-agent: KiroIDE-{版本}-{机器ID}
amz-sdk-invocation-id: {UUID}
amz-sdk-request: attempt=1; max=3
```

## 请求体结构

```json
{
  "conversationState": {
    "conversationId": "uuid",
    "chatTriggerType": "MANUAL",
    "currentMessage": {
      "userInputMessage": {
        "content": ["用户消息"],
        "modelId": "qdev::claude-sonnet-4.5",
        "userIntent": "CODE_GENERATION",
        "userInputMessageContext": {
          "tools": [],
          "additionalContext": { "systemPrompt": "..." }
        }
      }
    },
    "history": []
  },
  "profileArn": ""
}
```

## SSE 事件类型

Kiro 返回 AWS Event Stream 二进制格式：

- `assistantResponseEvent` → 文本内容
- `toolUseEvent` → 工具调用
- `metadataEvent.tokenUsage` → usage 统计（inputTokens = totalTokens - outputTokens）
- `reasoningContentEvent` → thinking block
- `codeEvent` → 代码块（转 Markdown）
- `dryRunSucceedEvent` → 干运行成功（健康检查用）
