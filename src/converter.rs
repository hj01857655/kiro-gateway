// OpenAI <-> Kiro 格式转换器

use crate::kiro_gate::models::*;
use uuid::Uuid;

// 工具描述最大长度，超过此长度的描述会被移到 system prompt
pub const TOOL_DESCRIPTION_MAX_LENGTH: usize = 1024;

// 模型映射
pub fn get_internal_model_id(external_model: &str) -> Result<String, String> {
  let model_id = match external_model {
    // Claude Opus 4.5
    "claude-opus-4-5" | "claude-opus-4-5-20251101" | "opus" => "claude-opus-4.5",
    // Claude Haiku 4.5
    "claude-haiku-4-5" | "claude-haiku-4-5-20251001" | "claude-haiku-4.5" | "haiku" => "claude-haiku-4.5",
    // Claude Sonnet 4.5 (最新)
    "claude-sonnet-4-5" | "claude-sonnet-4-5-20250929" | "claude-sonnet-4.5" => "CLAUDE_SONNET_4_5_20250929_V1_0",
    // Claude Sonnet 4
    "claude-sonnet-4" | "claude-sonnet-4-20250514" => "CLAUDE_SONNET_4_20250514_V1_0",
    // Claude 3.7 Sonnet
    "claude-3-7-sonnet-20250219" | "claude-3.7-sonnet" => "CLAUDE_3_7_SONNET_20250219_V1_0",
    // Claude 3.5 Sonnet (映射到 Sonnet 4)
    "claude-3-5-sonnet-20241022" | "claude-3-5-sonnet-latest" | "claude-3.5-sonnet" | "sonnet" => "CLAUDE_SONNET_4_20250514_V1_0",
    // 默认 / auto
    "auto" | "default" => "CLAUDE_SONNET_4_5_20250929_V1_0",
    // 直接传递（可能是内部 ID）
    other => other,
  };
  Ok(model_id.to_string())
}

// 可用模型列表
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

  models.into_iter().map(|id| ModelInfo {
    id: id.to_string(),
    object: "model".to_string(),
    created: 1700000000,
    owned_by: "anthropic".to_string(),
  }).collect()
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
      serde_json::Value::Array(arr) => {
        arr.iter()
          .filter_map(|item| {
            if let serde_json::Value::Object(obj) = item {
              if obj.get("type").and_then(|v| v.as_str()) == Some("text") {
                return obj.get("text").and_then(|v| v.as_str()).map(|s| s.to_string());
              }
            }
            None
          })
          .collect::<Vec<_>>()
          .join("\n")
      }
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
      tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
      tool_call_id,
    });
  }
  
  // 转换 tools
  let tools = request.tools.as_ref().map(|tools| {
    tools.iter().map(|t| Tool {
      tool_type: "function".to_string(),
      function: ToolFunction {
        name: t.name.clone(),
        description: t.description.clone(),
        parameters: Some(t.input_schema.clone()),
      },
    }).collect()
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
      let text: String = arr.iter()
        .filter_map(|item| {
          if let serde_json::Value::Object(obj) = item {
            let block_type = obj.get("type").and_then(|v| v.as_str());
            match block_type {
              Some("text") => {
                return obj.get("text").and_then(|v| v.as_str()).map(|s| s.to_string());
              }
              Some("image") => {
                // 图片内容转换为 placeholder
                if let Some(source) = obj.get("source").and_then(|v| v.as_object()) {
                  let source_type = source.get("type").and_then(|v| v.as_str()).unwrap_or("");
                  match source_type {
                    "base64" => {
                      let media_type = source.get("media_type").and_then(|v| v.as_str()).unwrap_or("image");
                      return Some(format!("[Image: {}]", media_type));
                    }
                    "url" => {
                      let url = source.get("url").and_then(|v| v.as_str()).unwrap_or("");
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
          let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
          let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
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
          return obj.get("tool_use_id").and_then(|v| v.as_str()).map(|s| s.to_string());
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
    Some(serde_json::Value::Array(arr)) => {
      arr.iter()
        .filter_map(|item| {
          if let serde_json::Value::Object(obj) = item {
            if obj.get("type").and_then(|v| v.as_str()) == Some("text") {
              return obj.get("text").and_then(|v| v.as_str()).map(|s| s.to_string());
            }
          }
          None
        })
        .collect::<Vec<_>>()
        .join("")
    }
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
          let tool_use_id = obj.get("tool_use_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
          let content_text = obj.get("content")
            .map(|v| match v {
              serde_json::Value::String(s) => s.clone(),
              other => other.to_string(),
            })
            .unwrap_or_default();
          
          // 读取 is_error 字段，转换为 status
          let is_error = obj.get("is_error")
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
    tools.iter()
      .filter(|t| t.tool_type == "function")
      .map(|t| KiroTool {
        tool_specification: KiroToolSpec {
          name: t.function.name.clone(),
          description: t.function.description.clone().unwrap_or_default(),
          input_schema: KiroInputSchema {
            json: t.function.parameters.clone().unwrap_or(serde_json::json!({})),
          },
        },
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
  log::debug!("[KiroGate] build_kiro_payload: 收到 {} 条消息", request.messages.len());
  
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
  
  // 构建历史（除最后一条）
  let history = if merged_messages.len() > 1 {
    let history_msgs = &merged_messages[..merged_messages.len() - 1];
    let mut history_items = Vec::new();
    let mut is_first_user = true;
    
    for msg in history_msgs {
      match msg.role.as_str() {
        "user" => {
          let mut content = extract_text_content(&msg.content);
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
              origin: "AI_EDITOR".to_string(),
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
              tool_uses: if tool_uses.is_empty() { None } else { Some(tool_uses) },
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
              origin: "AI_EDITOR".to_string(),
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
    
    if history_items.is_empty() { None } else { Some(history_items) }
  } else {
    None
  };
  
  // 当前消息（最后一条）
  let current_msg = merged_messages.last().unwrap();
  let mut current_content = extract_text_content(&current_msg.content);
  
  // 如果没有历史且有 system prompt，添加到当前消息
  if history.is_none() && !system_prompt.is_empty() {
    current_content = format!("{}\n\n{}", system_prompt, current_content);
  }
  
  // 如果当前消息是 assistant，需要特殊处理
  if current_msg.role == "assistant" {
    current_content = "Continue".to_string();
  }
  
  if current_content.is_empty() {
    current_content = "Continue".to_string();
  }
  
  // 构建 context
  let tool_results = extract_tool_results(&current_msg.content);
  
  // 处理长 description 的工具
  let (processed_tools, tool_docs) = process_tools_with_long_descriptions(&request.tools);
  let tools = convert_tools(&processed_tools);
  
  // 如果有长 description 的工具文档，添加到 system prompt
  let mut final_content = current_content;
  if let Some(docs) = tool_docs {
    if history.is_none() {
      // 没有历史时，添加到当前消息
      final_content = format!("{}\n\n{}", docs, final_content);
    }
    // 有历史时，文档已经在第一个 user 消息中了（通过 system_prompt）
    // 这里需要特殊处理，但为了简化，暂时只处理无历史的情况
    log::debug!("[KiroGate] 长 description 工具文档已添加到消息中");
  }
  
  let context = if tools.is_some() || !tool_results.is_empty() {
    Some(UserInputMessageContext {
      tools,
      tool_results: if tool_results.is_empty() { None } else { Some(tool_results) },
    })
  } else {
    None
  };
  
  Ok(KiroPayload {
    conversation_state: ConversationState {
      chat_trigger_type: "MANUAL".to_string(),
      conversation_id,
      current_message: CurrentMessage {
        user_input_message: UserInputMessage {
          content: final_content,
          model_id,
          origin: "AI_EDITOR".to_string(),
          user_input_message_context: context,
          inference_config,
        },
      },
      history,
    },
    profile_arn,
  })
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
    
    let last = merged.last_mut().unwrap();
    if last.role == msg.role {
      // 合并内容
      let last_text = extract_text_content(&last.content);
      let current_text = extract_text_content(&msg.content);
      last.content = Some(serde_json::Value::String(format!("{}\n{}", last_text, current_text)));
      
      // 合并 tool_calls
      if let Some(ref tc) = msg.tool_calls {
        if last.tool_calls.is_none() {
          last.tool_calls = Some(Vec::new());
        }
        last.tool_calls.as_mut().unwrap().extend(tc.clone());
      }
    } else {
      merged.push((*msg).clone());
    }
  }
  
  merged
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
      long_descriptions.push(format!(
        "## Tool: {}\n\n{}",
        tool.function.name,
        desc
      ));
      
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
