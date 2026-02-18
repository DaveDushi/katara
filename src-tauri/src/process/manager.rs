use std::sync::Arc;
use tokio::process::Command;

use tauri::Emitter;

use crate::error::KataraError;
use crate::process::session::SessionStatus;
use crate::state::AppState;

/// Spawns a Claude CLI process connected to our WebSocket server.
///
/// With `--sdk-url`, Claude CLI opens a WebSocket back to us for all communication.
/// The `-p` flag provides the initial prompt to start a conversation turn.
/// Subsequent messages are sent via the WebSocket (ServerMessage::User).
pub async fn spawn_claude(
    ws_port: u16,
    session_id: &str,
    working_dir: &str,
    initial_prompt: Option<&str>,
    model: Option<&str>,
    permission_mode: Option<&str>,
    resume_session_id: Option<&str>,
) -> Result<tokio::process::Child, KataraError> {
    // Embed session ID in the URL path so the WS server can identify the session
    // on connect (same pattern as Companion: /ws/cli/{sessionId})
    let ws_url = format!("ws://127.0.0.1:{}/ws/cli/{}", ws_port, session_id);

    let mut args = vec![
        "--sdk-url".to_string(),
        ws_url,
        "--print".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--input-format".to_string(),
        "stream-json".to_string(),
        "--verbose".to_string(),
    ];

    // Model selection (e.g. "claude-sonnet-4-5-20250929", "claude-opus-4-5-20250918")
    if let Some(m) = model {
        if !m.is_empty() {
            args.push("--model".to_string());
            args.push(m.to_string());
        }
    }

    // Permission mode (default, plan, acceptEdits, bypassPermissions)
    if let Some(mode) = permission_mode {
        if mode != "default" && !mode.is_empty() {
            args.push("--permission-mode".to_string());
            args.push(mode.to_string());
        }
    }

    // Resume a previous CLI session
    if let Some(resume_id) = resume_session_id {
        if !resume_id.is_empty() {
            args.push("--resume".to_string());
            args.push(resume_id.to_string());
        }
    }

    // If an initial prompt is provided, use -p to kick off the first turn.
    // Otherwise pass -p "" as a required placeholder for headless/SDK mode
    // (Companion pattern: CLI needs -p to enter prompt mode with --sdk-url).
    if let Some(prompt) = initial_prompt {
        args.push("-p".to_string());
        args.push(prompt.to_string());
    } else {
        args.push("-p".to_string());
        args.push(String::new());
    }

    println!(
        "[katara] Spawning Claude CLI: claude {}",
        args.join(" ")
    );

    let mut child = Command::new("claude")
        .args(&args)
        .current_dir(working_dir)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| {
            KataraError::Process(format!(
                "Failed to spawn Claude CLI (is it installed?): {}",
                e
            ))
        })?;

    // Capture stderr in a background task for debugging
    if let Some(stderr) = child.stderr.take() {
        let sid = session_id.to_string();
        tokio::spawn(async move {
            use tokio::io::AsyncBufReadExt;
            let reader = tokio::io::BufReader::new(stderr);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                eprintln!("[katara][stderr:{}] {}", &sid[..8], line);
            }
        });
    }

    // Capture stdout in a background task for debugging
    if let Some(stdout) = child.stdout.take() {
        let sid = session_id.to_string();
        tokio::spawn(async move {
            use tokio::io::AsyncBufReadExt;
            let reader = tokio::io::BufReader::new(stdout);
            let mut lines = reader.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                println!("[katara][stdout:{}] {}", &sid[..8], line);
            }
        });
    }

    println!(
        "[katara] Spawned Claude CLI for session {} in {}",
        session_id, working_dir
    );

    Ok(child)
}

/// Monitors a Claude CLI process and updates session status when it exits.
pub fn monitor_process(
    state: Arc<AppState>,
    app_handle: tauri::AppHandle,
    session_id: String,
) {
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let mut sessions = state.sessions.write().await;
            let Some(session) = sessions.get_mut(&session_id) else {
                break; // Session was removed
            };

            if let Some(ref mut child) = session.process {
                match child.try_wait() {
                    Ok(Some(exit_status)) => {
                        let new_status = if exit_status.success() {
                            SessionStatus::Terminated
                        } else {
                            SessionStatus::Error(format!(
                                "Process exited with code {}",
                                exit_status.code().unwrap_or(-1)
                            ))
                        };
                        println!(
                            "[katara] Claude CLI for session {} exited: {:?}",
                            session_id, exit_status
                        );
                        session.status = new_status.clone();
                        session.ws_sender = None;

                        let _ = app_handle.emit(
                            "claude:status",
                            serde_json::json!({
                                "session_id": session_id,
                                "status": new_status,
                            }),
                        );
                        break;
                    }
                    Ok(None) => {} // Still running
                    Err(e) => {
                        eprintln!(
                            "[katara] Error checking process for session {}: {}",
                            session_id, e
                        );
                        break;
                    }
                }
            } else {
                break;
            }
        }
    });
}

/// Check if the Claude CLI is available and supports --sdk-url.
pub async fn check_claude_cli() -> Result<bool, KataraError> {
    let output = Command::new("claude")
        .arg("--help")
        .output()
        .await
        .map_err(|e| KataraError::Process(format!("Claude CLI not found: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let help_text = format!("{}{}", stdout, stderr);
    let has_sdk_url = help_text.contains("sdk-url");

    Ok(has_sdk_url)
}
