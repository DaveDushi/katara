use std::sync::Arc;

use crate::error::KataraError;
use crate::state::AppState;
use crate::terminal::pty::PtyHandle;

#[tauri::command]
pub async fn spawn_terminal(
    state: tauri::State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
    rows: u16,
    cols: u16,
    cwd: Option<String>,
) -> Result<String, KataraError> {
    let id = uuid::Uuid::new_v4().to_string();
    let handle =
        PtyHandle::spawn(id.clone(), rows, cols, cwd, app_handle).map_err(KataraError::Terminal)?;
    state.terminals.write().await.insert(id.clone(), handle);
    Ok(id)
}

#[tauri::command]
pub async fn write_terminal(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
    data: String,
) -> Result<(), KataraError> {
    let terminals = state.terminals.read().await;
    let handle = terminals
        .get(&id)
        .ok_or(KataraError::Terminal(format!("Terminal {} not found", id)))?;
    handle
        .write(data.as_bytes())
        .map_err(KataraError::Terminal)?;
    Ok(())
}

#[tauri::command]
pub async fn resize_terminal(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
    rows: u16,
    cols: u16,
) -> Result<(), KataraError> {
    let terminals = state.terminals.read().await;
    let handle = terminals
        .get(&id)
        .ok_or(KataraError::Terminal(format!("Terminal {} not found", id)))?;
    handle.resize(rows, cols).map_err(KataraError::Terminal)?;
    Ok(())
}

#[tauri::command]
pub async fn kill_terminal(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), KataraError> {
    // Dropping PtyHandle closes the PTY
    state.terminals.write().await.remove(&id);
    Ok(())
}
