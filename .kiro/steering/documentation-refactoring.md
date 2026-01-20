# 文档重构规范

## 任务目标

对 `.kiro/steering/` 目录下的所有规则文件进行**系统性重构**，包括：
1. **去重（Deduplication）** - 消除跨文件和文件内的重复内容
2. **重组（Reorganization）** - 优化文件结构和逻辑顺序
3. **标准化（Standardization）** - 统一格式和术语

## 具体要求

### 1. 跨文件去重分析（Cross-file Deduplication Analysis）

**目标**：消除多个文件中的重复内容，建立清晰的引用关系

**执行步骤**：
- 识别多个文件中重复出现的规则、示例、说明
- 将通用规则提取到更高层级的文件
- 建立文件间的引用关系，避免内容冗余
- 合并功能相似或重叠的文件

**示例**：
```markdown
# ❌ 错误：多个文件重复相同内容
# file1.md
## Git 提交规范
- feat: 新功能
- fix: 修复

# file2.md
## Git 提交规范
- feat: 新功能
- fix: 修复

# ✅ 正确：提取到通用文件并引用
# git-workflow.md
## Git 提交规范
- feat: 新功能
- fix: 修复

# file1.md
参考 [Git 工作流规范](./git-workflow.md)

# file2.md
参考 [Git 工作流规范](./git-workflow.md)
```

### 2. 文件内容重构（Intra-file Refactoring）

**目标**：优化单个文件的内部结构，提高可读性和信息密度

**执行步骤**：
- 检查每个文件内部的重复段落
- 优化章节顺序，遵循"概述→规则→示例→注意事项"的逻辑
- 统一标题层级（H1/H2/H3）的使用规范
- 精简冗长描述，提高信息密度

**标准文件结构**：
```markdown
# 文件标题（H1）

## 概述（H2）
简要说明文件的目的和适用范围

## 核心规则（H2）

### 规则 1（H3）
具体规则说明

### 规则 2（H3）
具体规则说明

## 示例（H2）

### 示例 1（H3）
代码示例和说明

### 示例 2（H3）
代码示例和说明

## 注意事项（H2）
- 注意点 1
- 注意点 2

## 相关文档（H2）
- [相关文档 1](./file1.md)
- [相关文档 2](./file2.md)
```

### 3. 文件组织优化（File Organization Optimization）

**目标**：建立清晰的文件分类和导航体系

**执行步骤**：
- 按功能域对文件进行分类（如：开发流程、API规范、架构设计）
- 建议文件重命名方案，使命名更具描述性
- 创建索引文件（如 `README.md`）提供导航
- 定义文件优先级和加载顺序

**推荐分类结构**：
```
.kiro/steering/
├── README.md                    # 索引文件
├── 01-workflow/                 # 开发流程
│   ├── git-workflow.md
│   └── release-process.md
├── 02-architecture/             # 架构设计
│   ├── project-structure.md
│   └── tauri-production.md
├── 03-api/                      # API 规范
│   ├── kiro-api.md
│   ├── format-conversion.md
│   └── error-handling.md
├── 04-implementation/           # 实现细节
│   ├── account-management.md
│   ├── auth-methods.md
│   └── token-refresh.md
└── 05-reference/                # 参考资料
    ├── kiro-ide-source.md
    └── kiro-source-reference.md
```

**文件命名规范**：
- 使用 kebab-case（小写+连字符）
- 名称应清晰描述文件内容
- 避免缩写，使用完整单词
- 可添加数字前缀表示优先级

### 4. 内容标准化（Content Standardization）

**目标**：统一文档格式和术语，提高一致性

**执行步骤**：
- 统一术语表（Glossary）- 确保关键术语一致
- 统一代码示例格式（语言标注、注释风格）
- 统一列表格式（有序/无序、缩进规则）
- 统一强调标记（✅❌⚠️等符号的使用规范）

**术语标准化**：
```markdown
# ✅ 正确：统一术语
- 账号（Account）
- 刷新令牌（Refresh Token）
- 访问令牌（Access Token）

# ❌ 错误：术语不一致
- 账号 / 帐号 / Account
- 刷新 Token / refreshToken / refresh_token
- Access Token / 访问令牌 / 访问 token
```

**代码示例标准化**：
```markdown
# ✅ 正确：标注语言、添加注释
\`\`\`bash
# 提交并推送代码
git add -A && git commit -m "feat: 新功能" && git push origin main
\`\`\`

# ❌ 错误：无语言标注、无注释
\`\`\`
git add -A && git commit -m "feat: 新功能" && git push origin main
\`\`\`
```

**强调标记标准化**：
- ✅ 表示正确做法、推荐方案
- ❌ 表示错误做法、禁止操作
- ⚠️ 表示警告、需要注意的事项
- 📋 表示清单、检查项
- 🚀 表示最佳实践、优化建议

## 输出要求

重构完成后，需要提供：

### 1. 重复内容分析报告
```markdown
## 重复内容分析

### 跨文件重复
- **Git 提交规范**
  - 出现位置：git.md, git-rules.md, git-workflow.md
  - 建议：合并到 git-workflow.md

- **账号类型说明**
  - 出现位置：account-management.md, auth-methods.md
  - 建议：保留 account-management.md，auth-methods.md 引用

### 文件内重复
- **account-management.md**
  - 第 10-20 行与第 50-60 行重复
  - 建议：删除第 50-60 行
```

### 2. 文件重组方案
```markdown
## 文件重组方案

### 需要合并的文件
- git.md + git-rules.md → git-workflow.md

### 需要拆分的文件
- kiro-gateway.md → 拆分为：
  - project-overview.md（项目概述）
  - repository-management.md（仓库管理）
  - deployment.md（部署流程）

### 需要重命名的文件
- rust-rules.md → rust-coding-standards.md
- sse-events.md → sse-event-types.md
```

### 3. 内容迁移计划
```markdown
## 内容迁移计划

### 第一阶段：去重
1. 合并 git.md 和 git-rules.md 到 git-workflow.md
2. 删除 account-management.md 中的重复段落
3. 将通用规则提取到 global.md

### 第二阶段：重组
1. 创建分类目录结构
2. 移动文件到对应目录
3. 更新文件间的引用链接

### 第三阶段：标准化
1. 统一术语表
2. 统一代码示例格式
3. 统一强调标记
```

### 4. 重构后的文件清单
```markdown
## 重构后文件清单

### 开发流程（Workflow）
- git-workflow.md - Git 工作流规范
- release-process.md - 发布流程

### 架构设计（Architecture）
- project-structure.md - 项目结构
- tauri-production.md - Tauri 生产环境

### API 规范（API Specification）
- kiro-api.md - Kiro API 规范
- format-conversion.md - 格式转换
- error-handling.md - 错误处理

### 实现细节（Implementation）
- account-management.md - 账号管理
- auth-methods.md - 认证方式
- token-refresh.md - Token 刷新

### 参考资料（Reference）
- kiro-ide-source.md - Kiro IDE 源码
- kiro-source-reference.md - 源码参考
```

## 约束条件

重构过程中必须遵守：

1. **向后兼容性**
   - 保持现有引用路径有效
   - 如需移动文件，创建重定向或说明

2. **版本控制**
   - 优先保留更详细、更新的版本
   - 删除过时或不准确的内容

3. **适用场景**
   - 确保每个规则都有明确的适用场景
   - 避免过于抽象或过于具体

4. **精简目标**
   - 重构后总文件数应减少 20-30%
   - 总字数应减少 15-25%
   - 信息密度应提高 30-40%

## 执行流程

### 阶段 1：分析（Analysis）
1. 扫描所有文件，识别重复内容
2. 分析文件间的依赖关系
3. 评估每个文件的质量和完整性
4. 生成重复内容分析报告

### 阶段 2：规划（Planning）
1. 制定文件重组方案
2. 设计新的目录结构
3. 规划内容迁移路径
4. 定义标准化规范

### 阶段 3：执行（Execution）
1. 创建新的目录结构
2. 合并和拆分文件
3. 迁移和重组内容
4. 应用标准化规范

### 阶段 4：验证（Validation）
1. 检查所有引用链接是否有效
2. 验证内容完整性和准确性
3. 确认符合约束条件
4. 生成最终文件清单

### 阶段 5：提交（Commit）
1. 提交重构后的文件
2. 更新相关文档
3. 同步到公开仓库（如需要）
4. 通知相关人员

## 最佳实践

### 1. 渐进式重构
- 不要一次性重构所有文件
- 按功能域逐步进行
- 每次重构后立即提交

### 2. 保留历史
- 重要的历史版本保留备份
- 在 commit message 中说明重构原因
- 必要时创建 CHANGELOG

### 3. 文档审查
- 重构后进行同行审查
- 确保术语和格式一致
- 验证示例代码可执行

### 4. 持续优化
- 定期检查文档质量
- 根据反馈持续改进
- 保持文档与代码同步

## 相关工具

### 文档分析工具
- `grep` - 搜索重复内容
- `diff` - 比较文件差异
- `wc` - 统计字数和行数

### 文档格式化工具
- `prettier` - Markdown 格式化
- `markdownlint` - Markdown 语法检查
- `vale` - 文档风格检查

### 示例命令
```bash
# 查找重复的标题
grep -r "^## " .kiro/steering/ | sort | uniq -d

# 统计文件数量和总字数
find .kiro/steering/ -name "*.md" | wc -l
find .kiro/steering/ -name "*.md" -exec wc -w {} + | tail -1

# 检查 Markdown 格式
npx markdownlint .kiro/steering/**/*.md
```
