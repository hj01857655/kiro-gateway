use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
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
async fn start_backend_server() -> Result<String, String> {
    // TODO: 启动嵌入的 kiro-gateway 后端服务
    Ok("Backend server started".to_string())
}

// Tauri 命令：停止后端服务器
#[tauri::command]
async fn stop_backend_server() -> Result<String, String> {
    // TODO: 停止后端服务器
    Ok("Backend server stopped".to_string())
}
