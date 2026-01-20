# Kiro Gateway 使用教程

> 📖 从零开始，5 分钟学会使用 Kiro Gateway

## 目录

- [什么是 Kiro Gateway？](#什么是-kiro-gateway)
- [安装应用](#安装应用)
- [添加账号](#添加账号)
- [配置客户端](#配置客户端)
- [开始使用](#开始使用)
- [常见问题](#常见问题)

---

## 什么是 Kiro Gateway？

Kiro Gateway 是一个**桌面应用**，它的作用是：

1. **转换 API 格式** - 把 Kiro API 转成 OpenAI/Claude 的标准格式
2. **管理多个账号** - 自动轮换账号，避免单个账号配额用完
3. **自动刷新 Token** - 不用担心 Token 过期，自动帮你刷新
4. **提供管理界面** - 可视化管理账号、查看日志、监控统计

**简单来说**：装上它，你就可以用 Claude Desktop、Cursor、Continue 等工具直接调用 Kiro API 了。

---

## 安装应用

### 1. 下载安装包

访问 [GitHub Releases](https://github.com/hj01857655/kiro-gateway/releases/latest)，根据你的系统下载对应的安装包：

| 系统 | 下载文件 |
|------|---------|
| Windows | `kiro-gateway_x64-setup.msi` |
| macOS (Intel) | `kiro-gateway_x64.dmg` |
| macOS (M1/M2/M3) | `kiro-gateway_aarch64.dmg` |
| Linux (通用) | `kiro-gateway_amd64.AppImage` |
| Linux (Debian/Ubuntu) | `kiro-gateway_amd64.deb` |

### 2. 安装

**Windows**：
- 双击 `.msi` 文件
- 按提示安装即可

**macOS**：
- 双击 `.dmg` 文件
- 拖动应用到"应用程序"文件夹
- 首次打开可能需要在"系统偏好设置 → 安全性与隐私"中允许

**Linux (AppImage)**：
```bash
chmod +x kiro-gateway_amd64.AppImage
./kiro-gateway_amd64.AppImage
```

**Linux (deb)**：
```bash
sudo dpkg -i kiro-gateway_amd64.deb
```

### 3. 启动应用

安装完成后，启动 Kiro Gateway 应用。

---

## 添加账号

打开应用后，点击左侧菜单的"账号管理"，有三种方式添加账号：

### 方式 1：从 Kiro IDE 导入（推荐）

如果你已经在用 Kiro IDE（Amazon Q），可以一键导入：

1. 点击"从 Kiro IDE 导入"按钮
2. 应用会自动读取 `~/.aws/sso/cache/kiro-auth-token.json`
3. 导入成功后，账号会自动添加到列表

**优点**：最简单，不需要手动复制 Token

### 方式 2：手动添加

如果你有 Refresh Token，可以手动添加：

#### Social 账号（Google/GitHub 登录）

1. 点击"添加账号"按钮
2. 选择"Social"
3. 填写 Refresh Token（从 Kiro IDE 的 `kiro-auth-token.json` 中复制）
4. 点击"保存"

#### IDC 账号（Builder ID 登录）

1. 点击"添加账号"按钮
2. 选择"IdC"
3. 填写以下信息：
   - **Client ID**：从 Kiro IDE 配置中获取
   - **Client Secret**：从 Kiro IDE 配置中获取
   - **Refresh Token**：从 Kiro IDE 配置中获取
4. 点击"保存"

### 方式 3：批量导入 JSON 文件

如果你有多个账号，可以批量导入：

1. 创建一个 JSON 文件（例如 `accounts.json`）
2. 按以下格式填写：

**Social 账号示例**：
```json
[
  {
    "authMethod": "social",
    "refreshToken": "eyJ..."
  },
  {
    "authMethod": "social",
    "refreshToken": "eyJ..."
  }
]
```

**IDC 账号示例**：
```json
[
  {
    "authMethod": "IdC",
    "clientId": "MkAG97...",
    "clientSecret": "eyJraWQ...",
    "refreshToken": "aorAAAAA..."
  }
]
```

3. 在"账号管理"页面点击"批量导入"按钮
4. 选择你的 JSON 文件
5. 导入成功后，所有账号会自动添加到列表

**注意**：
- `refreshToken` 可以从 Kiro IDE 的配置文件中获取
- 路径：`~/.aws/sso/cache/kiro-auth-token.json`（macOS/Linux）
- 路径：`%USERPROFILE%\.aws\sso\cache\kiro-auth-token.json`（Windows）

---

## 配置客户端

添加账号后，需要配置你的客户端（Claude Desktop、Cursor、Continue 等）来使用 Kiro Gateway。

### 方式 1：一键配置 Claude Desktop（推荐）

1. 点击左侧菜单的"设置"
2. 找到"配置生成器"部分
3. 点击"生成 Claude Desktop 配置"
4. 点击"应用配置"按钮
5. 重启 Claude Desktop

**完成！** Claude Desktop 现在会通过 Kiro Gateway 调用 Kiro API。

### 方式 2：手动配置 Claude Desktop

如果一键配置失败，可以手动配置：

1. 打开 Claude Desktop 的配置文件：
   - **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
   - **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`

2. 添加以下内容：
```json
{
  "mcpServers": {
    "kiro-gateway": {
      "command": "curl",
      "args": [
        "-X", "POST",
        "http://127.0.0.1:8080/v1/messages",
        "-H", "Content-Type: application/json",
        "-H", "x-api-key: your-api-key",
        "-H", "anthropic-version: 2023-06-01",
        "-d", "@-"
      ]
    }
  }
}
```

3. 将 `your-api-key` 替换为你在"设置"页面生成的 API Key
4. 保存文件并重启 Claude Desktop

### 方式 3：配置 Cursor

1. 打开 Cursor 设置
2. 找到"Models"部分
3. 添加自定义模型：
   - **Base URL**: `http://127.0.0.1:8080/v1`
   - **API Key**: 在"设置"页面生成的 API Key
   - **Model**: `claude-3-5-sonnet-20241022`

### 方式 4：配置 Continue

1. 打开 Continue 配置文件（`~/.continue/config.json`）
2. 添加以下内容：
```json
{
  "models": [
    {
      "title": "Kiro Gateway",
      "provider": "openai",
      "model": "gpt-4",
      "apiBase": "http://127.0.0.1:8080/v1",
      "apiKey": "your-api-key"
    }
  ]
}
```

3. 将 `your-api-key` 替换为你在"设置"页面生成的 API Key
4. 重启 VS Code

---

## 开始使用

配置完成后，你就可以开始使用了！

### 测试连接

1. 打开 Claude Desktop / Cursor / Continue
2. 发送一条消息："你好"
3. 如果收到回复，说明配置成功！

### 查看日志

在 Kiro Gateway 应用中：
1. 点击左侧菜单的"日志"
2. 可以看到所有请求的详细日志
3. 支持搜索、过滤、导出

### 查看统计

在 Kiro Gateway 应用中：
1. 点击左侧菜单的"统计"
2. 可以看到：
   - 总请求数
   - 成功率
   - 平均响应时间
   - 24 小时请求趋势图
   - 延迟百分位（P50/P95/P99）

### 管理账号

在 Kiro Gateway 应用中：
1. 点击左侧菜单的"账号管理"
2. 可以：
   - 查看所有账号的状态
   - 启用/禁用账号
   - 删除账号
   - 手动刷新 Token
   - 查看配额使用情况

---

## 常见问题

### 1. 应用启动失败

**问题**：双击应用没有反应，或者提示端口被占用。

**解决方法**：
- 检查端口 8080 是否被占用：
  ```bash
  # Windows
  netstat -ano | findstr :8080
  
  # macOS/Linux
  lsof -i :8080
  ```
- 如果被占用，关闭占用端口的程序，或者修改配置文件中的端口

### 2. 账号导入失败

**问题**：点击"从 Kiro IDE 导入"后提示找不到文件。

**解决方法**：
- 确保你已经登录过 Kiro IDE
- 检查文件是否存在：
  - macOS/Linux: `~/.aws/sso/cache/kiro-auth-token.json`
  - Windows: `%USERPROFILE%\.aws\sso\cache\kiro-auth-token.json`
- 如果文件不存在，使用"手动添加"方式

### 3. Token 刷新失败

**问题**：日志中显示"Token 刷新失败"。

**解决方法**：
- 检查网络连接
- 检查 Refresh Token 是否正确
- 检查 Refresh Token 是否过期（需要重新登录 Kiro IDE）
- 如果是 IDC 账号，检查 Client ID 和 Client Secret 是否正确

### 4. Claude Desktop 无法连接

**问题**：Claude Desktop 提示"无法连接到服务器"。

**解决方法**：
- 确保 Kiro Gateway 应用正在运行
- 检查配置文件中的 URL 是否正确（`http://127.0.0.1:8080`）
- 检查 API Key 是否正确
- 重启 Claude Desktop

### 5. 请求失败或超时

**问题**：发送消息后长时间没有响应，或者提示请求失败。

**解决方法**：
- 检查所有账号是否都被禁用或过期
- 在"账号管理"页面手动刷新 Token
- 查看"日志"页面的错误信息
- 检查网络连接

### 6. 配额用完了怎么办？

**问题**：所有账号的配额都用完了。

**解决方法**：
- 等待配额重置（通常是每月重置）
- 添加更多账号
- 升级到付费账号（如果有）

### 7. 如何查看 Admin Token？

**问题**：忘记了 Admin Token，无法访问管理 API。

**解决方法**：
- 在"设置"页面可以查看当前的 Admin Token
- 或者查看配置文件：
  - Windows: `%APPDATA%\com.kiro.gateway\.admin_token`
  - macOS: `~/Library/Application Support/com.kiro.gateway/.admin_token`
  - Linux: `~/.local/share/com.kiro.gateway/.admin_token`

### 8. 如何重置所有配置？

**问题**：想要清空所有配置，重新开始。

**解决方法**：
- 删除用户数据目录：
  - Windows: `%APPDATA%\com.kiro.gateway\`
  - macOS: `~/Library/Application Support/com.kiro.gateway/`
  - Linux: `~/.local/share/com.kiro.gateway/`
- 重启应用，会自动创建新的配置

### 9. 数据安全吗？

**问题**：担心 Refresh Token 等敏感信息泄露。

**解决方法**：
- Kiro Gateway 使用 AES-256-GCM 加密存储所有敏感数据
- 加密密钥由机器特定信息（hostname + username）派生
- 加密密钥文件权限设置为仅当前用户可读
- 所有数据存储在本地，不会上传到任何服务器

### 10. 如何更新应用？

**问题**：有新版本发布，如何更新？

**解决方法**：
- 访问 [GitHub Releases](https://github.com/hj01857655/kiro-gateway/releases/latest)
- 下载最新版本的安装包
- 安装新版本（会自动覆盖旧版本）
- 配置文件和账号数据会自动保留

---

## 进阶使用

### 使用 API 直接调用

如果你想通过代码调用 Kiro Gateway，可以使用以下方式：

#### OpenAI 格式

```python
import openai

client = openai.OpenAI(
    base_url="http://127.0.0.1:8080/v1",
    api_key="your-api-key"
)

response = client.chat.completions.create(
    model="gpt-4",
    messages=[
        {"role": "user", "content": "你好"}
    ],
    stream=True
)

for chunk in response:
    if chunk.choices[0].delta.content:
        print(chunk.choices[0].delta.content, end="")
```

#### Anthropic 格式

```python
import anthropic

client = anthropic.Anthropic(
    base_url="http://127.0.0.1:8080",
    api_key="your-api-key"
)

response = client.messages.create(
    model="claude-3-5-sonnet-20241022",
    max_tokens=1024,
    messages=[
        {"role": "user", "content": "你好"}
    ],
    stream=True
)

for event in response:
    if event.type == "content_block_delta":
        print(event.delta.text, end="")
```

### 查看实时日志

```bash
# 获取所有日志
curl -H "x-admin-token: your-admin-token" \
  http://127.0.0.1:8080/admin/logs

# 获取统计数据
curl -H "x-admin-token: your-admin-token" \
  http://127.0.0.1:8080/admin/metrics
```

### 管理账号

```bash
# 获取所有账号
curl -H "x-admin-token: your-admin-token" \
  http://127.0.0.1:8080/admin/accounts

# 添加账号
curl -X POST \
  -H "x-admin-token: your-admin-token" \
  -H "Content-Type: application/json" \
  -d '{"authMethod":"social","refreshToken":"eyJ..."}' \
  http://127.0.0.1:8080/admin/accounts

# 删除账号
curl -X DELETE \
  -H "x-admin-token: your-admin-token" \
  http://127.0.0.1:8080/admin/accounts/account-id
```

---

## 获取帮助

如果遇到问题，可以通过以下方式获取帮助：

1. **查看文档**：[GitHub 仓库](https://github.com/hj01857655/kiro-gateway)
2. **提交 Issue**：[GitHub Issues](https://github.com/hj01857655/kiro-gateway/issues)
3. **加入 QQ 群**：[1081058179（Kiro GateWay交流群）](https://qm.qq.com/q/oQbUA0cxO2)

---

## 总结

恭喜你！现在你已经学会了如何使用 Kiro Gateway。

**快速回顾**：
1. ✅ 下载并安装应用
2. ✅ 添加 Kiro 账号（从 Kiro IDE 导入或手动添加）
3. ✅ 配置客户端（Claude Desktop / Cursor / Continue）
4. ✅ 开始使用！

**下一步**：
- 在"统计"页面查看使用情况
- 在"日志"页面排查问题
- 添加更多账号提高可用性

祝你使用愉快！🎉
