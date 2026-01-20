# Kiro IDE 会话管理源码分析

## 版本信息
- Kiro IDE 版本：v0.8.140
- 分析日期：2026-01-20
- 源码位置：`C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js`

---

## 核心发现

### 1. 会话存储结构

Kiro IDE 使用**双文件存储**策略：

1. **完整会话文件**：`{sessionId}.json` - 存储完整的会话数据
2. **会话列表文件**：`sessions.json` - 存储所有会话的元数据

**存储路径**：
- 无工作区：`{globalPath}/sessions/`
- 有工作区：`{globalPath}/workspace-sessions/{workspaceHash}/`
  - `workspaceHash` = Base64(workspaceDir).replace(/[/+=]/g, "_")

---

## 源码分析

### 1. getSessionsFolderPath (行 166162-166176)

```javascript
function getSessionsFolderPath(workspaceDir) {
  if (!workspaceDir) {
    // 无工作区：存储在全局 sessions 目录
    const sessionsPath2 = path5.join(getContinueGlobalPath(), "sessions");
    if (!fs5.existsSync(sessionsPath2)) {
      fs5.mkdirSync(sessionsPath2);
    }
    return sessionsPath2;
  }
  
  // 有工作区：存储在 workspace-sessions/{hash} 目录
  const workspaceHash = Buffer.from(workspaceDir).toString("base64").replace(/[/+=]/g, "_");
  const sessionsPath = path5.join(getContinueGlobalPath(), "workspace-sessions", workspaceHash);
  if (!fs5.existsSync(sessionsPath)) {
    fs5.mkdirSync(sessionsPath, { recursive: true });
  }
  return sessionsPath;
}
```

**关键点**：
- 工作区路径通过 Base64 编码并替换特殊字符作为目录名
- 自动创建目录（如果不存在）
- 支持无工作区和有工作区两种模式

---

### 2. getSessionFilePath (行 166187-166189)

```javascript
function getSessionFilePath(sessionId, workspaceDir) {
  return path5.join(getSessionsFolderPath(workspaceDir), `${sessionId}.json`);
}
```

**用途**：获取完整会话文件的路径

---

### 3. getSessionsListPath (行 166190-166195)

```javascript
function getSessionsListPath(workspaceDir) {
  const filepath = path5.join(getSessionsFolderPath(workspaceDir), "sessions.json");
  if (!fs5.existsSync(filepath)) {
    fs5.writeFileSync(filepath, JSON.stringify([]));
  }
  return filepath;
}
```

**关键点**：
- 如果 `sessions.json` 不存在，自动创建空数组
- 确保文件始终存在

---

### 4. save() - 保存会话 (行 569994-570040)

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
    if (error3 instanceof SyntaxError) {
      throw new Error(
        `It looks like there is a JSON formatting error in your sessions.json file (${sessionsListFilePath}). Please fix this before creating a new session.`
      );
    }
    throw new Error(
      `It looks like there is a validation error in your sessions.json file (${sessionsListFilePath}). Please fix this before creating a new session. Error: ${error3}`
    );
  }
}
```

**保存流程**：
1. 保存完整会话到 `{sessionId}.json`
2. 读取 `sessions.json` 列表
3. 如果会话已存在，更新元数据
4. 如果会话不存在，添加新元数据
5. 保存更新后的列表

**错误处理**：
- JSON 解析错误 → 提示格式错误
- 其他错误 → 提示验证错误

---

### 5. load() - 加载会话 (行 569974-569990)

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

**加载流程**：
1. 检查会话文件是否存在
2. 读取并解析 JSON
3. 确保 sessionId 字段存在
4. 如果失败，返回默认会话结构

**默认会话结构**：
```javascript
{
  history: [],
  title: "New Session",
  workspaceDirectory: "",
  sessionId: sessionId
}
```

---

### 6. delete() - 删除会话 (行 569956-569973)

```javascript
delete(sessionId, workspaceDir) {
  const sessionFile = getSessionFilePath(sessionId, workspaceDir);
  if (!fs24.existsSync(sessionFile)) {
    throw new Error(`Session file ${sessionFile} does not exist`);
  }
  
  // 删除会话文件
  fs24.unlinkSync(sessionFile);
  
  // 从 sessions.json 列表中移除
  const sessionsListFile = getSessionsListPath(workspaceDir);
  let sessionsList = JSON.parse(fs24.readFileSync(sessionsListFile, "utf-8"));
  sessionsList = sessionsList.filter((session) => session.sessionId !== sessionId);
  fs24.writeFileSync(sessionsListFile, JSON.stringify(sessionsList, void 0, 2));
}
```

**删除流程**：
1. 检查会话文件是否存在
2. 删除 `{sessionId}.json` 文件
3. 从 `sessions.json` 列表中移除对应元数据
4. 保存更新后的列表

---

## 数据结构

### 1. Session（完整会话）

存储在 `{sessionId}.json` 文件中：

```typescript
interface Session {
  sessionId: string;
  title: string;
  workspaceDirectory?: string;
  history: Array<any>;  // 对话历史
  hidden?: boolean;
  // ... 其他字段（selectedModel, autonomyMode, contextUsagePercentage 等）
}
```

### 2. SessionInfo（会话元数据）

存储在 `sessions.json` 列表中：

```typescript
interface SessionInfo {
  sessionId: string;
  title: string;
  dateCreated: string;  // Unix 时间戳字符串
  workspaceDirectory?: string;
  hidden?: boolean;
}
```

---

## 关键设计原则

### 1. 双文件存储

**优势**：
- 快速列出所有会话（只需读取 `sessions.json`）
- 完整会话数据按需加载（避免一次性加载所有会话）
- 元数据和完整数据分离，提高性能

### 2. 工作区隔离

**优势**：
- 不同工作区的会话互不干扰
- 支持无工作区模式（全局会话）
- 通过 Base64 编码确保路径安全

### 3. 容错处理

**策略**：
- 文件不存在 → 自动创建
- JSON 解析失败 → 返回默认结构
- 空文件 → 初始化为空数组

### 4. 原子操作

**保存流程**：
1. 先保存完整会话文件
2. 再更新会话列表
3. 确保数据一致性

---

## 实现建议

### 1. Rust 实现对应关系

| Kiro IDE | kiro-gateway Rust |
|----------|-------------------|
| `getSessionsFolderPath()` | `get_sessions_folder_path()` |
| `getSessionFilePath()` | `get_session_file_path()` |
| `getSessionsListPath()` | `get_sessions_list_path()` |
| `save()` | `save_session()` |
| `load()` | `load_session()` |
| `delete()` | `delete_session()` |

### 2. 数据结构定义

```rust
/// 完整会话（存储在 {sessionId}.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub session_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_directory: Option<String>,
    pub history: Vec<serde_json::Value>,
    #[serde(default)]
    pub hidden: bool,
}

/// 会话元数据（存储在 sessions.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub session_id: String,
    pub title: String,
    pub date_created: String,  // Unix 时间戳字符串
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_directory: Option<String>,
    #[serde(default)]
    pub hidden: bool,
}
```

### 3. 实现要点

**保存会话**：
1. 使用 `serde_json::to_string_pretty()` 格式化 JSON（缩进 2 空格）
2. 先保存完整会话，再更新列表
3. 更新列表时检查是否已存在（更新 vs 添加）

**加载会话**：
1. 检查文件是否存在
2. 解析 JSON，失败时返回默认结构
3. 确保 sessionId 字段存在

**删除会话**：
1. 删除完整会话文件
2. 从列表中过滤掉对应元数据
3. 保存更新后的列表

**工作区路径处理**：
```rust
use base64::{Engine as _, engine::general_purpose};

fn encode_workspace_path(path: &str) -> String {
    general_purpose::STANDARD
        .encode(path.as_bytes())
        .replace('/', "_")
        .replace('+', "_")
        .replace('=', "_")
}
```

---

## API 设计

### 1. 列出所有会话

```
GET /admin/sessions?workspace={workspaceDir}
```

**实现**：
- 读取 `sessions.json` 文件
- 返回会话元数据列表

### 2. 获取会话详情

```
GET /admin/sessions/:id?workspace={workspaceDir}
```

**实现**：
- 调用 `load_session()`
- 返回完整会话数据

### 3. 创建/更新会话

```
POST /admin/sessions
Body: { session: Session }
```

**实现**：
- 调用 `save_session()`
- 返回保存后的会话

### 4. 删除会话

```
DELETE /admin/sessions/:id?workspace={workspaceDir}
```

**实现**：
- 调用 `delete_session()`
- 返回成功状态

### 5. 搜索会话

```
GET /admin/sessions/search?q={query}&workspace={workspaceDir}
```

**实现**：
- 读取 `sessions.json`
- 按标题或 ID 过滤
- 返回匹配的会话元数据

---

## 注意事项

### 1. 文件操作

- 使用 `fs::write()` 而不是 `fs::File::create()` + `write!()`
- 确保目录存在（`fs::create_dir_all()`）
- 处理文件不存在的情况

### 2. JSON 序列化

- 使用 `serde_json::to_string_pretty()` 保持可读性
- 缩进 2 空格（与 Kiro IDE 一致）
- 处理 JSON 解析错误

### 3. 错误处理

- 文件不存在 → 返回默认结构（不抛出错误）
- JSON 解析失败 → 返回默认结构
- 其他错误 → 返回 AppError

### 4. 并发安全

- 使用 `RwLock` 保护会话列表
- 避免同时写入同一个文件
- 考虑使用文件锁（可选）

---

## 参考资料

- Kiro IDE 源码：`C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js`
- 行号：166162-166195（路径函数）、569956-570040（CRUD 操作）
- 会话文件示例：`C:\Users\12925\AppData\Roaming\Kiro\User\globalStorage\kiro.kiroagent\workspace-sessions\`

---

## 总结

Kiro IDE 的会话管理采用**双文件存储**策略：
1. **完整会话文件**（`{sessionId}.json`）- 存储完整数据
2. **会话列表文件**（`sessions.json`）- 存储元数据

这种设计兼顾了**性能**（快速列出会话）和**灵活性**（按需加载完整数据），是一个成熟且经过验证的方案。

我们的 Rust 实现应该完全遵循这个设计，确保与 Kiro IDE 的兼容性。
