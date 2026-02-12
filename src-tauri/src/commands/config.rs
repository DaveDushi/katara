use crate::config::manager::{self as config_mgr, AppSettings, ClaudeMdEntry};
use crate::error::KataraError;

#[tauri::command]
pub async fn read_claude_md(
    level: String,
    project_dir: Option<String>,
) -> Result<ClaudeMdEntry, KataraError> {
    config_mgr::read_claude_md(&level, project_dir.as_deref())
}

#[tauri::command]
pub async fn write_claude_md(path: String, content: String) -> Result<(), KataraError> {
    config_mgr::write_claude_md(&path, &content)
}

#[tauri::command]
pub async fn read_settings() -> Result<AppSettings, KataraError> {
    config_mgr::read_settings()
}

#[tauri::command]
pub async fn write_settings(settings: AppSettings) -> Result<(), KataraError> {
    config_mgr::write_settings(&settings)
}
