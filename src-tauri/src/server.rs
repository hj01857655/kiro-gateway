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
use tokio_stream::StreamExt;
use tower_http::cors::CorsLayer;
use tracing::{info, warn};

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
}

pub async fn start_server() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "kiro_gateway=info".to_string()))
        .init();

    let config = AppConfig::from_env();
    let client = KiroClient::new(config.clone());
    let accounts = Arc::new(AccountManager::new());
    let http_client = reqwest::Client::new();
    let auth_cache = crate::auth::AuthCache::new();
    let api_keys = api_key::ApiKeyManager::new();
    
    // 设置默认账号文件路径
    let default_accounts_file = "data/accounts.json";
    
    // 确保 data 目录存在
    if let Err(e) = std::fs::create_dir_all("data") {
        tracing::warn!("创建 data 目录失败: {}", e);
    }
    
    // 加载账号
    if let Some(ref json) = config.accounts_json {
        if let Err(e) = accounts.load_from_json(json) {
            tracing::error!("从环境变量加载账号失败: {}", e);
        }
        // 即使从环境变量加载，也设置默认文件路径用于保存
        accounts.set_accounts_file(default_accounts_file);
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
        // 没有配置任何账号来源，使用默认文件路径
        accounts.set_accounts_file(default_accounts_file);
        // 尝试从默认文件加载
        if let Ok(content) = std::fs::read_to_string(default_accounts_file) {
            if let Err(e) = accounts.load_from_json(&content) {
                tracing::error!("从默认文件 {} 加载账号失败: {}", default_accounts_file, e);
            }
        }
    }

    // 加载 API Keys
    if let Ok(api_keys_file) = std::env::var("API_KEYS_FILE") {
        match std::fs::read_to_string(&api_keys_file) {
            Ok(content) => {
                if let Err(e) = api_keys.load_from_json(&content) {
                    tracing::error!("从文件 {} 加载 API Keys 失败: {}", api_keys_file, e);
                }
            }
            Err(e) => {
                tracing::warn!("读取 API Keys 文件 {} 失败: {}", api_keys_file, e);
            }
        }
        api_keys.set_keys_file(&api_keys_file);
    }
    
    // 创建健康检查器（每 5 分钟检查一次）
    let health_checker = Arc::new(crate::health_checker::HealthChecker::new(Arc::clone(&accounts), 300));

    // 启动健康检查器
    Arc::clone(&health_checker).start();
    info!("账号健康检查器已启动");

    // 注释掉 Metrics 持久化任务，避免触发 Tauri 文件监听导致重启
    // tokio::spawn(async {
    //     let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300));
    //     loop {
    //         interval.tick().await;
    //         if let Err(e) = crate::metrics::METRICS.save_to_file("data/metrics.json") {
    //             tracing::warn!("保存 metrics 数据失败: {}", e);
    //         } else {
    //             tracing::debug!("已保存 metrics 数据");
    //         }
    //     }
    // });
    // info!("Metrics 持久化任务已启动");

    let state = Arc::new(AppState { 
        config: config.clone(), 
        client, 
        accounts,
        http_client,
        auth_cache,
        api_keys,
        health_checker,
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
                .route("/config/apply", post(admin_apply_config))
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
        if let Some(ref admin_key) = config.api_key {
            if key == admin_key {
                return Ok(());
            }
        }
        return api_keys.verify_key(key);
    }
    
    if config.api_key.is_some() {
        return Err(AppError::BadRequest("Missing API key".into()));
    }
    
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
    verify_api_key(&headers, &state.config, &state.api_keys)?;

    let is_stream = is_stream_request_openai(&request);
    let model = request.model.as_str();
    
    let account = state.accounts.get_account().await?;
    // 将空字符串的 profileArn 转换为 None
    let profile_arn = if account.profile_arn.is_empty() {
        None
    } else {
        Some(account.profile_arn.clone())
    };
    let kiro_request = build_kiro_payload(&request, profile_arn)
        .map_err(AppError::BadRequest)?;
    
    let stream = state.client.generate_with_refresh(kiro_request, &state.accounts, model).await?;
    let request_id = uuid::Uuid::new_v4().to_string();

    if is_stream {
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
                if let Some(chunk) = kiro_to_openai(&event, &request_id) {
                    yield Ok::<_, Infallible>(Event::default().data(serde_json::to_string(&chunk).unwrap_or_default()));
                }
            }

            let end = create_openai_end_with_reason(&request_id, has_tool, false, usage);
            yield Ok::<_, Infallible>(Event::default().data(serde_json::to_string(&end).unwrap_or_default()));
            yield Ok::<_, Infallible>(Event::default().data("[DONE]"));
        };
        
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
    verify_api_key(&headers, &state.config, &state.api_keys)?;

    // 检查是否为 WebSearch 请求
    if crate::websearch::is_web_search_request(&request) {
        let account = state.accounts.get_account().await?;
        let verify_result = crate::websearch::VerifyResult {
            refresh_token: account.refresh_token.clone(),
            auth_method: "social".to_string(),
            profile_arn: Some(account.profile_arn.clone()),
            client_id: None,
            client_secret: None,
            region: Some("us-east-1".to_string()),
        };
        return Ok(crate::websearch::handle_web_search_request(
            state,
            headers,
            request,
            verify_result,
        ).await);
    }

    let is_stream = is_stream_request_anthropic(&request);
    let model = request.model.as_str();

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

    let stream = state.client.generate_with_refresh(kiro_request, &state.accounts, model).await?;
    let request_id = uuid::Uuid::new_v4().to_string();

    if is_stream {
        let anthropic_stream = async_stream::stream! {
            tokio::pin!(stream);
            
            yield Ok::<_, Infallible>(Event::default().event("message_start").data(format!(
                r#"{{"type":"message_start","message":{{"id":"msg_{}","type":"message","role":"assistant","content":[],"model":"claude","stop_reason":null}}}}"#,
                request_id
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
        
        Ok(Sse::new(anthropic_stream).into_response())
    } else {
        Err(AppError::BadRequest("非流式响应暂未实现".into()))
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
    // 获取账号
    let accounts = state.accounts.list_accounts();
    let account = accounts.iter()
        .find(|a| a.id == id)
        .ok_or_else(|| AppError::BadRequest(format!("账号 {} 不存在", id)))?;
    
    // 如果 Token 过期，先刷新
    let account = if account.is_expired() {
        info!("账号 {} Token 过期，刷新后查询配额", id);
        state.accounts.refresh_account(&id).await?
    } else {
        account.clone()
    };
    
    // 查询配额
    match state.client.get_usage_limits(&account).await {
        Ok(quota) => Ok(Json(quota)),
        Err(AppError::TokenExpired) => {
            // Token 过期，刷新后重试
            info!("配额查询时 Token 过期，刷新后重试");
            let refreshed = state.accounts.refresh_account(&id).await?;
            let quota = state.client.get_usage_limits(&refreshed).await?;
            Ok(Json(quota))
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
async fn admin_import_accounts(State(_state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, AppError> {
    use std::path::PathBuf;
    
    // 读取 Kiro IDE 缓存文件
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| AppError::BadRequest("无法获取用户目录".into()))?;
    
    let kiro_token_path: PathBuf = [&home, ".aws", "sso", "cache", "kiro-auth-token.json"]
        .iter()
        .collect();
    
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
    let api_key = state.api_keys.generate_key(name)?;
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

// 应用配置到 Claude Desktop
async fn admin_apply_config(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    // 获取 API Key
    let api_key = payload.get("apiKey")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("缺少 apiKey 字段".into()))?;
    
    // 获取基础 URL
    let base_url = format!("http://{}:{}", state.config.host, state.config.port);
    
    // 写入 Claude Desktop 配置
    let config_path = config_generator::write_claude_desktop_config(&base_url, api_key).await
        .map_err(AppError::BadRequest)?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "configPath": config_path,
        "message": "Claude Desktop 配置已成功写入"
    })))
}
