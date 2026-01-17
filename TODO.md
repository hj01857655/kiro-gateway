# kiro-gateway 待办列表

> 最后更新：2026-01-17

---

## 📋 前端待办

### 技术栈
- React + Vite + TypeScript ✅
- TailwindCSS ✅
- 部署：Vercel

### 功能模块

**账号管理** ✅
- [x] 账号列表展示（状态、名称、类型）
- [x] 添加账号（Social/IDC）
- [x] 编辑账号（启用/禁用）
- [x] 删除账号
- [x] Token 手动刷新
- [x] 从 Kiro IDE 导入账号

**状态监控** ✅
- [x] 账号状态实时显示（active/expired/throttled/error）
- [x] 请求统计（Metrics）
- [x] 延迟分布（P50/P95/P99）
- [x] 端点/状态码/模型统计
- [x] 最近请求记录

**日志查看** ✅
- [x] 日志列表展示
- [x] 日志级别过滤
- [x] 日志搜索
- [x] 清空日志
- [x] 自动刷新

**聊天界面**
- [ ] 基础聊天 UI
- [ ] 模型选择
- [ ] 流式响应显示
- [ ] 历史记录

**系统设置** ✅
- [x] API 地址配置说明
- [x] 主题切换（深色/浅色）

### 项目结构
```
web/
├── src/
│   ├── api/           # API 调用 ✅
│   ├── pages/         # 页面 ✅
│   │   ├── Accounts.tsx   # 账号管理 ✅
│   │   ├── Metrics.tsx    # 统计监控 ✅
│   │   ├── Logs.tsx       # 日志查看 ✅
│   │   ├── Chat.tsx       # 聊天测试
│   │   └── Settings.tsx   # 设置 ✅
│   ├── App.tsx        # 主应用 ✅
│   ├── main.tsx       # 入口 ✅
│   └── index.css      # 样式 ✅
├── public/
└── package.json       # 依赖 ✅
```

---

## ✅ 后端已完成（100%）

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
- [x] WebSearch 集成
- [x] Logger 日志系统
- [x] Metrics 统计系统

### 其他
- [x] API Key 验证
- [x] x-amz-user-agent 请求头
- [x] machine_id 生成（SHA256，64 字符）
- [x] 基础日志 (tracing)
- [x] 流式请求检查

---

## 📋 待办

### 前端
- [ ] 聊天测试界面（可选）

### 后端（可选）
- [ ] API Key 系统（生成用户 API Key）
- [ ] Metrics 持久化（SQLite）
- [ ] Token 计数 API (`/v1/messages/count_tokens`)

---

## 工程师进度

**后端**：
- **main.rs** - ✅ 100%
- **converter.rs** - ✅ 100%
- **kiro_client.rs** - ✅ 100%
- **account.rs** - ✅ 100%
- **config.rs** - ✅ 100%
- **models.rs** - ✅ 100%
- **error.rs** - ✅ 100%
- **websearch.rs** - ✅ 100%
- **logger.rs** - ✅ 100%
- **metrics.rs** - ✅ 100%
- **thinking_parser.rs** - ✅ 100%
- **auth.rs** - ✅ 100%

**前端**：
- **App.tsx** - ✅ 100%
- **pages/Accounts.tsx** - ✅ 100%
- **pages/Metrics.tsx** - ✅ 100%
- **pages/Logs.tsx** - ✅ 100%
- **pages/Chat.tsx** - ⚠️ 50% (占位页面)
- **pages/Settings.tsx** - ✅ 100%
- **api/accounts.ts** - ✅ 100%

**整体完成度：95%** 🎉
