# Kiro Token 刷新 API 文档

## 概述

Kiro 支持两种账号类型，每种类型有不同的 Token 刷新方式：
- **Social 账号**：个人账号（Google/GitHub 登录）
- **IDC 账号**：企业账号（AWS IAM Identity Center）

## Social 账号刷新

### 端点

```
POST https://prod.{region}.auth.desktop.kiro.dev/refreshToken
```

**默认 region**: `us-east-1`

### 请求格式

```json
{
  "refreshToken": "aorAAAAA..."
}
```

### 响应格式

```json
{
  "accessToken": "eyJraWQ...",
  "refreshToken": "aorAAAAA...",
  "expiresIn": 3600
}
```

### 字段说明

**请求**:
- `refreshToken`: 刷新令牌（必需）

**响应**:
- `accessToken`: 新的访问令牌
- `refreshToken`: 新的刷新令牌（可选，如果返回则需要更新）
- `expiresIn`: 过期时间（秒），默认 3600（1 小时）

### 错误响应

```json
{
  "error": "invalid_grant",
  "error_description": "Invalid refresh token"
}
```

## IDC 账号刷新

### 端点

```
POST https://oidc.{region}.amazonaws.com/token
```

**默认 region**: `us-east-1`

### 请求格式

```json
{
  "clientId": "MkAG97...",
  "clientSecret": "eyJraWQ...",
  "grantType": "refresh_token",
  "refreshToken": "aorAAAAA..."
}
```

### 响应格式

```json
{
  "accessToken": "eyJraWQ...",
  "refreshToken": "aorAAAAA...",
  "expiresIn": 3600,
  "tokenType": "Bearer"
}
```

### 字段说明

**请求**:
- `clientId`: 客户端 ID（必需）
- `clientSecret`: 客户端密钥（必需）
- `grantType`: 固定为 `"refresh_token"`
- `refreshToken`: 刷新令牌（必需）

**响应**:
- `accessToken`: 新的访问令牌
- `refreshToken`: 新的刷新令牌（可选）
- `expiresIn`: 过期时间（秒）
- `tokenType`: 令牌类型（通常为 `"Bearer"`）

### 错误响应

```json
{
  "error": "invalid_client",
  "error_description": "Client authentication failed"
}
```

## 客户端注册信息

IDC 账号的 `clientId` 和 `clientSecret` 存储在本地缓存中：

**路径**: `~/.aws/sso/cache/{clientIdHash}.json`

**格式**:
```json
{
  "clientId": "MkAG97...",
  "clientSecret": "eyJraWQ...",
  "clientIdIssuedAt": 1234567890,
  "clientSecretExpiresAt": 1234567890,
  "registrationAccessToken": "...",
  "tokenEndpoint": "https://oidc.us-east-1.amazonaws.com/token"
}
```

## Token 过期策略

### 过期时间计算

```
expiresAt = currentTime + (expiresIn * 1000)  // 转换为毫秒
```

### 提前刷新

为避免 Token 在请求过程中过期，建议提前 5 分钟刷新：

```
shouldRefresh = currentTime >= (expiresAt - 5 * 60 * 1000)
```

### 自动刷新流程

1. **请求前检查**: 如果 Token 即将过期（5 分钟内），先刷新
2. **请求失败处理**: 如果收到 401 错误，刷新后重试
3. **刷新失败**: 标记账号为 `expired` 状态，提示用户重新登录

## KiroGate 实现

### 后端接口

```
POST /admin/accounts/:account_id/refresh
```

**功能**:
1. 根据账号类型选择刷新方式
2. 更新内存中的 Token
3. 更新配置文件中的 Token
4. 返回刷新结果

**响应**:
```json
{
  "success": true,
  "accountId": "xxx",
  "status": "active"
}
```

### 自动刷新逻辑

KiroGate 在以下情况自动刷新 Token：

1. **请求前检查** (`get_account`):
   - 检查 Token 是否即将过期（5 分钟内）
   - 如果是，自动刷新后返回

2. **请求失败重试** (`generate_with_refresh`):
   - 如果收到 `TokenExpired` 错误
   - 自动刷新后重试请求

3. **配额查询** (`admin_get_quota`):
   - 查询前自动刷新 Token
   - 确保配额数据准确

### 刷新失败处理

```rust
match result {
    Ok(()) => {
        // 更新内存和文件
        account.status = AccountStatus::Active;
        Ok(account)
    }
    Err(e) => {
        // 标记为过期
        account.status = AccountStatus::Expired;
        Err(e)
    }
}
```

## 常见问题

### Q: refreshToken 长度过短怎么办？

A: refreshToken 通常长度 > 100 字符，如果过短可能被截断。检查：
- 复制时是否完整
- 配置文件格式是否正确
- 是否有换行符或空格

### Q: 刷新后 refreshToken 会变吗？

A: 
- **Social 账号**: 可能返回新的 refreshToken，需要更新
- **IDC 账号**: 通常不变，但建议检查响应并更新

### Q: Token 刷新频率限制？

A: 
- 建议提前 5 分钟刷新，避免频繁请求
- 刷新失败后不要立即重试，等待一段时间

### Q: 如何判断账号类型？

A:
```rust
fn is_idc(account: &Account) -> bool {
    account.client_id.is_some() || 
    account.auth_method.to_lowercase() == "idc"
}
```

## 参考资料

- **Kiro 源码**: `C:\Users\{用户名}\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js`
- **AWS OIDC 文档**: https://docs.aws.amazon.com/singlesignon/latest/OIDCAPIReference/Welcome.html
- **参考项目**: [AIClient-2-API](https://github.com/justlovemaki/AIClient-2-API)

## 安全建议

1. **不要硬编码**: refreshToken 和 clientSecret 不要提交到代码仓库
2. **文件权限**: 配置文件应设置为仅当前用户可读（600）
3. **HTTPS 传输**: 所有 Token 刷新请求必须使用 HTTPS
4. **定期轮换**: 建议定期重新登录，获取新的 refreshToken
5. **错误日志**: 不要在日志中输出完整的 Token 内容
