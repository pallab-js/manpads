use thiserror::Error;

#[derive(Debug, Error, serde::Serialize)]
pub enum AppError {
    #[error("State lock poisoned")]
    LockPoisoned,

    #[error("Command rate limited: wait {wait_ms}ms")]
    RateLimited { wait_ms: u64 },

    #[error("Unknown action: {0}")]
    UnknownAction(String),

    #[error("Network send failed: {0}")]
    NetworkSend(String),

    #[error("Settings error: {0}")]
    Settings(String),

    #[error("IO error: {0}")]
    Io(String),
}

impl From<AppError> for String {
    fn from(e: AppError) -> Self {
        e.to_string()
    }
}
