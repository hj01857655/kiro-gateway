# 一键配置 Claude 功能

## 功能概述

kiro-gateway 提供了"一键配置 Claude"功能，可以自动生成 Claude CLI 和 OpenAI 兼容工具的配置脚本。

**注意**：Claude Desktop 使用 MCP 服务器配置，不支持直接配置 HTTP API。kiro-gateway 提供的是 OpenAI/Anthropic 兼容的 HTTP API，因此只支持 Claude CLI 和其他支持 OpenAI API 的工具。

## 使用方法

### 1. 打开设置页面

在 kiro-gateway 应用中，点击左侧导航栏的"设置"。

### 2. 生成配置

在"一键配置 Claude"卡片中，点击"生成配置"按钮。

系统会自动：
- 生成一个新的 API Key（如果需要）
- 创建 Claude CLI 配置脚本
- 创建 OpenAI 兼容配置

### 3. 选择配置方式

配置生成后，会弹出一个模态框，包含两个标签页：

#### Claude CLI

- **配置脚本**：显示环境变量设置脚本
- **支持平台**：
  - Windows PowerShell
  - Linux/macOS Bash/Zsh
  - Claude CLI 命令
- **操作**：
  - 点击"复制脚本"：复制脚本到剪贴板
  - 在终端中执行脚本

#### OpenAI 兼容

- **配置内容**：显示 OpenAI API 格式的配置
- **适用工具**：
  - Continue
  - Cursor
  - LangChain
  - 其他支持 OpenAI API 的工具
- **操作**：
  - 点击"复制配置"：复制配置到剪贴板
  - 根据工具要求设置环境变量或配置文件

## 配置格式

### Claude CLI 配置

```bash
# Windows (PowerShell)
$env:ANTHROPIC_BASE_URL="http://127.0.0.1:8080"
$env:ANTHROPIC_API_KEY="sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

# Linux/macOS (Bash/Zsh)
export ANTHROPIC_BASE_URL="http://127.0.0.1:8080"
export ANTHROPIC_API_KEY="sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

# 或者使用 claude config 命令
claude config set --global apiBaseUrl "http://127.0.0.1:8080"
claude config set --global apiKey "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
```

### OpenAI 兼容配置

```bash
# .env 文件格式
OPENAI_API_BASE=http://127.0.0.1:8080
OPENAI_API_KEY=sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

# 或环境变量格式
# Windows (PowerShell)
$env:OPENAI_API_BASE="http://127.0.0.1:8080"
$env:OPENAI_API_KEY="sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"

# Linux/macOS (Bash/Zsh)
export OPENAI_API_BASE="http://127.0.0.1:8080"
export OPENAI_API_KEY="sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
```

## 注意事项

1. **API Key 安全**：生成的 API Key 是敏感信息，请妥善保管
2. **重启生效**：应用配置后，需要重新打开终端才能生效
3. **网络地址**：默认使用 `http://127.0.0.1:8080`，如果修改了端口，需要手动调整配置
4. **Claude Desktop 不支持**：Claude Desktop 使用 MCP 服务器配置，无法直接配置 HTTP API

## API 端点

### 生成配置包

```
POST /admin/config/generate
Content-Type: application/json

{}
```

**响应**：

```json
{
  "claude_cli_script": "...",
  "openai_config": "..."
}
```

## 技术实现

### 后端

- **文件**：`src-tauri/src/config_generator.rs`
- **功能**：
  - 生成 Claude CLI 脚本
  - 生成 OpenAI 兼容配置

### 前端

- **文件**：`src/pages/Settings.tsx`
- **功能**：
  - 配置生成 UI
  - 两标签页展示不同配置
  - 一键复制
  - 加载状态和错误处理

## 常见问题

### Q: 为什么没有 Claude Desktop 配置？

A: Claude Desktop 使用 MCP (Model Context Protocol) 服务器配置，不支持直接配置 HTTP API。kiro-gateway 提供的是 OpenAI/Anthropic 兼容的 HTTP API，因此只支持 Claude CLI 和其他支持 OpenAI API 的工具。

### Q: 如何在 Claude Desktop 中使用 kiro-gateway？

A: 目前无法直接在 Claude Desktop 中使用 kiro-gateway。如果需要使用 Claude Desktop，建议直接使用 Kiro IDE 的账号。

### Q: Claude CLI 配置后没有生效？

A: 确保：
1. 已正确设置环境变量
2. 已重新打开终端
3. 使用 `echo $ANTHROPIC_BASE_URL` (Linux/macOS) 或 `echo $env:ANTHROPIC_BASE_URL` (Windows) 验证环境变量

### Q: 如何验证配置是否成功？

A: 
1. 使用 Claude CLI 发送一条消息
2. 查看 kiro-gateway 的日志和统计
3. 确认请求通过 kiro-gateway 转发
