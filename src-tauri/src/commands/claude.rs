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
}

#[tauri::command]
pub async fn spawn_session(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    working_dir: String,
    initial_prompt: Option<String>,
) -> Result<String, KataraError> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let ws_port = *state.ws_port.read().await;

    if ws_port == 0 {
        return Err(KataraError::WebSocket(
            "WebSocket server not ready yet".into(),
        ));
    }

    // Insert session BEFORE spawning CLI so it exists when system/init arrives
    let session = Session::new(session_id.clone(), working_dir.clone());
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
        })
        .collect();
    Ok(infos)
}
