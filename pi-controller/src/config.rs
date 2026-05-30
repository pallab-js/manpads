use crate::error::ControllerError;

#[derive(Debug, Clone)]
pub struct ControllerConfig {
    pub listen_addr: std::net::SocketAddr,
    pub desktop_addr: std::net::SocketAddr,
    pub hmac_secret: String,
    pub operator_token: String,
    pub watchdog_timeout_ms: u64,
    pub telemetry_interval_ms: u64,
}

impl ControllerConfig {
    pub fn from_env() -> Result<Self, ControllerError> {
        Ok(Self {
            listen_addr: std::env::var("MANPADS_LISTEN_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:8080".into())
                .parse()
                .map_err(|e| ControllerError::Config(format!("Invalid MANPADS_LISTEN_ADDR: {e}")))?,
            desktop_addr: std::env::var("MANPADS_DESKTOP_ADDR")
                .unwrap_or_else(|_| "127.0.0.1:8081".into())
                .parse()
                .map_err(|e| ControllerError::Config(format!("Invalid MANPADS_DESKTOP_ADDR: {e}")))?,
            hmac_secret: std::env::var("MANPADS_HMAC_SECRET")
                .map_err(|_| ControllerError::Config("MANPADS_HMAC_SECRET must be set".into()))?,
            operator_token: std::env::var("MANPADS_OPERATOR_TOKEN")
                .map_err(|_| ControllerError::Config("MANPADS_OPERATOR_TOKEN must be set".into()))?,
            watchdog_timeout_ms: std::env::var("MANPADS_WATCHDOG_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1500),
            telemetry_interval_ms: std::env::var("MANPADS_TELEMETRY_INTERVAL_MS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
        })
    }
}
