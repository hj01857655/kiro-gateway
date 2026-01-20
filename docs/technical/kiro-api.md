---
inclusion: fileMatch
fileMatchPattern: "**/kiro_client*.{rs,ts}"
---

# Kiro API 规范

## 端点

### 聊天接口
`POST https://codewhisperer.us-east-1.amazonaws.com/generateAssistantResponse`

### 模型列表接口
`GET https://codewhisperer.us-east-1.amazonaws.com/ListAvailableModels`

参数：
- `origin`: "AI_EDITOR"
- `profileArn`: 账号的 profileArn（可为空）

返回：
```json
{
  "models": [
    {
      "modelId": "qdev::auto",
      "tokenLimits": {
        "maxInputTokens": 200000,
        "maxOutputTokens": 8192
      }
    },
    {
      "modelId": "qdev::claude-haiku-4.5",
      "tokenLimits": {
        "maxInputTokens": 200000,
        "maxOutputTokens": 8192
      }
    },
    {
      "modelId": "qdev::claude-sonnet-4",
      "tokenLimits": {
        "maxInputTokens": 200000,
        "maxOutputTokens": 8192
      }
    },
    {
      "modelId": "qdev::claude-sonnet-4.5",
      "tokenLimits": {
        "maxInputTokens": 200000,
        "maxOutputTokens": 8192
      }
    }
  ]
}
```

注意：
- 模型 ID 带有 `qdev::` 前缀
- 返回的模型列表取决于账号类型和权限
- Claude Opus 4.5 已不再可用（2026-01-15 测试）

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
