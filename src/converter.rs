// 格式转换模块
// OpenAI <-> Kiro <-> Anthropic 格式转换

use crate::models::*;

// ============================================================
// 图片处理
// ============================================================

/// 从 data URL 或文件扩展名检测图片格式
/// 参考 Kiro IDE 的 formatFromImageUrl 实现
fn detect_image_format(url: &str) -> String {
  // 优先从 data URL 的 media_type 提取
  if url.starts_with("data:") {
    if let Some(header) = url.split(',').next() {
      if let Some(media_type) = header.strip_prefix("data:").and_then(|s| s.split(';').next()) {
        if let Some(format) = media_type.split('/').nth(1) {
          return format.to_lowercase();
        }
      }
    }
  }
  
  // 回退到扩展名检测 (Kiro IDE 的方式)
  match url.split('.').last().map(|s| s.to_lowercase()).as_deref() {
    Some("png") => "png".to_string(),
    Some("gif") => "gif".to_string(),
    Some("webp") => "webp".to_string(),
    _ => "jpeg".to_string(),  // 默认 JPEG (和 Kiro IDE 一致)
  }
}

/// 从消息内容中提取图片
/// 返回 (images, image_count)
/// 
/// 参考实现:
/// - Kiro IDE: extension.js 的 extractImages 函数
/// - KiroGate: converters.py 的 extract_images_from_content 函数
pub fn extract_images_from_content(content: &Option<serde_json::Value>) -> (Vec<KiroImage>, usize) {
  let content = match content {
    Some(c) => c,
    None => return (Vec::new(), 0),
  };
  
  let content_array = match content.as_array() {
    Some(arr) => arr,
    None => return (Vec::new(), 0),
  };
  
  let mut images = Vec::new();
  
  for item in content_array {
    let obj = match item.as_object() {
      Some(o) => o,
      None => continue,
    };
    
    let item_type = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
    
    // OpenAI 格式: {"type": "image_url", "image_url": {"url": "data:image/png;base64,..."}}
    // 参考 Kiro IDE 的实现
    if item_type == "image_url" {
      // image_url 可能是字符串或对象
      let url = if let Some(url_str) = obj.get("image_url").and_then(|v| v.as_str()) {
        url_str
      } else if let Some(url_obj) = obj.get("image_url").and_then(|v| v.as_object()) {
        url_obj.get("url").and_then(|v| v.as_str()).unwrap_or("")
      } else {
        ""
      };
      
      if url.starts_with("data:") {
        // 解析 data URL: data:image/png;base64,xxxxx
        // Kiro IDE: Buffer.from(imageUrl.split(",")[1], "base64")
        if let Some((_, data)) = url.split_once(',') {
          let format = detect_image_format(url);
          
          images.push(KiroImage {
            format: format.clone(),
            source: KiroImageSource {
              bytes: data.to_string(),
            },
          });
          
          tracing::debug!("[KiroGate] 提取 OpenAI 图片: format={}", format);
        }
      } else {
        tracing::warn!("[KiroGate] 不支持外部图片 URL: {}...", &url[..url.len().min(50)]);
      }
    }
    // Anthropic 格式: {"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "..."}}
    // Kiro IDE 不支持此格式，但我们需要支持
    else if item_type == "image" {
      if let Some(source) = obj.get("source").and_then(|v| v.as_object()) {
        let source_type = source.get("type").and_then(|v| v.as_str()).unwrap_or("");
        
        if source_type == "base64" {
          let media_type = source.get("media_type").and_then(|v| v.as_str()).unwrap_or("image/jpeg");
          let data = source.get("data").and_then(|v| v.as_str()).unwrap_or("");
          
          if !data.is_empty() {
            let format = media_type
              .split('/')
              .nth(1)
              .unwrap_or("jpeg")
              .to_lowercase();
            
            images.push(KiroImage {
              format,
              source: KiroImageSource {
                bytes: data.to_string(),
              },
            });
            
            tracing::debug!("[KiroGate] 提取 Anthropic 图片: format={}", media_type);
          }
        } else if source_type == "url" {
          let url = source.get("url").and_then(|v| v.as_str()).unwrap_or("");
          tracing::warn!("[KiroGate] 不支持图片 URL: {}...", &url[..url.len().min(50)]);
        }
      }
    }
  }
  
  let count = images.len();
  (images, count)
}

// ============================================================
// 模型映射
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
    _usage: Option<Usage>,
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