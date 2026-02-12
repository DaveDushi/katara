use std::sync::Arc;

use serde::Serialize;

use crate::error::KataraError;
use crate::state::AppState;

#[derive(Serialize)]
pub struct PortInfo {
    pub ws_port: u16,
    pub axum_port: u16,
}

#[tauri::command]
pub async fn get_ports(state: tauri::State<'_, Arc<AppState>>) -> Result<PortInfo, KataraError> {
    Ok(PortInfo {
        ws_port: *state.ws_port.read().await,
        axum_port: *state.axum_port.read().await,
    })
}

#[tauri::command]
pub async fn get_version() -> Result<String, KataraError> {
    Ok(env!("CARGO_PKG_VERSION").to_string())
}
