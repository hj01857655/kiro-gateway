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
        // 不再使用默认的 data/accounts.json，统一从用户数据目录读取
        let accounts_file = env::var("ACCOUNTS_FILE").ok();

        // 验证端口范围
        let port = env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .unwrap_or_else(|_| {
                tracing::warn!("无效的 PORT 环境变量，使用默认值 8080");
                8080
            });

        // 验证主机地址
        let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let host = if host.is_empty() {
            tracing::warn!("HOST 为空，使用默认值 127.0.0.1");
            "127.0.0.1".to_string()
        } else {
            host
        };

        Self {
            host,
            port,
            kiro_endpoint: env::var("KIRO_ENDPOINT")
                .unwrap_or_else(|_| "https://codewhisperer.us-east-1.amazonaws.com".to_string()),
            api_key: env::var("API_KEY").ok(),
            accounts_file,
            accounts_json: env::var("ACCOUNTS_JSON").ok(),
            machine_id: env::var("MACHINE_ID")
                .ok()
                .or_else(|| Some(generate_machine_id())),
        }
    }

    /// 验证配置是否有效
    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("端口不能为 0".to_string());
        }
        if self.host.is_empty() {
            return Err("主机地址不能为空".to_string());
        }
        if self.kiro_endpoint.is_empty() {
            return Err("Kiro 端点不能为空".to_string());
        }
        Ok(())
    }
}

/// 生成机器 ID（SHA256 哈希，64 字符）
fn generate_machine_id() -> String {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();

    // 使用主机名和用户名生成唯一标识
    if let Ok(hostname) = std::env::var("COMPUTERNAME").or_else(|_| std::env::var("HOSTNAME")) {
        hasher.update(hostname.as_bytes());
    }
    if let Ok(user) = std::env::var("USERNAME").or_else(|_| std::env::var("USER")) {
        hasher.update(user.as_bytes());
    }

    // 添加一些额外的熵
    hasher.update(b"kiro-gate");

    // 返回 64 字符的十六进制字符串
    format!("{:064x}", hasher.finalize())
}
