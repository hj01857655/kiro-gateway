# kiro-gateway 项目规范

## 项目信息

- **项目名称**: kiro-gateway (原 kiro-gate)
- **GitHub 私有仓库**: https://github.com/hj01857655/kiro-gateway_dev
- **GitHub 公开仓库**: https://github.com/hj01857655/kiro-gateway
- **本地路径**: `E:\VSCodeSpace\Kiro\kiro-gateway`
- **技术栈**: Rust + Axum (后端)，React + TypeScript + Vite (前端)

## 项目定位

**专注 Kiro API 的网关服务**，提供 OpenAI/Anthropic 兼容接口。

⚠️ **重要说明**：
- **只支持 Kiro API**，不支持 Gemini、Antigravity、Qwen 等其他 Provider
- 专注于 Kiro 的多账号管理和 API 转换
- 参考项目中的其他 Provider 功能仅作架构参考，不实现

**核心功能**：
- OpenAI 兼容 `/v1/chat/completions`
- Anthropic 兼容 `/v1/messages`
- Kiro 多账号轮询 + 自动 Token 刷新
- 限流跳过、过期标记
- 工具调用、图片、Thinking block 支持
- Web 管理界面（可选）

## 参考项目

### 1. aliom-v/KiroGate ⭐⭐⭐⭐⭐ 主要参考
- **GitHub**: https://github.com/aliom-v/KiroGate
- **本地路径**: `E:\VSCodeSpace\Kiro\KiroGate`
- **技术栈**: Python + FastAPI
- **维护状态**: 与 aliom-v 共同维护
- **上游项目**: Fork 自 [Jwadow/kiro-openai-gateway](https://github.com/Jwadow/kiro-openai-gateway)
- **特点**:
  - 完整的格式转换实现（converters.py）
  - 完善的 Metrics 系统（使用 SQLite 持久化）
  - 支持 IP 统计和黑名单
  - 支持站点开关、自用模式、审批开关
  - 支持 Prometheus 格式导出
  - WebSearch 集成
  - 图片支持
  - IDC (Builder ID) 认证支持
  - 用户系统（LinuxDo/GitHub OAuth2）
  - Admin 管理后台
- **开发建议**: 可以直接参考和借鉴 KiroGate 的实现思路，有问题可以与 aliom-v 讨论

**访问方式**（工作区外文件）:
```powershell
Get-Content "E:\VSCodeSpace\Kiro\KiroGate\文件路径" -Raw
```

**关键文件参考**:
- `kiro_gateway/converters.py` - 格式转换（图片提取、消息合并、历史构建）
- `kiro_gateway/metrics.py` - Metrics 系统（SQLite 持久化）
- `kiro_gateway/models.py` - 数据模型定义
- `kiro_gateway/websearch.py` - WebSearch 工具集成
- `kiro_gateway/auth.py` - Token 管理和刷新
- `kiro_gateway/user_manager.py` - 用户系统和 OAuth2
- `kiro_gateway/token_allocator.py` - Token 智能分配
- `kiro_gateway/health_checker.py` - Token 健康检查

### 1.1. Jwadow/kiro-openai-gateway ⭐⭐⭐⭐⭐ KiroGate 上游
- **GitHub**: https://github.com/Jwadow/kiro-openai-gateway
- **技术栈**: Python + FastAPI
- **特点**:
  - KiroGate 的原始版本
  - 基础的 OpenAI/Anthropic 兼容实现
  - 简洁的架构设计
- **参考用途**: 了解 KiroGate 的基础架构和设计思路

### 2. justlovemaki/AIClient-2-API ⭐⭐⭐⭐ 架构参考
- **GitHub**: https://github.com/justlovemaki/AIClient-2-API
- **技术栈**: Node.js + Express
- **Stars**: 2.7k+
- **特点**:
  - 统一多种客户端 API（Gemini CLI、Antigravity、Qwen Code、**Kiro**）
  - OpenAI/Claude/Gemini 三协议智能互转
  - 账号池管理（多账号轮询、自动故障转移、健康检查）
  - Web UI 管理控制台（实时配置、健康监控、日志查看）
  - 模块化架构（策略模式 + 适配器模式）
  - 跨类型 Fallback 配置（配额耗尽时自动降级）
  - 代理配置支持（统一代理 + 提供商自带端点）
  - 模型过滤配置（notSupportedModels）
- **参考用途**（仅 Kiro 相关部分）:
  - Kiro 账号池管理架构设计
  - Kiro 健康检查和故障转移机制
  - Web UI 管理界面设计（可选）
  - Kiro 多账号轮询策略

**关键功能参考**（仅 Kiro 部分）:
- Kiro 账号池轮询和健康检查
- Kiro 账号故障转移机制
- Kiro 配额监控和自动切换
- Web UI 实时配置管理（可选）

⚠️ **注意**：该项目支持多种 Provider，但 kiro-gateway **只实现 Kiro 相关功能**，其他 Provider（Gemini、Antigravity、Qwen）仅作架构参考。

### 3. aiclientproxy/proxycast ⭐⭐⭐⭐ 桌面应用参考
- **GitHub**: https://github.com/aiclientproxy/proxycast
- **技术栈**: Tauri 2.0 + React 18 + Rust
- **Stars**: 1.1k+
- **特点**:
  - 多 Provider 统一管理（**Kiro**、Gemini、通义千问、Antigravity、Vertex AI）
  - 智能凭证管理（自动检测变化、Token 自动刷新、配额超限自动切换）
  - OpenAI Chat API + Anthropic Messages API 完整兼容
  - 友好图形界面（Dashboard 监控、Provider 管理、日志查看）
  - 一键读取凭证功能
- **参考用途**（仅 Kiro 相关部分）:
  - Kiro 凭证自动检测和管理
  - Kiro 健康监控界面
  - Kiro 配额超限自动切换逻辑
  - Dashboard 实时监控界面（可选）

**关键功能参考**（仅 Kiro 部分）:
- Kiro 凭证文件自动检测和加载
- Kiro 健康状态监控
- Kiro 配额超限自动切换账号
- Dashboard 实时监控界面（可选）

⚠️ **注意**：该项目支持多种 Provider，但 kiro-gateway **只实现 Kiro 相关功能**，其他 Provider 仅作架构参考。

### 4. hank9999/kiro.rs ⭐⭐⭐ 前端参考
- **GitHub**: https://github.com/hank9999/kiro.rs
- **技术栈**: Rust + Axum + React 18 + TypeScript + Vite + TailwindCSS + Radix UI
- **特点**: 
  - 有完整的前端 UI（React + TypeScript）
  - 使用 TanStack Query 管理数据
  - 支持凭据管理
  - **没有 Metrics 系统**
- **参考用途**: 前端 UI 设计和实现

## 已完成的功能迁移

从 Kiro Account Manager 迁移到 kiro-gateway 的功能：

### ✅ Logger 日志系统
- 文件: `src/logger.rs`
- 不依赖 Tauri，使用 `once_cell` + `tokio`
- 结构化日志存储（最多保留 1000 条）
- 异步和同步日志记录
- 便捷宏：`kirogate_info!`, `kirogate_debug!`, `kirogate_warn!`, `kirogate_error!`
- API 端点：
  - `GET /admin/logs` - 获取所有日志
  - `POST /admin/logs/clear` - 清空日志

### ✅ Metrics 统计系统
- 文件: `src/metrics.rs`
- 请求计数（按端点、状态码、模型）
- 流式/非流式请求统计
- API 类型使用量统计（OpenAI/Anthropic）
- 响应时间记录（最近 100 条）
- 延迟直方图（P50/P95/P99）
- 最近请求记录（最近 50 条）
- 24 小时请求统计
- API 端点：
  - `GET /admin/metrics` - 获取统计数据

### ✅ ThinkingParser
- 文件: `src/thinking_parser.rs`
- 解析 Kiro API 返回的 thinking block
- 与 Kiro IDE 实现完全一致

### ⚠️ WebSearch（已复制但未集成）
- 文件: `src/websearch.rs`
- 需要适配独立服务架构
- 需要移除对 Tauri 相关结构的依赖

## 待完成功能

### 1. WebSearch 集成
- 修改 `src/websearch.rs` 适配 kiro-gateway 架构
- 在 `main.rs` 中启用 `mod websearch;`
- 在 `messages` 函数中添加 WebSearch 请求检测逻辑

### 2. API Key 系统（可选）
- 生成 `sk-{48位十六进制}` 格式的 API Key
- API Key 映射存储（JSON 文件）
- 修改 `verify_api_key` 函数支持用户 API Key
- 管理 API：
  - `POST /admin/api-keys` - 生成新 API Key
  - `GET /admin/api-keys` - 列出所有 API Key
  - `DELETE /admin/api-keys/:id` - 删除 API Key

### 3. Metrics 持久化（可选）
- 参考 KiroGate 的 SQLite 实现
- 或使用 JSON 文件简单持久化

## Metrics 说明

**Metrics 不是必须的**：
- hank9999/kiro.rs 没有 Metrics 系统
- aliom-v/KiroGate 有完善的 Metrics 系统（包括 IP 统计、黑名单、持久化）
- 当前 kiro-gateway 的 Metrics 是从 Kiro Account Manager 移植的简化版本
- 如果需要更完善的 Metrics，可以参考 KiroGate 的实现

## 访问工作区外文件

KiroGate 项目在工作区外，需要通过 PowerShell 访问：
```powershell
Get-Content "E:\VSCodeSpace\Kiro\KiroGate\文件路径" -Raw
```

**注意**: KiroGate 是与 aliom-v 共同维护的项目，可以直接参考和借鉴其实现。

## 开发规范

- 代码注释：中文
- 变量/函数命名：英文（snake_case）
- 日志 target：`kiro_gateway`
- 二进制名称：`kiro-gateway`

## Git 仓库规则

- **私有仓库** (`kiro-gateway_dev`): 所有开发代码提交到 `main` 分支
- **公开仓库** (`kiro-gateway`): 仅用于发布 Release
- 发布时在公开仓库打 tag 触发 Actions 构建

## 相关文档

- 迁移报告：`E:\VSCodeSpace\Kiro\kiro-gateway\MIGRATION_REPORT.md`
- Kiro IDE 源码位置：`C:\Users\12925\.kiro\steering\kiro-ide-source.md`
- KiroGate 参考规范：`.kiro/steering/kirogate.md`
