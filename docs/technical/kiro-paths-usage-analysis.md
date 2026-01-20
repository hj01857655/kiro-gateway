# Kiro IDE 路径使用分析

## 版本信息
- Kiro IDE 版本：v0.8.140
- 分析日期：2026-01-21
- 源码位置：extension.js

---

## getContinueGlobalPath() 的所有使用场景

**调用次数**：14 次

### 1. .utils 目录 - 工具文件存储

**源码位置**：行 166145-166150

```javascript
function getUtilsPath() {
  const utilsPath = path5.join(getContinueGlobalPath(), ".utils");
  if (!fs5.existsSync(utilsPath)) {
    fs5.mkdirSync(utilsPath);
  }
  return utilsPath;
}
```

**路径**：`{CONTINUE_GLOBAL_DIR}/.utils`

**用途**：存储工具文件

---

### 2. sessions 目录 - 全局会话（无工作区）

**源码位置**：行 166164-166168

```javascript
if (!workspaceDir) {
  const sessionsPath2 = path5.join(getContinueGlobalPath(), "sessions");
  if (!fs5.existsSync(sessionsPath2)) {
    fs5.mkdirSync(sessionsPath2);
  }
  return sessionsPath2;
}
```

**路径**：`{CONTINUE_GLOBAL_DIR}/sessions`

**用途**：存储无工作区的全局会话

---

### 3. workspace-sessions 目录 - 工作区会话

**源码位置**：行 166171-166175

```javascript
const workspaceHash = Buffer.from(workspaceDir).toString("base64").replace(/[/+=]/g, "_");
const sessionsPath = path5.join(getContinueGlobalPath(), "workspace-sessions", workspaceHash);
if (!fs5.existsSync(sessionsPath)) {
  fs5.mkdirSync(sessionsPath, { recursive: true });
}
return sessionsPath;
```

**路径**：`{CONTINUE_GLOBAL_DIR}/workspace-sessions/{workspaceHash}`

**用途**：存储工作区特定的会话

---

### 4. index 目录 - 代码索引

**源码位置**：行 166178-166182

```javascript
function getIndexFolderPath() {
  const indexPath = path5.join(getContinueGlobalPath(), "index");
  if (!fs5.existsSync(indexPath)) {
    fs5.mkdirSync(indexPath);
  }
  return indexPath;
}
```

**路径**：`{CONTINUE_GLOBAL_DIR}/index`

**用途**：存储代码索引数据（Embeddings）

---

### 5. config.json - 主配置文件

**源码位置**：行 166211-166216

```javascript
function getConfigJsonPath() {
  const p8 = path5.join(getContinueGlobalPath(), "config.json");
  if (!fs5.existsSync(p8)) {
    fs5.writeFileSync(p8, JSON.stringify(defaultConfig, null, 2));
  }
  return p8;
}
```

**路径**：`{CONTINUE_GLOBAL_DIR}/config.json`

**用途**：存储主配置文件

**自动创建**：如果不存在，自动创建默认配置

---

### 6. .continuerc.json - Continue 配置

**源码位置**：行 166218-166228

```javascript
function getContinueRcPath() {
  const continuercPath = path5.join(getContinueGlobalPath(), ".continuerc.json");
  if (!fs5.existsSync(continuercPath)) {
    fs5.writeFileSync(
      continuercPath,
      JSON.stringify(
        {
          disableIndexing: true
        },
        null,
        2
      )
    );
  }
  return continuercPath;
}
```

**路径**：`{CONTINUE_GLOBAL_DIR}/.continuerc.json`

**用途**：Continue 特定配置

**默认配置**：`{ "disableIndexing": true }`

---

### 7. dev_data 目录 - 开发数据

**源码位置**：行 166234-166239

```javascript
function devDataPath() {
  const sPath = path5.join(getContinueGlobalPath(), "dev_data");
  if (!fs5.existsSync(sPath)) {
    fs5.mkdirSync(sPath);
  }
  return sPath;
}
```

**路径**：`{CONTINUE_GLOBAL_DIR}/dev_data`

**用途**：存储开发数据

**关联文件**：
- `devdata.sqlite` - SQLite 数据库

---

### 8. .migrations 目录 - 数据库迁移

**源码位置**：行 166257-166261

```javascript
function getMigrationsPath() {
  const migrationsPath = path5.join(getContinueGlobalPath(), ".migrations");
  if (!fs5.existsSync(migrationsPath)) {
    fs5.mkdirSync(migrationsPath);
  }
  return migrationsPath;
}
```

**路径**：`{CONTINUE_GLOBAL_DIR}/.migrations`

**用途**：存储数据库迁移记录

---

### 9. .configs 目录 - 配置备份

**源码位置**：行 166294-166298

```javascript
function getConfigsPath() {
  const dir = path5.join(getContinueGlobalPath(), ".configs");
  if (!fs5.existsSync(dir)) {
    fs5.mkdirSync(dir);
  }
  return dir;
}
```

**路径**：`{CONTINUE_GLOBAL_DIR}/.configs`

**用途**：存储配置文件备份

---

### 10. .env 文件 - 环境变量

**源码位置**：行 166316-166320

```javascript
function loadEnvFile() {
  const filepath = path5.join(getContinueGlobalPath(), ".env");
  if (fs5.existsSync(filepath)) {
    return import_dotenv.default.parse(fs5.readFileSync(filepath));
  }
  return {};
}
```

**路径**：`{CONTINUE_GLOBAL_DIR}/.env`

**用途**：存储环境变量配置

**格式**：dotenv 格式

---

### 11. kiroAgent.log - 主日志文件

**源码位置**：行 575174-575177

```javascript
const logFile = path29.join(getContinueGlobalPath(), "kiroAgent.log");
if (!fs25.existsSync(logFile)) {
  fs25.mkdirSync(path29.dirname(logFile), { recursive: true });
  fs25.writeFileSync(logFile, "");
}
```

**路径**：`{CONTINUE_GLOBAL_DIR}/kiroAgent.log`

**用途**：存储 Kiro Agent 主日志

---

### 12. .diffs 目录 - Diff 文件存储

**源码位置**：行 577832

```javascript
DIFF_DIRECTORY = path31.join(getContinueGlobalPath(), ".diffs").replace(/^C:/, "c:");
```

**路径**：`{CONTINUE_GLOBAL_DIR}/.diffs`

**用途**：存储代码差异文件

**特殊处理**：Windows 路径规范化（`C:` → `c:`）

---

### 13. 全局路径获取（API）

**源码位置**：行 582612

```javascript
return getContinueGlobalPath();
```

**用途**：提供给外部模块获取全局路径

---

## getSessionFilePath() 的使用场景

**调用次数**：4 次

### 1. 加载会话 - load()

**源码位置**：行 569956-569960

```javascript
const sessionFile = getSessionFilePath(sessionId, workspaceDir);
if (!fs24.existsSync(sessionFile)) {
  throw new Error(`Session file ${sessionFile} does not exist`);
}
const session = JSON.parse(fs24.readFileSync(sessionFile, "utf8"));
```

**操作**：读取会话文件

---

### 2. 删除会话 - delete()

**源码位置**：行 569976-569980

```javascript
const sessionFile = getSessionFilePath(sessionId, workspaceDir);
if (!fs24.existsSync(sessionFile)) {
  throw new Error(`Session file ${sessionFile} does not exist`);
}
fs24.unlinkSync(sessionFile);
```

**操作**：删除会话文件

---

### 3. 保存会话 - save()

**源码位置**：行 569994-569995

```javascript
const filePath = getSessionFilePath(session.sessionId, session.workspaceDirectory);
fs24.writeFileSync(filePath, JSON.stringify(session, void 0, 2));
```

**操作**：保存会话到文件

**格式**：JSON，缩进 2 空格

---

## getSessionsListPath() 的使用场景

**调用次数**：4 次

### 1. 列出所有会话 - list()

**源码位置**：行 569941-569950

```javascript
const filepath = getSessionsListPath(workspaceDir);
if (!fs24.existsSync(filepath)) {
  return [];
}
const rawSessionsList = fs24.readFileSync(filepath, "utf-8");
let sessionsList;
try {
  sessionsList = JSON.parse(rawSessionsList);
} catch (e11) {
  // 错误处理
}
```

**操作**：读取会话列表

---

### 2. 删除会话时更新列表 - delete()

**源码位置**：行 569961-569990

```javascript
const sessionsListFile = getSessionsListPath(workspaceDir);
const sessionsListRaw = fs24.readFileSync(sessionsListFile, "utf-8");
let sessionsList;
try {
  sessionsList = JSON.parse(sessionsListRaw);
} catch (e11) {
  if (sessionsListRaw.trim() === "") {
    fs24.writeFileSync(sessionsListFile, JSON.stringify([]));
    sessionsList = [];
  } else {
    throw e11;
  }
}
// 从列表中移除会话
const updatedSessionsList = sessionsList.filter(
  (sessionInfo) => sessionInfo.sessionId !== sessionId
);
fs24.writeFileSync(sessionsListFile, JSON.stringify(updatedSessionsList, void 0, 2));
```

**操作**：从列表中移除已删除的会话

---

### 3. 保存会话时更新列表 - save()

**源码位置**：行 569996-570040

```javascript
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
```

**操作**：更新或添加会话到列表

---

## 完整目录结构（实际使用）

```
C:\Users\{用户名}\AppData\Roaming\Kiro\User\globalStorage\kiro.kiroagent\  # CONTINUE_GLOBAL_DIR
├── .utils/                                       # 工具文件
├── .diffs/                                       # Diff 文件
├── .migrations/                                  # 数据库迁移
├── .configs/                                     # 配置备份
├── sessions/                                     # 全局会话（无工作区）
│   ├── sessions.json                             # 会话列表
│   └── {sessionId}.json                          # 单个会话
├── workspace-sessions/                           # 工作区会话
│   └── {workspaceHash}/                          # 工作区特定会话
│       ├── sessions.json                         # 会话列表
│       └── {sessionId}.json                      # 单个会话
├── index/                                        # 代码索引
│   └── globalContext.json                        # 全局上下文
├── dev_data/                                     # 开发数据
│   └── devdata.sqlite                            # SQLite 数据库
├── config.json                                   # 主配置文件
├── .continuerc.json                              # Continue 配置
├── .env                                          # 环境变量
└── kiroAgent.log                                 # 主日志文件
```

---

## 会话管理操作流程

### 1. 列出会话（list）

```
1. 调用 getSessionsListPath(workspaceDir)
2. 读取 sessions.json
3. 解析 JSON 数组
4. 返回会话列表
```

### 2. 加载会话（load）

```
1. 调用 getSessionFilePath(sessionId, workspaceDir)
2. 检查文件是否存在
3. 读取并解析 JSON
4. 确保 sessionId 字段存在
5. 返回会话对象
```

### 3. 保存会话（save）

```
1. 调用 getSessionFilePath(sessionId, workspaceDir)
2. 保存完整会话到 {sessionId}.json
3. 调用 getSessionsListPath(workspaceDir)
4. 读取 sessions.json
5. 查找是否已存在该会话
6. 如果存在，更新元数据；否则添加新元数据
7. 保存更新后的 sessions.json
```

### 4. 删除会话（delete）

```
1. 调用 getSessionFilePath(sessionId, workspaceDir)
2. 删除 {sessionId}.json 文件
3. 调用 getSessionsListPath(workspaceDir)
4. 读取 sessions.json
5. 从列表中移除该会话
6. 保存更新后的 sessions.json
```

---

## 关键发现

### 1. 双层存储结构

- **完整会话**：`{sessionId}.json`（包含完整的 history 和所有字段）
- **会话列表**：`sessions.json`（只包含元数据：sessionId、title、dateCreated、workspaceDirectory、hidden）

**优势**：
- 列表文件小，加载快
- 完整会话按需加载
- 减少内存占用

### 2. 自动创建机制

所有路径函数都会：
- 检查目录/文件是否存在
- 不存在时自动创建
- 确保路径始终可用

### 3. 错误处理

- JSON 解析失败时的容错处理
- 空文件自动初始化为 `[]`
- 文件不存在时抛出明确错误

### 4. 工作区隔离

- 使用 Base64 Hash 隔离不同工作区
- 每个工作区有独立的会话存储
- 全局会话和工作区会话分开存储

### 5. 配置文件管理

- 主配置：`config.json`
- Continue 配置：`.continuerc.json`
- 环境变量：`.env`
- 配置备份：`.configs/`

### 6. 日志和调试

- 主日志：`kiroAgent.log`
- Diff 文件：`.diffs/`
- 开发数据：`dev_data/devdata.sqlite`

---

## 对 kiro-gateway 的启示

### 1. 采用双层存储结构

```rust
// 会话元数据（轻量级）
struct SessionInfo {
    session_id: String,
    title: String,
    date_created: i64,
    workspace_directory: String,
    hidden: bool,
}

// 完整会话（按需加载）
struct Session {
    session_id: String,
    title: String,
    history: Vec<Message>,
    workspace_directory: String,
    hidden: bool,
}
```

### 2. 实现自动创建机制

```rust
fn ensure_directory(path: &Path) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}
```

### 3. 完善错误处理

```rust
fn load_sessions_list(workspace_dir: &str) -> Result<Vec<SessionInfo>> {
    let path = get_sessions_list_path(workspace_dir);
    
    if !path.exists() {
        return Ok(vec![]);
    }
    
    let content = fs::read_to_string(&path)?;
    
    if content.trim().is_empty() {
        fs::write(&path, "[]")?;
        return Ok(vec![]);
    }
    
    match serde_json::from_str(&content) {
        Ok(list) => Ok(list),
        Err(e) => Err(AppError::JsonParseError(e.to_string())),
    }
}
```

### 4. 支持工作区隔离

```rust
fn get_workspace_hash(workspace_dir: &str) -> String {
    use base64::{Engine as _, engine::general_purpose};
    let encoded = general_purpose::STANDARD.encode(workspace_dir);
    encoded.replace(['/', '+', '='], "_")
}
```

---

## 相关文档

- Kiro IDE 路径配置分析：`docs/technical/kiro-paths-analysis.md`
- 会话管理源码分析：`docs/technical/kiro-session-management-source-analysis.md`
- Kiro API 规范：`docs/technical/kiro-api.md`
