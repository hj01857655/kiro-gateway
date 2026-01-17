// OpenAI 兼容的请求/响应模型

use serde::{Deserialize, Serialize};

// ============================================================
// OpenAI Chat Completion 请求
// ============================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionRequest {
  pub model: String,
  pub messages: Vec<ChatMessage>,
  #[serde(default)]
  pub stream: bool,
  pub max_tokens: Option<i32>,
  pub temperature: Option<f32>,
  pub top_p: Option<f32>,
  pub stop: Option<Vec<String>>,
  pub tools: Option<Vec<Tool>>,
  #[allow(dead_code)]
  pub tool_choice: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
  pub role: String,
  pub content: Option<serde_json::Value>,
  pub tool_calls: Option<Vec<ToolCall>>,
  pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
  #[serde(rename = "type")]
  pub tool_type: String,
  pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
  pub name: String,
  pub description: Option<String>,
  pub parameters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
  pub id: String,
  #[serde(rename = "type")]
  pub call_type: String,
  pub function: ToolCallFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
  pub name: String,
  pub arguments: String,
}

// ============================================================
// OpenAI Chat Completion 响应
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionResponse {
  pub id: String,
  pub object: String,
  pub created: i64,
  pub model: String,
  pub choices: Vec<Choice>,
  pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Choice {
  pub index: i32,
  pub message: ResponseMessage,
  pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResponseMessage {
  pub role: String,
  pub content: Option<String>,
  pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Usage {
  pub prompt_tokens: i32,
  pub completion_tokens: i32,
  pub total_tokens: i32,
}

// ============================================================
// OpenAI 流式响应
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionChunk {
  pub id: String,
  pub object: String,
  pub created: i64,
  pub model: String,
  pub choices: Vec<ChunkChoice>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChunkChoice {
  pub index: i32,
  pub delta: Delta,
  pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Delta {
  pub role: Option<String>,
  pub content: Option<String>,
  pub tool_calls: Option<Vec<DeltaToolCall>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeltaToolCall {
  pub index: i32,
  pub id: Option<String>,
  #[serde(rename = "type")]
  pub call_type: Option<String>,
  pub function: Option<DeltaToolCallFunction>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeltaToolCallFunction {
  pub name: Option<String>,
  pub arguments: Option<String>,
}

// ============================================================
// Models 列表响应
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct ModelsResponse {
  pub object: String,
  pub data: Vec<ModelInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModelInfo {
  pub id: String,
  pub object: String,
  pub created: i64,
  pub owned_by: String,
}

// ============================================================
// Kiro API 请求/响应
// ============================================================

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KiroPayload {
  pub conversation_state: ConversationState,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub profile_arn: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationState {
  pub chat_trigger_type: String,
  pub conversation_id: String,
  pub current_message: CurrentMessage,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub history: Option<Vec<HistoryItem>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentMessage {
  pub user_input_message: UserInputMessage,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInputMessage {
  pub content: String,
  pub model_id: String,
  pub origin: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user_input_message_context: Option<UserInputMessageContext>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inference_config: Option<InferenceConfig>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InferenceConfig {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub max_tokens: Option<i32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub temperature: Option<f32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub top_p: Option<f32>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInputMessageContext {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tools: Option<Vec<KiroTool>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tool_results: Option<Vec<KiroToolResult>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KiroTool {
  pub tool_specification: KiroToolSpec,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KiroToolSpec {
  pub name: String,
  pub description: String,
  pub input_schema: KiroInputSchema,
}

#[derive(Debug, Clone, Serialize)]
pub struct KiroInputSchema {
  pub json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KiroToolResult {
  pub content: Vec<KiroToolResultContent>,
  pub status: String,
  pub tool_use_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct KiroToolResultContent {
  pub text: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum HistoryItem {
  User { #[serde(rename = "userInputMessage")] user_input_message: HistoryUserMessage },
  Assistant { #[serde(rename = "assistantResponseMessage")] assistant_response_message: HistoryAssistantMessage },
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryUserMessage {
  pub content: String,
  pub model_id: String,
  pub origin: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub user_input_message_context: Option<UserInputMessageContext>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inference_config: Option<InferenceConfig>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryAssistantMessage {
  pub content: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub tool_uses: Option<Vec<KiroToolUse>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KiroToolUse {
  pub name: String,
  pub input: serde_json::Value,
  pub tool_use_id: String,
}

// ============================================================
// 错误响应
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
  pub error: ErrorDetail,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorDetail {
  pub message: String,
  #[serde(rename = "type")]
  pub error_type: String,
  pub code: Option<i32>,
}

// ============================================================
// Anthropic Messages API 请求
// ============================================================

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicMessagesRequest {
  pub model: String,
  pub messages: Vec<AnthropicMessage>,
  pub max_tokens: i32,
  #[serde(default)]
  pub system: Option<serde_json::Value>,
  #[serde(default)]
  pub stream: bool,
  pub temperature: Option<f32>,
  pub top_p: Option<f32>,
  #[allow(dead_code)]
  pub top_k: Option<i32>,
  pub stop_sequences: Option<Vec<String>>,
  pub tools: Option<Vec<AnthropicTool>>,
  #[allow(dead_code)]
  pub tool_choice: Option<serde_json::Value>,
  #[allow(dead_code)]
  pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicMessage {
  pub role: String,
  pub content: serde_json::Value, // 可以是字符串或内容块数组
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicTool {
  pub name: String,
  pub description: Option<String>,
  pub input_schema: serde_json::Value,
}

// ============================================================
// Anthropic Messages API 响应
// ============================================================

#[derive(Debug, Clone, Serialize)]
pub struct AnthropicMessagesResponse {
  pub id: String,
  #[serde(rename = "type")]
  pub response_type: String,
  pub role: String,
  pub content: Vec<AnthropicContentBlock>,
  pub model: String,
  pub stop_reason: Option<String>,
  pub stop_sequence: Option<String>,
  pub usage: AnthropicUsage,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnthropicContentBlock {
  #[serde(rename = "type")]
  pub block_type: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub text: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub id: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub input: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnthropicUsage {
  pub input_tokens: i32,
  pub output_tokens: i32,
}
