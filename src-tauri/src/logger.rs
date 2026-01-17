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

/// 同步发送日志（用于非异步上下文）
#[allow(dead_code)]
pub fn emit_log_sync(level: &str, target: &str, message: &str) {
    let entry = LogEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        level: level.to_string(),
        target: target.to_string(),
        message: message.to_string(),
    };

    // 同时输出到 tracing（使用固定 target）
    match level {
        "INFO" => tracing::info!(target: "kiro_gateway", "[{}] {}", target, message),
        "DEBUG" => tracing::debug!(target: "kiro_gateway", "[{}] {}", target, message),
        "WARN" => tracing::warn!(target: "kiro_gateway", "[{}] {}", target, message),
        "ERROR" => tracing::error!(target: "kiro_gateway", "[{}] {}", target, message),
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
