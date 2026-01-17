// KiroGate 日志系统
// 不依赖 Tauri，使用 once_cell + tokio 实现

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use serde::Serialize;
use std::collections::VecDeque;

const MAX_LOGS: usize = 1000;

#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
}

static LOGS: Lazy<Mutex<VecDeque<LogEntry>>> = Lazy::new(|| Mutex::new(VecDeque::with_capacity(MAX_LOGS)));

/// 异步记录日志
pub async fn emit_log(level: &str, target: &str, message: &str) {
    let log = LogEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        level: level.to_string(),
        target: target.to_string(),
        message: message.to_string(),
    };

    let mut logs = LOGS.lock();
    logs.push_back(log);
    if logs.len() > MAX_LOGS {
        logs.pop_front();
    }
    drop(logs);

    // 输出到 tracing
    match level {
        "ERROR" => tracing::error!(target: "kiro_gate", "[{}] {}", target, message),
        "WARN" => tracing::warn!(target: "kiro_gate", "[{}] {}", target, message),
        "INFO" => tracing::info!(target: "kiro_gate", "[{}] {}", target, message),
        "DEBUG" => tracing::debug!(target: "kiro_gate", "[{}] {}", target, message),
        _ => tracing::info!(target: "kiro_gate", "[{}] {}", target, message),
    }
}

/// 同步记录日志
pub fn emit_log_sync(level: &str, target: &str, message: &str) {
    let log = LogEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        level: level.to_string(),
        target: target.to_string(),
        message: message.to_string(),
    };

    let mut logs = LOGS.lock();
    logs.push_back(log);
    if logs.len() > MAX_LOGS {
        logs.pop_front();
    }
    drop(logs);

    // 输出到 tracing
    match level {
        "ERROR" => tracing::error!(target: "kiro_gate", "[{}] {}", target, message),
        "WARN" => tracing::warn!(target: "kiro_gate", "[{}] {}", target, message),
        "INFO" => tracing::info!(target: "kiro_gate", "[{}] {}", target, message),
        "DEBUG" => tracing::debug!(target: "kiro_gate", "[{}] {}", target, message),
        _ => tracing::info!(target: "kiro_gate", "[{}] {}", target, message),
    }
}

/// 获取所有日志
pub async fn get_logs() -> Vec<LogEntry> {
    LOGS.lock().iter().cloned().collect()
}

/// 清空日志
pub async fn clear_logs() {
    LOGS.lock().clear();
}

/// 便捷宏
#[macro_export]
macro_rules! kirogate_info {
    ($target:expr, $($arg:tt)*) => {
        $crate::logger::emit_log_sync("INFO", $target, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! kirogate_debug {
    ($target:expr, $($arg:tt)*) => {
        $crate::logger::emit_log_sync("DEBUG", $target, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! kirogate_warn {
    ($target:expr, $($arg:tt)*) => {
        $crate::logger::emit_log_sync("WARN", $target, &format!($($arg)*))
    };
}

#[macro_export]
macro_rules! kirogate_error {
    ($target:expr, $($arg:tt)*) => {
        $crate::logger::emit_log_sync("ERROR", $target, &format!($($arg)*))
    };
}
