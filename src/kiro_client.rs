use bytes::Bytes;
use futures::stream::{Stream, StreamExt};
use reqwest::Client;
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout, Instant};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, warn, info};

use crate::account::AccountManager;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::{KiroEvent, KiroRequest};
use crate::account::Account;

const THROTTLING_ERRORS: &[&str] = &[
    "ThrottlingException",
    "TooManyRequestsException",
    "RequestThrottledException",
    "LimitExceededException",
    "SlowDown",
];

const MAX_RETRIES: u32 = 3;
const THROTTLE_BASE_DELAY_MS: u64 = 500;
const NORMAL_BASE_DELAY_MS: u64 = 100;
const MAX_DELAY_MS: u64 = 20000;

pub fn get_first_token_timeout(model: &str) -> Duration {
    if model.contains("haiku") {
        Duration::from_secs(30)
    } else if model.contains("opus") {
        Duration::from_secs(120)
    } else {
        Duration::from_secs(60)
    }
}

pub fn get_stream_timeout(model: &str) -> Duration {
    if model.contains("haiku") {
        Duration::from_secs(60)
    } else if model.contains("opus") {
        Duration::from_secs(300)
    } else {
        Duration::from_secs(120)
    }
}

pub struct KiroClient {
    pub client: Client,
    config: AppConfig,
}

impl KiroClient {
    pub fn new(config: AppConfig) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(300))
                .build()
                .unwrap(),
            config,
        }
    }

    fn get_user_agent(&self) -> String {
        let version = env!("CARGO_PKG_VERSION");
        let machine_id = self.config.machine_id.as_deref().unwrap_or("unknown");
        format!("KiroIDE-{}-{}", version, machine_id)
    }

    /// 带 Token 刷新和超时控制的请求
    pub async fn generate_with_refresh(
        &self,
        request: KiroRequest,
        accounts: &AccountManager,
        model: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<KiroEvent, AppError>> + Send>>, AppError> {
        let account = accounts.get_account().await?;
        let first_token_timeout = get_first_token_timeout(model);
        let stream_timeout = get_stream_timeout(model);
        
        match self.generate_with_timeout(request.clone(), &account, accounts, first_token_timeout, stream_timeout).await {
            Ok(stream) => Ok(stream),
            Err(AppError::TokenExpired) => {
                info!("Token 过期，刷新后重试...");
                let refreshed = accounts.refresh_account(&account.id).await?;
                self.generate_with_timeout(request, &refreshed, accounts, first_token_timeout, stream_timeout).await
            }
            Err(AppError::RateLimited) => {
                // 限流时标记账号
                accounts.mark_throttled(&account.id);
                Err(AppError::RateLimited)
            }
            Err(e) => Err(e),
        }
    }

    async fn generate_with_timeout(
        &self,
        request: KiroRequest,
        account: &Account,
        accounts: &AccountManager,
        first_token_timeout: Duration,
        stream_timeout: Duration,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<KiroEvent, AppError>> + Send>>, AppError> {
        // 首 Token 超时控制
        match timeout(first_token_timeout, self.generate_assistant_response(request, account, accounts)).await {
            Ok(result) => {
                // 包装流，添加流读取超时
                result.map(|stream| wrap_stream_with_timeout(stream, stream_timeout))
            }
            Err(_) => {
                warn!("首 Token 超时 ({:?})", first_token_timeout);
                Err(AppError::NetworkError("首 Token 超时".to_string()))
            }
        }
    }

    async fn generate_assistant_response(
        &self,
        request: KiroRequest,
        account: &Account,
        accounts: &AccountManager,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<KiroEvent, AppError>> + Send>>, AppError> {
        let mut last_error = AppError::NetworkError("未知错误".to_string());
        
        for attempt in 0..MAX_RETRIES {
            match self.do_request(&request, account).await {
                Ok(stream) => return Ok(stream),
                Err(e) => {
                    last_error = e.clone();
                    
                    let (should_retry, delay) = match &e {
                        AppError::RateLimited => {
                            // 限流时标记账号
                            accounts.mark_throttled(&account.id);
                            let delay = calculate_backoff(attempt, THROTTLE_BASE_DELAY_MS);
                            warn!("限流，{}ms 后重试 (attempt {})", delay, attempt + 1);
                            (true, delay)
                        }
                        AppError::TokenExpired => {
                            (false, 0)
                        }
                        AppError::NetworkError(_) => {
                            let delay = calculate_backoff(attempt, NORMAL_BASE_DELAY_MS);
                            warn!("网络错误，{}ms 后重试 (attempt {})", delay, attempt + 1);
                            (true, delay)
                        }
                        _ => (false, 0),
                    };
                    
                    if !should_retry || attempt >= MAX_RETRIES - 1 {
                        break;
                    }
                    
                    sleep(Duration::from_millis(delay)).await;
                }
            }
        }
        
        Err(last_error)
    }

    async fn do_request(
        &self,
        request: &KiroRequest,
        account: &Account,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<KiroEvent, AppError>> + Send>>, AppError> {
        let url = format!("{}/generateAssistantResponse", self.config.kiro_endpoint);
        let invocation_id = uuid::Uuid::new_v4().to_string();

        // 调试：打印请求体
        let request_json = serde_json::to_string_pretty(request).unwrap_or_default();
        info!("Kiro 请求体:\n{}", request_json);

        debug!("调用 Kiro API: {}", url);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", account.access_token))
            .header("Content-Type", "application/json")
            .header("x-amzn-kiro-agent-mode", "vibe")
            .header("x-amz-user-agent", self.get_user_agent())
            .header("amz-sdk-invocation-id", &invocation_id)
            .header("amz-sdk-request", "attempt=1; max=3")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        let status = response.status();

        if !status.is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Kiro API 错误: {} - {}", status, error_text);

            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                let error_type = error_json.get("__type")
                    .or_else(|| error_json.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if error_type == "ExpiredTokenException" {
                    return Err(AppError::TokenExpired);
                }
                if THROTTLING_ERRORS.contains(&error_type) {
                    return Err(AppError::RateLimited);
                }
                if error_type == "ServiceQuotaExceededException" {
                    return Err(AppError::QuotaExceeded);
                }
            }

            if status.as_u16() == 401 {
                return Err(AppError::TokenExpired);
            }
            if status.as_u16() == 429 {
                return Err(AppError::RateLimited);
            }

            return Err(AppError::KiroApiError(format!("{}: {}", status, error_text)));
        }

        let (tx, rx) = mpsc::channel::<Result<KiroEvent, AppError>>(100);
        let byte_stream = response.bytes_stream();

        tokio::spawn(async move {
            parse_event_stream(byte_stream, tx).await;
        });

        Ok(Box::pin(ReceiverStream::new(rx)))
    }
}

/// 包装流，添加流读取超时
fn wrap_stream_with_timeout(
    stream: Pin<Box<dyn Stream<Item = Result<KiroEvent, AppError>> + Send>>,
    stream_timeout: Duration,
) -> Pin<Box<dyn Stream<Item = Result<KiroEvent, AppError>> + Send>> {
    let wrapped = async_stream::stream! {
        tokio::pin!(stream);
        let mut last_event_time = Instant::now();
        
        loop {
            let remaining = stream_timeout.saturating_sub(last_event_time.elapsed());
            if remaining.is_zero() {
                warn!("流读取超时 ({:?})", stream_timeout);
                yield Err(AppError::NetworkError("流读取超时".to_string()));
                break;
            }
            
            match timeout(remaining, stream.next()).await {
                Ok(Some(event)) => {
                    last_event_time = Instant::now();
                    yield event;
                }
                Ok(None) => break, // 流结束
                Err(_) => {
                    warn!("流读取超时 ({:?})", stream_timeout);
                    yield Err(AppError::NetworkError("流读取超时".to_string()));
                    break;
                }
            }
        }
    };
    Box::pin(wrapped)
}

fn calculate_backoff(attempt: u32, base_ms: u64) -> u64 {
    let exp_delay = base_ms * 2u64.pow(attempt);
    let jitter = (rand_simple() * 0.3 * exp_delay as f64) as u64;
    (exp_delay + jitter).min(MAX_DELAY_MS)
}

fn rand_simple() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    (nanos % 1000) as f64 / 1000.0
}

async fn parse_event_stream<S>(mut stream: S, tx: mpsc::Sender<Result<KiroEvent, AppError>>)
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    let mut buffer = String::new();
    let mut event_count = 0;

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => {
                let chunk_str = String::from_utf8_lossy(&bytes);
                debug!("收到数据块 ({} bytes)", bytes.len());
                buffer.push_str(&chunk_str);

                // 解析 AWS Event Stream 格式的 JSON payload
                // Kiro 返回格式: {"content":"..."} 或 {"name":"xxx","toolUseId":"xxx",...}
                let mut search_start = 0;
                loop {
                    // 查找 JSON 对象的开始位置
                    let json_start = match buffer[search_start..].find('{') {
                        Some(pos) => search_start + pos,
                        None => break,
                    };

                    // 使用括号计数法找到完整的 JSON 对象
                    let mut brace_count = 0;
                    let mut in_string = false;
                    let mut escape_next = false;
                    let mut json_end = None;

                    for (i, ch) in buffer[json_start..].char_indices() {
                        if escape_next {
                            escape_next = false;
                            continue;
                        }

                        match ch {
                            '\\' if in_string => escape_next = true,
                            '"' => in_string = !in_string,
                            '{' if !in_string => brace_count += 1,
                            '}' if !in_string => {
                                brace_count -= 1;
                                if brace_count == 0 {
                                    json_end = Some(json_start + i + 1);
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }

                    let json_end = match json_end {
                        Some(end) => end,
                        None => {
                            // 不完整的 JSON，保留在缓冲区等待更多数据
                            buffer = buffer[json_start..].to_string();
                            break;
                        }
                    };

                    let json_str = &buffer[json_start..json_end];
                    
                    // 调试：打印原始 JSON（安全截取，避免 UTF-8 边界问题）
                    let preview = if json_str.len() > 500 {
                        json_str.chars().take(500).collect::<String>()
                    } else {
                        json_str.to_string()
                    };
                    info!("📥 收到 JSON #{}: {}", event_count + 1, preview);
                    
                    // 尝试解析 JSON
                    match serde_json::from_str::<KiroEvent>(json_str) {
                        Ok(event) => {
                            event_count += 1;
                            
                            // 安全截取字符串用于日志
                            let content_preview = event.content.as_ref().map(|s| {
                                if s.len() > 50 {
                                    s.chars().take(50).collect::<String>()
                                } else {
                                    s.clone()
                                }
                            });
                            let text_preview = event.text.as_ref().map(|s| {
                                if s.len() > 50 {
                                    s.chars().take(50).collect::<String>()
                                } else {
                                    s.clone()
                                }
                            });
                            
                            info!("✅ 解析事件 #{}: content={:?}, text={:?}, tool_use_id={:?}, language={:?}, usage={:?}", 
                                event_count,
                                content_preview,
                                text_preview,
                                event.tool_use_id,
                                event.language,
                                event.usage
                            );
                            
                            if let Some(ref reason) = event.reason {
                                let msg = event.message.clone().unwrap_or_else(|| reason.clone());
                                let _ = tx.send(Err(AppError::KiroApiError(msg))).await;
                                return;
                            }

                            if tx.send(Ok(event)).await.is_err() {
                                return;
                            }
                        }
                        Err(e) => {
                            debug!("解析 JSON 失败: {} - {}", e, &json_str[..json_str.len().min(200)]);
                        }
                    }

                    search_start = json_end;
                    if search_start >= buffer.len() {
                        buffer.clear();
                        break;
                    }
                }

                // 如果 search_start 有进展，截取剩余部分（安全处理 UTF-8 边界）
                if search_start > 0 && !buffer.is_empty() {
                    // 找到 search_start 之后的第一个字符边界
                    if search_start < buffer.len() {
                        let mut idx = search_start;
                        while idx < buffer.len() && !buffer.is_char_boundary(idx) {
                            idx += 1;
                        }
                        if idx < buffer.len() {
                            buffer = buffer[idx..].to_string();
                        } else {
                            buffer.clear();
                        }
                    } else {
                        buffer.clear();
                    }
                }
            }
            Err(e) => {
                warn!("读取流错误: {}", e);
                let _ = tx.send(Err(AppError::NetworkError(e.to_string()))).await;
                return;
            }
        }
    }
    
    info!("流结束，共处理 {} 个事件", event_count);
}

// ============ 高级功能 API ============

impl KiroClient {
    /// 获取配额使用情况
    /// 返回: Ok(json) 正常, Err(TokenExpired) token过期, Err(AccountBanned) 封禁
    pub async fn get_usage_limits(&self, account: &Account) -> Result<serde_json::Value, AppError> {
        let url = format!(
            "{}/getUsageLimits?isEmailRequired=true&origin=AI_EDITOR&resourceType=AGENTIC_REQUEST",
            self.config.kiro_endpoint
        );
        
        let resp = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", account.access_token))
            .header("x-amz-user-agent", self.get_user_agent())
            .header("amz-sdk-invocation-id", uuid::Uuid::new_v4().to_string())
            .header("amz-sdk-request", "attempt=1; max=1")
            .send()
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        let status = resp.status().as_u16();
        
        if resp.status().is_success() {
            return resp.json().await.map_err(|e| AppError::ParseError(e.to_string()));
        }

        let text = resp.text().await.unwrap_or_default();
        let msg_lower = text.to_lowercase();
        
        match status {
            // 401 → token 过期
            401 => Err(AppError::TokenExpired),
            
            // 403/423 → 检查是 token 无效还是封禁
            403 | 423 => {
                // token 无效（不是封禁）
                if msg_lower.contains("invalid") || msg_lower.contains("expired") {
                    return Err(AppError::TokenExpired);
                }
                // 403/423 其他情况都是封禁
                Err(AppError::AccountBanned(text))
            }
            
            _ => Err(AppError::KiroApiError(format!("{}: {}", status, text))),
        }
    }

    /// 健康检查（使用 dryRun）
    pub async fn health_check(&self, account: &Account) -> bool {
        let url = format!("{}/SendMessageStreaming", self.config.kiro_endpoint);
        
        let body = serde_json::json!({
            "conversationState": {
                "conversationId": uuid::Uuid::new_v4().to_string(),
                "currentMessage": {
                    "userInputMessage": {
                        "content": ["ping"],
                        "userIntent": "CODE_GENERATION"
                    }
                }
            },
            "profileArn": &account.profile_arn,
            "dryRun": true,
            "source": "AGENT"
        });

        let result = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", account.access_token))
            .header("Content-Type", "application/json")
            .header("x-amz-user-agent", self.get_user_agent())
            .json(&body)
            .send()
            .await;

        match result {
            Ok(resp) => resp.status().is_success(),
            Err(_) => false,
        }
    }

    /// 列出用户记忆
    pub async fn list_user_memory(&self, account: &Account) -> Result<serde_json::Value, AppError> {
        let url = format!("{}/ListUserMemoryEntries", self.config.kiro_endpoint);
        
        let body = serde_json::json!({
            "origin": "KIROGATE",
            "profileArn": &account.profile_arn
        });

        let resp = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", account.access_token))
            .header("Content-Type", "application/json")
            .header("x-amz-user-agent", self.get_user_agent())
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::KiroApiError(format!("{}: {}", status, text)));
        }

        resp.json().await.map_err(|e| AppError::ParseError(e.to_string()))
    }

    /// 创建用户记忆
    pub async fn create_user_memory(&self, account: &Account, content: &str) -> Result<serde_json::Value, AppError> {
        let url = format!("{}/CreateUserMemoryEntry", self.config.kiro_endpoint);
        
        let body = serde_json::json!({
            "memoryEntryString": content,
            "origin": "KIROGATE",
            "profileArn": &account.profile_arn
        });

        let resp = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", account.access_token))
            .header("Content-Type", "application/json")
            .header("x-amz-user-agent", self.get_user_agent())
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::KiroApiError(format!("{}: {}", status, text)));
        }

        resp.json().await.map_err(|e| AppError::ParseError(e.to_string()))
    }

    /// 删除用户记忆
    pub async fn delete_user_memory(&self, account: &Account, entry_id: &str) -> Result<(), AppError> {
        let url = format!("{}/DeleteUserMemoryEntry/{}", self.config.kiro_endpoint, entry_id);
        
        let resp = self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", account.access_token))
            .header("x-amz-user-agent", self.get_user_agent())
            .send()
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::KiroApiError(format!("{}: {}", status, text)));
        }

        Ok(())
    }
}

