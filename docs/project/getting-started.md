# 快速开始

## 安装

### 从源码编译

```bash
# 克隆仓库
git clone https://github.com/your-username/kiro-gate.git
cd kiro-gate

# 编译
cargo build --release

# 运行
./target/release/kiro-gate
```

### 使用预编译二进制

从 [Releases](https://github.com/your-username/kiro-gate/releases) 下载对应平台的二进制文件。

## 配置

### 1. 创建账号配置文件

创建 `data/accounts.json`：

```json
[
  {
    "name": "主账号",
    "accessToken": "your-access-token",
    "refreshToken": "your-refresh-token",
    "profileArn": "",
    "authMethod": "social",
    "provider": "google"
  }
]
```

### 2. 环境变量（可选）

```bash
# 监听地址
export HOST=127.0.0.1
export PORT=8080

# API 密钥（用于客户端访问）
export API_KEY=your-secret-key

# 账号配置文件路径
export ACCOUNTS_FILE=./data/accounts.json
```

## 运行

```bash
# 直接运行
./target/release/kiro-gate

# 或使用环境变量
HOST=0.0.0.0 PORT=3000 ./target/release/kiro-gate
```

服务启动后访问：
- OpenAI 接口：`http://localhost:8080/v1/chat/completions`
- Anthropic 接口：`http://localhost:8080/v1/messages`
- 健康检查：`http://localhost:8080/health`

## 测试

### 使用 curl 测试

```bash
curl -X POST http://localhost:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-api-key" \
  -d '{
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello"}],
    "stream": true
  }'
```

### 使用 Python 测试

```python
from openai import OpenAI

client = OpenAI(
    base_url="http://localhost:8080/v1",
    api_key="your-api-key"
)

response = client.chat.completions.create(
    model="gpt-4",
    messages=[{"role": "user", "content": "Hello"}],
    stream=True
)

for chunk in response:
    print(chunk.choices[0].delta.content, end="")
```

## 下一步

- [账号管理](./account-management.md) - 了解如何刷新 Token
- [API 使用](./api-usage.md) - 查看完整的 API 文档
- [部署指南](./deployment.md) - 生产环境部署建议
