use std::sync::Arc;

use tauri::Emitter;
use tokio_tungstenite::tungstenite::http;

use crate::error::KataraError;
use crate::state::AppState;
use crate::websocket::protocol::{ClaudeMessage, WsEvent};

/// Starts the WebSocket server that Claude CLI processes connect to via --sdk-url.
///
/// The server accepts connections at ws://127.0.0.1:{port}/ws/cli/{sessionId}.
/// The session ID is embedded in the URL path so we can associate each
/// CLI connection with the correct session immediately on connect.
pub async fn start_ws_server(
    state: Arc<AppState>,
    app_handle: tauri::AppHandle,
) -> Result<(), KataraError> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .map_err(|e| KataraError::WebSocket(e.to_string()))?;

    let port = listener
        .local_addr()
        .map_err(|e| KataraError::WebSocket(e.to_string()))?
        .port();

    *state.ws_port.write().await = port;
    println!("[katara] WebSocket server listening on port {}", port);

    // Notify frontend of the WS port
    let _ = app_handle.emit("ws:port", port);

    while let Ok((stream, addr)) = listener.accept().await {
        println!("[katara] WebSocket connection from {}", addr);
        let state = state.clone();
        let app_handle = app_handle.clone();
        tokio::spawn(handle_connection(stream, state, app_handle));
    }

    Ok(())
}

/// Extract session ID from the WebSocket upgrade request path.
/// Expects /ws/cli/{sessionId}.
fn extract_session_id_from_request(req: &http::Request<()>) -> Option<String> {
    let path = req.uri().path();
    let parts: Vec<&str> = path.split('/').collect();
    // /ws/cli/{sessionId} -> ["", "ws", "cli", "{sessionId}"]
    if parts.len() >= 4 && parts[1] == "ws" && parts[2] == "cli" && !parts[3].is_empty() {
        Some(parts[3].to_string())
    } else {
        None
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    state: Arc<AppState>,
    app_handle: tauri::AppHandle,
) {
    // Use accept_hdr_async to inspect the HTTP upgrade request and extract
    // the session ID from the URL path before completing the handshake.
    let url_session_id: Arc<std::sync::Mutex<Option<String>>> =
        Arc::new(std::sync::Mutex::new(None));

    let ws_stream = {
        let sid_ref = url_session_id.clone();
        let callback =
            |req: &http::Request<()>,
             resp: http::Response<()>|
             -> Result<http::Response<()>, http::Response<Option<String>>> {
                let extracted = extract_session_id_from_request(req);
                println!("[katara] WS upgrade request path: {} -> session_id: {:?}", req.uri().path(), extracted);
                *sid_ref.lock().unwrap() = extracted;
                Ok(resp)
            };

        match tokio_tungstenite::accept_hdr_async(stream, callback).await {
            Ok(ws) => ws,
            Err(e) => {
                eprintln!("[katara] WebSocket handshake failed: {}", e);
                return;
            }
        }
    };

    let mut session_id = url_session_id
        .lock()
        .unwrap()
        .take()
        .unwrap_or_else(|| "unknown".to_string());

    let (mut write, mut read) = futures_util::StreamExt::split(ws_stream);

    // Create a channel for sending messages back to CLI
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(64);

    // Spawn writer task: reads from channel, writes to WebSocket
    tokio::spawn(async move {
        use futures_util::SinkExt;
        use tokio_tungstenite::tungstenite::Message;
        while let Some(msg) = rx.recv().await {
            if write.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // If we got a session ID from the URL, immediately associate the
    // WebSocket sender with that session.
    if session_id != "unknown" {
        let mut sessions = state.sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.ws_sender = Some(tx.clone());
            println!("[katara] Session {} CLI connected (from URL path)", session_id);
        } else {
            eprintln!("[katara] URL session_id {} not found in state", session_id);
        }
    }

    // Read loop: parse messages from Claude CLI.
    //
    // Each WebSocket frame may contain a single JSON object or multiple
    // newline-delimited JSON objects (NDJSON).
    use futures_util::StreamExt;

    while let Some(msg) = read.next().await {
        let msg = match msg {
            Ok(m) => m,
            Err(e) => {
                eprintln!("[katara] WebSocket read error: {}", e);
                break;
            }
        };

        let text = match msg {
            tokio_tungstenite::tungstenite::Message::Text(t) => t.to_string(),
            tokio_tungstenite::tungstenite::Message::Close(_) => break,
            _ => continue,
        };

        // NDJSON: split on newlines, parse each line (like Companion does)
        let lines: Vec<&str> = text.split('\n').filter(|l| !l.trim().is_empty()).collect();

        for line in lines {
            let line = line.trim();
            let claude_msg = match serde_json::from_str::<ClaudeMessage>(line) {
                Ok(msg) => msg,
                Err(e) => {
                    let preview = &line[..line.len().min(200)];
                    eprintln!("[katara] Failed to parse JSON: {} | {}", e, preview);
                    continue;
                }
            };

            // Handle system/init
            if let ClaudeMessage::System(ref sys) = claude_msg {
                if sys.subtype == "init" {
                    // If we didn't get session_id from URL, fall back to pending queue
                    if session_id == "unknown" {
                        let pending_id = state.pending_connections.lock().await.pop_front();
                        if let Some(pid) = pending_id {
                            session_id = pid;
                        } else if let Some(ref sid) = sys.session_id {
                            session_id = sid.clone();
                        }
                    }

                    let mut sessions = state.sessions.write().await;
                    if let Some(session) = sessions.get_mut(&session_id) {
                        session.ws_sender = Some(tx.clone());
                        session.status =
                            crate::process::session::SessionStatus::Connected;

                        // Store CLI's internal session_id for future --resume
                        if let Some(ref cli_sid) = sys.session_id {
                            session.cli_session_id = Some(cli_sid.clone());
                        }

                        // Capture model and permission mode from CLI
                        if let Some(ref model) = sys.model {
                            session.model = Some(model.clone());
                        }
                        if let Some(ref mode) = sys.permission_mode {
                            session.permission_mode = mode.clone();
                        }

                        println!(
                            "[katara] Session {} system/init received (CLI session_id: {:?}, model: {:?}, permissionMode: {:?})",
                            session_id, sys.session_id, sys.model, sys.permission_mode
                        );

                        let _ = app_handle.emit(
                            "claude:status",
                            serde_json::json!({
                                "session_id": session_id,
                                "status": "Connected",
                            }),
                        );
                    } else {
                        eprintln!(
                            "[katara] system/init: no session found for {}",
                            session_id
                        );
                    }
                }
            }

            // Mark Active on assistant/stream_event
            if matches!(
                claude_msg,
                ClaudeMessage::Assistant(_) | ClaudeMessage::StreamEvent(_)
            ) {
                let mut sessions = state.sessions.write().await;
                if let Some(session) = sessions.get_mut(&session_id) {
                    if session.status == crate::process::session::SessionStatus::Connected
                        || session.status == crate::process::session::SessionStatus::Idle
                    {
                        session.status = crate::process::session::SessionStatus::Active;
                        let _ = app_handle.emit(
                            "claude:status",
                            serde_json::json!({
                                "session_id": session_id,
                                "status": "Active",
                            }),
                        );
                    }
                }
            }

            // Track token usage from assistant messages
            if let ClaudeMessage::Assistant(ref assistant) = claude_msg {
                if let Some(ref usage) = assistant.message.usage {
                    let mut sessions = state.sessions.write().await;
                    if let Some(session) = sessions.get_mut(&session_id) {
                        session.usage_totals.add(usage);
                        let _ = app_handle.emit(
                            "claude:usage",
                            serde_json::json!({
                                "session_id": session_id,
                                "usage_totals": session.usage_totals,
                            }),
                        );
                    }
                }
            }

            // Permission-mode auto-resolve for tool approval requests.
            // Intercept before broadcast so the frontend never sees auto-handled requests.
            if let ClaudeMessage::ControlRequest(ref ctrl) = claude_msg {
                if ctrl.request.subtype == "can_use_tool" {
                    let (perm_mode, ws_sender) = {
                        let sessions = state.sessions.read().await;
                        sessions.get(&session_id).map(|s| {
                            (s.permission_mode.clone(), s.ws_sender.clone())
                        }).unwrap_or(("default".to_string(), None))
                    };

                    let auto_behavior = match perm_mode.as_str() {
                        "bypassPermissions" => Some("allow"),
                        "plan" => Some("deny"),
                        "acceptEdits" => {
                            let tool_name = ctrl.request.tool_name.as_deref().unwrap_or("");
                            if matches!(tool_name, "Edit" | "Write" | "MultiEdit" | "write_to_file" | "edit_file" | "create_file") {
                                Some("allow")
                            } else {
                                None // Ask user
                            }
                        }
                        _ => None, // "default" — ask user
                    };

                    if let Some(behavior) = auto_behavior {
                        if let (Some(ref req_id), Some(ref ws_tx)) = (&ctrl.request.request_id, &ws_sender) {
                            use crate::websocket::protocol::{
                                ControlResponseBody, ControlResponsePayload, ServerMessage,
                            };
                            let msg = ServerMessage::ControlResponse {
                                response: ControlResponseBody {
                                    subtype: "success".into(),
                                    request_id: req_id.clone(),
                                    response: ControlResponsePayload {
                                        behavior: behavior.into(),
                                        updated_input: if behavior == "allow" {
                                            Some(serde_json::json!({}))
                                        } else {
                                            None
                                        },
                                    },
                                },
                            };
                            let json = serde_json::to_string(&msg).unwrap_or_default();
                            let _ = ws_tx.send(format!("{}\n", json)).await;
                            println!(
                                "[katara] Auto-{} tool {} (permission_mode={})",
                                behavior,
                                ctrl.request.tool_name.as_deref().unwrap_or("unknown"),
                                perm_mode
                            );
                            continue; // Skip broadcast — handled automatically
                        }
                    }
                }
            }

            // Mark Idle on result
            if matches!(claude_msg, ClaudeMessage::Result(_)) {
                let mut sessions = state.sessions.write().await;
                if let Some(session) = sessions.get_mut(&session_id) {
                    session.status = crate::process::session::SessionStatus::Idle;
                    let _ = app_handle.emit(
                        "claude:status",
                        serde_json::json!({
                            "session_id": session_id,
                            "status": "Idle",
                        }),
                    );
                }
            }

            // Store in message history for persistence.
            // Skip CLI-echoed "user" messages since we already store them in send_message.
            // Skip system, keep_alive, and auth_status — they're not chat content.
            if !matches!(
                claude_msg,
                ClaudeMessage::User(_)
                    | ClaudeMessage::System(_)
                    | ClaudeMessage::KeepAlive {}
                    | ClaudeMessage::AuthStatus(_)
            ) {
                let mut sessions = state.sessions.write().await;
                if let Some(session) = sessions.get_mut(&session_id) {
                    if let Ok(val) = serde_json::to_value(&claude_msg) {
                        session.message_history.push(val);
                    }
                }
            }

            // Broadcast to event bus and frontend
            let event = WsEvent {
                session_id: session_id.clone(),
                message: claude_msg.clone(),
            };
            let _ = state.event_tx.send(event);

            let _ = app_handle.emit(
                "claude:message",
                serde_json::json!({
                    "session_id": session_id,
                    "message": claude_msg,
                }),
            );
        }
    }

    println!(
        "[katara] WebSocket connection closed for session {}",
        session_id
    );

    // Mark session as disconnected
    let mut sessions = state.sessions.write().await;
    if let Some(session) = sessions.get_mut(&session_id) {
        session.status = crate::process::session::SessionStatus::Disconnected;
        session.ws_sender = None;

        let _ = app_handle.emit(
            "claude:status",
            serde_json::json!({
                "session_id": session_id,
                "status": "Disconnected",
            }),
        );
    }
}
