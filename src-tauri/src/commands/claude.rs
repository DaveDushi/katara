use std::sync::Arc;

use serde::Serialize;
use tauri::Emitter;

use crate::error::KataraError;
use crate::process::manager;
use crate::process::session::{Session, SessionStatus};
use crate::state::AppState;
use crate::websocket::protocol::{
    ControlRequestPayload, ControlResponseBody, ControlResponsePayload, ServerMessage,
};

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub id: String,
    pub status: SessionStatus,
    pub working_dir: String,
    pub model: Option<String>,
    pub permission_mode: String,
}

#[derive(Debug, Serialize)]
pub struct SessionCost {
    pub session_id: String,
    pub model: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_input_tokens: u64,
    pub cache_read_input_tokens: u64,
    pub estimated_cost_usd: f64,
}

#[tauri::command]
pub async fn spawn_session(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    working_dir: String,
    initial_prompt: Option<String>,
    model: Option<String>,
    permission_mode: Option<String>,
) -> Result<String, KataraError> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let ws_port = *state.ws_port.read().await;

    if ws_port == 0 {
        return Err(KataraError::WebSocket(
            "WebSocket server not ready yet".into(),
        ));
    }

    // Insert session BEFORE spawning CLI so it exists when system/init arrives
    let session = Session::new(
        session_id.clone(),
        working_dir.clone(),
        model.clone(),
        permission_mode.clone(),
    );
    state
        .sessions
        .write()
        .await
        .insert(session_id.clone(), session);

    // Push to pending queue so the WS handler can match the next connection
    state
        .pending_connections
        .lock()
        .await
        .push_back(session_id.clone());

    // Notify frontend of new session
    let _ = app_handle.emit(
        "claude:status",
        serde_json::json!({
            "session_id": &session_id,
            "status": SessionStatus::Starting,
        }),
    );

    // Spawn the Claude CLI process
    let child = manager::spawn_claude(
        ws_port,
        &session_id,
        &working_dir,
        initial_prompt.as_deref(),
        model.as_deref(),
        permission_mode.as_deref(),
        None,
    )
    .await?;

    // Store the process handle
    {
        let mut sessions = state.sessions.write().await;
        if let Some(s) = sessions.get_mut(&session_id) {
            s.process = Some(child);
        }
    }

    // Start monitoring the process lifecycle
    let arc_state: Arc<AppState> = state.inner().clone();
    manager::monitor_process(arc_state, app_handle, session_id.clone());

    Ok(session_id)
}

#[tauri::command]
pub async fn kill_session(
    state: tauri::State<'_, Arc<AppState>>,
    session_id: String,
) -> Result<(), KataraError> {
    let mut sessions = state.sessions.write().await;
    if let Some(mut session) = sessions.remove(&session_id) {
        if let Some(ref mut child) = session.process {
            let _ = child.kill().await;
        }
        session.status = SessionStatus::Terminated;
    }
    drop(sessions);

    // Clean up thread <-> session mappings
    let thread_id = state
        .session_to_thread
        .write()
        .await
        .remove(&session_id);
    if let Some(tid) = thread_id {
        state.thread_to_session.write().await.remove(&tid);
    }

    Ok(())
}

#[tauri::command]
pub async fn send_message(
    state: tauri::State<'_, Arc<AppState>>,
    session_id: String,
    content: String,
) -> Result<(), KataraError> {
    // Store user message in history BEFORE forwarding to CLI (Companion pattern).
    // This ensures user messages persist even if the CLI doesn't echo them back.
    let (cli_sid, ws_tx) = {
        let mut sessions = state.sessions.write().await;
        let session = sessions
            .get_mut(&session_id)
            .ok_or(KataraError::SessionNotFound(session_id.clone()))?;

        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        session.message_history.push(serde_json::json!({
            "type": "user_message",
            "content": content,
            "timestamp": ts,
            "id": format!("user-{}", ts),
        }));

        let cli_sid = session.cli_session_id.clone().unwrap_or_default();
        let ws_tx = session.ws_sender.clone();
        (cli_sid, ws_tx)
    };

    let msg = ServerMessage::User {
        message: crate::websocket::protocol::UserContent {
            role: "user".into(),
            content,
        },
        parent_tool_use_id: None,
        session_id: cli_sid,
    };

    let json = serde_json::to_string(&msg).map_err(KataraError::Serde)?;
    let tx = ws_tx.ok_or(KataraError::WebSocket(
        "No WebSocket connection for this session".into(),
    ))?;
    tx.send(format!("{}\n", json))
        .await
        .map_err(|e| KataraError::WebSocket(e.to_string()))?;

    Ok(())
}

#[tauri::command]
pub async fn approve_tool(
    state: tauri::State<'_, Arc<AppState>>,
    session_id: String,
    request_id: String,
    approved: bool,
    updated_input: Option<serde_json::Value>,
) -> Result<(), KataraError> {
    let sessions = state.sessions.read().await;
    let session = sessions
        .get(&session_id)
        .ok_or(KataraError::SessionNotFound(session_id.clone()))?;

    // For allow responses, always include updatedInput (Companion pattern).
    // If not provided, default to empty object {}.
    let final_input = if approved {
        Some(updated_input.unwrap_or(serde_json::json!({})))
    } else {
        None
    };

    let msg = ServerMessage::ControlResponse {
        response: ControlResponseBody {
            subtype: "success".into(),
            request_id,
            response: ControlResponsePayload {
                behavior: if approved {
                    "allow".into()
                } else {
                    "deny".into()
                },
                updated_input: final_input,
            },
        },
    };

    let json = serde_json::to_string(&msg).map_err(KataraError::Serde)?;
    session
        .send_raw(&json)
        .await
        .map_err(KataraError::WebSocket)?;

    Ok(())
}

/// Send an interrupt control_request to cancel the current execution.
/// This is the same pattern Companion uses: send { type: "control_request", request: { subtype: "interrupt" } }
#[tauri::command]
pub async fn interrupt_session(
    state: tauri::State<'_, Arc<AppState>>,
    session_id: String,
) -> Result<(), KataraError> {
    let sessions = state.sessions.read().await;
    let session = sessions
        .get(&session_id)
        .ok_or(KataraError::SessionNotFound(session_id.clone()))?;

    let msg = ServerMessage::ControlRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        request: ControlRequestPayload {
            subtype: "interrupt".into(),
        },
    };

    let json = serde_json::to_string(&msg).map_err(KataraError::Serde)?;
    session
        .send_raw(&json)
        .await
        .map_err(KataraError::WebSocket)?;

    Ok(())
}

/// Return stored message history for a session (for persistence across tab switches / reconnects).
#[tauri::command]
pub async fn get_message_history(
    state: tauri::State<'_, Arc<AppState>>,
    session_id: String,
) -> Result<Vec<serde_json::Value>, KataraError> {
    let sessions = state.sessions.read().await;
    let session = sessions
        .get(&session_id)
        .ok_or(KataraError::SessionNotFound(session_id.clone()))?;

    Ok(session.message_history.clone())
}

#[tauri::command]
pub async fn list_sessions(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<SessionInfo>, KataraError> {
    let sessions = state.sessions.read().await;
    let infos: Vec<SessionInfo> = sessions
        .values()
        .map(|s| SessionInfo {
            id: s.id.clone(),
            status: s.status.clone(),
            working_dir: s.working_dir.clone(),
            model: s.model.clone(),
            permission_mode: s.permission_mode.clone(),
        })
        .collect();
    Ok(infos)
}

/// Update the permission mode for an active session.
#[tauri::command]
pub async fn set_permission_mode(
    state: tauri::State<'_, Arc<AppState>>,
    session_id: String,
    permission_mode: String,
) -> Result<(), KataraError> {
    let mut sessions = state.sessions.write().await;
    let session = sessions
        .get_mut(&session_id)
        .ok_or(KataraError::SessionNotFound(session_id.clone()))?;
    session.permission_mode = permission_mode;
    Ok(())
}

/// Get cost/usage metrics for a session.
#[tauri::command]
pub async fn get_session_cost(
    state: tauri::State<'_, Arc<AppState>>,
    session_id: String,
) -> Result<SessionCost, KataraError> {
    let sessions = state.sessions.read().await;
    let session = sessions
        .get(&session_id)
        .ok_or(KataraError::SessionNotFound(session_id.clone()))?;

    let u = &session.usage_totals;
    let model_name = session.model.as_deref().unwrap_or("claude-sonnet-4-5-20250929");

    // Pricing per million tokens (input, output, cache_write, cache_read)
    let (input_per_m, output_per_m, cache_write_per_m, cache_read_per_m) =
        if model_name.contains("opus") {
            (15.0, 75.0, 18.75, 1.5)
        } else if model_name.contains("haiku") {
            (0.80, 4.0, 1.0, 0.08)
        } else {
            // Sonnet (default)
            (3.0, 15.0, 3.75, 0.30)
        };

    let cost = (u.input_tokens as f64 * input_per_m
        + u.output_tokens as f64 * output_per_m
        + u.cache_creation_input_tokens as f64 * cache_write_per_m
        + u.cache_read_input_tokens as f64 * cache_read_per_m)
        / 1_000_000.0;

    Ok(SessionCost {
        session_id,
        model: session.model.clone(),
        input_tokens: u.input_tokens,
        output_tokens: u.output_tokens,
        cache_creation_input_tokens: u.cache_creation_input_tokens,
        cache_read_input_tokens: u.cache_read_input_tokens,
        estimated_cost_usd: cost,
    })
}

/// Resume a previous Claude CLI session using its CLI session ID.
#[tauri::command]
pub async fn resume_session(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    working_dir: String,
    cli_session_id: String,
    model: Option<String>,
    permission_mode: Option<String>,
) -> Result<String, KataraError> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let ws_port = *state.ws_port.read().await;

    if ws_port == 0 {
        return Err(KataraError::WebSocket(
            "WebSocket server not ready yet".into(),
        ));
    }

    let session = Session::new(
        session_id.clone(),
        working_dir.clone(),
        model.clone(),
        permission_mode.clone(),
    );
    state
        .sessions
        .write()
        .await
        .insert(session_id.clone(), session);

    state
        .pending_connections
        .lock()
        .await
        .push_back(session_id.clone());

    let _ = app_handle.emit(
        "claude:status",
        serde_json::json!({
            "session_id": &session_id,
            "status": SessionStatus::Starting,
        }),
    );

    let child = manager::spawn_claude(
        ws_port,
        &session_id,
        &working_dir,
        None,
        model.as_deref(),
        permission_mode.as_deref(),
        Some(&cli_session_id),
    )
    .await?;

    {
        let mut sessions = state.sessions.write().await;
        if let Some(s) = sessions.get_mut(&session_id) {
            s.process = Some(child);
        }
    }

    let arc_state: Arc<AppState> = state.inner().clone();
    manager::monitor_process(arc_state, app_handle, session_id.clone());

    Ok(session_id)
}
