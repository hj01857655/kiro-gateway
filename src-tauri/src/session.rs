use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::error::AppError;

/// 完整会话数据（存储在 {sessionId}.json）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    /// 会话 ID
    pub session_id: String,
    
    /// 会话标题
    pub title: String,
    
    /// 工作区路径（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_directory: Option<String>,
    
    /// 对话历史
    pub history: Vec<serde_json::Value>,
    
    /// 是否隐藏
    #[serde(default)]
    pub hidden: bool,
}

/// 会话元数据（存储在 sessions.json 列表中）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    /// 会话 ID
    pub session_id: String,
    
    /// 会话标题
    pub title: String,
    
    /// 创建时间（Unix 时间戳字符串）
    pub date_created: String,
    
    /// 工作区路径（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_directory: Option<String>,
    
    /// 是否隐藏
    #[serde(default)]
    pub hidden: bool,
}

impl Session {
    /// 创建新会话
    pub fn new(title: String, workspace_directory: Option<String>) -> Self {
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            title,
            workspace_directory,
            history: Vec::new(),
            hidden: false,
        }
    }
}

/// 会话管理器
pub struct SessionManager {
    /// 会话存储根目录
    storage_root: PathBuf,
}

impl SessionManager {
    /// 创建新的会话管理器
    pub fn new(storage_root: PathBuf) -> Result<Self, AppError> {
        // 确保根目录存在
        fs::create_dir_all(&storage_root)
            .map_err(|e| AppError::ConfigError(format!("创建会话根目录失败: {}", e)))?;

        Ok(Self { storage_root })
    }

    /// 获取会话文件夹路径
    /// 
    /// 无工作区：{storage_root}/sessions/
    /// 有工作区：{storage_root}/workspace-sessions/{workspaceHash}/
    fn get_sessions_folder_path(&self, workspace_dir: Option<&str>) -> Result<PathBuf, AppError> {
        let path = if let Some(workspace) = workspace_dir {
            // 有工作区：使用 Base64 编码的工作区路径作为目录名
            let workspace_hash = Self::encode_workspace_path(workspace);
            self.storage_root.join("workspace-sessions").join(workspace_hash)
        } else {
            // 无工作区：使用全局 sessions 目录
            self.storage_root.join("sessions")
        };

        // 确保目录存在
        fs::create_dir_all(&path)
            .map_err(|e| AppError::ConfigError(format!("创建会话目录失败: {}", e)))?;

        Ok(path)
    }

    /// 编码工作区路径为安全的目录名
    /// 
    /// Base64(workspaceDir).replace(/[/+=]/g, "_")
    fn encode_workspace_path(path: &str) -> String {
        use base64::{Engine as _, engine::general_purpose};
        
        general_purpose::STANDARD
            .encode(path.as_bytes())
            .replace('/', "_")
            .replace('+', "_")
            .replace('=', "_")
    }

    /// 获取会话文件路径
    fn get_session_file_path(&self, session_id: &str, workspace_dir: Option<&str>) -> Result<PathBuf, AppError> {
        let folder = self.get_sessions_folder_path(workspace_dir)?;
        Ok(folder.join(format!("{}.json", session_id)))
    }

    /// 获取会话列表文件路径
    fn get_sessions_list_path(&self, workspace_dir: Option<&str>) -> Result<PathBuf, AppError> {
        let folder = self.get_sessions_folder_path(workspace_dir)?;
        let filepath = folder.join("sessions.json");
        
        // 如果文件不存在，创建空数组
        if !filepath.exists() {
            fs::write(&filepath, "[]")
                .map_err(|e| AppError::ConfigError(format!("创建会话列表文件失败: {}", e)))?;
        }
        
        Ok(filepath)
    }

    /// 保存会话
    /// 
    /// 1. 保存完整会话到 {sessionId}.json
    /// 2. 更新 sessions.json 列表
    pub async fn save_session(&self, session: &Session) -> Result<(), AppError> {
        // 1. 保存完整会话
        let file_path = self.get_session_file_path(&session.session_id, session.workspace_directory.as_deref())?;
        let content = serde_json::to_string_pretty(session)
            .map_err(|e| AppError::ConfigError(format!("序列化会话失败: {}", e)))?;
        
        fs::write(&file_path, content)
            .map_err(|e| AppError::ConfigError(format!("保存会话文件失败: {}", e)))?;

        // 2. 更新会话列表
        let sessions_list_path = self.get_sessions_list_path(session.workspace_directory.as_deref())?;
        
        // 读取现有列表
        let raw_sessions_list = fs::read_to_string(&sessions_list_path)
            .map_err(|e| AppError::ConfigError(format!("读取会话列表失败: {}", e)))?;
        
        let mut sessions_list: Vec<SessionInfo> = if raw_sessions_list.trim().is_empty() {
            // 空文件，初始化为空数组
            fs::write(&sessions_list_path, "[]")
                .map_err(|e| AppError::ConfigError(format!("初始化会话列表失败: {}", e)))?;
            Vec::new()
        } else {
            // 解析现有列表
            serde_json::from_str(&raw_sessions_list)
                .map_err(|e| AppError::ConfigError(format!("解析会话列表失败: {}. 请检查 sessions.json 文件格式", e)))?
        };

        // 查找是否已存在
        let mut found = false;
        for session_info in &mut sessions_list {
            if session_info.session_id == session.session_id {
                // 更新现有会话元数据
                session_info.title = session.title.clone();
                session_info.workspace_directory = session.workspace_directory.clone();
                session_info.hidden = session.hidden;
                found = true;
                break;
            }
        }

        // 如果不存在，添加新会话元数据
        if !found {
            let session_info = SessionInfo {
                session_id: session.session_id.clone(),
                title: session.title.clone(),
                date_created: chrono::Utc::now().timestamp_millis().to_string(),
                workspace_directory: session.workspace_directory.clone(),
                hidden: session.hidden,
            };
            sessions_list.push(session_info);
        }

        // 保存更新后的列表
        let list_content = serde_json::to_string_pretty(&sessions_list)
            .map_err(|e| AppError::ConfigError(format!("序列化会话列表失败: {}", e)))?;
        
        fs::write(&sessions_list_path, list_content)
            .map_err(|e| AppError::ConfigError(format!("保存会话列表失败: {}", e)))?;

        Ok(())
    }

    /// 加载会话
    /// 
    /// 如果会话不存在，返回默认会话结构
    pub async fn load_session(&self, session_id: &str, workspace_dir: Option<&str>) -> Result<Session, AppError> {
        let session_file = self.get_session_file_path(session_id, workspace_dir)?;
        
        if !session_file.exists() {
            // 会话文件不存在，返回默认结构
            tracing::warn!("会话文件不存在: {:?}", session_file);
            return Ok(Session {
                session_id: session_id.to_string(),
                title: "New Session".to_string(),
                workspace_directory: workspace_dir.map(|s| s.to_string()),
                history: Vec::new(),
                hidden: false,
            });
        }

        // 读取并解析会话文件
        let content = fs::read_to_string(&session_file)
            .map_err(|e| AppError::ConfigError(format!("读取会话文件失败: {}", e)))?;
        
        let mut session: Session = serde_json::from_str(&content)
            .map_err(|e| {
                tracing::error!("解析会话文件失败: {}", e);
                // 解析失败，返回默认结构
                AppError::ConfigError(format!("解析会话文件失败: {}", e))
            })?;

        // 确保 sessionId 字段存在
        session.session_id = session_id.to_string();

        Ok(session)
    }

    /// 删除会话
    /// 
    /// 1. 删除 {sessionId}.json 文件
    /// 2. 从 sessions.json 列表中移除
    pub async fn delete_session(&self, session_id: &str, workspace_dir: Option<&str>) -> Result<(), AppError> {
        // 1. 删除会话文件
        let session_file = self.get_session_file_path(session_id, workspace_dir)?;
        
        if !session_file.exists() {
            return Err(AppError::NotFound(format!("会话文件不存在: {}", session_id)));
        }

        fs::remove_file(&session_file)
            .map_err(|e| AppError::ConfigError(format!("删除会话文件失败: {}", e)))?;

        // 2. 从会话列表中移除
        let sessions_list_path = self.get_sessions_list_path(workspace_dir)?;
        let raw_sessions_list = fs::read_to_string(&sessions_list_path)
            .map_err(|e| AppError::ConfigError(format!("读取会话列表失败: {}", e)))?;
        
        let mut sessions_list: Vec<SessionInfo> = serde_json::from_str(&raw_sessions_list)
            .map_err(|e| AppError::ConfigError(format!("解析会话列表失败: {}", e)))?;
        
        // 过滤掉要删除的会话
        sessions_list.retain(|s| s.session_id != session_id);

        // 保存更新后的列表
        let list_content = serde_json::to_string_pretty(&sessions_list)
            .map_err(|e| AppError::ConfigError(format!("序列化会话列表失败: {}", e)))?;
        
        fs::write(&sessions_list_path, list_content)
            .map_err(|e| AppError::ConfigError(format!("保存会话列表失败: {}", e)))?;

        Ok(())
    }

    /// 列出所有会话（只返回元数据）
    pub async fn list_sessions(&self, workspace_dir: Option<&str>) -> Result<Vec<SessionInfo>, AppError> {
        let sessions_list_path = self.get_sessions_list_path(workspace_dir)?;
        let raw_sessions_list = fs::read_to_string(&sessions_list_path)
            .map_err(|e| AppError::ConfigError(format!("读取会话列表失败: {}", e)))?;
        
        if raw_sessions_list.trim().is_empty() {
            return Ok(Vec::new());
        }

        let sessions_list: Vec<SessionInfo> = serde_json::from_str(&raw_sessions_list)
            .map_err(|e| AppError::ConfigError(format!("解析会话列表失败: {}", e)))?;
        
        Ok(sessions_list)
    }

    /// 搜索会话（按标题搜索）
    pub async fn search_sessions(&self, query: &str, workspace_dir: Option<&str>) -> Result<Vec<SessionInfo>, AppError> {
        let all_sessions = self.list_sessions(workspace_dir).await?;
        let query_lower = query.to_lowercase();
        
        let results: Vec<SessionInfo> = all_sessions
            .into_iter()
            .filter(|s| s.title.to_lowercase().contains(&query_lower))
            .collect();
        
        Ok(results)
    }
}
