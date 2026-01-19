# Enterprise OAuth 登录流程详解

## 概述

AWS IAM Identity Center (Enterprise) 使用标准的 **OAuth 2.0 Authorization Code Grant with PKCE** 流程。

---

## 完整流程图

```
┌─────────────┐                                    ┌──────────────┐
│             │                                    │              │
│  KiroGate   │                                    │  AWS OIDC    │
│  (Client)   │                                    │   Server     │
│             │                                    │              │
└──────┬──────┘                                    └──────┬───────┘
       │                                                  │
       │ 1. Register Client                              │
       │ POST /client/register                           │
       ├────────────────────────────────────────────────>│
       │ {                                               │
       │   clientName: "KiroGate",                       │
       │   clientType: "public",                         │
       │   grantTypes: ["authorization_code"],           │
       │   redirectUris: ["http://127.0.0.1:PORT/..."]   │
       │ }                                               │
       │                                                  │
       │<────────────────────────────────────────────────┤
       │ { clientId, clientSecret }                      │
       │                                                  │
       │ 2. Generate PKCE                                │
       │ - codeVerifier = random(32 bytes)               │
       │ - codeChallenge = SHA256(codeVerifier)          │
       │ - state = UUID                                  │
       │                                                  │
       │ 3. Start Local HTTP Server                      │
       │ - Listen on 127.0.0.1:PORT                      │
       │ - Wait for callback                             │
       │                                                  │
       │ 4. Open Browser                                 │
       │ GET /authorize?                                 │
       │   response_type=code&                           │
       │   client_id=xxx&                                │
       │   redirect_uri=http://127.0.0.1:PORT/callback&  │
       │   code_challenge=xxx&                           │
       │   code_challenge_method=S256&                   │
       │   state=xxx                                     │
       ├────────────────────────────────────────────────>│
       │                                                  │
       │                  User Login in Browser          │
       │                  ┌──────────────┐               │
       │                  │              │               │
       │                  │   Browser    │               │
       │                  │              │               │
       │                  └──────┬───────┘               │
       │                         │                       │
       │                         │ User Authenticates    │
       │                         │                       │
       │                         ↓                       │
       │                  ┌──────────────┐               │
       │                  │  AWS Login   │               │
       │                  │    Page      │               │
       │                  └──────┬───────┘               │
       │                         │                       │
       │                         │ Authorize             │
       │                         │                       │
       │<────────────────────────┴───────────────────────┤
       │ 5. Callback                                     │
       │ GET /callback?code=xxx&state=xxx                │
       │                                                  │
       │ 6. Verify State                                 │
       │ if (state !== savedState) throw Error           │
       │                                                  │
       │ 7. Exchange Code for Token                      │
       │ POST /token                                     │
       ├────────────────────────────────────────────────>│
       │ {                                               │
       │   clientId,                                     │
       │   clientSecret,                                 │
       │   grantType: "authorization_code",              │
       │   code,                                         │
       │   redirectUri,                                  │
       │   codeVerifier                                  │
       │ }                                               │
       │                                                  │
       │<────────────────────────────────────────────────┤
       │ {                                               │
       │   accessToken,                                  │
       │   refreshToken,                                 │
       │   expiresIn                                     │
       │ }                                               │
       │                                                  │
       │ 8. Close Local Server                           │
       │ 9. Save Tokens                                  │
       │                                                  │
```

---

## 实现步骤详解

### Step 1: 注册 OIDC 客户端

**端点**: `https://oidc.{region}.amazonaws.com/client/register`

**请求**:
```json
{
  "clientName": "KiroGate",
  "clientType": "public",
  "scopes": [
    "codewhisperer:completions",
    "codewhisperer:analysis",
    "codewhisperer:conversations"
  ],
  "grantTypes": ["authorization_code", "refresh_token"],
  "redirectUris": ["http://127.0.0.1:PORT/oauth/callback"],
  "issuerUrl": "https://view.awsapps.com/start"  // 用户的 SSO Start URL
}
```

**响应**:
```json
{
  "clientId": "MkAG97...",
  "clientSecret": "eyJraWQ...",
  "clientIdIssuedAt": 1234567890,
  "clientSecretExpiresAt": 1234567890
}
```

**复杂点**:
- 需要用户提供 SSO Start URL
- clientSecret 有过期时间（通常 90 天）
- 需要处理注册失败（权限不足、URL 错误等）

---

### Step 2: 生成 PKCE (Proof Key for Code Exchange)

**目的**: 防止授权码拦截攻击

**实现**:
```rust
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

// 1. 生成 code_verifier (43-128 字符)
let code_verifier: String = {
    let random_bytes: [u8; 32] = rand::random();
    URL_SAFE_NO_PAD.encode(random_bytes)
};

// 2. 计算 code_challenge
let code_challenge: String = {
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
};

// 3. 生成 state (防止 CSRF 攻击)
let state = uuid::Uuid::new_v4().to_string();
```

**复杂点**:
- 需要加密库（sha2）
- 需要 Base64 URL-safe 编码
- 需要保存 code_verifier 用于后续验证

---

### Step 3: 启动本地 HTTP 服务器

**目的**: 接收 OAuth 回调

**实现**:
```rust
use axum::{Router, routing::get, extract::Query};
use tokio::net::TcpListener;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct OAuthState {
    result: Arc<Mutex<Option<OAuthResult>>>,
}

async fn oauth_callback(
    Query(params): Query<HashMap<String, String>>,
    state: axum::extract::State<OAuthState>,
) -> impl IntoResponse {
    let code = params.get("code");
    let returned_state = params.get("state");
    let error = params.get("error");
    
    if let Some(err) = error {
        // 处理错误
        let mut result = state.result.lock().unwrap();
        *result = Some(OAuthResult::Error(err.clone()));
        return Html("<h1>授权失败</h1>");
    }
    
    // 验证 state
    // 保存 code
    // 返回成功页面
    Html("<h1>授权成功！正在获取令牌...</h1>")
}

async fn start_oauth_server() -> Result<u16> {
    let state = OAuthState {
        result: Arc::new(Mutex::new(None)),
    };
    
    let app = Router::new()
        .route("/oauth/callback", get(oauth_callback))
        .with_state(state);
    
    // 绑定到随机端口
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    
    Ok(port)
}
```

**复杂点**:
- 需要找一个可用端口（避免冲突）
- 需要管理服务器生命周期（启动/停止）
- 需要处理并发（多个回调）
- 需要跨线程共享状态（Arc + Mutex）
- 需要超时处理（10 分钟无响应）

---

### Step 4: 构建授权 URL 并打开浏览器

**URL 格式**:
```
https://oidc.{region}.amazonaws.com/authorize?
  response_type=code&
  client_id={clientId}&
  redirect_uri=http://127.0.0.1:{port}/oauth/callback&
  scopes=codewhisperer:completions,codewhisperer:analysis&
  state={state}&
  code_challenge={codeChallenge}&
  code_challenge_method=S256
```

**实现**:
```rust
use url::Url;

let mut authorize_url = Url::parse(&format!(
    "https://oidc.{}.amazonaws.com/authorize",
    region
))?;

authorize_url.query_pairs_mut()
    .append_pair("response_type", "code")
    .append_pair("client_id", &client_id)
    .append_pair("redirect_uri", &redirect_uri)
    .append_pair("scopes", &scopes.join(","))
    .append_pair("state", &state)
    .append_pair("code_challenge", &code_challenge)
    .append_pair("code_challenge_method", "S256");

// 打开浏览器
open::that(authorize_url.as_str())?;
```

**复杂点**:
- URL 编码
- 跨平台打开浏览器（Windows/macOS/Linux）
- 用户可能拒绝授权
- 浏览器可能被防火墙阻止

---

### Step 5: 接收回调并验证

**回调 URL**:
```
http://127.0.0.1:PORT/oauth/callback?code=xxx&state=xxx
```

**验证逻辑**:
```rust
async fn handle_callback(
    code: String,
    returned_state: String,
    saved_state: String,
) -> Result<()> {
    // 1. 验证 state（防止 CSRF）
    if returned_state != saved_state {
        return Err("State 不匹配".into());
    }
    
    // 2. 验证 code 不为空
    if code.is_empty() {
        return Err("未收到授权码".into());
    }
    
    // 3. 验证超时（10 分钟内）
    if Instant::now() > start_time + Duration::from_secs(600) {
        return Err("授权超时".into());
    }
    
    Ok(())
}
```

**复杂点**:
- 需要保存 state 用于验证
- 需要处理多种错误情况
- 需要超时检测

---

### Step 6: 用授权码交换 Token

**端点**: `https://oidc.{region}.amazonaws.com/token`

**请求**:
```json
{
  "clientId": "MkAG97...",
  "clientSecret": "eyJraWQ...",
  "grantType": "authorization_code",
  "code": "xxx",
  "redirectUri": "http://127.0.0.1:PORT/oauth/callback",
  "codeVerifier": "xxx"  // 之前生成的
}
```

**响应**:
```json
{
  "accessToken": "eyJraWQ...",
  "refreshToken": "eyJraWQ...",
  "expiresIn": 3600,
  "tokenType": "Bearer"
}
```

**复杂点**:
- 需要保存 code_verifier 用于验证
- 需要处理交换失败（code 过期、verifier 错误等）
- 需要保存 clientId 和 clientSecret 用于后续刷新

---

### Step 7: 保存凭证

**需要保存的信息**:
```rust
struct EnterpriseCredentials {
    access_token: String,
    refresh_token: String,
    client_id: String,
    client_secret: String,
    start_url: String,  // 用于计算 clientIdHash
    region: String,
    expires_at: i64,
}
```

**clientIdHash 计算**:
```rust
use sha1::{Sha1, Digest};

fn calculate_client_id_hash(start_url: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(format!(r#"{{"startUrl":"{}"}}"#, start_url).as_bytes());
    hex::encode(hasher.finalize())
}
```

**复杂点**:
- 需要计算 clientIdHash（用于后续切号）
- 需要保存到文件（加密存储）
- 需要同时保存到 `~/.aws/sso/cache/` 用于兼容 Kiro IDE

---

## 错误处理

### 常见错误

1. **UnauthorizedException** - 组织未配置权限
   ```
   您的组织可能未配置 Amazon Q Developer 访问权限。
   请联系组织管理员在 IAM Identity Center 中启用相关权限。
   ```

2. **access_denied** - 用户拒绝授权
   ```
   用户拒绝授权
   ```

3. **expired_token** - 授权码过期
   ```
   授权码已过期，请重新开始
   ```

4. **invalid_grant** - code_verifier 错误
   ```
   PKCE 验证失败
   ```

5. **端口被占用**
   ```
   无法启动本地服务器，端口被占用
   ```

---

## 完整代码示例（简化版）

```rust
use axum::{Router, routing::get, extract::Query};
use tokio::net::TcpListener;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

pub struct EnterpriseOAuth {
    client_id: String,
    client_secret: String,
    code_verifier: String,
    state: String,
    port: u16,
    result: Arc<Mutex<Option<OAuthResult>>>,
}

enum OAuthResult {
    Success {
        access_token: String,
        refresh_token: String,
        expires_in: u64,
    },
    Error(String),
}

impl EnterpriseOAuth {
    pub async fn new(start_url: &str, region: &str) -> Result<Self> {
        // Step 1: 注册客户端
        let (client_id, client_secret) = Self::register_client(start_url, region).await?;
        
        // Step 2: 生成 PKCE
        let code_verifier = Self::generate_code_verifier();
        let code_challenge = Self::generate_code_challenge(&code_verifier);
        let state = uuid::Uuid::new_v4().to_string();
        
        // Step 3: 启动本地服务器
        let port = Self::start_local_server().await?;
        
        Ok(Self {
            client_id,
            client_secret,
            code_verifier,
            state,
            port,
            result: Arc::new(Mutex::new(None)),
        })
    }
    
    async fn register_client(start_url: &str, region: &str) -> Result<(String, String)> {
        let url = format!("https://oidc.{}.amazonaws.com/client/register", region);
        let response = reqwest::Client::new()
            .post(&url)
            .json(&serde_json::json!({
                "clientName": "KiroGate",
                "clientType": "public",
                "scopes": ["codewhisperer:completions"],
                "grantTypes": ["authorization_code", "refresh_token"],
                "redirectUris": [format!("http://127.0.0.1:{}/oauth/callback", 0)],
                "issuerUrl": start_url
            }))
            .send()
            .await?;
        
        let data: serde_json::Value = response.json().await?;
        Ok((
            data["clientId"].as_str().unwrap().to_string(),
            data["clientSecret"].as_str().unwrap().to_string(),
        ))
    }
    
    fn generate_code_verifier() -> String {
        let random_bytes: [u8; 32] = rand::random();
        URL_SAFE_NO_PAD.encode(random_bytes)
    }
    
    fn generate_code_challenge(verifier: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(verifier.as_bytes());
        URL_SAFE_NO_PAD.encode(hasher.finalize())
    }
    
    async fn start_local_server() -> Result<u16> {
        // 实现本地服务器...
        Ok(8080)
    }
    
    pub fn get_authorize_url(&self, region: &str) -> String {
        format!(
            "https://oidc.{}.amazonaws.com/authorize?\
             response_type=code&\
             client_id={}&\
             redirect_uri=http://127.0.0.1:{}/oauth/callback&\
             state={}&\
             code_challenge={}&\
             code_challenge_method=S256",
            region,
            self.client_id,
            self.port,
            self.state,
            Self::generate_code_challenge(&self.code_verifier)
        )
    }
    
    pub async fn exchange_code(&self, code: &str, region: &str) -> Result<OAuthResult> {
        let url = format!("https://oidc.{}.amazonaws.com/token", region);
        let response = reqwest::Client::new()
            .post(&url)
            .json(&serde_json::json!({
                "clientId": self.client_id,
                "clientSecret": self.client_secret,
                "grantType": "authorization_code",
                "code": code,
                "redirectUri": format!("http://127.0.0.1:{}/oauth/callback", self.port),
                "codeVerifier": self.code_verifier
            }))
            .send()
            .await?;
        
        let data: serde_json::Value = response.json().await?;
        Ok(OAuthResult::Success {
            access_token: data["accessToken"].as_str().unwrap().to_string(),
            refresh_token: data["refreshToken"].as_str().unwrap().to_string(),
            expires_in: data["expiresIn"].as_u64().unwrap(),
        })
    }
}
```

---

## 为什么复杂？

### 技术复杂度

1. **多个异步操作** - 注册、服务器、Token 交换
2. **状态管理** - code_verifier、state、result 需要跨线程共享
3. **并发处理** - 服务器需要处理多个请求
4. **超时处理** - 10 分钟无响应需要清理
5. **错误处理** - 至少 10 种错误情况

### 用户体验复杂度

1. **需要用户输入 SSO Start URL** - 大部分用户不知道这是什么
2. **需要打开浏览器** - 可能被防火墙阻止
3. **需要等待回调** - 用户可能关闭浏览器
4. **可能失败** - 权限不足、URL 错误等

### 维护复杂度

1. **依赖多个库** - axum、tokio、sha2、base64、uuid
2. **跨平台兼容** - Windows/macOS/Linux 打开浏览器方式不同
3. **安全性** - PKCE、state 验证、HTTPS
4. **调试困难** - 涉及浏览器、服务器、AWS 三方

---

## 对比：从 Kiro IDE 导入

```rust
// 只需要 10 行代码
pub fn import_from_kiro_ide() -> Result<Account> {
    let token_file = dirs::home_dir()
        .unwrap()
        .join(".aws/sso/cache/kiro-auth-token.json");
    
    let token_data = fs::read_to_string(&token_file)?;
    let token: KiroToken = serde_json::from_str(&token_data)?;
    
    Ok(Account::from(token))
}
```

**优势**:
- ✅ 代码简单（10 行 vs 300+ 行）
- ✅ 无需用户输入
- ✅ 无需启动服务器
- ✅ 无需打开浏览器
- ✅ 无需处理回调
- ✅ 无需 PKCE
- ✅ 100% 成功率

---

## 结论

Enterprise OAuth 登录流程复杂，主要体现在：

1. **技术实现复杂** - 需要实现完整的 OAuth 2.0 + PKCE 流程
2. **状态管理复杂** - 需要管理多个异步状态
3. **错误处理复杂** - 至少 10 种错误情况
4. **用户体验复杂** - 需要用户多步操作
5. **维护成本高** - 依赖多个库，跨平台兼容

**推荐方案**: 继续使用"从 Kiro IDE 导入"，简单可靠。
