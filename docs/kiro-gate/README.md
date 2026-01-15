# KiroGate

## 概述

KiroGate 是一个 API 网关，将 Kiro 的 AI 能力暴露为 **OpenAI / Anthropic 兼容格式**，让你可以在 Claude Code、Cursor、Continue 等工具中使用 Kiro 账号的 Claude 额度。

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Claude Code   │     │    KiroGate     │     │    Kiro API     │
│   Cursor        │ ──→ │  (API 网关)      │ ──→ │  (AWS 后端)     │
│   Continue      │     │                 │     │                 │
│   其他客户端     │     │  格式转换        │     │  Claude 模型    │
└─────────────────┘     │  Token 管理      │     └─────────────────┘
                        └─────────────────┘
```

## 核心功能

1. **格式转换** - OpenAI/Anthropic 格式 ↔ Kiro 格式
   - 请求转换：messages、tools、system prompt
   - 响应转换：AWS Event Stream → SSE
   - 工具调用：tool_calls ↔ toolUseEvent

2. **Token 管理** - 自动刷新过期的 accessToken
   - 提前 5 分钟检测过期
   - Social / IDC 两种刷新方式
   - refreshToken 截断检测（<100 字符警告）

3. **多账号支持** - 管理多个 Kiro 账号
   - 账号池轮询
   - 自动跳过失效账号
   - 配额监控

4. **流式响应** - 支持 SSE 流式输出
   - AWS Event Stream 二进制解析
   - 实时转换为 OpenAI/Anthropic SSE
   - thinking block 支持（Anthropic 格式）

5. **模型选择** - 支持多种 Claude 模型
   - `auto` / `claude-haiku-4.5` / `claude-sonnet-4` / `claude-sonnet-4.5` / `claude-opus-4.5`
   - 自动映射 OpenAI/Anthropic 模型名

6. **错误处理** - AWS SDK 标准重试策略
   - 限流自动重试（指数退避）
   - Token 过期自动刷新
   - First Token Timeout 检测

---

## 支持的 API 格式

### OpenAI 兼容

```
POST /v1/chat/completions
Authorization: Bearer {api-key}

{
  "model": "kiro",
  "messages": [
    {"role": "user", "content": "Hello"}
  ],
  "stream": true
}
```

### Anthropic 兼容

```
POST /v1/messages
x-api-key: {api-key}
anthropic-version: 2023-06-01

{
  "model": "claude-3-5-sonnet",
  "max_tokens": 4096,
  "messages": [
    {"role": "user", "content": "Hello"}
  ],
  "stream": true
}
```

---

## 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                        KiroGate                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────┐ │
│  │ OpenAI API  │  │ Anthropic   │  │    管理 API         │ │
│  │ /v1/chat/   │  │ /v1/messages│  │ /admin/accounts     │ │
│  │ completions │  │             │  │ /admin/stats        │ │
│  └──────┬──────┘  └──────┬──────┘  └──────────┬──────────┘ │
│         │                │                     │            │
│         └────────┬───────┘                     │            │
│                  ↓                             │            │
│  ┌───────────────────────────┐                 │            │
│  │      格式转换层            │                 │            │
│  │  OpenAI/Anthropic → Kiro  │                 │            │
│  └─────────────┬─────────────┘                 │            │
│                ↓                               │            │
│  ┌───────────────────────────┐                 │            │
│  │      账号管理器            │←────────────────┘            │
│  │  - Token 自动刷新          │                             │
│  │  - 多账号轮询              │                             │
│  │  - 配额监控                │                             │
│  └─────────────┬─────────────┘                             │
│                ↓                                            │
│  ┌───────────────────────────┐                             │
│  │      Kiro API 客户端       │                             │
│  │  generateAssistantResponse │                             │
│  └───────────────────────────┘                             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

## 数据流

### 请求流程

```
1. 客户端发送 OpenAI/Anthropic 格式请求
2. KiroGate 验证 API Key
3. 选择可用的 Kiro 账号（Token 有效、配额充足）
4. 转换请求格式为 Kiro 格式
5. 调用 Kiro API
6. 转换响应格式为 OpenAI/Anthropic 格式
7. 返回给客户端
```

### Token 刷新流程

```
1. 请求前检查 accessToken 是否即将过期（提前 5 分钟）
2. 过期则用 refreshToken 刷新
3. 更新存储的 Token
4. 刷新失败则标记账号为不可用
```

---

## 配置

### 环境变量

```bash
# 服务配置
PORT=8080
API_KEY=your-kirogate-api-key  # 客户端访问 KiroGate 的密钥

# 数据库（可选，用于持久化账号）
DATABASE_URL=sqlite://kirogate.db

# 日志
LOG_LEVEL=info
```

### 账号配置

```json
{
  "accounts": [
    {
      "id": "account-1",
      "accessToken": "eyJ...",
      "refreshToken": "eyJ...",
      "expiresAt": 1704067200000,
      "enabled": true
    }
  ]
}
```

---

## 客户端配置示例

### Claude Code

```bash
# 设置 API 端点
export ANTHROPIC_BASE_URL=http://localhost:8080
export ANTHROPIC_API_KEY=your-kirogate-api-key
```

### Cursor

```json
// settings.json
{
  "openai.apiKey": "your-kirogate-api-key",
  "openai.baseUrl": "http://localhost:8080/v1"
}
```

### Continue

```json
// config.json
{
  "models": [
    {
      "title": "Kiro Claude",
      "provider": "openai",
      "model": "kiro",
      "apiBase": "http://localhost:8080/v1",
      "apiKey": "your-kirogate-api-key"
    }
  ]
}
```

---

## 认证方式

KiroGate 支持两种 Kiro 认证方式：

**Social 登录（个人账号）**
- Google / GitHub 登录
- Token 刷新: `https://prod.us-east-1.auth.desktop.kiro.dev/refreshToken`
- `profileArn` 为空

**IDC 登录（企业账号）**
- AWS IAM Identity Center
- Token 刷新: `https://oidc.{region}.amazonaws.com/token`
- 需要 `clientId` + `clientSecret`
- `profileArn` 有值

**凭证文件位置**（Kiro 源码确认）：
```
~/.aws/sso/cache/kiro-auth-token.json      # 主凭证文件
~/.aws/sso/cache/{clientIdHash}.json       # IDC 客户端注册信息
```

详见 [认证方式](./auth-methods.md)

---

## Kiro API 请求头

调用 Kiro API 时需要以下请求头（Kiro 源码确认）：

```javascript
{
  'Authorization': `Bearer ${accessToken}`,
  'Content-Type': 'application/json',
  
  // AWS SDK 标准头
  'amz-sdk-invocation-id': crypto.randomUUID(),
  'amz-sdk-request': 'attempt=1; max=3',
  
  // Kiro 特定头
  'x-amzn-kiro-agent-mode': 'vibe',
  'x-amz-user-agent': `KiroIDE-${version}-${machineId}`
}
```

**User-Agent 格式**：`KiroIDE-{版本}-{机器ID}`
- 版本：Kiro IDE 版本号（如 `0.1.25`）
- 机器ID：系统硬件 ID 的 SHA256 哈希（64 字符）

详见 [实现指南](./implementation-guide.md#附录请求头详解)

---

## 相关文档

### 🚀 快速开始
- [实现指南](./implementation-guide.md) - **完整流程图 + 最简代码实现**

### 核心文档
- [Kiro API 规范](./kiro-api.md) - **官方源码参考 + API 详解**
- [Kiro Chat API](./chat-api.md) - Kiro 原生对话 API 详解
- [认证方式](./auth-methods.md) - Social 和 IDC 两种认证的区别
- [格式转换](./format-conversion.md) - OpenAI/Anthropic ↔ Kiro 请求格式转换

### 实现细节
- [SSE 事件类型](./sse-events.md) - Kiro SSE 响应事件转换
- [模型映射](./model-mapping.md) - 模型名称映射和超时配置
- [错误处理](./error-handling.md) - 错误转换和重试策略
- [高级格式转换](./advanced-conversion.md) - 工具调用、图片、System Prompt 转换

### 运维相关
- [账号管理](./account-management.md) - 多账号管理和 Token 刷新
- [部署指南](./deployment.md) - 部署和运维
