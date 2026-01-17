use axum::{
    Router,
    routing::{get, post},
    extract::State,
    response::{sse::{Event, Sse}, Response, IntoResponse},
    http::HeaderMap,
    Json,
};
use std::sync::Arc;
use std::convert::Infallible;
use tokio_stream::StreamExt;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::account::AccountManager;
use crate::config::AppConfig;
use crate::converter::{build_kiro_payload, anthropic_to_openai, is_stream_request_openai, is_stream_request_anthropic, kiro_to_openai, kiro_to_anthropic, create_openai_end_with_reason};
use crate::error::AppError;
use crate::kiro_client::KiroClient;
use crate::models::{OpenAIRequest, AnthropicRequest, Usage};
use crate::api_key;
use crate::health_checker::HealthChecker;

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
    
    // 加载账号
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
        accounts.set_accounts_file(file);
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

    // 加载账号
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
        accounts.set_accounts_file(file);
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

    // 启动健康检查器
    Arc::clone(&health_checker).start();
    info!("账号健康检查器已启动");

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
        // Admin API
        .route("/admin/accounts", get(admin_get_accounts).post(admin_add_account))
        .route("/admin/accounts/:id", axum::routing::patch(admin_update_account).delete(admin_delete_account))
        .route("/admin/accounts/:id/refresh", post(admin_refresh_account))
        .route("/admin/health", get(admin_get_health).post(admin_check_health))
        .route("/admin/allocator/stats", get(admin_get_allocator_stats))
        .route("/admin/metrics", get(admin_get_metrics))
        .route("/admin/logs", get(admin_get_logs))
        .route("/admin/logs/clear", post(admin_clear_logs))
        .with_state(state)
        .layer(CorsLayer::permissive());

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

async fn chat_completions(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(request): Json<OpenAIRequest>,
) -> Result<Response, AppError> {
    verify_api_key(&headers, &state.config, &state.api_keys)?;

    let is_stream = is_stream_request_openai(&request);
    let model = request.model.as_str();
    
    let account = state.accounts.get_account().await?;
    let kiro_request = build_kiro_payload(&request, Some(account.profile_arn.clone()))
        .map_err(|e| AppError::BadRequest(e))?;
    
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
            yield Ok::<_, Infallible>(Event::default().data("[DONE]".to_string()));
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

    let is_stream = is_stream_request_anthropic(&request);
    let model = request.model.as_str();
    
    let account = state.accounts.get_account().await?;
    let openai_request = anthropic_to_openai(&request);
    let kiro_request = build_kiro_payload(&openai_request, Some(account.profile_arn.clone()))
        .map_err(|e| AppError::BadRequest(e))?;
    
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
                r#"{"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#.to_string()
            ));

            while let Some(Ok(event)) = stream.next().await {
                if let Some(data) = kiro_to_anthropic(&event) {
                    yield Ok::<_, Infallible>(Event::default().event("content_block_delta").data(data));
                }
            }

            yield Ok::<_, Infallible>(Event::default().event("content_block_stop").data(
                r#"{"type":"content_block_stop","index":0}"#.to_string()
            ));

            yield Ok::<_, Infallible>(Event::default().event("message_delta").data(
                r#"{"type":"message_delta","delta":{"stop_reason":"end_turn"}}"#.to_string()
            ));

            yield Ok::<_, Infallible>(Event::default().event("message_stop").data(
                r#"{"type":"message_stop"}"#.to_string()
            ));
        };
        
        Ok(Sse::new(anthropic_stream).into_response())
    } else {
        Err(AppError::BadRequest("非流式响应暂未实现".into()))
    }
}

async fn list_models() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "object": "list",
        "data": [
            {"id": "kiro", "object": "model", "owned_by": "amazon"},
            {"id": "claude-sonnet-4.5", "object": "model", "owned_by": "anthropic"},
        ]
    }))
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
        .map_err(|e| AppError::BadRequest(e))?;
    
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

// 从 Kiro IDE 缓存导入账号（暂未启用）
#[allow(dead_code)]
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
