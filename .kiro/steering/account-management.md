# 账号管理规范

## 账号类型

**Social** - 个人账号（Google/GitHub）
- profileArn 为空
- 刷新端点：`https://prod.us-east-1.auth.desktop.kiro.dev/refreshToken`
- Body: `{ "refreshToken": "..." }`

**IDC** - 企业账号（AWS IAM Identity Center）
- profileArn 有值
- 刷新端点：`https://oidc.{region}.amazonaws.com/token`
- Body: `{ "clientId", "clientSecret", "grantType": "refresh_token", "refreshToken" }`

## Token 管理

- accessToken 有效期 1 小时
- 提前 5 分钟刷新
- refreshToken < 100 字符 → 可能被截断，警告用户

## 凭证来源

- Kiro IDE 缓存：`~/.aws/sso/cache/kiro-auth-token.json`
- IDC 客户端注册：`~/.aws/sso/cache/{clientIdHash}.json`

## 多账号策略

- 轮询选择可用账号
- Token 过期自动刷新
- 刷新失败标记为 expired
- 限流时跳过该账号 60 秒

## 账号状态

- `active` - 正常可用
- `expired` - Token 过期，需重新登录
- `error` - 其他错误
- `disabled` - 手动禁用
