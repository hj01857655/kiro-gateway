---
inclusion: manual
---

# Kiro IDE 源码参考

## 源码位置

### 本地安装路径

**Windows**:
```
C:\Users\{用户名}\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\
```

**关键文件**:
- `dist/extension.js` - 主扩展代码（39.58 MB，已压缩）
- `node_modules/` - NPM 依赖包（包含 MCP SDK 等）

### 源码分析项目

**项目路径**: `E:\VSCodeSpace\Kiro\kiro-agent-source-analysis`

用于存放 Kiro IDE 源码的**分析文档**，而不是直接复制源码。

---

## 访问方式

### 访问 Kiro IDE 源码（工作区外）

由于 Kiro IDE 源码在工作区外，必须使用 PowerShell 访问：

```powershell
# 搜索关键词（推荐用于大文件）
Select-String -Path "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js" -Pattern "关键词" -Context 5,5

# 读取主扩展文件（39.58 MB，较大，慎用）
Get-Content "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js" -Raw

# 访问 node_modules 依赖包
Get-Content "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\node_modules\@modelcontextprotocol\sdk\dist\esm\types.js" -Raw

# 列出 node_modules 中的包
dir "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\node_modules"
```

### 访问源码分析项目（工作区外）

```powershell
# 读取分析文档
Get-Content "E:\VSCodeSpace\Kiro\kiro-agent-source-analysis\MCP-Validation-Analysis.md" -Raw

# 列出所有分析文档
Get-ChildItem "E:\VSCodeSpace\Kiro\kiro-agent-source-analysis" -Filter "*.md"
```

---

## 关键函数和位置

### CONTEXT TRANSFER 位置

**文件**: `dist/extension.js`  
**行号**: 约 836970

这是 Kiro IDE 生成会话总结的位置，当会话过长时会自动触发。

### 消息格式转换

**函数**: `convertToGenerateAssistantMessages`  
**位置**: extension.js 约 702988 行  
**功能**: 转换消息格式为 Kiro API 格式

### API 调用命令

**函数**: `GenerateAssistantResponseCommand`  
**功能**: 调用 Kiro API 的主命令

### 文件访问安全

**UnauthorizedFileAccessError** (行 846846-846850):
```javascript
var UnauthorizedFileAccessError = class extends KiroError {
  constructor(filePath, reason = "outside-workspace") {
    const message = reason === "symlink" 
      ? `Symlink access denied in untrusted workspace: ${filePath}` 
      : `Access denied: File access is restricted to workspace. Attempted path: ${filePath}`;
    super(message);
  }
}
```

**isPathAllowed 函数** (行 846877-846887):
- 允许访问工作区目录
- 允许访问 `.kiro` 目录
- 允许访问多工作区文件夹
- 禁止访问工作区外的任意路径
- 禁止访问符号链接（在不受信任的工作区中）

---

## 已验证的 API 行为

### 响应格式（已确认）

Kiro API 返回**扁平 JSON 对象**，不是嵌套结构：

```json
// ✅ 实际格式
{"content": "文本内容"}
{"content": "代码", "language": "python"}
{"toolUseId": "xxx", "name": "tool", "input": {...}}
{"unit": "credit", "usage": 0.003}
{"contextUsagePercentage": 0.022}

// ❌ 错误理解（嵌套格式）
{"assistantResponseEvent": {"content": "..."}}
{"codeEvent": {"content": "...", "language": "..."}}
```

### 请求格式（已确认）

```json
{
  "conversationState": {
    "conversationId": "uuid",
    "chatTriggerType": "MANUAL",
    "currentMessage": {
      "userInputMessage": {
        "content": "字符串",  // 不是数组
        "modelId": "auto",    // 不需要 qdev:: 前缀
        "origin": "AI_EDITOR"
      }
    },
    "history": []
  }
}
```

---

## 源码分析规范

### 创建分析文档

**文件命名**: `功能名-分析.md` 或 `功能名-v{版本号}.md`

**文档结构**:
```markdown
# 功能名称分析

## 版本信息
- Kiro IDE 版本：v0.8.140
- 分析日期：2026-01-18

## 功能概述
简要描述该功能的作用和使用场景

## 实现原理
详细分析实现逻辑、关键算法、数据流程

## 关键代码片段
只摘录**核心逻辑**（10-30行），添加详细注释说明

## 参考位置
- 文件：dist/extension.js
- 行号：约 12345-12400
- 相关函数：functionName()
```

### 代码片段原则

- ✅ 只摘录关键逻辑（10-30行核心代码）
- ✅ 添加详细注释，解释每个步骤的作用
- ✅ 简化变量名，提高可读性
- ✅ 标注来源（文件路径、行号）
- ❌ 不要直接复制大段源码
- ❌ 不要包含无关的辅助代码
- ❌ 不要保留混淆后的变量名

---

## 已分析的功能

### 0.8.140 版本
- **MCP Schema 验证**: `MCP-Schema-Validation-v0.8.140.md`
- **MCP 工具数量警告**: `MCP-Tool-Count-Warning-v0.8.140.md`
- **.kiroignore 支持**: `Kiroignore-Support-v0.8.140.md`

### 早期版本
- **MCP 验证**: `MCP-Validation-Analysis.md`

---

## 模型可用性测试（2026-01-15）

测试结果（可能受 Anthropic 服务状态影响）：

**可用模型**：
- ✅ `auto` - 自动选择
- ✅ `claude-sonnet-4` - Sonnet 4
- ✅ `claude-sonnet-4.5` - Sonnet 4.5

**不可用模型**（可能是临时故障）：
- ❌ `claude-haiku-4.5` - 返回空错误
- ❌ `claude-opus-4.5` - INVALID_MODEL_ID

**注意**：
- 模型可用性可能取决于账号类型（免费/付费）或服务状态
- 建议定期重新测试模型可用性

---

## 更新流程

当发现新的源码信息时：

1. **立即记录** - 在本文件中添加新发现
2. **更新文档** - 同步到 `docs/kiro-gate/kiro-api.md`
3. **更新规则** - 同步到 `.kiro/steering/kiro-api.md`
4. **验证代码** - 根据源码验证当前实现是否正确

---

## 参考资料

### GitHub 参考

**AWS Toolkit for VS Code**: [aws/aws-toolkit-vscode](https://github.com/aws/aws-toolkit-vscode)
- 路径: `packages/core/src/codewhispererChat/clients/chat/v0/chat.ts`
- 包含类似的 API 调用逻辑

### 相关项目

- **kiro-gateway**: `E:\VSCodeSpace\Kiro\kiro-gateway` (Rust + Axum)
- **源码分析项目**: `E:\VSCodeSpace\Kiro\kiro-agent-source-analysis`
