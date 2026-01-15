# 附加功能

## 概述

本文档包含 KiroGate 已实现的附加功能。

**核心功能**：
- 配额检查 - 查询账号使用量和限额
- 负载均衡 - 多账号轮询分配
- 健康检查 - 定期检查账号状态
- 用户记忆 - 持久化用户偏好

---

## 1. 配额检查 (GetUsageLimits)

查询账号的配额使用情况，Social 和 IdC 账号都可用。

### API 端点

```bash
GET https://codewhisperer.us-east-1.amazonaws.com/GetUsageLimits
Authorization: Bearer YOUR_ACCESS_TOKEN
```

### 响应示例

```json
{
  "usageLimits": {
    "chatUsage": {
      "used": 150,
      "limit": 1000,
      "resetTime": "2026-01-12T00:00:00Z"
    },
    "codeCompletionUsage": {
      "used": 500,
      "limit": 5000
    }
  }
}
```

### 使用场景

```javascript
async function checkQuota(account) {
  const response = await fetch(
    'https://codewhisperer.us-east-1.amazonaws.com/GetUsageLimits',
    {
      headers: { 'Authorization': `Bearer ${account.accessToken}` }
    }
  )
  
  const data = await response.json()
  const chat = data.usageLimits?.chatUsage
  
  if (chat && chat.used >= chat.limit) {
    return { available: false, reason: '配额已用尽' }
  }
  
  return { available: true, remaining: chat.limit - chat.used }
}

// 在选择账号时检查
async function getAvailableAccount(accounts) {
  for (const account of accounts) {
    const quota = await checkQuota(account)
    if (quota.available) {
      return account
    }
  }
  return null  // 所有账号配额用尽
}
```

---

## 2. 负载均衡

多账号时的分配策略。

### 轮询 (Round Robin)

```javascript
class RoundRobinBalancer {
  constructor(accounts) {
    this.accounts = accounts.filter(a => a.enabled)
    this.index = 0
  }
  
  next() {
    const account = this.accounts[this.index]
    this.index = (this.index + 1) % this.accounts.length
    return account
  }
}
```

### 随机

```javascript
function randomSelect(accounts) {
  const enabled = accounts.filter(a => a.enabled)
  return enabled[Math.floor(Math.random() * enabled.length)]
}
```

### 权重

```javascript
function weightedSelect(accounts) {
  // 按剩余配额分配权重
  const weights = accounts.map(a => a.remainingQuota || 1)
  const total = weights.reduce((a, b) => a + b, 0)
  
  let random = Math.random() * total
  for (let i = 0; i < accounts.length; i++) {
    random -= weights[i]
    if (random <= 0) return accounts[i]
  }
  
  return accounts[0]
}
```

---

## 3. 健康检查

定期检查账号状态。

```javascript
class HealthChecker {
  constructor(accounts, interval = 60000) {
    this.accounts = accounts
    this.interval = interval
  }
  
  start() {
    this.timer = setInterval(() => this.check(), this.interval)
  }
  
  async check() {
    for (const account of this.accounts) {
      try {
        // 检查 Token 是否过期
        if (this.isTokenExpired(account)) {
          await this.refreshToken(account)
        }
        
        // 检查配额
        const quota = await checkQuota(account)
        account.remainingQuota = quota.remaining
        account.healthy = quota.available
        
      } catch (error) {
        account.healthy = false
        account.error = error.message
      }
    }
  }
  
  isTokenExpired(account) {
    return Date.now() > account.expiresAt - 5 * 60 * 1000  // 提前 5 分钟
  }
}
```

---

## 4. 请求日志

记录请求和响应，方便调试。

```javascript
function logRequest(req, account, startTime) {
  const duration = Date.now() - startTime
  
  console.log(JSON.stringify({
    timestamp: new Date().toISOString(),
    method: req.method,
    path: req.path,
    model: req.body?.model,
    accountId: account.id,
    duration,
    status: 'success'
  }))
}

function logError(req, account, error, startTime) {
  const duration = Date.now() - startTime
  
  console.error(JSON.stringify({
    timestamp: new Date().toISOString(),
    method: req.method,
    path: req.path,
    accountId: account.id,
    duration,
    status: 'error',
    error: error.message
  }))
}
```

---

## 5. 使用统计

按账号/模型统计用量。

```javascript
class UsageStats {
  constructor() {
    this.stats = {}
  }
  
  record(accountId, model, tokens) {
    const key = `${accountId}:${model}`
    if (!this.stats[key]) {
      this.stats[key] = { requests: 0, tokens: 0 }
    }
    this.stats[key].requests++
    this.stats[key].tokens += tokens
  }
  
  getStats() {
    return this.stats
  }
  
  // 定期重置
  reset() {
    this.stats = {}
  }
}
```

---

## 6. 用户记忆 (User Memory)

Kiro 提供了 User Memory API，可以存储用户偏好、对话摘要等持久化信息。

### API 端点

**创建记忆**
```bash
POST https://codewhisperer.us-east-1.amazonaws.com/CreateUserMemoryEntry
Authorization: Bearer YOUR_ACCESS_TOKEN
Content-Type: application/json

{
  "memoryEntryString": "用户偏好：使用中文回复，代码风格偏好简洁",
  "origin": "KIROGATE",
  "profileArn": ""
}
```

**列出记忆**
```bash
POST https://codewhisperer.us-east-1.amazonaws.com/ListUserMemoryEntries
Authorization: Bearer YOUR_ACCESS_TOKEN
Content-Type: application/json

{
  "origin": "KIROGATE",
  "profileArn": ""
}
```

**删除记忆**
```bash
DELETE https://codewhisperer.us-east-1.amazonaws.com/DeleteUserMemoryEntry/{entryId}
Authorization: Bearer YOUR_ACCESS_TOKEN
```

### 响应示例

```json
{
  "memoryEntries": [
    {
      "id": "mem-a1b2c3d4-e5f6-7890",
      "memoryEntryString": "用户偏好：使用中文回复",
      "origin": "KIROGATE",
      "createdAt": "2026-01-11T10:00:00Z"
    }
  ]
}
```

### 使用场景

```javascript
class UserMemory {
  constructor(account) {
    this.account = account
    this.baseUrl = 'https://codewhisperer.us-east-1.amazonaws.com'
  }
  
  // 保存记忆
  async save(content) {
    const response = await fetch(`${this.baseUrl}/CreateUserMemoryEntry`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.account.accessToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        memoryEntryString: content,
        origin: 'KIROGATE',
        profileArn: this.account.profileArn || ''
      })
    })
    return response.json()
  }
  
  // 获取所有记忆
  async list() {
    const response = await fetch(`${this.baseUrl}/ListUserMemoryEntries`, {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.account.accessToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        origin: 'KIROGATE',
        profileArn: this.account.profileArn || ''
      })
    })
    const data = await response.json()
    return data.memoryEntries || []
  }
  
  // 删除记忆
  async delete(entryId) {
    await fetch(`${this.baseUrl}/DeleteUserMemoryEntry/${entryId}`, {
      method: 'DELETE',
      headers: {
        'Authorization': `Bearer ${this.account.accessToken}`
      }
    })
  }
}

// 使用示例：在对话开始时加载用户记忆
async function loadUserContext(account) {
  const memory = new UserMemory(account)
  const entries = await memory.list()
  
  // 把记忆内容拼接到 system prompt
  const memoryContext = entries
    .map(e => e.memoryEntryString)
    .join('\n')
  
  return memoryContext
}
```

### 应用场景

- **用户偏好** - 语言、代码风格、回复长度等
- **对话摘要** - 长对话的关键信息摘要
- **项目上下文** - 项目相关的背景信息
- **自定义指令** - 用户的特殊要求

### 在 KiroGate 中集成

```javascript
// 修改请求处理流程，加入记忆
async function handleOpenAI(req, res) {
  const account = await getAccount()
  
  // 1. 加载用户记忆
  const memory = new UserMemory(account)
  const memories = await memory.list()
  const memoryContext = memories.map(m => m.memoryEntryString).join('\n')
  
  // 2. 把记忆加到 system prompt
  let systemPrompt = extractSystemPrompt(req.body)
  if (memoryContext) {
    systemPrompt = `${memoryContext}\n\n---\n\n${systemPrompt}`
  }
  
  // 3. 转换请求（带上增强的 system prompt）
  const kiroRequest = convertRequest(req.body, 'openai', { systemPrompt })
  
  // 4. 调用 Kiro API...
  const stream = await callKiroAPI(kiroRequest, account)
  
  // 5. 转换响应并返回...
}

// 保存记忆的 API（可选，让用户主动保存）
app.post('/v1/memory', async (req, res) => {
  const account = await getAccount()
  const memory = new UserMemory(account)
  
  if (req.method === 'POST') {
    await memory.save(req.body.content)
    res.json({ success: true })
  }
})

app.get('/v1/memory', async (req, res) => {
  const account = await getAccount()
  const memory = new UserMemory(account)
  const entries = await memory.list()
  res.json({ memories: entries })
})

app.delete('/v1/memory/:id', async (req, res) => {
  const account = await getAccount()
  const memory = new UserMemory(account)
  await memory.delete(req.params.id)
  res.json({ success: true })
})
```

### 自动提取记忆（高级）

让 AI 自动识别并保存重要信息：

```javascript
// 在响应完成后，检查是否有值得保存的信息
async function checkAndSaveMemory(response, account) {
  // 简单规则：如果用户说了"记住"、"以后"等关键词
  const keywords = ['记住', '以后', '偏好', '总是', '不要']
  
  if (keywords.some(k => response.includes(k))) {
    const memory = new UserMemory(account)
    // 提取关键信息保存
    await memory.save(`用户偏好: ${response.substring(0, 200)}`)
  }
}
```

---

## 7. 备用对话接口 (SendMessageStreaming)

Kiro 源码中定义了另一个对话接口，虽然目前 Kiro 自己没用，但可以作为备选。

### API 端点

```bash
POST https://codewhisperer.us-east-1.amazonaws.com/SendMessageStreaming
Authorization: Bearer YOUR_ACCESS_TOKEN
Content-Type: application/json

{
  "conversationState": { ... },  # 同 generateAssistantResponse
  "profileArn": "",
  "dryRun": false,               # 干运行模式
  "source": "AGENT"              # 来源标识
}
```

### 参数说明

**dryRun** - 干运行模式
- `false`（默认）：正常执行，调用模型
- `true`：只验证请求，不调用模型，不消耗配额

**source** - 请求来源
- `"AGENT"` - 代理模式
- `"IDE"` - IDE 模式

### 使用场景

**1. 健康检查（用 dryRun）**

```javascript
async function healthCheck(account) {
  const response = await fetch(
    'https://codewhisperer.us-east-1.amazonaws.com/SendMessageStreaming',
    {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${account.accessToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        conversationState: {
          conversationId: crypto.randomUUID(),
          currentMessage: {
            userInputMessage: {
              content: ['ping'],
              userIntent: 'CODE_GENERATION'
            }
          }
        },
        profileArn: account.profileArn || '',
        dryRun: true,  // ← 只验证，不执行
        source: 'AGENT'
      })
    }
  )
  
  return response.ok  // true = Token 有效，配额正常
}
```

**2. 预检请求**

在发送大请求前，先用 dryRun 验证：

```javascript
async function validateRequest(request, account) {
  const response = await fetch(
    'https://codewhisperer.us-east-1.amazonaws.com/SendMessageStreaming',
    {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${account.accessToken}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        ...request,
        dryRun: true  // 验证请求格式、Token、配额
      })
    }
  )
  
  if (!response.ok) {
    const error = await response.json()
    throw new Error(`请求验证失败: ${error.message}`)
  }
  
  return true
}

// 使用
async function sendWithValidation(request, account) {
  // 1. 先验证
  await validateRequest(request, account)
  
  // 2. 再真正发送
  return callKiroAPI(request, account)
}
```

### 与 generateAssistantResponse 的区别

- `generateAssistantResponse` - 主接口，Kiro 实际使用
- `SendMessageStreaming` - 备用接口，多了 `dryRun` 和 `source` 参数

**建议**：正常对话用 `generateAssistantResponse`，健康检查/预检用 `SendMessageStreaming` + `dryRun`。

---

## 8. 代码补全 (可选)

如果要支持代码补全，可以调用 `GenerateCompletions` API。

### API 端点

```bash
POST https://codewhisperer.us-east-1.amazonaws.com/GenerateCompletions
Authorization: Bearer YOUR_ACCESS_TOKEN
Content-Type: application/json

{
  "fileContext": {
    "filename": "main.py",
    "programmingLanguage": { "languageName": "python" },
    "leftFileContent": "def hello():\n    ",
    "rightFileContent": ""
  }
}
```

### 响应

```json
{
  "completions": [
    { "content": "print('Hello, World!')" },
    { "content": "return 'Hello'" }
  ]
}
```

这个功能 Claude Code 不需要，但如果要做通用的 AI 代理可以考虑。

---

## 总结

这些功能都是**可选的**：

- **配额检查** - 推荐，避免浪费请求
- **负载均衡** - 多账号时推荐
- **健康检查** - 生产环境推荐
- **请求日志** - 调试时有用
- **使用统计** - 运营时有用
- **用户记忆** - 个性化体验
- **备用对话接口** - 健康检查/预检用
- **代码补全** - 看需求
