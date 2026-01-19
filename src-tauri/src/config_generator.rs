// Claude 配置生成器
// 支持生成 Claude Desktop、Claude CLI、OpenAI 兼容配置

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct ClaudeDesktopConfig {
    #[serde(rename = "apiBaseUrl")]
    pub api_base_url: String,
    #[serde(rename = "apiKey")]
    pub api_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub openai_api_base: String,
    pub openai_api_key: String,
}

/// 生成 Claude Desktop 配置
pub fn generate_claude_desktop_config(base_url: &str, api_key: &str) -> Result<String, String> {
    let config = ClaudeDesktopConfig {
        api_base_url: base_url.to_string(),
        api_key: api_key.to_string(),
    };
    
    serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))
}

/// 生成 Claude CLI 配置脚本
pub fn generate_claude_cli_config(base_url: &str, api_key: &str) -> String {
    format!(
        r#"# Claude CLI 配置
# 复制以下命令到终端执行

# Windows (PowerShell)
$env:ANTHROPIC_BASE_URL="{}"
$env:ANTHROPIC_API_KEY="{}"

# Linux/macOS (Bash/Zsh)
export ANTHROPIC_BASE_URL="{}"
export ANTHROPIC_API_KEY="{}"

# 或者使用 claude config 命令
claude config set --global apiBaseUrl "{}"
claude config set --global apiKey "{}"
"#,
        base_url, api_key, base_url, api_key, base_url, api_key
    )
}

/// 生成 OpenAI 兼容配置
pub fn generate_openai_config(base_url: &str, api_key: &str) -> String {
    format!(
        r#"# OpenAI 兼容配置
# 适用于支持 OpenAI API 的工具（如 Continue、Cursor 等）

# .env 文件格式
OPENAI_API_BASE={}
OPENAI_API_KEY={}

# 或环境变量格式
# Windows (PowerShell)
$env:OPENAI_API_BASE="{}"
$env:OPENAI_API_KEY="{}"

# Linux/macOS (Bash/Zsh)
export OPENAI_API_BASE="{}"
export OPENAI_API_KEY="{}"
"#,
        base_url, api_key, base_url, api_key, base_url, api_key
    )
}

/// 获取 Claude Desktop 配置文件路径
pub fn get_claude_desktop_config_path() -> Option<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()?;
    
    #[cfg(target_os = "windows")]
    {
        Some(PathBuf::from(home).join("AppData").join("Roaming").join("Claude").join("config.json"))
    }
    
    #[cfg(target_os = "macos")]
    {
        Some(PathBuf::from(home).join("Library").join("Application Support").join("Claude").join("config.json"))
    }
    
    #[cfg(target_os = "linux")]
    {
        Some(PathBuf::from(home).join(".config").join("claude").join("config.json"))
    }
}

/// 写入 Claude Desktop 配置文件
pub async fn write_claude_desktop_config(base_url: &str, api_key: &str) -> Result<String, String> {
    let config_path = get_claude_desktop_config_path()
        .ok_or_else(|| "无法确定配置文件路径".to_string())?;
    
    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        tokio::fs::create_dir_all(parent).await
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    
    // 生成配置内容
    let config_content = generate_claude_desktop_config(base_url, api_key)?;
    
    // 写入文件
    tokio::fs::write(&config_path, config_content).await
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    
    info!("Claude Desktop 配置已写入: {:?}", config_path);
    Ok(config_path.to_string_lossy().to_string())
}

/// 生成完整的配置包
#[derive(Debug, Serialize)]
pub struct ConfigPackage {
    pub claude_desktop_json: String,
    pub claude_desktop_path: Option<String>,
    pub claude_cli_script: String,
    pub openai_config: String,
}

pub fn generate_config_package(base_url: &str, api_key: &str) -> Result<ConfigPackage, String> {
    Ok(ConfigPackage {
        claude_desktop_json: generate_claude_desktop_config(base_url, api_key)?,
        claude_desktop_path: get_claude_desktop_config_path().map(|p| p.to_string_lossy().to_string()),
        claude_cli_script: generate_claude_cli_config(base_url, api_key),
        openai_config: generate_openai_config(base_url, api_key),
    })
}
