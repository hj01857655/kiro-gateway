# 沟通风格规范

## 核心原则

**双模式沟通策略（Dual-Mode Communication Strategy）**

根据不同场景采用不同的表达方式，确保沟通效率和文档质量。

---

## 场景 1：需求沟通（Requirement Discussion）

**适用场景**：
- 用户提出需求或问题
- 讨论实现方案
- 解释技术概念
- 日常对话交流

**表达方式**：
- ✅ 使用直白的口语化表达
- ✅ 用大白话解释复杂概念
- ✅ 举生活化的例子
- ✅ 避免过度使用专业术语
- ✅ 确保用户能快速理解

**示例**：

```markdown
# ❌ 错误：过于专业
"我们需要实现原子提交原则（Atomic Commit Principle），确保每个提交都是一个独立的、可回滚的变更单元。"

# ✅ 正确：直白易懂
"就是完成一个功能就推一个，别攒着。这样如果某个功能有问题，回滚的时候不会影响其他功能。"
```

---

## 场景 2：规则文档（Rule Documentation）

**适用场景**：
- 编写 `.kiro/steering/` 规则文件
- 编写技术文档
- 编写 API 规范
- 编写架构设计文档

**表达方式**：
- ✅ 使用专业术语和规范表述
- ✅ 确保表达严谨、准确
- ✅ 遵循行业标准命名
- ✅ 提供完整的技术细节
- ✅ 使用标准化的文档结构

**示例**：

```markdown
# ✅ 正确：专业规范
## 增量提交策略（Incremental Commit Strategy）

**核心原则**：完成任何独立的代码变更后，立即提交并推送到远程仓库，避免批量累积。

**风险控制（Risk Management）**：
- 精确回滚（Precise Rollback）
- 降低冲突风险（Reduce Merge Conflicts）
- 快速定位问题（Fast Bug Localization）
```

---

## 场景 3：代码注释（Code Comments）

**适用场景**：
- Rust/TypeScript/JavaScript 代码注释
- 函数说明
- 复杂逻辑解释

**表达方式**：
- ✅ 使用中文注释
- ✅ 清晰说明逻辑和意图
- ✅ 解释"为什么"而不只是"做什么"
- ✅ 标注关键步骤

**示例**：

```rust
// ✅ 正确：中文注释，说明意图
/// 刷新账号的 Access Token
/// 
/// 根据账号类型（Social/IDC）选择不同的刷新端点
/// 刷新成功后更新内存和配置文件中的 Token
async fn refresh_token(account: &mut Account) -> Result<()> {
    // 检查是否需要刷新（提前 5 分钟）
    if !should_refresh(&account.expires_at) {
        return Ok(());
    }
    
    // 根据账号类型选择刷新方式
    match account.auth_method.as_str() {
        "social" => refresh_social_token(account).await,
        "IdC" => refresh_idc_token(account).await,
        _ => Err(AppError::InvalidAuthMethod),
    }
}
```

---

## 场景 4：变量/函数命名（Naming Conventions）

**适用场景**：
- 变量命名
- 函数命名
- 类型命名
- 文件命名

**表达方式**：
- ✅ 使用英文命名
- ✅ 遵循语言规范（Rust: snake_case, TypeScript: camelCase）
- ✅ 名称要有描述性
- ✅ 避免缩写（除非是通用缩写）

**示例**：

```rust
// ✅ 正确：英文命名，遵循 Rust 规范
async fn refresh_social_token(account: &mut Account) -> Result<()> {
    let refresh_endpoint = "https://prod.us-east-1.auth.desktop.kiro.dev/refreshToken";
    let request_body = json!({ "refreshToken": account.refresh_token });
    // ...
}

// ❌ 错误：中文命名或不规范
async fn 刷新社交账号Token(账号: &mut Account) -> Result<()> {
    let 刷新端点 = "...";
    // ...
}
```

---

## 实践指南

### 判断使用哪种风格

**问自己三个问题**：

1. **这是在跟用户聊天吗？** → 用大白话
2. **这是在写规则文档吗？** → 用专业术语
3. **这是在写代码吗？** → 中文注释 + 英文命名

### 快速切换技巧

**从口语转专业术语**：
- "完成一个推一个" → "增量提交策略"
- "别攒着" → "避免批量累积"
- "方便回滚" → "精确回滚（Precise Rollback）"
- "不会影响其他功能" → "降低风险（Risk Reduction）"

**从专业术语转口语**：
- "原子提交原则" → "每次只提交一个独立的改动"
- "幂等性" → "重复执行结果一样"
- "依赖注入" → "把需要的东西从外面传进来"
- "解耦" → "让模块之间不要互相依赖"

---

## 常见错误

### ❌ 错误 1：跟用户聊天时过于专业

```markdown
用户："这个功能怎么实现？"
AI："我们需要实现依赖注入模式（Dependency Injection Pattern），通过控制反转（IoC）容器管理对象生命周期..."

# 问题：用户听不懂，沟通效率低
```

### ✅ 正确做法

```markdown
用户："这个功能怎么实现？"
AI："就是把需要的东西从外面传进来，而不是在函数里直接创建。这样以后想换实现方式就很方便。"

# 优势：用户能快速理解
```

### ❌ 错误 2：写规则文档时过于口语化

```markdown
# Git 规范

就是完成一个功能就推一个，别攒着，不然回滚的时候麻烦。

# 问题：不够严谨，缺乏专业性
```

### ✅ 正确做法

```markdown
# Git 工作流规范

## 增量提交策略（Incremental Commit Strategy）

**核心原则**：完成任何独立的代码变更后，立即提交并推送到远程仓库，避免批量累积。

**风险控制（Risk Management）**：
- 精确回滚（Precise Rollback）：可以精确回滚到任意一个稳定状态
- 降低风险（Risk Reduction）：避免因一个问题影响多个功能

# 优势：严谨规范，便于参考
```

---

## 总结

| 场景 | 表达方式 | 目的 |
|------|---------|------|
| 需求沟通 | 直白口语 | 快速理解 |
| 规则文档 | 专业术语 | 严谨规范 |
| 代码注释 | 中文说明 | 清晰易懂 |
| 变量命名 | 英文规范 | 遵循标准 |

**记住**：
- 跟人聊天 → 说人话
- 写文档 → 说专业话
- 写代码 → 中文注释 + 英文命名

这样既能高效沟通，又能保证文档和代码的专业性。
