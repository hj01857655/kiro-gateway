# KiroGate

Kiro API 网关，提供 OpenAI/Anthropic 兼容接口，将请求转发到 Kiro 后端。

## 功能特性

- ✅ OpenAI 兼容接口 `/v1/chat/completions`
- ✅ Anthropic 兼容接口 `/v1/messages`
- ✅ 多账号轮询和自动 Token 刷新
- ✅ 账号状态管理（限流跳过、过期标记）
- ✅ 重试逻辑（指数退避 + 随机抖动）
- ✅ 超时控制（首 Token + 流读取）
- ✅ 工具调用转换
- ✅ 图片内容转换
- ✅ Thinking block 支持

## 快速开始

### 1. 配置账号

创建 `accounts.json`：

```json
{
  "accounts": [
    {
      "id": "account-1",
      "name": "My Account",
      "authMethod": "social",
      "accessToken": "eyJ...",
      "refreshToken": "eyJ...",
      "region": "us-east-1",
      "enabled": true
    }
  ]
}
```

### 2. 启动服务

```bash
# 使用配置文件
ACCOUNTS_FILE=accounts.json cargo run --release

# 或使用环境变量
ACCOUNTS_JSON='{"accounts":[...]}' cargo run --release
```

### 3. 调用接口

```bash
# OpenAI 格式
curl http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4.5",
    "messages": [{"role": "user", "content": "Hello"}],
    "stream": true
  }'

# Anthropic 格式
curl http://localhost:8080/v1/messages \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4.5",
    "messages": [{"role": "user", "content": "Hello"}],
    "stream": true
  }'
```

## 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `HOST` | 监听地址 | `127.0.0.1` |
| `PORT` | 监听端口 | `8080` |
| `API_KEY` | API 密钥（可选） | - |
| `ACCOUNTS_FILE` | 账号配置文件路径 | - |
| `ACCOUNTS_JSON` | 账号配置 JSON | - |
| `KIRO_ENDPOINT` | Kiro API 地址 | `https://codewhisperer.us-east-1.amazonaws.com` |
| `MACHINE_ID` | 机器 ID | 自动生成 |
| `RUST_LOG` | 日志级别 | `kiro_gate=info` |

## 模型映射

| 请求模型 | Kiro 模型 |
|----------|-----------|
| `gpt-4` / `gpt-4o` | `claude-sonnet-4.5` |
| `gpt-3.5-turbo` | `claude-haiku-4.5` |
| `claude-3-5-sonnet` | `claude-sonnet-4.5` |
| `claude-3-opus` | `claude-opus-4.5` |
| `sonnet` | `claude-sonnet-4.5` |
| `opus` | `claude-opus-4.5` |
| `haiku` | `claude-haiku-4.5` |
| `auto` / `kiro` | `auto` |

## 账号配置

### Social 账号

```json
{
  "id": "social-1",
  "authMethod": "social",
  "accessToken": "eyJ...",
  "refreshToken": "eyJ...",
  "region": "us-east-1",
  "enabled": true
}
```

### IDC 账号

```json
{
  "id": "idc-1",
  "authMethod": "IdC",
  "accessToken": "eyJ...",
  "refreshToken": "eyJ...",
  "profileArn": "arn:aws:...",
  "clientId": "...",
  "clientSecret": "...",
  "region": "us-east-1",
  "enabled": true
}
```

## 超时配置

| 模型 | 首 Token 超时 | 流读取超时 |
|------|---------------|------------|
| Haiku | 30s | 60s |
| Sonnet | 60s | 120s |
| Opus | 120s | 300s |

## 错误处理

- Token 过期：自动刷新后重试
- 限流：标记账号 60 秒，指数退避重试
- 网络错误：指数退避重试（最多 3 次）

## 开发

```bash
# 开发模式
cargo run

# 构建 release
cargo build --release

# 运行测试
cargo test
```

## License

MIT
