# Kiro IDE 路径配置分析

## 版本信息
- Kiro IDE 版本：v0.8.140
- 分析日期：2026-01-21
- 源码位置：extension.js (行 161190-166200)

---

## 核心路径函数

### 1. getHomeKiroPath() - 用户主目录 .kiro 路径

**源码位置**：行 161193-161196

```javascript
function getHomeKiroPath() {
  const homePath = os4.homedir();
  return path2.join(homePath, ".kiro");
}
```

**返回值**：
- Windows: `C:\Users\{用户名}\.kiro`
- macOS: `~/.kiro`
- Linux: `~/.kiro`

**用途**：
- 存储用户级配置
- MCP 配置文件：`~/.kiro/settings/mcp.json`

---

### 2. getWorkspaceKiroPath(workspaceDir) - 工作区 .kiro 路径

**源码位置**：行 161197-161199

```javascript
function getWorkspaceKiroPath(workspaceDir) {
  return path2.join(workspaceDir, ".kiro");
}
```

**返回值**：
- `{workspaceDir}/.kiro`

**用途**：
- 存储工作区级配置
- Steering 规则：`{workspaceDir}/.kiro/steering/*.md`
- MCP 配置：`{workspaceDir}/.kiro/settings/mcp.json`

---

### 3. getContinueGlobalPath() - 全局数据目录

**源码位置**：行 166152-166161

```javascript
function getContinueGlobalPath() {
  const continuePath = loadEnvVar("CONTINUE_GLOBAL_DIR");
  if (!continuePath) {
    throw new Error("CONTINUE_GLOBAL_DIR environment variable is not set");
  }
  if (!fs5.existsSync(continuePath)) {
    fs5.mkdirSync(continuePath);
  }
  return continuePath;
}
```

**环境变量**：`CONTINUE_GLOBAL_DIR`

**实际路径**（Windows）：
- `C:\Users\{用户名}\AppData\Roaming\Kiro`

**用途**：
- 存储全局数据（会话、索引、日志等）
- 所有会话文件的根目录

---

### 4. getSessionsFolderPath(workspaceDir) - 会话文件夹路径

**源码位置**：行 166162-166178

```javascript
function getSessionsFolderPath(workspaceDir) {
  if (!workspaceDir) {
    // 无工作区：全局会话
    const sessionsPath2 = path5.join(getContinueGlobalPath(), "sessions");
    if (!fs5.existsSync(sessionsPath2)) {
      fs5.mkdirSync(sessionsPath2);
    }
    return sessionsPath2;
  }
  
  // 有工作区：工作区特定会话
  const workspaceHash = Buffer.from(workspaceDir).toString("base64").replace(/[/+=]/g, "_");
  const sessionsPath = path5.join(getContinueGlobalPath(), "workspace-sessions", workspaceHash);
  if (!fs5.existsSync(sessionsPath)) {
    fs5.mkdirSync(sessionsPath, { recursive: true });
  }
  return sessionsPath;
}
```

**返回值**：
- 无工作区：`{CONTINUE_GLOBAL_DIR}/sessions`
- 有工作区：`{CONTINUE_GLOBAL_DIR}/workspace-sessions/{workspaceHash}`

**workspaceHash 计算**：
- Base64 编码工作区路径
- 替换 `/+=` 为 `_`（文件系统安全）

**示例**：
```
工作区路径: E:\VSCodeSpace\Kiro\kiro-gateway
Base64: RTpcVlNDb2RlU3BhY2VcS2lyb1xraXJvLWdhdGV3YXk=
Hash: RTpcVlNDb2RlU3BhY2VcS2lyb1xraXJvLWdhdGV3YXk_

最终路径: C:\Users\12925\AppData\Roaming\Kiro\workspace-sessions\RTpcVlNDb2RlU3BhY2VcS2lyb1xraXJvLWdhdGV3YXk_
```

---

### 5. getSessionFilePath(sessionId, workspaceDir) - 单个会话文件路径

**源码位置**：行 166187-166189

```javascript
function getSessionFilePath(sessionId, workspaceDir) {
  return path5.join(getSessionsFolderPath(workspaceDir), `${sessionId}.json`);
}
```

**返回值**：
- `{sessionsFolder}/{sessionId}.json`

**示例**：
```
C:\Users\12925\AppData\Roaming\Kiro\workspace-sessions\RTpcVlNDb2RlU3BhY2VcS2lyb1xraXJvLWdhdGV3YXk_\550e8400-e29b-41d4-a716-446655440000.json
```

---

### 6. getSessionsListPath(workspaceDir) - 会话列表文件路径

**源码位置**：行 166190-166196

```javascript
function getSessionsListPath(workspaceDir) {
  const filepath = path5.join(getSessionsFolderPath(workspaceDir), "sessions.json");
  if (!fs5.existsSync(filepath)) {
    fs5.writeFileSync(filepath, JSON.stringify([]));
  }
  return filepath;
}
```

**返回值**：
- `{sessionsFolder}/sessions.json`

**自动创建**：
- 如果文件不存在，自动创建空数组 `[]`

**示例**：
```
C:\Users\12925\AppData\Roaming\Kiro\workspace-sessions\RTpcVlNDb2RlU3BhY2VcS2lyb1xraXJvLWdhdGV3YXk_\sessions.json
```

---

### 7. getIndexFolderPath() - 索引文件夹路径

**源码位置**：行 166179-166185

```javascript
function getIndexFolderPath() {
  const indexPath = path5.join(getContinueGlobalPath(), "index");
  if (!fs5.existsSync(indexPath)) {
    fs5.mkdirSync(indexPath);
  }
  return indexPath;
}
```

**返回值**：
- `{CONTINUE_GLOBAL_DIR}/index`

**用途**：
- 存储代码索引数据（Embeddings）

---

### 8. getGlobalContextFilePath() - 全局上下文文件路径

**源码位置**：行 166186

```javascript
function getGlobalContextFilePath() {
  return path5.join(getIndexFolderPath(), "globalContext.json");
}
```

**返回值**：
- `{CONTINUE_GLOBAL_DIR}/index/globalContext.json`

---

## 完整目录结构

```
C:\Users\{用户名}\
├── .kiro/                                    # 用户主目录配置
│   └── settings/
│       └── mcp.json                          # 用户级 MCP 配置
│
└── AppData\Roaming\Kiro\                     # CONTINUE_GLOBAL_DIR
    ├── logs/                                 # 日志目录
    │   └── {时间戳}/
    │       └── window1/
    │           └── exthost/
    │               └── kiro.kiroAgent/
    │                   ├── q-client.log      # API 请求日志
    │                   ├── Kiro Logs.log     # 主日志
    │                   ├── KiroLLMLogs.log   # LLM 日志
    │                   └── Kiro - MCP Logs.log  # MCP 日志
    │
    ├── sessions/                             # 全局会话（无工作区）
    │   ├── sessions.json                     # 会话列表
    │   └── {sessionId}.json                  # 单个会话
    │
    ├── workspace-sessions/                   # 工作区会话
    │   └── {workspaceHash}/                  # 工作区特定会话
    │       ├── sessions.json                 # 会话列表
    │       └── {sessionId}.json              # 单个会话
    │
    └── index/                                # 索引数据
        └── globalContext.json                # 全局上下文

{workspaceDir}/                               # 工作区目录
└── .kiro/                                    # 工作区配置
    ├── steering/                             # Steering 规则
    │   ├── global.md
    │   └── ...
    └── settings/
        └── mcp.json                          # 工作区级 MCP 配置
```

---

## MCP 配置合并优先级

**源码位置**：行 161200-161220

```javascript
function getActiveMcpConfigLocation(workspaceDirs) {
  const workspaceConfigPaths = [];
  let userConfigPath;
  
  // 1. 收集所有工作区配置
  if (workspaceDirs && workspaceDirs.length > 0) {
    for (const workspaceDir of workspaceDirs) {
      const workspacePath = path2.join(
        getWorkspaceKiroPath(workspaceDir), 
        "settings", 
        "mcp.json"
      );
      if (fs2.existsSync(workspacePath)) {
        workspaceConfigPaths.push(workspacePath);
      }
    }
  }
  
  // 2. 获取用户级配置
  const kiroHomePath = getHomeKiroPath();
  if (kiroHomePath) {
    const homePath = path2.join(kiroHomePath, "settings", "mcp.json");
    if (fs2.existsSync(homePath)) {
      userConfigPath = homePath;
    }
  }
  
  return {
    workspaceConfigPaths,
    userConfigPath
  };
}
```

**合并优先级**（从低到高）：
1. 用户级配置：`~/.kiro/settings/mcp.json`
2. 工作区1配置：`{workspace1}/.kiro/settings/mcp.json`
3. 工作区2配置：`{workspace2}/.kiro/settings/mcp.json`
4. ...

**规则**：
- 后面的配置覆盖前面的配置
- 多工作区时，后面的工作区优先级更高

---

## 环境变量

### CONTINUE_GLOBAL_DIR

**作用**：指定 Kiro IDE 的全局数据目录

**默认值**（推测）：
- Windows: `%APPDATA%\Kiro`
- macOS: `~/Library/Application Support/Kiro`
- Linux: `~/.local/share/Kiro`

**如何设置**：
```bash
# Windows (PowerShell)
$env:CONTINUE_GLOBAL_DIR = "C:\Users\12925\AppData\Roaming\Kiro"

# macOS/Linux
export CONTINUE_GLOBAL_DIR="$HOME/.local/share/Kiro"
```

---

## 关键发现

### 1. 会话隔离策略

- **无工作区**：所有会话存储在 `sessions/` 目录
- **有工作区**：每个工作区的会话存储在独立的 `workspace-sessions/{hash}/` 目录
- **工作区 Hash**：Base64 编码工作区路径，确保唯一性

### 2. 配置层级

- **用户级**：`~/.kiro/settings/mcp.json`（全局生效）
- **工作区级**：`{workspace}/.kiro/settings/mcp.json`（工作区生效）
- **优先级**：工作区配置覆盖用户配置

### 3. 自动创建机制

- 所有路径函数都会自动创建不存在的目录
- `sessions.json` 不存在时自动创建空数组

### 4. 路径安全性

- 工作区 Hash 使用 Base64 编码
- 替换文件系统不安全字符 `/+=` 为 `_`
- 确保跨平台兼容性

---

## 对 kiro-gateway 的启示

### 1. 数据目录设计

参考 Kiro IDE 的设计，kiro-gateway 应该：

```
用户数据目录/
├── accounts.json           # 账号配置
├── api_keys.json          # API Key 配置
├── metrics.json           # 统计数据
├── sessions/              # 全局会话
│   ├── sessions.json
│   └── {sessionId}.json
└── workspace-sessions/    # 工作区会话
    └── {workspaceHash}/
        ├── sessions.json
        └── {sessionId}.json
```

### 2. 会话隔离

- 支持全局会话（无工作区）
- 支持工作区特定会话（有工作区）
- 使用 Base64 Hash 隔离不同工作区

### 3. 配置合并

- 支持用户级和工作区级配置
- 工作区配置覆盖用户配置
- 多工作区时按顺序合并

### 4. 自动创建

- 所有路径函数自动创建目录
- 配置文件不存在时自动初始化

---

## 相关文档

- Kiro IDE 日志分析：`.kiro/steering/kiro-ide-logs.md`
- 会话管理源码分析：`docs/technical/kiro-session-management-source-analysis.md`
- Kiro API 规范：`docs/technical/kiro-api.md`
