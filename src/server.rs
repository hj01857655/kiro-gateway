// KiroGate HTTP 服务器

use axum::{
  extract::{Json, State},
  http::{header, HeaderMap, Method, StatusCode},
  response::{IntoResponse, Response},
  routing::{get, post},
  Router,
};
use reqwest::Client;
use serde::Serialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{oneshot, RwLock};
use tower_http::cors::{Any, CorsLayer};

use crate::kiro_gate::auth::{AuthCache, TokenConfig};
use crate::kiro_gate::converter::{build_kiro_payload, get_available_models, anthropic_to_openai};
use crate::kiro_gate::logger::emit_log_sync;
use crate::kiro_gate::metrics::METRICS;
use crate::kiro_gate::models::*;
use crate::commands::kiro_gate_cmd::KiroGateToken;

// ============================================================
// 服务器状态
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct ServerStatus {
  pub running: bool,
  pub port: u16,
  pub url: String,
}

pub struct ServerState {
  pub proxy_api_key: String,
  pub auth_cache: AuthCache,
  pub http_client: Client,
}

// 全局服务器句柄
static SERVER_HANDLE: RwLock<Option<ServerHandle>> = RwLock::const_new(None);

struct ServerHandle {
  shutdown_tx: oneshot::Sender<()>,
  port: u16,
}

// ============================================================
// 公开 API
// ============================================================

/// 启动服务器
pub async fn start_server(port: u16, proxy_api_key: String) -> Result<(), String> {
  // 检查是否已运行
  {
    let handle = SERVER_HANDLE.read().await;
    if handle.is_some() {
      return Err("服务器已在运行".to_string());
    }
  }

  let http_client = Client::builder()
    .timeout(Duration::from_secs(300))
    .build()
    .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

  let state = Arc::new(ServerState {
    proxy_api_key,
    auth_cache: AuthCache::new(),
    http_client,
  });

  let cors = CorsLayer::new()
    .allow_origin(Any)
    .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
    .allow_headers(Any)
    .expose_headers(Any)
    .max_age(Duration::from_secs(3600));

  let app = Router::new()
    .route("/", get(health_handler))
    .route("/health", get(health_handler))
    .route("/v1/models", get(models_handler))
    .route("/v1/chat/completions", post(chat_completions_handler))
    .route("/v1/messages", post(anthropic_messages_handler))
    .route("/messages", post(anthropic_messages_handler)) // Claude Code 兼容
    .route("/metrics", get(metrics_handler))
    .layer(cors)
    .with_state(state);

  let addr = SocketAddr::from(([127, 0, 0, 1], port));
  
  let listener = tokio::net::TcpListener::bind(addr)
    .await
    .map_err(|e| format!("绑定端口 {} 失败: {}", port, e))?;

  let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

  // 保存句柄
  {
    let mut handle = SERVER_HANDLE.write().await;
    *handle = Some(ServerHandle { shutdown_tx, port });
  }

  emit_log_sync("INFO", "server", &format!("服务器已启动: http://127.0.0.1:{}", port));

  // 启动服务器
  tokio::spawn(async move {
    axum::serve(listener, app)
      .with_graceful_shutdown(async {
        let _ = shutdown_rx.await;
      })
      .await
      .ok();
    
    emit_log_sync("INFO", "server", "服务器已停止");
    
    // 清理句柄
    let mut handle = SERVER_HANDLE.write().await;
    *handle = None;
  });

  Ok(())
}

/// 停止服务器
pub async fn stop_server() -> Result<(), String> {
  let mut handle = SERVER_HANDLE.write().await;
  
  if let Some(h) = handle.take() {
    let _ = h.shutdown_tx.send(());
    Ok(())
  } else {
    Err("服务器未运行".to_string())
  }
}

/// 获取服务器状态
pub async fn get_server_status() -> ServerStatus {
  let handle = SERVER_HANDLE.read().await;
  
  if let Some(h) = handle.as_ref() {
    ServerStatus {
      running: true,
      port: h.port,
      url: format!("http://127.0.0.1:{}", h.port),
    }
  } else {
    ServerStatus {
      running: false,
      port: 0,
      url: String::new(),
    }
  }
}

// ============================================================
// 路由处理器
// ============================================================

async fn health_handler() -> impl IntoResponse {
  Json(serde_json::json!({
    "status": "ok",
    "message": "KiroGate is running",
    "version": "1.0.0"
  }))
}

async fn models_handler(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
) -> Response {
  // 验证 API Key
  let verify_result = match verify_api_key(&headers, &state.proxy_api_key) {
    Ok(result) => result,
    Err(_) => {
      // 无 API Key 时返回静态列表
      return Json(ModelsResponse {
        object: "list".to_string(),
        data: get_available_models(),
      }).into_response();
    }
  };

  // 构建 TokenConfig
  let config = TokenConfig {
    refresh_token: verify_result.refresh_token.clone(),
    auth_method: verify_result.auth_method.clone(),
    profile_arn: verify_result.profile_arn.clone(),
    client_id: verify_result.client_id.clone(),
    client_secret: verify_result.client_secret.clone(),
    region: verify_result.region.clone(),
  };

  // 获取 TokenManager
  let token_manager = state.auth_cache.get_or_create(&verify_result.refresh_token, config).await;
  
  // 获取 access_token
  let access_token = match token_manager.get_access_token().await {
    Ok(token) => token,
    Err(_) => {
      // Token 获取失败时返回静态列表
      return Json(ModelsResponse {
        object: "list".to_string(),
        data: get_available_models(),
      }).into_response();
    }
  };

  // 调用 Kiro API 获取模型列表
  let region = verify_result.region.as_deref().unwrap_or("us-east-1");
  let url = format!("https://q.{}.amazonaws.com/ListAvailableModels?origin=AI_EDITOR", region);
  
  let resp = match state.http_client
    .get(&url)
    .header("Authorization", format!("Bearer {}", access_token))
    .timeout(std::time::Duration::from_secs(10))
    .send()
    .await
  {
    Ok(r) => r,
    Err(_) => {
      return Json(ModelsResponse {
        object: "list".to_string(),
        data: get_available_models(),
      }).into_response();
    }
  };

  if !resp.status().is_success() {
    return Json(ModelsResponse {
      object: "list".to_string(),
      data: get_available_models(),
    }).into_response();
  }

  // 解析响应
  let body = match resp.text().await {
    Ok(b) => b,
    Err(_) => {
      return Json(ModelsResponse {
        object: "list".to_string(),
        data: get_available_models(),
      }).into_response();
    }
  };

  // 解析 Kiro 模型列表
  #[derive(serde::Deserialize)]
  #[serde(rename_all = "camelCase")]
  struct KiroModel {
    model_id: String,
  }
  
  #[derive(serde::Deserialize)]
  struct KiroModelsResponse {
    models: Vec<KiroModel>,
  }

  let kiro_models: KiroModelsResponse = match serde_json::from_str(&body) {
    Ok(m) => m,
    Err(_) => {
      return Json(ModelsResponse {
        object: "list".to_string(),
        data: get_available_models(),
      }).into_response();
    }
  };

  // 转换为 OpenAI 格式
  let models: Vec<ModelInfo> = kiro_models.models.iter().map(|m| ModelInfo {
    id: m.model_id.clone(),
    object: "model".to_string(),
    created: 1700000000,
    owned_by: "anthropic".to_string(),
  }).collect();

  Json(ModelsResponse {
    object: "list".to_string(),
    data: models,
  }).into_response()
}

async fn metrics_handler() -> impl IntoResponse {
  Json(METRICS.get_metrics())
}

async fn chat_completions_handler(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(request): Json<ChatCompletionRequest>,
) -> Response {
  let start_time = std::time::Instant::now();
  let model = request.model.clone();
  let is_stream = request.stream;
  
  emit_log_sync("INFO", "openai", &format!("收到请求: model={}, stream={}", model, is_stream));

  // 验证 API Key 并获取完整的 Token 信息
  let verify_result = match verify_api_key(&headers, &state.proxy_api_key) {
    Ok(result) => result,
    Err(e) => {
      emit_log_sync("ERROR", "openai", &format!("认证失败: {}", e));
      METRICS.record_request("/v1/chat/completions", 401, start_time.elapsed().as_millis() as f64, &model, is_stream, "openai");
      return error_response(StatusCode::UNAUTHORIZED, &e);
    }
  };

  // 构建 TokenConfig（直接使用验证结果中的信息）
  let config = TokenConfig {
    refresh_token: verify_result.refresh_token.clone(),
    auth_method: verify_result.auth_method.clone(),
    profile_arn: verify_result.profile_arn.clone(),
    client_id: verify_result.client_id.clone(),
    client_secret: verify_result.client_secret.clone(),
    region: verify_result.region.clone(),
  };

  // 获取 TokenManager
  let token_manager = state.auth_cache.get_or_create(&verify_result.refresh_token, config).await;
  
  // 获取 access_token
  let access_token = match token_manager.get_access_token().await {
    Ok(token) => token,
    Err(e) => {
      METRICS.record_request("/v1/chat/completions", 401, start_time.elapsed().as_millis() as f64, &model, is_stream, "openai");
      return error_response(StatusCode::UNAUTHORIZED, &e);
    }
  };

  let profile_arn = token_manager.get_profile_arn().await;

  // 构建 Kiro payload
  let kiro_payload = match build_kiro_payload(&request, profile_arn) {
    Ok(p) => p,
    Err(e) => {
      METRICS.record_request("/v1/chat/completions", 400, start_time.elapsed().as_millis() as f64, &model, is_stream, "openai");
      return error_response(StatusCode::BAD_REQUEST, &e);
    }
  };

  // 根据 region 选择 API host
  let region = verify_result.region.as_deref().unwrap_or("us-east-1");
  let api_host = format!("https://codewhisperer.{}.amazonaws.com", region);
  let url = format!("{}/generateAssistantResponse", api_host);

  // 发送请求
  let resp = match state.http_client
    .post(&url)
    .header("Authorization", format!("Bearer {}", access_token))
    .header("Content-Type", "application/json")
    .header("Accept", "application/vnd.amazon.eventstream")
    .json(&kiro_payload)
    .send()
    .await
  {
    Ok(r) => r,
    Err(e) => {
      METRICS.record_request("/v1/chat/completions", 502, start_time.elapsed().as_millis() as f64, &model, is_stream, "openai");
      return error_response(StatusCode::BAD_GATEWAY, &format!("请求 Kiro API 失败: {}", e));
    }
  };

  if !resp.status().is_success() {
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    METRICS.record_request("/v1/chat/completions", status.as_u16(), start_time.elapsed().as_millis() as f64, &model, is_stream, "openai");
    return error_response(
      StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
      &format!("Kiro API 错误: {}", text),
    );
  }

  // 记录成功请求
  METRICS.record_request("/v1/chat/completions", 200, start_time.elapsed().as_millis() as f64, &model, is_stream, "openai");
  emit_log_sync("INFO", "openai", &format!("请求成功: model={}, 耗时={}ms", model, start_time.elapsed().as_millis()));

  // 处理响应
  if request.stream {
    stream_response(resp, &request.model).await
  } else {
    non_stream_response(resp, &request.model).await
  }
}

// ============================================================
// Anthropic Messages API 端点
// ============================================================

async fn anthropic_messages_handler(
  State(state): State<Arc<ServerState>>,
  headers: HeaderMap,
  Json(request): Json<AnthropicMessagesRequest>,
) -> Response {
  let start_time = std::time::Instant::now();
  let model = request.model.clone();
  let is_stream = request.stream;
  
  emit_log_sync("INFO", "anthropic", &format!("收到请求: model={}, stream={}", model, is_stream));

  // 验证 API Key（支持 x-api-key 和 Authorization 两种方式）
  let verify_result = match verify_anthropic_api_key(&headers, &state.proxy_api_key) {
    Ok(result) => result,
    Err(e) => {
      emit_log_sync("ERROR", "anthropic", &format!("认证失败: {}", e));
      METRICS.record_request("/v1/messages", 401, start_time.elapsed().as_millis() as f64, &model, is_stream, "anthropic");
      return anthropic_error_response(StatusCode::UNAUTHORIZED, "authentication_error", &e);
    }
  };

  // 检查是否是 WebSearch 请求
  if crate::kiro_gate::websearch::is_web_search_request(&request) {
    emit_log_sync("INFO", "anthropic", "检测到 WebSearch 请求，转发到 WebSearch 处理器");
    return crate::kiro_gate::websearch::handle_web_search_request(state, headers, request, verify_result).await;
  }

  // 构建 TokenConfig
  let config = TokenConfig {
    refresh_token: verify_result.refresh_token.clone(),
    auth_method: verify_result.auth_method.clone(),
    profile_arn: verify_result.profile_arn.clone(),
    client_id: verify_result.client_id.clone(),
    client_secret: verify_result.client_secret.clone(),
    region: verify_result.region.clone(),
  };

  // 获取 TokenManager
  let token_manager = state.auth_cache.get_or_create(&verify_result.refresh_token, config).await;
  
  // 获取 access_token
  let access_token = match token_manager.get_access_token().await {
    Ok(token) => token,
    Err(e) => {
      METRICS.record_request("/v1/messages", 401, start_time.elapsed().as_millis() as f64, &model, is_stream, "anthropic");
      return anthropic_error_response(StatusCode::UNAUTHORIZED, "authentication_error", &e);
    }
  };

  let profile_arn = token_manager.get_profile_arn().await;

  // 转换为 OpenAI 格式，复用现有逻辑
  let openai_request = anthropic_to_openai(&request);

  // 构建 Kiro payload
  let kiro_payload = match build_kiro_payload(&openai_request, profile_arn) {
    Ok(p) => p,
    Err(e) => {
      METRICS.record_request("/v1/messages", 400, start_time.elapsed().as_millis() as f64, &model, is_stream, "anthropic");
      return anthropic_error_response(StatusCode::BAD_REQUEST, "invalid_request_error", &e);
    }
  };

  // 根据 region 选择 API host
  let region = verify_result.region.as_deref().unwrap_or("us-east-1");
  let api_host = format!("https://codewhisperer.{}.amazonaws.com", region);
  let url = format!("{}/generateAssistantResponse", api_host);

  // 发送请求
  let resp = match state.http_client
    .post(&url)
    .header("Authorization", format!("Bearer {}", access_token))
    .header("Content-Type", "application/json")
    .header("Accept", "application/vnd.amazon.eventstream")
    .json(&kiro_payload)
    .send()
    .await
  {
    Ok(r) => r,
    Err(e) => {
      METRICS.record_request("/v1/messages", 502, start_time.elapsed().as_millis() as f64, &model, is_stream, "anthropic");
      return anthropic_error_response(StatusCode::BAD_GATEWAY, "api_error", &format!("请求 Kiro API 失败: {}", e));
    }
  };

  if !resp.status().is_success() {
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    METRICS.record_request("/v1/messages", status.as_u16(), start_time.elapsed().as_millis() as f64, &model, is_stream, "anthropic");
    return anthropic_error_response(
      StatusCode::from_u16(status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
      "api_error",
      &format!("Kiro API 错误: {}", text),
    );
  }

  // 记录成功请求
  METRICS.record_request("/v1/messages", 200, start_time.elapsed().as_millis() as f64, &model, is_stream, "anthropic");
  emit_log_sync("INFO", "anthropic", &format!("请求成功: model={}, 耗时={}ms", model, start_time.elapsed().as_millis()));

  // 处理响应（转换为 Anthropic 格式）
  if request.stream {
    anthropic_stream_response(resp, &request.model).await
  } else {
    anthropic_non_stream_response(resp, &request.model).await
  }
}

// ============================================================
// 辅助函数
// ============================================================

/// 验证结果：包含完整的 Token 信息
pub struct VerifyResult {
  pub refresh_token: String,
  pub auth_method: String,
  pub profile_arn: Option<String>,
  pub client_id: Option<String>,
  pub client_secret: Option<String>,
  pub region: Option<String>,
}

fn verify_api_key(headers: &HeaderMap, proxy_api_key: &str) -> Result<VerifyResult, String> {
  let auth_header = headers
    .get(header::AUTHORIZATION)
    .and_then(|v| v.to_str().ok())
    .ok_or("缺少 Authorization 头")?;

  let token = if auth_header.starts_with("Bearer ") {
    &auth_header[7..]
  } else {
    auth_header
  };

  // 支持多种格式：
  // 1. 多租户 IdC 格式：PROXY_API_KEY|idc|REFRESH_TOKEN|CLIENT_ID|CLIENT_SECRET（用 | 分隔）
  // 2. 多租户 Social 格式：PROXY_API_KEY:REFRESH_TOKEN（用 : 分隔，refresh_token 可能包含冒号）
  // 3. 用户 API Key：sk-{48位十六进制}
  
  // 检查是否是 IdC 格式（用 | 分隔）
  if token.contains('|') {
    let parts: Vec<&str> = token.split('|').collect();
    
    // IdC 格式：PROXY_API_KEY|idc|REFRESH_TOKEN|CLIENT_ID|CLIENT_SECRET
    if parts.len() >= 5 && parts[1].to_lowercase() == "idc" {
      // 验证 PROXY_API_KEY 部分
      if parts[0] != proxy_api_key {
        return Err("API Key 无效".to_string());
      }
      
      return Ok(VerifyResult {
        refresh_token: parts[2].to_string(),
        auth_method: "IdC".to_string(),
        profile_arn: None,
        client_id: Some(parts[3].to_string()),
        client_secret: Some(parts[4..].join("|")), // client_secret 可能包含 |
        region: Some("us-east-1".to_string()),
      });
    }
    
    return Err("API Key 格式无效，IdC 格式应为：PROXY_API_KEY|idc|REFRESH_TOKEN|CLIENT_ID|CLIENT_SECRET".to_string());
  }
  
  // 检查是否包含冒号（Social 多租户格式）
  if token.contains(':') {
    let parts: Vec<&str> = token.splitn(2, ':').collect();
    
    // 验证 PROXY_API_KEY 部分
    if parts[0] != proxy_api_key {
      return Err("API Key 无效".to_string());
    }
    
    // Social 格式：PROXY_API_KEY:REFRESH_TOKEN（refresh_token 可能包含冒号）
    if parts.len() >= 2 {
      return Ok(VerifyResult {
        refresh_token: parts[1].to_string(),
        auth_method: "social".to_string(),
        profile_arn: None,
        client_id: None,
        client_secret: None,
        region: Some("us-east-1".to_string()),
      });
    }
    
    return Err("API Key 格式无效".to_string());
  }
  // 检查传统格式：整个 token 就是 PROXY_API_KEY
  else if token == proxy_api_key {
    Err("传统模式需要服务器配置全局 REFRESH_TOKEN，请使用 PROXY_API_KEY:REFRESH_TOKEN 格式".to_string())
  }
  // 检查用户 API Key 格式：sk-{48位十六进制}
  else if token.starts_with("sk-") && token.len() == 51 {
    // 先检查 Token 池是否为空
    if is_token_pool_empty() {
      return Err("Token 池为空，请先在「Token」页面添加 Token".to_string());
    }
    // 用户 API Key 格式，查找完整的 Token 信息
    match find_token_by_api_key(token) {
      Some(kiro_token) => Ok(VerifyResult {
        refresh_token: kiro_token.refresh_token,
        auth_method: if kiro_token.auth_method.is_empty() { "social".to_string() } else { kiro_token.auth_method },
        profile_arn: kiro_token.profile_arn,
        client_id: kiro_token.client_id,
        client_secret: kiro_token.client_secret,
        region: kiro_token.region.or(Some("us-east-1".to_string())),
      }),
      None => Err("API Key 无效或已过期".to_string()),
    }
  }
  else {
    Err("API Key 格式无效".to_string())
  }
}

// 查找 API Key 对应的完整 Token 信息
fn find_token_by_api_key(api_key: &str) -> Option<KiroGateToken> {
  // 从 API Key 映射表查找
  let path = dirs::data_dir()
    .unwrap_or_else(|| std::path::PathBuf::from("."))
    .join(".kiro-account-manager")
    .join("kirogate-api-keys.json");
  
  if !path.exists() {
    return None;
  }

  let content = std::fs::read_to_string(&path).ok()?;
  
  #[derive(serde::Deserialize)]
  #[serde(rename_all = "camelCase")]
  struct ApiKeyMapping {
    api_key: String,
    token_id: String,
  }
  
  let mappings: Vec<ApiKeyMapping> = serde_json::from_str(&content).ok()?;
  let mapping = mappings.iter().find(|m| m.api_key == api_key)?;
  
  // 根据 token_id 查找完整的 Token 信息
  let tokens_path = dirs::data_dir()
    .unwrap_or_else(|| std::path::PathBuf::from("."))
    .join(".kiro-account-manager")
    .join("kirogate-tokens.json");
  
  let tokens_content = std::fs::read_to_string(&tokens_path).ok()?;
  let tokens: Vec<KiroGateToken> = serde_json::from_str(&tokens_content).ok()?;
  
  tokens.iter().find(|t| t.id == mapping.token_id).cloned()
}

// 检查 Token 池是否为空
fn is_token_pool_empty() -> bool {
  let tokens_path = dirs::data_dir()
    .unwrap_or_else(|| std::path::PathBuf::from("."))
    .join(".kiro-account-manager")
    .join("kirogate-tokens.json");
  
  if !tokens_path.exists() {
    return true;
  }
  
  match std::fs::read_to_string(&tokens_path) {
    Ok(content) => {
      match serde_json::from_str::<Vec<KiroGateToken>>(&content) {
        Ok(tokens) => tokens.is_empty(),
        Err(_) => true,
      }
    }
    Err(_) => true,
  }
}

fn error_response(status: StatusCode, message: &str) -> Response {
  let body = Json(ErrorResponse {
    error: ErrorDetail {
      message: message.to_string(),
      error_type: "api_error".to_string(),
      code: Some(status.as_u16() as i32),
    },
  });
  
  (status, body).into_response()
}

async fn stream_response(resp: reqwest::Response, model: &str) -> Response {
  let model = model.to_string();
  let id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
  let created = chrono::Utc::now().timestamp();

  let stream = async_stream::stream! {
    let mut bytes_stream = resp.bytes_stream();
    let mut buffer = String::new();
    let mut sent_role = false;
    let mut last_content: Option<String> = None; // 去重
    
    use futures::StreamExt;
    
    while let Some(chunk_result) = bytes_stream.next().await {
      match chunk_result {
        Ok(bytes) => {
          buffer.push_str(&String::from_utf8_lossy(&bytes));
          
          // 解析所有 JSON 对象（Kiro 返回的是连续的 JSON，不是 SSE 格式）
          while let Some(start) = buffer.find('{') {
            let remaining = &buffer[start..];
            if let Some(json_str) = extract_json(remaining) {
              let json_len = json_str.len();
              
              // 解析 Kiro 事件
              if let Some(content) = parse_kiro_content(&json_str, &mut last_content) {
                // 发送 role（仅第一次）
                if !sent_role {
                  let chunk = ChatCompletionChunk {
                    id: id.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created,
                    model: model.clone(),
                    choices: vec![ChunkChoice {
                      index: 0,
                      delta: Delta {
                        role: Some("assistant".to_string()),
                        content: None,
                        tool_calls: None,
                      },
                      finish_reason: None,
                    }],
                  };
                  yield Ok::<_, Infallible>(format!("data: {}\n\n", serde_json::to_string(&chunk).unwrap()));
                  sent_role = true;
                }
                
                // 发送内容
                let chunk = ChatCompletionChunk {
                  id: id.clone(),
                  object: "chat.completion.chunk".to_string(),
                  created,
                  model: model.clone(),
                  choices: vec![ChunkChoice {
                    index: 0,
                    delta: Delta {
                      role: None,
                      content: Some(content),
                      tool_calls: None,
                    },
                    finish_reason: None,
                  }],
                };
                yield Ok::<_, Infallible>(format!("data: {}\n\n", serde_json::to_string(&chunk).unwrap()));
              }
              
              // 移除已处理的 JSON
              buffer = buffer[start + json_len..].to_string();
            } else {
              // JSON 不完整，等待更多数据
              break;
            }
          }
        }
        Err(e) => {
          eprintln!("Stream error: {}", e);
          break;
        }
      }
    }
    
    // 发送结束
    let chunk = ChatCompletionChunk {
      id: id.clone(),
      object: "chat.completion.chunk".to_string(),
      created,
      model: model.clone(),
      choices: vec![ChunkChoice {
        index: 0,
        delta: Delta {
          role: None,
          content: None,
          tool_calls: None,
        },
        finish_reason: Some("stop".to_string()),
      }],
    };
    yield Ok::<_, Infallible>(format!("data: {}\n\n", serde_json::to_string(&chunk).unwrap()));
    yield Ok::<_, Infallible>("data: [DONE]\n\n".to_string());
  };

  Response::builder()
    .status(StatusCode::OK)
    .header(header::CONTENT_TYPE, "text/event-stream")
    .header(header::CACHE_CONTROL, "no-cache")
    .body(axum::body::Body::from_stream(stream))
    .unwrap()
}

async fn non_stream_response(resp: reqwest::Response, model: &str) -> Response {
  let id = format!("chatcmpl-{}", uuid::Uuid::new_v4());
  let created = chrono::Utc::now().timestamp();
  
  let bytes = match resp.bytes().await {
    Ok(b) => b,
    Err(e) => return error_response(StatusCode::BAD_GATEWAY, &format!("读取响应失败: {}", e)),
  };

  let text = String::from_utf8_lossy(&bytes);
  let mut content = String::new();
  
  // 解析所有事件（按 JSON 对象提取）
  let mut remaining = text.as_ref();
  while let Some(start) = remaining.find('{') {
    remaining = &remaining[start..];
    if let Some(json_str) = extract_json(remaining) {
      let json_len = json_str.len();
      if let Some(c) = parse_kiro_event(&json_str) {
        content.push_str(&c);
      }
      remaining = &remaining[json_len..];
    } else {
      break;
    }
  }

  let response = ChatCompletionResponse {
    id,
    object: "chat.completion".to_string(),
    created,
    model: model.to_string(),
    choices: vec![Choice {
      index: 0,
      message: ResponseMessage {
        role: "assistant".to_string(),
        content: Some(content),
        tool_calls: None,
      },
      finish_reason: Some("stop".to_string()),
    }],
    usage: None,
  };

  Json(response).into_response()
}

fn parse_kiro_event(event: &str) -> Option<String> {
  parse_kiro_content(event, &mut None)
}

// Kiro 事件解析结果
#[derive(Debug, Clone)]
enum KiroEvent {
  Text(String),
  ToolUseStart { id: String, name: String },
  ToolUseInputDelta { id: String, input_delta: String },
  ToolUseStop { id: String },
  Usage { input_tokens: i32, output_tokens: i32 },
  ContextUsage { percentage: f32 },
}

// 解析 Kiro 事件，返回文本或工具调用
fn parse_kiro_event_full(json_str: &str) -> Option<KiroEvent> {
  let value: serde_json::Value = serde_json::from_str(json_str).ok()?;
  
  // 检查是否是 usage 事件
  if let Some(usage) = value.get("usage").and_then(|v| v.as_object()) {
    let input_tokens = usage.get("inputTokens")
      .or_else(|| usage.get("input_tokens"))
      .and_then(|v| v.as_i64())
      .unwrap_or(0) as i32;
    let output_tokens = usage.get("outputTokens")
      .or_else(|| usage.get("output_tokens"))
      .and_then(|v| v.as_i64())
      .unwrap_or(0) as i32;
    
    if input_tokens > 0 || output_tokens > 0 {
      log::debug!("[KiroGate] Usage: input={}, output={}", input_tokens, output_tokens);
      return Some(KiroEvent::Usage { input_tokens, output_tokens });
    }
  }
  
  // 检查是否是 contextUsagePercentage 事件
  if let Some(percentage) = value.get("contextUsagePercentage").and_then(|v| v.as_f64()) {
    log::debug!("[KiroGate] Context usage: {}%", percentage);
    return Some(KiroEvent::ContextUsage { percentage: percentage as f32 });
  }
  
  // 检查是否是工具调用事件（有 toolUseId 字段）
  if let Some(tool_use_id) = value.get("toolUseId").and_then(|v| v.as_str()) {
    let name = value.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let has_stop = value.get("stop").and_then(|v| v.as_bool()) == Some(true);
    let input_value = value.get("input");
    
    // 优先级：stop > input > start
    // 1. 检查是否是结束事件（有 stop: true）
    if has_stop {
      log::info!("[KiroGate] 工具调用结束: name={}, id={}", name, tool_use_id);
      return Some(KiroEvent::ToolUseStop { id: tool_use_id.to_string() });
    }
    
    // 2. 检查是否有 input 分片（input 字段存在且不为空）
    if let Some(input_val) = input_value {
      // input 可能是字符串或对象
      let input_str = if let Some(s) = input_val.as_str() {
        s.to_string()
      } else if input_val.is_object() {
        serde_json::to_string(input_val).unwrap_or_default()
      } else {
        String::new()
      };
      
      if !input_str.is_empty() {
        log::debug!("[KiroGate] 工具调用 input 分片: name={}, id={}, delta_len={}", name, tool_use_id, input_str.len());
        return Some(KiroEvent::ToolUseInputDelta { 
          id: tool_use_id.to_string(), 
          input_delta: input_str 
        });
      }
    }
    
    // 3. 工具调用开始（只有 name 和 toolUseId，没有 input 和 stop）
    if !name.is_empty() {
      log::info!("[KiroGate] 工具调用开始: name={}, id={}", name, tool_use_id);
      return Some(KiroEvent::ToolUseStart { 
        id: tool_use_id.to_string(), 
        name 
      });
    }
  }
  
  // 检查旧格式：assistantResponseEvent.toolUses
  if let Some(tool_uses) = value.get("assistantResponseEvent")
    .and_then(|e| e.get("toolUses"))
    .and_then(|t| t.as_array())
  {
    if let Some(tool) = tool_uses.first() {
      let id = tool.get("toolUseId").and_then(|v| v.as_str()).unwrap_or("").to_string();
      let name = tool.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
      if !name.is_empty() {
        log::info!("[KiroGate] 检测到完整工具调用: name={}, id={}", name, id);
        return Some(KiroEvent::ToolUseStart { id, name });
      }
    }
  }
  
  // 文本内容解析
  if let Some(text) = parse_text_content(&value) {
    if !text.is_empty() {
      return Some(KiroEvent::Text(text));
    }
  }
  
  None
}

// 从 JSON 值中提取文本内容
fn parse_text_content(value: &serde_json::Value) -> Option<String> {
  // 1. 直接 content 字段
  if let Some(text) = value.get("content").and_then(|c| c.as_str()) {
    if !text.is_empty() {
      return Some(text.to_string());
    }
  }
  
  // 2. delta.text 格式
  if let Some(text) = value.get("delta").and_then(|d| d.get("text")).and_then(|t| t.as_str()) {
    if !text.is_empty() {
      return Some(text.to_string());
    }
  }
  
  // 3. contentBlockDelta 格式
  if let Some(text) = value.get("contentBlockDelta")
    .and_then(|e| e.get("delta"))
    .and_then(|d| d.get("text"))
    .and_then(|t| t.as_str())
  {
    if !text.is_empty() {
      return Some(text.to_string());
    }
  }
  
  // 4. assistantResponseEvent 格式
  if let Some(text) = value.get("assistantResponseEvent")
    .and_then(|e| e.get("content"))
    .and_then(|c| c.as_str())
  {
    if !text.is_empty() {
      return Some(text.to_string());
    }
  }
  
  None
}

// 解析 Kiro 事件内容，带去重（兼容旧接口）
fn parse_kiro_content(json_str: &str, last_content: &mut Option<String>) -> Option<String> {
  let value: serde_json::Value = serde_json::from_str(json_str).ok()?;
  
  if let Some(text) = parse_text_content(&value) {
    // 去重：跳过重复内容
    if last_content.as_deref() == Some(&text) {
      return None;
    }
    *last_content = Some(text.clone());
    return Some(text);
  }
  
  None
}

// 提取完整的 JSON 字符串（处理嵌套大括号）
fn extract_json(s: &str) -> Option<String> {
  if !s.starts_with('{') {
    return None;
  }
  
  let mut brace_count = 0;
  let mut in_string = false;
  let mut escape_next = false;
  
  for (i, c) in s.char_indices() {
    if escape_next {
      escape_next = false;
      continue;
    }
    
    if c == '\\' && in_string {
      escape_next = true;
      continue;
    }
    
    if c == '"' {
      in_string = !in_string;
      continue;
    }
    
    if !in_string {
      if c == '{' {
        brace_count += 1;
      } else if c == '}' {
        brace_count -= 1;
        if brace_count == 0 {
          return Some(s[..=i].to_string());
        }
      }
    }
  }
  
  None
}

// ============================================================
// Anthropic API 辅助函数
// ============================================================

/// 验证 Anthropic API Key（支持 x-api-key 和 Authorization）
fn verify_anthropic_api_key(headers: &HeaderMap, proxy_api_key: &str) -> Result<VerifyResult, String> {
  // 优先检查 x-api-key（Anthropic 标准）
  if let Some(x_api_key) = headers.get("x-api-key").and_then(|v| v.to_str().ok()) {
    return verify_token_string(x_api_key, proxy_api_key);
  }
  
  // 回退到 Authorization（兼容 OpenAI 格式）
  if let Some(auth_header) = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()) {
    let token = if auth_header.starts_with("Bearer ") {
      &auth_header[7..]
    } else {
      auth_header
    };
    return verify_token_string(token, proxy_api_key);
  }
  
  Err("缺少 x-api-key 或 Authorization 头".to_string())
}

/// 验证 token 字符串（复用逻辑）
fn verify_token_string(token: &str, proxy_api_key: &str) -> Result<VerifyResult, String> {
  // 多租户格式：PROXY_API_KEY:REFRESH_TOKEN
  if token.contains(':') {
    let parts: Vec<&str> = token.splitn(2, ':').collect();
    if parts.len() != 2 {
      return Err("API Key 格式无效".to_string());
    }
    if parts[0] != proxy_api_key {
      return Err("API Key 无效".to_string());
    }
    return Ok(VerifyResult {
      refresh_token: parts[1].to_string(),
      auth_method: "social".to_string(),
      profile_arn: None,
      client_id: None,
      client_secret: None,
      region: Some("us-east-1".to_string()),
    });
  }
  
  // 用户 API Key 格式：sk-{48位十六进制}
  if token.starts_with("sk-") && token.len() == 51 {
    // 先检查 Token 池是否为空
    if is_token_pool_empty() {
      return Err("Token 池为空，请先在「Token」页面添加 Token".to_string());
    }
    match find_token_by_api_key(token) {
      Some(kiro_token) => return Ok(VerifyResult {
        refresh_token: kiro_token.refresh_token,
        auth_method: if kiro_token.auth_method.is_empty() { "social".to_string() } else { kiro_token.auth_method },
        profile_arn: kiro_token.profile_arn,
        client_id: kiro_token.client_id,
        client_secret: kiro_token.client_secret,
        region: kiro_token.region.or(Some("us-east-1".to_string())),
      }),
      None => return Err("API Key 无效或已过期".to_string()),
    }
  }
  
  // 传统格式
  if token == proxy_api_key {
    return Err("传统模式需要服务器配置全局 REFRESH_TOKEN".to_string());
  }
  
  Err("API Key 格式无效".to_string())
}

/// Anthropic 格式的错误响应
fn anthropic_error_response(status: StatusCode, error_type: &str, message: &str) -> Response {
  let body = Json(serde_json::json!({
    "type": "error",
    "error": {
      "type": error_type,
      "message": message
    }
  }));
  (status, body).into_response()
}

/// 工具调用去重
/// 参考 Kiro IDE 源码的去重逻辑（extension.js 655267-655279）
/// 按 id 去重，后出现的替换先出现的
fn deduplicate_tool_calls(tool_calls: Vec<(String, String, String)>) -> Vec<(String, String, String)> {
  use std::collections::HashMap;
  
  if tool_calls.is_empty() {
    return tool_calls;
  }
  
  let original_count = tool_calls.len();
  
  // 按 id 去重，后出现的替换先出现的（和 Kiro IDE 一致）
  let mut by_id: HashMap<String, (String, String, String)> = HashMap::new();
  let mut order: Vec<String> = Vec::new();
  
  for (id, name, args) in tool_calls {
    if !by_id.contains_key(&id) {
      order.push(id.clone());
    }
    // 直接替换（后出现的覆盖先出现的）
    by_id.insert(id.clone(), (id, name, args));
  }
  
  // 按原始顺序返回
  let unique: Vec<(String, String, String)> = order
    .into_iter()
    .filter_map(|id| by_id.remove(&id))
    .collect();
  
  if original_count != unique.len() {
    log::info!("[KiroGate] 工具调用去重: {} -> {}", original_count, unique.len());
  }
  
  unique
}

/// Anthropic 非流式响应
async fn anthropic_non_stream_response(resp: reqwest::Response, model: &str) -> Response {
  let id = format!("msg_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..24].to_string());
  
  let bytes = match resp.bytes().await {
    Ok(b) => b,
    Err(e) => return anthropic_error_response(StatusCode::BAD_GATEWAY, "api_error", &format!("读取响应失败: {}", e)),
  };

  let text = String::from_utf8_lossy(&bytes);
  let mut content = String::new();
  
  // 工具调用累积器
  let mut tool_accumulators: std::collections::HashMap<String, (String, String)> = std::collections::HashMap::new();
  let mut completed_tools: Vec<(String, String, String)> = Vec::new();
  
  // usage 统计
  let mut input_tokens = 0i32;
  let mut output_tokens = 0i32;
  
  // 解析所有事件
  let mut remaining = text.as_ref();
  while let Some(start) = remaining.find('{') {
    remaining = &remaining[start..];
    if let Some(json_str) = extract_json(remaining) {
      let json_len = json_str.len();
      
      if let Some(event) = parse_kiro_event_full(&json_str) {
        match event {
          KiroEvent::Text(t) => content.push_str(&t),
          KiroEvent::ToolUseStart { id, name } => {
            tool_accumulators.insert(id, (name, String::new()));
          }
          KiroEvent::ToolUseInputDelta { id, input_delta } => {
            if let Some((_, ref mut input_str)) = tool_accumulators.get_mut(&id) {
              input_str.push_str(&input_delta);
            }
          }
          KiroEvent::ToolUseStop { id } => {
            if let Some((name, input_str)) = tool_accumulators.remove(&id) {
              completed_tools.push((id, name, input_str));
            }
          }
          KiroEvent::Usage { input_tokens: i, output_tokens: o } => {
            input_tokens = i;
            output_tokens = o;
          }
          KiroEvent::ContextUsage { percentage } => {
            // 非流式响应中记录 context usage
            log::debug!("[KiroGate] Context usage: {:.2}%", percentage);
          }
        }
      }
      
      remaining = &remaining[json_len..];
    } else {
      break;
    }
  }
  
  // 工具调用去重
  let completed_tools = deduplicate_tool_calls(completed_tools);

  // 构建 content 数组
  let mut content_blocks: Vec<AnthropicContentBlock> = Vec::new();
  
  // 添加文本块（如果有内容）
  if !content.is_empty() {
    content_blocks.push(AnthropicContentBlock {
      block_type: "text".to_string(),
      text: Some(content),
      id: None,
      name: None,
      input: None,
    });
  }
  
  // 添加工具调用块
  for (tool_id, tool_name, input_json_str) in completed_tools.iter() {
    let tool_input: serde_json::Value = serde_json::from_str(input_json_str)
      .unwrap_or(serde_json::json!({}));
    
    content_blocks.push(AnthropicContentBlock {
      block_type: "tool_use".to_string(),
      text: None,
      id: Some(tool_id.clone()),
      name: Some(tool_name.clone()),
      input: Some(tool_input),
    });
  }
  
  // 确定 stop_reason
  let stop_reason = if !completed_tools.is_empty() {
    Some("tool_use".to_string())
  } else {
    Some("end_turn".to_string())
  };

  let response = AnthropicMessagesResponse {
    id,
    response_type: "message".to_string(),
    role: "assistant".to_string(),
    content: content_blocks,
    model: model.to_string(),
    stop_reason,
    stop_sequence: None,
    usage: AnthropicUsage {
      input_tokens,
      output_tokens,
    },
  };

  Json(response).into_response()
}

/// Anthropic 流式响应
async fn anthropic_stream_response(resp: reqwest::Response, model: &str) -> Response {
  let model = model.to_string();
  let id = format!("msg_{}", uuid::Uuid::new_v4().to_string().replace("-", "")[..24].to_string());

  let stream = async_stream::stream! {
    let mut bytes_stream = resp.bytes_stream();
    let mut buffer = String::new();
    let mut sent_start = false;
    let mut content_index = 0;
    let mut last_content: Option<String> = None;
    
    // Thinking Parser
    let mut thinking_parser = crate::kiro_gate::thinking_parser::ThinkingParser::new();
    let mut thinking_index: Option<usize> = None;
    
    // 工具调用累积器: HashMap<tool_use_id, (name, input_json_string)>
    let mut tool_accumulators: std::collections::HashMap<String, (String, String)> = std::collections::HashMap::new();
    let mut completed_tools: Vec<(String, String, String)> = Vec::new(); // (id, name, input_json)
    
    // usage 统计
    let mut input_tokens = 0i32;
    let mut output_tokens = 0i32;
    
    use futures::StreamExt;
    
    while let Some(chunk_result) = bytes_stream.next().await {
      match chunk_result {
        Ok(bytes) => {
          buffer.push_str(&String::from_utf8_lossy(&bytes));
          
          while let Some(start) = buffer.find('{') {
            let remaining = &buffer[start..];
            if let Some(json_str) = extract_json(remaining) {
              let json_len = json_str.len();
              
              // 使用完整解析器
              if let Some(event) = parse_kiro_event_full(&json_str) {
                match event {
                  KiroEvent::Text(content) => {
                    // 去重
                    if last_content.as_deref() == Some(&content) {
                      buffer = buffer[start + json_len..].to_string();
                      continue;
                    }
                    last_content = Some(content.clone());
                    
                    // 使用 Thinking Parser 解析内容
                    let segments = thinking_parser.push_and_parse(&content);
                    
                    for segment in segments {
                      match segment.segment_type {
                        crate::kiro_gate::thinking_parser::SegmentType::Thinking => {
                          // 发送 message_start（如果还没发送）
                          if !sent_start {
                            let event = serde_json::json!({
                              "type": "message_start",
                              "message": {
                                "id": id,
                                "type": "message",
                                "role": "assistant",
                                "content": [],
                                "model": model,
                                "stop_reason": null,
                                "stop_sequence": null,
                                "usage": { "input_tokens": input_tokens, "output_tokens": output_tokens }
                              }
                            });
                            yield Ok::<_, Infallible>(format!("event: message_start\ndata: {}\n\n", serde_json::to_string(&event).unwrap()));
                            sent_start = true;
                          }
                          
                          // 发送 thinking 块（如果还没开始）
                          if thinking_index.is_none() {
                            let block_start = serde_json::json!({
                              "type": "content_block_start",
                              "index": content_index,
                              "content_block": { "type": "thinking", "thinking": "" }
                            });
                            yield Ok::<_, Infallible>(format!("event: content_block_start\ndata: {}\n\n", serde_json::to_string(&block_start).unwrap()));
                            thinking_index = Some(content_index);
                            content_index += 1;
                          }
                          
                          // 发送 thinking delta
                          let delta = serde_json::json!({
                            "type": "content_block_delta",
                            "index": thinking_index.unwrap(),
                            "delta": { "type": "thinking_delta", "thinking": segment.content }
                          });
                          yield Ok::<_, Infallible>(format!("event: content_block_delta\ndata: {}\n\n", serde_json::to_string(&delta).unwrap()));
                        }
                        crate::kiro_gate::thinking_parser::SegmentType::Text => {
                          // 关闭 thinking 块（如果有）
                          if let Some(idx) = thinking_index.take() {
                            let block_stop = serde_json::json!({
                              "type": "content_block_stop",
                              "index": idx
                            });
                            yield Ok::<_, Infallible>(format!("event: content_block_stop\ndata: {}\n\n", serde_json::to_string(&block_stop).unwrap()));
                          }
                          
                          // 发送 message_start（如果还没发送）
                          if !sent_start {
                            let event = serde_json::json!({
                              "type": "message_start",
                              "message": {
                                "id": id,
                                "type": "message",
                                "role": "assistant",
                                "content": [],
                                "model": model,
                                "stop_reason": null,
                                "stop_sequence": null,
                                "usage": { "input_tokens": input_tokens, "output_tokens": output_tokens }
                              }
                            });
                            yield Ok::<_, Infallible>(format!("event: message_start\ndata: {}\n\n", serde_json::to_string(&event).unwrap()));
                            
                            // 发送 content_block_start
                            let block_start = serde_json::json!({
                              "type": "content_block_start",
                              "index": content_index,
                              "content_block": { "type": "text", "text": "" }
                            });
                            yield Ok::<_, Infallible>(format!("event: content_block_start\ndata: {}\n\n", serde_json::to_string(&block_start).unwrap()));
                            
                            sent_start = true;
                          }
                          
                          // 发送 text delta
                          let delta = serde_json::json!({
                            "type": "content_block_delta",
                            "index": content_index,
                            "delta": { "type": "text_delta", "text": segment.content }
                          });
                          yield Ok::<_, Infallible>(format!("event: content_block_delta\ndata: {}\n\n", serde_json::to_string(&delta).unwrap()));
                        }
                      }
                    }
                  }
                  KiroEvent::ToolUseStart { id: tool_id, name } => {
                    // 开始累积新的工具调用
                    tool_accumulators.insert(tool_id, (name, String::new()));
                  }
                  KiroEvent::ToolUseInputDelta { id: tool_id, input_delta } => {
                    // 累积 input 分片
                    if let Some((_, ref mut input_str)) = tool_accumulators.get_mut(&tool_id) {
                      input_str.push_str(&input_delta);
                    }
                  }
                  KiroEvent::ToolUseStop { id: tool_id } => {
                    // 工具调用完成，移到完成列表
                    if let Some((name, input_str)) = tool_accumulators.remove(&tool_id) {
                      // 安全截取 UTF-8 字符串（避免在多字节字符中间截断）
                      let preview: String = input_str.chars().take(100).collect();
                      log::info!("[KiroGate] 工具调用完成: name={}, input={}", name, preview);
                      completed_tools.push((tool_id, name, input_str));
                    }
                  }
                  KiroEvent::Usage { input_tokens: i, output_tokens: o } => {
                    input_tokens = i;
                    output_tokens = o;
                  }
                  KiroEvent::ContextUsage { percentage } => {
                    // 发送 context usage 事件（自定义扩展）
                    let ctx_event = serde_json::json!({
                      "type": "context_usage",
                      "percentage": percentage
                    });
                    yield Ok::<_, Infallible>(format!("event: context_usage\ndata: {}\n\n", serde_json::to_string(&ctx_event).unwrap()));
                  }
                }
              }
              
              buffer = buffer[start + json_len..].to_string();
            } else {
              break;
            }
          }
        }
        Err(e) => {
          eprintln!("Stream error: {}", e);
          break;
        }
      }
    }
    
    // 刷新 Thinking Parser 缓冲区
    let final_segments = thinking_parser.flush();
    for segment in final_segments {
      match segment.segment_type {
        crate::kiro_gate::thinking_parser::SegmentType::Thinking => {
          if thinking_index.is_none() {
            // 开始 thinking 块
            if !sent_start {
              let event = serde_json::json!({
                "type": "message_start",
                "message": {
                  "id": id,
                  "type": "message",
                  "role": "assistant",
                  "content": [],
                  "model": model,
                  "stop_reason": null,
                  "stop_sequence": null,
                  "usage": { "input_tokens": input_tokens, "output_tokens": output_tokens }
                }
              });
              yield Ok::<_, Infallible>(format!("event: message_start\ndata: {}\n\n", serde_json::to_string(&event).unwrap()));
              sent_start = true;
            }
            
            let block_start = serde_json::json!({
              "type": "content_block_start",
              "index": content_index,
              "content_block": { "type": "thinking", "thinking": "" }
            });
            yield Ok::<_, Infallible>(format!("event: content_block_start\ndata: {}\n\n", serde_json::to_string(&block_start).unwrap()));
            thinking_index = Some(content_index);
            content_index += 1;
          }
          
          let delta = serde_json::json!({
            "type": "content_block_delta",
            "index": thinking_index.unwrap(),
            "delta": { "type": "thinking_delta", "thinking": segment.content }
          });
          yield Ok::<_, Infallible>(format!("event: content_block_delta\ndata: {}\n\n", serde_json::to_string(&delta).unwrap()));
        }
        crate::kiro_gate::thinking_parser::SegmentType::Text => {
          if let Some(idx) = thinking_index.take() {
            let block_stop = serde_json::json!({
              "type": "content_block_stop",
              "index": idx
            });
            yield Ok::<_, Infallible>(format!("event: content_block_stop\ndata: {}\n\n", serde_json::to_string(&block_stop).unwrap()));
          }
          
          if !sent_start {
            let event = serde_json::json!({
              "type": "message_start",
              "message": {
                "id": id,
                "type": "message",
                "role": "assistant",
                "content": [],
                "model": model,
                "stop_reason": null,
                "stop_sequence": null,
                "usage": { "input_tokens": input_tokens, "output_tokens": output_tokens }
              }
            });
            yield Ok::<_, Infallible>(format!("event: message_start\ndata: {}\n\n", serde_json::to_string(&event).unwrap()));
            
            let block_start = serde_json::json!({
              "type": "content_block_start",
              "index": content_index,
              "content_block": { "type": "text", "text": "" }
            });
            yield Ok::<_, Infallible>(format!("event: content_block_start\ndata: {}\n\n", serde_json::to_string(&block_start).unwrap()));
            
            sent_start = true;
          }
          
          let delta = serde_json::json!({
            "type": "content_block_delta",
            "index": content_index,
            "delta": { "type": "text_delta", "text": segment.content }
          });
          yield Ok::<_, Infallible>(format!("event: content_block_delta\ndata: {}\n\n", serde_json::to_string(&delta).unwrap()));
        }
      }
    }
    
    // 发送结束事件
    // 关闭 thinking 块（如果还在打开状态）
    if let Some(idx) = thinking_index {
      let block_stop = serde_json::json!({
        "type": "content_block_stop",
        "index": idx
      });
      yield Ok::<_, Infallible>(format!("event: content_block_stop\ndata: {}\n\n", serde_json::to_string(&block_stop).unwrap()));
    }
    
    if sent_start {
      // 关闭文本块
      let block_stop = serde_json::json!({
        "type": "content_block_stop",
        "index": content_index
      });
      yield Ok::<_, Infallible>(format!("event: content_block_stop\ndata: {}\n\n", serde_json::to_string(&block_stop).unwrap()));
      content_index += 1;
    } else if !completed_tools.is_empty() {
      // 如果没有文本但有工具调用，先发送 message_start
      let event = serde_json::json!({
        "type": "message_start",
        "message": {
          "id": id,
          "type": "message",
          "role": "assistant",
          "content": [],
          "model": model,
          "stop_reason": null,
          "stop_sequence": null,
          "usage": { "input_tokens": 0, "output_tokens": 0 }
        }
      });
      yield Ok::<_, Infallible>(format!("event: message_start\ndata: {}\n\n", serde_json::to_string(&event).unwrap()));
      sent_start = true;
    }
    
    // 工具调用去重
    let completed_tools = deduplicate_tool_calls(completed_tools);
    
    // 发送工具调用块
    for (tool_id, tool_name, input_json_str) in &completed_tools {
      // 解析 input JSON
      let tool_input: serde_json::Value = serde_json::from_str(input_json_str)
        .unwrap_or(serde_json::json!({}));
      
      // content_block_start
      let tool_block_start = serde_json::json!({
        "type": "content_block_start",
        "index": content_index,
        "content_block": {
          "type": "tool_use",
          "id": tool_id,
          "name": tool_name,
          "input": {}
        }
      });
      yield Ok::<_, Infallible>(format!("event: content_block_start\ndata: {}\n\n", serde_json::to_string(&tool_block_start).unwrap()));
      
      // input_json_delta
      let input_delta = serde_json::json!({
        "type": "content_block_delta",
        "index": content_index,
        "delta": {
          "type": "input_json_delta",
          "partial_json": serde_json::to_string(&tool_input).unwrap_or_default()
        }
      });
      yield Ok::<_, Infallible>(format!("event: content_block_delta\ndata: {}\n\n", serde_json::to_string(&input_delta).unwrap()));
      
      // content_block_stop
      let tool_block_stop = serde_json::json!({
        "type": "content_block_stop",
        "index": content_index
      });
      yield Ok::<_, Infallible>(format!("event: content_block_stop\ndata: {}\n\n", serde_json::to_string(&tool_block_stop).unwrap()));
      
      content_index += 1;
    }
    
    // 确定 stop_reason
    let stop_reason = if !completed_tools.is_empty() { "tool_use" } else { "end_turn" };
    
    if sent_start {
      // message_delta
      let msg_delta = serde_json::json!({
        "type": "message_delta",
        "delta": { "stop_reason": stop_reason, "stop_sequence": null },
        "usage": { "output_tokens": output_tokens }
      });
      yield Ok::<_, Infallible>(format!("event: message_delta\ndata: {}\n\n", serde_json::to_string(&msg_delta).unwrap()));
      
      // message_stop
      let msg_stop = serde_json::json!({ "type": "message_stop" });
      yield Ok::<_, Infallible>(format!("event: message_stop\ndata: {}\n\n", serde_json::to_string(&msg_stop).unwrap()));
    }
  };

  Response::builder()
    .status(StatusCode::OK)
    .header(header::CONTENT_TYPE, "text/event-stream")
    .header(header::CACHE_CONTROL, "no-cache")
    .body(axum::body::Body::from_stream(stream))
    .unwrap()
}
