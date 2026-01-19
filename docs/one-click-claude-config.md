# 一键配置 Claude 功能

## 功能概述

kiro-gateway 提供了"一键配置 Claude"功能，可以自动生成并应用 Claude Desktop、Claude CLI 和 OpenAI 兼容工具的配置文件。

## 使用方法

### 1. 打开设置页面

在 kiro-gateway 应用中，点击左侧导航栏的"设置"。

### 2. 生成配置

在"一键配置 Claude"卡片中，点击"生成配置"按钮。

系统会自动：
- 生成一个新的 API Key（如果需要）
- 创建 Claude Desktop 配置
- 创建 Claude CLI 配置脚本
- 创建 OpenAI 兼容配置

### 3. 选择配置方式

配置生成后，会弹出一个模态框，包含三个标签页：

#### Claude Desktop

- **配置内容**：显示 JSON 格式的配置文件
- **配置路径**：显示配置文件将被写入的位置
  - Windows: `%APPDATA%\Claude\config.json`
  - macOS: `~/Library/Application Support/Claude/config.json`
  - Linux: `~/.config/claude/config.json`
- **操作**：
  - 点击"复制配置"：复制配置内容到剪贴板
  - 点击"一键应用"：自动写入配置文件到系统目录

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

### Claude Desktop 配置

```json
{
  "apiBaseUrl": "http://127.0.0.1:8080",
  "apiKey": "sk-xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
}
```

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
2. **重启生效**：应用配置后，需要重启 Claude Desktop 或重新打开终端才能生效
3. **配置覆盖**：一键应用会覆盖现有的 Claude Desktop 配置文件
4. **网络地址**：默认使用 `http://127.0.0.1:8080`，如果修改了端口，需要手动调整配置

## API 端点

### 生成配置包

```
POST /admin/config/generate
Content-Type: application/json

{
  "apiKey": "sk-xxx" // 可选，不提供则自动生成
}
```

**响应**：

```json
{
  "claude_desktop_json": "{...}",
  "claude_desktop_path": "C:\\Users\\xxx\\AppData\\Roaming\\Claude\\config.json",
  "claude_cli_script": "...",
  "openai_config": "..."
}
```

### 应用配置到 Claude Desktop

```
POST /admin/config/apply
Content-Type: application/json

{
  "apiKey": "sk-xxx"
}
```

**响应**：

```json
{
  "success": true,
  "configPath": "C:\\Users\\xxx\\AppData\\Roaming\\Claude\\config.json",
  "message": "Claude Desktop 配置已成功写入"
}
```

## 技术实现

### 后端

- **文件**：`src-tauri/src/config_generator.rs`
- **功能**：
  - 生成 Claude Desktop JSON 配置
  - 生成 Claude CLI 脚本
  - 生成 OpenAI 兼容配置
  - 自动检测配置文件路径（跨平台）
  - 写入配置文件到系统目录

### 前端

- **文件**：`src/pages/Settings.tsx`
- **功能**：
  - 配置生成 UI
  - 三标签页展示不同配置
  - 一键复制和应用
  - 加载状态和错误处理

## 常见问题

### Q: 一键应用失败怎么办？

A: 可能是权限问题，尝试：
1. 以管理员身份运行 kiro-gateway
2. 手动复制配置内容，创建配置文件
3. 检查配置文件路径是否正确

### Q: Claude Desktop 没有生效？

A: 确保：
1. 配置文件已正确写入
2. 已重启 Claude Desktop
3. 配置文件格式正确（JSON 格式）

### Q: 如何验证配置是否成功？

A: 
1. 打开 Claude Desktop，查看设置中的 API 配置
2. 发送一条消息，查看是否通过 kiro-gateway 转发
3. 检查 kiro-gateway 的日志和统计

### Q: 可以自定义 API Key 吗？

A: 可以，在生成配置前，先在"API Key 管理"中生成一个 API Key，然后在配置生成时会自动使用。
