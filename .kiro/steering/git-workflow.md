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

### 增量提交策略（Incremental Commit Strategy）

**核心原则**：完成任何独立的代码变更后，立即提交并推送到远程仓库，避免批量累积。

**强制要求**：
- ✅ 每完成一个独立变更 → 立即提交推送
- ✅ 每修复一个问题 → 立即提交推送
- ✅ 每优化一个模块 → 立即提交推送
- ❌ 禁止累积多个变更后批量推送
- ❌ 禁止等待所有功能完成后统一推送

**风险控制（Risk Management）**：

```bash
# ❌ 高风险：累积多个变更后统一推送
# 完成功能A、B、C后一起推送
git add -A && git commit -m "feat: 添加A、B、C功能" && git push
# 问题：如果发现B有问题，回滚会影响A和C

# ✅ 低风险：增量推送
git add fileA && git commit -m "feat: 添加功能A" && git push origin main
git add fileB && git commit -m "feat: 添加功能B" && git push origin main
git add fileC && git commit -m "feat: 添加功能C" && git push origin main
# 优势：如果B有问题，只回滚B，不影响A和C
```

**增量提交的优势**：

1. **精确回滚（Precise Rollback）**
   - 可以精确回滚到任意一个稳定状态
   - 不会因为回滚一个问题而丢失其他正常的功能

2. **降低冲突风险（Reduce Merge Conflicts）**
   - 及时同步到远程，减少与他人代码的冲突
   - 冲突范围小，容易解决

3. **快速定位问题（Fast Bug Localization）**
   - 通过 `git bisect` 快速定位引入问题的提交
   - 提交粒度小，问题范围明确

4. **代码审查友好（Review-Friendly）**
   - 每个提交职责单一，易于理解和审查
   - 审查者可以逐个提交审查，而不是面对一大堆变更

5. **持续集成保障（CI/CD Assurance）**
   - 每次推送触发 CI 检查，及时发现问题
   - 避免累积大量变更后才发现集成问题

**实践指南**：

```bash
# 场景1：完成一个功能
# 立即推送，不要等其他功能
git add src/feature-a.rs && git commit -m "feat: 实现功能A" && git push origin main

# 场景2：修复一个bug
# 立即推送，不要等其他修复
git add src/bugfix.rs && git commit -m "fix: 修复登录问题" && git push origin main

# 场景3：优化代码
# 立即推送，不要等其他优化
git add src/optimize.rs && git commit -m "refactor: 优化性能" && git push origin main

# 场景4：更新文档
# 立即推送，不要等代码变更
git add README.md && git commit -m "docs: 更新安装说明" && git push origin main
```

**回滚场景示例**：

```bash
# 假设提交历史
A (功能A) → B (功能B有问题) → C (功能C)

# 增量提交：可以精确回滚B
git revert <commit-B>  # 只回滚B，保留A和C

# 批量提交：无法精确回滚
ABC (三个功能一起提交)
git revert <commit-ABC>  # 回滚会丢失A和C
```

**原因**：
1. **精确回滚（Precise Rollback）**：问题定位准确，回滚范围可控
2. **降低风险（Risk Reduction）**：避免因一个问题影响多个功能
3. **提高效率（Efficiency）**：问题排查和修复更快速
4. **保障质量（Quality Assurance）**：每次变更都经过验证

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
