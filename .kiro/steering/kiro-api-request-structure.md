# Kiro API 请求结构分析

## 版本信息
- 分析来源：Kiro IDE q-client.log
- 分析日期：2026-01-20
- Kiro IDE 版本：v0.8.140

---

## 核心发现

### 1. conversationState 完整结构

从日志中发现，Kiro IDE 的请求包含以下字段：

```json
{
  "conversationState": {
    "conversationId": "uuid",
    "agentContinuationId": "uuid",  // ⚠️ 重要：我们当前缺少这个字段
    "agentTaskType": "vibe",        // ⚠️ 重要：我们当前缺少这个字段
    "currentMessage": {
      "userInputMessage": {
        "content": "用户消息",
        "modelId": "claude-sonnet-4.5",  // ⚠️ 注意：不带 qdev:: 前缀
        "origin": "AI_EDITOR",
        "userInputMessageContext": {
          "tools": [...],
          "toolResults": [...]  // 工具结果在这里
        }
      }
    },
    "history": [...],
    "chatTriggerType": "MANUAL"
  }
}
```

### 2. 缺失字段说明

#### agentContinuationId
- **类型**：UUID 字符串
- **作用**：标识 Agent 的连续会话
- **生成方式**：每次会话开始时生成新的 UUID
- **持久性**：在同一个会话中保持不变

#### agentTaskType
- **类型**：字符串
- **值**：`"vibe"`
- **作用**：指定 Agent 的任务类型
- **说明**：Kiro IDE 固定使用 `"vibe"` 模式

### 3. modelId 格式差异

**日志中的格式**：
```json
"modelId": "claude-sonnet-4.5"
```

**我们当前的格式**：
```json
"modelId": "qdev::claude-sonnet-4.5"
```

**结论**：
- Kiro API 接受两种格式
- 日志中使用不带前缀的格式
- 我们当前的实现（带前缀）也是正确的
- 建议：保持当前实现，因为模型列表 API 返回的就是带前缀的

---

## 工具调用流程

### 完整流程示例

#### 第1轮：用户消息 + 工具定义

```json
{
  "currentMessage": {
    "userInputMessage": {
      "content": "用户的问题",
      "modelId": "claude-sonnet-4.5",
      "origin": "AI_EDITOR",
      "userInputMessageContext": {
        "tools": [
          {
            "toolSpecification": {
              "name": "readFile",
              "description": "读取文件内容",
              "inputSchema": {
                "json": "{\"type\":\"object\",\"properties\":{...}}"
              }
            }
          }
        ]
      }
    }
  },
  "history": []
}
```

#### 第2轮：Assistant 返回工具调用

```json
{
  "assistantResponseMessage": {
    "content": "我需要读取文件来回答你的问题。",
    "toolUses": [
      {
        "toolUseId": "tooluse_WD15QVGYRfO9bZ4KPJDLtw",
        "name": "readFile",
        "input": "{\"path\":\"src/main.rs\"}"
      }
    ]
  }
}
```

#### 第3轮：用户提交工具结果

```json
{
  "currentMessage": {
    "userInputMessage": {
      "content": "",  // ⚠️ 注意：工具结果消息的 content 为空字符串
      "modelId": "claude-sonnet-4.5",
      "origin": "AI_EDITOR",
      "userInputMessageContext": {
        "toolResults": [
          {
            "toolUseId": "tooluse_WD15QVGYRfO9bZ4KPJDLtw",
            "content": [
              {
                "text": "文件内容..."
              }
            ],
            "status": "success"
          }
        ],
        "tools": [...]  // ⚠️ 工具定义仍然需要传递
      }
    }
  },
  "history": [
    {
      "userInputMessage": {
        "content": "用户的问题",
        "modelId": "claude-sonnet-4.5",
        "origin": "AI_EDITOR"
      }
    },
    {
      "assistantResponseMessage": {
        "content": "我需要读取文件来回答你的问题。",
        "toolUses": [
          {
            "toolUseId": "tooluse_WD15QVGYRfO9bZ4KPJDLtw",
            "name": "readFile",
            "input": "{\"path\":\"src/main.rs\"}"
          }
        ]
      }
    }
  ]
}
```

### 关键发现

1. **工具结果消息的 content 为空字符串**
   - 不是 `null`，不是省略，而是 `""`
   - 工具结果放在 `userInputMessageContext.toolResults` 中

2. **工具定义需要持续传递**
   - 即使在提交工具结果时，`tools` 数组仍然需要包含
   - 这样 Assistant 可以继续调用工具

3. **toolResults 结构**
   ```json
   {
     "toolUseId": "工具调用ID",
     "content": [
       {
         "text": "结果文本"
       }
     ],
     "status": "success"  // 或 "error"
   }
   ```

---

## 配额查询 API

### 端点

```
POST https://app.kiro.dev/service/KiroWebPortalService/operation/GetUserUsageAndLimits
```

### 请求头

```
Authorization: Bearer {accessToken}
Content-Type: application/cbor
Accept: application/cbor
smithy-protocol: rpc-v2-cbor
x-amz-user-agent: KiroIDE-{版本}-{机器ID}
Cookie: Idp=BuilderId; AccessToken={accessToken}
```

### 请求体（CBOR 格式）

```json
{
  "isEmailRequired": true,
  "origin": "AI_EDITOR"
}
```

### 响应结构

```json
{
  "daysUntilReset": 0,
  "limits": [],
  "nextDateReset": "2026-02-01T00:00:00.000Z",
  "overageConfiguration": {
    "overageStatus": "DISABLED"
  },
  "subscriptionInfo": {
    "overageCapability": "OVERAGE_INCAPABLE",
    "subscriptionManagementTarget": "PURCHASE",
    "subscriptionTitle": "KIRO FREE",
    "type": "Q_DEVELOPER_STANDALONE_FREE",
    "upgradeCapability": "UPGRADE_CAPABLE"
  },
  "usageBreakdownList": [
    {
      "bonuses": [],
      "currency": "USD",
      "currentOverages": 0,
      "currentOveragesWithPrecision": 0,
      "currentUsage": 0,
      "currentUsageWithPrecision": 0,
      "displayName": "Credit",
      "displayNamePlural": "Credits",
      "freeTrialInfo": {
        "currentUsage": 389,
        "currentUsageWithPrecision": 389.88,
        "freeTrialExpiry": "2026-02-06T18:17:54.171Z",
        "freeTrialStatus": "ACTIVE",
        "usageLimit": 500,
        "usageLimitWithPrecision": 500
      },
      "nextDateReset": "2026-02-01T00:00:00.000Z",
      "overageCap": 10000,
      "overageCapWithPrecision": 10000,
      "overageCharges": 0,
      "overageRate": 0.04,
      "resourceType": "CREDIT",
      "unit": "INVOCATIONS",
      "usageLimit": 50,
      "usageLimitWithPrecision": 50
    }
  ],
  "userInfo": {
    "userId": "d-9067642ac7.64281478-3051-70bc-9412-a538e6d3e881"
  }
}
```

### 配额计算

**总配额 = 基础配额 + 试用配额**

```
基础配额：currentUsage / usageLimit
试用配额：freeTrialInfo.currentUsage / freeTrialInfo.usageLimit

总使用量 = currentUsage + freeTrialInfo.currentUsage
总限额 = usageLimit + freeTrialInfo.usageLimit
使用率 = (总使用量 / 总限额) * 100%
```

**示例**：
- 基础：0 / 50
- 试用：389.88 / 500
- 总计：389.88 / 550 = 70.9%

---

## 历史消息累积

### 消息流转示例

#### 初始请求
```json
{
  "currentMessage": { "userInputMessage": { "content": "问题1" } },
  "history": []
}
```

#### 第二轮请求
```json
{
  "currentMessage": { "userInputMessage": { "content": "问题2" } },
  "history": [
    { "userInputMessage": { "content": "问题1" } },
    { "assistantResponseMessage": { "content": "回答1" } }
  ]
}
```

#### 第三轮请求（工具调用）
```json
{
  "currentMessage": {
    "userInputMessage": {
      "content": "",
      "userInputMessageContext": {
        "toolResults": [...]
      }
    }
  },
  "history": [
    { "userInputMessage": { "content": "问题1" } },
    { "assistantResponseMessage": { "content": "回答1" } },
    { "userInputMessage": { "content": "问题2" } },
    { "assistantResponseMessage": {
        "content": "回答2",
        "toolUses": [...]
      }
    }
  ]
}
```

### 关键规则

1. **currentMessage 不进入 history**
   - 当前消息是独立的，不在历史中
   - 下一轮请求时，当前消息和响应才会进入 history

2. **history 按时间顺序排列**
   - 最早的消息在前
   - 最新的消息在后

3. **工具调用也进入 history**
   - Assistant 的工具调用消息进入 history
   - 用户的工具结果消息也进入 history

---

## 与当前实现的对比

### ✅ 已正确实现

1. **基本请求结构** - conversationId、currentMessage、history、chatTriggerType
2. **工具调用流程** - toolUses 和 toolResults 的处理
3. **配额查询** - 使用 CBOR 格式，正确计算使用率
4. **历史消息管理** - 正确累积和传递历史消息

### ⚠️ 需要补充

1. **agentContinuationId** - 缺少这个字段
2. **agentTaskType** - 缺少这个字段（固定为 "vibe"）
3. **工具结果消息的 content** - 需要确保为空字符串 `""`

### 📝 建议优化

1. **添加 agentContinuationId**
   - 在会话开始时生成 UUID
   - 在同一会话中保持不变
   - 存储在 conversationState 中

2. **添加 agentTaskType**
   - 固定设置为 `"vibe"`
   - 与 Kiro IDE 保持一致

3. **验证工具结果格式**
   - 确保工具结果消息的 content 为 `""`
   - 确保 toolResults 结构正确

---

## 实现建议

### 1. 更新 KiroPayload 结构

```rust
// src-tauri/src/models.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationState {
    #[serde(rename = "conversationId")]
    pub conversation_id: String,
    
    #[serde(rename = "agentContinuationId")]
    pub agent_continuation_id: String,  // 新增
    
    #[serde(rename = "agentTaskType")]
    pub agent_task_type: String,  // 新增，固定为 "vibe"
    
    #[serde(rename = "currentMessage")]
    pub current_message: CurrentMessage,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<HistoryItem>>,
    
    #[serde(rename = "chatTriggerType")]
    pub chat_trigger_type: String,
}
```

### 2. 初始化 conversationState

```rust
// src-tauri/src/converter.rs

pub fn build_kiro_payload(
    request: &ChatCompletionRequest,
    conversation_id: Option<String>,
    agent_continuation_id: Option<String>,  // 新增参数
) -> Result<KiroPayload, AppError> {
    let conversation_id = conversation_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let agent_continuation_id = agent_continuation_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    let conversation_state = ConversationState {
        conversation_id,
        agent_continuation_id,
        agent_task_type: "vibe".to_string(),  // 固定值
        current_message: build_current_message(request)?,
        history: build_history(request)?,
        chat_trigger_type: "MANUAL".to_string(),
    };
    
    Ok(KiroPayload {
        conversation_state,
        profile_arn: "".to_string(),
    })
}
```

### 3. 会话管理

```rust
// 在 server.rs 中管理会话状态

use std::collections::HashMap;
use tokio::sync::RwLock;

struct SessionState {
    conversation_id: String,
    agent_continuation_id: String,
}

lazy_static! {
    static ref SESSIONS: RwLock<HashMap<String, SessionState>> = RwLock::new(HashMap::new());
}

async fn get_or_create_session(session_id: &str) -> SessionState {
    let mut sessions = SESSIONS.write().await;
    sessions.entry(session_id.to_string()).or_insert_with(|| {
        SessionState {
            conversation_id: uuid::Uuid::new_v4().to_string(),
            agent_continuation_id: uuid::Uuid::new_v4().to_string(),
        }
    }).clone()
}
```

---

## 参考资料

- **日志来源**：`C:\Users\12925\AppData\Roaming\Kiro\logs\20260118T144803\window2\exthost\kiro.kiroAgent\q-client.log`
- **Kiro IDE 版本**：v0.8.140
- **分析日期**：2026-01-20
- **相关文档**：`.kiro/steering/kiro-ide-logs.md`

---

## 总结

通过分析 Kiro IDE 的日志，我们发现了两个重要的缺失字段：

1. **agentContinuationId** - 用于标识 Agent 的连续会话
2. **agentTaskType** - 固定为 "vibe"，指定 Agent 的任务类型

这些字段虽然不是必需的（我们当前的实现也能工作），但为了与 Kiro IDE 保持完全一致，建议添加这些字段。

此外，我们还确认了：
- 工具结果消息的 content 应该为空字符串 `""`
- 工具定义需要在每次请求中持续传递
- 配额计算需要同时考虑基础配额和试用配额
- 历史消息按时间顺序累积，currentMessage 不在 history 中
