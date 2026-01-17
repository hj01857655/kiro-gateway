use tauri::Manager;

mod backend;
mod credentials;

use backend::BackendServer;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 初始化后端服务器
            let backend = BackendServer::new();
            app.manage(backend);
            
            // 启动后端服务器
            let app_handle = app.handle().clone();
            let backend_state = app.state::<BackendServer>();
            if let Err(e) = backend_state.start(app_handle.clone()) {
                tracing::error!("Failed to start backend server: {}", e);
            }
            
            // 启动凭证文件监控
            if let Err(e) = credentials::start_credentials_watcher(app_handle.clone()) {
                tracing::warn!("Failed to start credentials watcher: {}", e);
            }
            
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_backend_url,
            start_backend_server,
            stop_backend_server,
            check_backend_status,
            credentials::read_kiro_credentials,
            credentials::check_credentials_exists,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Tauri 命令：获取后端服务器 URL
#[tauri::command]
fn get_backend_url() -> String {
    "http://127.0.0.1:8080".to_string()
}

// Tauri 命令：启动后端服务器
#[tauri::command]
async fn start_backend_server(
    backend: tauri::State<'_, BackendServer>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    backend.start(app)
}

// Tauri 命令：停止后端服务器
#[tauri::command]
async fn stop_backend_server(
    backend: tauri::State<'_, BackendServer>,
) -> Result<String, String> {
    backend.stop()
}

// Tauri 命令：检查后端服务器状态
#[tauri::command]
async fn check_backend_status(
    backend: tauri::State<'_, BackendServer>,
) -> Result<bool, String> {
    Ok(backend.is_running())
}
