// 嵌入式后端服务器管理
use std::process::{Child, Command};
use std::sync::{Arc, Mutex};
use tauri::Manager;

pub struct BackendServer {
    process: Arc<Mutex<Option<Child>>>,
}

impl BackendServer {
    pub fn new() -> Self {
        Self {
            process: Arc::new(Mutex::new(None)),
        }
    }

    /// 启动后端服务器
    pub fn start(&self, app_handle: tauri::AppHandle) -> Result<String, String> {
        let mut process_guard = self.process.lock().unwrap();
        
        // 检查是否已经运行
        if let Some(ref mut child) = *process_guard {
            if let Ok(None) = child.try_wait() {
                return Ok("Backend server is already running".to_string());
            }
        }

        // 获取后端可执行文件路径
        let backend_path = if cfg!(debug_assertions) {
            // 开发模式：使用 cargo run
            let workspace_dir = std::env::current_dir()
                .map_err(|e| format!("Failed to get current dir: {}", e))?;
            let backend_dir = workspace_dir.parent()
                .ok_or("Failed to get parent dir")?;
            
            // 使用 cargo run 启动后端
            let child = Command::new("cargo")
                .arg("run")
                .arg("--release")
                .current_dir(backend_dir)
                .env("HOST", "127.0.0.1")
                .env("PORT", "8080")
                .spawn()
                .map_err(|e| format!("Failed to start backend: {}", e))?;
            
            *process_guard = Some(child);
            return Ok("Backend server started in development mode".to_string());
        } else {
            // 生产模式：使用打包的二进制文件
            let resource_path = app_handle
                .path()
                .resource_dir()
                .map_err(|e| format!("Failed to get resource dir: {}", e))?;
            
            #[cfg(target_os = "windows")]
            let backend_exe = resource_path.join("kiro-gateway.exe");
            #[cfg(not(target_os = "windows"))]
            let backend_exe = resource_path.join("kiro-gateway");
            
            if !backend_exe.exists() {
                return Err(format!("Backend executable not found: {:?}", backend_exe));
            }
            
            backend_exe
        };

        // 启动后端进程
        let child = Command::new(backend_path)
            .env("HOST", "127.0.0.1")
            .env("PORT", "8080")
            .spawn()
            .map_err(|e| format!("Failed to start backend: {}", e))?;

        *process_guard = Some(child);
        Ok("Backend server started".to_string())
    }

    /// 停止后端服务器
    pub fn stop(&self) -> Result<String, String> {
        let mut process_guard = self.process.lock().unwrap();
        
        if let Some(mut child) = process_guard.take() {
            child.kill()
                .map_err(|e| format!("Failed to kill backend process: {}", e))?;
            child.wait()
                .map_err(|e| format!("Failed to wait for backend process: {}", e))?;
            Ok("Backend server stopped".to_string())
        } else {
            Ok("Backend server is not running".to_string())
        }
    }

    /// 检查后端服务器状态
    pub fn is_running(&self) -> bool {
        let mut process_guard = self.process.lock().unwrap();
        
        if let Some(ref mut child) = *process_guard {
            match child.try_wait() {
                Ok(None) => true,  // 进程仍在运行
                _ => false,        // 进程已退出或出错
            }
        } else {
            false
        }
    }
}

impl Drop for BackendServer {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}
