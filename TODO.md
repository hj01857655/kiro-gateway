# KiroGate 待办列表

> 最后更新：2026-01-11

## ✅ 已完成（100%）

### 核心功能
- [x] HTTP 服务框架 (Axum + Tokio)
- [x] OpenAI 兼容接口 `/v1/chat/completions`
- [x] Anthropic 兼容接口 `/v1/messages`
- [x] 模型列表接口 `/v1/models`
- [x] 健康检查接口 `/health`

### 请求转换
- [x] OpenAI → Kiro 格式转换
- [x] Anthropic → Kiro 格式转换
- [x] 模型名称映射（auto/haiku/sonnet-4/sonnet-4.5/opus）
- [x] 工具调用转换 (tools/tool_calls)
- [x] 图片内容转换
- [x] 历史消息转换
- [x] 相邻消息合并（merge_openai_messages / merge_anthropic_messages）
- [x] 长 tool description 处理（超过 4000 字符移到 system prompt）

### 响应转换
- [x] Kiro → OpenAI 流式响应
- [x] Kiro → OpenAI 非流式响应
- [x] Kiro → Anthropic 流式响应
- [x] Kiro → Anthropic 非流式响应
- [x] codeEvent 转 Markdown 代码块
- [x] toolUseEvent 转 tool_calls
- [x] reasoningContentEvent 转 thinking block（含 signature）
- [x] invalidStateEvent 错误处理
- [x] finish_reason 支持（stop/tool_calls/length）
- [x] OpenAI 流 usage 统计

### 账号管理
- [x] Social/IDC 两种认证类型
- [x] Token 自动刷新
- [x] 多账号轮询
- [x] 账号状态管理（active/expired/throttled/error/disabled）
- [x] 限流跳过逻辑（is_throttled / is_available / mark_throttled）
- [x] 刷新失败标记 expired
- [x] refreshToken 长度检查（< 100 字符警告）

### 超时与重试
- [x] 首 Token 超时（Haiku 30s / Sonnet 60s / Opus 120s）
- [x] 流读取超时（Haiku 60s / Sonnet 120s / Opus 300s）
- [x] 重试逻辑（指数退避，限流 500ms / 普通 100ms）

### 高级功能
- [x] 配额检查 API (GetUsageLimits) - `/admin/quota/{account_id}`
- [x] 健康检查 dryRun - `/admin/health`
- [x] 用户记忆 API - `/v1/memory` (GET/POST/DELETE)
- [x] 管理 API - `/admin/accounts`, `/admin/stats`

### 其他
- [x] API Key 验证
- [x] x-amz-user-agent 请求头
- [x] machine_id 生成（SHA256，64 字符）
- [x] 基础日志 (tracing)
- [x] 流式请求检查

---

## 📋 待办

暂无

---

## 工程师进度

- **main.rs** - ✅ 100%
- **converter.rs** - ✅ 100%
- **kiro_client.rs** - ✅ 100%
- **account.rs** - ✅ 100%
- **config.rs** - ✅ 100%
- **models.rs** - ✅ 100%
- **error.rs** - ✅ 100%

**整体完成度：100%** 🎉
