# 账号管理

## 概述

KiroGate 支持管理多个 Kiro 账号，实现：

- Token 自动刷新
- 多账号负载均衡
- 配额监控
- 故障转移

---

## Token 生命周期

```
┌─────────────────────────────────────────────────────────────┐
│                    Token 生命周期                            │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────┐    1小时后     ┌─────────┐    刷新成功         │
│  │ 有效    │ ───────────→  │ 即将过期 │ ───────────→ 有效   │
│  │ Token   │               │ (提前5分) │                    │
│  └─────────┘               └────┬─────┘                    │
│                                 │                          │
│                                 │ 刷新失败                  │
│                                 ↓                          │
│                           ┌─────────┐                      │
│                           │ 已过期   │ → 需要重新登录       │
│                           └─────────┘                      │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Token 结构

```javascript
{
  accessToken: "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  refreshToken: "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  
  // 过期时间（两种格式都要支持）
  expiresAt: 1704067200000,              // 时间戳格式（毫秒）
  expire: "2024-01-01T00:00:00Z",        // ISO 8601 / RFC 3339 格式
  
  refreshExpiresAt: null                 // refreshToken 过期时间（通常很长）
}
```

### 过期时间检测

⚠️ **重要**：Kiro 凭证文件可能使用两种不同的过期时间格式，实现时必须都支持。

```javascript
function isTokenExpired(account) {
  const BUFFER = 5 * 60 * 1000  // 提前 5 分钟判断为过期
  const now = Date.now()
  
  // 优先检查 ISO 8601 格式（新格式，字段名 expire）
  if (account.expire) {
    const expiresAt = new Date(account.expire).getTime()
    if (!isNaN(expiresAt)) {
      return now >= expiresAt - BUFFER
    }
  }
  
  // 兼容时间戳格式（旧格式，字段名 expiresAt）
  if (account.expiresAt) {
    // 可能是毫秒或秒
    let expiresAt = account.expiresAt
    if (expiresAt < 10000000000) {
      expiresAt *= 1000  // 秒转毫秒
    }
    return now >= expiresAt - BUFFER
  }
  
  // 没有过期时间信息，保守地认为需要刷新
  return true
}
```

### refreshToken 截断检测

⚠️ **重要**：Kiro IDE 可能会截断 refreshToken 以防止第三方工具使用。

```javascript
function validateRefreshToken(refreshToken) {
  if (!refreshToken || refreshToken.trim() === '') {
    throw new Error('refreshToken 为空')
  }
  
  // 正常的 refreshToken 长度应该在 500+ 字符
  // 被截断的 token 通常 < 100 字符
  if (refreshToken.length < 100) {
    throw new Error(
      `refreshToken 可能被截断（长度: ${refreshToken.length}）\n` +
      '解决方案：使用其他工具获取完整凭证'
    )
  }
  
  // 检查是否包含截断标记
  if (refreshToken.includes('...') || refreshToken.endsWith('...')) {
    throw new Error('refreshToken 已被截断')
  }
  
  return true
}
```

### 刷新逻辑

KiroGate 需要支持两种账号类型的刷新：

```javascript
class TokenManager {
  constructor() {
    this.REFRESH_BUFFER = 5 * 60 * 1000  // 提前 5 分钟刷新
  }
  
  // 检查是否需要刷新
  needsRefresh(account) {
    if (!account.expiresAt) return false
    return Date.now() >= account.expiresAt - this.REFRESH_BUFFER
  }
  
  // 统一刷新入口
  async refreshToken(account) {
    // 先验证 refreshToken
    validateRefreshToken(account.refreshToken)
    
    if (account.authMethod === 'IdC') {
      return this.refreshIdcToken(account)
    } else {
      return this.refreshSocialToken(account)
    }
  }
  
  // Social 账号刷新
  async refreshSocialToken(account) {
    const region = account.region || 'us-east-1'
    const machineId = this.getMachineId(account)
    const kiroVersion = '0.1.25'
    
    const response = await fetch(
      `https://prod.${region}.auth.desktop.kiro.dev/refreshToken`,
      {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'User-Agent': `KiroIDE-${kiroVersion}-${machineId}`,
          'Accept': 'application/json, text/plain, */*',
          'Accept-Encoding': 'br, gzip, deflate',
          'Accept-Language': '*',
          'Sec-Fetch-Mode': 'cors',
          'Connection': 'close'
        },
        body: JSON.stringify({ refreshToken: account.refreshToken })
      }
    )
    
    if (!response.ok) {
      if (response.status === 401) {
        account.status = 'expired'
        throw new Error('RefreshToken 已过期，需要重新登录')
      }
      throw new Error(`Social 刷新失败: ${response.status}`)
    }
    
    // Social 返回 camelCase
    const data = await response.json()
    
    account.accessToken = data.accessToken
    account.refreshToken = data.refreshToken || account.refreshToken
    account.expiresAt = Date.now() + (data.expiresIn || 3600) * 1000
    account.status = 'active'
    
    return account
  }
  
  // IDC 账号刷新
  async refreshIdcToken(account) {
    // IDC 需要 clientId 和 clientSecret
    if (!account.clientId || !account.clientSecret) {
      throw new Error('IDC 账号缺少 clientId/clientSecret，无法刷新')
    }
    
    const region = account.region || 'us-east-1'
    
    const response = await fetch(
      `https://oidc.${region}.amazonaws.com/token`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          clientId: account.clientId,
          clientSecret: account.clientSecret,
          grantType: 'refresh_token',
          refreshToken: account.refreshToken
        })
      }
    )
    
    if (!response.ok) {
      if (response.status === 401) {
        account.status = 'expired'
        throw new Error('IDC Token 已过期，需要重新登录')
      }
      throw new Error(`IDC 刷新失败: ${response.status}`)
    }
    
    const data = await response.json()
    
    // IDC 返回 snake_case
    account.accessToken = data.access_token
    account.refreshToken = data.refresh_token || account.refreshToken
    account.expiresAt = Date.now() + (data.expires_in || 3600) * 1000
    account.status = 'active'
    
    return account
  }
  
  // 获取有效 Token（自动刷新）
  async getValidToken(account) {
    if (this.needsRefresh(account)) {
      await this.refreshToken(account)
    }
    return account.accessToken
  }
}
```

---

## 多账号管理

### 账号结构

#### Social 账号

```javascript
{
  id: "social-1",
  name: "Google 账号",
  authMethod: "social",      // 认证方式
  provider: "Google",        // Google | Github
  
  accessToken: "eyJ...",
  refreshToken: "eyJ...",
  expiresAt: 1704067200000,
  profileArn: "",            // Social 账号为空
  
  status: "active",          // active | expired | error | disabled
  lastError: null,
  
  // 统计信息
  stats: {
    totalRequests: 1000,
    todayRequests: 50,
    lastUsed: 1704060000000
  },
  
  // 配置
  config: {
    enabled: true,
    priority: 1,
    maxRequestsPerDay: 0
  }
}
```

#### IDC 账号

```javascript
{
  id: "idc-1",
  name: "企业账号",
  authMethod: "IdC",         // 认证方式
  provider: "BuilderId",     // BuilderId | Enterprise | Internal
  region: "us-east-1",
  
  accessToken: "eyJ...",
  refreshToken: "eyJ...",
  expiresAt: 1704067200000,
  profileArn: "arn:aws:codewhisperer:us-east-1:123456789:profile/xxx",
  
  // IDC 特有字段（刷新 Token 必需）
  clientId: "xxx",
  clientSecret: "eyJ...",
  clientIdHash: "abc123...",
  
  status: "active",
  lastError: null,
  
  stats: {
    totalRequests: 500,
    todayRequests: 20,
    lastUsed: 1704060000000
  },
  
  config: {
    enabled: true,
    priority: 2,
    maxRequestsPerDay: 0
  }
}
```

### 账号选择策略

```javascript
class AccountSelector {
  constructor(accounts) {
    this.accounts = accounts
  }
  
  // 获取可用账号列表
  getAvailableAccounts() {
    return this.accounts.filter(acc => 
      acc.config.enabled && 
      acc.status === 'active' &&
      !this.isQuotaExceeded(acc)
    )
  }
  
  // 检查配额
  isQuotaExceeded(account) {
    if (account.config.maxRequestsPerDay === 0) return false
    return account.stats.todayRequests >= account.config.maxRequestsPerDay
  }
  
  // 轮询选择
  selectRoundRobin() {
    const available = this.getAvailableAccounts()
    if (available.length === 0) return null
    
    // 按优先级排序，然后选择使用次数最少的
    available.sort((a, b) => {
      if (a.config.priority !== b.config.priority) {
        return a.config.priority - b.config.priority
      }
      return a.stats.todayRequests - b.stats.todayRequests
    })
    
    return available[0]
  }
  
  // 随机选择
  selectRandom() {
    const available = this.getAvailableAccounts()
    if (available.length === 0) return null
    return available[Math.floor(Math.random() * available.length)]
  }
  
  // 最少使用选择
  selectLeastUsed() {
    const available = this.getAvailableAccounts()
    if (available.length === 0) return null
    return available.reduce((min, acc) => 
      acc.stats.todayRequests < min.stats.todayRequests ? acc : min
    )
  }
}
```

---

## 获取 Kiro Token

### 凭证来源

KiroGate 支持三种凭证来源：

1. **Kiro IDE 缓存文件**（推荐）- `~/.aws/sso/cache/kiro-auth-token.json`
2. **kiro-cli SQLite 数据库** - `~/.local/share/kiro-cli/data.sqlite3`
3. **开发者工具提取** - 从 Network 面板复制

---

### 方法 1: 从 Kiro IDE 缓存提取（推荐）

Kiro IDE 将凭证存储在 AWS SSO 缓存目录：

```
~/.aws/sso/cache/kiro-auth-token.json      # 主凭证文件
~/.aws/sso/cache/{clientIdHash}.json       # IDC 客户端注册信息
```

```javascript
const fs = require('fs')
const path = require('path')
const os = require('os')

function getKiroToken() {
  const cacheDir = path.join(os.homedir(), '.aws', 'sso', 'cache')
  const tokenPath = path.join(cacheDir, 'kiro-auth-token.json')
  
  const token = JSON.parse(fs.readFileSync(tokenPath, 'utf-8'))
  
  // 验证 refreshToken 未被截断
  if (token.refreshToken && token.refreshToken.length < 100) {
    console.warn('警告: refreshToken 可能被截断')
  }
  
  // 如果是 IDC 账号，还需要读取客户端注册信息
  if (token.authMethod === 'IdC' && token.clientIdHash) {
    const clientRegPath = path.join(cacheDir, `${token.clientIdHash}.json`)
    
    if (fs.existsSync(clientRegPath)) {
      const clientReg = JSON.parse(fs.readFileSync(clientRegPath, 'utf-8'))
      token.clientId = clientReg.clientId
      token.clientSecret = clientReg.clientSecret
    } else {
      console.warn('警告: 找不到 IDC 客户端注册文件，无法刷新 Token')
    }
  }
  
  return token
}
```

---

### 方法 2: 从 kiro-cli SQLite 数据库提取

[kiro-cli](https://github.com/pchaganti/gx-kiro-cli) 是一个第三方命令行工具，通过 AWS SSO OIDC 认证。

**数据库位置**：
```
~/.local/share/kiro-cli/data.sqlite3
```

**表结构**：`auth_kv` 表，key-value 存储

```javascript
const sqlite3 = require('better-sqlite3')
const path = require('path')
const os = require('os')

function getKiroCliToken() {
  const dbPath = path.join(os.homedir(), '.local/share/kiro-cli/data.sqlite3')
  const db = sqlite3(dbPath, { readonly: true })
  
  // 读取 token 数据（支持两种 key 格式）
  let tokenRow = db.prepare("SELECT value FROM auth_kv WHERE key = ?")
    .get('kirocli:odic:token')
  if (!tokenRow) {
    tokenRow = db.prepare("SELECT value FROM auth_kv WHERE key = ?")
      .get('codewhisperer:odic:token')
  }
  
  if (!tokenRow) {
    throw new Error('kiro-cli 凭证未找到，请先运行 kiro-cli login')
  }
  
  const tokenData = JSON.parse(tokenRow.value)
  
  // 读取设备注册信息（client_id, client_secret）
  let regRow = db.prepare("SELECT value FROM auth_kv WHERE key = ?")
    .get('kirocli:odic:device-registration')
  if (!regRow) {
    regRow = db.prepare("SELECT value FROM auth_kv WHERE key = ?")
      .get('codewhisperer:odic:device-registration')
  }
  
  if (regRow) {
    const regData = JSON.parse(regRow.value)
    tokenData.clientId = regData.client_id
    tokenData.clientSecret = regData.client_secret
  }
  
  db.close()
  
  return {
    accessToken: tokenData.access_token,
    refreshToken: tokenData.refresh_token,
    expiresAt: tokenData.expires_at,  // ISO 8601 格式
    region: tokenData.region,
    clientId: tokenData.clientId,
    clientSecret: tokenData.clientSecret,
    authMethod: 'IdC'  // kiro-cli 使用 AWS SSO OIDC
  }
}
```

**kiro-cli Token 刷新**：

kiro-cli 使用 AWS SSO OIDC 端点：

```javascript
async function refreshKiroCliToken(account) {
  const region = account.region || 'us-east-1'
  
  const response = await fetch(
    `https://oidc.${region}.amazonaws.com/token`,
    {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        clientId: account.clientId,
        clientSecret: account.clientSecret,
        grantType: 'refresh_token',
        refreshToken: account.refreshToken
      })
    }
  )
  
  if (!response.ok) {
    throw new Error(`kiro-cli Token 刷新失败: ${response.status}`)
  }
  
  const data = await response.json()
  
  // AWS SSO OIDC 返回 camelCase
  return {
    accessToken: data.accessToken,
    refreshToken: data.refreshToken || account.refreshToken,
    expiresAt: Date.now() + (data.expiresIn || 3600) * 1000
  }
}
```

⚠️ **注意**：kiro-cli 可能在内存中刷新 Token 而不持久化到 SQLite，导致数据库中的 refreshToken 过期。如果刷新失败，需要重新运行 `kiro-cli login`。

---

### 方法 3: 从开发者工具提取

如果无法直接读取凭证文件：

1. 打开 Kiro IDE
2. 登录账号
3. 打开开发者工具 (F12)
4. 在 Network 面板找到 `refreshToken` 请求
5. 复制 `accessToken` 和 `refreshToken`

### 方法 4: OAuth 流程（复杂）

完整的 OAuth 登录流程，参考 [Kiro 认证文档](../kiro-api/auth/oauth-social.md)

---

## 账号管理 API

### 添加账号

```
POST /admin/accounts
Authorization: Bearer {admin-api-key}

{
  "name": "账号名称",
  "accessToken": "eyJ...",
  "refreshToken": "eyJ..."
}
```

### 列出账号

```
GET /admin/accounts
Authorization: Bearer {admin-api-key}

Response:
{
  "accounts": [
    {
      "id": "account-1",
      "name": "主账号",
      "status": "active",
      "stats": {
        "totalRequests": 1000,
        "todayRequests": 50
      }
    }
  ]
}
```

### 更新账号

```
PUT /admin/accounts/{id}
Authorization: Bearer {admin-api-key}

{
  "enabled": true,
  "priority": 1
}
```

### 删除账号

```
DELETE /admin/accounts/{id}
Authorization: Bearer {admin-api-key}
```

### 刷新账号 Token

```
POST /admin/accounts/{id}/refresh
Authorization: Bearer {admin-api-key}
```

---

## 故障处理

### 自动故障转移

```javascript
class AccountManager {
  async executeWithFallback(fn) {
    const accounts = this.selector.getAvailableAccounts()
    
    for (const account of accounts) {
      try {
        // 确保 Token 有效
        await this.tokenManager.getValidToken(account)
        
        // 执行请求
        const result = await fn(account)
        
        // 更新统计
        account.stats.totalRequests++
        account.stats.todayRequests++
        account.stats.lastUsed = Date.now()
        
        return result
      } catch (error) {
        // 记录错误
        account.lastError = error.message
        
        // 特定错误标记账号状态
        if (error.status === 401) {
          account.status = 'expired'
        } else if (error.status === 429) {
          // 限流，暂时跳过这个账号
          account.stats.rateLimitedUntil = Date.now() + 60000
        }
        
        // 继续尝试下一个账号
        continue
      }
    }
    
    throw new Error('所有账号都不可用')
  }
}
```

### 健康检查

```javascript
class HealthChecker {
  constructor(accountManager) {
    this.accountManager = accountManager
    this.checkInterval = 5 * 60 * 1000  // 5 分钟检查一次
  }
  
  start() {
    setInterval(() => this.checkAll(), this.checkInterval)
  }
  
  async checkAll() {
    for (const account of this.accountManager.accounts) {
      await this.checkAccount(account)
    }
  }
  
  async checkAccount(account) {
    try {
      // 尝试刷新 Token
      if (this.accountManager.tokenManager.needsRefresh(account)) {
        await this.accountManager.tokenManager.refreshToken(account)
      }
      
      account.status = 'active'
      account.lastCheck = Date.now()
    } catch (error) {
      account.status = 'error'
      account.lastError = error.message
      account.lastCheck = Date.now()
    }
  }
}
```

---

## 统计和监控

### 每日统计重置

```javascript
// 每天 0 点重置统计
function scheduleDailyReset(accountManager) {
  const now = new Date()
  const tomorrow = new Date(now)
  tomorrow.setDate(tomorrow.getDate() + 1)
  tomorrow.setHours(0, 0, 0, 0)
  
  const msUntilMidnight = tomorrow - now
  
  setTimeout(() => {
    // 重置所有账号的每日统计
    for (const account of accountManager.accounts) {
      account.stats.todayRequests = 0
    }
    
    // 设置下一次重置
    scheduleDailyReset(accountManager)
  }, msUntilMidnight)
}
```

### 监控指标

```javascript
{
  // 全局统计
  global: {
    totalRequests: 10000,
    todayRequests: 500,
    activeAccounts: 3,
    errorAccounts: 1
  },
  
  // 每个账号统计
  accounts: [
    {
      id: "account-1",
      status: "active",
      totalRequests: 5000,
      todayRequests: 250,
      avgResponseTime: 1200,  // ms
      errorRate: 0.01         // 1%
    }
  ]
}
```
