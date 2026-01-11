use std::env;

#[derive(Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub kiro_endpoint: String,
    pub api_key: Option<String>,
    pub accounts_file: Option<String>,
    pub accounts_json: Option<String>,
    pub machine_id: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            kiro_endpoint: env::var("KIRO_ENDPOINT")
                .unwrap_or_else(|_| "https://codewhisperer.us-east-1.amazonaws.com".to_string()),
            api_key: env::var("API_KEY").ok(),
            accounts_file: env::var("ACCOUNTS_FILE").ok(),
            accounts_json: env::var("ACCOUNTS_JSON").ok(),
            machine_id: env::var("MACHINE_ID").ok().or_else(|| Some(generate_machine_id())),
        }
    }
}

/// 生成机器 ID（SHA256 哈希，64 字符）
fn generate_machine_id() -> String {
    use sha2::{Sha256, Digest};

    let mut hasher = Sha256::new();

    // 使用主机名和用户名生成唯一标识
    if let Ok(hostname) = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME")) {
        hasher.update(hostname.as_bytes());
    }
    if let Ok(user) = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER")) {
        hasher.update(user.as_bytes());
    }

    // 添加一些额外的熵
    hasher.update(b"kiro-gate");

    // 返回 64 字符的十六进制字符串
    format!("{:064x}", hasher.finalize())
}
