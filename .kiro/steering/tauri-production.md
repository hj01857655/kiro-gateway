---
inclusion: fileMatch
fileMatchPattern: "src-tauri/**/*"
---

# Tauri 生产环境规范

## 核心问题

**Tauri 生产环境不允许前端直接 fetch 访问 localhost HTTP 服务器**

即使后端在 `127.0.0.1:8080` 运行，前端的 `fetch('http://127.0.0.1:8080/api')` 也会失败。

## 解决方案

### ✅ 推荐方案：使用 Tauri invoke 命令

**后端**（`src-tauri/src/main.rs`）：

```rust
#[tauri::command]
async fn proxy_request(
    method: String,
    path: String,
    body: Option<String>,
) -> Result<String, String> {
    let url = format!("http://127.0.0.1:8080{}", path);
    let client = reqwest::Client::new();
    
    let request = match method.as_str() {
        "GET" => client.get(&url),
        "POST" => {
            let mut req = client.post(&url);
            if let Some(body_str) = body {
                req = req.header("Content-Type", "application/json").body(body_str);
            }
            req
        }
        "PATCH" => {
            let mut req = client.patch(&url);
            if let Some(body_str) = body {
                req = req.header("Content-Type", "application/json").body(body_str);
            }
            req
        }
        "DELETE" => client.delete(&url),
        _ => return Err("Unsupported method".to_string()),
    };
    
    match request.send().await {
        Ok(response) => {
            match response.text().await {
                Ok(text) => Ok(text),
                Err(e) => Err(format!("Failed to read response: {}", e)),
            }
        }
        Err(e) => Err(format!("Request failed: {}", e)),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![proxy_request])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**前端封装**（`src/lib/tauri.ts`）：

```typescript
import { invoke } from '@tauri-apps/api/core'

export async function tauriFetch(
  path: string,
  options: RequestInit = {}
): Promise<Response> {
  const method = options.method || 'GET'
  const body = options.body ? String(options.body) : undefined

  const responseText = await invoke<string>('proxy_request', {
    method,
    path,
    body,
  })

  return new Response(responseText, {
    status: 200,
    headers: { 'Content-Type': 'application/json' },
  })
}
```

**API 工具**（`src/api/utils.ts`）：

```typescript
import { tauriFetch } from '../lib/tauri'

const isTauri = '__TAURI_INTERNALS__' in window

export async function fetchWithTimeout(
  url: string,
  options: RequestInit = {}
): Promise<Response> {
  if (isTauri) {
    // 生产环境：Tauri invoke
    const path = url.replace(/^https?:\/\/[^/]+/, '')
    return tauriFetch(path, options)
  }
  
  // 开发环境：原生 fetch
  return fetch(url, options)
}
```

### ❌ 不推荐方案

1. **CSP 配置** - 无效，Tauri 2.0 不支持
2. **HTTP 插件** - 需要额外依赖，复杂度高
3. **localhost 插件** - 改变资源加载方式，有安全风险

## 开发环境 vs 生产环境

| 环境 | 前端访问方式 | 后端地址 |
|------|------------|---------|
| 开发 | Vite proxy → fetch | http://localhost:8080 |
| 生产 | Tauri invoke → Rust → reqwest | http://127.0.0.1:8080 |

## 注意事项

1. **开发者工具**：生产环境需要在 `main.rs` 中调用 `window.open_devtools()` 才能打开
2. **环境检测**：用 `'__TAURI_INTERNALS__' in window` 判断是否在 Tauri 环境
3. **路径处理**：invoke 只需要传路径（如 `/admin/accounts`），不需要完整 URL
4. **错误处理**：Tauri invoke 的错误是字符串，需要转换为 Error 对象

## 参考文档

- [Tauri 2.0 Calling Rust from Frontend](https://v2.tauri.app/develop/calling-rust/)
- [Tauri IPC 通信](https://v2.tauri.app/develop/inter-process-communication/)
