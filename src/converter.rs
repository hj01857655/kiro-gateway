// 格式转换模块
// OpenAI <-> Kiro <-> Anthropic 格式转换

use crate::models::*;
use uuid::Uuid;

/// 提取文本内容
fn extract_text_content(content: &Option<serde_json::Value>) -> String {
    match content {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Array(arr)) => {
            let mut text_parts = Vec::new();
            for item in arr {
                if let Some(obj) = item.as_object() {
                    if obj.get("type").and_then(|v| v.as_str()) == Some("text") {
                        if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                            text_parts.push(text.to_string());
                        }
                    }
                }
            }
            text_parts.join("\n")
        }
        _ => String::new(),
    }
}

/// 获取内部模型 ID
fn get_internal_model_id(model: &str) -> String {
    match model {
        "claude-sonnet-4" => "anthropic.claude-sonnet-4-20250514-v1:0",
        "claude-sonnet-4.5" => "anthropic.claude-sonnet-4.5-20250110-v1:0",
        "claude-opus-4" => "anthropic.claude-opus-4-20250514-v1:0",
        "claude-opus-4.5" => "anthropic.claude-opus-4.5-20250110-v1:0",
        "claude-haiku-4" => "anthropic.claude-haiku-4-20250110-v1:0",
        "claude-haiku-4.5" => "anthropic.claude-haiku-4.5-20250110-v1:0",
        _ => "anthropic.claude-sonnet-4-20250514-v1:0",
    }.to_string()
}

/// OpenAI -> Kiro 转换
pub fn openai_to_kiro(request: &ChatCompletionRequest, profile_arn: &str, _auth_method: &str) -> KiroRequest {
    let conversation_id = Uuid::new_v4().to_string();
    let model_id = get_internal_model_id(&request.model);
    
    // 提取最后一条消息作为当前消息
    let last_message = request.messages.last().unwrap();
    let content = extract_text_content(&last_message.content);
    
    KiroRequest {
        conversation_state: KiroConversationState {
            agent_continuation_id: Uuid::new_v4().to_string(),
            agent_task_type: "vibe".to_string(),
            chat_trigger_type: "MANUAL".to_string(),
            conversation_id,
            current_message: KiroCurrentMessage {
                user_input_message: KiroUserInputMessage {
                    content,
                    model_id,
                    origin: "AI_EDITOR".to_string(),
                    user_input_message_context: None,
                },
            },
            history: None,
        },
        profile_arn: Some(profile_arn.to_string()),
    }
}

/// Anthropic -> Kiro 转换
pub fn anthropic_to_kiro(request: &AnthropicMessagesRequest, profile_arn: &str, _auth_method: &str) -> KiroRequest {
    let conversation_id = Uuid::new_v4().to_string();
    let model_id = get_internal_model_id(&request.model);
    
    // 提取最后一条消息
    let last_message = request.messages.last().unwrap();
    let content = match &last_message.content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            let mut text_parts = Vec::new();
            for item in arr {
                if let Some(obj) = item.as_object() {
                    if obj.get("type").and_then(|v| v.as_str()) == Some("text") {
                        if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                            text_parts.push(text.to_string());
                        }
                    }
                }
            }
            text_parts.join("\n")
        }
        _ => String::new(),
    };
    
    KiroRequest {
        conversation_state: KiroConversationState {
            agent_continuation_id: Uuid::new_v4().to_string(),
            agent_task_type: "vibe".to_string(),
            chat_trigger_type: "MANUAL".to_string(),
            conversation_id,
            current_message: KiroCurrentMessage {
                user_input_message: KiroUserInputMessage {
                    content,
                    model_id,
                    origin: "AI_EDITOR".to_string(),
                    user_input_message_context: None,
                },
            },
            history: None,
        },
        profile_arn: Some(profile_arn.to_string()),
    }
}

/// Kiro -> OpenAI 转换
pub fn kiro_to_openai(event: &KiroEvent, request_id: &str) -> Option<ChatCompletionChunk> {
    if let Some(ref content) = event.content {
        Some(ChatCompletionChunk {
            id: format!("chatcmpl-{}", request_id),
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: "claude-sonnet-4".to_string(),
            choices: vec![ChunkChoice {
                index: 0,
                delta: Delta {
                    role: None,
                    content: Some(content.clone()),
                    tool_calls: None,
                },
                finish_reason: None,
            }],
        })
    } else {
        None
    }
}

/// Kiro -> Anthropic 转换
pub fn kiro_to_anthropic(event: &KiroEvent) -> Option<String> {
    if let Some(ref content) = event.content {
        let delta = serde_json::json!({
            "type": "content_block_delta",
            "index": 0,
            "delta": {
                "type": "text_delta",
                "text": content
            }
        });
        Some(format!("event: content_block_delta\ndata: {}\n\n", serde_json::to_string(&delta).unwrap()))
    } else {
        None
    }
}

/// 创建 OpenAI 结束事件
pub fn create_openai_end_with_reason(
    request_id: &str,
    _has_tool: bool,
    _context_exceeded: bool,
    usage: Option<Usage>,
) -> ChatCompletionChunk {
    ChatCompletionChunk {
        id: format!("chatcmpl-{}", request_id),
        object: "chat.completion.chunk".to_string(),
        created: chrono::Utc::now().timestamp(),
        model: "claude-sonnet-4".to_string(),
        choices: vec![ChunkChoice {
            index: 0,
            delta: Delta {
                role: None,
                content: None,
                tool_calls: None,
            },
            finish_reason: Some("stop".to_string()),
        }],
    }
}

/// 检查是否为流式请求 (OpenAI)
pub fn is_stream_request_openai(request: &ChatCompletionRequest) -> bool {
    request.stream
}

/// 检查是否为流式请求 (Anthropic)
pub fn is_stream_request_anthropic(request: &AnthropicMessagesRequest) -> bool {
    request.stream
}