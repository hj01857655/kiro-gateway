# Kiro IDE 源码分析

## 源码位置

**Kiro IDE 安装目录**:
```
C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\
```

**依赖包目录**:
```
C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\node_modules\
```

**源码分析项目**:
```
E:\VSCodeSpace\Kiro\kiro-agent-source-analysis
```

这个项目用于存放 Kiro IDE 源码的**分析文档**，而不是直接复制源码。

### 源码分析项目更新规则

**每次分析 Kiro IDE 源码时，应该：**

1. **创建分析文档**
   - 文件命名格式：`功能名-分析.md` 或 `功能名-实现原理.md`
   - 例如：`MCP-Validation-Analysis.md`、`Tool-Deduplication-Analysis.md`

2. **文档内容结构**
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
   
   ## 调用流程
   用流程图或文字描述函数调用关系
   
   ## 注意事项
   记录特殊处理、边界条件、已知问题
   
   ## 参考位置
   - 文件：dist/extension.js
   - 行号：约 12345-12400
   - 相关函数：functionName()
   ```

3. **代码片段原则**
   - ✅ **只摘录关键逻辑**（10-30行核心代码）
   - ✅ **添加详细注释**，解释每个步骤的作用
   - ✅ **简化变量名**，提高可读性
   - ✅ **标注来源**（文件路径、行号）
   - ❌ **不要**直接复制大段源码
   - ❌ **不要**包含无关的辅助代码
   - ❌ **不要**保留混淆后的变量名

4. **分析重点**
   - 功能的**设计思路**和**实现原理**
   - 关键**算法**和**数据结构**
   - 与其他模块的**交互方式**
   - 可能的**优化点**和**改进方向**

5. **使用 PowerShell 查询源码**
   ```powershell
   # 搜索关键词（推荐）
   Select-String -Path "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js" -Pattern "关键词" -Context 5,5
   
   # 读取分析文档
   Get-Content "E:\VSCodeSpace\Kiro\kiro-agent-source-analysis\功能名-分析.md" -Raw
   ```

**示例：好的分析文档 vs 不好的做法**

✅ **好的做法**：
```markdown
## 工具去重逻辑

### 实现原理
使用 Set 数据结构存储已调用的工具名称，每次调用前检查是否重复。

### 关键代码（简化）
\`\`\`javascript
// 工具去重检查（来源：extension.js 行 12345）
const calledTools = new Set();

function shouldCallTool(toolName) {
  if (calledTools.has(toolName)) {
    console.log(`工具 ${toolName} 已调用，跳过`);
    return false;
  }
  calledTools.add(toolName);
  return true;
}
\`\`\`

### 设计思路
- 使用 Set 保证 O(1) 查询效率
- 在每轮对话开始时清空 Set
- 支持通配符匹配（如 `tool_*`）
```

❌ **不好的做法**：
```markdown
## 工具去重

直接复制 500 行混淆后的源码...
var a1=function(b2,c3){var d4=new Set();...
```

## 源码文件结构

### 主要目录
- `dist/` - 编译后的源码（打包压缩）
- `node_modules/` - NPM 依赖包（包含 MCP SDK 等）
- `extension-resources/` - 扩展资源文件
- `models/` - AI 模型相关
- `packages/` - 子包
- `tree-sitter/` - 语法解析器
- `treesitter-wasm/` - WebAssembly 语法解析器

### dist 目录文件
- `extension.js` (39.58 MB) - 主扩展代码（包含所有功能）
- `llamaTokenizer.mjs` (0.64 MB) - Llama 分词器
- `llamaTokenizerWorkerPool.mjs` - Llama 分词器工作池
- `tiktokenWorkerPool.mjs` - Tiktoken 分词器工作池
- `xhr-sync-worker.js` - 同步 XHR 工作线程

### 配置文件
- `package.json` - 扩展配置
- `package.nls.json` - 国际化配置
- `tailwind.config.js` - Tailwind CSS 配置
- `tsconfig.common.json` - TypeScript 通用配置
- `tsconfig.test.json` - TypeScript 测试配置

### 文档文件
- `changelog.md` - 更新日志
- `readme.md` - 说明文档
- `LICENSE.txt` - 许可证
- `NOTICE` - 版权声明
- `CLEANUP_SUMMARY.md` - 清理摘要
- `RELEASE_STAGING_INFO.md` - 发布暂存信息

### UnauthorizedFileAccessError

**文件**: `dist/extension.js` (行 846846-846850)

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

### isPathAllowed 函数

**文件**: `dist/extension.js` (行 846877-846887)

```javascript
const isPathAllowed = (pathToCheck) => {
  // 允许访问 .kiro 目录
  if (pathToCheck.startsWith(kiroDir + path.sep) || pathToCheck === kiroDir) {
    return true;
  }
  // 允许访问工作区根目录
  if (pathToCheck.startsWith(workspacePath + path.sep) || pathToCheck === workspacePath) {
    return true;
  }
  // 允许访问多工作区文件夹
  for (const ws of workspaceFolders) {
    const wsPath = ws.uri.fsPath;
    if (pathToCheck.startsWith(wsPath + path.sep) || pathToCheck === wsPath) {
      return true;
    }
  }
  return false;
};
```

### 触发位置

1. **行 846872**: 没有工作区时抛出
   ```javascript
   if (!workspaceFolders || workspaceFolders.length === 0) {
     throw new UnauthorizedFileAccessError(filePath);
   }
   ```

2. **行 846893**: 路径不在允许范围内
   ```javascript
   if (!isPathAllowed(resolvedPath)) {
     throw new UnauthorizedFileAccessError(filePath);
   }
   ```

3. **行 846904**: 符号链接访问
   ```javascript
   if (stats.isSymbolicLink()) {
     throw new UnauthorizedFileAccessError(filePath, "symlink");
   }
   ```

## 安全机制

Kiro IDE 的文件访问工具（readFile、fsWrite、listDirectory 等）只能访问：

1. **工作区目录** - 当前打开的项目文件夹
2. **`.kiro` 目录** - Kiro IDE 配置目录（`~/.kiro`）
3. **多工作区文件夹** - 如果打开了多个工作区

**禁止访问**：
- 工作区外的任意路径
- 符号链接（在不受信任的工作区中）

## 访问方式

### 访问 Kiro IDE 源码（工作区外）

由于 Kiro IDE 源码在工作区外，必须使用 PowerShell 访问：

```powershell
# 读取主扩展文件（39.58 MB，较大）
Get-Content "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js" -Raw

# 读取其他源码文件
Get-Content "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\llamaTokenizer.mjs" -Raw
Get-Content "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\xhr-sync-worker.js" -Raw

# 访问 node_modules 依赖包
Get-Content "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\node_modules\@modelcontextprotocol\sdk\dist\esm\types.js" -Raw

# 列出 node_modules 中的包
dir "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\node_modules"

# 搜索关键词（推荐用于大文件）
Select-String -Path "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js" -Pattern "关键词" -Context 2,2

# 在 node_modules 中搜索
Select-String -Path "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\node_modules\@modelcontextprotocol\sdk\dist\esm\*.js" -Pattern "关键词" -Context 2,2

# 列出所有文件
dir "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist"

# 列出所有 JS 文件（递归）
Get-ChildItem "C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent" -Recurse -Filter "*.js"
```

### 访问源码分析项目（工作区外）

```powershell
# 读取分析文档
Get-Content "E:\VSCodeSpace\Kiro\kiro-agent-source-analysis\MCP-Validation-Analysis.md" -Raw

# 读取提取的代码片段
Get-Content "E:\VSCodeSpace\Kiro\kiro-agent-source-analysis\mcp-validation-v0.8.140.js" -Raw

# 列出所有分析文档
Get-ChildItem "E:\VSCodeSpace\Kiro\kiro-agent-source-analysis" -Filter "*.md"
```

## 源码分析工作流

### 查询源码后的更新流程

每次查询 Kiro IDE 源码并分析功能后，必须更新源码分析项目：

1. **创建分析文档**
   ```powershell
   # 在 E:\VSCodeSpace\Kiro\kiro-agent-source-analysis 创建 Markdown 文档
   # 命名格式: {功能名}-v{版本号}.md
   # 例如: MCP-Tool-Count-Warning-v0.8.140.md
   ```

2. **文档内容结构**
   - 版本信息（Kiro IDE 版本、分析日期）
   - 功能概述
   - 源码位置（文件路径、关键行号）
   - 核心代码片段
   - 实现逻辑分析
   - 设计考虑
   - 适配建议（如何在本项目中实现）
   - 相关文件列表
   - 更新记录

3. **提取代码片段**（可选）
   ```powershell
   # 如果代码片段较大，单独保存为 .js 文件
   # 命名格式: {功能名}-v{版本号}.js
   ```

4. **更新 README**
   - 在 `E:\VSCodeSpace\Kiro\kiro-agent-source-analysis\readme.md` 中添加新文档链接
   - 更新版本信息

### 已分析的功能

#### 0.8.140 版本
- **MCP Schema 验证**: `MCP-Schema-Validation-v0.8.140.md`
- **MCP 工具数量警告**: `MCP-Tool-Count-Warning-v0.8.140.md`
- **.kiroignore 支持**: `Kiroignore-Support-v0.8.140.md`

#### 早期版本
- **MCP 验证**: `MCP-Validation-Analysis.md`

## 相关文档

### 本项目文档
- 源码分析文档：`docs/kiro-source-analysis/`
- Machine ID: `docs/kiro-source-analysis/machine-id.md`
- Social Auth: `docs/kiro-source-analysis/social-auth-provider.md`
- SSO OIDC: `docs/kiro-source-analysis/sso-oidc-client.md`

### 源码分析项目
- **项目路径**: `E:\VSCodeSpace\Kiro\kiro-agent-source-analysis`
- **版本**: 基于 Kiro IDE 0.8.140
- **文档列表**:
  - `MCP-Validation-Analysis.md` - MCP 配置验证
  - `MCP-Schema-Validation-v0.8.140.md` - Schema 验证逻辑
  - `MCP-Tool-Count-Warning-v0.8.140.md` - 工具数量警告
  - `mcp-validation-v0.8.140.js` - 提取的代码片段

### 其他参考项目
- **kiro-gateway**: `E:\VSCodeSpace\Kiro\kiro-gateway` (Rust + Axum)
