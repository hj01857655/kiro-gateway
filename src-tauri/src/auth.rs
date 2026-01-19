// Token 管理模块
// 用于 WebSearch 的 Token 缓存和刷新

use chrono::Utc;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct TokenConfig {
    pub refresh_token: String,
    pub auth_method: String,
    pub profile_arn: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub region: Option<String>,
}

#[derive(Debug, Clone)]
struct TokenData {
    access_token: String,
    expires_at: i64,
}

pub struct TokenManager {
    config: TokenConfig,
    token: Arc<RwLock<Option<TokenData>>>,
}

impl TokenManager {
    fn new(config: TokenConfig) -> Self {
        Self {
            config,
            token: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn get_access_token(&self) -> Result<String, String> {
        // 检查缓存
        {
            let token = self.token.read();
            if let Some(ref data) = *token {
                let now = Utc::now().timestamp_millis();
                let buffer_ms = 5 * 60 * 1000; // 提前 5 分钟刷新
                if now < data.expires_at - buffer_ms {
                    return Ok(data.access_token.clone());
                }
            }
        }

        // 刷新 Token
        self.refresh_token().await
    }

    async fn refresh_token(&self) -> Result<String, String> {
        let is_idc = self.config.auth_method.to_lowercase() == "idc";

        let (url, body) = if is_idc {
            // IDC 刷新
            let region = self.config.region.as_deref().unwrap_or("us-east-1");
            let url = format!("https://oidc.{}.amazonaws.com/token", region);

            let client_id = self
                .config
                .client_id
                .as_ref()
                .ok_or("IDC 账号缺少 clientId")?;
            let client_secret = self
                .config
                .client_secret
                .as_ref()
                .ok_or("IDC 账号缺少 clientSecret")?;

            let body = serde_json::json!({
                "clientId": client_id,
                "clientSecret": client_secret,
                "grantType": "refresh_token",
                "refreshToken": self.config.refresh_token
            });

            (url, body)
        } else {
            // Social 刷新
            let url = "https://prod.us-east-1.auth.desktop.kiro.dev/refreshToken".to_string();
            let body = serde_json::json!({
                "refreshToken": self.config.refresh_token
            });

            (url, body)
        };

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("刷新 Token 请求失败: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("刷新 Token 失败: HTTP {} - {}", status, text));
        }

        let result: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("解析刷新响应失败: {}", e))?;

        let access_token = result
            .get("accessToken")
            .and_then(|v| v.as_str())
            .ok_or("响应中缺少 accessToken")?
            .to_string();

        let expires_in = result
            .get("expiresIn")
            .and_then(|v| v.as_i64())
            .unwrap_or(3600);

        let expires_at = Utc::now().timestamp_millis() + expires_in * 1000;

        // 更新缓存
        {
            let mut token = self.token.write();
            *token = Some(TokenData {
                access_token: access_token.clone(),
                expires_at,
            });
        }

        Ok(access_token)
    }
}

pub struct AuthCache {
    cache: Arc<RwLock<HashMap<String, Arc<TokenManager>>>>,
}

impl Default for AuthCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_or_create(
        &self,
        refresh_token: &str,
        config: TokenConfig,
    ) -> Arc<TokenManager> {
        // 先尝试读取
        {
            let cache = self.cache.read();
            if let Some(manager) = cache.get(refresh_token) {
                return manager.clone();
            }
        }

        // 创建新的 TokenManager
        let manager = Arc::new(TokenManager::new(config));

        // 写入缓存
        {
            let mut cache = self.cache.write();
            cache.insert(refresh_token.to_string(), manager.clone());
        }

        manager
    }
}
