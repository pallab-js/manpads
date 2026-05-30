use thiserror::Error;

#[derive(Debug, Error)]
pub enum ControllerError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("HMAC verification failed")]
    HmacVerification,

    #[error("Unauthorized operator token")]
    Unauthorized,

    #[error("Invalid state transition: {0}")]
    InvalidTransition(String),

    #[error("Sequence replay detected: received seq={received}, last={last}")]
    SequenceReplay { received: u64, last: u64 },

    #[error("Timestamp outside freshness window: delta={delta_ms}ms")]
    TimestampStale { delta_ms: u64 },

    #[error("E-STOP interlock is active")]
    EstopActive,

    #[error("Hardware error: {0}")]
    Hardware(String),

    #[error("Network error: {source}")]
    Network { source: std::io::Error },

    #[error("Serialization error: {source}")]
    Serialization { source: serde_json::Error },
}

impl From<std::io::Error> for ControllerError {
    fn from(source: std::io::Error) -> Self {
        Self::Network { source }
    }
}

impl From<serde_json::Error> for ControllerError {
    fn from(source: serde_json::Error) -> Self {
        Self::Serialization { source }
    }
}
