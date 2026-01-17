use axum::{
    Router,
    routing::{get, post, delete},
    extract::{State, Path},
    response::{sse::{Event, Sse}, Response, IntoResponse},
    http::HeaderMap,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use std::convert::Infallible;
use tokio_stream::StreamExt;
use tower_http::cors::CorsLayer;
use tracing::info;

mod account;
mod converter;
mod config;
mod error;
mod kiro_client;
mod models;
mod thinking_parser;
// mod websearch; // TODO: 需要适配独立服务架构
mod logger;
mod metrics;

use account::AccountManager;
use config::AppConfig;
use converter::{openai_to_kiro, anthropic_to_kiro, kiro_to_openai, kiro_to_anthropic, create_openai_end_with_reason, is_stream_request_openai, is_stream_request_anthropic};
use error::AppError;
use kiro_client::KiroClient;
use models::{OpenAIRequest, AnthropicRequest, Usage};

struct AppState {
    config: AppConfig,
    client: KiroClient,
    accounts: AccountManager,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG.*kiro_gateway=info".to_string()))
        .init();

    let config = AppConfig::from_env();
    let client = KiroClient::new(config.clone());
    let accounts = AccountManager::new();

    if let Some(ref json) = config.accounts_json {
        if let Err(e) = accounts.load_from_json(json) {
            tracing::error!("从环境变量加载账号失败: {}", e);
        }
    } else if let Some(ref file) = config.accounts_file {
        match std::fs::read_to_string(file) {
            Ok(content) => {
                if let Err(e) = accounts.load_from_json(&content) {
                    tracing::error!("从文件 {} 加载账号失败: {}", file, e);
                }
            }
            Err(e) => {
                tracing::error!("读取账号文件 {} 失败: {}", file, e);
            }
        }
        // 设置文件路径，用于后续更新 token
        accounts.set_accounts_file(file);
    }

    let state = Arc::new(AppState { config: config.clone(), client, accounts });

    let app = Router::new()
        // 核心 API
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/messages", post(messages))
        .route("/v1/models", get(list_models))
        .route("/health", get(health))
        // 管理 API
        .route("/admin/accounts", get(admin_list_accounts))
        .route("/admin/quota/:account_id", get(admin_get_quota))
        .route("/admin/health", get(admin_health_check))
        .route("/admin/stats", get(admin_stats))
        .route("/admin/logs", get(admin_logs))
        .route("/admin/logs/clear", post(admin_clear_logs))
        .route("/admin/metrics", get(admin_metrics))
        .route("/admin/accounts/import-kiro", post(admin_import_kiro))
        .route("/admin/accounts", post(admin_add_account))
        .route("/admin/accounts/:account_id", delete(admin_delete_account))
        .route("/admin/accounts/:account_id/refresh", post(admin_refresh_account))
        .route("/admin/accounts/:account_id/enable", post(admin_enable_account))
        .route("/admin/accounts/:account_id/disable", post(admin_disable_account))
        // 用户记忆 API
        .route("/v1/memory", get(list_memory).post(create_memory))
        .route("/v1/memory/:entry_id", delete(delete_memory))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let addr = format!("{}:{}", config.host, config.port);
    info!("KiroGate 启动: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn verify_api_key(headers: &HeaderMap, config: &AppConfig) -> Result<(), AppError> {
    if let Some(ref key) = config.api_key {
        let provided = headers.get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .or_else(|| headers.get("x-api-key").and_then(|v| v.to_str().ok()));
        if provided != Some(key.as_str()) {
            return Err(AppError::BadRequest("Invalid API key".into()));
        }
    }
    Ok(())
}

#[axum::debug_handler]
async fn chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<OpenAIRequest>,
) -> Result<Response, AppError> {
    verify_api_key(&headers, &state.config)?;

    // 日志记录

    let is_stream = is_stream_request_openai(&request);
    let model = request.model.as_deref().unwrap_or("auto");
    
    // 获取账号信息
    let account = state.accounts.get_account().await?;
    let kiro_request = openai_to_kiro(&request, &account.profile_arn, &account.auth_method);
    
    let stream = state.client.generate_with_refresh(kiro_request, &state.accounts, model).await?;
    let request_id = uuid::Uuid::new_v4().to_string();

    if is_stream {
        // 流式响应
        let openai_stream = async_stream::stream! {
            tokio::pin!(stream);
            let mut has_tool = false;
            let mut usage: Option<Usage> = None;
            let mut context_exceeded = false;

            while let Some(Ok(event)) = stream.next().await {
                // 检查是否有工具调用
                if event.tool_use_id.is_some() { 
                    has_tool = true; 
                }

                // 收集 usage（从 unit + usage 字段计算）
                if let Some(usage_val) = event.usage {
                    // Kiro 返回的是 credit 使用量，需要转换为 token
                    // 暂时使用估算：1 credit ≈ 1000 tokens
                    let total = (usage_val * 1000.0) as i32;
                    usage = Some(Usage {
                        prompt_tokens: total / 2,  // 粗略估算
                        completion_tokens: total / 2,
                        total_tokens: total,
                    });
                }

                // 检查错误
                if let Some(ref reason) = event.reason {
                    if reason == "CONTEXT_LENGTH_EXCEEDED" {
                        context_exceeded = true;
                    }
                }

                // 记录消息 ID
                if let Some(ref msg_id) = event.message_id {
                    tracing::debug!("消息 ID: {}", msg_id);
                }

                // 记录上下文使用率
                if let Some(pct) = event.context_usage_percentage {
                    tracing::debug!("上下文使用率: {:.1}%", pct * 100.0);
                }

                if let Some(chunk) = kiro_to_openai(&event, &request_id) {
                    let json = serde_json::to_string(&chunk).unwrap_or_default();
                    tracing::info!("发送 SSE 事件: {}", &json[..json.len().min(200)]);
                    yield Ok::<_, Infallible>(Event::default().data(json));
                } else {
                    tracing::debug!("事件未转换: {:?}", event);
                }
            }

            let end = create_openai_end_with_reason(&request_id, has_tool, context_exceeded, usage);
            yield Ok::<_, Infallible>(Event::default().data(serde_json::to_string(&end).unwrap_or_default()));
            yield Ok::<_, Infallible>(Event::default().data("[DONE]".to_string()));
        };
        Ok(Sse::new(openai_stream).into_response())
    } else {
        // 非流式响应：收集所有内容
        tokio::pin!(stream);
        let mut content_str = String::new();
        let mut tool_calls: Vec<serde_json::Value> = Vec::new();
        let mut usage: Option<Usage> = None;
        let mut context_exceeded = false;

        while let Some(Ok(event)) = stream.next().await {
            // 收集文本
            if let Some(ref content) = event.content {
                if event.language.is_some() {
                    // 代码块
                    let lang = event.language.as_deref().unwrap_or("");
                    content_str.push_str(&format!("\n```{}\n{}\n```\n", lang, content));
                } else {
                    // 普通文本
                    content_str.push_str(content);
                }
            }

            // 收集工具调用
            if let (Some(ref tool_use_id), Some(ref name), Some(ref input)) = 
                (&event.tool_use_id, &event.name, &event.input) {
                tool_calls.push(serde_json::json!({
                    "id": tool_use_id,
                    "type": "function",
                    "function": {
                        "name": name,
                        "arguments": serde_json::to_string(input).unwrap_or_default()
                    }
                }));
            }

            // 收集 usage
            if let Some(usage_val) = event.usage {
                let total = (usage_val * 1000.0) as i32;
                usage = Some(Usage {
                    prompt_tokens: total / 2,
                    completion_tokens: total / 2,
                    total_tokens: total,
                });
            }

            // 检查错误
            if let Some(ref reason) = event.reason {
                if reason == "CONTEXT_LENGTH_EXCEEDED" {
                    context_exceeded = true;
                }
            }
        }

        let finish_reason = if !tool_calls.is_empty() {
            "tool_calls"
        } else if context_exceeded {
            "length"
        } else {
            "stop"
        };

        let mut message = serde_json::json!({
            "role": "assistant",
            "content": if content_str.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(content_str) }
        });
        if !tool_calls.is_empty() {
            message["tool_calls"] = serde_json::Value::Array(tool_calls);
        }

        let response = serde_json::json!({
            "id": format!("chatcmpl-{}", request_id),
            "object": "chat.completion",
            "created": chrono::Utc::now().timestamp(),
            "model": model,
            "choices": [{
                "index": 0,
                "message": message,
                "finish_reason": finish_reason
            }],
            "usage": usage
        });

        Ok(Json(response).into_response())
    }
}

#[axum::debug_handler]
async fn messages(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<AnthropicRequest>,
) -> Result<Response, AppError> {
    verify_api_key(&headers, &state.config)?;

    let is_stream = is_stream_request_anthropic(&request);
    let model = request.model.as_deref().unwrap_or("auto");
    
    // 获取账号信息
    let account = state.accounts.get_account().await?;
    let kiro_request = anthropic_to_kiro(&request, &account.profile_arn, &account.auth_method);
    
    let stream = state.client.generate_with_refresh(kiro_request, &state.accounts, model).await?;
    let request_id = uuid::Uuid::new_v4().to_string();

    if is_stream {
        // 流式响应
        let anthropic_stream = async_stream::stream! {
            tokio::pin!(stream);
            
            // 创建 ThinkingParser
            let mut thinking_parser = thinking_parser::ThinkingParser::new();

            // message_start
            yield Ok::<_, Infallible>(Event::default().event("message_start").data(format!(
                r#"{{"type":"message_start","message":{{"id":"msg_{}","type":"message","role":"assistant","content":[],"model":"claude","stop_reason":null}}}}"#,
                request_id
            )));

            // content_block_start (text)
            yield Ok::<_, Infallible>(Event::default().event("content_block_start").data(
                r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#.to_string()
            ));

            let mut block_index = 0;
            let mut in_thinking = false;
            let mut has_tool = false;
            let mut output_tokens: Option<i32> = None;
            let mut context_exceeded = false;
            let mut thinking_signature: Option<String> = None;

            while let Some(Ok(event)) = stream.next().await {
                // 收集 usage
                if let Some(usage_val) = event.usage {
                    output_tokens = Some((usage_val * 1000.0) as i32);
                }

                // 检查错误
                if let Some(ref reason) = event.reason {
                    if reason == "CONTEXT_LENGTH_EXCEEDED" {
                        context_exceeded = true;
                    }
                }

                // 处理 thinking block (text + signature 字段)
                if let Some(ref _text) = event.text {
                    if event.signature.is_some() {
                        thinking_signature = event.signature.clone();
                    }
                    if !in_thinking {
                        block_index += 1;
                        let sig = thinking_signature.as_deref().unwrap_or("");
                        yield Ok::<_, Infallible>(Event::default().event("content_block_start").data(format!(
                            r#"{{"type":"content_block_start","index":{},"content_block":{{"type":"thinking","thinking":"","signature":"{}"}}}}"#,
                            block_index, sig
                        )));
                        in_thinking = true;
                    }
                    if let Some(data) = kiro_to_anthropic(&event) {
                        yield Ok::<_, Infallible>(Event::default().event("content_block_delta").data(data));
                    }
                }

                // 处理文本响应和代码块 (content 字段，但不是 thinking)
                if event.content.is_some() && event.text.is_none() {
                    if in_thinking {
                        yield Ok::<_, Infallible>(Event::default().event("content_block_stop").data(format!(
                            r#"{{"type":"content_block_stop","index":{}}}"#, block_index
                        )));
                        in_thinking = false;
                    }
                    
                    // 使用 ThinkingParser 解析内容
                    if let Some(ref content) = event.content {
                        let segments = thinking_parser.push_and_parse(content);
                        
                        for segment in segments {
                            match segment.segment_type {
                                thinking_parser::SegmentType::Thinking => {
                                    // 发送 thinking block
                                    if !in_thinking {
                                        block_index += 1;
                                        let sig = thinking_signature.as_deref().unwrap_or("");
                                        yield Ok::<_, Infallible>(Event::default().event("content_block_start").data(format!(
                                            r#"{{"type":"content_block_start","index":{},"content_block":{{"type":"thinking","thinking":"","signature":"{}"}}}}"#,
                                            block_index, sig
                                        )));
                                        in_thinking = true;
                                    }
                                    let escaped = segment.content.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
                                    yield Ok::<_, Infallible>(Event::default().event("content_block_delta").data(format!(
                                        r#"{{"type":"content_block_delta","index":{},"delta":{{"type":"thinking_delta","thinking":"{}"}}}}"#,
                                        block_index, escaped
                                    )));
                                }
                                thinking_parser::SegmentType::Text => {
                                    // 关闭 thinking block（如果有）
                                    if in_thinking {
                                        yield Ok::<_, Infallible>(Event::default().event("content_block_stop").data(format!(
                                            r#"{{"type":"content_block_stop","index":{}}}"#, block_index
                                        )));
                                        in_thinking = false;
                                    }
                                    // 发送 text delta
                                    let escaped = segment.content.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
                                    yield Ok::<_, Infallible>(Event::default().event("content_block_delta").data(format!(
                                        r#"{{"type":"content_block_delta","index":0,"delta":{{"type":"text_delta","text":"{}"}}}}"#,
                                        escaped
                                    )));
                                }
                            }
                        }
                    }
                }

                // 处理工具调用
                if let (Some(ref tool_use_id), Some(ref name), Some(ref input)) = 
                    (&event.tool_use_id, &event.name, &event.input) {
                    has_tool = true;
                    yield Ok::<_, Infallible>(Event::default().event("content_block_stop").data(
                        r#"{"type":"content_block_stop","index":0}"#.to_string()
                    ));
                    block_index += 1;
                    yield Ok::<_, Infallible>(Event::default().event("content_block_start").data(format!(
                        r#"{{"type":"content_block_start","index":{},"content_block":{{"type":"tool_use","id":"{}","name":"{}","input":{}}}}}"#,
                        block_index, tool_use_id, name, serde_json::to_string(input).unwrap_or_default()
                    )));
                    yield Ok::<_, Infallible>(Event::default().event("content_block_stop").data(format!(
                        r#"{{"type":"content_block_stop","index":{}}}"#, block_index
                    )));
                }
            }
            
            // 刷新 ThinkingParser 缓冲区
            let final_segments = thinking_parser.flush();
            for segment in final_segments {
                match segment.segment_type {
                    thinking_parser::SegmentType::Thinking => {
                        if !in_thinking {
                            block_index += 1;
                            let sig = thinking_signature.as_deref().unwrap_or("");
                            yield Ok::<_, Infallible>(Event::default().event("content_block_start").data(format!(
                                r#"{{"type":"content_block_start","index":{},"content_block":{{"type":"thinking","thinking":"","signature":"{}"}}}}"#,
                                block_index, sig
                            )));
                            in_thinking = true;
                        }
                        let escaped = segment.content.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
                        yield Ok::<_, Infallible>(Event::default().event("content_block_delta").data(format!(
                            r#"{{"type":"content_block_delta","index":{},"delta":{{"type":"thinking_delta","thinking":"{}"}}}}"#,
                            block_index, escaped
                        )));
                    }
                    thinking_parser::SegmentType::Text => {
                        if in_thinking {
                            yield Ok::<_, Infallible>(Event::default().event("content_block_stop").data(format!(
                                r#"{{"type":"content_block_stop","index":{}}}"#, block_index
                            )));
                            in_thinking = false;
                        }
                        let escaped = segment.content.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
                        yield Ok::<_, Infallible>(Event::default().event("content_block_delta").data(format!(
                            r#"{{"type":"content_block_delta","index":0,"delta":{{"type":"text_delta","text":"{}"}}}}"#,
                            escaped
                        )));
                    }
                }
            }

            // 结束 text block
            yield Ok::<_, Infallible>(Event::default().event("content_block_stop").data(
                r#"{"type":"content_block_stop","index":0}"#.to_string()
            ));

            // message_delta
            let stop_reason = if has_tool { "tool_use" } else if context_exceeded { "max_tokens" } else { "end_turn" };
            let usage_str = output_tokens.map(|t| format!(r#","usage":{{"output_tokens":{}}}"#, t)).unwrap_or_default();
            yield Ok::<_, Infallible>(Event::default().event("message_delta").data(format!(
                r#"{{"type":"message_delta","delta":{{"stop_reason":"{}"}}{}}}"#, stop_reason, usage_str
            )));

            // message_stop
            yield Ok::<_, Infallible>(Event::default().event("message_stop").data(r#"{"type":"message_stop"}"#.to_string()));
        };
        Ok(Sse::new(anthropic_stream).into_response())
    } else {
        // 非流式响应：收集所有内容
        tokio::pin!(stream);
        let mut content_blocks: Vec<serde_json::Value> = Vec::new();
        let mut text_content = String::new();
        let mut thinking_content = String::new();
        let mut thinking_signature: Option<String> = None;
        let mut output_tokens: Option<i32> = None;
        let mut input_tokens: Option<i32> = None;
        let mut context_exceeded = false;
        let mut has_tool = false;

        while let Some(Ok(event)) = stream.next().await {
            // 收集文本
            if let Some(ref content) = event.content {
                if event.language.is_some() {
                    // 代码块
                    let lang = event.language.as_deref().unwrap_or("");
                    text_content.push_str(&format!("\n```{}\n{}\n```\n", lang, content));
                } else {
                    // 普通文本
                    text_content.push_str(content);
                }
            }

            // 收集 thinking
            if let Some(ref text) = event.text {
                thinking_content.push_str(text);
                if event.signature.is_some() {
                    thinking_signature = event.signature.clone();
                }
            }

            // 收集工具调用
            if let (Some(ref tool_use_id), Some(ref name), Some(ref input)) = 
                (&event.tool_use_id, &event.name, &event.input) {
                has_tool = true;
                content_blocks.push(serde_json::json!({
                    "type": "tool_use",
                    "id": tool_use_id,
                    "name": name,
                    "input": input
                }));
            }

            // 收集 usage
            if let Some(usage_val) = event.usage {
                output_tokens = Some((usage_val * 1000.0) as i32);
                // 粗略估算 input tokens
                input_tokens = Some((usage_val * 1000.0) as i32 / 2);
            }

            // 检查错误
            if let Some(ref reason) = event.reason {
                if reason == "CONTEXT_LENGTH_EXCEEDED" {
                    context_exceeded = true;
                }
            }
        }

        // 构建 content 数组
        let mut content = Vec::new();
        
        // 添加 thinking block（如果有）
        if !thinking_content.is_empty() {
            content.push(serde_json::json!({
                "type": "thinking",
                "thinking": thinking_content,
                "signature": thinking_signature.unwrap_or_default()
            }));
        }
        
        // 添加 text block
        if !text_content.is_empty() {
            content.push(serde_json::json!({
                "type": "text",
                "text": text_content
            }));
        }
        
        // 添加 tool_use blocks
        content.extend(content_blocks);

        let stop_reason = if has_tool { "tool_use" } else if context_exceeded { "max_tokens" } else { "end_turn" };

        let response = serde_json::json!({
            "id": format!("msg_{}", request_id),
            "type": "message",
            "role": "assistant",
            "content": content,
            "model": model,
            "stop_reason": stop_reason,
            "stop_sequence": null,
            "usage": {
                "input_tokens": input_tokens.unwrap_or(0),
                "output_tokens": output_tokens.unwrap_or(0)
            }
        });

        Ok(Json(response).into_response())
    }
}

async fn list_models() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "object": "list",
        "data": [
            {"id": "kiro", "object": "model", "owned_by": "amazon"},
            {"id": "claude-sonnet-4.5", "object": "model", "owned_by": "anthropic"},
            {"id": "claude-sonnet-4", "object": "model", "owned_by": "anthropic"},
            {"id": "claude-opus-4.5", "object": "model", "owned_by": "anthropic"},
            {"id": "claude-haiku-4.5", "object": "model", "owned_by": "anthropic"},
            {"id": "auto", "object": "model", "owned_by": "amazon"}
        ]
    }))
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok", "version": env!("CARGO_PKG_VERSION")}))
}

// ============ 管理 API ============

/// 列出所有账号
async fn admin_list_accounts(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;
    
    let accounts = state.accounts.list_accounts();
    let account_info: Vec<_> = accounts.iter().map(|a| {
        serde_json::json!({
            "id": a.id,
            "name": a.name,
            "provider": a.provider,
            "authMethod": a.auth_method,
            "enabled": a.enabled,
            "status": a.status,
            "isExpired": a.is_expired(),
            "isThrottled": a.is_throttled(),
        })
    }).collect();
    
    Ok(Json(serde_json::json!({
        "accounts": account_info,
        "total": accounts.len()
    })))
}

/// 获取指定账号的配额（自动刷新 Token）
async fn admin_get_quota(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(account_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;
    
    // 先尝试刷新 Token
    let account = state.accounts.refresh_account(&account_id).await
        .map_err(|e| AppError::BadRequest(format!("刷新 Token 失败: {}", e)))?;
    
    let quota = state.client.get_usage_limits(&account).await?;
    Ok(Json(quota))
}

/// 健康检查所有账号
async fn admin_health_check(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;
    
    let accounts = state.accounts.list_accounts();
    let mut results = Vec::new();
    
    for account in &accounts {
        let healthy = state.client.health_check(account).await;
        results.push(serde_json::json!({
            "id": account.id,
            "name": account.name,
            "healthy": healthy
        }));
    }
    
    Ok(Json(serde_json::json!({
        "results": results,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// 使用统计
async fn admin_stats(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;
    
    let accounts = state.accounts.list_accounts();
    let active_count = accounts.iter().filter(|a| a.is_available()).count();
    
    Ok(Json(serde_json::json!({
        "totalAccounts": accounts.len(),
        "activeAccounts": active_count,
        "version": env!("CARGO_PKG_VERSION"),
        "uptime": "N/A"  // TODO: 实现运行时间统计
    })))
}

/// 从 Kiro IDE 缓存导入账号
async fn admin_import_kiro(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;
    
    // 读取 Kiro 缓存文件
    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| AppError::BadRequest("无法获取用户目录".into()))?;
    
    let cache_path = std::path::Path::new(&home)
        .join(".aws")
        .join("sso")
        .join("cache")
        .join("kiro-auth-token.json");
    
    let content = std::fs::read_to_string(&cache_path)
        .map_err(|e| AppError::BadRequest(format!("无法读取 Kiro 缓存: {}", e)))?;
    
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct KiroCache {
        access_token: String,
        refresh_token: String,
        expires_at: Option<String>,
        auth_method: Option<String>,
        provider: Option<String>,
        region: Option<String>,
        client_id_hash: Option<String>,
    }
    
    let cache: KiroCache = serde_json::from_str(&content)
        .map_err(|e| AppError::ParseError(format!("解析 Kiro 缓存失败: {}", e)))?;
    
    // 构建账号 JSON
    let account_id = cache.client_id_hash.as_deref().unwrap_or("kiro-default");
    let auth_method = cache.auth_method.as_deref().unwrap_or("social");
    let provider = cache.provider.as_deref().unwrap_or("unknown");
    
    // 如果是 IDC，需要读取客户端注册信息
    let (client_id, client_secret) = if auth_method.to_lowercase() == "idc" {
        if let Some(ref hash) = cache.client_id_hash {
            let client_reg_path = std::path::Path::new(&home)
                .join(".aws")
                .join("sso")
                .join("cache")
                .join(format!("{}.json", hash));
            
            if let Ok(reg_content) = std::fs::read_to_string(&client_reg_path) {
                #[derive(Deserialize)]
                #[serde(rename_all = "camelCase")]
                struct ClientReg {
                    client_id: String,
                    client_secret: String,
                }
                
                if let Ok(reg) = serde_json::from_str::<ClientReg>(&reg_content) {
                    (Some(reg.client_id), Some(reg.client_secret))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };
    
    let accounts_json = serde_json::json!({
        "accounts": [{
            "id": account_id,
            "name": format!("Kiro IDE ({})", provider),
            "authMethod": auth_method,
            "accessToken": cache.access_token,
            "refreshToken": cache.refresh_token,
            "expire": cache.expires_at,
            "region": cache.region.unwrap_or_else(|| "us-east-1".to_string()),
            "profileArn": "",
            "clientId": client_id,
            "clientSecret": client_secret,
            "enabled": true
        }]
    });
    
    state.accounts.load_from_json(&accounts_json.to_string())?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "已从 Kiro IDE 导入账号",
        "accountId": account_id
    })))
}

/// 手动添加账号
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddAccountRequest {
    id: Option<String>,
    provider: Option<String>,
    #[serde(alias = "authMethod", default)]
    auth_method: Option<String>,
    #[serde(alias = "accessToken")]
    access_token: Option<String>,
    #[serde(alias = "refreshToken")]
    refresh_token: String,
    region: Option<String>,
    #[serde(alias = "profileArn")]
    profile_arn: Option<String>,
    #[serde(alias = "clientId")]
    client_id: Option<String>,
    #[serde(alias = "clientSecret")]
    client_secret: Option<String>,
}

async fn admin_add_account(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<AddAccountRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;
    
    // 自动推断 authMethod：有 clientId 就是 idc，否则是 social
    let auth_method = request.auth_method
        .unwrap_or_else(|| {
            if request.client_id.is_some() { "idc".to_string() } else { "social".to_string() }
        });
    
    // 校验必须字段
    if auth_method.to_lowercase() == "idc" {
        if request.client_id.is_none() {
            return Err(AppError::BadRequest("IdC 账号必须提供 clientId".into()));
        }
        if request.client_secret.is_none() {
            return Err(AppError::BadRequest("IdC 账号必须提供 clientSecret".into()));
        }
    }
    
    // 自动推断 provider
    let provider = request.provider.unwrap_or_else(|| {
        if auth_method == "idc" { "BuilderId".to_string() } else { "Google".to_string() }
    });
    
    let account_id = request.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    
    // 获取现有账号
    let mut accounts = state.accounts.list_accounts();
    
    // 添加新账号
    let new_account = account::Account {
        id: account_id.clone(),
        name: Some(provider.clone()),
        provider: Some(provider),
        auth_method,
        access_token: request.access_token.unwrap_or_default(),
        refresh_token: request.refresh_token,
        profile_arn: request.profile_arn.unwrap_or_default(),
        region: request.region,
        expires_at: None,
        expire: None,
        client_id: request.client_id,
        client_secret: request.client_secret,
        enabled: true,
        status: account::AccountStatus::Active,
        throttled_until: None,
    };
    
    accounts.push(new_account);
    
    // 重新加载
    let json = serde_json::json!({ "accounts": accounts });
    state.accounts.load_from_json(&json.to_string())?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "accountId": account_id
    })))
}

/// 删除账号
async fn admin_delete_account(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(account_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;
    
    let mut accounts = state.accounts.list_accounts();
    let original_len = accounts.len();
    accounts.retain(|a| a.id != account_id);
    
    if accounts.len() == original_len {
        return Err(AppError::BadRequest(format!("账号 {} 不存在", account_id)));
    }
    
    let json = serde_json::json!({ "accounts": accounts });
    state.accounts.load_from_json(&json.to_string())?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "accountId": account_id
    })))
}

/// 刷新账号 Token
async fn admin_refresh_account(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(account_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;
    
    let account = state.accounts.refresh_account(&account_id).await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "accountId": account.id,
        "status": account.status
    })))
}

/// 启用账号
async fn admin_enable_account(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(account_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;
    state.accounts.set_account_enabled(&account_id, true)?;
    Ok(Json(serde_json::json!({ "success": true, "accountId": account_id, "enabled": true })))
}

/// 禁用账号
async fn admin_disable_account(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(account_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;
    state.accounts.set_account_enabled(&account_id, false)?;
    Ok(Json(serde_json::json!({ "success": true, "accountId": account_id, "enabled": false })))
}

// ============ 用户记忆 API ============

/// 列出用户记忆
async fn list_memory(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;
    
    let account = state.accounts.get_account().await?;
    let memories = state.client.list_user_memory(&account).await?;
    Ok(Json(memories))
}

/// 创建用户记忆
#[derive(Deserialize)]
struct CreateMemoryRequest {
    content: String,
}

async fn create_memory(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<CreateMemoryRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;
    
    let account = state.accounts.get_account().await?;
    let result = state.client.create_user_memory(&account, &request.content).await?;
    Ok(Json(result))
}

/// 删除用户记忆
async fn delete_memory(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(entry_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;
    
    let account = state.accounts.get_account().await?;
    state.client.delete_user_memory(&account, &entry_id).await?;
    Ok(Json(serde_json::json!({"success": true})))
}


// ============ 日志 API ============

/// 获取日志
async fn admin_logs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;
    
    let logs = logger::get_logs().await;
    Ok(Json(serde_json::json!({
        "logs": logs,
        "total": logs.len()
    })))
}

/// 清空日志
async fn admin_clear_logs(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;
    
    logger::clear_logs().await;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "日志已清空"
    })))
}
/// 获取统计数据
async fn admin_metrics(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;

    let metrics_data = metrics::METRICS.get_metrics();
    Ok(Json(serde_json::to_value(metrics_data).unwrap()))
}









