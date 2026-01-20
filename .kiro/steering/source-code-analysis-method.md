# 源码分析方法规范

## 核心原则

**按行号精确读取源码**是最正确、最准确的分析方式。

❌ **错误做法**：只搜索关键词，看搜索结果的片段
✅ **正确做法**：先搜索定位行号，再按行号读取完整代码

---

## 正确的分析流程

### 第一步：搜索关键词定位行号

```powershell
Select-String -Path "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js" -Pattern "function saveSession" -Context 0,3
```

**输出示例**：
```
> extension.js:569994:        const filePath = getSessionFilePath(session.sessionId, session.workspaceDirectory);
```

**得到行号**：569994

### 第二步：按行号读取完整代码 ⭐ 最重要

```powershell
$lines = Get-Content "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js"
$lines[569994..570040] -join "`n"
```

**这才是真正的分析**，可以看到：
- ✅ 完整的函数实现
- ✅ 所有的逻辑步骤
- ✅ 错误处理方式
- ✅ 边界条件
- ✅ 上下文关系

### 第三步：分析代码逻辑

理解代码的：
- 输入输出
- 关键步骤
- 边界条件
- 错误处理
- 设计思路

### 第四步：记录到文档

提取核心逻辑，添加注释，记录到分析文档。

---

## 实际示例

### 示例 1：分析会话管理的 load 函数

**第一步：搜索定位**
```powershell
Select-String -Path "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js" -Pattern "load\(sessionId" -Context 0,3
```

**第二步：按行号读取**
```powershell
$lines = Get-Content "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js"
$lines[569973..569990] -join "`n"
```

**输出**（完整的 load 函数）：
```javascript
load(sessionId, workspaceDir) {
  try {
    const sessionFile = getSessionFilePath(sessionId, workspaceDir);
    if (!fs24.existsSync(sessionFile)) {
      throw new Error(`Session file ${sessionFile} does not exist`);
    }
    const session = JSON.parse(fs24.readFileSync(sessionFile, "utf8"));
    session.sessionId = sessionId;
    return session;
  } catch (e11) {
    console.log(`Error loading session: ${e11}`);
    return {
      history: [],
      title: "New Session",
      workspaceDirectory: "",
      sessionId
    };
  }
}
```

**第三步：分析逻辑**
- 检查文件是否存在
- 读取并解析 JSON
- 确保 sessionId 字段存在
- 失败时返回默认结构

### 示例 2：分析会话管理的 save 函数

**第一步：搜索定位**
```powershell
Select-String -Path "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js" -Pattern "save\(session\)" -Context 0,3
```

**第二步：按行号读取**
```powershell
$lines = Get-Content "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js"
$lines[569994..570040] -join "`n"
```

**输出**（完整的 save 函数，约 47 行）：
```javascript
save(session) {
  // 1. 保存完整会话到 {sessionId}.json
  const filePath = getSessionFilePath(session.sessionId, session.workspaceDirectory);
  fs24.writeFileSync(filePath, JSON.stringify(session, void 0, 2));
  
  // 2. 更新 sessions.json 列表
  const sessionsListFilePath = getSessionsListPath(session.workspaceDirectory);
  try {
    const rawSessionsList = fs24.readFileSync(sessionsListFilePath, "utf-8");
    let sessionsList;
    
    // 解析现有列表
    try {
      sessionsList = JSON.parse(rawSessionsList);
    } catch (e11) {
      if (rawSessionsList.trim() === "") {
        fs24.writeFileSync(sessionsListFilePath, JSON.stringify([]));
        sessionsList = [];
      } else {
        throw e11;
      }
    }
    
    // 查找是否已存在
    let found = false;
    for (const sessionInfo of sessionsList) {
      if (sessionInfo.sessionId === session.sessionId) {
        // 更新现有会话元数据
        sessionInfo.title = session.title;
        sessionInfo.workspaceDirectory = session.workspaceDirectory;
        sessionInfo.hidden = session.hidden;
        found = true;
        break;
      }
    }
    
    // 如果不存在，添加新会话元数据
    if (!found) {
      const sessionInfo = {
        sessionId: session.sessionId,
        title: session.title,
        dateCreated: String(Date.now()),
        workspaceDirectory: session.workspaceDirectory,
        hidden: session.hidden
      };
      sessionsList.push(sessionInfo);
    }
    
    // 保存更新后的列表
    fs24.writeFileSync(sessionsListFilePath, JSON.stringify(sessionsList, void 0, 2));
  } catch (error3) {
    // 错误处理
    if (error3 instanceof SyntaxError) {
      throw new Error(`JSON formatting error in sessions.json`);
    }
    throw new Error(`Validation error in sessions.json: ${error3}`);
  }
}
```

**第三步：分析逻辑**
- 先保存完整会话，再更新列表
- 使用 `JSON.stringify(session, void 0, 2)` 格式化（缩进 2 空格）
- 如果会话已存在，更新元数据；否则添加新元数据
- 完善的错误处理（JSON 解析错误、验证错误）

---

## 为什么这种方式最好？

### ✅ 优势

1. **看到完整逻辑** - 不会遗漏任何细节
2. **理解上下文** - 知道函数的输入输出和调用关系
3. **发现边界条件** - 看到所有的 if/else 分支
4. **学习错误处理** - 看到如何处理异常情况
5. **准确实现** - 可以完全按照源码实现

### ❌ 只搜索关键词的问题

1. **只看到片段** - 看不到完整逻辑
2. **遗漏细节** - 可能错过重要的边界条件
3. **理解不深** - 不知道为什么这样实现
4. **实现不准** - 容易漏掉关键步骤
5. **太假了** - 这不叫分析，只是搜索

---

## 常用命令模板

### 读取指定行范围

```powershell
$lines = Get-Content "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js"
$lines[起始行..结束行] -join "`n"
```

### 读取单行

```powershell
$lines = Get-Content "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js"
$lines[行号]
```

### 读取多个不连续的行范围

```powershell
$lines = Get-Content "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js"
$lines[100..120] -join "`n"
$lines[200..250] -join "`n"
```

---

## 分析文档模板

```markdown
## 函数名 - 功能描述 (行 起始-结束)

### 源码位置
- 文件：extension.js
- 行号：569973-569990
- 函数：load(sessionId, workspaceDir)

### 完整代码
\`\`\`javascript
// 粘贴按行号读取的完整代码
\`\`\`

### 逻辑分析
1. 第一步：做什么
2. 第二步：做什么
3. 第三步：做什么

### 关键点
- 关键点 1
- 关键点 2
- 关键点 3

### 错误处理
- 如何处理文件不存在
- 如何处理 JSON 解析失败
- 返回什么默认值

### Rust 实现建议
\`\`\`rust
// 对应的 Rust 实现
\`\`\`
```

---

## 总结

**记住**：
- ❌ 不要只搜索关键词就说分析完了
- ✅ 必须按行号读取完整代码
- ✅ 理解每一行的作用
- ✅ 记录关键逻辑和边界条件
- ✅ 这才是真正的源码分析

**这种方式叫什么？**
- **按行号精确读取源码分析法**
- **完整代码逻辑分析法**
- **深度源码研读法**

**核心思想**：
> 不要只看搜索结果的片段，要看完整的函数实现。
> 只有看到完整代码，才能真正理解实现逻辑。
