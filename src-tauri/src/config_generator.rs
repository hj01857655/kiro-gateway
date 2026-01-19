// Claude 配置生成器
// 支持生成 Claude CLI、OpenAI 兼容配置

use serde::Serialize;

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

/// 生成完整的配置包
#[derive(Debug, Serialize)]
pub struct ConfigPackage {
    pub claude_cli_script: String,
    pub openai_config: String,
}

pub fn generate_config_package(base_url: &str, api_key: &str) -> Result<ConfigPackage, String> {
    Ok(ConfigPackage {
        claude_cli_script: generate_claude_cli_config(base_url, api_key),
        openai_config: generate_openai_config(base_url, api_key),
    })
}
