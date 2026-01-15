# Kiro 配额查询 API 文档

## 概述

Kiro 提供配额查询接口，用于获取账号的使用情况和限制信息。

## 端点

```
GET https://q.{region}.amazonaws.com/getUsageLimits
```

**默认 region**: `us-east-1`

## 请求参数

### Query Parameters

| 参数 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `isEmailRequired` | boolean | 是 | 是否返回邮箱信息，固定为 `true` |
| `origin` | string | 是 | 来源标识，固定为 `AI_EDITOR` |
| `resourceType` | string | 是 | 资源类型，固定为 `AGENTIC_REQUEST` |
| `profileArn` | string | 否 | 仅 Social 账号需要，IDC 账号省略此参数 |

### Headers

```
Authorization: Bearer {accessToken}
x-amz-user-agent: KiroIDE-{版本}-{机器ID}
amz-sdk-invocation-id: {UUID}
amz-sdk-request: attempt=1; max=1
```

## 响应格式

### 成功响应 (200 OK)

```json
{
  "daysUntilReset": 0,
  "limits": [],
  "nextDateReset": 1769904000.0,
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
  "usageBreakdown": null,
  "usageBreakdownList": [
    {
      "bonuses": [],
      "currency": "USD",
      "currentOverages": 0,
      "currentOveragesWithPrecision": 0.0,
      "currentUsage": 0,
      "currentUsageWithPrecision": 0.08,
      "displayName": "Credit",
      "displayNamePlural": "Credits",
      "freeTrialInfo": {
        "currentUsage": 0,
        "currentUsageWithPrecision": 0.08,
        "freeTrialExpiry": 1770742909.617,
        "freeTrialStatus": "ACTIVE",
        "usageLimit": 500,
        "usageLimitWithPrecision": 500.0
      },
      "nextDateReset": 1769904000.0,
      "overageCap": 10000,
      "overageCapWithPrecision": 10000.0,
      "overageCharges": 0.0,
      "overageRate": 0.04,
      "resourceType": "CREDIT",
      "unit": "INVOCATIONS",
      "usageLimit": 50,
      "usageLimitWithPrecision": 50.0
    }
  ],
  "userInfo": {
    "email": "user@example.com",
    "userId": "d-9067642ac7.9408b4f8-d041-707d-1a73-e430875f2214"
  }
}
```

### 字段说明

#### 顶层字段

- `daysUntilReset`: 距离配额重置的天数
- `nextDateReset`: 配额重置时间戳（Unix 时间）
- `limits`: 旧版限制列表（已废弃，使用 `usageBreakdownList`）
- `usageBreakdown`: 旧版使用情况（已废弃）
- `usageBreakdownList`: 使用情况详细列表（**主要数据源**）

#### subscriptionInfo - 订阅信息

- `subscriptionTitle`: 订阅类型名称（如 `"KIRO FREE"`）
- `type`: 订阅类型标识（如 `"Q_DEVELOPER_STANDALONE_FREE"`）
- `overageCapability`: 超额能力（`"OVERAGE_INCAPABLE"` 表示不支持超额）
- `upgradeCapability`: 升级能力（`"UPGRADE_CAPABLE"` 表示可升级）
- `subscriptionManagementTarget`: 订阅管理目标

#### usageBreakdownList[0] - 使用情况详情

**基础字段**:
- `resourceType`: 资源类型（`"CREDIT"`）
- `unit`: 单位（`"INVOCATIONS"` 表示调用次数）
- `currency`: 货币单位（`"USD"`）
- `displayName`: 显示名称（单数）
- `displayNamePlural`: 显示名称（复数）

**当前使用情况**:
- `currentUsage`: 当前使用量（整数，已废弃）
- `currentUsageWithPrecision`: 当前使用量（精确值，**推荐使用**）
- `usageLimit`: 使用限制（整数，已废弃）
- `usageLimitWithPrecision`: 使用限制（精确值，**推荐使用**）

**超额相关**:
- `currentOverages`: 当前超额量（整数）
- `currentOveragesWithPrecision`: 当前超额量（精确值）
- `overageCap`: 超额上限（整数）
- `overageCapWithPrecision`: 超额上限（精确值）
- `overageRate`: 超额费率（每单位价格）
- `overageCharges`: 超额费用

**免费试用信息** (`freeTrialInfo`):
- `freeTrialStatus`: 试用状态（`"ACTIVE"` 表示激活中）
- `currentUsage`: 试用期当前使用量（整数，已废弃）
- `currentUsageWithPrecision`: 试用期当前使用量（精确值，**推荐使用**）
- `usageLimit`: 试用期限制（整数，已废弃）
- `usageLimitWithPrecision`: 试用期限制（精确值，**推荐使用**）
- `freeTrialExpiry`: 试用期到期时间戳（Unix 时间）

**其他**:
- `bonuses`: 奖励列表
- `nextDateReset`: 下次重置时间戳

#### userInfo - 用户信息

- `email`: 用户邮箱
- `userId`: 用户 ID

## 配额计算逻辑

### 优先级规则

1. **优先使用 `freeTrialInfo`**（如果存在且 `freeTrialStatus` 为 `"ACTIVE"`）
   - 使用量: `freeTrialInfo.currentUsageWithPrecision`
   - 限制: `freeTrialInfo.usageLimitWithPrecision`

2. **否则使用顶层字段**
   - 使用量: `currentUsageWithPrecision`
   - 限制: `usageLimitWithPrecision`

### 示例代码

```typescript
function getQuota(response: any): { used: number; limit: number } {
  const breakdown = response.usageBreakdownList?.[0]
  if (!breakdown) return { used: 0, limit: 0 }

  // 优先使用免费试用配额
  if (breakdown.freeTrialInfo?.freeTrialStatus === 'ACTIVE') {
    return {
      used: breakdown.freeTrialInfo.currentUsageWithPrecision ?? 0,
      limit: breakdown.freeTrialInfo.usageLimitWithPrecision ?? 0
    }
  }

  // 否则使用常规配额
  return {
    used: breakdown.currentUsageWithPrecision ?? 0,
    limit: breakdown.usageLimitWithPrecision ?? 0
  }
}
```

## 错误响应

### 401 Unauthorized - Token 过期

```json
{
  "__type": "ExpiredTokenException",
  "message": "Token has expired"
}
```

**处理**: 刷新 Token 后重试

### 403 Forbidden - Token 无效

```json
{
  "__type": "AccessDeniedException",
  "message": "Invalid token"
}
```

**处理**: 提示用户重新登录

### 423 Locked - 账号被封禁

```json
{
  "message": "Account has been suspended"
}
```

**处理**: 提示用户账号已被封禁，无法使用

## KiroGate 实现

### 后端接口

```
GET /admin/quota/:account_id
```

**功能**:
1. 自动刷新 Token（如果过期）
2. 调用 Kiro API 获取配额
3. 返回原始响应（不做转换）

**响应**: 直接返回 Kiro API 的原始 JSON

### 前端使用

```typescript
import { api } from './api/client'

// 获取配额
const quota = await api.getQuota(accountId)

// 提取使用情况
const breakdown = quota.usageBreakdownList?.[0]
const used = breakdown?.freeTrialInfo?.currentUsageWithPrecision ?? 
             breakdown?.currentUsageWithPrecision ?? 0
const limit = breakdown?.freeTrialInfo?.usageLimitWithPrecision ?? 
              breakdown?.usageLimitWithPrecision ?? 0
```

## 参考资料

- **参考项目**: [AIClient-2-API](https://github.com/justlovemaki/AIClient-2-API)
  - 文件: `src/providers/claude/claude-kiro.js` (line 2950+)
- **Kiro 源码**: `C:\Users\{用户名}\AppData\Local\Programs\Kiro\resources\app\extensions\kiro.kiro-agent\dist\extension.js`

## 注意事项

1. **使用精确值字段**: 优先使用 `*WithPrecision` 字段，整数字段已废弃
2. **免费试用优先**: 如果有 `freeTrialInfo` 且状态为 `ACTIVE`，优先使用其配额数据
3. **Token 刷新**: 配额查询前建议先刷新 Token，避免 401 错误
4. **账号类型差异**: Social 账号需要 `profileArn` 参数，IDC 账号不需要
5. **时间戳格式**: 所有时间戳为 Unix 时间（秒）
