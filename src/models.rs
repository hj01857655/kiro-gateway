use serde::{Deserialize, Serialize};
use serde_json::Value;

// ============ OpenAI 格式 ============

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIRequest {
    pub model: Option<String>,
    pub messages: Vec<OpenAIMessage>,
    #[serde(default)]
    pub stream: bool,
    pub tools: Option<Vec<OpenAITool>>,
    #[allow(dead_code)]
    pub max_tokens: Option<i32>,
    #[allow(dead_code)]
    pub temperature: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: Option<MessageContent>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_call_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageUrl {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAITool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: OpenAIFunction,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenAIFunction {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<Value>,
}

// OpenAI 响应
#[derive(Debug, Serialize)]
pub struct OpenAIChunk {
    pub id: String,
    pub object: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<OpenAIChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

#[derive(Debug, Serialize)]
pub struct OpenAIChoice {
    pub index: i32,
    pub delta: OpenAIDelta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Default)]
pub struct OpenAIDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallDelta>>,
}

#[derive(Debug, Serialize)]
pub struct ToolCallDelta {
    pub index: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub call_type: Option<String>,
    pub function: FunctionCallDelta,
}

#[derive(Debug, Serialize)]
pub struct FunctionCallDelta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Usage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
    pub total_tokens: i32,
}

// ============ Anthropic 格式 ============

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicRequest {
    pub model: Option<String>,
    pub messages: Vec<AnthropicMessage>,
    pub system: Option<String>,
    #[serde(default)]
    pub stream: bool,
    pub tools: Option<Vec<AnthropicTool>>,
    #[allow(dead_code)]
    pub max_tokens: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnthropicMessage {
    pub role: String,
    pub content: AnthropicContent,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AnthropicContent {
    Text(String),
    Blocks(Vec<AnthropicBlock>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum AnthropicBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ImageSource },
    #[serde(rename = "tool_use")]
    ToolUse { id: String, name: String, input: Value },
    #[serde(rename = "tool_result")]
    ToolResult { tool_use_id: String, content: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ImageSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub media_type: String,
    pub data: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnthropicTool {
    pub name: String,
    pub description: Option<String>,
    pub input_schema: Option<Value>,
}

// ============ Kiro 格式 ============

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KiroRequest {
    pub conversation_state: ConversationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_arn: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationState {
    pub conversation_id: String,
    pub chat_trigger_type: String,
    pub current_message: CurrentMessage,
    pub history: Vec<HistoryMessage>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentMessage {
    pub user_input_message: UserInputMessage,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInputMessage {
    /// content 是字符串，不是数组
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model_id: Option<String>,
    /// origin 必须是 "AI_EDITOR"
    pub origin: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_input_message_context: Option<UserInputMessageContext>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<KiroImage>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInputMessageContext {
    /// tools 放在这里
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<KiroTool>>,
    /// toolResults 放在这里（用于多轮对话）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_results: Option<Vec<ToolResultContent>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KiroTool {
    /// 注意：是 toolSpecification 不是 toolSpec
    pub tool_specification: ToolSpec,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSpec {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub input_schema: InputSchema,
}

#[derive(Debug, Clone, Serialize)]
pub struct InputSchema {
    pub json: Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct KiroImage {
    pub image: KiroImageData,
}

#[derive(Debug, Clone, Serialize)]
pub struct KiroImageData {
    pub format: String,
    pub source: KiroImageSource,
}

#[derive(Debug, Clone, Serialize)]
pub struct KiroImageSource {
    pub bytes: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum HistoryMessage {
    User(UserHistoryMessage),
    Assistant(AssistantHistoryMessage),
    ToolResult(ToolResultMessage),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserHistoryMessage {
    pub user_input_message: UserHistoryContent,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserHistoryContent {
    pub content: String,
    pub user_intent: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantHistoryMessage {
    pub assistant_response_message: AssistantHistoryContent,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantHistoryContent {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use: Option<Vec<KiroToolUse>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KiroToolUse {
    pub tool_use_id: String,
    pub name: String,
    pub input: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResultMessage {
    pub tool_result: ToolResultContent,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResultContent {
    pub tool_use_id: String,
    pub status: String,
    pub content: Vec<TextContent>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TextContent {
    pub text: String,
}

// ============ Kiro 事件 ============

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KiroEvent {
    // 文本响应 - 直接 content 字段
    pub content: Option<String>,
    
    // 工具调用
    #[serde(rename = "toolUseId")]
    pub tool_use_id: Option<String>,
    pub name: Option<String>,
    pub input: Option<Value>,
    
    // 代码块
    pub language: Option<String>,
    // content 字段已在上面定义
    
    // Token 使用统计
    #[allow(dead_code)]
    pub unit: Option<String>,
    pub usage: Option<f64>,
    
    // 上下文使用率
    pub context_usage_percentage: Option<f64>,
    
    // 消息元数据
    pub message_id: Option<String>,
    #[allow(dead_code)]
    pub conversation_id: Option<String>,
    
    // Thinking block
    pub text: Option<String>,
    pub signature: Option<String>,
    
    // 错误
    pub reason: Option<String>,
    pub message: Option<String>,
}

// 以下结构体保留用于文档参考，但当前使用扁平的 KiroEvent
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct AssistantResponseEvent {
    pub content: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KiroToolUseEvent {
    pub tool_use_id: String,
    pub name: String,
    pub input: Value,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageMetadataEvent {
    pub message_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataEvent {
    pub token_usage: Option<TokenUsage>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    pub output_tokens: Option<i32>,
    pub total_tokens: Option<i32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextUsageEvent {
    pub context_usage_percentage: Option<f32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct CodeEvent {
    pub language: Option<String>,
    pub content: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ReasoningContentEvent {
    pub text: String,
    pub signature: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct InvalidStateEvent {
    pub reason: Option<String>,
    pub message: Option<String>,
}

