# Kiro API 规范

## 官方源码参考

**AWS Toolkit for VS Code (GitHub)**: [aws/aws-toolkit-vscode](https://github.com/aws/aws-toolkit-vscode)
- 聊天客户端: `packages/core/src/codewhispererChat/clients/chat/v0/chat.ts`
- SSO 账号使用 `generateAssistantResponse` 接口
- IAM 账号使用 `sendMessage` 接口

**Kiro IDE 本地安装路径**:
- Windows: `C:\Users\{用户名}\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js`
- 关键函数: `convertToGenerateAssistantMessages` (行号约 702988)
- API 调用: `GenerateAssistantResponseCommand`

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
        "content": "用户消息",
        "modelId": "auto",
        "origin": "AI_EDITOR"
      }
    },
    "history": []
  }
}
```

**注意事项**：
- `content` 是字符串，不是数组
- `modelId` 不需要 `qdev::` 前缀，直接用 `"auto"`, `"claude-sonnet-4.5"` 等
- `origin` 必须是 `"AI_EDITOR"`
- `userInputMessageContext` 只在有 tools 时才添加
- Social 账号可选添加 `profileArn`（通常为空字符串）
- IDC 账号不要添加 `profileArn` 字段

## SSE 事件类型

Kiro 返回文本流，内嵌 JSON 对象：

- `assistantResponseEvent` → 文本内容
  ```json
  {"content": "响应文本"}
  ```

- `codeEvent` → 代码块
  ```json
  {"content": "代码内容", "language": "python"}
  ```

- `toolUseEvent` → 工具调用
  ```json
  {"name": "工具名", "toolUseId": "id", "input": {...}}
  ```

- `metadataEvent` → Token 使用统计
  ```json
  {"tokenUsage": {"totalTokens": 100, "outputTokens": 50}}
  ```
  - inputTokens = totalTokens - outputTokens

- `reasoningContentEvent` → Thinking block
  ```json
  {"text": "思考内容", "signature": "签名"}
  ```

- `messageMetadataEvent` → 消息元数据
  ```json
  {"messageId": "msg_xxx"}
  ```

- `contextUsageEvent` → 上下文使用率
  ```json
  {"contextUsagePercentage": 0.75}
  ```

- `invalidStateEvent` → 错误事件
  ```json
  {"reason": "CONTEXT_LENGTH_EXCEEDED", "message": "错误信息"}
  ```

## 响应格式

Kiro 返回的是**文本流**，其中包含多个 JSON 对象（不是标准 SSE 格式）。

解析方式：
1. 在文本流中查找 `{` 字符
2. 使用括号计数法找到完整的 JSON 对象
3. 解析每个 JSON 对象为事件

**不是** AWS Event Stream 二进制格式，是纯文本 JSON 流。
