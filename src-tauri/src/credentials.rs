// 凭证文件监控（参考 proxycast）
use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;
use tauri::Manager;

/// 获取 Kiro 凭证文件路径
pub fn get_kiro_credentials_path() -> Result<PathBuf, String> {
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| "Failed to get home directory".to_string())?;
    
    Ok(PathBuf::from(home)
        .join(".aws")
        .join("sso")
        .join("cache")
        .join("kiro-auth-token.json"))
}

/// 启动凭证文件监控
pub fn start_credentials_watcher(app_handle: tauri::AppHandle) -> Result<(), String> {
    let credentials_path = get_kiro_credentials_path()?;
    
    if !credentials_path.exists() {
        return Err(format!("Credentials file not found: {:?}", credentials_path));
    }

    let watch_dir = credentials_path.parent()
        .ok_or("Failed to get credentials directory")?
        .to_path_buf();

    // 创建文件监控器
    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            let _ = tx.send(event);
        }
    }).map_err(|e| format!("Failed to create watcher: {}", e))?;

    // 监控凭证目录
    watcher.watch(&watch_dir, RecursiveMode::NonRecursive)
        .map_err(|e| format!("Failed to watch directory: {}", e))?;

    // 在后台线程处理文件变更事件
    std::thread::spawn(move || {
        let _watcher = watcher; // 保持 watcher 存活
        
        loop {
            match rx.recv_timeout(Duration::from_secs(1)) {
                Ok(event) => {
                    // 检查是否是凭证文件的修改事件
                    if let EventKind::Modify(_) = event.kind {
                        for path in event.paths {
                            if path.file_name() == credentials_path.file_name() {
                                // 发送事件到前端
                                let _ = app_handle.emit("credentials-changed", ());
                                tracing::info!("Credentials file changed, notifying frontend");
                                break;
                            }
                        }
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    // 超时，继续等待
                    continue;
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    // 通道断开，退出
                    break;
                }
            }
        }
    });

    Ok(())
}

/// 读取 Kiro 凭证文件
#[tauri::command]
pub async fn read_kiro_credentials() -> Result<String, String> {
    let credentials_path = get_kiro_credentials_path()?;
    
    std::fs::read_to_string(&credentials_path)
        .map_err(|e| format!("Failed to read credentials: {}", e))
}

/// 检查凭证文件是否存在
#[tauri::command]
pub async fn check_credentials_exists() -> Result<bool, String> {
    let credentials_path = get_kiro_credentials_path()?;
    Ok(credentials_path.exists())
}
