use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum KataraError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Session not found: {0}")]
    SessionNotFound(String),

    #[error("Terminal error: {0}")]
    Terminal(String),

    #[error("Config error: {0}")]
    Config(String),

    #[error("Skill error: {0}")]
    Skill(String),

    #[error("Process error: {0}")]
    Process(String),
}

// Tauri commands require Serialize on error types
impl Serialize for KataraError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
