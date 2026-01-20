# 升级指南 - kiro-gateway v0.2.x → v0.3.0

## 重要变更

v0.3.0 版本包含重大安全改进，升级前请仔细阅读本指南。

---

## 🔒 安全变更摘要

1. **Admin API 现在需要认证** - 所有管理接口需要 Admin Token
2. **加密框架已就绪** - 为未来的敏感数据加密做准备
3. **CORS 策略收紧** - 生产环境仅允许 Tauri 协议
4. **Token 刷新优化** - 修复并发刷新问题
5. **错误信息脱敏** - 生产环境不泄露详细错误

---

## 升级步骤

### 1. 备份现有数据 ⚠️

**非常重要！** 升级前务必备份所有数据：

```bash
# Linux/macOS
cp -r ~/.local/share/kiro-gateway ~/.local/share/kiro-gateway.backup.$(date +%Y%m%d)

# Windows (PowerShell)
Copy-Item -Recurse "$env:APPDATA\kiro-gateway" "$env:APPDATA\kiro-gateway.backup.$(Get-Date -Format 'yyyyMMdd')"
```

备份内容包括：
- `accounts.json` - 账号数据
- `api_keys.json` - API Keys
- `metrics.json` - 统计数据
- `.env` - 环境配置（如果有）

---

### 2. 安装新版本

#### 方式 A: 从 Release 下载

1. 访问 [Releases 页面](https://github.com/your-repo/kiro-gateway/releases)
2. 下载 v0.3.0 对应平台的安装包
3. 运行安装程序

#### 方式 B: 从源码构建

```bash
git clone https://github.com/your-repo/kiro-gateway.git
cd kiro-gateway
git checkout v0.3.0

# 安装依赖
npm install

# 构建
npm run tauri:build
```

---

### 3. 首次启动

首次启动 v0.3.0 时，应用会自动：

1. ✅ 生成 Admin Token（保存在 `{app_data_dir}/.admin_token`）
2. ✅ 创建加密密钥（保存在 `{app_data_dir}/.encryption_key`）
3. ✅ 迁移现有配置文件到新位置（如果需要）

**启动日志示例**：
```
[INFO] 应用数据目录: /home/user/.local/share/kiro-gateway
[INFO] 加密管理器已初始化
[INFO] 已生成新的 Admin Token，保存在: /home/user/.local/share/kiro-gateway/.admin_token
[INFO] 从默认路径加载账号: /home/user/.local/share/kiro-gateway/accounts.json
[INFO] 账号加载成功
[INFO] kiro-gateway Axum 服务启动: http://127.0.0.1:8080
```

---

### 4. 验证升级

#### 4.1 检查 Admin Token

```bash
# Linux/macOS
cat ~/.local/share/kiro-gateway/.admin_token

# Windows
type %APPDATA%\kiro-gateway\.admin_token
```

应该看到一个 64 字符的随机字符串。

#### 4.2 测试 Admin API 认证

```bash
# 无 token 访问（应该失败）
curl http://127.0.0.1:8080/admin/accounts
# 预期输出: {"error":{"message":"缺少 Admin Token",...}}

# 使用 token 访问（应该成功）
TOKEN=$(cat ~/.local/share/kiro-gateway/.admin_token)
curl -H "x-admin-token: $TOKEN" http://127.0.0.1:8080/admin/accounts
# 预期输出: {"accounts":[...]}
```

#### 4.3 检查账号数据

打开应用，进入"账号管理"页面，确认：
- ✅ 所有账号都正常显示
- ✅ 可以正常刷新 Token
- ✅ 可以查看配额信息

#### 4.4 测试 API 调用

```bash
# 测试 OpenAI 兼容接口
curl -X POST http://127.0.0.1:8080/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "model": "claude-sonnet-4",
    "messages": [{"role": "user", "content": "Hello"}],
    "stream": true
  }'
```

---

### 5. 配置迁移（可选）

#### 5.1 环境变量

如果你面
2. 找到"Admin Token"部分
3. 点击"显示 Token"

或通过 API：
```bash
TOKEN=$(cat ~/.local/share/kiro-gateway/.admin_token)
curl -H "x-admin-token: $TOKEN" http://127.0.0.1:8080/admin/token
```

### 2. 安全状态检查

应用会在 UI 中显示安全状态指示器：
- 🟢 **安全** - Admin Token 已设置，CORS 配置正确
- 🟡 **警告** - 未配置 API Key 认证
- 🔴 **危险** - 存在安全风险

### 3. 加密功能（未来版本）

v0.3.0 已包含加密框架，但账号数据尚未自动加密。
未来版本将自动加密以下敏感字段：
- `refresh_token`
- `access_token`
- `client_secret`

---

## ⚠️ 已知问题

### 1. Windows 链接器问题

**症状**: Release 构建失败，提示链接器错误

**解决方案**:
```bash
# 方式 A: 使用预编译版本
# 从 Releases 页面下载

# 方式 B: 修复 Visual Studio 构建工具
# 运

**解决方案**:
1. 删除该账号
2. 重新从 Kiro IDE 导入
3. 或手动添加新的 refresh_token

---

## 🔄 回滚到旧版本

如果升级后遇到问题，可以回滚：

### 1. 卸载 v0.3.0

```bash
# Linux
sudon# 通过"添加或删除程序"卸载
```

### 2. 恢复备份数据

```bash
# Linux/macOS
rm -rf ~/.local/share/kiro-gat -Recurse "$env:APPDATA\kiro-gatewa 3. 安装 v0.2.x

从 [Releases](https://github.本并安装。

---

## 📝 配置文件位置变更

| 文件 | v0.2.x | v0.3.0 |
|------|--------|--------|
| 账号数据 | `./data/accounts.json` | `{app_data_dir}/accounts.json` |
| API Keys | `./don` | `{app_data_dir}/api_keys.json` |
| Metrics | `./data/metrics.json` | `{app_data_dir}/metrics.json` |
| Admin Token | ❌ 不存在 | `{app_data_dir}/.admin_token` |
| 加密密钥 | ❌ 不存在 | `{app_data_dir}/.encryption_key` |

**`{app_data_dir}` 位置**:
- Linux: `~/.local/share/kiro-gateway`
- macOS: `~/Library/Application Support/kiro-gateway`
- Windows: `%APPDATA%\kiro-gateway`

---

## 🛡️ 安全建议

### 1. 保护 Admin Token

```bash
# 设置严格的文件权限（Unix）
chmod 600 ~/.local/share/kiro-gateway/.admin_token
chmod 600 ~/.local/share/kiro-gateway/.encryption_key
```

### 2. 启用 API Key 认证

1. 进入"设置" → "API Key 管理"
2. 点击"生成新 Key"
3. 复制并保存 API Key
4. 在客户端配置中使用该 Key

### 3. 定期备份

建议设置自动备份：

```bash
# 添加到 crontab（每天备份）
0 2 * * * cp -r ~/.local/share/kiro-gateway ~/.local/share/kiro-gateway.backup.$(date +\%Y\%m\%d)

# 保留最近 7 天的备份
0 3 * * * find ~/.local/share -name "kiro-gateway.backup.*" -mtime +7 -delete
```

### 4. 监控异常访问

检查日志文件：
```bash
# 查看最近的访问日志
tail -f ~/.local/share/kiro-gateway/logs/app.log

# 搜索失败的认证尝试
grep "无效的 Admin Token" ~/.local/share/kiro-gateway/logs/app.log
```

---

## 📞 获取帮助

### 问题排查

1. **查看日志**:
   - 应用内：设置 → 日志查看
   - 文件：`{app_data_dir}/logs/app.log`

2. **检查配置**:
   ```bash
   curl -H "x-admin-token: $(cat ~/.local/share/kiro-gateway/.admin_token)" \
     http://127.0.0.1:8080/admin/config/server
   ```

3. **健康检查**:
   ```bash
   curl http://127.0.0.1:8080/health
   ```

### 报告问题

如遇到问题，请提供以下信息：

1. 操作系统和版本
2. kiro-gateway 版本（`v0.3.0`）
3. 错误日志（脱敏后）
4. 复现步骤

提交 Issue: https://github.com/your-repo/kiro-gateway/issues

---

## 📚 相关文档

- [SECURITY.md](./SECURITY.md) - 安全修复详情
- [SECURITY_FIXES_SUMMARY.md](./SECURITY_FIXES_SUMMARY.md) - 修复总结
- [README.md](./README.md) - 项目说明

---

**最后更新**: 2026-01-20
**适用版本**: v0.2.x → v0.3.0
**预计升级时间**: 5-10 分钟
