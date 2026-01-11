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
use tracing::info;

mod account;
mod config;
mod converter;
mod error;
mod kiro_client;
mod models;

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
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "kiro_gate=info".to_string()))
        .init();

    let config = AppConfig::from_env();
    let client = KiroClient::new(config.clone());
    let accounts = AccountManager::new();

    if let Some(ref json) = config.accounts_json {
        let _ = accounts.load_from_json(json);
    } else if let Some(ref file) = config.accounts_file {
        if let Ok(content) = std::fs::read_to_string(file) {
            let _ = accounts.load_from_json(&content);
        }
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
        .route("/admin/quota/{account_id}", get(admin_get_quota))
        .route("/admin/health", get(admin_health_check))
        .route("/admin/stats", get(admin_stats))
        // 用户记忆 API
        .route("/v1/memory", get(list_memory).post(create_memory))
        .route("/v1/memory/{entry_id}", delete(delete_memory))
        .with_state(state)
        .layer(tower_http::cors::CorsLayer::permissive());

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

    let is_stream = is_stream_request_openai(&request);
    let model = request.model.as_deref().unwrap_or("auto");
    let kiro_request = openai_to_kiro(&request, "");
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
                if event.tool_use_event.is_some() { has_tool = true; }

                if let Some(ref meta) = event.metadata_event {
                    if let Some(ref token_usage) = meta.token_usage {
                        let total = token_usage.total_tokens.unwrap_or(0);
                        let output = token_usage.output_tokens.unwrap_or(0);
                        usage = Some(Usage {
                            prompt_tokens: total - output,
                            completion_tokens: output,
                            total_tokens: total,
                        });
                    }
                }

                if let Some(ref invalid) = event.invalid_state_event {
                    if invalid.reason.as_deref() == Some("CONTEXT_LENGTH_EXCEEDED") {
                        context_exceeded = true;
                    }
                }

                // 记录消息 ID
                if let Some(ref msg_meta) = event.message_metadata_event {
                    if let Some(ref msg_id) = msg_meta.message_id {
                        tracing::debug!("消息 ID: {}", msg_id);
                    }
                }

                // 记录上下文使用率
                if let Some(ref ctx) = event.context_usage_event {
                    if let Some(pct) = ctx.context_usage_percentage {
                        tracing::debug!("上下文使用率: {:.1}%", pct * 100.0);
                    }
                }

                if let Some(chunk) = kiro_to_openai(&event, &request_id) {
                    yield Ok::<_, Infallible>(Event::default().data(serde_json::to_string(&chunk).unwrap_or_default()));
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
        let mut content = String::new();
        let mut tool_calls: Vec<serde_json::Value> = Vec::new();
        let mut usage: Option<Usage> = None;
        let mut context_exceeded = false;

        while let Some(Ok(event)) = stream.next().await {
            // 收集文本
            if let Some(ref resp) = event.assistant_response_event {
                content.push_str(&resp.content);
            }
            if let Some(ref code) = event.code_event {
                let lang = code.language.as_deref().unwrap_or("");
                content.push_str(&format!("\n```{}\n{}\n```\n", lang, code.content));
            }

            // 收集工具调用
            if let Some(ref tool) = event.tool_use_event {
                tool_calls.push(serde_json::json!({
                    "id": tool.tool_use_id,
                    "type": "function",
                    "function": {
                        "name": tool.name,
                        "arguments": serde_json::to_string(&tool.input).unwrap_or_default()
                    }
                }));
            }

            // 收集 usage
            if let Some(ref meta) = event.metadata_event {
                if let Some(ref token_usage) = meta.token_usage {
                    let total = token_usage.total_tokens.unwrap_or(0);
                    let output = token_usage.output_tokens.unwrap_or(0);
                    usage = Some(Usage {
                        prompt_tokens: total - output,
                        completion_tokens: output,
                        total_tokens: total,
                    });
                }
            }

            if let Some(ref invalid) = event.invalid_state_event {
                if invalid.reason.as_deref() == Some("CONTEXT_LENGTH_EXCEEDED") {
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
            "content": if content.is_empty() { serde_json::Value::Null } else { serde_json::Value::String(content) }
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
    let kiro_request = anthropic_to_kiro(&request, "");
    let stream = state.client.generate_with_refresh(kiro_request, &state.accounts, model).await?;
    let request_id = uuid::Uuid::new_v4().to_string();

    if is_stream {
        // 流式响应
        let anthropic_stream = async_stream::stream! {
            tokio::pin!(stream);

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
                if let Some(ref meta) = event.metadata_event {
                    if let Some(ref token_usage) = meta.token_usage {
                        output_tokens = token_usage.output_tokens;
                    }
                }

                if let Some(ref invalid) = event.invalid_state_event {
                    if invalid.reason.as_deref() == Some("CONTEXT_LENGTH_EXCEEDED") {
                        context_exceeded = true;
                    }
                }

                // 处理 thinking block
                if let Some(ref reasoning) = event.reasoning_content_event {
                    if reasoning.signature.is_some() {
                        thinking_signature = reasoning.signature.clone();
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

                // 处理文本响应和代码块
                if event.assistant_response_event.is_some() || event.code_event.is_some() {
                    if in_thinking {
                        yield Ok::<_, Infallible>(Event::default().event("content_block_stop").data(format!(
                            r#"{{"type":"content_block_stop","index":{}}}"#, block_index
                        )));
                        in_thinking = false;
                    }
                    if let Some(data) = kiro_to_anthropic(&event) {
                        yield Ok::<_, Infallible>(Event::default().event("content_block_delta").data(data));
                    }
                }

                // 处理工具调用
                if let Some(ref tool) = event.tool_use_event {
                    has_tool = true;
                    yield Ok::<_, Infallible>(Event::default().event("content_block_stop").data(
                        r#"{"type":"content_block_stop","index":0}"#.to_string()
                    ));
                    block_index += 1;
                    yield Ok::<_, Infallible>(Event::default().event("content_block_start").data(format!(
                        r#"{{"type":"content_block_start","index":{},"content_block":{{"type":"tool_use","id":"{}","name":"{}","input":{}}}}}"#,
                        block_index, tool.tool_use_id, tool.name, serde_json::to_string(&tool.input).unwrap_or_default()
                    )));
                    yield Ok::<_, Infallible>(Event::default().event("content_block_stop").data(format!(
                        r#"{{"type":"content_block_stop","index":{}}}"#, block_index
                    )));
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
            if let Some(ref resp) = event.assistant_response_event {
                text_content.push_str(&resp.content);
            }
            if let Some(ref code) = event.code_event {
                let lang = code.language.as_deref().unwrap_or("");
                text_content.push_str(&format!("\n```{}\n{}\n```\n", lang, code.content));
            }

            // 收集 thinking
            if let Some(ref reasoning) = event.reasoning_content_event {
                thinking_content.push_str(&reasoning.text);
                if reasoning.signature.is_some() {
                    thinking_signature = reasoning.signature.clone();
                }
            }

            // 收集工具调用
            if let Some(ref tool) = event.tool_use_event {
                has_tool = true;
                content_blocks.push(serde_json::json!({
                    "type": "tool_use",
                    "id": tool.tool_use_id,
                    "name": tool.name,
                    "input": tool.input
                }));
            }

            // 收集 usage
            if let Some(ref meta) = event.metadata_event {
                if let Some(ref token_usage) = meta.token_usage {
                    output_tokens = token_usage.output_tokens;
                    if let Some(total) = token_usage.total_tokens {
                        input_tokens = Some(total - token_usage.output_tokens.unwrap_or(0));
                    }
                }
            }

            if let Some(ref invalid) = event.invalid_state_event {
                if invalid.reason.as_deref() == Some("CONTEXT_LENGTH_EXCEEDED") {
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

/// 获取指定账号的配额
async fn admin_get_quota(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(account_id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    verify_api_key(&headers, &state.config)?;
    
    let accounts = state.accounts.list_accounts();
    let account = accounts.iter()
        .find(|a| a.id == account_id)
        .ok_or_else(|| AppError::BadRequest(format!("账号 {} 不存在", account_id)))?;
    
    let quota = state.client.get_usage_limits(account).await?;
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
