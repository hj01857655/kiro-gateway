// WebSearch 工具处理模块
// 实现 Anthropic WebSearch 请求到 Kiro MCP 的转换和响应生成

use axum::{
    extract::Json,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::sync::Arc;
use std::time::SystemTime;
use uuid::Uuid;

use crate::kiro_gate::auth::TokenConfig;
use crate::kiro_gate::logger::emit_log_sync;
use crate::kiro_gate::models::*;
use crate::kiro_gate::server::ServerState;

/// 检查请求是否为纯 WebSearch 请求
pub fn is_web_search_request(request: &AnthropicMessagesRequest) -> bool {
    if request.tools.is_none() {
        return false;
    }

    let tools = request.tools.as_ref().unwrap();
    if tools.len() != 1 {
        return false;
    }

    let tool = &tools[0];
    tool.name == "web_search" || tool.name.starts_with("web_search")
}

/// 从消息中提取搜索查询
pub fn extract_search_query(request: &AnthropicMessagesRequest) -> Option<String> {
    if request.messages.is_empty() {
        return None;
    }

    let first_msg = &request.messages[0];
    let content = &first_msg.content;

    let text = match content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            for block in arr {
                if let Some(obj) = block.as_object() {
                    if obj.get("type").and_then(|v| v.as_str()) == Some("text") {
                        if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                            return Some(text.to_string());
                        }
                    }
                }
            }
            return None;
        }
        _ => return None,
    };

    // 去除前缀 "Perform a web search for the query: "
    let prefix = "Perform a web search for the query: ";
    let query = if text.starts_with(prefix) {
        text[prefix.len()..].to_string()
    } else {
        text
    };

    if query.trim().is_empty() {
        None
    } else {
        Some(query.trim().to_string())
    }
}

/// 生成随机 ID
fn generate_random_id(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// 创建 MCP 请求
fn create_mcp_request(query: &str) -> (String, serde_json::Value) {
    let random_22 = generate_random_id(22);
    let timestamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let random_8 = generate_random_id(8);

    let request_id = format!("web_search_tooluse_{}_{}_{}", random_22, timestamp, random_8);
    let tool_use_id = format!("srvtoolu_{}", Uuid::new_v4().to_string().replace("-", "")[..32].to_string());

    let mcp_request = json!({
        "id": request_id,
        "jsonrpc": "2.0",
        "method": "tools/call",
        "params": {
            "name": "web_search",
            "arguments": {
                "query": query
            }
        }
    });

    (tool_use_id, mcp_request)
}

/// 调用 Kiro MCP API
async fn call_mcp_api(
    state: &ServerState,
    access_token: &str,
    region: &str,
    mcp_request: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let mcp_url = format!("https://q.{}.amazonaws.com/mcp", region);

    let resp = state
        .http_client
        .post(&mcp_url)
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", "application/json")
        .json(mcp_request)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| format!("MCP API 请求失败: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("MCP API 错误: HTTP {} - {}", status, text));
    }

    let result: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("解析 MCP 响应失败: {}", e))?;

    if let Some(error) = result.get("error") {
        let code = error.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
        let message = error.get("message").and_then(|v| v.as_str()).unwrap_or("Unknown error");
        return Err(format!("MCP 错误: {} - {}", code, message));
    }

    Ok(result)
}

/// 解析搜索结果
fn parse_search_results(mcp_response: &serde_json::Value) -> Option<Vec<SearchResult>> {
    let result = mcp_response.get("result")?;
    let content_list = result.get("content")?.as_array()?;
    let first_content = content_list.first()?;

    if first_content.get("type")?.as_str()? != "text" {
        return None;
    }

    let text = first_content.get("text")?.as_str()?;
    let search_data: serde_json::Value = serde_json::from_str(text).ok()?;
    let results = search_data.get("results")?.as_array()?;

    let parsed: Vec<SearchResult> = results
        .iter()
        .filter_map(|r| {
            Some(SearchResult {
                title: r.get("title")?.as_str()?.to_string(),
                url: r.get("url")?.as_str()?.to_string(),
                snippet: r.get("snippet")?.as_str()?.to_string(),
            })
        })
        .collect();

    Some(parsed)
}

#[derive(Debug, Clone)]
struct SearchResult {
    title: String,
    url: String,
    snippet: String,
}

/// 生成搜索结果摘要
fn generate_search_summary(query: &str, results: &[SearchResult]) -> String {
    let mut summary = format!("Here are the search results for \"{}\":\n\n", query);

    if results.is_empty() {
        summary.push_str("No results found.\n");
    } else {
        for (i, result) in results.iter().enumerate() {
            summary.push_str(&format!("{}. **{}**\n", i + 1, result.title));
            
            let snippet = if result.snippet.len() > 200 {
                format!("{}...", &result.snippet[..200])
            } else {
                result.snippet.clone()
            };
            
            if !snippet.is_empty() {
                summary.push_str(&format!("   {}\n", snippet));
            }
            summary.push_str(&format!("   Source: {}\n\n", result.url));
        }
    }

    summary.push_str("\nPlease note that these are web search results and may not be fully accurate or up-to-date.");
    summary
}

/// 估算 token 数量（简单估算）
fn estimate_tokens(text: &str) -> i32 {
    (text.len() / 4) as i32
}

/// 处理 WebSearch 请求
pub async fn handle_web_search_request(
    state: Arc<ServerState>,
    _headers: HeaderMap,
    request: AnthropicMessagesRequest,
    verify_result: super::server::VerifyResult,
) -> Response {
    let _start_time = std::time::Instant::now();
    let model = request.model.clone();

    emit_log_sync("INFO", "websearch", &format!("收到 WebSearch 请求: model={}", model));

    // 1. 提取搜索查询
    let query = match extract_search_query(&request) {
        Some(q) => q,
        None => {
            emit_log_sync("ERROR", "websearch", "无法从消息中提取搜索查询");
            return anthropic_error_response(
                StatusCode::BAD_REQUEST,
                "invalid_request_error",
                "无法从消息中提取搜索查询",
            );
        }
    };

    emit_log_sync("INFO", "websearch", &format!("搜索查询: {}", query));

    // 2. 构建 TokenConfig
    let config = TokenConfig {
        refresh_token: verify_result.refresh_token.clone(),
        auth_method: verify_result.auth_method.clone(),
        profile_arn: verify_result.profile_arn.clone(),
        client_id: verify_result.client_id.clone(),
        client_secret: verify_result.client_secret.clone(),
        region: verify_result.region.clone(),
    };

    // 3. 获取 TokenManager
    let token_manager = state.auth_cache.get_or_create(&verify_result.refresh_token, config).await;

    // 4. 获取 access_token
    let access_token = match token_manager.get_access_token().await {
        Ok(token) => token.to_string(),
        Err(e) => {
            emit_log_sync("ERROR", "websearch", &format!("获取 access_token 失败: {}", e));
            return anthropic_error_response(
                StatusCode::UNAUTHORIZED,
                "authentication_error",
                &e,
            );
        }
    };

    // 5. 创建 MCP 请求
    let (tool_use_id, mcp_request) = create_mcp_request(&query);

    // 6. 调用 Kiro MCP API
    let region = verify_result.region.as_deref().unwrap_or("us-east-1");
    let mcp_response = match call_mcp_api(&state, &access_token, region, &mcp_request).await {
        Ok(resp) => resp,
        Err(e) => {
            emit_log_sync("ERROR", "websearch", &format!("MCP API 调用失败: {}", e));
            return anthropic_error_response(
                StatusCode::BAD_GATEWAY,
                "api_error",
                &format!("MCP API 调用失败: {}", e),
            );
        }
    };

    // 7. 解析搜索结果
    let search_results = parse_search_results(&mcp_response).unwrap_or_default();
    emit_log_sync("INFO", "websearch", &format!("搜索结果数量: {}", search_results.len()));

    // 8. 生成摘要
    let summary = generate_search_summary(&query, &search_results);

    // 9. 估算 tokens
    let input_tokens = estimate_tokens(&format!("{:?}", request.messages));
    let output_tokens = estimate_tokens(&summary);

    // 10. 构建响应
    if request.stream {
        generate_stream_response(model, query, tool_use_id, search_results, summary, input_tokens, output_tokens)
    } else {
        generate_non_stream_response(model, query, tool_use_id, search_results, summary, input_tokens, output_tokens)
    }
}

/// 生成非流式响应
fn generate_non_stream_response(
    model: String,
    query: String,
    tool_use_id: String,
    search_results: Vec<SearchResult>,
    summary: String,
    input_tokens: i32,
    output_tokens: i32,
) -> Response {
    let id = format!("msg_{}", Uuid::new_v4().to_string().replace("-", "")[..24].to_string());

    let mut content_blocks = Vec::new();

    // 1. server_tool_use 块
    content_blocks.push(json!({
        "type": "server_tool_use",
        "id": tool_use_id,
        "name": "web_search",
        "input": { "query": query }
    }));

    // 2. web_search_tool_result 块
    let search_content: Vec<serde_json::Value> = search_results
        .iter()
        .map(|r| {
            json!({
                "type": "web_search_result",
                "title": r.title,
                "url": r.url,
                "encrypted_content": r.snippet,
                "page_age": null
            })
        })
        .collect();

    content_blocks.push(json!({
        "type": "web_search_tool_result",
        "tool_use_id": tool_use_id,
        "content": search_content
    }));

    // 3. text 块
    content_blocks.push(json!({
        "type": "text",
        "text": summary
    }));

    let response = json!({
        "id": id,
        "type": "message",
        "role": "assistant",
        "content": content_blocks,
        "model": model,
        "stop_reason": "end_turn",
        "stop_sequence": null,
        "usage": {
            "input_tokens": input_tokens,
            "output_tokens": output_tokens
        }
    });

    Json(response).into_response()
}

/// 生成流式响应
fn generate_stream_response(
    model: String,
    query: String,
    tool_use_id: String,
    search_results: Vec<SearchResult>,
    summary: String,
    input_tokens: i32,
    output_tokens: i32,
) -> Response {
    use axum::body::Body;
    use axum::http::header;

    let id = format!("msg_{}", Uuid::new_v4().to_string().replace("-", "")[..24].to_string());

    let stream = async_stream::stream! {
        // 1. message_start
        let event = json!({
            "type": "message_start",
            "message": {
                "id": id,
                "type": "message",
                "role": "assistant",
                "content": [],
                "model": model,
                "stop_reason": null,
                "stop_sequence": null,
                "usage": {
                    "input_tokens": input_tokens,
                    "output_tokens": 0,
                    "cache_creation_input_tokens": 0,
                    "cache_read_input_tokens": 0
                }
            }
        });
        yield Ok::<_, std::convert::Infallible>(format!("event: message_start\ndata: {}\n\n", serde_json::to_string(&event).unwrap()));

        // 2. content_block_start (server_tool_use)
        let event = json!({
            "type": "content_block_start",
            "index": 0,
            "content_block": {
                "id": tool_use_id,
                "type": "server_tool_use",
                "name": "web_search",
                "input": {}
            }
        });
        yield Ok(format!("event: content_block_start\ndata: {}\n\n", serde_json::to_string(&event).unwrap()));

        // 3. content_block_delta (input_json_delta)
        let input_json = json!({ "query": query });
        let event = json!({
            "type": "content_block_delta",
            "index": 0,
            "delta": {
                "type": "input_json_delta",
                "partial_json": serde_json::to_string(&input_json).unwrap()
            }
        });
        yield Ok(format!("event: content_block_delta\ndata: {}\n\n", serde_json::to_string(&event).unwrap()));

        // 4. content_block_stop
        let event = json!({ "type": "content_block_stop", "index": 0 });
        yield Ok(format!("event: content_block_stop\ndata: {}\n\n", serde_json::to_string(&event).unwrap()));

        // 5. content_block_start (web_search_tool_result)
        let search_content: Vec<serde_json::Value> = search_results
            .iter()
            .map(|r| {
                json!({
                    "type": "web_search_result",
                    "title": r.title,
                    "url": r.url,
                    "encrypted_content": r.snippet,
                    "page_age": null
                })
            })
            .collect();

        let event = json!({
            "type": "content_block_start",
            "index": 1,
            "content_block": {
                "type": "web_search_tool_result",
                "tool_use_id": tool_use_id,
                "content": search_content
            }
        });
        yield Ok(format!("event: content_block_start\ndata: {}\n\n", serde_json::to_string(&event).unwrap()));

        // 6. content_block_stop
        let event = json!({ "type": "content_block_stop", "index": 1 });
        yield Ok(format!("event: content_block_stop\ndata: {}\n\n", serde_json::to_string(&event).unwrap()));

        // 7. content_block_start (text)
        let event = json!({
            "type": "content_block_start",
            "index": 2,
            "content_block": { "type": "text", "text": "" }
        });
        yield Ok(format!("event: content_block_start\ndata: {}\n\n", serde_json::to_string(&event).unwrap()));

        // 8. content_block_delta (text_delta) - 分块发送
        let chunk_size = 100;
        for chunk in summary.as_bytes().chunks(chunk_size) {
            let text = String::from_utf8_lossy(chunk);
            let event = json!({
                "type": "content_block_delta",
                "index": 2,
                "delta": { "type": "text_delta", "text": text }
            });
            yield Ok(format!("event: content_block_delta\ndata: {}\n\n", serde_json::to_string(&event).unwrap()));
        }

        // 9. content_block_stop
        let event = json!({ "type": "content_block_stop", "index": 2 });
        yield Ok(format!("event: content_block_stop\ndata: {}\n\n", serde_json::to_string(&event).unwrap()));

        // 10. message_delta
        let event = json!({
            "type": "message_delta",
            "delta": { "stop_reason": "end_turn", "stop_sequence": null },
            "usage": { "output_tokens": output_tokens }
        });
        yield Ok(format!("event: message_delta\ndata: {}\n\n", serde_json::to_string(&event).unwrap()));

        // 11. message_stop
        let event = json!({ "type": "message_stop" });
        yield Ok(format!("event: message_stop\ndata: {}\n\n", serde_json::to_string(&event).unwrap()));
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from_stream(stream))
        .unwrap()
}

fn anthropic_error_response(status: StatusCode, error_type: &str, message: &str) -> Response {
    let body = Json(json!({
        "type": "error",
        "error": {
            "type": error_type,
            "message": message
        }
    }));
    (status, body).into_response()
}
