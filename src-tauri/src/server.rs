use axum::{
    Router,
    routing::{get, post},
    extract::State,
    response::{sse::{Event, Sse}, Response, IntoResponse},
    http::{HeaderMap, Request},
    middleware::{self, Next},
    Json,
};
use std::sync::Arc;
use std::convert::Infallible;
use std::path::PathBuf;
use tokio_stream::StreamExt;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};
use tauri::{AppHandle, Manager};

use crate::account::AccountManager;
use crate::config::AppConfig;
use crate::converter::{build_kiro_payload, anthropic_to_openai, is_stream_request_openai, is_stream_request_anthropic, kiro_to_openai, kiro_to_anthropic, create_openai_end_with_reason};
use crate::error::AppError;
use crate::kiro_client::KiroClient;
use crate::models::{OpenAIRequest, AnthropicRequest, Usage};
use crate::api_key;
use crate::health_checker::HealthChecker;
use crate::config_generator;

pub struct AppState {
    pub config: AppConfig,
    pub client: KiroClient,
    pub accounts: Arc<AccountManager>,
    pub http_client: reqwest::Client,
    pub auth_cache: crate::auth::AuthCache,
    pub api_keys: api_key::ApiKeyManager,
    pub health_checker: Arc<HealthChecker>,
    pub app_handle: AppHandle,
}

// 获取应用数据目录
fn get_app_data_dir(app_handle: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let data_dir = app_handle.path().app_data_dir()?;
    std::fs::create_dir_all(&data_dir)?;
    Ok(data_dir)
}

pub async fn start_server(app_handle: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "kiro_gateway=info".to_string()))
        .init();

    // 获取应用数据目录
    let data_dir = get_app_data_dir(&app_handle)?;
    info!("应用数据目录: {:?}", data_dir);

    let config = AppConfig::from_env();
    let client = KiroClient::new(config.clone());
    let accounts = Arc::new(AccountManager::new());
    let http_client = reqwest::Client::new();
    let auth_cache = crate::auth::AuthCache::new();
    let api_keys = api_key::ApiKeyManager::new();
    
    // 设置账号文件路径（使用应用数据目录）
    let accounts_file = data_dir.join("accounts.json");
    let accounts_file_str = accounts_file.to_string_lossy().to_string();
    
    // 加载账号
    if let Some(ref json) = config.accounts_json {
        if let Err(e) = accounts.load_from_json(json) {
            tracing::error!("从环境变量加载账号失败: {}", e);
        }
        accounts.set_accounts_file(&accounts_file_str);
    } else if let Some(ref file) = config.accounts_file {
        match std::fs::read_to_string(file) {
            Ok(content) => {
                if let Err(e) = accounts.load_from_json(&content) {
                    tracing::error!("从文件 {} 加载账号失败: {}", file, e);
                }
            }
            Err(e) => {
                tracing::warn!("读取账号文件 {} 失败: {}", file, e);
            }
        }
        accounts.set_accounts_file(file);
    } else {
        accounts.set_accounts_file(&accounts_file_str);
        if let Ok(content) = std::fs::read_to_string(&accounts_file) {
            if let Err(e) = accounts.load_from_json(&content) {
                tracing::error!("从默认文件加载账号失败: {}", e);
            }
        }
    }

    // 加载 API Keys（使用应用数据目录）
    let api_keys_file = data_dir.join("api_keys.json");
    let api_keys_file_str = api_keys_file.to_string_lossy().to_string();
    
    match std::fs::read_to_string(&api_keys_file) {
        Ok(content) => {
            if let Err(e) = api_keys.load_from_json(&content) {
                tracing::error!("从文件加载 API Keys 失败: {}", e);
            } else {
                tracing::info!("从文件加载了 API Keys");
            }
        }
        Err(e) => {
            tracing::info!("API Keys 文件不存在或读取失败: {}，将创建新文件", e);
        }
    }
    api_keys.set_keys_file(&api_keys_file_str);
    
    // 创建健康检查器（每 5 分钟检查一次）
    let health_checker = Arc::new(crate::health_checker::HealthChecker::new(Arc::clone(&accounts), 300));

    // 启动健康检查器
    Arc::clone(&health_checker).start();
    info!("账号健康检查器已启动");

    // 加载 Metrics 数据（使用应用数据目录）
    let metrics_file = data_dir.join("metrics.json");
    let metrics_file_str = metrics_file.to_string_lossy().to_string();
    
    if let Err(e) = crate::metrics::METRICS.load_from_file(&metrics_file_str) {
        tracing::warn!("加载 metrics 数据失败: {}，将使用空数据", e);
    } else {
        tracing::info!("已从 {:?} 加载 metrics 数据", metrics_file);
    }

    // 启用 Metrics 持久化任务（每 5 分钟保存一次）
    let metrics_file_for_task = metrics_file_str.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
        loop {
            interval.tick().await;
            if let Err(e) = crate::metrics::METRICS.save_to_file(&metrics_file_for_task) {
                tracing::warn!("保存 metrics 数据失败: {}", e);
            } else {
                tracing::debug!("已保存 metrics 数据");
            }
        }
    });
    info!("Metrics 持久化任务已启动");

    let state = Arc::new(AppState { 
        config: config.clone(), 
        client, 
        accounts,
        http_client,
        auth_cache,
        api_keys,
        health_checker,
        app_handle,
    });

    let app = Router::new()
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/messages", post(messages))
        .route("/v1/models", get(list_models))
        .route("/health", get(health))
        // Admin API - 需要认证
        .nest("/admin", 
            Router::new()
                .route("/accounts", get(admin_get_accounts).post(admin_add_account))
                .route("/accounts/import", post(admin_import_accounts))
                .route("/accounts/:id", axum::routing::patch(admin_update_account).delete(admin_delete_account))
                .route("/accounts/:id/refresh", post(admin_refresh_account))
                .route("/accounts/:id/quota", get(admin_get_quota))
                .route("/health", get(admin_get_health).post(admin_check_health))
                .route("/allocator/stats", get(admin_get_allocator_stats))
                .route("/metrics", get(admin_get_metrics))
                .route("/logs", get(admin_get_logs))
                .route("/logs/clear", post(admin_clear_logs))
                .route("/api-keys", get(admin_list_api_keys).post(admin_generate_api_key))
                .route("/api-keys/:id", axum::routing::patch(admin_update_api_key).delete(admin_delete_api_key))
                .route("/config/generate", post(admin_generate_config))
                .route("/config/server", get(admin_get_server_config).post(admin_update_server_config))
                .layer(middleware::from_fn_with_state(Arc::clone(&state), admin_auth_middleware))
        )
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin([
                    "http://localhost:5173".parse().unwrap(),
                    "http://127.0.0.1:5173".parse().unwrap(),
                    "http://localhost:8080".parse().unwrap(),
                    "http://127.0.0.1:8080".parse().unwrap(),
                    "tauri://localhost".parse().unwrap(),
                ])
                .allow_methods([
                    axum::http::Method::GET,
                    axum::http::Method::POST,
                    axum::http::Method::PATCH,
                    axum::http::Method::DELETE,
                    axum::http::Method::OPTIONS,
                ])
                .allow_headers([
                    axum::http::header::CONTENT_TYPE,
                    axum::http::header::AUTHORIZATION,
                    "x-api-key".parse().unwrap(),
                ])
                .allow_credentials(true)
        );

    let addr = format!("{}:{}", config.host, config.port);
    info!("kiro-gateway Axum 服务启动: http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

fn verify_api_key(headers: &HeaderMap, config: &AppConfig, api_keys: &api_key::ApiKeyManager) -> Result<(), AppError> {
    let provided = headers.get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .or_else(|| headers.get("x-api-key").and_then(|v| v.to_str().ok()));
    
    if let Some(key) = provided {
        // 如果提供了 API Key，先检查是否是 Admin Key
        if let Some(ref admin_key) = config.api_key {
            if key == admin_key {
                return Ok(());
            }
        }
        // 然后验证是否是用户生成的 API Key
        return api_keys.verify_key(key);
    }
    
    // 如果没有提供 API Key
    if config.api_key.is_some() {
        // 配置了 Admin Key，必须提供
        return Err(AppError::BadRequest("Missing API key".into()));
    }
    
    // 没有配置 Admin Key，允许无认证访问（开发模式）
    Ok(())
}

// Admin API 认证中间件
async fn admin_auth_middleware(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    request: Request<axum::body::Body>,
    next: Next,
) -> Result<Response, AppError> {
    // 检查是否提供了 Admin API Key
    let provided = headers.get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .or_else(|| headers.get("x-api-key").and_then(|v| v.to_str().ok()));
    
    // 如果配置了 API_KEY，必须提供且匹配
    if let Some(ref admin_key) = state.config.api_key {
        match provided {
            Some(key) if key == admin_key => {
                // 认证通过
                Ok(next.run(request).await)
            }
            Some(_) => {
                // 提供了 key 但不匹配
                Err(AppError::BadRequest("Invalid admin API key".into()))
            }
            None => {
                // 未提供 key
                Err(AppError::BadRequest("Admin API key required".into()))
            }
        }
    } else {
        // 未配置 API_KEY，允许访问（仅限本地开发）
        Ok(next.run(request).await)
    }
}

async fn chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<OpenAIRequest>,
) -> Result<Response, AppError> {
    let start_time = std::time::Instant::now();
    
    verify_api_key(&headers, &state.config, &state.api_keys)?;

    let is_stream = is_stream_request_openai(&request);
    let model = request.model.clone();
    
    // 记录请求日志
    crate::kirogate_info!("收到 OpenAI 请求: model={}, stream={}", model, is_stream);
    
    let account = state.accounts.get_account().await?;
    // 将空字符串的 profileArn 转换为 None
    let profile_arn = if account.profile_arn.is_empty() {
        None
    } else {
        Some(account.profile_arn.clone())
    };
    let kiro_request = build_kiro_payload(&request, profile_arn)
        .map_err(AppError::BadRequest)?;
    
    let stream = state.client.generate_with_refresh(kiro_request, &state.accounts, &model).await?;
    let request_id = uuid::Uuid::new_v4().to_string();

    if is_stream {
        let request_id_clone = request_id.clone();
        let model_clone = model.clone();
        let openai_stream = async_stream::stream! {
            tokio::pin!(stream);
            let mut has_tool = false;
            let mut usage: Option<Usage> = None;

            while let Some(Ok(event)) = stream.next().await {
                if event.tool_use_id.is_some() { has_tool = true; }
                if let Some(usage_val) = event.usage {
                    let total = (usage_val * 1000.0) as i32;
                    usage = Some(Usage {
                        prompt_tokens: total / 2,
                        completion_tokens: total / 2,
                        total_tokens: total,
                    });
                }
                if let Some(chunk) = kiro_to_openai(&event, &request_id_clone) {
                    yield Ok::<_, Infallible>(Event::default().data(serde_json::to_string(&chunk).unwrap_or_default()));
                }
            }

            let end = create_openai_end_with_reason(&request_id_clone, has_tool, false, usage);
            yield Ok::<_, Infallible>(Event::default().data(serde_json::to_string(&end).unwrap_or_default()));
            yield Ok::<_, Infallible>(Event::default().data("[DONE]"));
        };
        
        // 记录 Metrics
        let duration_ms = start_time.elapsed().as_millis() as f64;
        crate::metrics::METRICS.record_request(
            "/v1/chat/completions",
            200,
            duration_ms,
            &model_clone,
            true,
            "openai"
        );
        
        crate::kirogate_info!("OpenAI 流式响应开始: request_id={}", request_id);
        Ok(Sse::new(openai_stream).into_response())
    } else {
        Err(AppError::BadRequest("非流式响应暂未实现".into()))
    }
}

async fn messages(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<AnthropicRequest>,
) -> Result<Response, AppError> {
    let start_time = std::time::Instant::now();
    
    verify_api_key(&headers, &state.config, &state.api_keys)?;

    // 记录请求日志
    crate::kirogate_info!("收到 Anthropic 请求: model={}", request.model);

    // 检查是否为 WebSearch 请求
    if crate::websearch::is_web_search_request(&request) {
        let account = state.accounts.get_account().await?;
        let verify_result = account.to_verify_result();
        return Ok(crate::websearch::handle_web_search_request(
            state,
            headers,
            request,
            verify_result,
        ).await);
    }

    let is_stream = is_stream_request_anthropic(&request);
    let model = request.model.clone();

    let account = state.accounts.get_account().await?;
    let openai_request = anthropic_to_openai(&request);
    // 将空字符串的 profileArn 转换为 None
    let profile_arn = if account.profile_arn.is_empty() {
        None
    } else {
        Some(account.profile_arn.clone())
    };
    let kiro_request = build_kiro_payload(&openai_request, profile_arn)
        .map_err(AppError::BadRequest)?;

    let stream = state.client.generate_with_refresh(kiro_request, &state.accounts, &model).await?;
    let request_id = uuid::Uuid::new_v4().to_string();

    if is_stream {
        let request_id_clone = request_id.clone();
        let model_clone = model.clone();
        let anthropic_stream = async_stream::stream! {
            tokio::pin!(stream);
            
            yield Ok::<_, Infallible>(Event::default().event("message_start").data(format!(
                r#"{{"type":"message_start","message":{{"id":"msg_{}","type":"message","role":"assistant","content":[],"model":"claude","stop_reason":null}}}}"#,
                request_id_clone
            )));

            yield Ok::<_, Infallible>(Event::default().event("content_block_start").data(
                r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#
            ));

            while let Some(Ok(event)) = stream.next().await {
                if let Some(data) = kiro_to_anthropic(&event) {
                    yield Ok::<_, Infallible>(Event::default().event("content_block_delta").data(data));
                }
            }

            yield Ok::<_, Infallible>(Event::default().event("content_block_stop").data(
                r#"{"type":"content_block_stop","index":0}"#
            ));

            yield Ok::<_, Infallible>(Event::default().event("message_delta").data(
                r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"}}"#
            ));

            yield Ok::<_, Infallible>(Event::default().event("message_stop").data(
                r#"{"type":"message_stop"}"#
            ));
        };
        
        // 记录 Metrics
        let duration_ms = start_time.elapsed().as_millis() as f64;
        crate::metrics::METRICS.record_request(
            "/v1/messages",
            200,
            duration_ms,
            &model_clone,
            true,
            "anthropic"
        );
        
        crate::kirogate_info!("Anthropic 流式响应开始: request_id={}", request_id);
        Ok(Sse::new(anthropic_stream).into_response())
    } else {
        // 非流式响应：收集所有流事件并组装成完整响应
        let request_id_clone = request_id.clone();
        let model_clone = model.clone();
        
        tokio::pin!(stream);
        let mut content_parts: Vec<String> = Vec::new();
        let mut tool_calls_map: std::collections::HashMap<String, (String, Vec<String>)> = std::collections::HashMap::new();
        let mut usage_info: Option<crate::models::Usage> = None;
        
        // 收集所有流事件
        while let Some(Ok(event)) = stream.next().await {
            // 收集文本内容
            if let Some(text) = event.content {
                content_parts.push(text);
            }
            
            // 收集工具调用（input 是分片传输的，需要合并）
            if let Some(tool_use_id) = event.tool_use_id {
                if let Some(tool_name) = event.name {
                    // 获取或创建工具调用条目
                    let entry = tool_calls_map.entry(tool_use_id.clone())
                        .or_insert_with(|| (tool_name.clone(), Vec::new()));
                    
                    // 添加 input 片段
                    if let Some(input) = event.input {
                        if let Some(input_str) = input.as_str() {
                            entry.1.push(input_str.to_string());
                        } else {
                            // 如果是对象，直接序列化
                            entry.1.push(serde_json::to_string(&input).unwrap_or_default());
                        }
                    }
                }
            }
            
            // 收集 usage 信息
            if let Some(usage_val) = event.usage {
                let total = (usage_val * 1000.0) as i32;
                usage_info = Some(crate::models::Usage {
                    prompt_tokens: total / 2,
                    completion_tokens: total / 2,
                    total_tokens: total,
                });
            }
        }
        
        // 合并所有文本内容
        let full_content = content_parts.join("");
        
        // 构建 content blocks
        let mut content_blocks: Vec<serde_json::Value> = Vec::new();
        
        // 添加文本 block（如果有内容）
        if !full_content.is_empty() {
            content_blocks.push(serde_json::json!({
                "type": "text",
                "text": full_content
            }));
        }
        
        // 添加 tool_use blocks（合并 input 片段）
        for (tool_use_id, (tool_name, input_parts)) in tool_calls_map {
            let full_input = input_parts.join("");
            // 尝试解析为 JSON
            let input_json: serde_json::Value = serde_json::from_str(&full_input)
                .unwrap_or_else(|_| serde_json::json!({}));
            
            content_blocks.push(serde_json::json!({
                "type": "tool_use",
                "id": tool_use_id,
                "name": tool_name,
                "input": input_json
            }));
        }
        
        // 确定 stop_reason
        let stop_reason = if content_blocks.iter().any(|b| b.get("type").and_then(|t| t.as_str()) == Some("tool_use")) {
            "tool_use"
        } else {
            "end_turn"
        };
        
        // 构建完整响应
        let response = serde_json::json!({
            "id": format!("msg_{}", request_id_clone),
            "type": "message",
            "role": "assistant",
            "content": content_blocks,
            "model": "claude",
            "stop_reason": stop_reason,
            "stop_sequence": null,
            "usage": usage_info.map(|u| serde_json::json!({
                "input_tokens": u.prompt_tokens,
                "output_tokens": u.completion_tokens
            }))
        });
        
        // 记录 Metrics
        let duration_ms = start_time.elapsed().as_millis() as f64;
        crate::metrics::METRICS.record_request(
            "/v1/messages",
            200,
            duration_ms,
            &model_clone,
            false,
            "anthropic"
        );
        
        crate::kirogate_info!("Anthropic 非流式响应完成: request_id={}", request_id);
        Ok(Json(response).into_response())
    }
}

async fn list_models(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, AppError> {
    // 尝试获取一个可用账号
    match state.accounts.get_account().await {
        Ok(account) => {
            // 调用 Kiro API 获取模型列表
            match state.client.list_available_models(&account).await {
                Ok(kiro_models) => {
                    // 转换为 OpenAI 格式
                    let models: Vec<serde_json::Value> = kiro_models.iter()
                        .filter_map(|m| {
                            m.get("modelId")
                                .and_then(|id| id.as_str())
                                .map(|id| {
                                    // 移除 qdev:: 前缀
                                    let clean_id = id.strip_prefix("qdev::").unwrap_or(id);
                                    serde_json::json!({
                                        "id": clean_id,
                                        "object": "model",
                                        "owned_by": "anthropic"
                                    })
                                })
                        })
                        .collect();
                    
                    Ok(Json(serde_json::json!({
                        "object": "list",
                        "data": models
                    })))
                }
                Err(AppError::TokenExpired) => {
                    // Token 过期，尝试刷新后重试
                    match state.accounts.refresh_account(&account.id).await {
                        Ok(refreshed) => {
                            match state.client.list_available_models(&refreshed).await {
                                Ok(kiro_models) => {
                                    let models: Vec<serde_json::Value> = kiro_models.iter()
                                        .filter_map(|m| {
                                            m.get("modelId")
                                                .and_then(|id| id.as_str())
                                                .map(|id| {
                                                    let clean_id = id.strip_prefix("qdev::").unwrap_or(id);
                                                    serde_json::json!({
                                                        "id": clean_id,
                                                        "object": "model",
                                                        "owned_by": "anthropic"
                                                    })
                                                })
                                        })
                                        .collect();
                                    
                                    Ok(Json(serde_json::json!({
                                        "object": "list",
                                        "data": models
                                    })))
                                }
                                Err(_) => {
                                    // 刷新后仍失败，返回默认列表
                                    Ok(Json(get_default_models()))
                                }
                            }
                        }
                        Err(_) => {
                            // 刷新失败，返回默认列表
                            Ok(Json(get_default_models()))
                        }
                    }
                }
                Err(_) => {
                    // 其他错误，返回默认列表
                    Ok(Json(get_default_models()))
                }
            }
        }
        Err(_) => {
            // 没有可用账号，返回默认列表
            Ok(Json(get_default_models()))
        }
    }
}

// 默认模型列表（当无法从 Kiro API 获取时使用）
fn get_default_models() -> serde_json::Value {
    serde_json::json!({
        "object": "list",
        "data": [
            {"id": "claude-haiku-4.5", "object": "model", "owned_by": "anthropic"},
            {"id": "claude-sonnet-4", "object": "model", "owned_by": "anthropic"},
            {"id": "claude-sonnet-4.5", "object": "model", "owned_by": "anthropic"},
        ]
    })
}

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

// Admin API handlers
async fn admin_get_accounts(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let accounts = state.accounts.list_accounts();
    Json(serde_json::json!({ "accounts": accounts }))
}

async fn admin_add_account(
    State(state): State<Arc<AppState>>,
    Json(account): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 解析账号数据
    let account: crate::account::Account = serde_json::from_value(account)
        .map_err(|e| AppError::BadRequest(format!("账号数据格式错误: {}", e)))?;
    
    // 添加到账号列表
    state.accounts.add_account(account.clone())?;
    
    // 保存到文件
    let all_accounts = state.accounts.list_accounts();
    state.accounts.save_accounts_to_file(&all_accounts)?;
    
    Ok(Json(serde_json::json!({ "success": true, "account": account })))
}

async fn admin_update_account(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let id = payload.get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("缺少账号 ID".into()))?;
    
    let updates = payload.get("updates")
        .ok_or_else(|| AppError::BadRequest("缺少更新数据".into()))?;
    
    // 更新账号
    state.accounts.update_account(id, |account| {
        // 更新字段
        if let Some(status) = updates.get("status").and_then(|v| v.as_str()) {
            account.status = match status {
                "active" => crate::account::AccountStatus::Active,
                "disabled" => crate::account::AccountStatus::Disabled,
                "expired" => crate::account::AccountStatus::Expired,
                "throttled" => crate::account::AccountStatus::Throttled,
                "error" => crate::account::AccountStatus::Error,
                "banned" => crate::account::AccountStatus::Banned,
                _ => crate::account::AccountStatus::Active,
            };
        }
        
        if let Some(enabled) = updates.get("enabled").and_then(|v| v.as_bool()) {
            account.enabled = enabled;
        }
    })?;
    
    // 保存到文件
    let all_accounts = state.accounts.list_accounts();
    state.accounts.save_accounts_to_file(&all_accounts)?;
    
    Ok(Json(serde_json::json!({ "success": true })))
}

async fn admin_delete_account(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 删除账号
    state.accounts.delete_account(&id)?;
    
    // 保存到文件
    let all_accounts = state.accounts.list_accounts();
    state.accounts.save_accounts_to_file(&all_accounts)?;
    
    Ok(Json(serde_json::json!({ "success": true })))
}

async fn admin_refresh_account(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.accounts.refresh_account(&id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

async fn admin_get_metrics() -> Json<serde_json::Value> {
    let metrics = crate::metrics::METRICS.get_metrics();
    Json(serde_json::to_value(metrics).unwrap_or_default())
}

async fn admin_get_logs() -> Json<serde_json::Value> {
    let logs = crate::logger::get_logs().await;
    Json(serde_json::json!(logs))
}

async fn admin_clear_logs() -> Json<serde_json::Value> {
    crate::logger::clear_logs().await;
    Json(serde_json::json!({"success": true}))
}

// 获取健康检查状态
async fn admin_get_health(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let accounts = state.accounts.list_accounts();
    let allocator = state.accounts.get_allocator();
    let stats = allocator.get_all_stats();
    
    let health_info: Vec<_> = accounts.iter().map(|acc| {
        let account_stats = stats.get(&acc.id);
        serde_json::json!({
            "id": acc.id,
            "name": acc.name,
            "status": acc.status,
            "enabled": acc.enabled,
            "is_throttled": acc.is_throttled(),
            "is_available": acc.is_available(),
            "success_count": account_stats.map(|s| s.success_count).unwrap_or(0),
            "fail_count": account_stats.map(|s| s.fail_count).unwrap_or(0),
            "success_rate": account_stats.map(|s| s.success_rate()).unwrap_or(1.0),
            "last_used": account_stats.and_then(|s| s.last_used),
        })
    }).collect();
    
    Json(serde_json::json!({
        "accounts": health_info,
        "total": accounts.len(),
        "available": accounts.iter().filter(|a| a.is_available()).count(),
    }))
}

// 手动触发健康检查
async fn admin_check_health(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, AppError> {
    let summary = state.health_checker.check_all_accounts().await
        .map_err(AppError::BadRequest)?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "checked": summary.checked,
        "valid": summary.valid,
        "invalid": summary.invalid,
    })))
}

// 获取智能分配器统计
async fn admin_get_allocator_stats(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let allocator = state.accounts.get_allocator();
    let stats = allocator.get_all_stats();
    
    let stats_list: Vec<_> = stats.iter().map(|(id, stat)| {
        serde_json::json!({
            "account_id": id,
            "success_count": stat.success_count,
            "fail_count": stat.fail_count,
            "success_rate": stat.success_rate(),
            "last_used": stat.last_used,
        })
    }).collect();
    
    Json(serde_json::json!({
        "stats": stats_list
    }))
}

// 获取账号配额
async fn admin_get_quota(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 先检查缓存（5 分钟内有效）
    if let Some(cached_quota) = state.accounts.get_quota_cache(&id) {
        info!("使用账号 {} 的配额缓存", id);
        return Ok(Json(cached_quota));
    }

    // 获取账号
    let accounts = state.accounts.list_accounts();
    let account = accounts.iter()
        .find(|a| a.id == id)
        .ok_or_else(|| AppError::BadRequest(format!("账号 {} 不存在", id)))?;
    
    // 如果 Token 即将过期（5 分钟内），先刷新
    let account = if account.is_expired() {
        info!("账号 {} Token 即将过期，刷新后查询配额", id);
        state.accounts.refresh_account(&id).await?
    } else {
        account.clone()
    };
    
    // 查询配额
    match state.client.get_usage_limits(&account).await {
        Ok(quota) => {
            // 缓存配额数据（5 分钟）
            state.accounts.update_quota_cache(&id, quota.clone());
            Ok(Json(quota))
        }
        Err(AppError::TokenExpired) => {
            // Token 过期，刷新后重试
            info!("配额查询时 Token 过期，刷新后重试");
            let refreshed = state.accounts.refresh_account(&id).await?;
            let quota = state.client.get_usage_limits(&refreshed).await?;
            // 缓存配额数据
            state.accounts.update_quota_cache(&id, quota.clone());
            Ok(Json(quota))
        }
        Err(AppError::RateLimited) => {
            // 限流错误，返回缓存（如果有）或错误
            warn!("配额查询被限流");
            if let Some(cached_quota) = state.accounts.get_quota_cache(&id) {
                info!("限流时使用账号 {} 的旧缓存", id);
                return Ok(Json(cached_quota));
            }
            Err(AppError::RateLimited)
        }
        Err(AppError::AccountBanned(msg)) => {
            // 账号被封禁，标记状态
            warn!("账号 {} 已被封禁: {}", id, msg);
            state.accounts.mark_status(&id, crate::account::AccountStatus::Banned);
            Err(AppError::AccountBanned(msg))
        }
        Err(e) => Err(e),
    }
}

// 从 Kiro IDE 缓存导入账号
async fn admin_import_accounts(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, AppError> {
    // 获取用户主目录
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::BadRequest("无法获取用户目录".into()))?;
    
    let kiro_token_path = home.join(".aws").join("sso").join("cache").join("kiro-auth-token.json");
    
    if !kiro_token_path.exists() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "未找到 Kiro IDE 缓存文件",
            "accounts": []
        })));
    }
    
    let content = std::fs::read_to_string(&kiro_token_path)
        .map_err(|e| AppError::BadRequest(format!("读取文件失败: {}", e)))?;
    
    let token_data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| AppError::BadRequest(format!("解析 JSON 失败: {}", e)))?;
    
    // 提取 refreshToken 和 accessToken
    let refresh_token = token_data["refreshToken"]
        .as_str()
        .ok_or_else(|| AppError::BadRequest("未找到 refreshToken".into()))?;
    
    let access_token = token_data["accessToken"]
        .as_str()
        .unwrap_or("");
    
    let expires_at = token_data["expiresAt"]
        .as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.timestamp_millis() as u64)
        .unwrap_or(0);
    
    // 创建账号对象
    let account = serde_json::json!({
        "name": "Kiro IDE (导入)",
        "type": "social",
        "refreshToken": refresh_token,
        "accessToken": access_token,
        "expiresAt": expires_at,
        "profileArn": "",
        "region": "us-east-1"
    });
    
    Ok(Json(serde_json::json!({
        "success": true,
        "accounts": [account]
    })))
}

// API Key 管理端点
async fn admin_list_api_keys(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let keys = state.api_keys.list_keys();
    Json(serde_json::json!({ "keys": keys }))
}

async fn admin_generate_api_key(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let name = payload.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
    let format = payload.get("format").and_then(|v| v.as_str()).unwrap_or("base62");
    
    let api_key = match format {
        "hex" => state.api_keys.generate_key_hex(name)?,
        _ => state.api_keys.generate_key(name)?,
    };
    
    Ok(Json(serde_json::json!({ "key": api_key })))
}

async fn admin_delete_api_key(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    state.api_keys.delete_key(&id)?;
    Ok(Json(serde_json::json!({ "success": true })))
}

async fn admin_update_api_key(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(enabled) = payload.get("enabled").and_then(|v| v.as_bool()) {
        state.api_keys.set_key_enabled(&id, enabled)?;
        Ok(Json(serde_json::json!({ "success": true })))
    } else {
        Err(AppError::BadRequest("缺少 enabled 字段".into()))
    }
}

// 生成配置包
async fn admin_generate_config(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 获取 API Key（可选，如果不提供则生成新的）
    let api_key = if let Some(key) = payload.get("apiKey").and_then(|v| v.as_str()) {
        key.to_string()
    } else {
        // 生成新的 API Key
        let new_key = state.api_keys.generate_key(Some("自动生成（配置用）".to_string()))?;
        new_key.key
    };
    
    // 获取基础 URL
    let base_url = format!("http://{}:{}", state.config.host, state.config.port);
    
    // 生成配置包
    let config_package = config_generator::generate_config_package(&base_url, &api_key)
        .map_err(AppError::BadRequest)?;
    
    Ok(Json(serde_json::to_value(config_package).unwrap_or_default()))
}

// 获取服务器配置
async fn admin_get_server_config(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "host": state.config.host,
        "port": state.config.port,
    }))
}

// 更新服务器配置
async fn admin_update_server_config(
    State(_state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let host = payload.get("host")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("缺少 host 参数".into()))?;
    
    let port = payload.get("port")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| AppError::BadRequest("缺少 port 参数".into()))? as u16;
    
    // 验证端口范围
    if port < 1024 {
        return Err(AppError::BadRequest("端口必须大于等于 1024".into()));
    }
    
    // 读取或创建 .env 文件
    let env_path = "src-tauri/.env";
    let mut env_content = std::fs::read_to_string(env_path).unwrap_or_default();
    
    // 更新或添加 HOST 和 PORT
    let host_line = format!("HOST={}", host);
    let port_line = format!("PORT={}", port);
    
    if env_content.contains("HOST=") {
        // 替换现有的 HOST
        let lines: Vec<String> = env_content.lines()
            .map(|line| {
                if line.starts_with("HOST=") {
                    host_line.clone()
                } else {
                    line.to_string()
                }
            })
            .collect();
        env_content = lines.join("\n");
    } else {
        // 添加新的 HOST
        if !env_content.is_empty() && !env_content.ends_with('\n') {
            env_content.push('\n');
        }
        env_content.push_str(&host_line);
        env_content.push('\n');
    }
    
    if env_content.contains("PORT=") {
        // 替换现有的 PORT
        let lines: Vec<String> = env_content.lines()
            .map(|line| {
                if line.starts_with("PORT=") {
                    port_line.clone()
                } else {
                    line.to_string()
                }
            })
            .collect();
        env_content = lines.join("\n");
    } else {
        // 添加新的 PORT
        if !env_content.is_empty() && !env_content.ends_with('\n') {
            env_content.push('\n');
        }
        env_content.push_str(&port_line);
        env_content.push('\n');
    }
    
    // 写入文件
    std::fs::write(env_path, env_content)
        .map_err(|e| AppError::BadRequest(format!("写入配置文件失败: {}", e)))?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "配置已保存，请重启应用使其生效",
        "needRestart": true
    })))
}
