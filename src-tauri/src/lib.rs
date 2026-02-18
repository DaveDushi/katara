pub mod agui;
pub mod commands;
pub mod config;
pub mod error;
pub mod process;
pub mod skills;
pub mod state;
pub mod terminal;
pub mod websocket;

use std::sync::Arc;
use state::AppState;

pub fn run() {
    let state = Arc::new(AppState::new());

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state.clone())
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let state_for_ws = state.clone();
            let state_for_axum = state.clone();

            // Spawn WebSocket server for Claude CLI connections
            tauri::async_runtime::spawn(async move {
                if let Err(e) = websocket::server::start_ws_server(state_for_ws, app_handle.clone()).await {
                    eprintln!("WebSocket server error: {}", e);
                }
            });

            // Spawn Axum HTTP server for AG-UI (CopilotKit runtimeUrl)
            let app_handle_axum = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = agui::server::start_agui_server(state_for_axum, app_handle_axum).await {
                    eprintln!("AG-UI server error: {}", e);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Claude session commands
            commands::claude::spawn_session,
            commands::claude::kill_session,
            commands::claude::send_message,
            commands::claude::approve_tool,
            commands::claude::interrupt_session,
            commands::claude::get_message_history,
            commands::claude::list_sessions,
            commands::claude::set_permission_mode,
            commands::claude::get_session_cost,
            commands::claude::resume_session,
            // Terminal commands
            commands::terminal::spawn_terminal,
            commands::terminal::write_terminal,
            commands::terminal::resize_terminal,
            commands::terminal::kill_terminal,
            // Config commands
            commands::config::read_claude_md,
            commands::config::write_claude_md,
            commands::config::read_settings,
            commands::config::write_settings,
            // Skill commands
            commands::skills::list_skills,
            commands::skills::read_skill,
            commands::skills::write_skill,
            commands::skills::delete_skill,
            // App commands
            commands::app::get_ports,
            commands::app::get_version,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Katara");
}
