// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

pub mod account;
pub mod api_key;
pub mod auth;
pub mod config;
pub mod config_generator;
pub mod converter;
pub mod encryption;
pub mod error;
pub mod health_checker;
pub mod kiro_client;
pub mod logger;
pub mod metrics;
pub mod models;
pub mod server;
pub mod thinking_parser;
pub mod token_allocator;
pub mod websearch;

// Tauri 命令：代理 HTTP 请求到本地 Axum 服务器
#[tauri::command]
async fn proxy_request(
    app: tauri::AppHandle,
    method: String,
    path: String,
    body: Option<String>,
) -> Result<String, String> {
    let url = format!("http://127.0.0.1:8080{}", path);
    let client = reqwest::Client::new();

    // 读取 Admin Token
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("获取数据目录失败: {}", e))?;
    let admin_token_file = data_dir.join(".admin_token");
    let admin_token = if admin_token_file.exists() {
        std::fs::read_to_string(&admin_token_file)
            .map_err(|e| format!("读取 Admin Token 失败: {}", e))?
            .trim()
            .to_string()
    } else {
        return Err("Admin Token 未初始化".to_string());
    };

    let request = match method.as_str() {
        "GET" => client.get(&url).header("x-admin-token", &admin_token),
        "POST" => {
            let mut req = client.post(&url).header("x-admin-token", &admin_token);
            if let Some(body_str) = body {
                req = req.header("Content-Type", "application/json").body(body_str);
            }
            req
        }
        "PATCH" => {
            let mut req = client.patch(&url).header("x-admin-token", &admin_token);
            if let Some(body_str) = body {
                req = req.header("Content-Type", "application/json").body(body_str);
            }
            req
        }
        "DELETE" => client.delete(&url).header("x-admin-token", &admin_token),
        _ => return Err("Unsupported method".to_string()),
    };

    match request.send().await {
        Ok(response) => {
            match response.text().await {
                Ok(text) => Ok(text),
                Err(e) => Err(format!("Failed to read response: {}", e)),
            }
        }
        Err(e) => Err(format!("Request failed: {}", e)),
    }
}

// Tauri 命令：获取数据目录路径
#[tauri::command]
fn get_data_dir(app: tauri::AppHandle) -> Result<String, String> {
    match app.path().app_data_dir() {
        Ok(path) => Ok(path.to_string_lossy().to_string()),
        Err(e) => Err(format!("获取数据目录失败: {}", e)),
    }
}

// Tauri 命令：获取应用版本号
#[tauri::command]
fn get_app_version(app: tauri::AppHandle) -> Result<String, String> {
    Ok(app.package_info().version.to_string())
}

// Tauri 命令：打开数据目录
#[tauri::command]
async fn open_data_dir(app: tauri::AppHandle) -> Result<(), String> {
    let data_dir = app.path().app_data_dir()
        .map_err(|e| format!("获取数据目录失败: {}", e))?;
    
    // 确保目录存在
    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("创建数据目录失败: {}", e))?;
    }
    
    // 使用系统默认程序打开目录
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(data_dir)
            .spawn()
            .map_err(|e| format!("打开目录失败: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(data_dir)
            .spawn()
            .map_err(|e| format!("打开目录失败: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(data_dir)
            .spawn()
            .map_err(|e| format!("打开目录失败: {}", e))?;
    }
    
    Ok(())
}

fn main() {
    // 初始化 Logger
    logger::init_logger();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            proxy_request,
            get_data_dir,
            get_app_version,
            open_data_dir
        ])
        .setup(|app| {
            // 获取 AppHandle
            let app_handle = app.handle().clone();

            // 启动 Axum 后端服务器
            tauri::async_runtime::spawn(async move {
                println!("正在启动后端服务器...");
                match server::start_server(app_handle).await {
                    Ok(_) => println!("后端服务器已启动"),
                    Err(e) => {
                        eprintln!("后端服务器启动失败: {}", e);
                        eprintln!("错误详情: {:?}", e);
                    }
                }
            });

            println!("Tauri 应用初始化完成");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
