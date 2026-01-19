use chrono::Utc;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

use crate::error::AppError;
use crate::token_allocator::SmartTokenAllocator;

// 账号状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum AccountStatus {
    #[default]
    Active,
    Expired,
    Throttled,
    Error,
    Disabled,
    Banned,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub name: Option<String>,
    pub provider: Option<String>,
    #[serde(default)]
    pub auth_method: String,
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
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

        // 没有过期时间信息：
        // - 如果有 accessToken，假设有效，让请求去验证
        // - 如果没有 accessToken，需要刷新
        self.access_token.is_empty()
    }

    pub fn is_idc(&self) -> bool {
        // 有 clientId 就是 IDC 账号，或者 authMethod 明确指定为 idc
        self.client_id.is_some() || self.auth_method.to_lowercase() == "idc"
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
    http_client: reqwest::Client,
    accounts_file: RwLock<Option<String>>,
    // 防抖保存相关
    pending_saves: Arc<Mutex<HashSet<String>>>,
    save_task_running: Arc<Mutex<bool>>,
    // 智能 Token 分配器
    allocator: Arc<SmartTokenAllocator>,
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AccountManager {
    pub fn new() -> Self {
        Self {
            accounts: RwLock::new(Vec::new()),
            http_client: reqwest::Client::new(),
            accounts_file: RwLock::new(None),
            pending_saves: Arc::new(Mutex::new(HashSet::new())),
            save_task_running: Arc::new(Mutex::new(false)),
            allocator: Arc::new(SmartTokenAllocator::new()),
        }
    }

    /// 获取智能分配器（用于外部访问统计数据）
    pub fn get_allocator(&self) -> Arc<SmartTokenAllocator> {
        Arc::clone(&self.allocator)
    }

    /// 设置账号文件路径
    pub fn set_accounts_file(&self, path: &str) {
        *self.accounts_file.write() = Some(path.to_string());
    }

    pub fn load_from_json(&self, json_str: &str) -> Result<(), AppError> {
        // 支持两种格式：数组 [] 或对象 { "accounts": [] }
        let accounts: Vec<Account> = if json_str.trim().starts_with('[') {
            serde_json::from_str(json_str)
                .map_err(|e| AppError::ParseError(e.to_string()))?
        } else {
            #[derive(Deserialize)]
            struct AccountsConfig {
                accounts: Vec<Account>,
            }
            let config: AccountsConfig = serde_json::from_str(json_str)
                .map_err(|e| AppError::ParseError(e.to_string()))?;
            config.accounts
        };

        // 检查 refreshToken 长度
        for acc in &accounts {
            if acc.refresh_token.len() < 100 {
                warn!("账号 {} 的 refreshToken 长度 < 100，可能无效", acc.id);
            }
        }

        let mut stored = self.accounts.write();
        *stored = accounts;
        info!("加载了 {} 个账号", stored.len());
        Ok(())
    }

    /// 列出所有账号（预留给管理 API）
    pub fn list_accounts(&self) -> Vec<Account> {
        self.accounts.read().clone()
    }

    /// 添加账号
    pub fn add_account(&self, account: Account) -> Result<(), AppError> {
        let mut accounts = self.accounts.write();
        accounts.push(account);
        Ok(())
    }

    /// 更新账号
    pub fn update_account<F>(&self, id: &str, update_fn: F) -> Result<(), AppError>
    where
        F: FnOnce(&mut Account),
    {
        let mut accounts = self.accounts.write();
        let account = accounts.iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| AppError::BadRequest(format!("账号 {} 不存在", id)))?;
        update_fn(account);
        Ok(())
    }

    /// 删除账号
    pub fn delete_account(&self, id: &str) -> Result<(), AppError> {
        let mut accounts = self.accounts.write();
        let index = accounts.iter()
            .position(|a| a.id == id)
            .ok_or_else(|| AppError::BadRequest(format!("账号 {} 不存在", id)))?;
        accounts.remove(index);
        Ok(())
    }

    /// 获取账号文件路径
    pub fn get_accounts_file(&self) -> Option<String> {
        self.accounts_file.read().clone()
    }

    /// 保存账号列表到文件
    pub fn save_accounts_to_file(&self, accounts: &[Account]) -> Result<(), AppError> {
        if let Some(ref file_path) = *self.accounts_file.read() {
            // 直接保存为数组格式，不包装在 {"accounts": ...} 中
            std::fs::write(file_path, serde_json::to_string_pretty(&accounts).unwrap_or_default())
                .map_err(|e| AppError::BadRequest(format!("保存账号文件失败: {}", e)))?;
            info!("账号已保存到文件: {}", file_path);
        }
        Ok(())
    }

    pub async fn get_account(&self) -> Result<Account, AppError> {
        // 最多尝试 3 次
        for attempt in 0..3 {
            // 使用智能分配器选择最优账号
            let accounts = self.accounts.read().clone();
            let mut account = self.allocator.get_best_account(&accounts)?;

            // 检查是否需要刷新 Token
            if account.is_expired() {
                info!("账号 {} Token 即将过期，刷新中...", account.id);
                match self.refresh_token(account.clone()).await {
                    Ok(refreshed) => {
                        account = refreshed;
                        return Ok(account);
                    }
                    Err(e) => {
                        // 刷新失败，记录失败并尝试下一个账号
                        warn!("账号 {} 刷新失败 (尝试 {}/3): {}", account.id, attempt + 1, e);
                        self.allocator.record_usage(&account.id, false);
                        self.mark_status(&account.id, AccountStatus::Expired);
                        continue;
                    }
                }
            }

            return Ok(account);
        }

        Err(AppError::NoToken)
    }

    /// 记录账号使用成功
    pub fn record_success(&self, account_id: &str) {
        self.allocator.record_usage(account_id, true);
    }

    /// 记录账号使用失败
    pub fn record_failure(&self, account_id: &str) {
        self.allocator.record_usage(account_id, false);
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
        drop(accounts); // 释放锁
        // 防抖保存到文件
        self.schedule_save(account_id.to_string());
    }

    /// 设置账号启用/禁用
    pub fn set_account_enabled(&self, account_id: &str, enabled: bool) -> Result<(), AppError> {
        let result = {
            let mut accounts = self.accounts.write();
            if let Some(acc) = accounts.iter_mut().find(|a| a.id == account_id) {
                acc.enabled = enabled;
                acc.status = if enabled { AccountStatus::Active } else { AccountStatus::Disabled };
                info!("账号 {} 已{}", account_id, if enabled { "启用" } else { "禁用" });
                Ok(())
            } else {
                Err(AppError::BadRequest(format!("账号 {} 不存在", account_id)))
            }
        };
        
        if result.is_ok() {
            // 防抖保存到文件
            self.schedule_save(account_id.to_string());
        }
        
        result
    }

    /// 更新账号文件中指定账号的 token（防抖保存）
    fn schedule_save(&self, account_id: String) {
        let pending_saves = Arc::clone(&self.pending_saves);
        let save_task_running = Arc::clone(&self.save_task_running);
        let accounts = Arc::new(self.accounts.read().clone());
        let accounts_file = self.accounts_file.read().clone();
        
        tokio::spawn(async move {
            // 添加到待保存队列
            {
                let mut pending = pending_saves.lock().await;
                pending.insert(account_id);
            }
            
            // 检查是否已有保存任务在运行
            {
                let mut running = save_task_running.lock().await;
                if *running {
                    // 已有任务在运行，直接返回
                    return;
                }
                *running = true;
            }
            
            // 等待 1 秒（防抖延迟）
            sleep(Duration::from_secs(1)).await;
            
            // 批量保存所有待保存的账号
            let account_ids: Vec<String> = {
                let mut pending = pending_saves.lock().await;
                let ids: Vec<String> = pending.drain().collect();
                ids
            };
            
            if !account_ids.is_empty() {
                Self::flush_saves_to_file(accounts, accounts_file, &account_ids).await;
            }
            
            // 标记任务完成
            {
                let mut running = save_task_running.lock().await;
                *running = false;
            }
        });
    }
    
    /// 批量保存账号到文件
    async fn flush_saves_to_file(
        accounts: Arc<Vec<Account>>,
        accounts_file: Option<String>,
        account_ids: &[String],
    ) {
        if let Some(ref path) = accounts_file {
            // 读取原文件
            let content = match tokio::fs::read_to_string(path).await {
                Ok(c) => c,
                Err(e) => {
                    warn!("读取账号文件失败: {}", e);
                    return;
                }
            };
            
            // 解析为 JSON
            let mut json: serde_json::Value = match serde_json::from_str(&content) {
                Ok(j) => j,
                Err(e) => {
                    warn!("解析账号文件失败: {}", e);
                    return;
                }
            };
            
            // 找到并更新对应账号
            let accounts_array = if json.is_array() {
                json.as_array_mut()
            } else {
                json.get_mut("accounts").and_then(|a| a.as_array_mut())
            };
            
            let mut has_changes = false;
            
            if let Some(accounts_array) = accounts_array {
                for account_id in account_ids {
                    // 从内存中找到账号
                    if let Some(account) = accounts.iter().find(|a| &a.id == account_id) {
                        // 在 JSON 中找到并更新
                        for acc in accounts_array.iter_mut() {
                            if acc.get("id").and_then(|v| v.as_str()) == Some(&account.id) {
                                // 检查是否真的有变化
                                let old_access_token = acc.get("accessToken").and_then(|v| v.as_str()).unwrap_or("");
                                let old_refresh_token = acc.get("refreshToken").and_then(|v| v.as_str()).unwrap_or("");
                                let old_expires_at = acc.get("expiresAt").and_then(|v| v.as_i64());
                                
                                if old_access_token != account.access_token 
                                    || old_refresh_token != account.refresh_token 
                                    || old_expires_at != account.expires_at 
                                {
                                    acc["accessToken"] = serde_json::Value::String(account.access_token.clone());
                                    acc["refreshToken"] = serde_json::Value::String(account.refresh_token.clone());
                                    if let Some(expires_at) = account.expires_at {
                                        acc["expiresAt"] = serde_json::Value::Number(expires_at.into());
                                    }
                                    acc["status"] = serde_json::to_value(&account.status).unwrap_or(serde_json::json!("active"));
                                    has_changes = true;
                                }
                                break;
                            }
                        }
                    }
                }
            }
            
            // 只有真正有变化时才写文件
            if has_changes {
                if let Err(e) = tokio::fs::write(path, serde_json::to_string_pretty(&json).unwrap_or_default()).await {
                    warn!("更新账号文件失败: {}", e);
                } else {
                    info!("批量更新 {} 个账号到文件", account_ids.len());
                }
            }
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
                {
                    let mut accounts = self.accounts.write();
                    if let Some(stored) = accounts.iter_mut().find(|a| a.id == account.id) {
                        stored.access_token = account.access_token.clone();
                        stored.refresh_token = account.refresh_token.clone();
                        stored.expires_at = account.expires_at;
                        stored.status = AccountStatus::Active;
                        stored.throttled_until = None;
                    }
                }
                // 防抖保存到文件
                self.schedule_save(account.id.clone());
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

        // 先获取原始响应文本用于调试
        let text = resp.text().await.unwrap_or_default();
        info!("Social Token 刷新响应: {}", &text[..text.len().min(500)]);

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct RefreshResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: Option<i64>,
        }

        let data: RefreshResponse = serde_json::from_str(&text)
            .map_err(|e| AppError::ParseError(format!("解析刷新响应失败: {} - 响应: {}", e, &text[..text.len().min(200)])))?;
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

        // 按文档用 JSON + camelCase
        let resp = self.http_client
            .post(&url)
            .header("Content-Type", "application/json")
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

        let text = resp.text().await.unwrap_or_default();
        info!("IDC Token 刷新响应: {}", &text[..text.len().min(500)]);

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct IdcRefreshResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: Option<i64>,
        }

        let data: IdcRefreshResponse = serde_json::from_str(&text)
            .map_err(|e| AppError::ParseError(format!("解析 IDC 刷新响应失败: {} - 响应: {}", e, &text[..text.len().min(200)])))?;
        account.access_token = data.access_token;
        if let Some(rt) = data.refresh_token {
            account.refresh_token = rt;
        }
        account.expires_at = Some(Utc::now().timestamp_millis() + (data.expires_in.unwrap_or(3600) * 1000));

        info!("IDC Token 刷新成功: {}", account.id);
        Ok(())
    }
}

