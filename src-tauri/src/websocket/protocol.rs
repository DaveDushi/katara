use serde::{Deserialize, Serialize};

// ============================================================
// Claude CLI -> Server (inbound NDJSON messages)
// ============================================================

/// Top-level message from Claude CLI, dispatched by `type` field.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ClaudeMessage {
    #[serde(rename = "system")]
    System(SystemMessage),

    #[serde(rename = "assistant")]
    Assistant(AssistantMessage),

    #[serde(rename = "result")]
    Result(ResultMessage),

    #[serde(rename = "stream_event")]
    StreamEvent(StreamEventMessage),

    #[serde(rename = "control_request")]
    ControlRequest(ControlRequestMessage),

    #[serde(rename = "tool_progress")]
    ToolProgress(serde_json::Value),

    #[serde(rename = "tool_use_summary")]
    ToolUseSummary(serde_json::Value),

    #[serde(rename = "keep_alive")]
    KeepAlive {},

    // CLI echoes back user messages (with tool_result content)
    #[serde(rename = "user")]
    User(serde_json::Value),

    // Auth status events
    #[serde(rename = "auth_status")]
    AuthStatus(serde_json::Value),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SystemMessage {
    pub subtype: String,
    pub session_id: Option<String>,
    pub tools: Option<Vec<String>>,
    pub model: Option<String>,
    pub cwd: Option<String>,
    #[serde(rename = "permissionMode")]
    pub permission_mode: Option<String>,
    pub claude_code_version: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssistantMessage {
    pub message: AssistantContent,
    pub session_id: String,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssistantContent {
    pub id: String,
    pub role: String,
    pub model: String,
    pub content: Vec<ContentBlock>,
    pub stop_reason: Option<String>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },

    #[serde(rename = "tool_use")]
    ToolUse {
        id: String,
        name: String,
        input: serde_json::Value,
    },

    #[serde(rename = "tool_result")]
    ToolResult {
        tool_use_id: String,
        content: serde_json::Value,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Usage {
    pub input_tokens: u64,
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResultMessage {
    pub result: Option<String>,
    pub subtype: Option<String>,
    pub session_id: Option<String>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StreamEventMessage {
    pub event: StreamEventPayload,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StreamEventPayload {
    #[serde(rename = "type")]
    pub event_type: String,
    pub delta: Option<StreamDelta>,
    pub index: Option<u64>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StreamDelta {
    #[serde(rename = "type")]
    pub delta_type: String,
    pub text: Option<String>,
    pub partial_json: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ControlRequestMessage {
    pub request: ControlRequestBody,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ControlRequestBody {
    pub subtype: String,
    pub request_id: Option<String>,
    pub tool_name: Option<String>,
    pub tool_use_id: Option<String>,
    pub input: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: serde_json::Value,
}

// ============================================================
// Server -> Claude CLI (outbound NDJSON messages)
// ============================================================

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "user")]
    User {
        message: UserContent,
        parent_tool_use_id: Option<String>,
        session_id: String,
    },

    #[serde(rename = "control_response")]
    ControlResponse { response: ControlResponseBody },

    #[serde(rename = "keep_alive")]
    KeepAlive {},

    #[serde(rename = "control_request")]
    ControlRequest {
        request_id: String,
        request: ControlRequestPayload,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct ControlRequestPayload {
    pub subtype: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserContent {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ControlResponseBody {
    pub subtype: String,
    pub request_id: String,
    pub response: ControlResponsePayload,
}

#[derive(Debug, Clone, Serialize)]
pub struct ControlResponsePayload {
    pub behavior: String,
    #[serde(rename = "updatedInput")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_input: Option<serde_json::Value>,
}

// ============================================================
// Internal event bus type
// ============================================================

/// Wrapper for broadcasting Claude messages with session context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEvent {
    pub session_id: String,
    pub message: ClaudeMessage,
}
