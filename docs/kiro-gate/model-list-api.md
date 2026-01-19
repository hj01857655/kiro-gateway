# Kiro 模型列表 API

## 接口信息

**端点**: `GET https://codewhisperer.us-east-1.amazonaws.com/ListAvailableModels`

**请求头**:
```
Authorization: Bearer {accessToken}
x-amz-user-agent: KiroIDE-{版本}-{机器ID}
```

**查询参数**:
- `origin`: "AI_EDITOR"
- `profileArn`: 账号的 profileArn（可为空字符串）

## 响应格式

```json
{
  "models": [
    {
      "modelId": "qdev::auto",
      "tokenLimits": {
        "maxInputTokens": 200000,
        "maxOutputTokens": 8192
      }
    },
    {
      "modelId": "qdev::claude-haiku-4.5",
      "tokenLimits": {
        "maxInputTokens": 200000,
        "maxOutputTokens": 8192
      }
    },
    {
      "modelId": "qdev::claude-sonnet-4",
      "tokenLimits": {
        "maxInputTokens": 200000,
        "maxOutputTokens": 8192
      }
    },
    {
      "modelId": "qdev::claude-sonnet-4.5",
      "tokenLimits": {
        "maxInputTokens": 200000,
        "maxOutputTokens": 8192
      }
    }
  ]
}
```

## 实现说明

### 后端实现

在 `src-tauri/src/kiro_client.rs` 中实现了 `list_available_models` 方法：

```rust
pub async fn list_available_models(&self, account: &Account) -> Result<Vec<serde_json::Value>, AppError> {
    let url = format!("{}/ListAvailableModels", self.config.kiro_endpoint);
    
    let resp = self.client
        .get(&url)
        .header("Authorization", format!("Bearer {}", account.access_token))
        .header("x-amz-user-agent", self.get_user_agent())
        .query(&[
            ("origin", "AI_EDITOR"),
            ("profileArn", &account.profile_arn),
        ])
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await?;
    
    // 处理响应...
}
```

### OpenAI 兼容接口

`GET /v1/models` 端点会：
1. 调用 Kiro API 获取模型列表
2. 移除 `qdev::` 前缀
3. 转换为 OpenAI 格式
4. 如果调用失败，返回默认模型列表

### 前端实现

在 `src/api/models.ts` 中实现了带缓存的模型加载：

```typescript
export const modelsApi = {
  async list(forceRefresh = false): Promise<Model[]> {
    // 5 分钟缓存
    if (!forceRefresh && modelsCache && now - cacheTimestamp < CACHE_DURATION) {
      return modelsCache
    }
    
    const res = await fetch('/v1/models')
    const data: ModelsResponse = await res.json()
    modelsCache = data.data
    cacheTimestamp = now
    
    return data.data
  }
}
```

## 注意事项

1. **模型可用性**：返回的模型列表取决于账号类型和权限
2. **Token 过期**：如果 Token 过期（401/403），会自动刷新后重试
3. **降级策略**：如果 Kiro API 调用失败，返回默认模型列表
4. **缓存策略**：前端缓存 5 分钟，减少 API 调用
5. **已移除模型**：Claude Opus 4.5 已不再可用（2026-01-15 测试）

## 默认模型列表

当无法从 Kiro API 获取时，使用以下默认列表：

- `claude-haiku-4.5` - 快速模型
- `claude-sonnet-4` - 常规模型
- `claude-sonnet-4.5` - 推荐模型

## 参考

- KiroGate 实现：`E:\VSCodeSpace\Kiro\KiroGate\kiro_gateway\cache.py`
- 本地实现：`src-tauri/src/kiro_client.rs` (line 636+)
- 前端 API：`src/api/models.ts`
