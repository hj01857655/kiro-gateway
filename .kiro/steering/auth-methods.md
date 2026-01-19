# 认证方式

## 概述

Kiro 支持两种认证方式，KiroGate 需要同时支持：

1. **Social 登录** - 个人账号（Google/GitHub）
2. **IDC 登录** - 企业账号（AWS IAM Identity Center）

两种方式的 **Token 格式相同**，但 **刷新方式不同**。

---

## 对比

```
┌─────────────────┬────────────────────────┬────────────────────────────┐
│                 │ Social 登录             │ IDC 登录                    │
├─────────────────┼────────────────────────┼────────────────────────────┤
│ 适用用户        │ 个人用户                │ 企业用户                    │
│ 登录方式        │ Google / GitHub        │ AWS IAM Identity Center    │
│ 刷新端点        │ Kiro Auth 服务          │ AWS SSO OIDC               │
│ 需要 clientId   │ ❌ 不需要               │ ✅ 需要                     │
│ profileArn      │ 空字符串                │ 有值                       │
└─────────────────┴────────────────────────┴────────────────────────────┘
```

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

## Token 刷新

### Social 刷新

```javascript
// 端点: https://prod.us-east-1.auth.desktop.kiro.dev/refreshToken

async function refreshSocialToken(refreshToken) {
  const response = await fetch(
    'https://prod.us-east-1.auth.desktop.kiro.dev/refreshToken',
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ refreshToken })
    }
  )
  
  if (!response.ok) {
    throw new Error(`刷新失败: ${response.status}`)
  }
  
  return await response.json()
  // { accessToken: "...", expiresIn: 3600 }
}
```

### IDC 刷新

```javascript
// 端点: https://oidc.{region}.amazonaws.com/token

async function refreshIdcToken(token, clientRegistration) {
  const { refreshToken, region } = token
  const { clientId, clientSecret } = clientRegistration
  
  const response = await fetch(
    `https://oidc.${region}.amazonaws.com/token`,
    {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        clientId,
        clientSecret,
        grantType: 'refresh_token',
        refreshToken
      })
    }
  )
  
  if (!response.ok) {
    throw new Error(`刷新失败: ${response.status}`)
  }
  
  return await response.json()
  // { accessToken: "...", refreshToken: "...", expiresIn: 3600 }
}
```

**IDC 刷新需要额外信息**：
- `clientId` 和 `clientSecret`（从客户端注册缓存获取）
- 缓存位置: `~/.aws/sso/cache/{clientIdHash}.json`

---

## 统一刷新逻辑

```javascript
class TokenRefresher {
  constructor() {
    // IDC 客户端注册缓存
    this.clientRegistrations = new Map()
  }
  
  // 加载 IDC 客户端注册信息
  loadClientRegistration(clientIdHash) {
    if (this.clientRegistrations.has(clientIdHash)) {
      return this.clientRegistrations.get(clientIdHash)
    }
    
    // 从文件加载
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
  
  // 统一刷新方法
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
    
    // 检查客户端注册是否过期
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
}
```

---

## 调用 API 的区别

**两种认证方式调用 Kiro API 的方式相同**：

```javascript
// 都是 Bearer Token
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

**IDC 账号需要额外保存 `clientRegistration`**，否则无法刷新 Token。

---

## 获取 Token 的方法

### 凭证文件位置

Kiro 凭证存储在固定位置（Kiro 源码确认）：

```
~/.aws/sso/cache/kiro-auth-token.json      # 主凭证文件
~/.aws/sso/cache/{clientIdHash}.json       # IDC 客户端注册信息
```

### 方法 1: 从 Kiro 缓存提取

Token 存储位置: `~/.aws/sso/cache/kiro-auth-token.json`

```javascript
const fs = require('fs')
const path = require('path')
const os = require('os')

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

### 方法 2: 手动登录

参考：
- [Social 登录流程](../kiro-api/auth/oauth-social.md)
- [IDC 登录流程](../kiro-api/auth/oauth-idc.md)

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
