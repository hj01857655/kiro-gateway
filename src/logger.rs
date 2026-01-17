// KiroGateway 日志发送模块
// 用于将日志事件发送到前端

use serde::Serialize;
use std::sync::OnceLock;
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

// 全局 AppHandle 存储
static APP_HANDLE: OnceLock<RwLock<Option<AppHandle>>> = OnceLock::new();

/// 日志条目
#[derive(Debug, Clone, Serialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub target: String,
    pub message: String,
}

/// 初始化日志发送器（在 main.rs setup 中调用）
pub fn init_logger(app_handle: AppHandle) {
    let lock = APP_HANDLE.get_or_init(|| RwLock::new(None));
    // 使用 blocking 方式设置，因为这在 setup 中同步调用
    if let Ok(mut guard) = lock.try_write() {
        *guard = Some(app_handle);
    }
}

/// 同步发送日志（用于非异步上下文）
pub fn emit_log_sync(level: &str, target: &str, message: &str) {
    let entry = LogEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        level: level.to_string(),
        target: target.to_string(),
        message: message.to_string(),
    };
    
    if let Some(lock) = APP_HANDLE.get() {
        if let Ok(guard) = lock.try_read() {
            if let Some(app) = guard.as_ref() {
                let _ = app.emit("kirogate-log", entry);
            }
        }
    }
}

/// 便捷宏：发送 INFO 日志
#[macro_export]
macro_rules! kirogate_info {
    ($($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            log::info!("[KiroGate] {}", msg);
            $crate::kiro_gate::logger::emit_log_sync("INFO", "kiro_gate", &msg);
        }
    };
}

/// 便捷宏：发送 DEBUG 日志
#[macro_export]
macro_rules! kirogate_debug {
    ($($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            log::debug!("[KiroGate] {}", msg);
            $crate::kiro_gate::logger::emit_log_sync("DEBUG", "kiro_gate", &msg);
        }
    };
}

/// 便捷宏：发送 WARN 日志
#[macro_export]
macro_rules! kirogate_warn {
    ($($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            log::warn!("[KiroGate] {}", msg);
            $crate::kiro_gate::logger::emit_log_sync("WARN", "kiro_gate", &msg);
        }
    };
}

/// 便捷宏：发送 ERROR 日志
#[macro_export]
macro_rules! kirogate_error {
    ($($arg:tt)*) => {
        {
            let msg = format!($($arg)*);
            log::error!("[KiroGate] {}", msg);
            $crate::kiro_gate::logger::emit_log_sync("ERROR", "kiro_gate", &msg);
        }
    };
}
