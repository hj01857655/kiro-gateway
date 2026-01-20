// kiro-gateway 日志模块
// 独立版本 - 使用内存存储和 tracing

use serde::Serialize;
use std::collections::VecDeque;
use std::sync::OnceLock;
use tokio::sync::RwLock;

// 全局日志存储（最多保留 1000 条）
static LOGS: OnceLock<RwLock<VecDeque<LogEntry>>> = OnceLock::new();

/// 日志条目
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
}

/// 初始化日志存储
#[allow(dead_code)]
pub fn init_logger() {
    LOGS.get_or_init(|| RwLock::new(VecDeque::with_capacity(1000)));
}

/// 脱敏处理敏感信息
fn sanitize_message(message: &str) -> String {
    let mut sanitized = message.to_string();
    
    // 脱敏 Token（保留前 10 个字符）
    // 匹配 "accessToken": "eyJ..." 或 "refreshToken": "aor..."
    let token_patterns = [
        (r#""accessToken"\s*:\s*"([^"]{10})[^"]*""#, r#""accessToken": "$1...""#),
        (r#""refreshToken"\s*:\s*"([^"]{10})[^"]*""#, r#""refreshToken": "$1...""#),
        (r#"accessToken=([a-zA-Z0-9]{10})[a-zA-Z0-9]*"#, "accessToken=$1..."),
        (r#"refreshToken=([a-zA-Z0-9]{10})[a-zA-Z0-9]*"#, "refreshToken=$1..."),
    ];
    
    for (pattern, replacement) in &token_patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            sanitized = re.replace_all(&sanitized, *replacement).to_string();
        }
    }
    
    // 脱敏 Authorization header（保留前 10 个字符）
    if let Ok(re) = regex::Regex::new(r#"Bearer\s+([a-zA-Z0-9]{10})[a-zA-Z0-9._-]*"#) {
        sanitized = re.replace_all(&sanitized, "Bearer $1...").to_string();
    }
    
    // 脱敏 API Key（保留 sk- 前缀和前 8 个字符）
    if let Ok(re) = regex::Regex::new(r#"sk-([a-zA-Z0-9]{8})[a-zA-Z0-9]*"#) {
        sanitized = re.replace_all(&sanitized, "sk-$1...").to_string();
    }
    
    // 脱敏 clientSecret（保留前 10 个字符）
    if let Ok(re) = regex::Regex::new(r#""clientSecret"\s*:\s*"([^"]{10})[^"]*""#) {
        sanitized = re.replace_all(&sanitized, r#""clientSecret": "$1...""#).to_string();
    }
    
    // 脱敏 email（保留前 3 个字符和域名）
    if let Ok(re) = regex::Regex::new(r#"([a-zA-Z0-9]{3})[a-zA-Z0-9._-]*@([a-zA-Z0-9.-]+)"#) {
        sanitized = re.replace_all(&sanitized, "$1***@$2").to_string();
    }
    
    sanitized
}

/// 同步发送日志（用于非异步上下文）
#[allow(dead_code)]
pub fn emit_log_sync(level: &str, target: &str, message: &str) {
    // 脱敏处理
    let sanitized_message = sanitize_message(message);
    
    let entry = LogEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        level: level.to_string(),
        target: target.to_string(),
        message: sanitized_message.clone(),
    };

    // 同时输出到 tracing（使用固定 target）
    match level {
        "INFO" => tracing::info!(target: "kiro_gateway", "[{}] {}", target, sanitized_message),
        "DEBUG" => tracing::debug!(target: "kiro_gateway", "[{}] {}", target, sanitized_message),
        "WARN" => tracing::warn!(target: "kiro_gateway", "[{}] {}", target, sanitized_message),
        "ERROR" => tracing::error!(target: "kiro_gateway", "[{}] {}", target, sanitized_message),
        _ => {}
    }

    // 存储到内存（用于 /admin/logs API）
    if let Some(logs) = LOGS.get() {
        if let Ok(mut guard) = logs.try_write() {
            guard.push_back(entry);
            if guard.len() > 1000 {
                guard.pop_front();
            }
        }
    }
}

/// 获取所有日志
pub async fn get_logs() -> Vec<LogEntry> {
    if let Some(logs) = LOGS.get() {
        logs.read().await.iter().cloned().collect()
    } else {
        Vec::new()
    }
}

/// 清空日志
pub async fn clear_logs() {
    if let Some(logs) = LOGS.get() {
        logs.write().await.clear();
    }
}

/// 便捷宏：发送 INFO 日志
#[macro_export]
macro_rules! kirogate_info {
    ($($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            $crate::logger::emit_log_sync("INFO", "kiro_gateway", &msg);
        }
    };
}

/// 便捷宏：发送 DEBUG 日志
#[macro_export]
macro_rules! kirogate_debug {
    ($($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            $crate::logger::emit_log_sync("DEBUG", "kiro_gateway", &msg);
        }
    };
}

/// 便捷宏：发送 WARN 日志
#[macro_export]
macro_rules! kirogate_warn {
    ($($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            $crate::logger::emit_log_sync("WARN", "kiro_gateway", &msg);
        }
    };
}

/// 便捷宏：发送 ERROR 日志
#[macro_export]
macro_rules! kirogate_error {
    ($($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            $crate::logger::emit_log_sync("ERROR", "kiro_gateway", &msg);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_token() {
        let message = r#"{"accessToken":"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9","refreshToken":"aorAAAABBBBCCCCDDDD"}"#;
        let sanitized = sanitize_message(message);
        assert!(sanitized.contains(r#""accessToken":"eyJhbGciOi...""#));
        assert!(sanitized.contains(r#""refreshToken":"aorAAAABBB...""#));
    }

    #[test]
    fn test_sanitize_bearer() {
        let message = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let sanitized = sanitize_message(message);
        assert!(sanitized.contains("Bearer eyJhbGciOi..."));
    }

    #[test]
    fn test_sanitize_api_key() {
        let message = "API Key: sk-1234567890abcdef";
        let sanitized = sanitize_message(message);
        assert!(sanitized.contains("sk-12345678..."));
    }

    #[test]
    fn test_sanitize_email() {
        let message = "User email: user123@example.com";
        let sanitized = sanitize_message(message);
        assert!(sanitized.contains("use***@example.com"));
    }
}
