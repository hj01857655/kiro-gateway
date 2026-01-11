use chrono::Utc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::error::AppError;

// 账号状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AccountStatus {
    Active,
    Expired,
    Throttled,
    Error,
    Disabled,
}

impl Default for AccountStatus {
    fn default() -> Self { AccountStatus::Active }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub name: Option<String>,
    pub auth_method: String,
    pub access_token: String,
    pub refresh_token: String,
    #[serde(default)]
    pub profile_arn: String,
    pub region: Option<String>,
    #[serde(default)]
    pub expires_at: Option<i64>,
    #[serde(default)]
    pub expire: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub status: AccountStatus,
    #[serde(default)]
    pub throttled_until: Option<i64>,
}

fn default_enabled() -> bool { true }

impl Account {
    pub fn is_expired(&self) -> bool {
        let buffer_ms = 5 * 60 * 1000;
        let now = Utc::now().timestamp_millis();

        if let Some(ref expire_str) = self.expire {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(expire_str) {
                return now >= dt.timestamp_millis() - buffer_ms;
            }
        }

        if let Some(expires_at) = self.expires_at {
            let expires_ms = if expires_at < 10_000_000_000 {
                expires_at * 1000
            } else {
                expires_at
            };
            return now >= expires_ms - buffer_ms;
        }

        true
    }

    pub fn is_idc(&self) -> bool {
        self.auth_method.to_lowercase() == "idc" || !self.profile_arn.is_empty()
    }

    pub fn is_throttled(&self) -> bool {
        if let Some(until) = self.throttled_until {
            Utc::now().timestamp_millis() < until
        } else {
            false
        }
    }

    pub fn is_available(&self) -> bool {
        self.enabled 
            && self.status == AccountStatus::Active 
            && !self.is_throttled()
    }
}

pub struct AccountManager {
    accounts: RwLock<Vec<Account>>,
    current_index: RwLock<usize>,
    http_client: reqwest::Client,
}

impl AccountManager {
    pub fn new() -> Self {
        Self {
            accounts: RwLock::new(Vec::new()),
            current_index: RwLock::new(0),
            http_client: reqwest::Client::new(),
        }
    }

    pub fn load_from_json(&self, json_str: &str) -> Result<(), AppError> {
        #[derive(Deserialize)]
        struct AccountsConfig {
            accounts: Vec<Account>,
        }

        let config: AccountsConfig = serde_json::from_str(json_str)
            .map_err(|e| AppError::ParseError(e.to_string()))?;

        // 检查 refreshToken 长度
        for acc in &config.accounts {
            if acc.refresh_token.len() < 100 {
                warn!("账号 {} 的 refreshToken 长度 < 100，可能无效", acc.id);
            }
        }

        let mut accounts = self.accounts.write();
        *accounts = config.accounts;
        info!("加载了 {} 个账号", accounts.len());
        Ok(())
    }

    /// 列出所有账号（预留给管理 API）
    #[allow(dead_code)]
    pub fn list_accounts(&self) -> Vec<Account> {
        self.accounts.read().clone()
    }

    pub async fn get_account(&self) -> Result<Account, AppError> {
        let (available, selected_idx) = {
            let accounts = self.accounts.read();
            if accounts.is_empty() {
                return Err(AppError::NoToken);
            }

            let available: Vec<_> = accounts.iter()
                .filter(|a| a.is_available())
                .cloned()
                .collect();

            if available.is_empty() {
                return Err(AppError::NoToken);
            }

            let mut index = self.current_index.write();
            *index = (*index + 1) % available.len();
            let idx = *index;
            (available, idx)
        };

        let mut account = available[selected_idx].clone();

        if account.is_expired() {
            info!("账号 {} Token 即将过期，刷新中...", account.id);
            account = self.refresh_token(account).await?;
        }

        Ok(account)
    }

    pub async fn refresh_account(&self, account_id: &str) -> Result<Account, AppError> {
        let account = {
            let accounts = self.accounts.read();
            accounts.iter().find(|a| a.id == account_id).cloned()
        };

        match account {
            Some(acc) => self.refresh_token(acc).await,
            None => Err(AppError::BadRequest(format!("账号 {} 不存在", account_id))),
        }
    }

    /// 标记账号限流（60 秒后恢复）
    pub fn mark_throttled(&self, account_id: &str) {
        let mut accounts = self.accounts.write();
        if let Some(acc) = accounts.iter_mut().find(|a| a.id == account_id) {
            acc.status = AccountStatus::Throttled;
            acc.throttled_until = Some(Utc::now().timestamp_millis() + 60_000);
            warn!("账号 {} 被限流，60 秒后恢复", account_id);
        }
    }

    /// 标记账号状态
    pub fn mark_status(&self, account_id: &str, status: AccountStatus) {
        let mut accounts = self.accounts.write();
        if let Some(acc) = accounts.iter_mut().find(|a| a.id == account_id) {
            acc.status = status;
        }
    }

    async fn refresh_token(&self, mut account: Account) -> Result<Account, AppError> {
        let result = if account.is_idc() {
            self.refresh_idc_token(&mut account).await
        } else {
            self.refresh_social_token(&mut account).await
        };

        match result {
            Ok(()) => {
                let mut accounts = self.accounts.write();
                if let Some(stored) = accounts.iter_mut().find(|a| a.id == account.id) {
                    stored.access_token = account.access_token.clone();
                    stored.refresh_token = account.refresh_token.clone();
                    stored.expires_at = account.expires_at;
                    stored.status = AccountStatus::Active;
                    stored.throttled_until = None;
                }
                Ok(account)
            }
            Err(e) => {
                warn!("账号 {} 刷新失败，标记为 expired", account.id);
                self.mark_status(&account.id, AccountStatus::Expired);
                Err(e)
            }
        }
    }

    async fn refresh_social_token(&self, account: &mut Account) -> Result<(), AppError> {
        let region = account.region.as_deref().unwrap_or("us-east-1");
        let url = format!("https://prod.{}.auth.desktop.kiro.dev/refreshToken", region);

        let resp = self.http_client
            .post(&url)
            .json(&serde_json::json!({ "refreshToken": account.refresh_token }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            warn!("Social Token 刷新失败: {} - {}", status, text);
            return Err(AppError::TokenRefreshFailed(format!("{}: {}", status, text)));
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct RefreshResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: Option<i64>,
        }

        let data: RefreshResponse = resp.json().await?;
        account.access_token = data.access_token;
        if let Some(rt) = data.refresh_token {
            account.refresh_token = rt;
        }
        account.expires_at = Some(Utc::now().timestamp_millis() + (data.expires_in.unwrap_or(3600) * 1000));

        info!("Social Token 刷新成功: {}", account.id);
        Ok(())
    }

    async fn refresh_idc_token(&self, account: &mut Account) -> Result<(), AppError> {
        let client_id = account.client_id.as_ref()
            .ok_or_else(|| AppError::TokenRefreshFailed("IDC 账号缺少 clientId".into()))?;
        let client_secret = account.client_secret.as_ref()
            .ok_or_else(|| AppError::TokenRefreshFailed("IDC 账号缺少 clientSecret".into()))?;

        let region = account.region.as_deref().unwrap_or("us-east-1");
        let url = format!("https://oidc.{}.amazonaws.com/token", region);

        let resp = self.http_client
            .post(&url)
            .json(&serde_json::json!({
                "clientId": client_id,
                "clientSecret": client_secret,
                "grantType": "refresh_token",
                "refreshToken": account.refresh_token
            }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            warn!("IDC Token 刷新失败: {} - {}", status, text);
            return Err(AppError::TokenRefreshFailed(format!("{}: {}", status, text)));
        }

        // IDC 返回 snake_case（与 Social 的 camelCase 不同）
        #[derive(Deserialize)]
        struct IdcRefreshResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: Option<i64>,
        }

        let data: IdcRefreshResponse = resp.json().await?;
        account.access_token = data.access_token;
        if let Some(rt) = data.refresh_token {
            account.refresh_token = rt;
        }
        account.expires_at = Some(Utc::now().timestamp_millis() + (data.expires_in.unwrap_or(3600) * 1000));

        info!("IDC Token 刷新成功: {}", account.id);
        Ok(())
    }
}

