---
inclusion: fileMatch
fileMatchPattern: "**/account*.{rs,ts,tsx}"
---

# 账号系统规范

## 概述

Kiro 支持两种认证方式：
1. **Social 登录** - 个人账号（Google/GitHub）
2. **IDC 登录** - 企业账号（AWS IAM Identity Center）

两种方式的 **Token 格式相同**，但 **刷新方式不同**。

### authMethod 标准值

**重要**：`authMethod` 字段的标准值（区分大小写）：
- Social 账号：`"social"` (全小写)
- IDC 账号：`"IdC"` (大写 I 和 C)

**前后端必须使用相同的标准值**，不要使用 `"Social"` 或 `"idc"` 等变体。

---

## 手动添加账号所需字段

### Social 账号（1个字段）
- **Refresh Token** - 刷新令牌

### IDC 账号（3个字段）
- **Client ID** - 客户端 ID
- **Client Secret** - 客户端密钥
- **Profile ARN** - 配置文件 ARN

**注意**：
- Social 账号不需要 Access Token（会自动通过 Refresh Token 获取）
- IDC 账号不需要 Region（默认使用 us-east-1）

---

## 账号类型对比

| 特性 | Social 账号 | IDC 账号 |
|------|------------|----------|
| 适用用户 | 个人用户 | 企业用户 |
| 登录方式 | Google / GitHub | AWS IAM Identity Center |
| 刷新端点 | Kiro Auth 服务 | AWS SSO OIDC |
| 需要 clientId | ❌ 不需要 | ✅ 需要 |
| profileArn | 空字符串 | 有值 |

---

## Token 结构

### Social Token

```json
{
  "accessToken": "eyJ...",
  "refreshToken": "eyJ...",
  "authMethod": "social",
  "provider": "Google",
  "region": "us-east-1",
  "expiresAt": "2026-01-10T13:02:24+00:00",
  "profileArn": ""
}
```

### IDC Token

```json
{
  "accessToken": "eyJ...",
  "refreshToken": "eyJ...",
  "authMethod": "IdC",
  "provider": "BuilderId",
  "region": "us-east-1",
  "expiresAt": "2026-01-10T13:02:24+00:00",
  "profileArn": "arn:aws:codewhisperer:us-east-1:123456789:profile/xxx",
  "clientIdHash": "abc123..."
}
```

**关键区别**：
- `authMethod`: `social` vs `IdC`
- `profileArn`: Social 为空，IDC 有值
- `clientIdHash`: 只有 IDC 有，用于查找客户端注册信息

---

## Token 管理

### 过期策略

- **有效期**: accessToken 有效期 1 小时
- **提前刷新**: 提前 5 分钟刷新，避免请求中过期
- **计算公式**: `shouldRefresh = currentTime >= (expiresAt - 5 * 60 * 1000)`

### 刷新流程

1. **请求前检查**: Token 即将过期（5 分钟内）→ 先刷新
2. **请求失败处理**: 收到 401 错误 → 刷新后重试
3. **刷新失败**: 标记账号为 `expired`，提示用户重新登录

---

## Token 刷新 API

### Social 账号刷新

**端点**: `POST https://prod.us-east-1.auth.desktop.kiro.dev/refreshToken`

**请求**:
```json
{
  "refreshToken": "aorAAAAA..."
}
```

**响应**:
```json
{
  "accessToken": "eyJraWQ...",
  "refreshToken": "aorAAAAA...",
  "expiresIn": 3600
}
```

### IDC 账号刷新

**端点**: `POST https://oidc.{region}.amazonaws.com/token`

**请求**:
```json
{
  "clientId": "MkAG97...",
  "clientSecret": "eyJraWQ...",
  "grantType": "refresh_token",
  "refreshToken": "aorAAAAA..."
}
```

**响应**:
```json
{
  "accessToken": "eyJraWQ...",
  "refreshToken": "aorAAAAA...",
  "expiresIn": 3600,
  "tokenType": "Bearer"
}
```

**IDC 刷新需要额外信息**：
- `clientId` 和 `clientSecret`（从客户端注册缓存获取）
- 缓存位置: `~/.aws/sso/cache/{clientIdHash}.json`

---

## 统一刷新实现

```javascript
class TokenRefresher {
  constructor() {
    this.clientRegistrations = new Map()
  }
  
  // 统一刷新入口
  async refresh(token) {
    if (token.authMethod === 'social') {
      return this.refreshSocial(token)
    } else if (token.authMethod === 'IdC') {
      return this.refreshIdc(token)
    } else {
      throw new Error(`未知的认证方式: ${token.authMethod}`)
    }
  }
  
  // Social 刷新
  async refreshSocial(token) {
    const response = await fetch(
      'https://prod.us-east-1.auth.desktop.kiro.dev/refreshToken',
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ refreshToken: token.refreshToken })
      }
    )
    
    if (!response.ok) {
      throw new Error(`Social Token 刷新失败: ${response.status}`)
    }
    
    const data = await response.json()
    
    return {
      ...token,
      accessToken: data.accessToken,
      refreshToken: data.refreshToken || token.refreshToken,
      expiresAt: new Date(Date.now() + data.expiresIn * 1000).toISOString()
    }
  }
  
  // IDC 刷新
  async refreshIdc(token) {
    const clientReg = this.loadClientRegistration(token.clientIdHash)
    
    if (!clientReg) {
      throw new Error('找不到 IDC 客户端注册信息，需要重新登录')
    }
    
    if (new Date(clientReg.expiresAt) <= new Date()) {
      throw new Error('IDC 客户端注册已过期，需要重新登录')
    }
    
    const response = await fetch(
      `https://oidc.${token.region || 'us-east-1'}.amazonaws.com/token`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          clientId: clientReg.clientId,
          clientSecret: clientReg.clientSecret,
          grantType: 'refresh_token',
          refreshToken: token.refreshToken
        })
      }
    )
    
    if (!response.ok) {
      throw new Error(`IDC Token 刷新失败: ${response.status}`)
    }
    
    const data = await response.json()
    
    return {
      ...token,
      accessToken: data.accessToken,
      refreshToken: data.refreshToken || token.refreshToken,
      expiresAt: new Date(Date.now() + data.expiresIn * 1000).toISOString()
    }
  }
  
  // 加载 IDC 客户端注册信息
  loadClientRegistration(clientIdHash) {
    if (this.clientRegistrations.has(clientIdHash)) {
      return this.clientRegistrations.get(clientIdHash)
    }
    
    const cachePath = path.join(
      os.homedir(), '.aws', 'sso', 'cache', `${clientIdHash}.json`
    )
    
    if (fs.existsSync(cachePath)) {
      const data = JSON.parse(fs.readFileSync(cachePath, 'utf-8'))
      this.clientRegistrations.set(clientIdHash, data)
      return data
    }
    
    return null
  }
}
```

---

## 凭证来源

### Kiro IDE 缓存

**主凭证文件**: `~/.aws/sso/cache/kiro-auth-token.json`  
**IDC 客户端注册**: `~/.aws/sso/cache/{clientIdHash}.json`

### 从缓存提取

```javascript
function getKiroToken() {
  const tokenPath = path.join(
    os.homedir(), '.aws', 'sso', 'cache', 'kiro-auth-token.json'
  )
  
  const token = JSON.parse(fs.readFileSync(tokenPath, 'utf-8'))
  
  // 如果是 IDC，还需要读取客户端注册信息
  if (token.authMethod === 'IdC' && token.clientIdHash) {
    const clientRegPath = path.join(
      os.homedir(), '.aws', 'sso', 'cache', `${token.clientIdHash}.json`
    )
    
    if (fs.existsSync(clientRegPath)) {
      token.clientRegistration = JSON.parse(
        fs.readFileSync(clientRegPath, 'utf-8')
      )
    }
  }
  
  return token
}
```

---

## 多账号策略

### 轮询机制

- 轮询选择可用账号
- Token 过期自动刷新
- 刷新失败标记为 `expired`
- 限流时跳过该账号 60 秒

### 账号状态

- `active` - 正常可用
- `expired` - Token 过期，需重新登录
- `error` - 其他错误
- `disabled` - 手动禁用

---

## KiroGate 账号配置

### Social 账号

```json
{
  "id": "social-1",
  "name": "Google 账号",
  "authMethod": "social",
  "provider": "Google",
  "accessToken": "eyJ...",
  "refreshToken": "eyJ...",
  "expiresAt": "2026-01-10T13:02:24+00:00",
  "profileArn": ""
}
```

### IDC 账号

```json
{
  "id": "idc-1",
  "name": "企业账号",
  "authMethod": "IdC",
  "provider": "BuilderId",
  "region": "us-east-1",
  "accessToken": "eyJ...",
  "refreshToken": "eyJ...",
  "expiresAt": "2026-01-10T13:02:24+00:00",
  "profileArn": "arn:aws:codewhisperer:us-east-1:123456789:profile/xxx",
  "clientIdHash": "abc123...",
  "clientRegistration": {
    "clientId": "xxx",
    "clientSecret": "eyJ...",
    "expiresAt": "2026-04-10T12:00:00.000Z"
  }
}
```

**IDC 账号必须保存 `clientRegistration`**，否则无法刷新 Token。

---

## 调用 Kiro API

两种认证方式调用 API 的方式相同：

```javascript
const response = await fetch(
  'https://codewhisperer.us-east-1.amazonaws.com/generateAssistantResponse',
  {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${accessToken}`
    },
    body: JSON.stringify({
      conversationState: { ... },
      profileArn: token.profileArn || ''  // IDC 有值，Social 为空
    })
  }
)
```

**唯一区别**：`profileArn` 字段
- Social: 空字符串 `""`
- IDC: 有值 `"arn:aws:codewhisperer:..."`

---

## 常见问题

### Q: refreshToken 长度过短怎么办？

refreshToken 通常长度 > 100 字符，如果过短可能被截断。检查：
- 复制时是否完整
- 配置文件格式是否正确
- 是否有换行符或空格

### Q: 刷新后 refreshToken 会变吗？

- **Social 账号**: 可能返回新的 refreshToken，需要更新
- **IDC 账号**: 通常不变，但建议检查响应并更新

### Q: Token 刷新频率限制？

- 建议提前 5 分钟刷新，避免频繁请求
- 刷新失败后不要立即重试，等待一段时间

### Q: 如何判断账号类型？

```rust
fn is_idc(account: &Account) -> bool {
    account.client_id.is_some() || 
    account.auth_method.to_lowercase() == "idc"
}
```

---

## 注意事项

1. **IDC 需要保存 clientRegistration**
   - 没有它无法刷新 Token
   - 它也有过期时间（通常 90 天）

2. **profileArn 必须正确传递**
   - Social 传空字符串
   - IDC 传实际值

3. **region 可能不同**
   - Social 固定 `us-east-1`
   - IDC 可能是其他 region

4. **错误处理**
   - Social 刷新失败 → 需要重新登录
   - IDC 刷新失败 → 可能是 clientRegistration 过期，需要重新登录

---

## 安全建议

1. **不要硬编码**: refreshToken 和 clientSecret 不要提交到代码仓库
2. **文件权限**: 配置文件应设置为仅当前用户可读（600）
3. **HTTPS 传输**: 所有 Token 刷新请求必须使用 HTTPS
4. **定期轮换**: 建议定期重新登录，获取新的 refreshToken
5. **错误日志**: 不要在日志中输出完整的 Token 内容
