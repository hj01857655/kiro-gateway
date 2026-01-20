# Context Providers 实现方案

## 版本信息
- 创建日期：2026-01-20
- 基于：Kiro IDE 源码分析

---

## 核心发现

### 1. Context Providers 的工作流程

通过分析 Kiro IDE 源码（行 579274-579292）和实际会话文件，发现 Context Providers 的工作流程：

```javascript
// 1. 用户选择 Context Provider
const selectedProviders = quickPick.selectedItems;

// 2. 调用 getContextItems() 获取上下文
const contextItems = await Promise.all(
  selectedProviders.map(provider => 
    provider.getContextItems("", {
      config, ide, embeddingsProvider, reranker, llm,
      fullInput: "", selectedCode: [], fetch
    })
  )
);

// 3. 拼接成字符串
const contextString = contextItems
  .map(item => item.content)
  .join("\n\n") + "\n\n---\n\n";

// 4. 添加到用户消息中（在客户端完成）
```

### 2. 实际使用方式（从会话文件中发现）

**Steering 文件的引用格式**：

当用户通过 `#steering` 引用文件时，Kiro IDE 会将文件内容包裹在特殊格式中：

```
## Included Rules (文件名) [Workspace/Global]

  说明文字...

<user-rule id=文件名>
```
文件完整内容
```
</user-rule>
```

**示例**（从实际会话中提取）：

```
## Included Rules (kiro-gateway/context-providers-implementation-plan.md) [Workspace]

  I am providing you some additional guidance that you should follow for your entire execution...

<user-rule id=kiro-gateway/context-providers-implementation-plan.md>
```
# Context Providers 实现方案

## 版本信息
- 创建日期：2026-01-20
...（完整文件内容）
```
</user-rule>
```

**关键结论**：
- ✅ 文件内容被完整拼接到消息中
- ✅ 使用 `<user-rule>` 标签包裹
- ✅ 添加到系统提示词（不是用户消息）
- ✅ 没有特殊的 API 参数，就是标准文本

### 2. `#` 触发机制（聊天输入框）

**源码位置**：`chat-input.md`（行 520-550）

用户在聊天输入框中输入 `#` 字符时，会触发 Context Provider 选择器：

```typescript
// # 触发的 Mention 节点配置
{
  suggestion: {
    char: "#",                    // 触发字符
    pluginKey: R8e,               // 插件唯一标识
    command: ({ editor, range, props }) => {
      // 支持文件行号范围 (如 #file.ts:10-20)
      let lineRange = ""
      if (props.itemType === "file" && text) {
        const colonIndex = text.lastIndexOf(":")
        if (colonIndex !== -1) {
          const suffix = text.substring(colonIndex)
          if (/^:\d+(-\d+)?$/.test(suffix)) {
            lineRange = suffix
          }
        }
      }

      const label = props.label + lineRange
      const query = (props.query || props.id) + lineRange

      // 插入 mention 节点
      editor.chain()
        .focus()
        .insertContentAt(range, [
          { type: this.name, attrs: { ...props, label, query } },
          { type: "text", text: " " }
        ])
        .run()
    },

    // 只在行首或空格后允许触发
    allow: ({ state, range }) => {
      const resolved = state.doc.resolve(range.from)
      const nodeType = state.schema.nodes[this.name]
      const canInsert = !!resolved.parent.type.contentMatch.matchType(nodeType)
      const from = range.from
      const to = state.selection.$to.pos
      const hasSpace = state.doc.textBetween(from, to).includes(" ")
      return canInsert && !hasSpace
    }
  }
}
```

**工作流程**：

1. **用户输入 `#`** → 触发 Suggestion 插件
2. **显示 Context Provider 菜单** → 使用 MiniSearch 模糊搜索
3. **用户选择 Provider** → 插入 mention 节点到编辑器
4. **提交消息时** → 调用 `getContextItems()` 获取内容
5. **拼接到消息** → 使用 `\n\n---\n\n` 分隔符
6. **发送给 API** → 标准文本消息（包含上下文）

**支持的格式**：
- `#file` - 引用整个文件
- `#file.ts:10` - 引用第 10 行
- `#file.ts:10-20` - 引用第 10-20 行
- `#diff` - 引用 Git Diff
- `#terminal` - 引用终端内容
- `#steering` - 引用 Steering 规则

**关键结论**：
- ✅ Context Providers 在**客户端**处理
- ✅ 获取的上下文内容作为**文本**添加到消息中
- ✅ API 服务器（kiro-gateway）只接收已包含上下文的消息
- ✅ 不需要在 API 请求中传递特殊的 context 字段

---

## 架构设计

### 方案 A：纯客户端实现（推荐）⭐⭐⭐⭐⭐

**适用场景**：
- 用户使用 Kiro IDE 或其他支持 Context Providers 的客户端
- 客户端负责获取上下文并添加到消息中
- kiro-gateway 只需要处理标准的 OpenAI/Anthropic 请求

**优势**：
- ✅ 符合 Kiro IDE 的设计
- ✅ kiro-gateway 保持简洁，专注于 API 转换
- ✅ 客户端可以灵活实现各种 Context Providers
- ✅ 不需要修改 kiro-gateway 的核心逻辑

**实现**：
- kiro-gateway：无需修改，继续处理标准请求
- 客户端：实现 Context Providers 并将结果添加到消息中

---

### 方案 B：提供 Context API（可选）⭐⭐⭐

**适用场景**：
- 为简单客户端提供便利
- 客户端不想实现复杂的 Context Providers
- 需要统一的上下文获取接口

**架构**：

```
客户端                    kiro-gateway
  │                           │
  │  1. 获取上下文             │
  ├──────────────────────────>│
  │  GET /context/file?path=  │
  │                           │
  │  2. 返回文件内容           │
  │<──────────────────────────┤
  │  { content: "..." }       │
  │                           │
  │  3. 客户端拼接到消息       │
  │                           │
  │  4. 发送完整消息           │
  ├──────────────────────────>│
  │  POST /v1/chat/completions│
  │  { messages: [...] }      │
  │                           │
  │  5. 返回响应               │
  │<──────────────────────────┤
```

**API 端点**：

```
GET /context/file?path={path}&start={line}&end={line}
  - 读取文件内容（支持行范围）

GET /context/diff
  - 获取 Git Diff

GET /context/terminal
  - 获取终端内容

GET /context/problems
  - 获取代码问题

GET /context/steering?name={name}
  - 获取 Steering 规则

POST /context/search
  Body: { query: string }
  - 搜索代码

GET /context/os
  - 获取操作系统信息
```

**优势**：
- ✅ 为简单客户端提供便利
- ✅ 统一的上下文获取接口
- ✅ 可以复用 kiro-gateway 的文件访问能力

**劣势**：
- ⚠️ 增加 kiro-gateway 的复杂度
- ⚠️ 需要实现文件系统访问（安全问题）
- ⚠️ 偏离 API 网关的核心定位

---

### 方案 C：桌面应用集成（最佳）⭐⭐⭐⭐⭐

**适用场景**：
- kiro-gateway 作为 Tauri 桌面应用
- 在桌面应用的前端实现 Context Providers
- 用户通过桌面应用的聊天界面使用

**架构**：

```
┌─────────────────────────────────────────┐
│  Tauri 桌面应用                          │
│  ┌───────────────────────────────────┐  │
│  │  React 前端                        │  │
│  │  ┌─────────────────────────────┐  │  │
│  │  │  聊天界面                    │  │  │
│  │  │  - 输入框（支持 # 触发）     │  │  │
│  │  │  - Context Selector         │  │  │
│  │  │  - 消息列表                  │  │  │
│  │  └─────────────────────────────┘  │  │
│  │  ┌─────────────────────────────┐  │  │
│  │  │  Context Providers          │  │  │
│  │  │  - file, diff, terminal     │  │  │
│  │  │  - problems, steering       │  │  │
│  │  │  - search, os               │  │  │
│  │  └─────────────────────────────┘  │  │
│  └──────────┬────────────────────────┘  │
│             │ HTTP (127.0.0.1:8080)     │
│             ↓                            │
│  ┌───────────────────────────────────┐  │
│  │  Axum 后端 (HTTP API)              │  │
│  │  - /v1/chat/completions            │  │
│  │  - /v1/messages                    │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

**工作流程**：

1. **用户输入消息**：在聊天界面输入 "帮我分析这个文件 #file"
2. **触发 Context Selector**：输入 `#` 时弹出 Context Provider 选择器
3. **选择 Context Provider**：用户选择 "file" → 选择文件 → 选择行范围
4. **获取上下文**：前端调用 Tauri 命令读取文件内容
5. **拼接消息**：前端将文件内容添加到消息中
6. **发送请求**：前端调用 `/v1/chat/completions` 发送完整消息
7. **显示响应**：前端接收流式响应并显示

**优势**：
- ✅ 完全符合 Kiro IDE 的设计
- ✅ 用户体验最好（类似 Kiro IDE）
- ✅ 可以访问本地文件系统（通过 Tauri）
- ✅ 不需要修改 kiro-gateway 的 API 层
- ✅ 前端可以灵活实现各种 Context Providers

**实现要点**：

1. **Tauri 命令**（Rust）：
```rust
// src-tauri/src/main.rs

#[tauri::command]
async fn read_file(path: String, start_line: Option<usize>, end_line: Option<usize>) -> Result<String, String> {
    // 读取文件内容
}

#[tauri::command]
async fn get_git_diff() -> Result<String, String> {
    // 调用 git diff
}

#[tauri::command]
async fn get_terminal_contents() -> Result<String, String> {
    // 获取终端内容（需要集成终端）
}

#[tauri::command]
async fn search_code(query: String) -> Result<String, String> {
    // 使用 ripgrep 搜索
}
```

2. **前端 Context Providers**（React + TypeScript）：
```typescript
// src/context/providers/FileProvider.ts

export class FileProvider implements ContextProvider {
  type = 'submenu';
  title = 'file';
  displayTitle = 'Files';
  
  async getContextItems(query: string): Promise<ContextItem[]> {
    const { path, startLine, endLine } = parseFileQuery(query);
    const content = await invoke('read_file', { path, startLine, endLine });
    
    return [{
      name: path,
      description: path,
      content: `\`\`\`${path}\n${content}\n\`\`\``,
      uri: { type: 'file', value: path }
    }];
  }
  
  async loadSubmenuItems(): Promise<SubmenuItem[]> {
    // 列出工作区文件
  }
}
```

3. **聊天界面集成**（React）：
```typescript
// src/pages/Chat.tsx

function Chat() {
  const [message, setMessage] = useState('');
  const [contextItems, setContextItems] = useState<ContextItem[]>([]);
  
  const handleSubmit = async () => {
    // 1. 拼接上下文
    const contextString = contextItems
      .map(item => item.content)
      .join('\n\n') + '\n\n---\n\n';
    
    // 2. 构建完整消息
    const fullMessage = contextString + message;
    
    // 3. 发送请求
    const response = await fetch('http://127.0.0.1:8080/v1/chat/completions', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        model: 'claude-sonnet-4.5',
        messages: [{ role: 'user', content: fullMessage }],
        stream: true
      })
    });
    
    // 4. 处理流式响应
    // ...
  };
  
  return (
    <div>
      <ContextSelector onSelect={setContextItems} />
      <MessageInput value={message} onChange={setMessage} />
      <button onClick={handleSubmit}>Send</button>
    </div>
  );
}
```

---

## 推荐方案

### 方案 D：作为 Tools 实现（最简单）⭐⭐⭐⭐⭐

**适用场景**：
- 用户希望在对话过程中 AI 主动调用
- 不需要搭建 UI 界面
- 像 tools 一样自动触发

**实现方式**：

将 Context Providers 实现为标准的 Tool Calling 工具：

```rust
// src-tauri/src/context_tools.rs

pub fn get_context_tools() -> Vec<Tool> {
    vec![
        Tool {
            name: "read_file".to_string(),
            description: "读取文件内容（支持行范围）".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "文件路径" },
         

**实现优先级**：

### 第一阶段：基础架构（1-2 天）

1. **创建聊天页面**
   - 消息输入框
   - 消息列表
   - 流式响应显示

2. **实现 Tauri 命令**
   - `read_file` - 读取文件
   - `get_git_diff` - Git Diff
   - `search_code` - 代码搜索

### 第二阶段：核心 Context Providers（2-3 天）

3. **实现必须的 providers**
   - file（支持行范围）
   - currentFile
   - diff
   - terminal
   - problems
   - steering

4. **实现 Context Selector**
   - `#` 符号触发
   - 下拉菜单选择
   - 子菜单支持

### 第三阶段：扩展功能（1-2 天）

5. **实现高优先级 providers**
   - search（ripgrep）
   - url（网页内容）
   - os（操作系统信息）

6. **优化用户体验**
   - 上下文预览
   - 最近使用记录
   - 快捷键支持

---

## 与 kiro-gateway 定位的关系

**kiro-gateway 的核心定位**：
- ✅ Kiro API 的网关服务
- ✅ OpenAI/Anthropic 兼容接口
- ✅ 多账号管理和 Token 刷新
- ✅ 流式响应和工具调用

**Context Providers 的定位**：
- ✅ 桌面应用的前端功能
- ✅ 增强用户体验
- ✅ 不影响 API 网关的核心功能
- ✅ 可选功能（用户可以不使用）

**结论**：
- Context Providers 应该在**桌面应用的前端**实现
- kiro-gateway 的 API 层**不需要修改**
- 这样既保持了 kiro-gateway 的简洁性，又提供了强大的功能

---

## 参考资料

- Kiro IDE 源码：`C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js`
- 关键函数：`getContextProvidersString`（行 579274-579292）
- Context Providers 分析：`.kiro/steering/context-providers-analysis.md`
- 项目定位：`.kiro/steering/kiro-gateway.md`

---

## 总结

通过分析 Kiro IDE 源码，我们发现：

1. **Context Providers 是客户端功能**
   - 在客户端获取上下文内容
   - 拼接成文本添加到消息中
   - 发送给 API 服务器

2. **kiro-gateway 不需要特殊处理**
   - 只接收标准的 OpenAI/Anthropic 请求
   - 消息中已经包含了上下文内容
   - 不需要修改 API 层

3. **最佳实现方式**
   - 在桌面应用的前端实现 Context Providers
   - 使用 Tauri 命令访问本地文件系统
   - 保持 kiro-gateway 的核心定位

这样既符合 Kiro IDE 的设计，又保持了 kiro-gateway 的简洁性！
