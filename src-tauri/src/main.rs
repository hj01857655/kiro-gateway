// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod account;
pub mod auth;
pub mod converter;
pub mod config;
pub mod error;
pub mod kiro_client;
pub mod models;
pub mod thinking_parser;
pub mod websearch;
pub mod logger;
pub mod metrics;
pub mod api_key;
pub mod server;
pub mod health_checker;
pub mod token_allocator;
pub mod config_generator;

fn main() {
    // 初始化 Logger
    logger::init_logger();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            // 获取 AppHandle
            let app_handle = app.handle().clone();
            
            // 启动 Axum 后端服务器
            tauri::async_runtime::spawn(async move {
                if let Err(e) = server::start_server(app_handle).await {
                    eprintln!("后端服务器启动失败: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
