# Kiro Gateway 发布指南

## 发布流程

### 1. 构建应用

```bash
npm run tauri:build
```

构建产物位于 `src-tauri/target/release/bundle/`：
- Windows: `.msi` 和 `.exe` 安装包
- macOS: `.dmg` 和 `.app` 应用包
- Linux: `.deb`, `.rpm`, `.AppImage` 等

### 2. 创建 GitHub Release

**重要**: 公开仓库 `kiro-gateway` 只用于发布 Release，不推送源码。

#### 步骤：

1. 在 GitHub 上打开公开仓库：https://github.com/hj01857655/kiro-gateway

2. 点击 "Releases" → "Create a new release"

3. 填写 Release 信息：
   - **Tag version**: `v0.1.0` (遵循语义化版本)
   - **Release title**: `Kiro Gateway v0.1.0`
   - **Description**: 参考下方模板

4. 上传构建产物：
   - 将 `src-tauri/target/release/bundle/` 中的安装包拖拽到 Release 页面
   - 建议上传：
     - Windows: `.msi` (推荐) 和 `.exe`
     - macOS: `.dmg` (推荐) 和 `.app.tar.gz`
     - Linux: `.AppImage` (推荐), `.deb`, `.rpm`

5. 发布 Release

### Release 描述模板

```markdown
# Kiro Gateway v0.1.0

Kiro API 网关桌面应用，提供 OpenAI/Anthropic 兼容接口。

## ✨ 主要功能

- ✅ OpenAI `/v1/chat/completions` API 兼容
- ✅ Anthropic `/v1/messages` API 兼容
- ✅ 多账号管理和智能轮询
- ✅ Token 自动刷新
- ✅ 账号健康检查
- ✅ 智能 Token 分配（基于成功率、新鲜度、负载均衡）
- ✅ 实时统计监控
- ✅ 日志查看和搜索
- ✅ 现代化 UI 界面

## 📦 安装

### Windows
下载 `.msi` 安装包，双击安装。

### macOS
下载 `.dmg` 文件，拖拽到 Applications 文件夹。

### Linux
下载 `.AppImage` 文件，添加执行权限后运行：
```bash
chmod +x kiro-gateway_0.1.0_amd64.AppImage
./kiro-gateway_0.1.0_amd64.AppImage
```

## 🚀 快速开始

1. 启动应用
2. 在"账号管理"页面添加 Kiro 账号
3. 应用会自动启动 HTTP 服务（默认 `http://127.0.0.1:8080`）
4. 使用 OpenAI 或 Anthropic SDK 连接到本地服务

## 📝 更新日志

### v0.1.0 (2026-01-17)

**新功能**:
- 完整的账号管理系统
- 智能 Token 分配和健康检查
- 统计监控和日志系统
- Mantine UI 现代化界面

**技术栈**:
- 后端: Rust + Axum + Tokio
- 前端: React 19 + TypeScript + Mantine v7
- 桌面框架: Tauri 2.0

## 🔗 相关链接

- 源码仓库（私有）: https://github.com/hj01857655/kiro-gateway_dev
- 问题反馈: https://github.com/hj01857655/kiro-gateway/issues
- 文档: 查看仓库 `docs/` 目录

## ⚠️ 注意事项

- 本应用仅用于个人学习和研究
- 请遵守 Kiro 服务条款
- 不要滥用 API 配额
```

## 版本号规范

遵循语义化版本 (Semantic Versioning):
- **主版本号 (Major)**: 不兼容的 API 变更
- **次版本号 (Minor)**: 向下兼容的功能新增
- **修订号 (Patch)**: 向下兼容的问题修复

示例：
- `v0.1.0` - 初始版本
- `v0.2.0` - 新增功能
- `v0.2.1` - Bug 修复
- `v1.0.0` - 正式版本

## 自动化发布（可选）

可以使用 GitHub Actions 自动构建和发布：

1. 在私有仓库创建 `.github/workflows/release.yml`
2. 推送 tag 时自动触发构建
3. 自动创建 Release 并上传构建产物

详见 `docs/github-actions.md`（待创建）

## 注意事项

- ✅ 公开仓库只用于 Release，不推送源码
- ✅ 所有开发在私有仓库 `kiro-gateway_dev` 进行
- ✅ Release 前确保所有测试通过
- ✅ Release 描述要清晰明了
- ✅ 上传所有平台的安装包
