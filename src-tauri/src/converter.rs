// OpenAI <-> Kiro 格式转换器

use crate::models::*;
use chrono;
use uuid::Uuid;

// 工具描述最大长度，超过此长度的描述会被移到 system prompt
pub const TOOL_DESCRIPTION_MAX_LENGTH: usize = 1024;

// ============================================================
// 图片处理
// ============================================================

/// 从 data URL 或文件扩展名检测图片格式
/// 参考 Kiro IDE 的 formatFromImageUrl 实现
#[allow(dead_code)]
fn detect_image_format(url: &str) -> String {
    // 优先从 data URL 的 media_type 提取
    if url.starts_with("data:") {
        if let Some(header) = url.split(',').next() {
            if let Some(media_type) = header
                .strip_prefix("data:")
                .and_then(|s| s.split(';').next())
            {
                if let Some(format) = media_type.split('/').nth(1) {
                    return format.to_lowercase();
                }
            }
        }
    }

    // 回退到扩展名检测 (Kiro IDE 的方式)
    match url
        .split('.')
        .next_back()
        .map(|s| s.to_lowercase())
        .as_deref()
    {
        Some("png") => "png".to_string(),
        Some("gif") => "gif".to_string(),
        Some("webp") => "webp".to_string(),
        _ => "jpeg".to_string(), // 默认 JPEG (和 Kiro IDE 一致)
    }
}

/// 从消息内容中提取图片
/// 返回 (images, image_count)
///
/// 参考实现:
/// - Kiro IDE: extension.js 的 extractImages 函数
/// - KiroGate: converters.py 的 extract_images_from_content 函数
#[allow(dead_code)]
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

                    tracing::debug!("[kiro-gateway] 提取 OpenAI 图片: format={}", format);
                }
            } else {
                tracing::warn!(
                    "[kiro-gateway] 不支持外部图片 URL: {}...",
                    &url[..url.len().min(50)]
                );
            }
        }
        // Anthropic 格式: {"type": "image", "source": {"type": "base64", "media_type": "image/png", "data": "..."}}
        // Kiro IDE 不支持此格式，但我们需要支持
        else if item_type == "image" {
            if let Some(source) = obj.get("source").and_then(|v| v.as_object()) {
                let source_type = source.get("type").and_then(|v| v.as_str()).unwrap_or("");

                if source_type == "base64" {
                    let media_type = source
                        .get("media_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("image/jpeg");
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

                        tracing::debug!(
                            "[kiro-gateway] 提取 Anthropic 图片: format={}",
                            media_type
                        );
                    }
                } else if source_type == "url" {
                    let url = source.get("url").and_then(|v| v.as_str()).unwrap_or("");
                    tracing::warn!(
                        "[kiro-gateway] 不支持图片 URL: {}...",
                        &url[..url.len().min(50)]
                    );
                }
            }
        }
    }

    let count = images.len();
    (images, count)
}

// ============================================================
// 模型映射
// ============================================================

// 模型映射
#[allow(dead_code)]
pub fn get_internal_model_id(external_model: &str) -> Result<String, String> {
    let model_id = match external_model {
        // Claude Opus 4.5
        "claude-opus-4-5" | "claude-opus-4-5-20251101" | "opus" => "claude-opus-4.5",
        // Claude Haiku 4.5
        "claude-haiku-4-5" | "claude-haiku-4-5-20251001" | "claude-haiku-4.5" | "haiku" => {
            "claude-haiku-4.5"
        }
        // Claude Sonnet 4.5 (最新)
        "claude-sonnet-4-5" | "claude-sonnet-4-5-20250929" | "claude-sonnet-4.5" => {
            "CLAUDE_SONNET_4_5_20250929_V1_0"
        }
        // Claude Sonnet 4
        "claude-sonnet-4" | "claude-sonnet-4-20250514" => "CLAUDE_SONNET_4_20250514_V1_0",
        // Claude 3.7 Sonnet
        "claude-3-7-sonnet-20250219" | "claude-3.7-sonnet" => "CLAUDE_3_7_SONNET_20250219_V1_0",
        // Claude 3.5 Sonnet (映射到 Sonnet 4)
        "claude-3-5-sonnet-20241022"
        | "claude-3-5-sonnet-latest"
        | "claude-3.5-sonnet"
        | "sonnet" => "CLAUDE_SONNET_4_20250514_V1_0",
        // 默认 / auto
        "auto" | "default" => "CLAUDE_SONNET_4_5_20250929_V1_0",
        // 直接传递（可能是内部 ID）
        other => other,
    };
    Ok(model_id.to_string())
}

// 可用模型列表
#[allow(dead_code)]
pub fn get_available_models() -> Vec<ModelInfo> {
    let models = vec![
        // Claude Code 常用名称
        "claude-3-5-sonnet-20241022",
        "claude-3-5-sonnet-latest",
        // Kiro 支持的模型
        "claude-opus-4-5",
        "claude-opus-4-5-20251101",
        "claude-haiku-4-5",
        "claude-haiku-4-5-20251001",
        "claude-sonnet-4-5",
        "claude-sonnet-4-5-20250929",
        "claude-sonnet-4",
        "claude-sonnet-4-20250514",
        "claude-3-7-sonnet-20250219",
    ];

    models
        .into_iter()
        .map(|id| ModelInfo {
            id: id.to_string(),
            object: "model".to_string(),
            created: 1700000000,
            owned_by: "anthropic".to_string(),
        })
        .collect()
}

// ============================================================
// Anthropic -> OpenAI 转换（复用 OpenAI -> Kiro 逻辑）
// ============================================================

/// 将 Anthropic 请求转换为 OpenAI 格式
pub fn anthropic_to_openai(request: &AnthropicMessagesRequest) -> ChatCompletionRequest {
    // 转换消息
    let mut messages: Vec<ChatMessage> = Vec::new();

    // 处理 system 消息
    if let Some(system) = &request.system {
        let system_text = match system {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Array(arr) => arr
                .iter()
                .filter_map(|item| {
                    if let serde_json::Value::Object(obj) = item {
                        if obj.get("type").and_then(|v| v.as_str()) == Some("text") {
                            return obj
                                .get("text")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                        }
                    }
                    None
                })
                .collect::<Vec<_>>()
                .join("\n"),
            _ => String::new(),
        };

        if !system_text.is_empty() {
            messages.push(ChatMessage {
                role: "system".to_string(),
                content: Some(serde_json::Value::String(system_text)),
                tool_calls: None,
                tool_call_id: None,
            });
        }
    }

    // 转换消息列表
    for msg in &request.messages {
        let content = convert_anthropic_content(&msg.content);
        let tool_calls = extract_anthropic_tool_calls(&msg.content);
        let tool_call_id = extract_anthropic_tool_result_id(&msg.content);

        messages.push(ChatMessage {
            role: msg.role.clone(),
            content: Some(content),
            tool_calls: if tool_calls.is_empty() {
                None
            } else {
                Some(tool_calls)
            },
            tool_call_id,
        });
    }

    // 转换 tools
    let tools = request.tools.as_ref().map(|tools| {
        tools
            .iter()
            .map(|t| Tool {
                tool_type: "function".to_string(),
                function: ToolFunction {
                    name: t.name.clone(),
                    description: t.description.clone(),
                    parameters: Some(t.input_schema.clone()),
                },
            })
            .collect()
    });

    ChatCompletionRequest {
        model: request.model.clone(),
        messages,
        stream: request.stream,
        max_tokens: Some(request.max_tokens),
        temperature: request.temperature,
        top_p: request.top_p,
        stop: request.stop_sequences.clone(),
        tools,
        tool_choice: request.tool_choice.clone(),
    }
}

// 转换 Anthropic 内容为 OpenAI 格式
fn convert_anthropic_content(content: &serde_json::Value) -> serde_json::Value {
    match content {
        serde_json::Value::String(s) => serde_json::Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            // 检查是否包含 tool_result（需要保留原始结构）
            let has_tool_result = arr.iter().any(|item| {
                if let serde_json::Value::Object(obj) = item {
                    obj.get("type").and_then(|v| v.as_str()) == Some("tool_result")
                } else {
                    false
                }
            });

            // 如果包含 tool_result，保留原始数组结构
            if has_tool_result {
                return content.clone();
            }

            // 提取文本内容（包括图片 placeholder）
            let text: String = arr
                .iter()
                .filter_map(|item| {
                    if let serde_json::Value::Object(obj) = item {
                        let block_type = obj.get("type").and_then(|v| v.as_str());
                        match block_type {
                            Some("text") => {
                                return obj
                                    .get("text")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string());
                            }
                            Some("image") => {
                                // 图片内容转换为 placeholder
                                if let Some(source) = obj.get("source").and_then(|v| v.as_object())
                                {
                                    let source_type =
                                        source.get("type").and_then(|v| v.as_str()).unwrap_or("");
                                    match source_type {
                                        "base64" => {
                                            let media_type = source
                                                .get("media_type")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("image");
                                            return Some(format!("[Image: {}]", media_type));
                                        }
                                        "url" => {
                                            let url = source
                                                .get("url")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("");
                                            return Some(format!("[Image URL: {}]", url));
                                        }
                                        _ => return Some("[Image]".to_string()),
                                    }
                                }
                                return Some("[Image]".to_string());
                            }
                            _ => {}
                        }
                    }
                    None
                })
                .collect::<Vec<_>>()
                .join("\n");

            if text.is_empty() {
                // 返回原始数组（可能包含 tool_use 等）
                content.clone()
            } else {
                serde_json::Value::String(text)
            }
        }
        _ => content.clone(),
    }
}

// 从 Anthropic 内容中提取 tool_calls
fn extract_anthropic_tool_calls(content: &serde_json::Value) -> Vec<ToolCall> {
    let mut tool_calls = Vec::new();

    if let serde_json::Value::Array(arr) = content {
        for item in arr {
            if let serde_json::Value::Object(obj) = item {
                if obj.get("type").and_then(|v| v.as_str()) == Some("tool_use") {
                    let id = obj
                        .get("id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = obj
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let input = obj.get("input").cloned().unwrap_or(serde_json::json!({}));

                    tool_calls.push(ToolCall {
                        id,
                        call_type: "function".to_string(),
                        function: ToolCallFunction {
                            name,
                            arguments: serde_json::to_string(&input).unwrap_or_default(),
                        },
                    });
                }
            }
        }
    }

    tool_calls
}

// 从 Anthropic 内容中提取 tool_result 的 tool_use_id
fn extract_anthropic_tool_result_id(content: &serde_json::Value) -> Option<String> {
    if let serde_json::Value::Array(arr) = content {
        for item in arr {
            if let serde_json::Value::Object(obj) = item {
                if obj.get("type").and_then(|v| v.as_str()) == Some("tool_result") {
                    return obj
                        .get("tool_use_id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                }
            }
        }
    }
    None
}

// 提取文本内容
fn extract_text_content(content: &Option<serde_json::Value>) -> String {
    match content {
        None => String::new(),
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Array(arr)) => arr
            .iter()
            .filter_map(|item| {
                if let serde_json::Value::Object(obj) = item {
                    if obj.get("type").and_then(|v| v.as_str()) == Some("text") {
                        return obj
                            .get("text")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                    }
                }
                None
            })
            .collect::<Vec<_>>()
            .join(""),
        Some(other) => other.to_string(),
    }
}

// 提取 tool_results
fn extract_tool_results(content: &Option<serde_json::Value>) -> Vec<KiroToolResult> {
    let mut results = Vec::new();

    if let Some(serde_json::Value::Array(arr)) = content {
        for item in arr {
            if let serde_json::Value::Object(obj) = item {
                if obj.get("type").and_then(|v| v.as_str()) == Some("tool_result") {
                    let tool_use_id = obj
                        .get("tool_use_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let content_text = obj
                        .get("content")
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .unwrap_or_default();

                    // 读取 is_error 字段，转换为 status
                    let is_error = obj
                        .get("is_error")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let status = if is_error { "error" } else { "success" };

                    results.push(KiroToolResult {
                        content: vec![KiroToolResultContent { text: content_text }],
                        status: status.to_string(),
                        tool_use_id,
                    });
                }
            }
        }
    }

    results
}

// 提取 tool_uses
fn extract_tool_uses(msg: &ChatMessage) -> Vec<KiroToolUse> {
    let mut uses = Vec::new();

    if let Some(tool_calls) = &msg.tool_calls {
        for tc in tool_calls {
            let input: serde_json::Value = serde_json::from_str(&tc.function.arguments)
                .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));

            uses.push(KiroToolUse {
                name: tc.function.name.clone(),
                input,
                tool_use_id: tc.id.clone(),
            });
        }
    }

    uses
}

// 转换 tools
fn convert_tools(tools: &Option<Vec<Tool>>) -> Option<Vec<KiroTool>> {
    tools.as_ref().map(|tools| {
        tools
            .iter()
            .filter(|t| t.tool_type == "function")
            .map(|t| {
                let params = t
                    .function
                    .parameters
                    .clone()
                    .unwrap_or(serde_json::json!({}));
                tracing::debug!(
                    "[kiro-gateway] 转换工具: name={}, description={}, params={}",
                    t.function.name,
                    t.function.description.as_deref().unwrap_or(""),
                    serde_json::to_string(&params).unwrap_or_default()
                );
                KiroTool {
                    tool_specification: KiroToolSpec {
                        name: t.function.name.clone(),
                        description: t.function.description.clone().unwrap_or_default(),
                        input_schema: KiroInputSchema { json: params },
                    },
                }
            })
            .collect()
    })
}

/// 构建 Kiro API payload
pub fn build_kiro_payload(
    request: &ChatCompletionRequest,
    profile_arn: Option<String>,
) -> Result<KiroPayload, String> {
    // 调试日志：打印消息数量
    tracing::debug!(
        "[kiro-gateway] build_kiro_payload: 收到 {} 条消息",
        request.messages.len()
    );

    let model_id = get_internal_model_id(&request.model)?;
    let conversation_id = Uuid::new_v4().to_string();

    // 构建推理配置
    let inference_config = build_inference_config(request);

    // 分离 system 消息和其他消息
    let mut system_prompt = String::new();
    let mut other_messages: Vec<&ChatMessage> = Vec::new();

    for msg in &request.messages {
        if msg.role == "system" {
            system_prompt.push_str(&extract_text_content(&msg.content));
            system_prompt.push('\n');
        } else {
            other_messages.push(msg);
        }
    }
    system_prompt = system_prompt.trim().to_string();

    if other_messages.is_empty() {
        return Err("没有可发送的消息".to_string());
    }

    // 合并相邻的同角色消息
    let merged_messages = merge_adjacent_messages(&other_messages);
    
    // 应用消息清理逻辑（参考 Kiro IDE 的 sanitizeConversation）
    let sanitized_messages = sanitize_conversation(merged_messages);

    // 构建历史（除最后一条）
    let history = if sanitized_messages.len() > 1 {
        let history_msgs = &sanitized_messages[..sanitized_messages.len() - 1];
        let mut history_items = Vec::new();
        let mut is_first_user = true;

        for msg in history_msgs {
            match msg.role.as_str() {
                "user" => {
                    let mut content = extract_text_content(&msg.content);

                    // 检测历史消息中的图片，用占位符替代（避免请求体过大）
                    let (_, image_count) = extract_images_from_content(&msg.content);
                    if image_count > 0 {
                        let placeholder =
                            format!("\n[此消息包含 {} 张图片，已在历史记录中省略]", image_count);
                        content = if content.is_empty() {
                            placeholder
                        } else {
                            format!("{}{}", content, placeholder)
                        };
                        tracing::debug!(
                            "[kiro-gateway] 历史消息中替换 {} 张图片为占位符",
                            image_count
                        );
                    }

                    // 将 system prompt 添加到第一个 user 消息
                    if is_first_user && !system_prompt.is_empty() {
                        content = format!("{}\n\n{}", system_prompt, content);
                        is_first_user = false;
                    }

                    let tool_results = extract_tool_results(&msg.content);
                    let context = if !tool_results.is_empty() {
                        Some(UserInputMessageContext {
                            tools: None,
                            tool_results: Some(tool_results),
                        })
                    } else {
                        None
                    };

                    history_items.push(HistoryItem::User {
                        user_input_message: HistoryUserMessage {
                            content,
                            model_id: model_id.clone(),
                            user_intent: Some("CODE_GENERATION".to_string()),
                            origin: "AI_EDITOR".to_string(),
                            images: None, // 历史消息不包含图片数据
                            user_input_message_context: context,
                            inference_config: inference_config.clone(),
                        },
                    });
                }
                "assistant" => {
                    let content = extract_text_content(&msg.content);
                    let tool_uses = extract_tool_uses(msg);

                    history_items.push(HistoryItem::Assistant {
                        assistant_response_message: HistoryAssistantMessage {
                            content,
                            tool_uses: if tool_uses.is_empty() {
                                None
                            } else {
                                Some(tool_uses)
                            },
                        },
                    });
                }
                "tool" => {
                    // tool 消息转换为 user 消息的 tool_result
                    let tool_result = KiroToolResult {
                        content: vec![KiroToolResultContent {
                            text: extract_text_content(&msg.content),
                        }],
                        status: "success".to_string(),
                        tool_use_id: msg.tool_call_id.clone().unwrap_or_default(),
                    };

                    history_items.push(HistoryItem::User {
                        user_input_message: HistoryUserMessage {
                            content: String::new(),
                            model_id: model_id.clone(),
                            user_intent: Some("CODE_GENERATION".to_string()),
                            origin: "AI_EDITOR".to_string(),
                            images: None, // tool 消息不包含图片
                            user_input_message_context: Some(UserInputMessageContext {
                                tools: None,
                                tool_results: Some(vec![tool_result]),
                            }),
                            inference_config: inference_config.clone(),
                        },
                    });
                }
                _ => {}
            }
        }

        if history_items.is_empty() {
            None
        } else {
            Some(history_items)
        }
    } else {
        None
    };

    // 当前消息（最后一条）
    let current_msg = sanitized_messages
        .last()
        .ok_or_else(|| "消息列表为空".to_string())?;
    let mut current_content = extract_text_content(&current_msg.content);

    // 如果没有历史且有 system prompt，添加到当前消息
    if history.is_none() && !system_prompt.is_empty() {
        current_content = format!("{}\n\n{}", system_prompt, current_content);
    }

    // 如果当前消息是 assistant，需要特殊处理
    if current_msg.role == "assistant" {
        current_content = CONTINUE_MESSAGE_CONTENT.to_string();
    }

    if current_content.is_empty() {
        current_content = CONTINUE_MESSAGE_CONTENT.to_string();
    }

    // 构建 context
    let tool_results = extract_tool_results(&current_msg.content);

    // 处理长 description 的工具
    tracing::debug!(
        "[kiro-gateway] 原始 tools 数量: {:?}",
        request.tools.as_ref().map(|t| t.len())
    );
    let (processed_tools, tool_docs) = process_tools_with_long_descriptions(&request.tools);
    let tools = convert_tools(&processed_tools);
    tracing::debug!(
        "[kiro-gateway] 转换后 tools 数量: {:?}",
        tools.as_ref().map(|t| t.len())
    );

    // 如果有长 description 的工具文档，添加到 system prompt
    let mut final_content = current_content;
    if let Some(docs) = tool_docs {
        if history.is_none() {
            // 没有历史时，添加到当前消息
            final_content = format!("{}\n\n{}", docs, final_content);
        }
        // 有历史时，文档已经在第一个 user 消息中了（通过 system_prompt）
        // 这里需要特殊处理，但为了简化，暂时只处理无历史的情况
        tracing::debug!("[kiro-gateway] 长 description 工具文档已添加到消息中");
    }

    let context = if tools.is_some() || !tool_results.is_empty() {
        Some(UserInputMessageContext {
            tools,
            tool_results: if tool_results.is_empty() {
                None
            } else {
                Some(tool_results)
            },
        })
    } else {
        None
    };

    // 提取当前消息中的图片（仅当前消息是 user 消息时）
    let (images, image_count) = if current_msg.role == "user" {
        extract_images_from_content(&current_msg.content)
    } else {
        (Vec::new(), 0)
    };

    // 如果有图片，记录日志
    if image_count > 0 {
        tracing::info!("[kiro-gateway] 添加 {} 张图片到当前消息", image_count);
    }

    let payload = KiroPayload {
        conversation_state: ConversationState {
            agent_continuation_id: Uuid::new_v4().to_string(),
            agent_task_type: "vibe".to_string(),
            chat_trigger_type: "MANUAL".to_string(),
            conversation_id,
            current_message: CurrentMessage {
                user_input_message: UserInputMessage {
                    content: final_content,
                    model_id,
                    user_intent: Some("CODE_GENERATION".to_string()),
                    origin: "AI_EDITOR".to_string(),
                    images: if images.is_empty() {
                        None
                    } else {
                        Some(images)
                    },
                    user_input_message_context: context,
                    inference_config,
                },
            },
            history,
        },
        profile_arn,
    };

    // 调试：打印完整的 payload
    if let Ok(json) = serde_json::to_string_pretty(&payload) {
        tracing::debug!("[kiro-gateway] Kiro API 请求体:\n{}", json);
    }

    Ok(payload)
}

// 构建推理配置
fn build_inference_config(request: &ChatCompletionRequest) -> Option<InferenceConfig> {
    if request.max_tokens.is_none()
        && request.temperature.is_none()
        && request.top_p.is_none()
        && request.stop.is_none()
    {
        return None;
    }

    Some(InferenceConfig {
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        top_p: request.top_p,
        stop_sequences: request.stop.clone(),
    })
}

// 合并相邻的同角色消息
fn merge_adjacent_messages(messages: &[&ChatMessage]) -> Vec<ChatMessage> {
    let mut merged: Vec<ChatMessage> = Vec::new();

    for msg in messages {
        if merged.is_empty() {
            merged.push((*msg).clone());
            continue;
        }

        if let Some(last) = merged.last_mut() {
            if last.role == msg.role {
                // 合并内容
                let last_text = extract_text_content(&last.content);
                let current_text = extract_text_content(&msg.content);
                last.content = Some(serde_json::Value::String(format!(
                    "{}\n{}",
                    last_text, current_text
                )));

                // 合并 tool_calls
                if let Some(ref tc) = msg.tool_calls {
                    if last.tool_calls.is_none() {
                        last.tool_calls = Some(Vec::new());
                    }
                    if let Some(tool_calls) = last.tool_calls.as_mut() {
                        tool_calls.extend(tc.clone());
                    }
                }
            } else {
                merged.push((*msg).clone());
            }
        }
    }

    merged
}

// ============================================================
// 消息清理（参考 Kiro IDE 的 sanitizeConversation）
// ============================================================

/// 标准占位消息
const HELLO_MESSAGE_CONTENT: &str = "Hello";
const CONTINUE_MESSAGE_CONTENT: &str = "Continue";
const UNDERSTOOD_MESSAGE_CONTENT: &str = "understood";

/// 检查消息是否为空（没有内容且没有 tool results）
fn is_empty_user_message(msg: &ChatMessage) -> bool {
    if msg.role != "user" {
        return false;
    }
    
    let has_content = match &msg.content {
        Some(serde_json::Value::String(s)) => !s.trim().is_empty(),
        Some(serde_json::Value::Array(arr)) => !arr.is_empty(),
        _ => false,
    };
    
    let has_tool_results = extract_tool_results(&msg.content).len() > 0;
    
    !has_content && !has_tool_results
}

/// 检查 tool uses 和 tool results 是否匹配
fn has_matching_tool_results(tool_uses: &[KiroToolUse], tool_results: &[KiroToolResult]) -> bool {
    if tool_uses.is_empty() {
        return true;
    }
    if tool_results.is_empty() {
        return false;
    }
    
    // 检查所有 tool use 都有对应的 result
    let all_uses_have_results = tool_uses.iter().all(|tool_use| {
        tool_results.iter().any(|result| result.tool_use_id == tool_use.tool_use_id)
    });
    
    // 检查所有 tool result 都有对应的 use
    let all_results_have_uses = tool_results.iter().all(|result| {
        tool_uses.iter().any(|tool_use| result.tool_use_id == tool_use.tool_use_id)
    });
    
    all_uses_have_results && all_results_have_uses
}

/// 确保消息以 user 消息开始
fn ensure_starts_with_user_message(messages: Vec<ChatMessage>) -> Vec<ChatMessage> {
    if messages.is_empty() || messages[0].role != "user" {
        let mut result = vec![ChatMessage {
            role: "user".to_string(),
            content: Some(serde_json::Value::String(HELLO_MESSAGE_CONTENT.to_string())),
            tool_calls: None,
            tool_call_id: None,
        }];
        result.extend(messages);
        result
    } else {
        messages
    }
}

/// 移除空的 user 消息（除了第一条）
fn remove_empty_user_messages(messages: Vec<ChatMessage>) -> Vec<ChatMessage> {
    if messages.len() <= 1 {
        return messages;
    }
    
    let first_user_index = messages.iter().position(|m| m.role == "user");
    
    messages.into_iter().enumerate().filter(|(index, msg)| {
        // 保留所有 assistant 消息
        if msg.role == "assistant" {
            return true;
        }
        
        // 保留第一条 user 消息
        if msg.role == "user" && Some(*index) == first_user_index {
            return true;
        }
        
        // 检查 user 消息是否有内容或 tool results
        if msg.role == "user" {
            return !is_empty_user_message(msg);
        }
        
        true
    }).map(|(_, msg)| msg).collect()
}

/// 确保消息交替（user → assistant → user → assistant）
fn ensure_alternating_messages(messages: Vec<ChatMessage>) -> Vec<ChatMessage> {
    if messages.len() <= 1 {
        return messages;
    }
    
    let mut result = vec![messages[0].clone()];
    
    for msg in messages.into_iter().skip(1) {
        let prev_role = &result.last().unwrap().role;
        
        // 两条连续的 user 消息 → 插入 UNDERSTOOD_MESSAGE
        if prev_role == "user" && msg.role == "user" {
            result.push(ChatMessage {
                role: "assistant".to_string(),
                content: Some(serde_json::Value::String(UNDERSTOOD_MESSAGE_CONTENT.to_string())),
                tool_calls: None,
                tool_call_id: None,
            });
        }
        // 两条连续的 assistant 消息 → 插入 CONTINUE_MESSAGE
        else if prev_role == "assistant" && msg.role == "assistant" {
            result.push(ChatMessage {
                role: "user".to_string(),
                content: Some(serde_json::Value::String(CONTINUE_MESSAGE_CONTENT.to_string())),
                tool_calls: None,
                tool_call_id: None,
            });
        }
        
        result.push(msg);
    }
    
    result
}

/// 确保消息以 user 消息结束
fn ensure_ends_with_user_message(messages: Vec<ChatMessage>) -> Vec<ChatMessage> {
    if messages.is_empty() {
        return vec![ChatMessage {
            role: "user".to_string(),
            content: Some(serde_json::Value::String(HELLO_MESSAGE_CONTENT.to_string())),
            tool_calls: None,
            tool_call_id: None,
        }];
    }
    
    if messages.last().unwrap().role != "user" {
        let mut result = messages;
        result.push(ChatMessage {
            role: "user".to_string(),
            content: Some(serde_json::Value::String(CONTINUE_MESSAGE_CONTENT.to_string())),
            tool_calls: None,
            tool_call_id: None,
        });
        result
    } else {
        messages
    }
}

/// 确保工具调用有对应的结果
/// 如果没有，自动添加失败消息（status: "error"）
fn ensure_valid_tool_uses_and_results(messages: Vec<ChatMessage>) -> Vec<ChatMessage> {
    let mut result = Vec::new();
    
    for (i, msg) in messages.iter().enumerate() {
        result.push(msg.clone());
        
        // 检查 assistant 消息是否有 tool uses
        if msg.role == "assistant" {
            let tool_uses = extract_tool_uses(msg);
            if !tool_uses.is_empty() {
                let next_msg = messages.get(i + 1);
                
                // 情况1：没有下一条消息，或下一条不是 user 消息，或没有 tool results
                if next_msg.is_none() || next_msg.unwrap().role != "user" {
                    let tool_use_ids: Vec<String> = tool_uses.iter()
                        .map(|tu| tu.tool_use_id.clone())
                        .collect();
                    result.push(create_failed_tool_use_message(tool_use_ids));
                } else if let Some(next) = next_msg {
                    let tool_results = extract_tool_results(&next.content);
                    
                    // 情况2：tool results 不匹配
                    if !has_matching_tool_results(&tool_uses, &tool_results) {
                        // 检查是否有其他地方匹配
                        let has_matching_elsewhere = messages.iter().enumerate().any(|(j, other_msg)| {
                            if j == i || other_msg.role != "assistant" {
                                return false;
                            }
                            let other_uses = extract_tool_uses(other_msg);
                            has_matching_tool_results(&other_uses, &tool_results)
                        });
                        
                        // 如果没有其他地方匹配，添加失败消息
                        if !has_matching_elsewhere {
                            let tool_use_ids: Vec<String> = tool_uses.iter()
                                .map(|tu| tu.tool_use_id.clone())
                                .collect();
                            result.push(create_failed_tool_use_message(tool_use_ids));
                        }
                    }
                }
            }
        }
    }
    
    result
}

/// 创建工具调用失败消息
fn create_failed_tool_use_message(tool_use_ids: Vec<String>) -> ChatMessage {
    let tool_results: Vec<serde_json::Value> = tool_use_ids.iter().map(|id| {
        serde_json::json!({
            "type": "tool_result",
            "tool_use_id": id,
            "content": "Tool execution failed",
            "is_error": true
        })
    }).collect();
    
    ChatMessage {
        role: "user".to_string(),
        content: Some(serde_json::Value::Array(tool_results)),
        tool_calls: None,
        tool_call_id: None,
    }
}

/// 清理消息列表，确保符合 Kiro API 要求
/// 参考 Kiro IDE 的 sanitizeConversation 实现
pub fn sanitize_conversation(messages: Vec<ChatMessage>) -> Vec<ChatMessage> {
    let original_len = messages.len();
    let mut sanitized = messages;
    
    // 1. 确保以 user 消息开始
    sanitized = ensure_starts_with_user_message(sanitized);
    
    // 2. 移除空的 user 消息
    sanitized = remove_empty_user_messages(sanitized);
    
    // 3. 确保工具调用有对应结果
    sanitized = ensure_valid_tool_uses_and_results(sanitized);
    
    // 4. 确保消息交替（user → assistant → user → assistant）
    sanitized = ensure_alternating_messages(sanitized);
    
    // 5. 确保以 user 消息结束
    sanitized = ensure_ends_with_user_message(sanitized);
    
    tracing::debug!(
        "[kiro-gateway] 消息清理: {} -> {} 条消息",
        original_len,
        sanitized.len()
    );
    
    sanitized
}

/// 截断消息历史，只保留最后一对对话
/// 参考 Kiro IDE 的 trimMessageHistory 实现
/// 
/// 逻辑：
/// 1. 找出所有完整的 human/AI 对话对
/// 2. 只保留最后一对
/// 3. 如果最后一条消息是 human（还没有 AI 回复），也保留
pub fn trim_message_history(messages: &[ChatMessage]) -> Vec<ChatMessage> {
    if messages.is_empty() {
        return Vec::new();
    }

    // 先合并相邻的同角色消息
    let merged = merge_adjacent_messages(&messages.iter().collect::<Vec<_>>());

    // 找出所有完整的 human/AI 对话对
    let mut pairs: Vec<(ChatMessage, ChatMessage)> = Vec::new();
    let mut i = 0;
    while i < merged.len() - 1 {
        if merged[i].role == "user" && merged[i + 1].role == "assistant" {
            pairs.push((merged[i].clone(), merged[i + 1].clone()));
            i += 2; // 跳过这一对
        } else {
            i += 1;
        }
    }

    let mut result = Vec::new();

    // 只保留最后一对
    if let Some((human, ai)) = pairs.last() {
        result.push(human.clone());
        result.push(ai.clone());
    }

    // 检查最后一条消息是否是 human（还没有 AI 回复）
    if let Some(last_msg) = merged.last() {
        if last_msg.role == "user" {
            // 检查是否已经包含在 pairs 中
            let already_included = pairs.last().map(|(h, _)| h.content == last_msg.content).unwrap_or(false);
            if !already_included {
                result.push(last_msg.clone());
            }
        }
    }

    // 如果结果为空，至少返回最后一条 human 消息
    if result.is_empty() && !merged.is_empty() {
        // 从后往前找第一条 user 消息
        if let Some(last_human) = merged.iter().rev().find(|m| m.role == "user") {
            result.push(last_human.clone());
        } else {
            // 实在没有，创建一个 "continue" 消息
            result.push(ChatMessage {
                role: "user".to_string(),
                content: Some(serde_json::Value::String("continue".to_string())),
                tool_calls: None,
                tool_call_id: None,
            });
        }
    }

    // 确保第一条是 user，最后一条也是 user
    if !result.is_empty() {
        if result[0].role != "user" {
            result.insert(
                0,
                ChatMessage {
                    role: "user".to_string(),
                    content: Some(serde_json::Value::String("continue".to_string())),
                    tool_calls: None,
                    tool_call_id: None,
                },
            );
        }
        if result.last().unwrap().role != "user" {
            result.push(ChatMessage {
                role: "user".to_string(),
                content: Some(serde_json::Value::String("continue".to_string())),
                tool_calls: None,
                tool_call_id: None,
            });
        }
    }

    // 确保所有消息都有内容
    for msg in &mut result {
        if let Some(serde_json::Value::String(s)) = &msg.content {
            if s.is_empty() {
                msg.content = Some(serde_json::Value::String(
                    if msg.role == "user" {
                        "continue"
                    } else {
                        "understood"
                    }
                    .to_string(),
                ));
            }
        }
    }

    tracing::info!(
        "[kiro-gateway] 截断消息历史: {} -> {} 条消息",
        messages.len(),
        result.len()
    );

    result
}

/// 处理长 description 的工具
/// 超过 TOOL_DESCRIPTION_MAX_LENGTH 的描述会被移到 system prompt
/// 返回 (处理后的 tools, 需要添加到 system prompt 的文档)
pub fn process_tools_with_long_descriptions(
    tools: &Option<Vec<Tool>>,
) -> (Option<Vec<Tool>>, Option<String>) {
    let tools = match tools {
        Some(t) if !t.is_empty() => t,
        _ => return (tools.clone(), None),
    };

    let mut processed_tools = Vec::new();
    let mut long_descriptions = Vec::new();

    for tool in tools {
        let desc = tool.function.description.as_deref().unwrap_or("");

        if desc.len() > TOOL_DESCRIPTION_MAX_LENGTH {
            // 描述过长，移到 system prompt
            long_descriptions.push(format!("## Tool: {}\n\n{}", tool.function.name, desc));

            // 工具中留引用
            processed_tools.push(Tool {
                tool_type: tool.tool_type.clone(),
                function: ToolFunction {
                    name: tool.function.name.clone(),
                    description: Some(format!(
                        "[Full documentation in system prompt under '## Tool: {}']",
                        tool.function.name
                    )),
                    parameters: tool.function.parameters.clone(),
                },
            });
        } else {
            processed_tools.push(tool.clone());
        }
    }

    let system_addition = if long_descriptions.is_empty() {
        None
    } else {
        Some(format!(
            "# Tool Documentation\n\n{}",
            long_descriptions.join("\n\n")
        ))
    };

    (Some(processed_tools), system_addition)
}
// 流式响应辅助函数

/// 检查 OpenAI 请求是否为流式
pub fn is_stream_request_openai(request: &ChatCompletionRequest) -> bool {
    request.stream
}

/// 检查 Anthropic 请求是否为流式
pub fn is_stream_request_anthropic(request: &AnthropicMessagesRequest) -> bool {
    request.stream
}

/// Kiro SSE 事件转 OpenAI 格式
pub fn kiro_to_openai(event: &KiroEvent, request_id: &str) -> Option<serde_json::Value> {
    // 文本内容
    if let Some(ref content) = event.content {
        return Some(serde_json::json!({
            "id": format!("chatcmpl-{}", request_id),
            "object": "chat.completion.chunk",
            "created": chrono::Utc::now().timestamp(),
            "model": "kiro",
            "choices": [{
                "index": 0,
                "delta": {
                    "content": content
                },
                "finish_reason": null
            }]
        }));
    }

    // 工具调用
    if let (Some(ref tool_use_id), Some(ref name), Some(ref input)) =
        (&event.tool_use_id, &event.name, &event.input)
    {
        return Some(serde_json::json!({
            "id": format!("chatcmpl-{}", request_id),
            "object": "chat.completion.chunk",
            "created": chrono::Utc::now().timestamp(),
            "model": "kiro",
            "choices": [{
                "index": 0,
                "delta": {
                    "tool_calls": [{
                        "id": tool_use_id,
                        "type": "function",
                        "function": {
                            "name": name,
                            "arguments": serde_json::to_string(input).unwrap_or_default()
                        }
                    }]
                },
                "finish_reason": null
            }]
        }));
    }

    None
}

/// Kiro SSE 事件转 Anthropic 格式
pub fn kiro_to_anthropic(event: &KiroEvent) -> Option<String> {
    // thinking block
    if let Some(ref text) = event.text {
        let escaped = text
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        return Some(format!(
            r#"{{"type":"content_block_delta","index":0,"delta":{{"type":"thinking_delta","thinking":"{}"}}}}"#,
            escaped
        ));
    }

    None
}

/// 创建 OpenAI 流式结束事件
pub fn create_openai_end_with_reason(
    request_id: &str,
    has_tool: bool,
    context_exceeded: bool,
    usage: Option<Usage>,
) -> serde_json::Value {
    let finish_reason = if has_tool {
        "tool_calls"
    } else if context_exceeded {
        "length"
    } else {
        "stop"
    };

    serde_json::json!({
        "id": format!("chatcmpl-{}", request_id),
        "object": "chat.completion.chunk",
        "created": chrono::Utc::now().timestamp(),
        "model": "kiro",
        "choices": [{
            "index": 0,
            "delta": {},
            "finish_reason": finish_reason
        }],
        "usage": usage
    })
}
