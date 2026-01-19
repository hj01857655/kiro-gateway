# Kiro Gateway

Kiro Gateway 是一个基于 Tauri 2.0 的桌面应用，提供 OpenAI/Anthropic 兼容的 Kiro API 网关服务。

## 项目简介

Kiro Gateway 将 Kiro API 转换为标准的 OpenAI Chat Completions API 和 Anthropic Messages API，支持多账号管理、自动 Token 刷新、流式响应等功能。

**核心特性**：
- 🔄 OpenAI/Anthropic API 完全兼容
- 👥 多账号轮询和自动切换
- 🔐 自动 Token 刷新和管理
- 📊 实时统计和日志监控
- 🖥️ 友好的桌面管理界面
- 🛠️ 工具调用和图片支持
- 💭 Thinking block 解析
- 🎯 动态模型列表加载

## 技术栈

**后端**：
- Rust + Axum - HTTP API 服务
- Tauri 2.0 - 桌面应用框架
- Tokio - 异步运行时
- Reqwest - HTTP 客户端

**前端**：
- React 19 - UI 框架
- TypeScript - 类型安全
- Vite - 构建工具
- TailwindCSS 4 - 样式框架
- Mantine v7 - UI 组件库
- TanStack Query - 数据管理
- Zustand - 状态管理

## 项目结构

```
kiro-gateway/
├── src-tauri/              # Tauri + Rust 后端
│   ├── src/
│   │   ├── main.rs        # Tauri 入口（启动 Axum + 窗口）
│   │   ├── server.rs      # Axum HTTP 服务器
│   │   ├── account.rs     # 账号管理
│   │   ├── auth.rs        # Token 刷新
│   │   ├── converter.rs   # 格式转换
│   │   ├── kiro_client.rs # Kiro API 客户端
│   │   ├── config.rs      # 配置管理
│   │   ├── error.rs       # 错误处理
│   │   ├── models.rs      # 数据模型
│   │   ├── logger.rs      # 日志系统
│   │   ├── metrics.rs     # 统计系统
│   │   └── thinking_parser.rs  # Thinking 解析
│   └── Cargo.toml
├── src/                    # React 前端
│   ├── App.tsx            # 主应用组件
│   ├── main.tsx           # React 入口
│   └── index.css          # 样式
├── index.html             # HTML 入口
├── package.json           # 前端依赖
├── vite.config.ts         # Vite 配置
└── tauri.conf.json        # Tauri 配置
```

## 开发命令

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

### 环境变量

在 `src-tauri/.env` 或系统环境变量中配置：

```bash
# HTTP 服务监听地址（默认 127.0.0.1）
HOST=127.0.0.1

# HTTP 服务监听端口（默认 8080）
PORT=8080

# 账号配置文件路径（默认 accounts.json）
ACCOUNTS_FILE=accounts.json

# 管理员 API Key（可选）
ADMIN_API_KEY=your-admin-key
```

### 账号配置

在 `accounts.json` 中配置 Kiro 账号：

```json
{
  "accounts": [
    {
      "name": "账号1",
      "type": "social",
      "access_token": "your-access-token",
      "refresh_token": "your-refresh-token",
      "expires_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

**获取凭证**：
- 从 Kiro IDE 缓存：`~/.aws/sso/cache/kiro-auth-token.json`
- 或使用 Kiro Account Manager 导出

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

```bash
# 健康检查
GET http://127.0.0.1:8080/health

# 获取模型列表（动态从 Kiro API 获取）
GET http://127.0.0.1:8080/v1/models

# 获取统计数据
GET http://127.0.0.1:8080/admin/metrics

# 获取日志
GET http://127.0.0.1:8080/admin/logs

# 清空日志
POST http://127.0.0.1:8080/admin/logs/clear
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
- ✅ 自动 Token 刷新
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

## 联系方式

- GitHub: [hj01857655/kiro-gateway](https://github.com/hj01857655/kiro-gateway)
- 问题反馈: [Issues](https://github.com/hj01857655/kiro-gateway/issues)

## 免责声明

本项目仅供学习和研究使用，请遵守 Kiro 服务条款，不要滥用 API 配额。
