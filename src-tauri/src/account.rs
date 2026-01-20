use chrono::Utc;
use parking_lot::RwLock;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

use crate::encryption::EncryptionManager;
use crate::error::AppError;
use crate::token_allocator::SmartTokenAllocator;

// 自定义反序列化函数：将数字时间戳转换为 ISO 8601 字符串
fn deserialize_expires_at<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Null => Ok(None),
        Value::String(s) => Ok(Some(s)),
        Value::Number(n) => {
            // 将数字时间戳转换为 ISO 8601 字符串
            if let Some(millis) = n.as_i64() {
                let dt = chrono::DateTime::<Utc>::from_timestamp_millis(millis)
                    .ok_or_else(|| Error::custom("invalid timestamp"))?;
                Ok(Some(dt.to_rfc3339()))
            } else {
                Err(Error::custom("invalid timestamp format"))
            }
        }
        _ => Err(Error::custom("expected string or number for expires_at")),
    }
}

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
    QuotaExhausted,  // 配额用尽（100%）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub name: Option<String>,
    pub provider: Option<String>,
    pub email: Option<String>,  // 新增 email 字段用于去重
    #[serde(default)]
    pub auth_method: String,
    #[serde(default)]
    pub access_token: String,
    #[serde(default)]
    pub refresh_token: String,
    #[serde(default)]
    pub profile_arn: String,
    pub region: Option<String>,
    #[serde(default, deserialize_with = "deserialize_expires_at")]
    pub expires_at: Option<String>,
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
    // 配额缓存
    #[serde(skip)]
    pub quota_cache: Option<serde_json::Value>,
    #[serde(skip)]
    pub quota_cached_at: Option<i64>,
}

fn default_enabled() -> bool {
    true
}

impl Account {
    pub fn is_expired(&self) -> bool {
        let buffer_ms = 5 * 60 * 1000;
        let now = Utc::now().timestamp_millis();

        // 优先检查 expire 字段（ISO 8601 字符串）
        if let Some(ref expire_str) = self.expire {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(expire_str) {
                return now >= dt.timestamp_millis() - buffer_ms;
            }
        }

        // 检查 expires_at 字段（ISO 8601 字符串）
        if let Some(ref expires_at_str) = self.expires_at {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(expires_at_str) {
                return now >= dt.timestamp_millis() - buffer_ms;
            }
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

    /// 转换为 WebSearch 的 VerifyResult
    /// 自动根据账号类型设置正确的认证方法和参数
    pub fn to_verify_result(&self) -> crate::websearch::VerifyResult {
        let (auth_method, client_id, client_secret) = if self.is_idc() {
            // IDC 账号
            (
                "idc".to_string(),
                self.client_id.clone(),
                self.client_secret.clone(),
            )
        } else {
            // Social 账号
            ("social".to_string(), None, None)
        };

        crate::websearch::VerifyResult {
            refresh_token: self.refresh_token.clone(),
            auth_method,
            profile_arn: Some(self.profile_arn.clone()),
            client_id,
            client_secret,
            region: Some(
                self.region
                    .clone()
                    .unwrap_or_else(|| "us-east-1".to_string()),
            ),
        }
    }

    /// 检查配额缓存是否有效（5 分钟内）
    pub fn is_quota_cache_valid(&self) -> bool {
        if let (Some(_), Some(cached_at)) = (&self.quota_cache, self.quota_cached_at) {
            let now = Utc::now().timestamp_millis();
            let cache_ttl = 5 * 60 * 1000; // 5 分钟
            return now - cached_at < cache_ttl;
        }
        false
    }

    /// 设置配额缓存
    pub fn set_quota_cache(&mut self, quota: serde_json::Value) {
        self.quota_cache = Some(quota);
        self.quota_cached_at = Some(Utc::now().timestamp_millis());
    }

    /// 获取配额缓存
    pub fn get_quota_cache(&self) -> Option<&serde_json::Value> {
        if self.is_quota_cache_valid() {
            self.quota_cache.as_ref()
        } else {
            None
        }
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
    // Token 刷新锁（防止竞态条件）
    refresh_locks: Arc<parking_lot::Mutex<std::collections::HashMap<String, Arc<tokio::sync::Mutex<()>>>>>,
    // 加密管理器
    encryption: Option<Arc<EncryptionManager>>,
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
            refresh_locks: Arc::new(parking_lot::Mutex::new(std::collections::HashMap::new())),
            encryption: None,
        }
    }

    /// 设置加密管理器
    pub fn set_encryption(&mut self, encryption: Arc<EncryptionManager>) {
        self.encryption = Some(encryption);
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
        // 支持两种格式：
        // 1. 数组: [{...}, {...}]
        // 2. 单个对象: {...}
        let mut accounts_json: Vec<serde_json::Value> = if json_str.trim().starts_with('[') {
            // 数组格式
            serde_json::from_str(json_str).map_err(|e| AppError::ParseError(e.to_string()))?
        } else if json_str.trim().starts_with('{') {
            // 单个对象格式，转为数组
            let account: serde_json::Value = serde_json::from_str(json_str)
                .map_err(|e| AppError::ParseError(e.to_string()))?;
            vec![account]
        } else {
            return Err(AppError::ParseError("无效的 JSON 格式".into()));
        };

        // 如果有加密管理器，自动检测并解密敏感字段
        if let Some(ref encryption) = self.encryption {
            for acc_json in &mut accounts_json {
                // 解密 refreshToken
                if let Some(refresh_token) = acc_json.get("refreshToken").and_then(|v| v.as_str()) {
                    if EncryptionManager::is_encrypted(refresh_token) {
                        match encryption.decrypt(refresh_token) {
                            Ok(decrypted) => {
                                acc_json["refreshToken"] = serde_json::Value::String(decrypted);
                            }
                            Err(e) => {
                                warn!("解密 refreshToken 失败: {}", e);
                            }
                        }
                    }
                }
                
                // 解密 accessToken
                if let Some(access_token) = acc_json.get("accessToken").and_then(|v| v.as_str()) {
                    if EncryptionManager::is_encrypted(access_token) {
                        match encryption.decrypt(access_token) {
                            Ok(decrypted) => {
                                acc_json["accessToken"] = serde_json::Value::String(decrypted);
                            }
                            Err(e) => {
                                warn!("解密 accessToken 失败: {}", e);
                            }
                        }
                    }
                }
                
                // 解密 clientSecret（IDC 账号）
                if let Some(client_secret) = acc_json.get("clientSecret").and_then(|v| v.as_str()) {
                    if EncryptionManager::is_encrypted(client_secret) {
                        match encryption.decrypt(client_secret) {
                            Ok(decrypted) => {
                                acc_json["clientSecret"] = serde_json::Value::String(decrypted);
                            }
                            Err(e) => {
                                warn!("解密 clientSecret 失败: {}", e);
                            }
                        }
                    }
                }
            }
        }

        // 反序列化为 Account 对象
        let accounts: Vec<Account> = accounts_json
            .into_iter()
            .filter_map(|json| {
                serde_json::from_value(json)
                    .map_err(|e| {
                        warn!("解析账号失败: {}", e);
                        e
                    })
                    .ok()
            })
            .collect();

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

    /// 添加账号（带去重）
    /// 去重策略：email + provider 组合必须唯一
    pub fn add_account(&self, account: Account) -> Result<(), AppError> {
        let mut accounts = self.accounts.write();

        // 使用 email + provider 去重（最准确的方式）
        if let (Some(email), Some(provider)) = (&account.email, &account.provider) {
            if accounts.iter().any(|a| {
                if let (Some(e), Some(p)) = (&a.email, &a.provider) {
                    e == email && p == provider
                } else {
                    false
                }
            }) {
                return Err(AppError::BadRequest(format!(
                    "该账号已存在（{} - {}），请勿重复导入",
                    email, provider
                )));
            }
        }

        // 其次检查 ID（防止手动指定相同 ID）
        if accounts.iter().any(|a| a.id == account.id) {
            return Err(AppError::BadRequest(format!("账号 ID {} 已存在", account.id)));
        }

        accounts.push(account);
        Ok(())
    }

    /// 批量添加账号（带去重）
    pub fn add_accounts_batch(&self, new_accounts: Vec<Account>) -> Result<(usize, Vec<String>), AppError> {
        let mut accounts = self.accounts.write();
        let mut added_count = 0;
        let mut skipped = Vec::new();

        for account in new_accounts {
            // 使用 email + provider 去重
            if let (Some(email), Some(provider)) = (&account.email, &account.provider) {
                if accounts.iter().any(|a| {
                    if let (Some(e), Some(p)) = (&a.email, &a.provider) {
                        e == email && p == provider
                    } else {
                        false
                    }
                }) {
                    skipped.push(format!("账号 {} - {} 已存在", email, provider));
                    continue;
                }
            }

            // 检查 ID 是否重复
            if accounts.iter().any(|a| a.id == account.id) {
                skipped.push(format!("ID {} 已存在", account.id));
                continue;
            }

            accounts.push(account);
            added_count += 1;
        }

        Ok((added_count, skipped))
    }

    /// 更新账号
    pub fn update_account<F>(&self, id: &str, update_fn: F) -> Result<(), AppError>
    where
        F: FnOnce(&mut Account),
    {
        let mut accounts = self.accounts.write();
        let account = accounts
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or_else(|| AppError::BadRequest(format!("账号 {} 不存在", id)))?;
        update_fn(account);
        Ok(())
    }

    /// 删除账号
    pub fn delete_account(&self, id: &str) -> Result<(), AppError> {
        let mut accounts = self.accounts.write();
        let index = accounts
            .iter()
            .position(|a| a.id == id)
            .ok_or_else(|| AppError::BadRequest(format!("账号 {} 不存在", id)))?;
        accounts.remove(index);
        Ok(())
    }

    /// 获取账号文件路径
    pub fn get_accounts_file(&self) -> Option<String> {
        self.accounts_file.read().clone()
    }

    /// 保存账号列表到文件（自动加密敏感字段）
    pub fn save_accounts_to_file(&self, accounts: &[Account]) -> Result<(), AppError> {
        if let Some(ref file_path) = *self.accounts_file.read() {
            // 确保父目录存在
            if let Some(parent) = std::path::Path::new(file_path).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| AppError::BadRequest(format!("创建目录失败: {}", e)))?;
            }

            // 如果有加密管理器，加密敏感字段
            let accounts_to_save: Vec<serde_json::Value> = if let Some(ref encryption) = self.encryption {
                accounts
                    .iter()
                    .map(|acc| {
                        let mut json = serde_json::to_value(acc).unwrap_or_default();
                        
                        // 加密 refreshToken
                        if let Some(refresh_token) = json.get("refreshToken").and_then(|v| v.as_str()) {
                            if !refresh_token.is_empty() && !EncryptionManager::is_encrypted(refresh_token) {
                                if let Ok(encrypted) = encryption.encrypt(refresh_token) {
                                    json["refreshToken"] = serde_json::Value::String(encrypted);
                                }
                            }
                        }
                        
                        // 加密 accessToken
                        if let Some(access_token) = json.get("accessToken").and_then(|v| v.as_str()) {
                            if !access_token.is_empty() && !EncryptionManager::is_encrypted(access_token) {
                                if let Ok(encrypted) = encryption.encrypt(access_token) {
                                    json["accessToken"] = serde_json::Value::String(encrypted);
                                }
                            }
                        }
                        
                        // 加密 clientSecret（IDC 账号）
                        if let Some(client_secret) = json.get("clientSecret").and_then(|v| v.as_str()) {
                            if !client_secret.is_empty() && !EncryptionManager::is_encrypted(client_secret) {
                                if let Ok(encrypted) = encryption.encrypt(client_secret) {
                                    json["clientSecret"] = serde_json::Value::String(encrypted);
                                }
                            }
                        }
                        
                        json
                    })
                    .collect()
            } else {
                // 没有加密管理器，直接保存明文
                accounts
                    .iter()
                    .map(|acc| serde_json::to_value(acc).unwrap_or_default())
                    .collect()
            };

            // 保存到文件
            std::fs::write(
                file_path,
                serde_json::to_string_pretty(&accounts_to_save).unwrap_or_default(),
            )
            .map_err(|e| AppError::BadRequest(format!("保存账号文件失败: {}", e)))?;
            
            if self.encryption.is_some() {
                info!("账号已加密保存到文件: {}", file_path);
            } else {
                info!("账号已保存到文件（未加密）: {}", file_path);
            }
        }
        Ok(())
    }

    pub async fn get_account(&self) -> Result<Account, AppError> {
        // 最多尝试 3 次
        for attempt in 0..3 {
            // 使用智能分配器选择最优账号（避免 clone 整个列表）
            let account_id = {
                let accounts = self.accounts.read();
                self.allocator.get_best_account(&accounts)?.id.clone()
            };

            // 获取账号副本用于后续操作
            let mut account = {
                let accounts = self.accounts.read();
                accounts.iter()
                    .find(|a| a.id == account_id)
                    .cloned()
                    .ok_or(AppError::NoToken)?
            };

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
                        warn!(
                            "账号 {} 刷新失败 (尝试 {}/3): {}",
                            account.id,
                            attempt + 1,
                            e
                        );
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

    /// 更新账号配额缓存
    pub fn update_quota_cache(&self, account_id: &str, quota: serde_json::Value) {
        let mut accounts = self.accounts.write();
        if let Some(acc) = accounts.iter_mut().find(|a| a.id == account_id) {
            acc.set_quota_cache(quota);
        }
    }

    /// 获取账号配额缓存
    pub fn get_quota_cache(&self, account_id: &str) -> Option<serde_json::Value> {
        let accounts = self.accounts.read();
        accounts
            .iter()
            .find(|a| a.id == account_id)
            .and_then(|acc| acc.get_quota_cache().cloned())
    }

    /// 设置账号启用/禁用
    pub fn set_account_enabled(&self, account_id: &str, enabled: bool) -> Result<(), AppError> {
        let result = {
            let mut accounts = self.accounts.write();
            if let Some(acc) = accounts.iter_mut().find(|a| a.id == account_id) {
                acc.enabled = enabled;
                acc.status = if enabled {
                    AccountStatus::Active
                } else {
                    AccountStatus::Disabled
                };
                info!(
                    "账号 {} 已{}",
                    account_id,
                    if enabled { "启用" } else { "禁用" }
                );
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
                                let old_access_token = acc
                                    .get("accessToken")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let old_refresh_token = acc
                                    .get("refreshToken")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                let old_expires_at = acc.get("expiresAt").and_then(|v| v.as_str());

                                if old_access_token != account.access_token
                                    || old_refresh_token != account.refresh_token
                                    || old_expires_at != account.expires_at.as_deref()
                                {
                                    acc["accessToken"] =
                                        serde_json::Value::String(account.access_token.clone());
                                    acc["refreshToken"] =
                                        serde_json::Value::String(account.refresh_token.clone());
                                    if let Some(expires_at) = &account.expires_at {
                                        acc["expiresAt"] =
                                            serde_json::Value::String(expires_at.clone());
                                    }
                                    acc["status"] = serde_json::to_value(&account.status)
                                        .unwrap_or(serde_json::json!("active"));
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
                if let Err(e) = tokio::fs::write(
                    path,
                    serde_json::to_string_pretty(&json).unwrap_or_default(),
                )
                .await
                {
                    warn!("更新账号文件失败: {}", e);
                } else {
                    info!("批量更新 {} 个账号到文件", account_ids.len());
                }
            }
        }
    }

    async fn refresh_token(&self, mut account: Account) -> Result<Account, AppError> {
        // 获取或创建该账号的刷新锁
        let lock = {
            let mut locks = self.refresh_locks.lock();
            locks
                .entry(account.id.clone())
                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                .clone()
        };

        // 获取锁，确保同一账号的刷新操作串行化
        let _guard = lock.lock().await;

        // 再次检查 token 是否已被其他线程刷新
        {
            let accounts = self.accounts.read();
            if let Some(stored) = accounts.iter().find(|a| a.id == account.id) {
                if !stored.is_expired() {
                    // Token 已被其他线程刷新，直接返回
                    info!("账号 {} 的 Token 已被其他线程刷新", account.id);
                    return Ok(stored.clone());
                }
                // 使用最新的账号数据
                account = stored.clone();
            }
        }

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
                        stored.expires_at = account.expires_at.clone();
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

        let resp = self
            .http_client
            .post(&url)
            .json(&serde_json::json!({ "refreshToken": account.refresh_token }))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            warn!("Social Token 刷新失败: {} - {}", status, text);
            return Err(AppError::TokenRefreshFailed(format!(
                "{}: {}",
                status, text
            )));
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
            // 新格式中包含 profileArn
            #[serde(default)]
            profile_arn: Option<String>,
            // AWS SSO 字段（IDC 账号可能包含，Social 账号没有）
            #[serde(default)]
            aws_sso_app_session_id: Option<String>,
            #[serde(default)]
            id_token: Option<String>,
            #[serde(default)]
            issued_token_type: Option<String>,
            #[serde(default)]
            origin_session_id: Option<String>,
        }

        let data: RefreshResponse = serde_json::from_str(&text).map_err(|e| {
            AppError::ParseError(format!(
                "解析刷新响应失败: {} - 响应: {}",
                e,
                &text[..text.len().min(200)]
            ))
        })?;
        account.access_token = data.access_token;
        if let Some(rt) = data.refresh_token {
            account.refresh_token = rt;
        }
        // 更新 profileArn（如果响应中包含）
        if let Some(profile_arn) = data.profile_arn {
            if !profile_arn.is_empty() {
                account.profile_arn = profile_arn;
            }
        }
        // 按 IDE 的方式设置 expiresAt：转换为 ISO 8601 字符串
        let expires_at_ms = Utc::now().timestamp_millis() + (data.expires_in.unwrap_or(3600) * 1000);
        let expires_at_dt = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(expires_at_ms)
            .unwrap_or_else(|| Utc::now());
        account.expires_at = Some(expires_at_dt.to_rfc3339());

        // 记录 AWS SSO 字段（如果存在）
        if data.aws_sso_app_session_id.is_some()
            || data.id_token.is_some()
            || data.issued_token_type.is_some()
            || data.origin_session_id.is_some()
        {
            info!(
                "Social Token 刷新成功（包含 AWS SSO 字段）: {} - session_id: {:?}, token_type: {:?}",
                account.id, data.aws_sso_app_session_id, data.issued_token_type
            );
        } else {
            info!("Social Token 刷新成功: {}", account.id);
        }
        Ok(())
    }

    async fn refresh_idc_token(&self, account: &mut Account) -> Result<(), AppError> {
        let client_id = account
            .client_id
            .as_ref()
            .ok_or_else(|| AppError::TokenRefreshFailed("IDC 账号缺少 clientId".into()))?;
        let client_secret = account
            .client_secret
            .as_ref()
            .ok_or_else(|| AppError::TokenRefreshFailed("IDC 账号缺少 clientSecret".into()))?;

        let region = account.region.as_deref().unwrap_or("us-east-1");
        let url = format!("https://oidc.{}.amazonaws.com/token", region);

        // 按文档用 JSON + camelCase
        let resp = self
            .http_client
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
            return Err(AppError::TokenRefreshFailed(format!(
                "{}: {}",
                status, text
            )));
        }

        let text = resp.text().await.unwrap_or_default();
        info!("IDC Token 刷新响应: {}", &text[..text.len().min(500)]);

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct IdcRefreshResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: Option<i64>,
            // AWS SSO 字段（可选，新格式中可能包含）
            #[serde(default)]
            aws_sso_app_session_id: Option<String>,
            #[serde(default)]
            id_token: Option<String>,
            #[serde(default)]
            issued_token_type: Option<String>,
            #[serde(default)]
            origin_session_id: Option<String>,
        }

        let data: IdcRefreshResponse = serde_json::from_str(&text).map_err(|e| {
            AppError::ParseError(format!(
                "解析 IDC 刷新响应失败: {} - 响应: {}",
                e,
                &text[..text.len().min(200)]
            ))
        })?;
        account.access_token = data.access_token;
        if let Some(rt) = data.refresh_token {
            account.refresh_token = rt;
        }
        // 按 IDE 的方式设置 expiresAt：转换为 ISO 8601 字符串
        let expires_at_ms = Utc::now().timestamp_millis() + (data.expires_in.unwrap_or(3600) * 1000);
        let expires_at_dt = chrono::DateTime::<chrono::Utc>::from_timestamp_millis(expires_at_ms)
            .unwrap_or_else(|| Utc::now());
        account.expires_at = Some(expires_at_dt.to_rfc3339());

        // 记录 AWS SSO 字段（如果存在）
        if data.aws_sso_app_session_id.is_some()
            || data.id_token.is_some()
            || data.issued_token_type.is_some()
            || data.origin_session_id.is_some()
        {
            info!(
                "IDC Token 刷新成功（包含 AWS SSO 字段）: {} - session_id: {:?}, token_type: {:?}",
                account.id, data.aws_sso_app_session_id, data.issued_token_type
            );
        } else {
            info!("IDC Token 刷新成功: {}", account.id);
        }
        Ok(())
    }
}
