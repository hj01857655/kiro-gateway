// 加密模块 - 用于保护敏感数据
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{rand_core::RngCore, SaltString};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::AppError;

const NONCE_SIZE: usize = 12;

#[derive(Serialize, Deserialize)]
struct EncryptedData {
    ciphertext: String,
    nonce: String,
}

pub struct EncryptionManager {
    cipher: Aes256Gcm,
    #[allow(dead_code)]
    key_file: PathBuf,
}

impl EncryptionManager {
    /// 初始化加密管理器
    /// 如果密钥文件不存在，会自动生成新密钥
    pub fn new(data_dir: &Path) -> Result<Self, AppError> {
        let key_file = data_dir.join(".encryption_key");

        let key = if key_file.exists() {
            // 读取现有密钥
            Self::load_key(&key_file)?
        } else {
            // 生成新密钥并保存
            let key = Self::generate_key()?;
            Self::save_key(&key_file, &key)?;
            key
        };

        let cipher = Aes256Gcm::new(&key.into());

        Ok(Self { cipher, key_file })
    }

    /// 生成新的加密密钥
    fn generate_key() -> Result<[u8; 32], AppError> {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Ok(key)
    }

    /// 保存密钥到文件（使用机器特定的派生密钥加密）
    fn save_key(path: &PathBuf, key: &[u8; 32]) -> Result<(), AppError> {
        // 使用机器特定信息派生密钥
        let machine_key = Self::derive_machine_key()?;
        let cipher = Aes256Gcm::new(&machine_key.into());

        // 生成随机 nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密密钥
        let ciphertext = cipher
            .encrypt(nonce, key.as_ref())
            .map_err(|e| AppError::ConfigError(format!("加密密钥失败: {}", e)))?;

        let encrypted = EncryptedData {
            ciphertext: general_purpose::STANDARD.encode(&ciphertext),
            nonce: general_purpose::STANDARD.encode(nonce_bytes),
        };

        // 确保父目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::FileError(format!("创建目录失败: {}", e)))?;
        }

        // 保存加密后的密钥
        let json = serde_json::to_string(&encrypted)
            .map_err(|e| AppError::ParseError(format!("序列化失败: {}", e)))?;

        fs::write(path, json)
            .map_err(|e| AppError::FileError(format!("写入密钥文件失败: {}", e)))?;

        // 设置文件权限（仅当前用户可读写）
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)
                .map_err(|e| AppError::FileError(format!("获取文件权限失败: {}", e)))?
                .permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms)
                .map_err(|e| AppError::FileError(format!("设置文件权限失败: {}", e)))?;
        }

        Ok(())
    }

    /// 从文件加载密钥
    fn load_key(path: &PathBuf) -> Result<[u8; 32], AppError> {
        let json = fs::read_to_string(path)
            .map_err(|e| AppError::FileError(format!("读取密钥文件失败: {}", e)))?;

        let encrypted: EncryptedData = serde_json::from_str(&json)
            .map_err(|e| AppError::ParseError(format!("解析密钥文件失败: {}", e)))?;

        // 解码
        let ciphertext = general_purpose::STANDARD
            .decode(&encrypted.ciphertext)
            .map_err(|e| AppError::ParseError(format!("解码密文失败: {}", e)))?;

        let nonce_bytes = general_purpose::STANDARD
            .decode(&encrypted.nonce)
            .map_err(|e| AppError::ParseError(format!("解码 nonce 失败: {}", e)))?;

        let nonce = Nonce::from_slice(&nonce_bytes);

        // 使用机器特定密钥解密
        let machine_key = Self::derive_machine_key()?;
        let cipher = Aes256Gcm::new(&machine_key.into());

        let plaintext = cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| AppError::ConfigError(format!("解密密钥失败: {}", e)))?;

        if plaintext.len() != 32 {
            return Err(AppError::ConfigError("密钥长度错误".into()));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&plaintext);
        Ok(key)
    }

    /// 派生机器特定的密钥（用于保护主密钥）
    fn derive_machine_key() -> Result<[u8; 32], AppError> {
        // 收集机器特定信息
        let mut machine_info = String::new();

        if let Ok(hostname) = std::env::var("COMPUTERNAME")
            .or_else(|_| std::env::var("HOSTNAME"))
        {
            machine_info.push_str(&hostname);
        }

        if let Ok(user) = std::env::var("USERNAME").or_else(|_| std::env::var("USER")) {
            machine_info.push_str(&user);
        }

        // 添加固定盐值
        machine_info.push_str("kiro-gateway-v1");

        // 使用 Argon2 派生密钥
        let salt = SaltString::from_b64("kiroGatewayFixedSalt123456789012")
            .map_err(|e| AppError::ConfigError(format!("创建盐值失败: {}", e)))?;

        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(machine_info.as_bytes(), &salt)
            .map_err(|e| AppError::ConfigError(format!("派生密钥失败: {}", e)))?;

        let hash_bytes = hash.hash.ok_or_else(|| {
            AppError::ConfigError("密钥派生失败".into())
        })?;

        let mut key = [0u8; 32];
        key.copy_from_slice(&hash_bytes.as_bytes()[..32]);
        Ok(key)
    }

    /// 加密字符串
    pub fn encrypt(&self, plaintext: &str) -> Result<String, AppError> {
        // 生成随机 nonce
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // 加密
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| AppError::ConfigError(format!("加密失败: {}", e)))?;

        let encrypted = EncryptedData {
            ciphertext: general_purpose::STANDARD.encode(&ciphertext),
            nonce: general_purpose::STANDARD.encode(nonce_bytes),
        };

        serde_json::to_string(&encrypted)
            .map_err(|e| AppError::ParseError(format!("序列化失败: {}", e)))
    }

    /// 解密字符串
    pub fn decrypt(&self, encrypted_str: &str) -> Result<String, AppError> {
        let encrypted: EncryptedData = serde_json::from_str(encrypted_str)
            .map_err(|e| AppError::ParseError(format!("解析加密数据失败: {}", e)))?;

        // 解码
        let ciphertext = general_purpose::STANDARD
            .decode(&encrypted.ciphertext)
            .map_err(|e| AppError::ParseError(format!("解码密文失败: {}", e)))?;

        let nonce_bytes = general_purpose::STANDARD
            .decode(&encrypted.nonce)
            .map_err(|e| AppError::ParseError(format!("解码 nonce 失败: {}", e)))?;

        let nonce = Nonce::from_slice(&nonce_bytes);

        // 解密
        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| AppError::ConfigError(format!("解密失败: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| AppError::ParseError(format!("解码字符串失败: {}", e)))
    }

    /// 检查字符串是否已加密
    pub fn is_encrypted(data: &str) -> bool {
        serde_json::from_str::<EncryptedData>(data).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_encryption_decryption() {
        let temp_dir = env::temp_dir().join("kiro-test");
        fs::create_dir_all(&temp_dir).unwrap();

        let manager = EncryptionManager::new(&temp_dir).unwrap();

        let plaintext = "my-secret-refresh-token-12345";
        let encrypted = manager.encrypt(plaintext).unwrap();
        let decrypted = manager.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, decrypted);
        assert_ne!(plaintext, encrypted);

        // 清理
        fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_is_encrypted() {
        let encrypted = r#"{"ciphertext":"abc","nonce":"def"}"#;
        let plaintext = "not encrypted";

        assert!(EncryptionManager::is_encrypted(encrypted));
        assert!(!EncryptionManager::is_encrypted(plaintext));
    }
}
