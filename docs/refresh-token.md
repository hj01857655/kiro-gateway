# Token 刷新接口文档

## 1. Social 登录刷新（Google/GitHub）

### 基本信息
- **URL**: `https://prod.us-east-1.auth.desktop.kiro.dev/refreshToken`
- **Method**: `POST`
- **Content-Type**: `application/json`

### 完整请求
```http
POST /refreshToken HTTP/1.1
Host: prod.us-east-1.auth.desktop.kiro.dev
Content-Type: application/json
User-Agent: KiroIDE-0.6.18-{machineId}

{
  "refreshToken": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### 请求字段
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| refreshToken | string | 是 | 刷新令牌 |

### 成功响应 (200)
```json
{
  "accessToken": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refreshToken": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "profileArn": "arn:aws:codewhisperer:us-east-1:123456789:profile/xxx",
  "expiresIn": 3600,
  "csrfToken": "abc123def456"
}
```

### 响应字段
| 字段 | 类型 | 说明 |
|------|------|------|
| accessToken | string | 新的访问令牌 |
| refreshToken | string | 新的刷新令牌 |
| profileArn | string | AWS 配置文件 ARN |
| expiresIn | number | 过期时间（秒） |
| csrfToken | string | CSRF 令牌 |

### 错误响应

#### 401 Unauthorized
```json
{
  "error": "invalid_grant",
  "error_description": "RefreshToken 已过期或无效"
}
```

---

## 2. BuilderId/Enterprise 刷新（AWS SSO OIDC）

### 基本信息
- **URL**: `https://oidc.{region}.amazonaws.com/token`
- **Method**: `POST`
- **Content-Type**: `application/json`
- **默认 Region**: `us-east-1`

### 完整请求
```http
POST /token HTTP/1.1
Host: oidc.us-east-1.amazonaws.com
Content-Type: application/json

{
  "clientId": "abc123def456ghi789",
  "clientSecret": "secret-xxx-yyy-zzz",
  "grantType": "refresh_token",
  "refreshToken": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

### 请求字段
| 字段 | 类型 | 必填 | 说明 |
|------|------|------|------|
| clientId | string | 是 | 客户端 ID（首次登录时获取） |
| clientSecret | string | 是 | 客户端密钥（首次登录时获取） |
| grantType | string | 是 | 固定值 `refresh_token` |
| refreshToken | string | 是 | 刷新令牌 |

### 成功响应 (200)
```json
{
  "accessToken": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refreshToken": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "idToken": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "tokenType": "Bearer",
  "expiresIn": 3600,
  "aws_sso_app_session_id": "session-xxx-yyy-zzz",
  "issuedTokenType": "urn:ietf:params:oauth:token-type:access_token",
  "originSessionId": "origin-session-id"
}
```

### 响应字段
| 字段 | 类型 | 说明 |
|------|------|------|
| accessToken | string | 新的访问令牌 |
| refreshToken | string | 新的刷新令牌 |
| idToken | string | ID 令牌（可选） |
| tokenType | string | 令牌类型，固定 `Bearer` |
| expiresIn | number | 过期时间（秒） |
| aws_sso_app_session_id | string | SSO 会话 ID（可选） |
| issuedTokenType | string | 颁发的令牌类型（可选） |
| originSessionId | string | 原始会话 ID（可选） |

### 错误响应

#### 401 Unauthorized
```json
{
  "error": "invalid_grant",
  "error_description": "RefreshToken 已过期或无效"
}
```

---

## 调用说明

### Social 登录（Google/GitHub）
1. 只需要 `refreshToken`
2. `machineId` 放在 `User-Agent` 头中
3. 格式：`KiroIDE-0.6.18-{machineId}`

### BuilderId/Enterprise
1. 需要 `clientId`、`clientSecret`、`refreshToken` 三个参数
2. `clientId` 和 `clientSecret` 在首次登录时通过 `/client/register` 获取
3. 必须保存这两个值用于后续刷新
4. 不同 region 的账号需要使用对应 region 的 OIDC 端点
