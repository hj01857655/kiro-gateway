# kiro-gateway

Kiro API 网关，提供 OpenAI/Anthropic 兼容接口。

## 功能

- OpenAI 兼容 `/v1/chat/completions`
- Anthropic 兼容 `/v1/messages`
- 多账号轮询 + 自动 Token 刷新
- 限流跳过、过期标记
- 工具调用、图片、Thinking block 支持
- Web 管理界面

## 快速开始

### 1. 配置账号

创建 `data/accounts.json`：

```json
[
  {
    "id": "my-account",
    "email": "user@example.com",
    "accessToken": "aoaAAAAA...",
    "refreshToken": "aorAAAAA...",
    "clientId": "CEiGMN1o...",
    "clientSecret": "eyJraWQi...",
    "region": "us-east-1",
    "status": "active"
  }
]
```

### 2. 启动服务

```bash
# 编译
cargo build --release

# 运行
./target/release/kiro-gateway
# 或 Windows: .\target\release\kiro-gateway.exe
```

默认监听 `http://127.0.0.1:8080`

### 3. 调用接口

```bash
# OpenAI 格式
curl http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{"model":"auto","messages":[{"role":"user","content":"Hello"}],"stream":true}'

# Anthropic 格式
curl http://localhost:8080/v1/messages \
  -H "Content-Type: application/json" \
  -d '{"model":"auto","messages":[{"role":"user","content":"Hello"}],"stream":true}'
```

## 环境变量

- `HOST` - 监听地址（默认 127.0.0.1）
- `PORT` - 监听端口（默认 8080）
- `API_KEY` - 访问密钥（可选）
- `ACCOUNTS_FILE` - 账号文件路径（默认 data/accounts.json）
- `RUST_LOG` - 日志级别（默认 kiro_gateway=info）

## 模型映射

- `gpt-4` / `gpt-4o` → `claude-sonnet-4.5`
- `gpt-3.5-turbo` → `claude-haiku-4.5`
- `claude-3-5-sonnet` → `claude-sonnet-4.5`
- `claude-3-opus` → `claude-opus-4.5`
- `auto` / `kiro` → `auto`

## 账号类型

**IDC 账号**（企业）：需要 `clientId`、`clientSecret`、`region`

**Social 账号**（个人）：只需要 `accessToken`、`refreshToken`

## Web 界面

```bash
cd web
npm install
npm run dev
```

访问 `http://localhost:5173`

## License

MIT
