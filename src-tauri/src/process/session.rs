use serde::Serialize;
use tokio::process::Child;

use crate::websocket::protocol::Usage;

/// Accumulated token usage for a session.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UsageTotals {
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
}

impl UsageTotals {
    pub fn add(&mut self, usage: &Usage) {
        self.input_tokens += usage.input_tokens;
        self.output_tokens += usage.output_tokens;
        self.cache_creation_input_tokens += usage.cache_creation_input_tokens;
        self.cache_read_input_tokens += usage.cache_read_input_tokens;
    }
}

/// Represents an active Claude Code CLI session.
pub struct Session {
    pub id: String,
    pub status: SessionStatus,
    pub working_dir: String,
    /// The spawned Claude CLI process.
    pub process: Option<Child>,
    /// Channel to send messages back to the CLI via WebSocket.
    pub ws_sender: Option<tokio::sync::mpsc::Sender<String>>,
    /// CLI's internal session ID (from system/init), used for --resume.
    pub cli_session_id: Option<String>,
    /// Message history for persistence (replayed when frontend reconnects).
    pub message_history: Vec<serde_json::Value>,
    /// Timestamp when the session was created.
    pub created_at: std::time::Instant,
    /// Model used for this session (e.g. "claude-sonnet-4-5-20250929").
    pub model: Option<String>,
    /// Permission mode: "default", "plan", "acceptEdits", "bypassPermissions".
    pub permission_mode: String,
    /// Accumulated token usage across all turns.
    pub usage_totals: UsageTotals,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum SessionStatus {
    Starting,
    Connected,
    Active,
    Idle,
    Disconnected,
    Error(String),
    Terminated,
}

impl Session {
    pub fn new(
        id: String,
        working_dir: String,
        model: Option<String>,
        permission_mode: Option<String>,
    ) -> Self {
        Self {
            id,
            status: SessionStatus::Starting,
            working_dir,
            process: None,
            ws_sender: None,
            cli_session_id: None,
            message_history: Vec::new(),
            created_at: std::time::Instant::now(),
            model,
            permission_mode: permission_mode.unwrap_or_else(|| "default".to_string()),
            usage_totals: UsageTotals::default(),
        }
    }

    /// Send a raw NDJSON message to the Claude CLI via the WebSocket.
    pub async fn send_raw(&self, message: &str) -> Result<(), String> {
        if let Some(ref tx) = self.ws_sender {
            tx.send(format!("{}\n", message))
                .await
                .map_err(|e| e.to_string())
        } else {
            Err("No WebSocket connection for this session".into())
        }
    }
}
