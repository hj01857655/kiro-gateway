use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::models::*;

const TOOL_DESCRIPTION_MAX_LENGTH: usize = 4000;

// 模型映射
pub fn map_model(model: &str) -> String {
    let model_lower = model.to_lowercase();
    let kiro_model = match model_lower.as_str() {
        // OpenAI → Kiro
        "gpt-4" | "gpt-4-turbo" | "gpt-4o" => "claude-sonnet-4.5",
        "gpt-3.5-turbo" => "claude-haiku-4.5",
        // Anthropic → Kiro
        "claude-3-5-sonnet-20241022" | "claude-3-5-sonnet" => "claude-sonnet-4.5",
        "claude-3-opus-20240229" | "claude-3-opus" => "claude-opus-4.5",
        "claude-3-haiku-20240307" | "claude-3-haiku" => "claude-haiku-4.5",
        // 直接使用 Kiro 模型名
        "claude-sonnet-4.5" => "claude-sonnet-4.5",
        "claude-sonnet-4" => "claude-sonnet-4",
        "claude-opus-4.5" => "claude-opus-4.5",
        "claude-haiku-4.5" => "claude-haiku-4.5",
        "auto" | "kiro" => "auto",
        // 模糊匹配
        _ => {
            if model_lower.contains("opus") {
                "claude-opus-4.5"
            } else if model_lower.contains("haiku") {
                "claude-haiku-4.5"
            } else if model_lower.contains("sonnet") {
                "claude-sonnet-4.5"
            } else {
                "auto"
            }
        }
    };
    // Kiro API 不需要 qdev:: 前缀！
    kiro_model.to_string()
}

/// 合并 OpenAI 消息（Kiro 不接受连续相同 role 的消息）
fn merge_openai_messages(messages: &[&OpenAIMessage]) -> Vec<OpenAIMessage> {
    if messages.is_empty() {
        return vec![];
    }
    
    let mut result: Vec<OpenAIMessage> = vec![];
    
    for msg in messages {
        if let Some(last) = result.last_mut() {
            if last.role == msg.role && msg.role != "tool" {
                // 合并文本内容
                let last_text = last.content.as_ref().map(|c| extract_text_content(c)).unwrap_or_default();
                let msg_text = msg.content.as_ref().map(|c| extract_text_content(c)).unwrap_or_default();
                if !msg_text.is_empty() {
                    last.content = Some(MessageContent::Text(format!("{}\n{}", last_text, msg_text)));
                }
                
                // 合并 tool_calls
                if let Some(ref new_calls) = msg.tool_calls {
                    if let Some(ref mut existing) = last.tool_calls {
                        existing.extend(new_calls.clone());
                    } else {
                        last.tool_calls = Some(new_calls.clone());
                    }
                }
                continue;
            }
        }
        result.push((*msg).clone());
    }
    
    result
}

/// 合并 Anthropic 消息
fn merge_anthropic_messages(messages: &[AnthropicMessage]) -> Vec<AnthropicMessage> {
    if messages.is_empty() {
        return vec![];
    }
    
    let mut result: Vec<AnthropicMessage> = vec![];
    
    for msg in messages {
        if let Some(last) = result.last_mut() {
            if last.role == msg.role {
                let last_text = extract_anthropic_text(&last.content);
                let msg_text = extract_anthropic_text(&msg.content);
                if !msg_text.is_empty() {
                    last.content = AnthropicContent::Text(format!("{}\n{}", last_text, msg_text));
                }
                continue;
            }
        }
        result.push(msg.clone());
    }
    
    result
}

// OpenAI 请求转 Kiro 请求
pub fn openai_to_kiro(request: &OpenAIRequest, profile_arn: &str, auth_method: &str) -> KiroRequest {
    let system_prompt = request.messages.iter()
        .find(|m| m.role == "system")
        .and_then(|m| m.content.as_ref())
        .map(|c| extract_text_content(c));

    let chat_messages: Vec<_> = request.messages.iter()
        .filter(|m| m.role != "system")
        .collect();

    // 合并相邻同 role 消息
    let merged_messages = merge_openai_messages(&chat_messages);

    let last_user_idx = merged_messages.iter()
        .rposition(|m| m.role == "user")
        .unwrap_or(merged_messages.len().saturating_sub(1));

    let current_content = merged_messages.get(last_user_idx)
        .and_then(|m| m.content.as_ref())
        .map(|c| extract_text_content(c))
        .unwrap_or_default();

    let images = merged_messages.get(last_user_idx)
        .and_then(|m| m.content.as_ref())
        .map(|c| extract_images(c))
        .filter(|v| !v.is_empty());

    let history_refs: Vec<_> = merged_messages[..last_user_idx].iter().collect();
    let history = build_history(&history_refs);

    let tools = request.tools.as_ref().map(|t| convert_openai_tools(t));
    let model_id = request.model.as_ref().map(|m| map_model(m));

    // 处理长 tool description（注意：Kiro 不支持 additionalContext，长描述移到 system prompt 后无法使用）
    let (tools, _system_prompt) = process_long_tool_descriptions(tools, system_prompt);

    // userInputMessageContext 只在有 tools 时才添加
    let user_input_message_context = if !tools.is_empty() {
        Some(UserInputMessageContext {
            tools: Some(tools),
            tool_results: None,
        })
    } else {
        None
    };

    KiroRequest {
        conversation_state: ConversationState {
            conversation_id: uuid::Uuid::new_v4().to_string(),
            chat_trigger_type: "MANUAL".to_string(),
            current_message: CurrentMessage {
                user_input_message: UserInputMessage {
                    content: current_content,
                    model_id,
                    origin: "AI_EDITOR".to_string(),
                    user_input_message_context,
                    images,
                },
            },
            history,
        },
        profile_arn: if auth_method == "social" && !profile_arn.is_empty() {
            Some(profile_arn.to_string())
        } else {
            None
        },
    }
}

/// 检查是否为流式请求
pub fn is_stream_request_openai(request: &OpenAIRequest) -> bool {
    request.stream
}

/// 检查是否为流式请求（Anthropic）
pub fn is_stream_request_anthropic(request: &AnthropicRequest) -> bool {
    request.stream
}

// Anthropic 请求转 Kiro 请求
pub fn anthropic_to_kiro(request: &AnthropicRequest, profile_arn: &str, auth_method: &str) -> KiroRequest {
    let system_prompt = request.system.clone();

    // 合并相邻同 role 消息
    let merged_messages = merge_anthropic_messages(&request.messages);

    let last_user_idx = merged_messages.iter()
        .rposition(|m| m.role == "user")
        .unwrap_or(merged_messages.len().saturating_sub(1));

    let current_content = merged_messages.get(last_user_idx)
        .map(|m| extract_anthropic_text(&m.content))
        .unwrap_or_default();

    let images = merged_messages.get(last_user_idx)
        .map(|m| extract_anthropic_images(&m.content))
        .filter(|v| !v.is_empty());

    let history = build_anthropic_history(&merged_messages[..last_user_idx]);
    let tools = request.tools.as_ref().map(|t| convert_anthropic_tools(t));
    let model_id = request.model.as_ref().map(|m| map_model(m));

    // 处理长 tool description（注意：Kiro 不支持 additionalContext，长描述移到 system prompt 后无法使用）
    let (tools, _system_prompt) = process_long_tool_descriptions(tools, system_prompt);

    // userInputMessageContext 只在有 tools 时才添加
    let user_input_message_context = if !tools.is_empty() {
        Some(UserInputMessageContext {
            tools: Some(tools),
            tool_results: None,
        })
    } else {
        None
    };

    KiroRequest {
        conversation_state: ConversationState {
            conversation_id: uuid::Uuid::new_v4().to_string(),
            chat_trigger_type: "MANUAL".to_string(),
            current_message: CurrentMessage {
                user_input_message: UserInputMessage {
                    content: current_content,
                    model_id,
                    origin: "AI_EDITOR".to_string(),
                    user_input_message_context,
                    images,
                },
            },
            history,
        },
        profile_arn: if auth_method == "social" && !profile_arn.is_empty() {
            Some(profile_arn.to_string())
        } else {
            None
        },
    }
}

fn extract_text_content(content: &MessageContent) -> String {
    match content {
        MessageContent::Text(s) => s.clone(),
        MessageContent::Parts(parts) => {
            parts.iter()
                .filter_map(|p| match p {
                    ContentPart::Text { text } => Some(text.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

fn extract_images(content: &MessageContent) -> Vec<KiroImage> {
    match content {
        MessageContent::Parts(parts) => {
            parts.iter()
                .filter_map(|p| match p {
                    ContentPart::ImageUrl { image_url } => parse_data_url(&image_url.url),
                    _ => None,
                })
                .collect()
        }
        _ => vec![],
    }
}

fn parse_data_url(url: &str) -> Option<KiroImage> {
    if !url.starts_with("data:image/") {
        return None;
    }
    let parts: Vec<_> = url.splitn(2, ',').collect();
    if parts.len() != 2 {
        return None;
    }
    let format = parts[0].strip_prefix("data:image/")?.split(';').next()?.to_string();
    Some(KiroImage {
        image: KiroImageData {
            format,
            source: KiroImageSource { bytes: parts[1].to_string() },
        },
    })
}

fn extract_anthropic_text(content: &AnthropicContent) -> String {
    match content {
        AnthropicContent::Text(s) => s.clone(),
        AnthropicContent::Blocks(blocks) => {
            blocks.iter()
                .filter_map(|b| match b {
                    AnthropicBlock::Text { text } => Some(text.clone()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
}

fn extract_anthropic_images(content: &AnthropicContent) -> Vec<KiroImage> {
    match content {
        AnthropicContent::Blocks(blocks) => {
            blocks.iter()
                .filter_map(|b| match b {
                    AnthropicBlock::Image { source } => {
                        let format = source.media_type.split('/').last().unwrap_or("png").to_string();
                        Some(KiroImage {
                            image: KiroImageData {
                                format,
                                source: KiroImageSource { bytes: source.data.clone() },
                            },
                        })
                    }
                    _ => None,
                })
                .collect()
        }
        _ => vec![],
    }
}

fn build_history(messages: &[&OpenAIMessage]) -> Vec<HistoryMessage> {
    let mut history = Vec::new();

    for msg in messages {
        match msg.role.as_str() {
            "user" => {
                let content = msg.content.as_ref().map(|c| extract_text_content(c)).unwrap_or_default();
                history.push(HistoryMessage::User(UserHistoryMessage {
                    user_input_message: UserHistoryContent {
                        content,
                        user_intent: "CODE_GENERATION".to_string(),
                    },
                }));
            }
            "assistant" => {
                let content = msg.content.as_ref().map(|c| extract_text_content(c)).unwrap_or_default();
                let tool_use = msg.tool_calls.as_ref().map(|calls| {
                    calls.iter().map(|c| KiroToolUse {
                        tool_use_id: c.id.clone(),
                        name: c.function.name.clone(),
                        input: serde_json::from_str(&c.function.arguments).unwrap_or(json!({})),
                    }).collect()
                });
                history.push(HistoryMessage::Assistant(AssistantHistoryMessage {
                    assistant_response_message: AssistantHistoryContent { content, tool_use },
                }));
            }
            "tool" => {
                if let Some(tool_call_id) = &msg.tool_call_id {
                    let content = msg.content.as_ref().map(|c| extract_text_content(c)).unwrap_or_default();
                    history.push(HistoryMessage::ToolResult(ToolResultMessage {
                        tool_result: ToolResultContent {
                            tool_use_id: tool_call_id.clone(),
                            status: "success".to_string(),
                            content: vec![TextContent { text: content }],
                        },
                    }));
                }
            }
            _ => {}
        }
    }

    history
}

fn build_anthropic_history(messages: &[AnthropicMessage]) -> Vec<HistoryMessage> {
    let mut history = Vec::new();

    for msg in messages {
        match msg.role.as_str() {
            "user" => {
                if let AnthropicContent::Blocks(blocks) = &msg.content {
                    for block in blocks {
                        if let AnthropicBlock::ToolResult { tool_use_id, content } = block {
                            history.push(HistoryMessage::ToolResult(ToolResultMessage {
                                tool_result: ToolResultContent {
                                    tool_use_id: tool_use_id.clone(),
                                    status: "success".to_string(),
                                    content: vec![TextContent { text: content.clone() }],
                                },
                            }));
                        }
                    }
                    let has_text = blocks.iter().any(|b| matches!(b, AnthropicBlock::Text { .. }));
                    if !has_text { continue; }
                }
                let content = extract_anthropic_text(&msg.content);
                history.push(HistoryMessage::User(UserHistoryMessage {
                    user_input_message: UserHistoryContent {
                        content,
                        user_intent: "CODE_GENERATION".to_string(),
                    },
                }));
            }
            "assistant" => {
                let mut content = String::new();
                let mut tool_use = Vec::new();

                if let AnthropicContent::Blocks(blocks) = &msg.content {
                    for block in blocks {
                        match block {
                            AnthropicBlock::Text { text } => {
                                if !content.is_empty() { content.push('\n'); }
                                content.push_str(text);
                            }
                            AnthropicBlock::ToolUse { id, name, input } => {
                                tool_use.push(KiroToolUse {
                                    tool_use_id: id.clone(),
                                    name: name.clone(),
                                    input: input.clone(),
                                });
                            }
                            _ => {}
                        }
                    }
                } else {
                    content = extract_anthropic_text(&msg.content);
                }

                history.push(HistoryMessage::Assistant(AssistantHistoryMessage {
                    assistant_response_message: AssistantHistoryContent {
                        content,
                        tool_use: if tool_use.is_empty() { None } else { Some(tool_use) },
                    },
                }));
            }
            _ => {}
        }
    }

    history
}

fn convert_openai_tools(tools: &[OpenAITool]) -> Vec<KiroTool> {
    tools.iter()
        .filter(|t| t.tool_type == "function")
        .map(|t| KiroTool {
            tool_specification: ToolSpec {
                name: t.function.name.clone(),
                description: t.function.description.clone(),
                input_schema: InputSchema {
                    json: t.function.parameters.clone().unwrap_or(json!({"type": "object", "properties": {}})),
                },
            },
        })
        .collect()
}

fn convert_anthropic_tools(tools: &[AnthropicTool]) -> Vec<KiroTool> {
    tools.iter()
        .map(|t| KiroTool {
            tool_specification: ToolSpec {
                name: t.name.clone(),
                description: t.description.clone(),
                input_schema: InputSchema {
                    json: t.input_schema.clone().unwrap_or(json!({"type": "object", "properties": {}})),
                },
            },
        })
        .collect()
}

/// 处理长 tool description（超过 4000 字符截断，因为 Kiro 不支持 additionalContext）
pub fn process_long_tool_descriptions(
    tools: Option<Vec<KiroTool>>,
    system_prompt: Option<String>,
) -> (Vec<KiroTool>, Option<String>) {
    let Some(tools) = tools else {
        return (vec![], system_prompt);
    };

    let mut processed = Vec::new();

    for tool in tools {
        let desc = tool.tool_specification.description.as_deref().unwrap_or("");

        if desc.len() <= TOOL_DESCRIPTION_MAX_LENGTH {
            processed.push(tool);
        } else {
            // 长 description 截断并警告
            tracing::warn!(
                "Tool '{}' description too long ({} chars), truncating to {} chars",
                tool.tool_specification.name,
                desc.len(),
                TOOL_DESCRIPTION_MAX_LENGTH
            );

            processed.push(KiroTool {
                tool_specification: ToolSpec {
                    name: tool.tool_specification.name.clone(),
                    description: Some(format!(
                        "{}... [truncated]",
                        &desc[..TOOL_DESCRIPTION_MAX_LENGTH.min(desc.len())]
                    )),
                    input_schema: tool.tool_specification.input_schema,
                },
            });
        }
    }

    (processed, system_prompt)
}

// Kiro 事件转 OpenAI 格式
pub fn kiro_to_openai(event: &KiroEvent, request_id: &str) -> Option<OpenAIChunk> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;

    // 文本响应 - content 字段
    if let Some(ref content) = event.content {
        // 如果有 language 字段，说明是代码块
        if let Some(ref lang) = event.language {
            let code_content = format!("```{}\n{}\n```", lang, content);
            return Some(OpenAIChunk {
                id: format!("chatcmpl-{}", request_id),
                object: "chat.completion.chunk".to_string(),
                created: timestamp,
                model: "kiro".to_string(),
                choices: vec![OpenAIChoice {
                    index: 0,
                    delta: OpenAIDelta { content: Some(code_content), ..Default::default() },
                    finish_reason: None,
                }],
                usage: None,
            });
        }
        
        // 普通文本响应
        return Some(OpenAIChunk {
            id: format!("chatcmpl-{}", request_id),
            object: "chat.completion.chunk".to_string(),
            created: timestamp,
            model: "kiro".to_string(),
            choices: vec![OpenAIChoice {
                index: 0,
                delta: OpenAIDelta { content: Some(content.clone()), ..Default::default() },
                finish_reason: None,
            }],
            usage: None,
        });
    }

    // 工具调用 - toolUseId + name + input
    if let (Some(ref tool_use_id), Some(ref name), Some(ref input)) = 
        (&event.tool_use_id, &event.name, &event.input) {
        return Some(OpenAIChunk {
            id: format!("chatcmpl-{}", request_id),
            object: "chat.completion.chunk".to_string(),
            created: timestamp,
            model: "kiro".to_string(),
            choices: vec![OpenAIChoice {
                index: 0,
                delta: OpenAIDelta {
                    tool_calls: Some(vec![ToolCallDelta {
                        index: 0,
                        id: Some(tool_use_id.clone()),
                        call_type: Some("function".to_string()),
                        function: FunctionCallDelta {
                            name: Some(name.clone()),
                            arguments: Some(serde_json::to_string(input).unwrap_or_default()),
                        },
                    }]),
                    ..Default::default()
                },
                finish_reason: None,
            }],
            usage: None,
        });
    }

    None
}

/// Kiro 事件转 Anthropic 格式（用于 SSE 事件）
pub fn kiro_to_anthropic(event: &KiroEvent) -> Option<String> {
    // 文本响应
    if let Some(ref content) = event.content {
        // 如果有 language 字段，说明是代码块
        if let Some(ref lang) = event.language {
            let code_content = format!("```{}\n{}\n```", lang, content);
            let escaped = code_content.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
            return Some(format!(
                r#"{{"type":"content_block_delta","index":0,"delta":{{"type":"text_delta","text":"{}"}}}}"#,
                escaped
            ));
        }
        
        // 普通文本
        let escaped = content.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
        return Some(format!(
            r#"{{"type":"content_block_delta","index":0,"delta":{{"type":"text_delta","text":"{}"}}}}"#,
            escaped
        ));
    }

    // thinking block - text + signature
    if let Some(ref thinking_text) = event.text {
        let escaped = thinking_text.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
        return Some(format!(
            r#"{{"type":"content_block_delta","index":0,"delta":{{"type":"thinking_delta","thinking":"{}"}}}}"#,
            escaped
        ));
    }

    None
}

// 创建结束事件（支持 finish_reason: stop/tool_calls/length）
pub fn create_openai_end_with_reason(request_id: &str, has_tool_use: bool, context_exceeded: bool, usage: Option<Usage>) -> OpenAIChunk {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64;
    let finish_reason = if has_tool_use {
        "tool_calls"
    } else if context_exceeded {
        "length"
    } else {
        "stop"
    };
    OpenAIChunk {
        id: format!("chatcmpl-{}", request_id),
        object: "chat.completion.chunk".to_string(),
        created: timestamp,
        model: "kiro".to_string(),
        choices: vec![OpenAIChoice {
            index: 0,
            delta: OpenAIDelta::default(),
            finish_reason: Some(finish_reason.to_string()),
        }],
        usage,
    }
}





