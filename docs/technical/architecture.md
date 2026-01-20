---
inclusion: fileMatch
fileMatchPattern: "src-tauri/**/*.rs"
---

# 项目架构规范

## 项目概述

**kiro-gateway** - Kiro API → OpenAI/Anthropic API 兼容网关

**技术栈**: Rust + Axum + Tokio + Tauri 2.0

---

## 目录结构

```
src-tauri/src/
├── main.rs           # Tauri 入口（启动 Axum + 窗口）
├── server.rs         # Axum HTTP 服务器
├── config.rs         # 配置管理
├── account.rs        # 账号管理、Token 刷新
├── auth.rs           # Token 刷新逻辑
├── converter.rs      # OpenAI/Anthropic ↔ Kiro 格式转换
├── kiro_client.rs    # Kiro API 客户端
├── error.rs          # 错误类型定义
├── models.rs         # 数据结构定义
├── logger.rs         # 日志系统
├── metrics.rs        # 统计系统
├── api_key.rs        # API Key 管理
├── thinking_parser.rs  # Thinking 解析
└── websearch.rs      # WebSearch 集成
```

---

## 模块职责

### 核心模块

- **main** - Tauri 入口，启动 Axum 服务器和窗口
- **server** - HTTP 服务、路由、中间件
- **config** - 环境变量、账号配置加载

### API 处理

- **converter** - 请求/响应格式转换
- **kiro_client** - 调用 Kiro API、处理 SSE 流
- **account** - Token 刷新、账号选择
- **auth** - Token 刷新逻辑（Social/IDC）

### 辅助模块

- **error** - 统一错误处理、错误转换
- **models** - 数据结构定义
- **logger** - 结构化日志存储
- **metrics** - 请求统计和监控
- **api_key** - API Key 生成和验证
- **thinking_parser** - 解析 Kiro API 的 thinking block
- **websearch** - WebSearch 工具集成

---

## API 路由

### 核心接口

- `POST /v1/chat/completions` - OpenAI 兼容接口
- `POST /v1/messages` - Anthropic 兼容接口
- `GET /v1/models` - 模型列表
- `GET /health` - 健康检查

### 管理接口

- `GET /admin/accounts` - 获取账号列表
- `POST /admin/accounts` - 添加账号
- `PATCH /admin/accounts/:id` - 更新账号
- `DELETE /admin/accounts/:id` - 删除账号
- `POST /admin/accounts/:id/refresh` - 刷新 Token
- `GET /admin/accounts/:id/quota` - 查询配额
- `POST /admin/accounts/import` - 导入账号

### 日志和统计

- `GET /admin/logs` - 获取日志
- `POST /admin/logs/clear` - 清空日志
- `GET /admin/metrics` - 获取统计数据

### API Key 管理

- `POST /admin/api-keys` - 生成 API Key
- `GET /admin/api-keys` - 列出 API Key
- `PATCH /admin/api-keys/:id` - 更新 API Key
- `DELETE /admin/api-keys/:id` - 删除 API Key

---

## Rust 代码规范

### 代码风格

- 使用 `rustfmt` 默认格式化
- 错误处理优先使用 `Result`，避免 `unwrap()`
- 异步函数使用 `async/await`
- SSE 流使用 `tokio_stream`
- HTTP 客户端使用 `reqwest`

### 命名规范

- 变量/函数：`snake_case`
- 类型/结构体：`PascalCase`
- 常量：`SCREAMING_SNAKE_CASE`
- 模块：`snake_case`

### 注释规范

- 使用中文注释
- 函数使用 `///` 文档注释
- 复杂逻辑添加行内注释
- 解释"为什么"而不只是"做什么"

**示例**:
```rust
/// 刷新账号的 Access Token
/// 
/// 根据账号类型（Social/IDC）选择不同的刷新端点
/// 刷新成功后更新内存和配置文件中的 Token
async fn refresh_token(account: &mut Account) -> Result<()> {
    // 检查是否需要刷新（提前 5 分钟）
    if !should_refresh(&account.expires_at) {
        return Ok(());
    }
    
    // 根据账号类型选择刷新方式
    match account.auth_method.as_str() {
        "social" => refresh_social_token(account).await,
        "IdC" => refresh_idc_token(account).await,
        _ => Err(AppError::InvalidAuthMethod),
    }
}
```

---

## 依赖管理

### 核心依赖

- `axum` - HTTP 服务器框架
- `tokio` - 异步运行时
- `reqwest` - HTTP 客户端
- `serde` / `serde_json` - 序列化/反序列化
- `tauri` - 桌面应用框架

### 辅助依赖

- `tracing` - 日志系统
- `chrono` - 时间处理
- `uuid` - UUID 生成
- `tokio-stream` - 异步流处理
- `tower` / `tower-http` - 中间件

### 依赖原则

- 生产依赖精简，避免引入不必要的 crate
- features 按需启用，减少编译时间
- 优先使用成熟稳定的 crate

---

## 日志系统

### 使用 tracing

```rust
use tracing::{info, debug, warn, error};

// 记录日志
info!("服务器启动成功");
debug!("请求参数: {:?}", params);
warn!("Token 即将过期");
error!("请求失败: {}", err);
```

### 日志 target

统一使用 `kiro_gateway` 作为 target：

```rust
tracing::info!(target: "kiro_gateway", "日志消息");
```

### 结构化日志

使用 `logger.rs` 模块存储结构化日志：

```rust
use crate::logger::{kirogate_info, kirogate_error};

kirogate_info!("账号刷新成功", account_id = "xxx");
kirogate_error!("请求失败", error = %err);
```

---

## 错误处理

### 统一错误类型

定义在 `error.rs` 中：

```rust
#[derive(Debug)]
pub enum AppError {
    InvalidRequest(String),
    TokenExpired,
    AccountNotFound,
    RefreshFailed(String),
    KiroApiError(String),
    // ...
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // 转换为 HTTP 响应
    }
}
```

### 错误处理原则

- 使用 `Result<T, AppError>` 返回类型
- 避免 `unwrap()` 和 `expect()`
- 使用 `?` 操作符传播错误
- 在边界处转换错误类型

**示例**:
```rust
async fn refresh_account(id: &str) -> Result<Account, AppError> {
    let account = get_account(id)
        .ok_or(AppError::AccountNotFound)?;
    
    refresh_token(&mut account).await
        .map_err(|e| AppError::RefreshFailed(e.to_string()))?;
    
    Ok(account)
}
```

---

## 测试

### 运行测试

```bash
cargo test              # 运行所有测试
cargo test --lib        # 只运行库测试
cargo test test_name    # 运行特定测试
```

### 代码检查

```bash
cargo clippy            # 代码检查
cargo clippy --fix      # 自动修复
cargo fmt               # 格式化代码
cargo fmt --check       # 检查格式
```

### 测试规范

- 单元测试放在模块内部
- 集成测试放在 `tests/` 目录
- 使用 `#[tokio::test]` 测试异步函数
- 使用 `mockito` 模拟 HTTP 请求

---

## 配置管理

### 环境变量

- `HOST` - 监听地址（默认 127.0.0.1）
- `PORT` - 监听端口（默认 8080）
- `API_KEY` - 客户端访问密钥
- `ACCOUNTS_FILE` - 账号配置文件路径

### 配置文件

使用 Tauri 的用户数据目录：

- **Windows**: `%APPDATA%\com.kiro.gateway\`
- **macOS**: `~/Library/Application Support/com.kiro.gateway/`
- **Linux**: `~/.local/share/com.kiro.gateway/`

**配置文件**:
- `accounts.json` - 账号配置
- `api_keys.json` - API Key 配置
- `metrics.json` - 统计数据

---

## 性能优化

### 异步处理

- 使用 `tokio::spawn` 并发处理
- 使用 `tokio::select!` 处理超时
- 使用 `tokio_stream` 处理流式数据

### 内存管理

- 使用 `Arc` 共享数据
- 使用 `RwLock` 保护共享状态
- 避免不必要的克隆

### HTTP 优化

- 复用 HTTP 客户端
- 使用连接池
- 设置合理的超时时间

---

## 安全规范

### Token 管理

- Token 不记录到日志
- 配置文件权限设置为仅当前用户可读
- 内存中的 Token 使用 `Arc<RwLock<>>` 保护

### API Key 验证

- 使用 `sk-{48位十六进制}` 格式
- 验证失败返回 401
- 支持禁用/启用 API Key

### 错误信息

- 不在错误响应中暴露敏感信息
- 不在日志中记录完整 Token
- 使用通用错误消息

---

## 开发工作流

### 本地开发

```bash
# 启动开发服务器
cargo run

# 启动 Tauri 开发模式
cargo tauri dev

# 构建生产版本
cargo tauri build
```

### 代码提交

```bash
# 格式化 + 检查 + 测试 + 提交
cargo fmt && cargo clippy && cargo test && git add -A && git commit -m "feat: 新功能" && git push origin main
```

### 版本发布

1. 更新版本号（`Cargo.toml` 和 `tauri.conf.json`）
2. 提交版本更新
3. 创建并推送 tag 到公开仓库
4. GitHub Actions 自动构建发布

---

## 注意事项

1. **Tauri 生产环境** - 前端不能直接 fetch localhost，必须通过 Tauri invoke
2. **Token 刷新** - 提前 5 分钟刷新，避免请求时过期
3. **错误处理** - 统一使用 `AppError`，不要 panic
4. **日志记录** - 使用结构化日志，不记录敏感信息
5. **配置文件** - 使用用户数据目录，不要硬编码路径
