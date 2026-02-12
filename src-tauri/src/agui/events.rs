use serde::{Deserialize, Serialize};

// ============================================================
// AG-UI event types (Server -> CopilotKit frontend via SSE)
//
// CopilotKit v1.51 uses the AG-UI protocol from @ag-ui/core.
// Event type discriminator values MUST be SCREAMING_SNAKE_CASE.
// Field names use camelCase (handled by serde renames).
// ============================================================

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum AguiEvent {
    #[serde(rename = "RUN_STARTED")]
    RunStarted {
        #[serde(rename = "threadId")]
        thread_id: String,
        #[serde(rename = "runId")]
        run_id: String,
    },

    #[serde(rename = "RUN_FINISHED")]
    RunFinished {
        #[serde(rename = "threadId")]
        thread_id: String,
        #[serde(rename = "runId")]
        run_id: String,
    },

    #[serde(rename = "RUN_ERROR")]
    RunError {
        #[serde(rename = "threadId")]
        thread_id: String,
        #[serde(rename = "runId")]
        run_id: String,
        message: String,
    },

    #[serde(rename = "TEXT_MESSAGE_START")]
    TextMessageStart {
        #[serde(rename = "messageId")]
        message_id: String,
        role: String,
    },

    #[serde(rename = "TEXT_MESSAGE_CONTENT")]
    TextMessageContent {
        #[serde(rename = "messageId")]
        message_id: String,
        delta: String,
    },

    #[serde(rename = "TEXT_MESSAGE_END")]
    TextMessageEnd {
        #[serde(rename = "messageId")]
        message_id: String,
    },

    #[serde(rename = "TOOL_CALL_START")]
    ToolCallStart {
        #[serde(rename = "toolCallId")]
        tool_call_id: String,
        #[serde(rename = "toolCallName")]
        tool_call_name: String,
        #[serde(rename = "parentMessageId")]
        #[serde(skip_serializing_if = "Option::is_none")]
        parent_message_id: Option<String>,
    },

    #[serde(rename = "TOOL_CALL_ARGS")]
    ToolCallArgs {
        #[serde(rename = "toolCallId")]
        tool_call_id: String,
        delta: String,
    },

    #[serde(rename = "TOOL_CALL_END")]
    ToolCallEnd {
        #[serde(rename = "toolCallId")]
        tool_call_id: String,
    },

    #[serde(rename = "STATE_SNAPSHOT")]
    StateSnapshot { snapshot: serde_json::Value },

    #[serde(rename = "CUSTOM")]
    Custom {
        name: String,
        value: serde_json::Value,
    },
}

// ============================================================
// AG-UI input (CopilotKit frontend -> Server via POST)
// ============================================================

#[derive(Debug, Deserialize)]
pub struct RunAgentInput {
    #[serde(rename = "threadId")]
    pub thread_id: Option<String>,

    #[serde(rename = "runId")]
    pub run_id: Option<String>,

    pub messages: Option<Vec<serde_json::Value>>,

    pub tools: Option<Vec<serde_json::Value>>,

    pub state: Option<serde_json::Value>,

    pub context: Option<Vec<serde_json::Value>>,

    #[serde(rename = "forwardedProps")]
    pub forwarded_props: Option<serde_json::Value>,
}
