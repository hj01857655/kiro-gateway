# Kiro IDE 源码分析

## 源码位置

`C:\Users\12925\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\`

## 文件访问限制

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

## 解决方案

访问工作区外的文件必须使用 PowerShell 命令：

```powershell
# 读取文件
Get-Content "E:\VSCodeSpace\Kiro\外部项目\文件路径" -Raw

# 列出目录
Get-ChildItem "E:\VSCodeSpace\Kiro\外部项目"

# 搜索文件
Get-ChildItem "E:\VSCodeSpace\Kiro\外部项目" -Recurse -Filter "*.py"
```

**示例**：访问 KiroGate (Python 原项目)
```powershell
Get-Content "E:\VSCodeSpace\Kiro\KiroGate\kiro_gateway\metrics.py" -Raw
```

**示例**：访问 kiro-gateway (你的 Rust 项目)
```powershell
Get-Content "E:\VSCodeSpace\Kiro\kiro-gateway\src\main.rs" -Raw
```

## 相关文档

- 源码分析文档：`docs/kiro-source-analysis/`
- Machine ID: `docs/kiro-source-analysis/machine-id.md`
- Social Auth: `docs/kiro-source-analysis/social-auth-provider.md`
- SSO OIDC: `docs/kiro-source-analysis/sso-oidc-client.md`
