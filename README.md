# kiro-gateway

Kiro API 网关服务，提供 OpenAI/Anthropic 兼容接口。

## 功能特性

- ✅ OpenAI 兼容 `/v1/chat/completions`
- ✅ Anthropic 兼容 `/v1/messages`
- ✅ Kiro 多账号轮询 + 自动 Token 刷新
- ✅ 限流跳过、过期标记
- ✅ 工具调用、图片、Thinking block 支持
- ✅ Web 管理界面
- ✅ Tauri 2 桌面应用
- ✅ 防抖保存（减少文件 I/O）
- ✅ Metrics 统计监控
- ✅ 日志系统

## 技术栈

- **后端**: Rust + Axum + Tokio
- **前端**: React 19 + TypeScript + Vite 6 + TailwindCSS 4
- **桌面**: Tauri 2.0

## 快速开始

### Web 版本

```bash
# 1. 配置账号
cp data/accounts.json.example data/accounts.json
# 编辑 data/accounts.json 添加你的 Kiro 账号

# 2. 启动后端
cargo run --release

# 3. 启动前端（可选）
cd web
npm install
npm run dev
```

访问 http://localhost:8080 使用 Web 管理界面。

### 桌面版本

```bash
# 1. 安装依赖
cd web
npm install

# 2. 启动桌面应用（开发模式）
npm run tauri:dev

# 3. 构建桌面应用
npm run tauri:build
```

## 配置说明

### 环境变量

```bash
HOST=127.0.0.1          # 监听地址
PORT=8080               # 监听端口
API_KEY=your-api-key    # 客户端访问密钥（可选）
ACCOUNTS_FILE=data/accounts.json  # 账号配置文件路径
```

### 账号配置

支持两种账号类型：

**Social 账号**（Google/GitHub）:
```json
{
  "id": "account-1",
  "authMethod": "social",
  "refreshToken": "your-refresh-token",
  "region": "us-east-1",
  "enabled": true
}
```

**IDC 账号**（AWS Builder ID）:
```json
{
  "id": "account-2",
  "authMethod": "idc",
  "refreshToken": "your-refresh-token",
  "clientId": "your-client-id",
  "clientSecret": "your-client-secret",
  "region": "us-east-1",
  "enabled": true
}
```

## API 端点

### 核心 API

- `POST /v1/chat/completions` - OpenAI 兼容接口
- `POST /v1/messages` - Anthropic 兼容接口
- `GET /v1/models` - 模型列表
- `GET /health` - 健康检查

### 管理 API

- `GET /admin/accounts` - 列出所有账号
- `POST /admin/accounts` - 添加账号
- `DELETE /admin/accounts/:id` - 删除账号
- `POST /admin/accounts/:id/refresh` - 刷新 Token
- `POST /admin/accounts/:id/enable` - 启用账号
- `POST /admin/accounts/:id/disable` - 禁用账号
- `GET /admin/quota/:id` - 获取配额
- `GET /admin/metrics` - 统计数据
- `GET /admin/logs` - 日志查看
- `POST /admin/logs/clear` - 清空日志

## 开发

```bash
# 检查代码
cargo check

# 运行测试
cargo test

# 格式化代码
cargo fmt

# Lint 检查
cargo clippy

# 前端开发
cd web
npm run dev

# Tauri 开发
cd web
npm run tauri:dev
```

## 部署

### Docker

```bash
docker build -t kiro-gateway .
docker run -d -p 8080:8080 -v ./data:/app/data kiro-gateway
```

### 二进制

```bash
cargo build --release
./target/release/kiro-gateway
```

## 参考项目

- [aliom-v/KiroGate](https://github.com/aliom-v/KiroGate) - Python 实现，主要参考
- [hank9999/kiro.rs](https://github.com/hank9999/kiro.rs) - Rust 实现，前端参考
- [aiclientproxy/proxycast](https://github.com/aiclientproxy/proxycast) - Tauri 桌面应用参考
- [justlovemaki/AIClient-2-API](https://github.com/justlovemaki/AIClient-2-API) - 架构设计参考

## License

MIT
