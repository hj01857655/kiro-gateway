# Kiro API 健康检查规范

## 概述

Kiro API 的健康检查有两种方式：
1. **dryRun 模式**（不推荐）- 理论上可用，但实际测试一直返回 400 错误
2. **配额查询**（推荐）- 稳定可靠，同时可以获取配额信息

## 推荐方案：配额查询

### 端点

```
POST https://app.kiro.dev/service/KiroWebPortalService/operation/GetUserUsageAndLimits
```

### 请求头

```
Authorization: Bearer {accessToken}
Content-Type: application/cbor
Accept: application/cbor
smithy-protocol: rpc-v2-cbor
x-amz-user-agent: KiroIDE-{版本}-{机器ID}
Cookie: Idp=BuilderId; AccessToken={accessToken}
```

### 请求体（CBOR 格式）

```json
{
  "isEmailRequired": true,
  "origin": "KIRO_IDE"
}
```

### 响应

#### 成功响应

- **状态码**: 200 OK
- **格式**: CBOR
- **说明**: Token 有效，账号健康，返回配额信息

#### 失败响应

- **401** → Token 过期，需要刷新
- **403/423** → Token 无效或账号封禁
- **其他** → 网络错误或服务异常

### 优势

1. **稳定可靠** - 配额查询是必须的功能，肯定可用
2. **一举两得** - 同时验证健康和获取配额信息
3. **避免格式问题** - 不需要处理复杂的 dryRun 格式

### 实现示例

```rust
pub async fn health_check(&self, account: &Account) -> bool {
    match self.get_usage_limits(account).await {
        Ok(_) => {
            tracing::debug!("[健康检查] 账号 {} 健康", account.id);
            true
        }
        Err(AppError::TokenExpired) => {
            tracing::warn!("[健康检查] 账号 {} Token 过期", account.id);
            false
        }
        Err(AppError::AccountBanned(_)) => {
            tracing::warn!("[健康检查] 账号 {} 已被封禁", account.id);
            false
        }
        Err(e) => {
            tracing::warn!("[健康检查] 账号 {} 检查失败: {}", account.id, e);
            false
        }
    }
}
```

---

## 备选方案：dryRun 模式（不推荐）

### 问题

经过多次测试，dryRun 模式一直返回 400 错误：`{"message":"Improperly formed request.","reason":null}`

尝试过的修复：
1. 将 `dryRun` 从布尔值改为空数组 `[]`
2. 添加 `origin: "AI_EDITOR"` 字段
3. 将 `content` 从数组改为字符串
4. 添加 `chatTriggerType: "MANUAL"` 字段
5. 添加 `history: []` 空数组

**结论**：dryRun 功能可能已被废弃或有未知的格式要求，不建议使用。

### 端点

```
POST https://codewhisperer.us-east-1.amazonaws.com/SendMessageStreaming
```

### 请求体格式（理论上）

```json
{
  "conversationState": {
    "conversationId": "uuid",
    "chatTriggerType": "MANUAL",
    "currentMessage": {
      "userInputMessage": {
        "content": "ping",
        "origin": "AI_EDITOR",
        "userIntent": "CODE_GENERATION"
      }
    },
    "history": []
  },
  "profileArn": "账号的 profileArn",
  "dryRun": [],
  "source": "AGENT"
}
```

### 为什么不推荐

1. **不稳定** - 实际测试一直失败
2. **格式复杂** - 需要精确匹配未知的格式要求
3. **无额外价值** - 只能验证健康，无法获取配额信息
4. **可能已废弃** - AWS 官方文档中未找到相关说明

---

## 参考

- **KiroGate 实现**: 使用配额查询验证健康（`E:\VSCodeSpace\Kiro\KiroGate\kiro_gateway\health_checker.py`）
- **当前实现**: `src-tauri/src/kiro_client.rs` 的 `health_check` 和 `get_usage_limits` 函数
- **Kiro IDE 源码**: `extension.js` 的 `se_SendMessageCommand` 函数（dryRun 相关）
- **AWS 官方**: `aws/aws-toolkit-vscode` 的 `SendMessageCommand.ts`（未找到 dryRun 实现）

---

## 总结

**推荐使用配额查询进行健康检查**，这是 KiroGate 的成熟方案，稳定可靠且一举两得。dryRun 模式虽然理论上可行，但实际测试失败，不建议使用。
