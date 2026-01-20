# Kiro IDE `.kiro` 目录使用分析

## 版本信息
- Kiro IDE 版本：v0.8.140
- 分析日期：2026-01-21
- 源码文件：extension.js

---

## 核心常量定义

### PRODUCT_CONFIG_DIRECTORY（行 684457）
```javascript
PRODUCT_CONFIG_DIRECTORY = ".kiro";
```

这是 Kiro IDE 的产品配置目录名称，所有工作区级别的配置都存储在 `.kiro/` 目录下。

---

## `.kiro` 目录结构

### 工作区级别（Workspace Level）

```
{workspace}/.kiro/
├── steering/          # Steering 规则文件
│   ├── *.md          # 规则文档（Markdown 格式）
│   └── AGENTS.md     # 自定义 Agent 定义（可选）
├── specs/            # Spec 规范文件
│   └── {feature}/    # 功能规范目录
│       ├── requirements.md  # 需求文档
│       ├── design.md        # 设计文档
│       ├── tasks.md         # 任务列表
│       └── .config.kiro     # Spec 配置文件
├── hooks/            # Hook 钩子文件
│   └── *.kiro.hook   # Hook 配置（JSON 格式）
├── agents/           # 自定义 Agent（实验性功能）
│   └── *.md          # Agent 定义文件
├── settings/         # 设置文件
│   └── mcp.json      # MCP 配置文件
└── debug/            # 调试信息（临时）
    ├── debug.log     # 调试日志
    ├── execution-log.json  # 执行日志
    └── chats/        # 聊天记录
        └── *.chat    # 聊天文件
```

### 全局级别（Global Level）

```
~/.kiro/
├── steering/         # 全局 Steering 规则
│   └── *.md         # 全局规则文档
└── settings/        # 全局设置
    └── mcp.json     # 全局 MCP 配置
```

**Windows 路径**：`C:\Users\{用户名}\.kiro\`  
**macOS/Linux 路径**：`~/.kiro/`

---

## 各子目录详细说明

### 1. steering/ - Steering 规则目录

**用途**：存储 AI 助手的行为规则和指导文档

**文件格式**：Markdown (`.md`)

**路径构建**（行 816203）：
```javascript
// 全局 Steering 目录
return vscode183.Uri.joinPath(vscode183.Uri.file(homeDir), ".kiro", "steering");

// 工作区 Steering 目录
return vscode183.Uri.joinPath(workspaceUri, PRODUCT_CONFIG_DIRECTORY, STEERING_DIRECTORY);
```

**扫描逻辑**（行 816220-816280）：
```javascript
async listSteeringUris() {
  // 1. 确保全局 Steering 目录存在
  await this.ensureGlobalSteeringDirectoryExists();
  
  // 2. 扫描所有工作区的 .kiro/steering/ 目录
  const workspaceResults = await Promise.all(
    this.steeringDirectoryUris.map(async (steeringDirectoryUri) => {
      return await this.findMarkdownFilesRecursively(steeringDirectoryUri);
    })
  );
  
  // 3. 扫描全局 Steering 目录
  const globalSteeringDir = this.getGlobalSteeringDirectory();
  const globalResults = await this.findMarkdownFilesRecursively(globalSteeringDir);
  
  // 4. 扫描工作区根目录的 AGENTS.md 文件
  const agentsMdResults = await Promise.all(
    this.workspaceUris.map(async (workspaceUri) => {
      return await this.findAgentsMdFiles(workspaceUri);
    })
  );
  
  return [...workspaceResults.flat(), ...globalResults, ...agentsMdResults.flat()];
}
```

**文件监控**（行 833838）：
```javascript
new vscode241.RelativePattern(folder, ".kiro/steering/**/*.md")
```

**Front Matter 格式**：
```markdown
---
inclusion: always  # 或 fileMatch 或 manual
fileMatchPattern: '**/*.ts'  # 仅当 inclusion: fileMatch 时需要
---

# 规则内容
```

**Inclusion 类型**：
- `always` - 总是包含在上下文中
- `fileMatch` - 当打开匹配的文件时包含
- `manual` - 手动引用（通过 `#steering文件名`）

**关键发现**：
- Steering 文件递归扫描（支持子目录）
- 支持全局和工作区两级 Steering
- 特殊文件 `AGENTS.md` 可以放在工作区根目录
- 文件变更自动刷新（FileSystemWatcher）

---

### 2. specs/ - Spec 规范目录

**用途**：存储功能规范文档（需求、设计、任务）

**目录结构**：
```
.kiro/specs/
└── {feature-name}/      # 功能名称（kebab-case）
    ├── requirements.md  # 需求文档
    ├── design.md        # 设计文档
    ├── tasks.md         # 任务列表
    └── .config.kiro     # Spec 配置文件
```

**常量定义**（行 684473）：
```javascript
SPECS_DIRECTORY = "specs";
SPEC_FILE_EXTENSIONS = ".md";
```

**路径构建**（行 684491）：
```javascript
return vscode91.Uri.joinPath(this.workspaceUri, WORKSPACE_CONFIG_PATH, SPECS_DIRECTORY);
// 结果：{workspace}/.kiro/specs
```

**文件监控**（行 796541）：
```javascript
const pattern = new vscode196.RelativePattern(folder, ".kiro/specs/**/.config.kiro");
const configWatcher = vscode196.workspace.createFileSystemWatcher(pattern);
```

**Spec 文档类型**：
- `requirements.md` - 需求文档（用户故事、验收标准）
- `design.md` - 设计文档（架构、技术选型）
- `tasks.md` - 任务列表（实现步骤）
- `.config.kiro` - Spec 配置文件（JSON 格式）

**识别逻辑**（行 796485）：
```javascript
const hasSpecsInPath = pathSegments.includes(".kiro") && pathSegments.includes("specs");
const isSpecDocument = uri2.path.endsWith("/requirements.md") || 
                       uri2.path.endsWith("/design.md") || 
                       uri2.path.endsWith("/tasks.md");
```

**上下文引用**（行 203873）：
```javascript
description: "Reference a spec from .kiro/specs"
```

**关键发现**：
- Spec 采用三文档结构（需求、设计、任务）
- 每个功能一个独立目录
- 支持配置文件 `.config.kiro`
- 可以通过 `#spec` 引用

---

### 3. hooks/ - Hook 钩子目录

**用途**：存储自动化钩子配置

**文件格式**：JSON (`.kiro.hook`)

**文件监控**（行 632383）：
```javascript
new vscode78.RelativePattern(folder, ".kiro/hooks/**/*.kiro.hook")
```

**路径验证**（行 630309）：
```javascript
if (!uri2.path.includes(".kiro/hooks/")) {
  return void 0;
}
```

**Hook 文件结构**：
```json
{
  "name": "Hook 名称",
  "version": "1.0.0",
  "description": "Hook 描述",
  "when": {
    "type": "fileEdited",  // 触发类型
    "patterns": ["*.ts"]   // 文件模式
  },
  "then": {
    "type": "askAgent",    // 动作类型
    "prompt": "提示词"
  }
}
```

**触发类型**：
- `fileEdited` - 文件保存时
- `fileCreated` - 文件创建时
- `fileDeleted` - 文件删除时
- `userTriggered` - 用户手动触发
- `promptSubmit` - 发送消息时
- `agentStop` - Agent 执行完成时

**动作类型**：
- `askAgent` - 发送提示词给 Agent
- `runCommand` - 执行 Shell 命令

**关键发现**：
- Hook 文件必须以 `.kiro.hook` 结尾
- 支持递归扫描子目录
- 文件变更自动同步（行 630835）

---

### 4. agents/ - 自定义 Agent 目录（实验性）

**用途**：存储自定义 Agent 定义

**文件格式**：Markdown (`.md`)

**配置开关**（行 629065）：
```javascript
description: "Enable custom agents in .kiro/agents/*.md",
default: false,
```

**关键发现**：
- 默认禁用，需要手动开启
- 实验性功能，可能不稳定
- 文件格式与 Steering 类似

---

### 5. settings/ - 设置目录

**用途**：存储配置文件

#### 5.1 mcp.json - MCP 配置文件

**路径**：
- 工作区：`{workspace}/.kiro/settings/mcp.json`
- 全局：`~/.kiro/settings/mcp.json`

**路径验证**（行 583536）：
```javascript
const expectedSuffix = path36.join(".kiro", "settings", "mcp.json");
if (filePath.endsWith(expectedSuffix)) {
  // 处理 MCP 配置变更
}
```

**配置结构**：
```json
{
  "mcpServers": {
    "server-name": {
      "command": "uvx",
      "args": ["package-name"],
      "env": {},
      "disabled": false,
      "autoApprove": []
    }
  }
}
```

**配置合并优先级**：
```
用户配置 < workspace1 < workspace2 < ...
```

**关键发现**：
- 支持全局和工作区两级配置
- 工作区配置覆盖全局配置
- 多工作区时，后面的覆盖前面的

---

### 6. debug/ - 调试目录（临时）

**用途**：存储调试信息和执行日志

**创建时机**：执行调试命令时临时创建

**目录结构**（行 799921-799938）：
```javascript
// 创建 debug 目录
await mkdir(uri2.fsPath + "/.kiro/debug", { recursive: true });

// 写入调试日志
await writeFile(uri2.fsPath + "/.kiro/debug/debug.log", debugLog, "utf8");

// 写入执行日志
await writeFile(
  uri2.fsPath + "/.kiro/debug/execution-log.json",
  await vscode106.workspace.fs.readFile(vscode106.Uri.file(paths.obj)),
  "utf8"
);

// 创建 chats 目录
await mkdir(uri2.fsPath + "/.kiro/debug/chats", { recursive: true });

// 写入聊天记录
for (let i13 = 0; i13 < paths.chats.length; i13++) {
  await writeFile(
    uri2.fsPath + `/.kiro/debug/chats/${i13 + 1}.chat`,
    await vscode106.workspace.fs.readFile(vscode106.Uri.file(paths.chats[i13])),
    "utf8"
  );
}
```

**文件内容**：
- `debug.log` - 调试日志
- `execution-log.json` - 执行日志（JSON 格式）
- `chats/*.chat` - 聊天记录

**关键发现**：
- 临时目录，不持久化
- 用于调试和问题排查
- 包含完整的执行上下文

---

## 排除规则

### Codebase 索引排除（行 569450）
```javascript
const excludePattern = "{**/node_modules/**,**/dist/**,**/build/**,**/.git/**,**/.github/**,**/.circleci/**,**/.next/**,**/.venv/**,**/coverage/**,**/.kiroignore,**/.kiro/**,}";
```

**关键发现**：
- `.kiro/` 目录被排除在 Codebase 索引之外
- 避免索引配置文件和临时文件
- 提高索引性能

---

## 路径获取函数

### getHomeKiroPath()（行 161194）
```javascript
return path2.join(homePath, ".kiro");
// Windows: C:\Users\{用户名}\.kiro
// macOS/Linux: ~/.kiro
```

### getWorkspaceKiroPath()（行 161197）
```javascript
return path2.join(workspaceDir, ".kiro");
// 结果：{workspace}/.kiro
```

---

## 使用场景总结

| 目录 | 用途 | 文件格式 | 级别 | 递归扫描 |
|------|------|---------|------|---------|
| `steering/` | AI 行为规则 | `.md` | 全局 + 工作区 | ✅ |
| `specs/` | 功能规范 | `.md` | 工作区 | ✅ |
| `hooks/` | 自动化钩子 | `.kiro.hook` | 工作区 | ✅ |
| `agents/` | 自定义 Agent | `.md` | 工作区 | ❓ |
| `settings/` | 配置文件 | `.json` | 全局 + 工作区 | ❌ |
| `debug/` | 调试信息 | 多种 | 工作区 | ❌ |

---

## 关键发现

### 1. 双层配置系统
- **全局级别**：`~/.kiro/` - 跨工作区共享
- **工作区级别**：`{workspace}/.kiro/` - 项目特定

### 2. 配置优先级
- Steering：全局 + 工作区（都加载）
- MCP：工作区覆盖全局
- Specs/Hooks：仅工作区

### 3. 文件监控
- 所有配置目录都有 FileSystemWatcher
- 文件变更自动刷新
- 支持热重载

### 4. 递归扫描
- Steering、Specs、Hooks 都支持子目录
- 可以按功能模块组织文件
- 提高可维护性

### 5. 排除规则
- `.kiro/` 目录不参与 Codebase 索引
- 避免索引配置文件
- 提高性能

---

## 与 kiro-gateway 的关系

**kiro-gateway 不需要实现 `.kiro` 目录功能**：
- `.kiro` 是 Kiro IDE 的客户端配置
- kiro-gateway 是服务端网关
- 两者职责不同

**但可以参考的设计思路**：
- 双层配置系统（全局 + 项目）
- 文件监控和热重载
- 递归扫描和模块化组织

---

## 相关文档

- Kiro IDE 路径分析：`docs/technical/kiro-paths-analysis.md`
- Kiro IDE 路径使用分析：`docs/technical/kiro-paths-usage-analysis.md`
- Kiro IDE 日志分析：`.kiro/steering/kiro-ide-logs.md`
- Kiro IDE 源码参考：`.kiro/steering/kiro-source.md`
