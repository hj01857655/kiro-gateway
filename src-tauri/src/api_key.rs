// API Key 管理系统
use chrono::Utc;
use hex;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKey {
    pub id: String,
    pub key: String,
    pub name: Option<String>,
    pub created_at: i64,
    pub last_used: Option<i64>,
    pub enabled: bool,
}

pub struct ApiKeyManager {
    keys: RwLock<HashMap<String, ApiKey>>,
    keys_file: RwLock<Option<String>>,
}

impl Default for ApiKeyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiKeyManager {
    pub fn new() -> Self {
        Self {
            keys: RwLock::new(HashMap::new()),
            keys_file: RwLock::new(None),
        }
    }

    /// 设置 API Key 文件路径
    pub fn set_keys_file(&self, path: &str) {
        *self.keys_file.write() = Some(path.to_string());
    }

    /// 获取 API Key 文件路径
    pub fn get_keys_file(&self) -> Option<String> {
        self.keys_file.read().clone()
    }

    /// 从 JSON 加载 API Keys
    pub fn load_from_json(&self, json_str: &str) -> Result<(), AppError> {
        let keys: Vec<ApiKey> = if json_str.trim().starts_with('[') {
            serde_json::from_str(json_str).map_err(|e| AppError::ParseError(e.to_string()))?
        } else {
            #[derive(Deserialize)]
            struct ApiKeysConfig {
                keys: Vec<ApiKey>,
            }
            let config: ApiKeysConfig =
                serde_json::from_str(json_str).map_err(|e| AppError::ParseError(e.to_string()))?;
            config.keys
        };

        let mut stored = self.keys.write();
        stored.clear();
        for key in keys {
            stored.insert(key.key.clone(), key);
        }
        tracing::info!("加载了 {} 个 API Key", stored.len());
        Ok(())
    }

    /// 生成新的 API Key
    pub fn generate_key(&self, name: Option<String>) -> Result<ApiKey, AppError> {
        // 生成 sk-{48位Base62字符} 格式（类似 OpenAI）
        use rand::Rng;
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        let mut rng = rand::thread_rng();
        let random_str: String = (0..48)
            .map(|_| {
                let idx = rng.gen_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        let key = format!("sk-{}", random_str);

        // 生成 ID（key 的 SHA256 前 16 位）
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hash = hasher.finalize();
        let id = hex::encode(&hash[..8]);

        let api_key = ApiKey {
            id,
            key: key.clone(),
            name,
            created_at: Utc::now().timestamp_millis(),
            last_used: None,
            enabled: true,
        };

        let mut keys = self.keys.write();
        keys.insert(key.clone(), api_key.clone());
        drop(keys);

        // 保存到文件
        self.save_to_file()?;

        Ok(api_key)
    }

    /// 生成新的 API Key（十六进制格式）
    pub fn generate_key_hex(&self, name: Option<String>) -> Result<ApiKey, AppError> {
        // 生成 sk-{64位十六进制} 格式（256-bit 随机数）
        let random_bytes: [u8; 32] = rand::random();
        let key = format!("sk-{}", hex::encode(random_bytes));

        // 生成 ID（key 的 SHA256 前 16 位）
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hash = hasher.finalize();
        let id = hex::encode(&hash[..8]);

        let api_key = ApiKey {
            id,
            key: key.clone(),
            name,
            created_at: Utc::now().timestamp_millis(),
            last_used: None,
            enabled: true,
        };

        let mut keys = self.keys.write();
        keys.insert(key.clone(), api_key.clone());
        drop(keys);

        // 保存到文件
        self.save_to_file()?;

        Ok(api_key)
    }

    /// 验证 API Key
    pub fn verify_key(&self, key: &str) -> Result<(), AppError> {
        let mut keys = self.keys.write();

        if let Some(api_key) = keys.get_mut(key) {
            if !api_key.enabled {
                return Err(AppError::BadRequest("API Key 已禁用".into()));
            }

            // 更新最后使用时间
            api_key.last_used = Some(Utc::now().timestamp_millis());
            drop(keys);

            // 保存到文件
            let _ = self.save_to_file();
            Ok(())
        } else {
            Err(AppError::BadRequest("无效的 API Key".into()))
        }
    }

    /// 列出所有 API Keys
    pub fn list_keys(&self) -> Vec<ApiKey> {
        self.keys.read().values().cloned().collect()
    }

    /// 删除 API Key
    pub fn delete_key(&self, id: &str) -> Result<(), AppError> {
        let mut keys = self.keys.write();

        // 找到对应的 key
        let key_to_remove = keys
            .iter()
            .find(|(_, v)| v.id == id)
            .map(|(k, _)| k.clone());

        if let Some(key) = key_to_remove {
            keys.remove(&key);
            drop(keys);
            self.save_to_file()?;
            Ok(())
        } else {
            Err(AppError::BadRequest(format!("API Key {} 不存在", id)))
        }
    }

    /// 启用/禁用 API Key
    pub fn set_key_enabled(&self, id: &str, enabled: bool) -> Result<(), AppError> {
        let mut keys = self.keys.write();

        let key = keys.iter_mut().find(|(_, v)| v.id == id).map(|(_, v)| v);

        if let Some(api_key) = key {
            api_key.enabled = enabled;
            drop(keys);
            self.save_to_file()?;
            Ok(())
        } else {
            Err(AppError::BadRequest(format!("API Key {} 不存在", id)))
        }
    }

    /// 保存到文件
    pub fn save_to_file(&self) -> Result<(), AppError> {
        let file_path = self.keys_file.read();
        if let Some(ref path) = *file_path {
            // 确保父目录存在
            if let Some(parent) = std::path::Path::new(path).parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| AppError::BadRequest(format!("创建目录失败: {}", e)))?;
            }

            let keys: Vec<ApiKey> = self.keys.read().values().cloned().collect();
            let json = serde_json::json!({ "keys": keys });
            std::fs::write(path, serde_json::to_string_pretty(&json).unwrap())
                .map_err(|e| AppError::BadRequest(format!("保存 API Keys 失败: {}", e)))?;
        }
        Ok(())
    }
}
