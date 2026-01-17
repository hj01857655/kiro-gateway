# 获取配额接口文档

## 基本信息
- **URL**: `https://app.kiro.dev/service/KiroWebPortalService/operation/GetUserUsageAndLimits`
- **Method**: `POST`
- **Content-Type**: `application/cbor`
- **协议**: Smithy RPC v2 CBOR

---

## 完整请求

```http
POST /service/KiroWebPortalService/operation/GetUserUsageAndLimits HTTP/1.1
Host: app.kiro.dev
Content-Type: application/cbor
Accept: application/cbor
smithy-protocol: rpc-v2-cbor
Authorization: Bearer eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...
Cookie: Idp=Google; AccessToken=eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...

<CBOR 编码的请求体>
```

---

## 请求头
| 头字段 | 值 | 说明 |
|--------|-----|------|
| Content-Type | application/cbor | CBOR 格式 |
| Accept | application/cbor | 接受 CBOR 响应 |
| smithy-protocol | rpc-v2-cbor | Smithy RPC 协议 |
| Authorization | Bearer {accessToken} | 访问令牌 |
| Cookie | Idp={provider}; AccessToken={accessToken} | 身份提供者和令牌 |

### provider 取值
| 值 | 说明 |
|-----|------|
| Google | Google 登录 |
| Github | GitHub 登录 |
| BuilderId | AWS Builder ID 登录 |

---

## 请求体（CBOR 编码前）

```json
{
  "isEmailRequired": true,
  "origin": "KIRO_IDE"
}
```

### 请求字段
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| isEmailRequired | boolean | 是 | 是否需要返回邮箱 |
| origin | string | 是 | 来源标识，固定 `KIRO_IDE` |

---

## 成功响应 (200)

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
  "usageBreakdown": {
    "resourceType": "AGENTIC_INTERACTIONS",
    "usageLimit": 50,
    "currentUsage": 10
  },
  "subscriptionInfo": {
    "type": "FREE",
    "subscriptionTitle": "Free Tier",
    "overageCapability": "DISABLED",
    "upgradeCapability": "ENABLED",
    "subscriptionManagementTarget": "AWS_CONSOLE"
  },
  "overageConfiguration": {
    "overageStatus": "DISABLED"
  },
  "daysUntilReset": 15,
  "nextDateReset": 1737100800000,
  "userInfo": {
    "email": "user@example.com",
    "userId": "d-9067642ac7.12345678-1234-1234-1234-123456789012",
    "idp": "Google",
    "status": "ACTIVE",
    "featureFlags": {}
  },
  "limits": []
}
```

---

## 响应字段详解

### 顶层字段
| 字段 | 类型 | 说明 |
|------|------|------|
| usageBreakdownList | array | 配额使用明细列表 |
| usageBreakdown | object | 配额使用明细（单个） |
| subscriptionInfo | object | 订阅信息 |
| overageConfiguration | object | 超额配置 |
| daysUntilReset | number | 距离重置天数 |
| nextDateReset | number | 下次重置时间戳（毫秒） |
| userInfo | object | 用户信息 |
| limits | array | 限制列表 |

### usageBreakdownList[].字段
| 字段 | 类型 | 说明 |
|------|------|------|
| resourceType | string | 资源类型 `AGENTIC_INTERACTIONS` |
| usageLimit | number | 配额上限 |
| currentUsage | number | 当前使用量 |
| usageLimitWithPrecision | number | 精确配额上限 |
| currentUsageWithPrecision | number | 精确当前使用量 |
| overageRate | number | 超额费率 |
| overageCap | number | 超额上限 |
| currency | string | 货币类型 |
| freeTrialInfo | object | 试用信息 |
| bonuses | array | 奖励配额列表 |

### freeTrialInfo 字段
| 字段 | 类型 | 说明 |
|------|------|------|
| freeTrialStatus | string | 试用状态 `ACTIVE`/`EXPIRED` |
| usageLimit | number | 试用配额上限 |
| currentUsage | number | 试用已使用量 |
| freeTrialExpiry | number | 试用过期时间戳（毫秒） |

### bonuses[].字段
| 字段 | 类型 | 说明 |
|------|------|------|
| bonusCode | string | 奖励代码 |
| displayName | string | 显示名称 |
| usageLimit | number | 奖励配额上限 |
| currentUsage | number | 奖励已使用量 |
| expiresAt | number | 过期时间戳（毫秒） |
| status | string | 状态 `ACTIVE`/`EXPIRED` |

### subscriptionInfo 字段
| 字段 | 类型 | 说明 |
|------|------|------|
| type | string | 订阅类型 `FREE`/`PRO` |
| subscriptionTitle | string | 订阅名称 |
| overageCapability | string | 超额能力 `ENABLED`/`DISABLED` |
| upgradeCapability | string | 升级能力 `ENABLED`/`DISABLED` |
| subscriptionManagementTarget | string | 订阅管理目标 |

### userInfo 字段
| 字段 | 类型 | 说明 |
|------|------|------|
| email | string | 用户邮箱 |
| userId | string | 用户 ID |
| idp | string | 身份提供者 |
| status | string | 账号状态 `ACTIVE` |
| featureFlags | object | 功能标志 |

---

## 错误响应

### 401 Unauthorized - Token 过期
```json
{
  "__type": "UnauthorizedException",
  "message": "Token expired or invalid"
}
```

### 403 Forbidden - 账号暂停
```json
{
  "reason": "TEMPORARILY_SUSPENDED",
  "message": "Your account has been temporarily suspended"
}
```

### 423 Locked - 账号封禁
```json
{
  "__type": "com.amazon.kirowebportalservice#AccountSuspendedException",
  "message": "Your User ID (d-9067642ac7.12345678-1234-1234-1234-123456789012) temporarily is suspended. We detected unusual user activity and locked it as a security precaution. To restore access, please contact our support team to verify your identity: https://support.aws.amazon.com/#/contacts/kiro"
}
```

---

## CBOR 编解码示例

### Rust
```rust
use ciborium;

// 编码
let request = serde_json::json!({
    "isEmailRequired": true,
    "origin": "KIRO_IDE"
});
let mut buf = Vec::new();
ciborium::into_writer(&request, &mut buf)?;

// 解码
let response: serde_json::Value = ciborium::from_reader(&bytes[..])?;
```

### JavaScript
```javascript
import { encode, decode } from 'cbor-x';

// 编码
const body = encode({ isEmailRequired: true, origin: 'KIRO_IDE' });

// 解码
const response = decode(responseBytes);
```

### Python
```python
import cbor2

# 编码
body = cbor2.dumps({"isEmailRequired": True, "origin": "KIRO_IDE"})

# 解码
response = cbor2.loads(response_bytes)
```
