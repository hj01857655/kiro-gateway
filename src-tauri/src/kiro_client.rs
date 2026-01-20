use bytes::Bytes;
use futures::stream::{Stream, StreamExt};
use reqwest::Client;
use std::pin::Pin;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::{sleep, timeout, Instant};
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, error, info, warn};

use crate::account::Account;
use crate::account::AccountManager;
use crate::config::AppConfig;
use crate::error::AppError;
use crate::models::{KiroEvent, KiroPayload};

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
        mut request: KiroPayload,
        accounts: &AccountManager,
        model: &str,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<KiroEvent, AppError>> + Send>>, AppError> {
        let account = accounts.get_account().await?;
        let first_token_timeout = get_first_token_timeout(model);
        let stream_timeout = get_stream_timeout(model);

        match self
            .generate_with_timeout(
                request.clone(),
                &account,
                accounts,
                first_token_timeout,
                stream_timeout,
            )
            .await
        {
            Ok(stream) => Ok(stream),
            Err(AppError::TokenExpired) => {
                info!("Token 过期，刷新后重试...");
                let refreshed = accounts.refresh_account(&account.id).await?;
                self.generate_with_timeout(
                    request,
                    &refreshed,
                    accounts,
                    first_token_timeout,
                    stream_timeout,
                )
                .await
            }
            Err(AppError::RateLimited) => {
                // 限流时标记账号
                accounts.mark_throttled(&account.id);
                Err(AppError::RateLimited)
            }
            Err(AppError::BadRequest(ref msg)) if msg.contains("CONTENT_LENGTH_EXCEEDS_THRESHOLD") => {
                // 内容长度超限，自动截断历史消息并重试
                warn!("内容长度超限，截断历史消息后重试...");
                
                // 截断历史消息（只保留最后一对对话）
                if let Some(ref mut history) = request.conversation_state.history {
                    use crate::converter::trim_message_history;
                    use crate::models::ChatMessage;
                    
                    // 将 HistoryItem 转换为 ChatMessage 进行截断
                    let mut messages: Vec<ChatMessage> = Vec::new();
                    for item in history.iter() {
                        match item {
                            crate::models::HistoryItem::User { user_input_message } => {
                                messages.push(ChatMessage {
                                    role: "user".to_string(),
                                    content: Some(serde_json::Value::String(user_input_message.content.clone())),
                                    tool_calls: None,
                                    tool_call_id: None,
                                });
                            }
                            crate::models::HistoryItem::Assistant { assistant_response_message } => {
                                messages.push(ChatMessage {
                                    role: "assistant".to_string(),
                                    content: Some(serde_json::Value::String(assistant_response_message.content.clone())),
                                    tool_calls: None,
                                    tool_call_id: None,
                                });
                            }
                        }
                    }
                    
                    // 截断消息
                    let trimmed = trim_message_history(&messages);
                    
                    // 转换回 HistoryItem
                    let mut new_history = Vec::new();
                    for msg in trimmed {
                        match msg.role.as_str() {
                            "user" => {
                                let content = match msg.content {
                                    Some(serde_json::Value::String(s)) => s,
                                    _ => "continue".to_string(),
                                };
                                new_history.push(crate::models::HistoryItem::User {
                                    user_input_message: crate::models::HistoryUserMessage {
                                        content,
                                        model_id: request.conversation_state.current_message.user_input_message.model_id.clone(),
                                        user_intent: Some("CODE_GENERATION".to_string()),
                                        origin: "AI_EDITOR".to_string(),
                                        images: None,
                                        user_input_message_context: None,
                                        inference_config: None,
                                    },
                                });
                            }
                            "assistant" => {
                                let content = match msg.content {
                                    Some(serde_json::Value::String(s)) => s,
                                    _ => "understood".to_string(),
                                };
                                new_history.push(crate::models::HistoryItem::Assistant {
                                    assistant_response_message: crate::models::HistoryAssistantMessage {
                                        content,
                                        tool_uses: None,
                                    },
                                });
                            }
                            _ => {}
                        }
                    }
                    
                    *history = new_history;
                    info!("历史消息已截断，重试请求...");
                    
                    // 重试请求
                    return self.generate_with_timeout(
                        request,
                        &account,
                        accounts,
                        first_token_timeout,
                        stream_timeout,
                    ).await;
                } else {
                    // 没有历史消息，无法截断
                    warn!("没有历史消息可截断，返回错误");
                    Err(AppError::BadRequest(msg.clone()))
                }
            }
            Err(e) => Err(e),
        }
    }

    async fn generate_with_timeout(
        &self,
        request: KiroPayload,
        account: &Account,
        accounts: &AccountManager,
        first_token_timeout: Duration,
        stream_timeout: Duration,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<KiroEvent, AppError>> + Send>>, AppError> {
        // 首 Token 超时控制
        match timeout(
            first_token_timeout,
            self.generate_assistant_response(request, account, accounts),
        )
        .await
        {
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
        request: KiroPayload,
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
                        AppError::TokenExpired => (false, 0),
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
        request: &KiroPayload,
        account: &Account,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<KiroEvent, AppError>> + Send>>, AppError> {
        let url = format!("{}/generateAssistantResponse", self.config.kiro_endpoint);
        let invocation_id = uuid::Uuid::new_v4().to_string();

        // 调试：打印完整请求体（生产环境应禁用）
        #[cfg(debug_assertions)]
        {
            debug!(
                "调用 Kiro API，conversationId: {}",
                request.conversation_state.conversation_id
            );
            // 不打印完整请求体，避免泄露敏感信息
            debug!("请求体大小: {} bytes", serde_json::to_string(&request).unwrap_or_default().len());
        }

        debug!("调用 Kiro API: {}", url);

        let response = self
            .client
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

            // 生产环境不记录完整错误信息
            #[cfg(debug_assertions)]
            {
                error!("Kiro API 错误: {} - {}", status, error_text);
            }
            #[cfg(not(debug_assertions))]
            {
                error!("Kiro API 错误: {}", status);
            }

            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&error_text) {
                let error_type = error_json
                    .get("__type")
                    .or_else(|| error_json.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                // 检查 reason 字段（CONTENT_LENGTH_EXCEEDS_THRESHOLD）
                if let Some(reason) = error_json.get("reason").and_then(|v| v.as_str()) {
                    if reason == "CONTENT_LENGTH_EXCEEDS_THRESHOLD" {
                        return Err(AppError::BadRequest(format!(
                            "Input is too long: {}",
                            error_text
                        )));
                    }
                }

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

            // 生产环境返回通用错误消息
            #[cfg(debug_assertions)]
            {
                return Err(AppError::KiroApiError(format!(
                    "{}: {}",
                    status, error_text
                )));
            }
            #[cfg(not(debug_assertions))]
            {
                return Err(AppError::KiroApiError(format!(
                    "API 请求失败: {}",
                    status
                )));
            }
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
                while let Some(pos) = buffer[search_start..].find('{') {
                    let json_start = search_start + pos;

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

                    // 调试：记录收到事件（不打印完整内容）
                    debug!("📥 收到 JSON 事件 #{}", event_count + 1);

                    // 尝试解析 JSON
                    match serde_json::from_str::<KiroEvent>(json_str) {
                        Ok(event) => {
                            event_count += 1;

                            // 记录事件类型（不打印内容）
                            debug!(
                                "✅ 解析事件 #{}: tool_use_id={:?}, language={:?}, usage={:?}",
                                event_count, event.tool_use_id, event.language, event.usage
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
                            debug!(
                                "解析 JSON 失败: {} - {}",
                                e,
                                &json_str[..json_str.len().min(200)]
                            );
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
        // 配额查询使用固定的 app.kiro.dev endpoint，使用 CBOR 格式
        let url =
            "https://app.kiro.dev/service/KiroWebPortalService/operation/GetUserUsageAndLimits";

        // 请求体（CBOR 格式）
        let body = serde_json::json!({
            "isEmailRequired": true,
            "origin": "KIRO_IDE"
        });

        // 序列化为 CBOR
        let cbor_body = serde_cbor::to_vec(&body)
            .map_err(|e| AppError::ParseError(format!("CBOR 序列化失败: {}", e)))?;

        let resp = self
            .client
            .post(url)
            .header("Authorization", format!("Bearer {}", account.access_token))
            .header("Content-Type", "application/cbor")
            .header("Accept", "application/cbor")
            .header("smithy-protocol", "rpc-v2-cbor")
            .header("x-amz-user-agent", self.get_user_agent())
            .header("amz-sdk-invocation-id", uuid::Uuid::new_v4().to_string())
            .header("amz-sdk-request", "attempt=1; max=1")
            .header(
                "Cookie",
                format!("Idp=BuilderId; AccessToken={}", account.access_token),
            )
            .body(cbor_body)
            .send()
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        let status = resp.status().as_u16();

        if resp.status().is_success() {
            // 解析 CBOR 响应
            let bytes = resp
                .bytes()
                .await
                .map_err(|e| AppError::NetworkError(e.to_string()))?;
            let value: serde_json::Value = serde_cbor::from_slice(&bytes)
                .map_err(|e| AppError::ParseError(format!("CBOR 解析失败: {}", e)))?;
            return Ok(value);
        }

        // 尝试解析 CBOR 错误响应
        let bytes = resp.bytes().await.unwrap_or_default();
        let text = if let Ok(value) = serde_cbor::from_slice::<serde_json::Value>(&bytes) {
            serde_json::to_string(&value)
                .unwrap_or_else(|_| String::from_utf8_lossy(&bytes).to_string())
        } else {
            String::from_utf8_lossy(&bytes).to_string()
        };
        let msg_lower = text.to_lowercase();

        // 记录错误响应用于调试
        warn!("配额查询失败 ({}): {}", status, text);

        match status {
            // 401 → token 过期
            401 => Err(AppError::TokenExpired),

            // 403/423 → 检查是 token 无效还是封禁
            403 | 423 => {
                // 尝试解析 JSON 响应
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                    // 检查是否是 Token 相关错误
                    if let Some(error_type) = json.get("__type").and_then(|v| v.as_str()) {
                        if error_type.contains("UnauthorizedException")
                            || error_type.contains("InvalidToken")
                            || error_type.contains("ExpiredToken")
                        {
                            return Err(AppError::TokenExpired);
                        }
                        // 检查是否是账号封禁
                        if error_type.contains("AccountSuspendedException")
                            || error_type.contains("Suspended")
                        {
                            return Err(AppError::AccountBanned(text));
                        }
                    }

                    // 检查 reason 字段
                    if let Some(reason) = json.get("reason").and_then(|v| v.as_str()) {
                        if reason == "TEMPORARILY_SUSPENDED" {
                            return Err(AppError::AccountBanned(text));
                        }
                    }
                }

                // 回退到文本匹配
                if msg_lower.contains("invalid")
                    || msg_lower.contains("expired")
                    || msg_lower.contains("unauthorized")
                {
                    return Err(AppError::TokenExpired);
                }

                if msg_lower.contains("suspend")
                    || msg_lower.contains("banned")
                    || msg_lower.contains("locked")
                {
                    return Err(AppError::AccountBanned(text));
                }

                // 默认当作 Token 过期处理（更安全）
                Err(AppError::TokenExpired)
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

        let result = self
            .client
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
            "origin": "KIRO_GATEWAY",
            "profileArn": &account.profile_arn
        });

        let resp = self
            .client
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

        resp.json()
            .await
            .map_err(|e| AppError::ParseError(e.to_string()))
    }

    /// 创建用户记忆
    pub async fn create_user_memory(
        &self,
        account: &Account,
        content: &str,
    ) -> Result<serde_json::Value, AppError> {
        let url = format!("{}/CreateUserMemoryEntry", self.config.kiro_endpoint);

        let body = serde_json::json!({
            "memoryEntryString": content,
            "origin": "KIRO_GATEWAY",
            "profileArn": &account.profile_arn
        });

        let resp = self
            .client
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

        resp.json()
            .await
            .map_err(|e| AppError::ParseError(e.to_string()))
    }

    /// 删除用户记忆
    pub async fn delete_user_memory(
        &self,
        account: &Account,
        entry_id: &str,
    ) -> Result<(), AppError> {
        let url = format!(
            "{}/DeleteUserMemoryEntry/{}",
            self.config.kiro_endpoint, entry_id
        );

        let resp = self
            .client
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

    /// 获取可用模型列表
    /// 调用 Kiro API 的 ListAvailableModels 接口
    pub async fn list_available_models(
        &self,
        account: &Account,
    ) -> Result<Vec<serde_json::Value>, AppError> {
        let url = format!("{}/ListAvailableModels", self.config.kiro_endpoint);

        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", account.access_token))
            .header("x-amz-user-agent", self.get_user_agent())
            .query(&[
                ("origin", "AI_EDITOR"),
                ("profileArn", &account.profile_arn),
            ])
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| AppError::NetworkError(e.to_string()))?;

        let status = resp.status();

        if status == 401 || status == 403 {
            return Err(AppError::TokenExpired);
        }

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(AppError::KiroApiError(format!("{}: {}", status, text)));
        }

        let data: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::ParseError(e.to_string()))?;

        let models = data
            .get("models")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        Ok(models)
    }
}
