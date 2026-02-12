use std::convert::Infallible;
use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::Request,
    response::sse::{Event, KeepAlive, Sse},
    routing::{get, post},
    Json, Router,
};
use futures_util::stream::Stream;
use tokio_stream::StreamExt;
use tower_http::cors::CorsLayer;

use tauri::Emitter;

use crate::agui::bridge::{translate_claude_message, BridgeState};
use crate::agui::events::{AguiEvent, RunAgentInput};
use crate::error::KataraError;
use crate::state::AppState;
use crate::websocket::protocol::ClaudeMessage;

/// Creates the Axum router with AG-UI endpoints.
///
/// CopilotKit v1.51 uses the AG-UI protocol with these endpoints:
///   - POST /agent/{agentId}/run  — main SSE streaming endpoint
///   - GET  /info                 — agent discovery
///   - POST /agent/{agentId}/stop/{threadId} — stop a running agent
///
/// We also keep /api/copilotkit as a fallback for older CopilotKit versions.
fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // AG-UI v1.51 endpoints (primary)
        .route("/agent/{agent_id}/run", post(agui_handler_with_agent))
        .route("/agent/{agent_id}/connect", post(agui_handler_with_agent))
        // Legacy / fallback endpoints
        .route("/api/copilotkit", post(agui_handler_legacy))
        // Info / discovery (GET for REST transport, POST for single transport)
        .route("/info", get(info_handler).post(info_handler_post))
        .route("/api/copilotkit/info", get(info_handler).post(info_handler_post))
        // Catch-all fallback for debugging unmatched requests
        .fallback(|req: Request<axum::body::Body>| async move {
            println!(
                "[katara] Unmatched request: {} {}",
                req.method(),
                req.uri()
            );
            (axum::http::StatusCode::NOT_FOUND, "Not Found")
        })
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// GET /api/copilotkit/info — CopilotKit runtime discovery endpoint.
///
/// Returns agent metadata so CopilotKit knows what agents are available.
/// CopilotKit expects agents as an object keyed by agent ID, not an array.
async fn info_handler() -> Json<serde_json::Value> {
    println!("[katara] /info endpoint hit — returning agent discovery response");
    Json(serde_json::json!({
        "agents": {
            "default": {
                "description": "Claude Code AI agent"
            }
        },
        "version": "1.0.0"
    }))
}

/// POST /info — CopilotKit "single" transport info endpoint.
///
/// Same response as GET /info but accepts POST with `{ "method": "info" }` body.
async fn info_handler_post() -> Json<serde_json::Value> {
    println!("[katara] /info endpoint hit (POST) — returning agent discovery response");
    Json(serde_json::json!({
        "agents": {
            "default": {
                "description": "Claude Code AI agent"
            }
        },
        "version": "1.0.0"
    }))
}

/// POST /agent/{agentId}/run — AG-UI SSE endpoint (CopilotKit v1.51).
/// Route with path parameter delegates to the shared handler.
async fn agui_handler_with_agent(
    State(state): State<Arc<AppState>>,
    Path(agent_id): Path<String>,
    Json(input): Json<RunAgentInput>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("[katara] AG-UI run request for agent: {}", agent_id);
    agui_handler_inner(state, input).await
}

/// POST /api/copilotkit — legacy fallback endpoint.
async fn agui_handler_legacy(
    State(state): State<Arc<AppState>>,
    Json(input): Json<RunAgentInput>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("[katara] AG-UI run request (legacy endpoint)");
    agui_handler_inner(state, input).await
}

/// Shared AG-UI handler logic.
///
/// Receives RunAgentInput from CopilotKit, forwards the user message to Claude
/// via WebSocket, and streams back AG-UI events as SSE.
async fn agui_handler_inner(
    state: Arc<AppState>,
    input: RunAgentInput,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let thread_id = input
        .thread_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let run_id = input
        .run_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    let (tx, rx) = tokio::sync::mpsc::channel::<AguiEvent>(128);

    // Spawn background task to bridge Claude messages to AG-UI events
    let state_clone = state.clone();
    let thread_id_clone = thread_id.clone();
    let run_id_clone = run_id.clone();

    tokio::spawn(async move {
        // 1. Emit RunStarted
        let _ = tx
            .send(AguiEvent::RunStarted {
                thread_id: thread_id_clone.clone(),
                run_id: run_id_clone.clone(),
            })
            .await;

        // 2. Extract last user message from CopilotKit input
        let user_message = input
            .messages
            .as_ref()
            .and_then(|msgs| {
                msgs.iter()
                    .rev()
                    .find(|m| m.get("role").and_then(|r| r.as_str()) == Some("user"))
            })
            .and_then(|m| m.get("content").and_then(|c| c.as_str()))
            .unwrap_or("")
            .to_string();

        if user_message.is_empty() {
            let _ = tx
                .send(AguiEvent::RunError {
                    thread_id: thread_id_clone,
                    run_id: run_id_clone,
                    message: "No user message provided".into(),
                })
                .await;
            return;
        }

        // 3. Build Gen-UI tool context from CopilotKit's tools array.
        //    This tells Claude about frontend-registered actions it can invoke.
        let tools_context = if let Some(ref tools) = input.tools {
            let tool_descriptions: Vec<String> = tools
                .iter()
                .filter_map(|t| {
                    let name = t.get("name")?.as_str()?;
                    let desc = t
                        .get("description")
                        .and_then(|d| d.as_str())
                        .unwrap_or("No description");
                    let schema = t.get("jsonSchema").or(t.get("parameters"));
                    Some(format!(
                        "- **{}**: {}\n  Parameters: {}",
                        name,
                        desc,
                        schema
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| "none".to_string())
                    ))
                })
                .collect();

            if tool_descriptions.is_empty() {
                String::new()
            } else {
                format!(
                    "\n\n[AVAILABLE UI ACTIONS - You can call these as tool_use to render rich UI components in the chat for the user:]\n{}\n\nTo use an action, output a tool_use block with the action name and parameters.\n\n",
                    tool_descriptions.join("\n")
                )
            }
        } else {
            String::new()
        };

        // 4. Combine tools context + user message
        let full_message = if tools_context.is_empty() {
            user_message.clone()
        } else {
            format!("{}{}", tools_context, user_message)
        };

        // 5. Find the active session and send the message.
        //    Wait up to 15s for a CLI to connect (handles race where
        //    CopilotKit sends a message before Claude CLI finishes connecting).
        let (cli_sid, ws_tx) = {
            let mut found = None;
            for attempt in 0..30 {
                let mut sessions = state_clone.sessions.write().await;

                // Log session state on first attempt for debugging
                if attempt == 0 {
                    let session_info: Vec<String> = sessions
                        .iter()
                        .map(|(id, s)| {
                            format!(
                                "{}(ws={}, status={:?})",
                                &id[..8.min(id.len())],
                                s.ws_sender.is_some(),
                                s.status
                            )
                        })
                        .collect();
                    println!(
                        "[katara] AG-UI looking for active session. {} session(s): [{}]",
                        sessions.len(),
                        session_info.join(", ")
                    );
                }

                let session = sessions.values_mut().find(|s| s.ws_sender.is_some());
                if let Some(session) = session {
                    let ts = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis();
                    session.message_history.push(serde_json::json!({
                        "type": "user_message",
                        "content": user_message,
                        "timestamp": ts,
                        "id": format!("user-{}", ts),
                    }));

                    let cli_sid = session.cli_session_id.clone().unwrap_or_default();
                    let ws_tx = session.ws_sender.clone();
                    if attempt > 0 {
                        println!("[katara] AG-UI found active session after {}ms wait", attempt * 500);
                    }
                    found = Some((cli_sid, ws_tx));
                    break;
                }

                drop(sessions); // Release lock before sleeping
                if attempt < 29 {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                }
            }

            match found {
                Some(result) => result,
                None => {
                    println!("[katara] AG-UI: No session with ws_sender found after 15s wait");
                    let _ = tx
                        .send(AguiEvent::RunError {
                            thread_id: thread_id_clone,
                            run_id: run_id_clone,
                            message: "No active Claude session. Start a session first.".into(),
                        })
                        .await;
                    return;
                }
            }
        };

        if let Some(ws_tx) = ws_tx {
            let msg = serde_json::json!({
                "type": "user",
                "message": { "role": "user", "content": full_message },
                "parent_tool_use_id": null,
                "session_id": cli_sid
            });
            let _ = ws_tx.send(format!("{}\n", msg)).await;
        }

        // 6. Subscribe to Claude events and translate to AG-UI
        let mut event_rx = state_clone.event_tx.subscribe();
        let mut bridge = BridgeState::new();

        loop {
            match event_rx.recv().await {
                Ok(ws_event) => {
                    let agui_events = translate_claude_message(
                        &ws_event.message,
                        &thread_id_clone,
                        &run_id_clone,
                        &mut bridge,
                    );

                    let mut is_finished = false;
                    for event in agui_events {
                        if matches!(event, AguiEvent::RunFinished { .. }) {
                            is_finished = true;
                        }
                        if tx.send(event).await.is_err() {
                            return; // Client disconnected
                        }
                    }

                    if is_finished {
                        break;
                    }

                    // Also break on Result message directly
                    if matches!(ws_event.message, ClaudeMessage::Result(_)) {
                        break;
                    }
                }
                Err(_) => break, // Broadcast channel closed
            }
        }
    });

    // Convert mpsc receiver to SSE stream
    let stream = tokio_stream::wrappers::ReceiverStream::new(rx).map(|event| {
        let json = serde_json::to_string(&event).unwrap_or_default();
        Ok::<_, Infallible>(Event::default().data(json))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// Starts the Axum HTTP server and emits the port to the frontend.
pub async fn start_agui_server(
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

    *state.axum_port.write().await = port;
    println!("[katara] AG-UI server listening on port {}", port);

    // Notify frontend of the AG-UI port (CopilotKit runtimeUrl)
    let _ = app_handle.emit("agui:port", port);

    let router = create_router(state);
    axum::serve(listener, router.into_make_service())
        .await
        .map_err(|e| KataraError::WebSocket(e.to_string()))?;

    Ok(())
}
