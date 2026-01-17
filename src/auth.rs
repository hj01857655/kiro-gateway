// KiroGate 认证管理
// 管理 access_token 生命周期，支持 Social 和 IdC 两种认证方式

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;


const TOKEN_REFRESH_THRESHOLD_SECS: u64 = 300; // 5 分钟前刷新

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResponse {
  pub access_token: String,
  #[serde(default)]
  pub refresh_token: Option<String>,
  pub expires_in: i64,
  #[serde(default)]
  pub profile_arn: Option<String>,
}

/// Token 配置
#[derive(Debug, Clone)]
pub struct TokenConfig {
  pub refresh_token: String,
  pub auth_method: String, // "social" 或 "IdC"
  pub profile_arn: Option<String>,
  pub client_id: Option<String>,
  pub client_secret: Option<String>,
  pub region: Option<String>,
}

/// Token 管理器
pub struct TokenManager {
  config: TokenConfig,
  access_token: RwLock<Option<String>>,
  cached_profile_arn: RwLock<Option<String>>,
  expires_at: RwLock<Option<Instant>>,
  client: Client,
}

impl TokenManager {
  pub fn new(config: TokenConfig) -> Self {
    let client = Client::builder()
      .timeout(Duration::from_secs(30))
      .build()
      .expect("failed to build reqwest client");

    let initial_profile_arn = config.profile_arn.clone();

    Self {
      config,
      access_token: RwLock::new(None),
      cached_profile_arn: RwLock::new(initial_profile_arn),
      expires_at: RwLock::new(None),
      client,
    }
  }

  /// 获取有效的 access_token，必要时刷新
  pub async fn get_access_token(&self) -> Result<String, String> {
    let needs_refresh = {
      let expires_at = self.expires_at.read().await;
      let access_token = self.access_token.read().await;
      
      if access_token.is_none() {
        true
      } else if let Some(exp) = *expires_at {
        exp.saturating_duration_since(Instant::now()) < Duration::from_secs(TOKEN_REFRESH_THRESHOLD_SECS)
      } else {
        true
      }
    };

    if needs_refresh {
      self.refresh().await?;
    }

    let token = self.access_token.read().await;
    token.clone().ok_or_else(|| "无法获取 access_token".to_string())
  }

  /// 获取 profile_arn
  pub async fn get_profile_arn(&self) -> Option<String> {
    self.cached_profile_arn.read().await.clone()
  }

  /// 刷新 token
  async fn refresh(&self) -> Result<(), String> {
    let region = self.config.region.as_deref().unwrap_or("us-east-1");

    if self.config.auth_method == "IdC" {
      // IdC 类型：使用 AWS SSO OIDC 刷新
      let client_id = self.config.client_id.as_ref()
        .ok_or("IdC Token 缺少 clientId")?;
      let client_secret = self.config.client_secret.as_ref()
        .ok_or("IdC Token 缺少 clientSecret")?;

      let sso_client = crate::aws_sso_client::AWSSSOClient::new(region);
      let resp = sso_client.refresh_token(client_id, client_secret, &self.config.refresh_token).await?;

      let expires_at = Instant::now() + Duration::from_secs((resp.expires_in - 60) as u64);
      *self.access_token.write().await = Some(resp.access_token);
      *self.expires_at.write().await = Some(expires_at);
    } else {
      // Social 类型：使用 Kiro Desktop Auth 刷新
      let refresh_url = format!("https://prod.{}.auth.desktop.kiro.dev/refreshToken", region);

      #[cfg(debug_assertions)]
      log::debug!("[KiroGate Auth] Refreshing Social token, URL: {}", refresh_url);

      let body = serde_json::json!({
        "refreshToken": self.config.refresh_token
      });

      let resp = self.client
        .post(&refresh_url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
          #[cfg(debug_assertions)]
          log::debug!("[KiroGate Auth] Request error: {:?}", e);
          format!("刷新 token 请求失败: {}", e)
        })?;

      let status = resp.status();
      if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        #[cfg(debug_assertions)]
        println!("[KiroGate Auth] Response error: {} - {}", status, text);
        if status.as_u16() == 401 {
          return Err("RefreshToken 已过期或无效".to_string());
        }
        return Err(format!("刷新 token 失败 ({}): {}", status, text));
      }

      let data: RefreshResponse = resp.json().await
        .map_err(|e| format!("解析刷新响应失败: {}", e))?;

      #[cfg(debug_assertions)]
      log::debug!("[KiroGate Auth] Token refreshed, expires_in: {}", data.expires_in);

      let expires_at = Instant::now() + Duration::from_secs((data.expires_in - 60) as u64);
      
      *self.access_token.write().await = Some(data.access_token);
      if let Some(arn) = data.profile_arn {
        *self.cached_profile_arn.write().await = Some(arn);
      }
      *self.expires_at.write().await = Some(expires_at);
    }

    Ok(())
  }
}

/// 认证缓存 - 缓存多个 token_id 对应的 TokenManager
pub struct AuthCache {
  cache: RwLock<std::collections::HashMap<String, Arc<TokenManager>>>,
}

impl AuthCache {
  pub fn new() -> Self {
    Self {
      cache: RwLock::new(std::collections::HashMap::new()),
    }
  }

  /// 获取或创建 TokenManager
  pub async fn get_or_create(&self, token_id: &str, config: TokenConfig) -> Arc<TokenManager> {
    {
      let cache = self.cache.read().await;
      if let Some(manager) = cache.get(token_id) {
        return manager.clone();
      }
    }

    let manager = Arc::new(TokenManager::new(config));
    
    {
      let mut cache = self.cache.write().await;
      cache.insert(token_id.to_string(), manager.clone());
    }

    manager
  }

  /// 清除缓存
  #[allow(dead_code)]
  pub async fn clear(&self) {
    let mut cache = self.cache.write().await;
    cache.clear();
  }
}
