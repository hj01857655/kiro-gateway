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
- `Input is too long` → 上下文超限

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
