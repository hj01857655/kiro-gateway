# Kiro 配额接口文档

## 📌 接口概览

获取 Kiro 账号的配额使用情况、订阅信息、试用状态等。

---

## 🔗 基本信息

| 项目 | 值 |
|------|-----|
| **URL** | `https://app.kiro.dev/service/KiroWebPortalService/operation/GetUserUsageAndLimits` |
| **Method** | `POST` |
| **Content-Type** | `application/cbor` |
| **协议** | Smithy RPC v2 CBOR |

---

## 📤 请求

### 请求头

```http
Content-Type: application/cbor
Accept: application/cbor
smithy-protocol: rpc-v2-cbor
Authorization: Bearer {accessToken}
Cookie: Idp={provider}; AccessToken={accessToken}
```

**provider 取值**:
- `Google` - Google 登录
- `Github` - GitHub 登录  
- `BuilderId` - AWS Builder ID 登录

### 请求体（CBOR 编码前）

```json
{
  "isEmailRequired": true,
  "origin": "KIRO_IDE"
}
```

| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| isEmailRequired | boolean | 是 | 是否需要返回邮箱 |
| origin | string | 是 | 来源标识，固定 `KIRO_IDE` |

---

## 📥 响应

### 成功响应 (200)

```json
{
  "usageBreakdownList": [
    {
      "resourceType": "AGENTIC_INTERACTIONS",
      "usageLimit": 50,
      "currentUsage": 10,
      "usageLimitWithPrecision": 50.0,
      "currentUsageWithPrecision": 10.5,
      "overageRate": 0.0,
      "overageCap": 0,
      "currency": "USD",
      "freeTrialInfo": {
        "freeTrialStatus": "ACTIVE",
        "usageLimit": 50,
        "currentUsage": 5,
        "freeTrialExpiry": 1737100800000
      },
      "bonuses": [
        {
          "bonusCode": "WELCOME_BONUS",
          "displayName": "欢迎奖励",
          "usageLimit": 100.0,
          "currentUsage": 20.0,
          "expiresAt": 1737100800000,
          "status": "ACTIVE"
        }
      ]
    }
  ],
  "subscriptionInfo": {
    "type": "FREE",
    "subscriptionTitle": "Free Tier",
    "overageCapability": "DISABLED",
    "upgradeCapability": "ENABLED"
  },
  "userInfo": {
    "email": "user@example.com",
    "userId": "d-9067642ac7.12345678-1234-1234-1234-123456789012",
    "idp": "Google",
    "status": "ACTIVE"
  },
  "daysUntilReset": 15,
  "nextDateReset": 1737100800000
}
```

### 响应字段说明

#### 顶层字段

| 字段 | 类型 | 说明 |
|------|------|------|
| usageBreakdownList | array | 配额使用明细列表 |
| subscriptionInfo | object | 订阅信息 |
| userInfo | object | 用户信息 |
| daysUntilReset | number | 距离配额重置天数 |
| nextDateReset | number | 下次重置时间戳（毫秒） |

#### usageBreakdownList[] 字段

| 字段 | 类型 | 说明 |
|------|------|------|
| resourceType | string | 资源类型，固定 `AGENTIC_INTERACTIONS` |
| usageLimit | number | 配额上限 |
| currentUsage | number | 当前使用量 |
| usageLimitWithPrecision | number | 精确配额上限 |
| currentUsageWithPrecision | number | 精确当前使用量 |
| freeTrialInfo | object | 试用信息 |
| bonuses | array | 奖励配额列表 |

#### freeTrialInfo 字段

| 字段 | 类型 | 说明 |
|------|------|------|
| freeTrialStatus | string | 试用状态 `ACTIVE`/`EXPIRED` |
| usageLimit | number | 试用配额上限 |
| currentUsage | number | 试用已使用量 |
| freeTrialExpiry | number | 试用过期时间戳（毫秒） |

#### bonuses[] 字段

| 字段 | 类型 | 说明 |
|------|------|------|
| bonusCode | string | 奖励代码 |
| displayName | string | 显示名称 |
| usageLimit | number | 奖励配额上限 |
| currentUsage | number | 奖励已使用量 |
| expiresAt | number | 过期时间戳（毫秒） |
| status | string | 状态 `ACTIVE`/`EXPIRED` |

#### subscriptionInfo 字段

| 字段 | 类型 | 说明 |
|------|------|------|
| type | string | 订阅类型 `FREE`/`PRO` |
| subscriptionTitle | string | 订阅名称 |
| overageCapability | string | 超额能力 `ENABLED`/`DISABLED` |
| upgradeCapability | string | 升级能力 `ENABLED`/`DISABLED` |

#### userInfo 字段

| 字段 | 类型 | 说明 |
|------|------|------|
| email | string | 用户邮箱 |
| userId | string | 用户 ID |
| idp | string | 身份提供者 `Google`/`Github`/`BuilderId` |
| status | string | 账号状态 `ACTIVE` |

---

## ❌ 错误响应

### 401 Unauthorized - Token 过期

```json
{
  "__type": "UnauthorizedException",
  "message": "Token expired or invalid"
}
```

**处理**: 需要刷新 Token

### 403 Forbidden - 账号暂停

```json
{
  "reason": "TEMPORARILY_SUSPENDED",
  "message": "Your account has been temporarily suspended"
}
```

**处理**: 账号被暂停，需要联系支持

### 423 Locked - 账号封禁

```json
{
  "__type": "com.amazon.kirowebportalservice#AccountSuspendedException",
  "message": "Your User ID (xxx) temporarily is suspended. We detected unusual user activity..."
}
```

**处理**: 账号被封禁，需要联系支持恢复

---

## 💻 代码示例

### Rust (使用 ciborium)

```rust
use ciborium;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GetUserUsageAndLimitsRequest {
    #[serde(rename = "isEmailRequired")]
    is_email_required: bool,
    origin: String,
}

async fn get_usage(access_token: &str, provider: &str) -> Result<serde_json::Value, String> {
    let url = "https://app.kiro.dev/service/KiroWebPortalService/operation/GetUserUsageAndLimits";
    
    let request = GetUserUsageAndLimitsRequest {
        is_email_required: true,
        origin: "KIRO_IDE".to_string(),
    };
    
    // CBOR 编码
    let mut body = Vec::new();
    ciborium::into_writer(&request, &mut body)
        .map_err(|e| format!("CBOR encode error: {}", e))?;
    
    let cookie = format!("Idp={}; AccessToken={}", provider, access_token);
    
    let response = reqwest::Client::new()
        .post(url)
        .header("Content-Type", "application/cbor")
        .header("Accept", "application/cbor")
        .header("smithy-protocol", "rpc-v2-cbor")
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Cookie", cookie)
        .body(body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    let bytes = response.bytes().await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    
    // CBOR 解码
    let result: serde_json::Value = ciborium::from_reader(&bytes[..])
        .map_err(|e| format!("CBOR decode error: {}", e))?;
    
    Ok(result)
}
```

### JavaScript (使用 cbor-x)

```javascript
import { encode, decode } from 'cbor-x';

async function getUsage(accessToken, provider) {
  const url = 'https://app.kiro.dev/service/KiroWebPortalService/operation/GetUserUsageAndLimits';
  
  const body = encode({
    isEmailRequired: true,
    origin: 'KIRO_IDE'
  });
  
  const response = await fetch(url, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/cbor',
      'Accept': 'application/cbor',
      'smithy-protocol': 'rpc-v2-cbor',
      'Authorization': `Bearer ${accessToken}`,
      'Cookie': `Idp=${provider}; AccessToken=${accessToken}`
    },
    body
  });
  
  const arrayBuffer = await response.arrayBuffer();
  const result = decode(new Uint8Array(arrayBuffer));
  
  return result;
}
```

### Python (使用 cbor2)

```python
import cbor2
import requests

def get_usage(access_token: str, provider: str) -> dict:
    url = "https://app.kiro.dev/service/KiroWebPortalService/operation/GetUserUsageAndLimits"
    
    body = cbor2.dumps({
        "isEmailRequired": True,
        "origin": "KIRO_IDE"
    })
    
    headers = {
        "Content-Type": "application/cbor",
        "Accept": "application/cbor",
        "smithy-protocol": "rpc-v2-cbor",
        "Authorization": f"Bearer {access_token}",
        "Cookie": f"Idp={provider}; AccessToken={access_token}"
    }
    
    response = requests.post(url, headers=headers, data=body)
    result = cbor2.loads(response.content)
    
    return result
```

---

## 📝 注意事项

1. **CBOR 编码**: 请求和响应都使用 CBOR 格式，需要使用对应语言的 CBOR 库
2. **Token 有效期**: accessToken 通常有效期为 1 小时，过期需要刷新
3. **错误处理**: 
   - 401 → 刷新 Token 后重试
   - 403/423 → 账号异常，停止使用
4. **配额计算**: 
   - 总配额 = usageLimit + freeTrialInfo.usageLimit + sum(bonuses.usageLimit)
   - 已使用 = currentUsage + freeTrialInfo.currentUsage + sum(bonuses.currentUsage)

---

## 🔗 相关文档

- [Kiro API 完整文档](./Kiro%20API.md)
- [配额获取详细说明](./get-usage.md)
