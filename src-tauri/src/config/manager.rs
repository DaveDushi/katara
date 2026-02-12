use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::error::KataraError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeMdEntry {
    pub level: String,
    pub path: String,
    pub content: String,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub default_model: String,
    pub skills_directory: String,
    pub terminal_font_size: u16,
    pub terminal_font_family: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        let skills_dir = dirs::home_dir()
            .unwrap_or_default()
            .join(".claude")
            .join("skills");

        Self {
            theme: "dark".into(),
            default_model: "claude-sonnet-4-5-20250929".into(),
            skills_directory: skills_dir.display().to_string(),
            terminal_font_size: 14,
            terminal_font_family: "Consolas, Monaco, 'Courier New', monospace".into(),
        }
    }
}

/// Read a CLAUDE.md file at the given level.
pub fn read_claude_md(level: &str, project_dir: Option<&str>) -> Result<ClaudeMdEntry, KataraError> {
    let path = resolve_claude_md_path(level, project_dir)?;
    let exists = path.exists();
    let content = if exists {
        std::fs::read_to_string(&path).map_err(KataraError::Io)?
    } else {
        String::new()
    };

    Ok(ClaudeMdEntry {
        level: level.to_string(),
        path: path.display().to_string(),
        content,
        exists,
    })
}

/// Write content to a CLAUDE.md file at the given level.
pub fn write_claude_md(path: &str, content: &str) -> Result<(), KataraError> {
    if let Some(parent) = Path::new(path).parent() {
        std::fs::create_dir_all(parent).map_err(KataraError::Io)?;
    }
    std::fs::write(path, content).map_err(KataraError::Io)?;
    Ok(())
}

/// Read application settings from the config directory.
pub fn read_settings() -> Result<AppSettings, KataraError> {
    let path = settings_path();
    if path.exists() {
        let content = std::fs::read_to_string(&path).map_err(KataraError::Io)?;
        serde_json::from_str(&content).map_err(KataraError::Serde)
    } else {
        Ok(AppSettings::default())
    }
}

/// Write application settings to the config directory.
pub fn write_settings(settings: &AppSettings) -> Result<(), KataraError> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(KataraError::Io)?;
    }
    let content = serde_json::to_string_pretty(settings).map_err(KataraError::Serde)?;
    std::fs::write(&path, content).map_err(KataraError::Io)?;
    Ok(())
}

fn resolve_claude_md_path(level: &str, project_dir: Option<&str>) -> Result<PathBuf, KataraError> {
    match level {
        "user" => Ok(dirs::home_dir()
            .unwrap_or_default()
            .join(".claude")
            .join("CLAUDE.md")),
        "project" => {
            let dir = project_dir.ok_or(KataraError::Config("No project directory".into()))?;
            Ok(PathBuf::from(dir).join("CLAUDE.md"))
        }
        "local" => {
            let dir = project_dir.ok_or(KataraError::Config("No project directory".into()))?;
            Ok(PathBuf::from(dir).join(".claude").join("CLAUDE.md"))
        }
        "enterprise" => {
            // Platform-specific enterprise config location
            if cfg!(windows) {
                Ok(PathBuf::from(
                    std::env::var("PROGRAMDATA").unwrap_or_default(),
                )
                .join("claude")
                .join("CLAUDE.md"))
            } else {
                Ok(PathBuf::from("/etc/claude/CLAUDE.md"))
            }
        }
        _ => Err(KataraError::Config(format!("Unknown level: {}", level))),
    }
}

fn settings_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_default()
        .join("katara")
        .join("settings.json")
}
