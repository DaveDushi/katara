use std::collections::{HashMap, VecDeque};
use tokio::sync::{broadcast, Mutex, RwLock};

use crate::process::session::Session;
use crate::terminal::pty::PtyHandle;
use crate::websocket::protocol::WsEvent;

/// Shared application state, wrapped in Arc by Tauri and shared with Axum.
pub struct AppState {
    /// Active Claude Code sessions keyed by session ID.
    pub sessions: RwLock<HashMap<String, Session>>,

    /// Active terminal PTY instances keyed by terminal ID.
    pub terminals: RwLock<HashMap<String, PtyHandle>>,

    /// Port the WebSocket server is listening on (for Claude CLI --sdk-url).
    pub ws_port: RwLock<u16>,

    /// Port the Axum HTTP server is listening on (for CopilotKit runtimeUrl).
    pub axum_port: RwLock<u16>,

    /// Broadcast channel for WebSocket events from Claude CLI.
    /// The AG-UI bridge and frontend event forwarding subscribe here.
    pub event_tx: broadcast::Sender<WsEvent>,

    /// Queue of session IDs awaiting a WebSocket connection from Claude CLI.
    /// When spawn_session creates a session, it pushes the ID here.
    /// When a new WS connection sends system/init, we pop the first pending
    /// session and associate the connection with it.
    pub pending_connections: Mutex<VecDeque<String>>,

    /// Maps CopilotKit thread IDs to Katara session IDs for multi-session routing.
    pub thread_to_session: RwLock<HashMap<String, String>>,

    /// Reverse map: Katara session ID to CopilotKit thread ID.
    pub session_to_thread: RwLock<HashMap<String, String>>,
}

impl AppState {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            sessions: RwLock::new(HashMap::new()),
            terminals: RwLock::new(HashMap::new()),
            ws_port: RwLock::new(0),
            axum_port: RwLock::new(0),
            event_tx,
            pending_connections: Mutex::new(VecDeque::new()),
            thread_to_session: RwLock::new(HashMap::new()),
            session_to_thread: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
