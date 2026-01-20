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

1. ✅ **刷新响应新增 `profileArn` 字段**
   - 格式：`arn:aws:codewhisperer:{region}:{accountId}:profile/{profileId}`
   - **重要说明**：Social 账号一直有 `profileArn`，只是之前刷新响应中不返回，现在新格式返回了
   - 这样可以在刷新时自动更新 `profileArn`，无需手动维护
   - 示例：`"arn:aws:codewhisperer:us-east-1:699475941385:profile/EHGA3GRVQMUK"`

2. ✅ **字段顺序变化**
   - 旧格式：`accessToken` → `refreshToken` → `expiresIn`
   - 新格式：`accessToken` → `expiresIn` → `profileArn` → `refreshToken`

3. ❌ **Social 账号没有 AWS SSO 字段**
   - 不包含 `aws_sso_app_session_id`、`idToken`、`issuedTokenType`、`originSessionId`
   - 这些字段只出现在 IDC 账号的刷新响应中

4. ✅ **核心字段保持不变**
   - `accessToken`、`refreshToken`、`expiresIn` 仍然存在
   - 字段类型和含义不变

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
   - `aws_sso_app_session_id` - AWS SSO 应用会话 ID（通常为 `null`）
   - `idToken` - ID Token（OpenID Connect，通常为 `null`）
   - `issuedTokenType` - 发行的 Token 类型（通常为 `null`）
   - `originSessionId` - 原始会话 ID（通常为 `null`）
   - 这些字段在实际响应中通常为 `null`，但可能在某些情况下有值
   - 字段位置：插入在 `accessToken` 之后

2. ❌ **IDC 刷新响应不包含 `profileArn` 字段**
   - IDC 账号的 `profileArn` 在添加账号时设置
   - 刷新响应中不返回此字段（与 Social 账号不同）

3. ✅ **核心字段保持不变**
   - `accessToken`、`refreshToken`、`expiresIn`、`tokenType` 仍然存在
   - 字段类型和含义不变

4. ✅ **字段顺序变化**
   - 旧格式：`accessToken` → `refreshToken` → `expiresIn` → `tokenType`
   - 新格式：`accessToken` → `aws_sso_app_session_id` → `expiresIn` → `idToken` → `issuedTokenType` → `originSessionId` → `refreshToken` → `tokenType`

---

## 对比总结

| 特性 | Social 账号 | IDC 账号 |
|------|------------|----------|
| 新增 `profileArn` | ✅ 有（新增） | ❌ 无（不在刷新响应中） |
| 新增 AWS SSO 字段 | ❌ 无 | ✅ 有（4个，通常为 null） |
| 核心字段 | 保持不变 | 保持不变 |
| 字段顺序 | 有变化 | 有变化 |
| `tokenType` 字段 | ❌ 无 | ✅ 有（固定为 "Bearer"） |

**关键区别**：
- Social 账号刷新响应**有** `profileArn`，**没有** AWS SSO 字段
- IDC 账号刷新响应**有** AWS SSO 字段，**没有** `profileArn`
- 两种账号的刷新响应格式完全不同

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

- ✅ **无需任何操作** - 现有账号自动适配新格式
- ✅ **Token 刷新继续正常工作** - 兼容新旧两种格式
- ✅ **Social 账号自动获取 `profileArn`** - 刷新成功后自动保存
- ✅ **IDC 账号的 `profileArn` 保持不变** - 不会被刷新响应覆盖
- ✅ **AWS SSO 字段自动处理** - 如果有值会在日志中记录
- ✅ **向后兼容** - 旧版本的账号配置仍然有效

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

A: Social 账号一直有 `profileArn`，只是之前刷新响应中不返回这个字段。现在 Kiro API 更新后，刷新响应中也会返回 `profileArn`，方便自动更新。现在 kiro-gateway 会自动提取并保存这个字段。

### Q: IDC 账号的 `profileArn` 从哪里来？

A: IDC 账号的 `profileArn` 在添加账号时由用户提供或从 Kiro IDE 导入，**不会从刷新响应中获取**。这是 Social 和 IDC 账号的关键区别之一。

### Q: AWS SSO 字段有什么用？

A: 这些字段是 AWS SSO 相关的元数据，在实际响应中通常为 `null`。kiro-gateway 会在日志中记录这些字段（如果有值），但不会使用它们。这些字段的存在是为了兼容 AWS SSO 的完整 OIDC 流程。

### Q: 为什么 Social 和 IDC 的响应格式这么不同？

A: 因为它们使用不同的认证端点：
- Social 账号使用 Kiro 自己的认证服务（`prod.us-east-1.auth.desktop.kiro.dev`）
- IDC 账号使用 AWS SSO OIDC 端点（`oidc.us-east-1.amazonaws.com`）

两个端点的响应格式遵循不同的规范，所以字段不同。

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
