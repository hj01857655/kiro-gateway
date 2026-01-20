---
inclusion: fileMatch
fileMatchPattern: "**/error*.{rs,ts}"
---

# 错误处理规范

## Kiro 错误类型

**认证错误 (401/403)**
- `ExpiredTokenException` → 刷新 Token 重试
- `AccessDeniedException` → 检查账号权限

**限流错误 (429)**
- `ThrottlingException` → 指数退避重试
- `ServiceQuotaExceededException` → 配额用尽，不重试

**请求错误 (400)**
- `ValidationException` → 检查请求格式
- `CONTENT_LENGTH_EXCEEDS_THRESHOLD` → 自动截断历史消息并重试

**服务错误 (5xx)**
- `InternalServerException` → 重试 3 次
- `MODEL_TEMPORARILY_UNAVAILABLE` → 稍后重试

## 重试策略

```
限流: 基础 500ms，指数退避，最大 20s
普通: 基础 100ms，指数退避，最大 20s
算法: Math.random() * 2^attempt * base
```

## 错误转换

**→ OpenAI**
```json
{ "error": { "message": "...", "type": "...", "code": "..." } }
```

**→ Anthropic**
```json
{ "type": "error", "error": { "type": "...", "message": "..." } }
```

## 流式错误

- First Token 超时：60s（Opus 120s）
- 流读取超时：120s（Opus 300s）
- 超时后重试，最多 2 次

---

## 历史消息自动截断

### 触发条件

当 Kiro API 返回 `CONTENT_LENGTH_EXCEEDS_THRESHOLD` 错误时，自动触发历史消息截断。

### 工作原理

```
用户请求 → Kiro API 返回 400 错误（内容过长）
    ↓
捕获 CONTENT_LENGTH_EXCEEDS_THRESHOLD 错误
    ↓
截断历史消息（只保留最后一对对话）
    ↓
递归调用 generate_with_refresh 重试
    ↓
如果仍然超长 → 再次截断 → 再次重试
    ↓
直到成功或无法继续截断
```

### 截断策略

参考 Kiro IDE 的 `trimMessageHistory` 实现：

1. **合并相邻同角色消息** - 避免重复处理
2. **识别完整对话对** - 找出所有 user/assistant 配对
3. **保留最后一对** - 只保留最近的一次完整对话
4. **保留未回复的 user 消息** - 如果最后一条是 user 消息且未配对

### 实现流程

```rust
// 1. 捕获错误
Err(AppError::BadRequest(ref msg)) 
    if msg.contains("CONTENT_LENGTH_EXCEEDS_THRESHOLD")

// 2. 转换格式
HistoryItem → ChatMessage

// 3. 截断历史
trim_message_history(&messages)

// 4. 转换回格式
ChatMessage → HistoryItem

// 5. 递归重试（使用 Box::pin 避免无限大小的 future）
Box::pin(self.generate_with_refresh(request, accounts, model)).await
```

### 边界检查

- **空历史检查** - 如果历史为空，返回错误（可能是当前消息过长）
- **截断效果检查** - 如果截断后消息数量未减少，返回错误避免无限循环
- **递归深度控制** - 通过边界检查自然限制递归深度

### 错误处理

| 场景 | 处理方式 |
|------|---------|
| 历史消息为空 | 返回 "当前消息内容过长，无法处理" |
| 截断后仍超长 | 继续截断直到成功或无法继续 |
| 截断无效果 | 返回 "历史消息已是最小，但仍超过长度限制" |

### 日志记录

```rust
info!("原始历史消息数量: {}", history.len());
info!("截断后历史消息数量: {}", trimmed.len());
info!("历史消息已截断，重新发起请求...");
```

### 注意事项

1. **递归异步函数** - 必须使用 `Box::pin` 包装递归调用
2. **格式转换** - `HistoryItem` ↔ `ChatMessage` 需要保留必要字段
3. **工具调用** - 截断时会丢失 `tool_uses` 和 `tool_results`（简化处理）
4. **图片数据** - 历史消息中的图片会被省略（避免请求体过大）
