# Rust 开发规范

## 项目概述

KiroGate - Kiro API → OpenAI/Anthropic API 兼容网关

技术栈：Rust + Axum + Tokio

## Rust 代码风格

- 使用 `rustfmt` 默认格式化
- 错误处理优先使用 `Result`，避免 `unwrap()`
- 异步函数使用 `async/await`
- SSE 流使用 `tokio_stream`
- HTTP 客户端使用 `reqwest`

## 依赖管理

- 生产依赖精简，避免引入不必要的 crate
- features 按需启用，减少编译时间

## 日志

使用 `tracing`：`info!`、`debug!`、`error!`

## 测试

```bash
cargo test    # 运行测试
cargo clippy  # 检查代码
cargo fmt     # 格式化
```
