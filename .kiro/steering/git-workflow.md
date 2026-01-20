# Git 工作流规范

## 提交和推送

### 一步完成原则

**禁止分多条命令执行 git 操作**，必须用一条命令完成：

```bash
# ✅ 正确：一条命令完成
git add -A && git commit -m "message" && git push origin main

# ❌ 错误：分三条命令
git add -A
git commit -m "message"
git push origin main
```

### 提交信息格式

```
<type>: <description>

[可选正文]
```

**Type 类型**：
- `feat` - 新功能
- `fix` - 修复 bug
- `docs` - 文档更新
- `style` - 代码格式（不影响逻辑）
- `refactor` - 重构
- `perf` - 性能优化
- `test` - 测试相关
- `chore` - 构建/工具变更

### 示例

```bash
# 新功能
git add -A && git commit -m "feat: 添加账号导入去重功能" && git push origin main

# 修复 bug
git add -A && git commit -m "fix: 修复 Token 刷新失败问题" && git push origin main

# 样式优化
git add -A && git commit -m "style: 优化主题样式和动画效果" && git push origin main

# 版本更新
git add -A && git commit -m "chore: bump version to 0.3.0" && git push origin main
```

## 分支管理

- `main` - 主分支，所有开发直接提交
- `feature/xxx` - 功能分支（可选）
- `fix/xxx` - 修复分支（可选）

## 持续集成原则

### 及时推送规范

**每次完成独立的功能模块、修复或优化后，必须立即提交并推送到远程仓库。**

**强制要求**：
- ✅ 完成一个功能点 → 立即推送
- ✅ 修复一个 bug → 立即推送
- ✅ 完成一次重构 → 立即推送
- ✅ 优化一个模块 → 立即推送
- ❌ 禁止累积多个改动后再统一推送
- ❌ 禁止"等会儿再推"或"一起推"

**原因**：
1. 保证代码变更的原子性和可追溯性
2. 降低代码丢失风险
3. 便于回滚到任意稳定状态
4. 提高团队协作效率

### 推送前检查清单

- [ ] 代码已通过编译（`npm run build` 或 `cargo build`）
- [ ] 功能已完成且可独立运行
- [ ] 提交信息清晰描述改动内容
- [ ] 版本号已更新（如有必要）

## 注意事项

- 提交前确保代码已编译通过
- 提交信息要清晰描述改动内容
- 大的功能改动建议在提交信息中添加详细说明
- **每次改动完成后立即推送，不要拖延**
