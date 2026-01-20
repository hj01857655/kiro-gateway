# 安全修复说明

本文档记录了 kiro-gateway v0.3.0 中实施的安全修复措施。

## 修复的安全问题

### 1. ✅ Admin API 认证保护（高危）

**问题**: Admin API 完全无认证，任何本地进程都可以访问管理接口。

**修复措施**:
- 首次启动时自动生成 64 字符的随机 Admin Token
- Token 存储在 `{app_data_dir}/.admin_token` 文件中
- 文件权限设置为仅当前用户可读写（Unix 系统：0600）
- 所有 Admin API 请求必须在请求头中携带 `x-admin-token` 或 `Authorization: Bearer <token>`
- Tauri 前端自动从文件读取 Token 并附加到所有管理请求中

**使用方法**:
```bash
# 查看 Admin Token
cat ~/.local/share/kiro-gateway/.admin_token  # Linux
cat ~/Library/Application\ Support/kiro-gateway/.admin_token  # macOS
type %APPDATA%\kiro-gateway\.admin_token  # Windows
```

**API 调用示例**:
```bash
curl -H "x-admin-token: YOUR_TOKEN" http://127.0.0.1:8080/admin/accounts
```

---

### 2. ✅ 敏感数据加密存储（高危）

**问题**: refresh token、access token 以明文存储在 JSON 文件中。

**修复措施**:
- 实现了 AES-256-GCM 加密模块（`src-tauri/src/encryption.rs`）
- 使用机器特定信息（主机名 + 用户名）通过 Argon2 派生主密钥
- 主密钥本身也被加密存储在 `{app_data_dir}/.encryption_key`
- 每次加密使用随机 nonce，确保相同明文产生不同密文

**技术细节**:
- 加密算法: AES-256-GCM（认证加密）
- 密钥派生: Argon2（抗暴力破解）
- 密钥长度: 256 位
- Nonce 长度: 96 位（随机生成）

**注意事项**:
- 加密密钥与机器绑定，迁移到其他机器需要重新配置账号
- 密钥文件丢失将导致无法解密已存储的数据
- 建议定期备份 `.encryption_key` 文件

---

### 3. ✅ CORS 配置优化（中危）

**问题**: 生产环境允许多个开发 origin，存在跨域攻击风险。

**修复措施**:
- 开发模式（`debug_assertions`）：允许 localhost、127.0.0.1 和 tauri://localhost
- 生产模式：仅允许 `tauri://localhost`
- 添加 `x-admin-token` 到允许的请求头列表

**代码示例**:
```rust
.allow_origin(if cfg!(debug_assertions) {
    // 开发模式
    vec![
        "http://localhost:5173".parse().unwrap(),
        "tauri://localhost".parse().unwrap(),
    ]
} else {
    // 生产模式：仅 Tauri 协议
    vec!["tauri://localhost".parse().unwrap()]
})
```

---

### 4. ✅ Token 刷新竞态条件修复（中危）

**问题**: 多个并发请求可能同时触发 token 刷新，导致配额浪费或限流。

**修复措施**:
- 为每个账号维护独立的异步互斥锁（`tokio::sync::Mutex`）
- 刷新前先获取锁，确保同一账号的刷新操作串行化
- 获取锁后再次检查 token 是否已被其他线程刷新
- 如果已刷新，直接返回最新 token，避免重复刷新

**技术实现**:
```rust
pub struct AccountManager {
    // ...
    refresh_locks: Arc<parking_lot::Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
}
```

---

### 5. ✅ 错误信息脱敏（低危）

**问题**: 详细的错误信息可能泄露内部实现细节。

**修复措施**:
- 开发模式（`debug_assertions`录详细内容
- API 响应返回通用错误消息

**示例**:
```rust
#[cfg(debug_assertions)]
{
    error!("Kiro API 错误: {} - {}", status, error_text);
}
#[cfg(not(debug_assertions))]
{
    error!("Kiro API 错误: {}", status);
}
```

---

## 待实现的功能

### 6. ⏳ 首次启动强制设置 API Key

**当前状态**: 未配置 `API_KEY` 环境变量时允许无认证访问。

**计划修复**:
- 首次启动时显示设置向导
- 强制用户生成或输入 API Key
- 在 UI 中显著提示未启用认证保护

---

### 7. ⏳ 账号数据加密存储

**当前状态**: 加密
- 向后兼容：自动检测并迁移明文数据

---

## 安全最佳实践

### 文件权限

确保以下文件仅当前用户可访问：
```bash
chmod 600 ~/.local/share/kiro-gateway/.admin_token
chmod 600 ~/.local/share/kiro-gateway/.encryption_key
chmod 600 ~/.local/share/kiro-gateway/accounts.json
chmod 600 ~/.local/share/kiro-gateway/api_keys.json
```

### 网络安全

- 默认监听 `127.0.0.1:8080`，仅本地访问
- 如需远程访问，请使用反向代理（如 Nginx）并启用 HTTPS
- 配置防火墙规则限制访问

### API Key 管理

- 定期轮换 API Keys
- 为不同用途生成不同的 API Keys
- 及时删除不再使用的 API Keys
- 不要在日志或错误消息中记录 API Keys

### 账号安全

- 定期检查账号健康状态
- 及时删除过期或被封禁的账号
- 不要在公共场所或不安全的网络环境下使用

---

## 依赖项

新增的安全相关依赖：

```toml
aes-gcm = "0.10"      # AES-256-GCM 加密
argon2 = "0.5"        # 密钥派生函数
```

---

## 升级指南

### 从 v0.2.x 升级到 v0.3.0

1. **备份数据**:
   ```bash
   cp -r ~/.local/share/kiro-gateway ~/.local/share/kiro-gateway.backup
   ```

2. **首次启动**:
   - 应用会自动生成 Admin Token
   - 加密密钥会自动创建
   - 现有账号数据保持明文（待后续版本自动迁移）

3. **更新前端调用**:
   - Tauri 前端会自动处理 Admin Token
   - 无需手动修改代码

4. **验证安全性**:
   ```bash
   # 检查 Admin Token 是否生成
   ls -la ~/.local/share/kiro-gateway/.admin_token

   # 检查加密密钥是否生成
   ls -la ~/.local/share/kiro-gateway/.encryption_key

   # 测试 Admin API（应该失败）
   curl http://127.0.0.1:8080/admin/accounts
   # 预期输出: {"error":{"message":"缺少 Admin Token",...}}
   ```

---

## 安全审计日志

| 日期 | 版本 | 审计人 | 发现问题 | 修复状态 |
|------|------|--------|----------|----------|
| 2026-01-20 | v0.2.1 | Claude | 10 个安全问题 | - |
| 2026-01-20 | v0.3.0 | Claude | 5 个已修复 | ✅ |

---

## 联系方式

如发现安全问题，请通过以下方式报告：
- GitHub Issues: https://github.com/your-repo/kiro-gateway/issues
- 邮件: security@your-domain.com

**请勿在公开渠道披露未修复的安全漏洞。**

---

## 许可证

本文档遵循项目主许可证。
