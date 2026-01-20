# Kiro IDE Context Providers 源码分析

## 版本信息
- Kiro IDE 版本：v0.8.140
- 分析日期：2026-01-20
- 源码位置：`C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js`

---

## 概述

Context Providers 是 Kiro IDE 的核心功能之一，允许用户通过 `#` 符号引用各种上下文信息（文件、代码、终端、Git Diff 等）。

**完整列表**（27 个）：
1. file - Files（submenu，支持行范围）
2. spec - Spec（submenu）
3. code - Code（submenu，依赖索引）
4. currentFile - Current File（normal）
5. database - Database（submenu）
6. diff - Git Diff（normal）
7. docs - Docs（submenu）
8. tree - File Tree（normal）
9. folder - Folder（submenu，依赖索引）
10. issue - GitHub Issues（submenu）
11. gitlab-mr - GitLab Merge Request（normal）
12. google - Google（query）
13. http - HTTP（normal）
14. jira - Jira Issues（submenu）
15. locals - Locals（submenu）
16. os - Operating System（normal）
17. open - Open Files（normal）
18. postgres - PostgreSQL（submenu）
19. problems - Problems（normal）
20. search - Search（query，使用 ripgrep）
21. terminal - Terminal（normal）
22. url - URL（query）
23. repo-map - Repository Map（submenu，依赖索引）
24. greptile - Greptile（query）
25. mcp - MCP（submenu）
26. steering - Steering（submenu）
27. codebase - Codebase（normal，使用 Embeddings）

---

## Context Provider 类型

### 1. normal
- 直接获取上下文，无需额外输入
- 例如：currentFile、diff、terminal、problems、os

### 2. submenu
- 需要从子菜单选择项目
- 例如：file、code、folder、steering、mcp

### 3. query
- 需要用户输入查询字符串
- 例如：url、search、google

---

## 详细分析

### 1. file - Files（行 203648-203700）⭐⭐⭐⭐⭐

**类型**：submenu  
**依赖索引**：是  
**支持行范围**：是（file.ts:42 或 file.ts:42-64）

**核心功能**：
```javascript
// 解析查询格式
parseQuery(query) {
  const trimmed = query.trim();
  const colonIndex = trimmed.lastIndexOf(":");
  
  if (colonIndex === -1) {
    return { filePath: trimmed, isValid: true };
  }
  
  const filePath = trimmed.substring(0, colonIndex);
  const lineSpec = trimmed.substring(colonIndex + 1);
  
  // 支持行范围：file.ts:42-64
  if (lineSpec.includes("-")) {
    const parts = lineSpec.split("-");
    const startLine = parseInt(parts[0]);
    const endLine = parseInt(parts[1]);
    return { filePath, startLine, endLine, isValid: true };
  }
  
  // 单行：file.ts:42
  const lineNumber = parseInt(lineSpec);
  return { filePath, startLine: lineNumber, endLine: lineNumber, isValid: true };
}
```

**实现要点**：
- 支持三种格式：`file.ts`、`file.ts:42`、`file.ts:42-64`
- 行号从 1 开始
- 验证行号有效性（startLine <= endLine）
- 依赖文件索引系统

**适合 kiro-gateway 实现**：⭐⭐⭐⭐⭐ 必须实现

---

### 2. code - Code（行 204838-204900）⭐⭐⭐⭐

**类型**：submenu  
**依赖索引**：是  
**描述**：Type to search

**核心功能**：
```javascript
async getContextItems(query, extras) {
  // 从索引中获取代码片段
  return [await CodeSnippetsCodebaseIndex.getForId(Number.parseInt(query, 10))];
}

async loadSubmenuItems(args) {
  // 获取所有代码片段标签
  const tags = await args.ide.getTags("codeSnippets");
  const snippets = await Promise.all(tags.map((tag) => CodeSnippetsCodebaseIndex.getAll(tag)));
  
  const submenuItems = [];
  for (const snippetList of snippets) {
    submenuItems.push(...snippetList.slice(-MAX_SUBMENU_ITEMS));
  }
  return submenuItems;
}
```

**实现要点**：
- 依赖代码索引系统（CodeSnippetsCodebaseIndex）
- 使用标签（tags）组织代码片段
- 限制子菜单项数量（MAX_SUBMENU_ITEMS）

**适合 kiro-gateway 实现**：⭐⭐⭐ 可选（需要代码索引）

---

### 3. currentFile - Current File（行 204200-204280）⭐⭐⭐⭐⭐

**类型**：normal  
**描述**：Reference the currently open file

**核心功能**：
```javascript
async getContextItems(query, extras) {
  const ide = extras.ide;
  const currentFile = await ide.getCurrentFile();
  
  if (!currentFile) {
    return [];
  }
  
  const contents = await ide.readFile(currentFile);
  
  return [{
    description: currentFile,
    content: `This is the currently open file:

\`\`\`${getBasename(currentFile)}
${contents}
\`\`\``,
    name: getBasename(currentFile),
    uri: {
      type: "file",
      value: currentFile
    }
  }];
}
```

**实现要点**：
- 获取当前打开的文件路径
- 读取文件内容
- 使用 Markdown 代码块格式化

**适合 kiro-gateway 实现**：⭐⭐⭐⭐⭐ 必须实现

---

### 4. diff - Git Diff（行 227875-227950）⭐⭐⭐⭐⭐

**类型**：normal  
**描述**：Reference the current git diff

**核心功能**：
```javascript
async getContextItems(query, extras) {
  const diff = await extras.ide.getDiff();
  
  return [{
    description: "The current git diff",
    content: diff.trim() === "" 
      ? "Git shows no current changes." 
      : `\`\`\`git diff
${diff}
\`\`\``,
    name: "Git Diff"
  }];
}
```

**实现要点**：
- 调用 `git diff` 命令
- 处理无变更的情况
- 使用 git diff 格式化

**适合 kiro-gateway 实现**：⭐⭐⭐⭐⭐ 必须实现

---

### 5. terminal - Terminal（行 529012-529080）⭐⭐⭐⭐⭐

**类型**：normal  
**描述**：Reference the contents of the terminal

**核心功能**：
```javascript
async getContextItems(query, extras) {
  const content = await extras.ide.getTerminalContents();
  
  return [{
    description: "The contents of the terminal",
    content: `Current terminal contents:

${content}`,
    name: "Terminal"
  }];
}
```

**实现要点**：
- 获取终端输出内容
- 简单的文本格式化

**适合 kiro-gateway 实现**：⭐⭐⭐⭐⭐ 必须实现

---

### 6. problems - Problems（行 528935-529000）⭐⭐⭐⭐⭐

**类型**：normal  
**描述**：Reference problems in the current file

**核心功能**：
```javascript
async getContextItems(query, extras) {
  const ide = extras.ide;
  const problems = await ide.getProblems();
  
  const items = await Promise.all(
    problems.map(async (problem) => {
      const content = await ide.readFile(problem.filepath);
      const lines = content.split("\n");
      
      // 提取问题周围的代码（前后 2 行）
      const rangeContent = lines.slice(
        Math.max(0, problem.range.start.line - 2),
        problem.range.end.line + 2
      ).join("\n");
      
      return {
        description: "Problems in current file",
        content: `\`\`\`${getBasename(problem.filepath)}
${rangeContent}
\`\`\`
${problem.message}
`,
        name: `Warning in ${getBasename(problem.filepath)}`
      };
    })
  );
  
  return items.length === 0 
    ? [{ description: "Problems in current file", content: "There are no problems found in the open file.", name: "No problems found" }]
    : items;
}
```

**实现要点**：
- 获取当前文件的诊断问题
- 提取问题周围的代码上下文（前后 2 行）
- 显示问题消息
- 处理无问题的情况

**适合 kiro-gateway 实现**：⭐⭐⭐⭐⭐ 必须实现

---

### 7. search - Search（行 528700-528780）⭐⭐⭐⭐

**类型**：query  
**描述**：Use ripgrep to exact search the workspace

**核心功能**：
```javascript
async getContextItems(query, extras) {
  const results = await extras.ide.getSearchResults(query);
  
  return [{
    description: "Search results",
    content: `Results of searching codebase for "${query}":

${results}`,
    name: "Search results"
  }];
}
```

**实现要点**：
- 使用 ripgrep 搜索工作区
- 返回搜索结果

**适合 kiro-gateway 实现**：⭐⭐⭐⭐ 应该实现

---

### 8. url - URL（行 534938-535020）⭐⭐⭐⭐

**类型**：query  
**描述**：Reference a webpage at a given URL

**核心功能**：
```javascript
async getContextItems(query, extras) {
  try {
    // 规范化 URL
    let normalizedQuery = query.trim();
    if (!normalizedQuery.startsWith("http://") && !normalizedQuery.startsWith("https://")) {
      normalizedQuery = "https://" + normalizedQuery;
    }
    
    const url = new URL(normalizedQuery);
    const icon = await fetchFavicon(url);
    
    // 获取网页内容
    const headers = {
      Accept: "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,image/apng,*/*;q=0.8",
      "User-Agent": `KiroIDE ${kiroVersion}`
    };
    
    const resp = await extras.fetch(url, { headers, redirect: "follow" });
    
    if (!resp.ok) {
      return [{ name: "Error: Unable to access URL", content: "..." }];
    }
    
    const html = await resp.text();
    
    // 使用 Readability 提取可读内容
    const dom = new JSDOM(html);
    let reader = new Readability(dom.window.document);
    let article = await new Promise((resolve) => {
      setImmediate(() => {
        resolve(reader.parse());
      });
    });
    
    if (!article) {
      return [{ name: "Error: No readable content", content: "..." }];
    }
    
    // 转换为 Markdown
    const content = article.content || "";
    const markdown = NodeHtmlMarkdown.translate(content, {}, void 0, void 0);
    
    return [{
      icon,
      description: url.toString(),
      content: markdown,
      name: article.title || url.toString(),
      uri: { type: "url", value: url.toString() }
    }];
  } catch (error) {
    return [{ name: "Error", content: error.message }];
  }
}
```

**实现要点**：
- 自动添加 https:// 前缀
- 使用 Readability 提取可读内容
- 转换 HTML 到 Markdown
- 处理各种错误情况（无法访问、无内容、JavaScript 页面等）

**适合 kiro-gateway 实现**：⭐⭐⭐⭐ 应该实现

---

### 9. codebase - Codebase（行 568172-568250）⭐⭐⭐

**类型**：normal  
**依赖**：Embeddings  
**描述**：Automatically find relevant files

**核心功能**：
```javascript
async getContextItems(query, extras) {
  // 使用 Embeddings 检索相关上下文
  return retrieveContextItemsFromEmbeddings(extras, this.options, void 0);
}
```

**实现要点**：
- 依赖 Embeddings 系统
- 自动查找相关文件
- 使用向量搜索

**适合 kiro-gateway 实现**：⭐⭐⭐ 可选（需要 Embeddings）

---

### 10. mcp - MCP（行 535300-535380）⭐⭐⭐⭐⭐

**类型**：submenu  
**描述**：Model Context Protocol

**核心功能**：
```javascript
static encodeMCPResourceId(serverName, type, uri) {
  return JSON.stringify({ serverName, type, uri });
}

static decodeMCPResourceId(mcpResourceId) {
  return JSON.parse(mcpResourceId);
}

async getContextItems(query, extras) {
  const { serverName, type, uri } = decodeMCPResourceId(query);
  const connection = MCPManagerSingleton.getInstance().getConnection(serverName);
  
  if (!connection) {
    throw new Error(`No MCP connection found for ${serverName}`);
  }
  
  if (type === "resource") {
    // 读取 MCP 资源
    const { contents } = await connection.mcpClient.readResource({ uri });
    
    return await Promise.all(
      contents.map(async (resource) => {
        let content;
        
        if ("text" in resource) {
          content = resource.text;
        } else if ("blob" in resource) {
          const mimeType = resource.mimeType;
          
          if (mimeType && (mimeType.startsWith("text/") || mimeType === "application/json")) {
            content = Buffer.from(resource.blob, "base64").toString("utf-8");
          } else {
            throw new Error(`Unsupported MCP resource type: ${mimeType}`);
          }
        } else {
          throw new Error(`Unsupported MCP resource format`);
        }
        
        return {
          name: resource.uri,
          description: resource.uri,
          content,
          uri: { type: "url", value: resource.uri }
        };
      })
    );
  } else if (type === "tool") {
    return [{
      name: uri,
      description: "",
      content: uri
    }];
  } else {
    throw new Error(`Unsupported MCP context type: ${type}`);
  }
}

async loadSubmenuItems(args) {
  const contextReferences = MCPManagerSingleton.getContextReferences();
  
  return contextReferences.map((reference) => ({
    id: encodeMCPResourceId(reference.serverName, reference.type, reference.uri),
    title: `[${reference.serverName}] ${reference.name}`,
    description: reference.description,
    metadata: {
      serverName: reference.serverName,
      type: reference.type
    }
  }));
}
```

**实现要点**：
- 支持 MCP 资源（resource）和工具（tool）
- 处理文本和二进制资源
- 支持 text/*, application/json 等 MIME 类型
- 从 MCP 服务器获取上下文引用

**适合 kiro-gateway 实现**：⭐⭐⭐⭐⭐ 必须实现（如果支持 MCP）

---

### 11. steering - Steering（行 535383-535480）⭐⭐⭐⭐⭐

**类型**：submenu  
**描述**：Reference steering files in your workspace

**核心功能**：
```javascript
async getContextItems(query, extras) {
  const steeringDocs = await extras.ide.getSteering();
  
  // 判断是否需要工作区区分符
  const useWorkspaceDiscriminator = new Set(
    steeringDocs.map(({ workspaceFolderName }) => workspaceFolderName)
  ).size > 1;
  
  // 查找对应的 steering 文件
  const steeringDoc = steeringDocs.find((doc) => {
    const discriminatedName = useWorkspaceDiscriminator 
      ? `${doc.workspaceFolderName}/${doc.name}` 
      : doc.name;
    return discriminatedName === query;
  });
  
  const filePath = steeringDoc?.filePath || path.join(".kiro", "steering", query);
  
  // 读取文件内容
  let content;
  try {
    content = await extras.ide.readFile(filePath);
  } catch (error) {
    content = `Error: Unable to read steering file "${query}". File path: ${filePath}. Error: ${error.message}`;
  }
  
  const scope = steeringDoc?.scope;
  const scopeLabel = scope ? ` [${scope === "global" ? "Global" : "Workspace"}]` : "";
  const precedenceNote = scope === "workspace" 
    ? "\n  Workspace-level rules take precedence over global-level rules when conflicts exist." 
    : "";
  
  const steeringMessage = `## Included Rules (${query})${scopeLabel}

  I am providing you some additional guidance that you should follow for your entire execution. These are intended to steer you in the right direction.
  They have been automatically suggested by the system and may be unrelated to my specific request which follows after them. Consider them, but your number one priority is my request.${precedenceNote}

<user-rule id=${query}>
\`\`\`
${content}
\`\`\`
</user-rule>
`;
  
  return [{
    name: `Steering: ${query}`,
    description: `Steering file: ${query}`,
    content: steeringMessage,
    uri: { type: "file", value: filePath }
  }];
}

async loadSubmenuItems(args) {
  const steeringDocs = await args.ide.getSteering();
  
  const useWorkspaceDiscriminator = new Set(
    steeringDocs.map(({ workspaceFolderName }) => workspaceFolderName)
  ).size > 1;
  
  return steeringDocs.map(({ name, workspaceFolderName, filePath }) => {
    const discriminatedName = useWorkspaceDiscriminator 
      ? `${workspaceFolderName}/${name}` 
      : name;
    
    return {
      id: discriminatedName,
      title: name,
      description: workspaceFolderName
    };
  });
}
```

**实现要点**：
- 支持全局和工作区级别的 steering 文件
- 多工作区时使用工作区名称区分
- 使用特殊的格式化消息（包含 `<user-rule>` 标签）
- 说明优先级规则（工作区 > 全局）

**适合 kiro-gateway 实现**：⭐⭐⭐⭐⭐ 必须实现

---

### 12. locals - Locals（行 528700-528780）⭐⭐

**类型**：submenu  
**描述**：Reference the contents of the local variables

**核心功能**：
```javascript
async getContextItems(query, extras) {
  const localVariables = await extras.ide.getDebugLocals(Number(query));
  const threadIndex = Number(query);
  const thread = (await extras.ide.getAvailableThreads()).find((t) => t.id === threadIndex);
  
  const callStacksSources = await extras.ide.getTopLevelCallStackSources(
    threadIndex, 
    this.options?.stackDepth || 3
  );
  
  const callStackContents = callStacksSources.reduce(
    (acc, source, index) => `${acc}

call stack ${index}
\`\`\`
${source}
\`\`\``,
    ""
  );
  
  return [{
    description: "The value, name and possibly type of the local variables",
    content: `This is a paused thread: ${thread?.name}
Current local variable contents:
${localVariables}.
Current top level call stacks: ${callStackContents}`,
    name: "Locals"
  }];
}

async loadSubmenuItems(args) {
  const threads = await args.ide.getAvailableThreads();
  
  return threads.map((thread) => ({
    id: `${thread.id}`,
    title: thread.name,
    description: `${thread.id}`
  }));
}
```

**实现要点**：
- 依赖调试器 API
- 获取局部变量和调用栈
- 支持多线程

**适合 kiro-gateway 实现**：⭐⭐ 可选（需要调试器集成）

---

### 13. os - Operating System（行 528700-528780）⭐⭐⭐⭐

**类型**：normal  
**描述**：Operating system and CPU Information

**核心功能**：
```javascript
async getContextItems(query, extras) {
  const cpu = os.arch();
  const platform = os.platform();
  
  return [{
    description: "Your operating system and CPU",
    content: `I am running ${platform === "win32" ? "Windows" : platform} on ${cpu}.`,
    name: "Operating System"
  }];
}
```

**实现要点**：
- 获取操作系统和 CPU 架构
- 简单的文本格式化

**适合 kiro-gateway 实现**：⭐⭐⭐⭐ 应该实现

---

## 实现优先级总结

### 最高优先级 ⭐⭐⭐⭐⭐ 必须实现

1. **file** - 文件引用（支持行范围）
2. **currentFile** - 当前文件
3. **diff** - Git Diff
4. **terminal** - 终端内容
5. **problems** - 代码问题
6. **steering** - Steering 规则
7. **mcp** - MCP 资源（如果支持 MCP）

### 高优先级 ⭐⭐⭐⭐ 应该实现

8. **search** - 代码搜索（ripgrep）
9. **url** - 网页内容
10. **os** - 操作系统信息

### 中优先级 ⭐⭐⭐ 可选

11. **code** - 代码片段（需要索引）
12. **codebase** - 自动查找相关文件（需要 Embeddings）
13. **folder** - 文件夹（需要索引）
14. **tree** - 文件树
15. **open** - 打开的文件

### 低优先级 ⭐⭐ 可选

16. **locals** - 局部变量（需要调试器）
17. **database** - 数据库（需要数据库连接）
18. **postgres** - PostgreSQL（需要数据库连接）

### 不推荐实现 ⭐

19. **issue** - GitHub Issues（需要 GitHub API）
20. **gitlab-mr** - GitLab MR（需要 GitLab API）
21. **jira** - Jira Issues（需要 Jira API）
22. **google** - Google 搜索（需要 Google API）
23. **http** - HTTP 请求（通用性不强）
24. **docs** - 文档（需要文档系统）
25. **spec** - Spec（需要 Spec 系统）
26. **repo-map** - 仓库地图（需要索引）
27. **greptile** - Greptile（第三方服务）

---

## 实现建议

### 架构设计

**后端 API**（Rust + Axum）：
```rust
// src-tauri/src/context_provider.rs

pub trait ContextProvider {
    fn get_type(&self) -> ContextProviderType;
    async fn get_context_items(&self, query: &str, extras: &ContextExtras) -> Result<Vec<ContextItem>>;
    async fn load_submenu_items(&self, args: &LoadSubmenuArgs) -> Result<Vec<SubmenuItem>>;
}

pub enum ContextProviderType {
    Normal,
    Submenu,
    Query,
}

pub struct ContextItem {
    pub name: String,
    pub description: String,
    pub content: String,
    pub uri: Option<ContextUri>,
}

pub struct ContextExtras {
    pub ide: Arc<IdeInterface>,
    pub fetch: Arc<FetchInterface>,
    pub selected_code: Vec<CodeSelection>,
}
```

**前端 UI**（React + TypeScript）：
```typescript
// src/components/ContextSelector.tsx

interface ContextProvider {
  title: string;
  displayTitle: string;
  description: string;
  type: 'normal' | 'submenu' | 'query';
}

function ContextSelector() {
  const [providers, setProviders] = useState<ContextProvider[]>([]);
  const [selectedProvider, setSelectedProvider] = useState<string | null>(null);
  
  // 加载 context providers
  useEffect(() => {
    fetch('/admin/context-providers')
      .then(res => res.json())
      .then(setProviders);
  }, []);
  
  // 渲染 UI
  return (
    <div>
      {providers.map(provider => (
        <ContextProviderItem 
          key={provider.title}
          provider={provider}
          onSelect={setSelectedProvider}
        />
      ))}
    </div>
  );
}
```

### API 端点设计

```
GET /admin/context-providers
  - 列出所有可用的 context providers

GET /admin/context-providers/:name
  - 获取特定 provider 的详情

POST /admin/context-providers/:name/items
  Body: { query: string }
  - 获取 context items

POST /admin/context-providers/:name/submenu
  - 加载子菜单项（仅 submenu 类型）
```

### 实现步骤

#### 第一阶段：核心功能（1-2 周）

1. **实现基础架构**
   - 定义 ContextProvider trait
   - 实现 ContextItem 和相关数据结构
   - 创建 ContextProviderManager

2. **实现必须的 providers**
   - file（支持行范围）
   - currentFile
   - diff
   - terminal
   - problems
   - steering

3. **前端集成**
   - 创建 ContextSelector 组件
   - 实现 # 符号触发
   - 实现子菜单选择

#### 第二阶段：扩展功能（1 周）

4. **实现高优先级 providers**
   - search（ripgrep）
   - url（Readability + HTML to Markdown）
   - os

5. **优化用户体验**
   - 添加搜索过滤
   - 添加最近使用记录
   - 添加快捷键支持

#### 第三阶段：可选功能（按需）

6. **实现中优先级 providers**
   - code（需要先实现代码索引）
   - codebase（需要先实现 Embeddings）
   - folder、tree、open

---

## 技术依赖

### 必需依赖

1. **ripgrep** - 代码搜索
   - Rust crate: `grep-searcher`, `grep-matcher`
   - 或直接调用 `rg` 命令

2. **Git** - Git Diff
   - 调用 `git diff` 命令

3. **文件系统** - 文件读取
   - Rust 标准库 `std::fs`

### 可选依赖

4. **Readability** - 网页内容提取
   - Node.js: `@mozilla/readability`
   - Rust: 可能需要调用 Node.js 或使用其他库

5. **HTML to Markdown** - HTML 转换
   - Node.js: `node-html-markdown`
   - Rust: `html2md` crate

6. **Embeddings** - 代码搜索
   - TransformersJS: `all-MiniLM-L6-v2`
   - 或使用 Rust 的 `candle` 框架

---

## 与 Kiro IDE 的对比

| 功能 | Kiro IDE | kiro-gateway | 实现难度 |
|------|----------|--------------|---------|
| file | ✅ | ⚠️ 待实现 | 中等 |
| currentFile | ✅ | ⚠️ 待实现 | 简单 |
| diff | ✅ | ⚠️ 待实现 | 简单 |
| terminal | ✅ | ⚠️ 待实现 | 简单 |
| problems | ✅ | ⚠️ 待实现 | 中等 |
| steering | ✅ | ⚠️ 待实现 | 简单 |
| mcp | ✅ | ⚠️ 待实现 | 中等 |
| search | ✅ | ⚠️ 待实现 | 中等 |
| url | ✅ | ⚠️ 待实现 | 困难 |
| os | ✅ | ⚠️ 待实现 | 简单 |
| code | ✅ | ❌ 不实现 | 困难（需要索引）|
| codebase | ✅ | ❌ 不实现 | 困难（需要 Embeddings）|
| locals | ✅ | ❌ 不实现 | 困难（需要调试器）|
| issue/jira/gitlab | ✅ | ❌ 不实现 | 困难（需要第三方 API）|

---

## 参考资料

- Kiro IDE 源码：`C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js`
- Context Providers 定义：行 44574
- 各 Provider 实现：行 203000-535500
- 相关文档：`.kiro/steering/chat-session-features-comparison.md`

---

## 总结

Context Providers 是 Kiro IDE 的核心功能，提供了丰富的上下文引用能力。对于 kiro-gateway 来说：

**必须实现的核心功能**（7 个）：
- file、currentFile、diff、terminal、problems、steering、mcp

**应该实现的扩展功能**（3 个）：
- search、url、os

**可选的高级功能**（5 个）：
- code、codebase、folder、tree、open

**不推荐实现的功能**（12 个）：
- 需要第三方 API 或复杂依赖的 providers

通过实现核心的 10 个 context providers，kiro-gateway 可以提供与 Kiro IDE 相当的上下文管理能力，同时保持实现的简洁性和可维护性。

