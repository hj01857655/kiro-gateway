# Token 刷新响应格式更新说明

## 更新日期

2026-01-20

## 概述

Kiro API 更新了 Token 刷新响应格式，Social 和 IDC 账号的响应都有变化。

---

## Social 账号刷新

### 端点

```
POST https://prod.us-east-1.auth.desktop.kiro.dev/refreshToken
```

### 请求格式

```json
{
  "refreshToken": "aorAAAAA..."
}
```

### 响应格式变化

**旧格式**（兼容）：
```json
{
  "accessToken": "eyJraWQ...",
  "refreshToken": "aorAAAAA...",
  "expiresIn": 3600
}
```

**新格式**（2026-01-20 后）：
```json
{
  "accessToken": "aoaAAAAA...",
  "expiresIn": 3600,
  "profileArn": "arn:aws:codewhisperer:us-east-1:699475941385:profile/EHGA3GRVQMUK",
  "refreshToken": "aorAAAAA..."
}
```

### 主要变化

1. ✅ **新增 `profileArn` 字段**
   - 格式：`arn:aws:codewhisperer:{region}:{accountId}:profile/{profileId}`
   - 之前认为 Social 账号没有 profileArn，现在确认响应中包含

2. ✅ **字段顺序变化**
   - 旧格式：`accessToken` → `refreshToken` → `expiresIn`
   - 新格式：`accessToken` → `expiresIn` → `profileArn` → `refreshToken`

3. ❌ **没有 AWS SSO 字段**
   - 不包含 `aws_sso_app_session_id`、`idToken`、`issuedTokenType`、`originSessionId`

4. ✅ **核心字段保持不变**
   - `accessToken`、`refreshToken`、`expiresIn` 仍然存在

---

## IDC 账号刷新

### 端点

```
POST https://oidc.{region}.amazonaws.com/token
```

### 请求格式

```json
{
  "clientId": "MkAG97...",
  "clientSecret": "eyJraWQ...",
  "grantType": "refresh_token",
  "refreshToken": "aorAAAAA..."
}
```

### 响应格式变化

**旧格式**（兼容）：
```json
{
  "accessToken": "eyJraWQ...",
  "refreshToken": "aorAAAAA...",
  "expiresIn": 3600,
  "tokenType": "Bearer"
}
```

**新格式**（2026-01-20 后）：
```json
{
  "accessToken": "aoaAAAAA...",
  "aws_sso_app_session_id": null,
  "expiresIn": 3600,
  "idToken": null,
  "issuedTokenType": null,
  "originSessionId": null,
  "refreshToken": "aorAAAAA...",
  "tokenType": "Bearer"
}
```

### 主要变化

1. ✅ **新增 4 个 AWS SSO 字段**（可选）
   - `aws_sso_app_session_id` - AWS SSO 应用会话 ID
   - `idToken` - ID Token（OpenID Connect）
   - `issuedTokenType` - 发行的 Token 类型
   - `originSessionId` - 原始会话 ID
   - 这些字段通常为 `null`，但可能在某些情况下有值
   - 字段顺序：插入在 `accessToken` 和 `expiresIn` 之间

2. ❌ **不包含 `profileArn` 字段**
   - IDC 账号的 `profileArn` 在添加账号时设置
   - 刷新响应中不返回此字段

3. ✅ **核心字段保持不变**
   - `accessToken`、`refreshToken`、`expiresIn`、`tokenType` 仍然存在

4. ✅ **字段顺序变化**
   - 旧格式：`accessToken` → `refreshToken` → `expiresIn` → `tokenType`
   - 新格式：`accessToken` → `aws_sso_*` → `expiresIn` → `idToken` → `issuedTokenType` → `originSessionId` → `refreshToken` → `tokenType`

---

## 对比总结

| 特性 | Social 账号 | IDC 账号 |
|------|------------|----------|
| 新增 `profileArn` | ✅ 有 | ❌ 无 |
| 新增 AWS SSO 字段 | ❌ 无 | ✅ 有（4个） |
| 核心字段 | 保持不变 | 保持不变 |
| 字段顺序 | 有变化 | 无变化 |

---

## kiro-gateway 兼容性

### 已实现的更新

1. **Social 账号刷新**（`src-tauri/src/account.rs`）
   - 解析结构体新增 `profile_arn` 字段（可选）
   - 如果响应中包含 `profileArn`，自动更新到账号配置
   - 兼容旧格式（没有 `profileArn` 的响应）

2. **IDC 账号刷新**（`src-tauri/src/account.rs`）
   - 解析结构体新增 4 个 AWS SSO 字段（可选）
   - 日志中记录这些字段（如果存在）
   - 兼容旧格式（没有 AWS SSO 字段的响应）

3. **向后兼容**
   - 所有新增字段使用 `#[serde(default)]` 标记
   - 旧格式响应仍然能正常解析
   - 不影响现有账号的使用

### 用户影响

- ✅ **无需任何操作**
- ✅ **现有账号自动适配新格式**
- ✅ **Token 刷新继续正常工作**
- ✅ **Social 账号会自动获取和保存 `profileArn`**
- ✅ **如果 AWS SSO 字段有值，会在日志中记录**

---

## 技术细节

### Social 账号解析结构体

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RefreshResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    // 新格式中包含 profileArn
    #[serde(default)]
    profile_arn: Option<String>,
    // AWS SSO 字段（IDC 账号可能包含，Social 账号没有）
    #[serde(default)]
    aws_sso_app_session_id: Option<String>,
    #[serde(default)]
    id_token: Option<String>,
    #[serde(default)]
    issued_token_type: Option<String>,
    #[serde(default)]
    origin_session_id: Option<String>,
}
```

### IDC 账号解析结构体

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct IdcRefreshResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<i64>,
    // AWS SSO 字段（可选，新格式中可能包含）
    #[serde(default)]
    aws_sso_app_session_id: Option<String>,
    #[serde(default)]
    id_token: Option<String>,
    #[serde(default)]
    issued_token_type: Option<String>,
    #[serde(default)]
    origin_session_id: Option<String>,
}
```

### profileArn 更新逻辑

```rust
// Social 账号刷新成功后
if let Some(profile_arn) = data.profile_arn {
    if !profile_arn.is_empty() {
        account.profile_arn = profile_arn;
    }
}
```

---

## 常见问题

### Q: 为什么 Social 账号现在有 `profileArn` 了？

A: 之前的理解有误。实际上 Social 账号的刷新响应中一直包含 `profileArn`，只是之前没有正确处理。

### Q: IDC 账号的 `profileArn` 从哪里来？

A: IDC 账号的 `profileArn` 在添加账号时由用户提供或从 Kiro IDE 导入，不会从刷新响应中获取。

### Q: AWS SSO 字段有什么用？

A: 这些字段是 AWS SSO 相关的元数据，通常为 `null`。kiro-gateway 会记录这些字段（如果有值），但不会使用它们。

### Q: 旧版本的 kiro-gateway 能处理新格式吗？

A: 不能。旧版本会因为无法解析新字段而失败。请更新到 v0.3.5 或更高版本。

### Q: 新版本的 kiro-gateway 能处理旧格式吗？

A: 可以。新版本完全兼容旧格式，所有新增字段都是可选的。

---

## 相关文件

- 代码实现：`src-tauri/src/account.rs`
- 账号系统文档：`.kiro/steering/account-system.md`
- 本文档：`docs/token-refresh-format-update.md`

---

## 版本信息

- kiro-gateway 版本：v0.3.5+
- 更新日期：2026-01-20
- 文档版本：1.0
