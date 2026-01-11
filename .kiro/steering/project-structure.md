# 项目结构规范

## 目录结构

```
src/
├── main.rs           # 入口，路由配置
├── config.rs         # 环境变量配置
├── converter.rs      # OpenAI/Anthropic ↔ Kiro 格式转换
├── kiro_client.rs    # Kiro API 客户端
├── account.rs        # 账号管理、Token 刷新
├── error.rs          # 错误类型定义
└── models.rs         # 数据结构定义
```

## 模块职责

- `main` - HTTP 服务、路由、中间件
- `config` - 环境变量、账号配置加载
- `converter` - 请求/响应格式转换
- `kiro_client` - 调用 Kiro API、处理 SSE 流
- `account` - Token 刷新、账号选择
- `error` - 统一错误处理、错误转换

## API 路由

- `POST /v1/chat/completions` - OpenAI 兼容
- `POST /v1/messages` - Anthropic 兼容
- `GET /v1/models` - 模型列表
- `GET /health` - 健康检查

## 配置方式

环境变量：
- `HOST` - 监听地址（默认 127.0.0.1）
- `PORT` - 监听端口（默认 8080）
- `API_KEY` - 客户端访问密钥
- `ACCOUNTS_FILE` - 账号配置文件路径
