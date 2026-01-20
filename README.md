# Kiro Gateway

[![GitHub release](https://img.shields.io/github/v/release/hj01857655/kiro-gateway)](https://github.com/hj01857655/kiro-gateway/releases/latest)
[![License](https://img.shields.io/github/license/hj01857655/kiro-gateway)](https://github.com/hj01857655/kiro-gateway/blob/main/LICENSE)
[![GitHub stars](https://img.shields.io/github/stars/hj01857655/kiro-gateway)](https://github.com/hj01857655/kiro-gateway/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/hj01857655/kiro-gateway)](https://github.com/hj01857655/kiro-gateway/network/members)
[![GitHub issues](https://img.shields.io/github/issues/hj01857655/kiro-gateway)](https://github.com/hj01857655/kiro-gateway/issues)
[![GitHub pull requests](https://img.shields.io/github/issues-pr/hj01857655/kiro-gateway)](https://github.com/hj01857655/kiro-gateway/pulls)
[![GitHub last commit](https://img.shields.io/github/last-commit/hj01857655/kiro-gateway)](https://github.com/hj01857655/kiro-gateway/commits/main)
[![GitHub downloads](https://img.shields.io/github/downloads/hj01857655/kiro-gateway/total)](https://github.com/hj01857655/kiro-gateway/releases)
[![QQ Group](https://img.shields.io/badge/QQ%20Group-1081058179-blue)](https://qm.qq.com/q/oQbUA0cxO2)

> 🚀 Kiro API 网关 - 提供 OpenAI/Anthropic 兼容接口的桌面应用

Kiro Gateway 是一个基于 Tauri 2.0 的桌面应用，将 Kiro API 转换为标准的 OpenAI Chat Completions API 和 Anthropic Messages API，支持多账号管理、自动 Token 刷新、流式响应等功能。

<!-- 
## 📸 应用截图

![主界面](docs/screenshots/main.png)
![账号管理](docs/screenshots/accounts.png)
![统计监控](docs/screenshots/metrics.png)
-->

## ✨ 核心特性

| 特性 | 说明 |
|------|------|
| 🔄 **API 兼容** | 完全兼容 OpenAI Chat Completions 和 Anthropic Messages API |
| 👥 **多账号管理** | 支持 Social 和 IDC 账号，自动轮询和故障转移 |
| 🔐 **安全加密** | AES-256-GCM 加密存储敏感数据，机器特定密钥保护 |
| 🔄 **自动刷新** | Token 自动检测过期并刷新，无需手动维护 |
| 📊 **实时监控** | 请求统计、延迟分析、成功率追踪 |
| 🖥️ **桌面界面** | 友好的管理界面，支持深色/浅色主题 |
| 🛠️ **工具调用** | 完整支持工具调用和图片上传 |
| 💭 **Thinking 解析** | 与 Kiro IDE 一致的 thinking block 解析 |
| 🎯 **动态模型** | 从 Kiro API 动态加载可用模型列表 |

## 📥 下载

**最新版本**：[GitHub Releases](https://github.com/hj01857655/kiro-gateway/releases/latest)

| 平台 | 下载链接 |
|------|---------|
| 🪟 **Windows** | [kiro-gateway_x64-setup.exe](https://github.com/hj01857655/kiro-gateway/releases/latest/download/kiro-gateway_x64-setup.exe) |
| 🍎 **macOS (Intel)** | [kiro-gateway_x64.dmg](https://github.com/hj01857655/kiro-gateway/releases/latest/download/kiro-gateway_x64.dmg) |
| 🍎 **macOS (Apple Silicon)** | [kiro-gateway_aarch64.dmg](https://github.com/hj01857655/kiro-gateway/releases/latest/download/kiro-gateway_aarch64.dmg) |
| 🐧 **Linux (AppImage)** | [kiro-gateway_amd64.AppImage](https://github.com/hj01857655/kiro-gateway/releases/latest/download/kiro-gateway_amd64.AppImage) |
| 🐧 **Linux (deb)** | [kiro-gateway_amd64.deb](https://github.com/hj01857655/kiro-gateway/releases/latest/download/kiro-gateway_amd64.deb) |

## 💬 交流群

**QQ 群**：[1081058179（Kiro GateWay交流群）](https://qm.qq.com/q/oQbUA0cxO2)

## 🚀 快速开始

### 1. 下载安装

从 [Releases](https://github.com/hj01857655/kiro-gateway/releases/latest) 下载对应平台的安装包并安装。

### 2. 添加账号

打开应用后，在"账号管理"页面添加 Kiro 账号：

- **从 Kiro IDE 导入**：自动读取 `~/.aws/sso/cache/kiro-auth-token.json`
- **手动添加 Social 账号**：
  - 必要字段：Refresh Token
- **手动添加 IDC 账号**：
  - 必要字段：Client ID、Client Secret、Refresh Token
- **批量导入**：支持 JSON 文件导入

### 3. 配置客户端

在"设置"页面生成配置文件：
- **Claude Desktop**：一键生成并应用配置
- **Claude CLI**：复制配置到 `~/.config/claude/config.json`
- **OpenAI 兼容**：使用 `http://127.0.0.1:8080` 作为 Base URL

### 4. 开始使用

配置完成后，即可通过 Claude Desktop、Claude CLI 或其他 OpenAI 兼容客户端使用。

## 📖 使用文档

### API 端点

### 安装依赖

```bash
# 安装前端依赖
npm install

# Rust 依赖会在构建时自动安装
```

### 开发模式

```bash
# 启动开发模式（热重载）
npm run tauri:dev
```

### 构建应用

```bash
# 构建桌面应用
npm run tauri:build

# 构建产物位于 src-tauri/target/release/
```

### 其他命令

```bash
# 仅启动前端开发服务器
npm run dev

# 仅构建前端
npm run build

# 代码检查
cd src-tauri && cargo clippy

# 代码格式化
cd src-tauri && cargo fmt
```

## 配置说明

### 数据存储

所有配置文件和敏感数据统一存储在用户数据目录：

- **Windows**: `%APPDATA%\com.kiro.gateway\`
- **macOS**: `~/Library/Application Support/com.kiro.gateway/`
- **Linux**: `~/.local/share/com.kiro.gateway/`

**配置文件**：
- `accounts.json` - 账号配置（敏感字段已加密）
- `api_keys.json` - API Key 配置
- `metrics.json` - 统计数据
- `.encryption_key` - 加密密钥（自动生成，机器特定）
- `.admin_token` - Admin Token（自动生成，64位随机字符串）

### 安全特性

**账号数据加密**：
- 使用 AES-256-GCM 加密 `refreshToken`、`accessToken`、`clientSecret`
- 主密钥由机器特定信息（hostname + username）派生
- 加密密钥文件权限设置为仅当前用户可读（Unix 系统）
- 自动检测并解密已加密数据
- 明文数据自动迁移到加密格式

**Admin API 认证**：
- 首次启动自动生成 64 位随机 Admin Token
- 所有 `/admin/*` 路由需要认证
- 支持两种认证方式：
  - `x-admin-token: {token}` 请求头
  - `Authorization: Bearer {token}` 请求头
- 通过 `GET /admin/token` 获取当前 Admin Token

### 环境变量

在 `src-tauri/.env` 或系统环境变量中配置：

```bash
# HTTP 服务监听地址（默认 127.0.0.1）
HOST=127.0.0.1

# HTTP 服务监听端口（默认 8080）
PORT=8080
```

### 账号配置

**方式 1：从 Kiro IDE 导入**（推荐）
- 使用桌面应用的"导入账号"功能
- 自动从 `~/.aws/sso/cache/kiro-auth-token.json` 读取

**方式 2：手动配置**

在用户数据目录创建 `accounts.json`：

**Social 账号示例**：
```json
[
  {
    "authMethod": "social",
    "refreshToken": "eyJ..."
  }
]
```

**IDC 账号示例**：
```json
[
  {
    "authMethod": "IdC",
    "clientId": "MkAG97...",
    "clientSecret": "eyJraWQ...",
    "refreshToken": "aorAAAAA..."
  }
]
```

**完整字段示例**（包含可选字段）：
```json
[
  {
    "id": "account-1",
    "name": "我的 Social 账号",
    "authMethod": "social",
    "provider": "Google",
    "refreshToken": "eyJ...",
    "profileArn": "",
    "region": "us-east-1",
    "enabled": true,
    "status": "active"
  },
  {
    "id": "account-2",
    "name": "我的 IDC 账号",
    "authMethod": "IdC",
    "provider": "BuilderId",
    "clientId": "MkAG97...",
    "clientSecret": "eyJraWQ...",
    "refreshToken": "aorAAAAA...",
    "profileArn": "arn:aws:codewhisperer:us-east-1:123456789:profile/xxx",
    "region": "us-east-1",
    "enabled": true,
    "status": "active"
  }
]
```

**必要字段说明**：
- **Social 账号**：`authMethod`（"social"）、`refreshToken`
- **IDC 账号**：`authMethod`（"IdC"）、`clientId`、`clientSecret`、`refreshToken`

**可选字段**：
- `id` - 账号 ID（不填会自动生成）
- `name` - 账号名称（不填会使用默认名称）
- `provider` - 提供商（Social: "Google"/"GitHub", IDC: "BuilderId"）
- `profileArn` - 配置文件 ARN（Social 为空，IDC 可选）
- `region` - 区域（默认 us-east-1）
- `enabled` - 是否启用（默认 true）
- `status` - 账号状态（默认 active）

**注意**：
- 敏感字段（`refreshToken`、`accessToken`、`clientSecret`）会在保存时自动加密
- 已加密的数据格式为 JSON 对象：`{"ciphertext":"...","nonce":"..."}`
- 不要手动编辑已加密的字段
- `accessToken` 和 `expiresAt` 不需要填写，会自动通过 `refreshToken` 获取

## API 端点

### OpenAI 兼容

```bash
POST http://127.0.0.1:8080/v1/chat/completions
Content-Type: application/json
Authorization: Bearer your-api-key

{
  "model": "gpt-4",
  "messages": [
    {"role": "user", "content": "Hello"}
  ],
  "stream": true
}
```

### Anthropic 兼容

```bash
POST http://127.0.0.1:8080/v1/messages
Content-Type: application/json
x-api-key: your-api-key
anthropic-version: 2023-06-01

{
  "model": "claude-3-5-sonnet-20241022",
  "messages": [
    {"role": "user", "content": "Hello"}
  ],
  "max_tokens": 1024,
  "stream": true
}
```

### 管理端点

**注意**：所有 `/admin/*` 端点需要 Admin Token 认证。

```bash
# 获取 Admin Token
GET http://127.0.0.1:8080/admin/token
x-admin-token: your-admin-token

# 健康检查（无需认证）
GET http://127.0.0.1:8080/health

# 获取模型列表（无需认证）
GET http://127.0.0.1:8080/v1/models

# 获取统计数据
GET http://127.0.0.1:8080/admin/metrics
x-admin-token: your-admin-token

# 获取日志
GET http://127.0.0.1:8080/admin/logs
x-admin-token: your-admin-token

# 清空日志
POST http://127.0.0.1:8080/admin/logs/clear
x-admin-token: your-admin-token

# 账号管理
GET http://127.0.0.1:8080/admin/accounts
POST http://127.0.0.1:8080/admin/accounts
PATCH http://127.0.0.1:8080/admin/accounts/:id
DELETE http://127.0.0.1:8080/admin/accounts/:id
x-admin-token: your-admin-token

# API Key 管理
GET http://127.0.0.1:8080/admin/api-keys
POST http://127.0.0.1:8080/admin/api-keys
PATCH http://127.0.0.1:8080/admin/api-keys/:id
DELETE http://127.0.0.1:8080/admin/api-keys/:id
x-admin-token: your-admin-token
```

## 模型映射

Kiro Gateway 支持从 Kiro API 动态获取可用模型列表，并自动映射到 OpenAI/Anthropic 模型名称。

**当前支持的 Kiro 模型**：
- `qdev::auto` - 自动选择（默认）
- `qdev::claude-haiku-4.5` - 快速模型
- `qdev::claude-sonnet-4` - 常规模型
- `qdev::claude-sonnet-4.5` - 最新模型

**OpenAI → Kiro 映射**：
- `gpt-4` / `gpt-4-turbo` / `gpt-4o` → `claude-sonnet-4.5`
- `gpt-3.5-turbo` → `claude-haiku-4.5`

**Anthropic → Kiro 映射**：
- `claude-3-5-sonnet-*` → `claude-sonnet-4.5`
- `claude-3-haiku-*` → `claude-haiku-4.5`

**模糊匹配**：
- 包含 `haiku` → `claude-haiku-4.5`
- 包含 `sonnet` → `claude-sonnet-4.5`
- 默认 → `auto`

## 功能特性

### 已实现

- ✅ OpenAI Chat Completions API 兼容
- ✅ Anthropic Messages API 兼容
- ✅ 多账号轮询和自动切换
- ✅ 自动 Token 刷新（Social 和 IDC 账号）
- ✅ 流式响应（SSE）
- ✅ 工具调用支持
- ✅ 图片上传支持
- ✅ Thinking block 解析
- ✅ 动态模型列表加载（从 Kiro API 获取）
- ✅ 日志系统（结构化存储、搜索、过滤）
- ✅ 统计监控（请求数、响应时间、延迟百分位、24小时趋势）
- ✅ 桌面管理界面（Mantine UI）
- ✅ 账号健康检查
- ✅ WebSearch 集成（Kiro MCP API）
- ✅ API Key 管理系统（生成、验证、持久化）
- ✅ Metrics 持久化（自动保存/加载）
- ✅ **账号数据加密存储**（AES-256-GCM，机器特定密钥保护）
- ✅ **Admin API 认证保护**（自动生成 Admin Token，统一 middleware 认证）

## 参考项目

本项目参考了以下优秀项目的设计和实现：

- [aliom-v/KiroGate](https://github.com/aliom-v/KiroGate) - Python + FastAPI 实现，主要参考
- [chaogei/Kiro-account-manager](https://github.com/chaogei/Kiro-account-manager) - Rust + Axum + Tauri 实现，反代架构参考
- [justlovemaki/AIClient-2-API](https://github.com/justlovemaki/AIClient-2-API) - 多 Provider 架构参考
- [aiclientproxy/proxycast](https://github.com/aiclientproxy/proxycast) - Tauri 桌面应用参考
- [hank9999/kiro.rs](https://github.com/hank9999/kiro.rs) - Rust + React 前端参考

感谢这些项目的开源贡献！

## 开发规范

- 代码注释：中文
- 变量/函数命名：英文（snake_case）
- 日志 target：`kiro_gateway`
- 提交信息：遵循 Conventional Commits

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！

## 作者

Made with ❤️ by [hj01857655](https://github.com/hj01857655)

## 联系方式

- GitHub: [hj01857655/kiro-gateway](https://github.com/hj01857655/kiro-gateway)
- 问题反馈: [Issues](https://github.com/hj01857655/kiro-gateway/issues)
- QQ 交流群: [1081058179（Kiro GateWay交流群）](https://qm.qq.com/q/oQbUA0cxO2)

## 免责声明

本项目仅供学习和研究使用，请遵守 Kiro 服务条款，不要滥用 API 配额。
