use std::sync::Mutex;
use pi_controller::network::protocol::TelemetryFrame;

#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppStateData {
    pub is_connected: bool,
    pub pi_ip: String,
    pub latency_ms: u64,
    pub last_telemetry: Option<TelemetryFrame>,
    pub command_seq: u64,
    pub audit_log: Vec<String>,
    #[serde(skip)]
    pub last_command_timestamp: Option<u64>,
}


pub struct AppState {
    pub data: Mutex<AppStateData>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(AppStateData {
                is_connected: false,
                pi_ip: "127.0.0.1:8080".to_string(),
                latency_ms: 0,
                last_telemetry: None,
                command_seq: 0,
                audit_log: vec!["System Initialized".to_string()],
                last_command_timestamp: None,
            }),

        }
    }

    pub fn log_event(&self, message: String) {
        if let Ok(mut d) = self.data.lock() {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            let formatted = format!("[{}] {}", timestamp, message);
            tracing::info!("{}", formatted);
            d.audit_log.push(formatted);
            // Throttle audit log to last 100 events to prevent memory overflow (M1 8GB RAM optimization)
            if d.audit_log.len() > 100 {
                d.audit_log.remove(0);
            }
        }
    }
}
