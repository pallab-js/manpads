use std::sync::Mutex;
use std::collections::VecDeque;
use pi_controller::network::protocol::TelemetryFrame;

#[derive(Debug, serde::Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AppStateData {
    pub is_connected: bool,
    pub pi_ip: String,
    pub latency_ms: u64,
    pub last_telemetry: Option<TelemetryFrame>,
    pub command_seq: u64,
    pub audit_log: VecDeque<String>,
    pub pi_port: u16,
    pub local_port: u16,
    pub operator_token: String,
    #[serde(skip)]
    pub last_command_time: Option<std::time::Instant>,
}


pub struct AppState {
    pub data: Mutex<AppStateData>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
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
                audit_log: VecDeque::from(["System Initialized".to_string()]),
                pi_port: 8080,
                local_port: 8081,
                operator_token: "DEMO-OPERATOR-TOKEN-2026".to_string(),
                last_command_time: None,
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
            d.audit_log.push_back(formatted);
            if d.audit_log.len() > 100 {
                d.audit_log.pop_front();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let state = AppState::new();
        let d = state.data.lock().unwrap();
        assert!(!d.is_connected);
        assert_eq!(d.command_seq, 0);
        assert_eq!(d.audit_log.len(), 1);
        assert_eq!(d.audit_log[0], "System Initialized");
    }

    #[test]
    fn test_log_event_appends() {
        let state = AppState::new();
        state.log_event("Test event".to_string());
        let d = state.data.lock().unwrap();
        assert_eq!(d.audit_log.len(), 2);
        assert!(d.audit_log[1].ends_with("Test event"));
    }

    #[test]
    fn test_audit_log_caps_at_100() {
        let state = AppState::new();
        for i in 0..200 {
            state.log_event(format!("Event {}", i));
        }
        let d = state.data.lock().unwrap();
        assert_eq!(d.audit_log.len(), 100);
        assert!(d.audit_log[0].ends_with("Event 100"));
        assert!(d.audit_log[99].ends_with("Event 199"));
    }

    #[test]
    fn test_command_seq_increments() {
        let state = AppState::new();
        for _ in 0..5 {
            if let Ok(mut d) = state.data.lock() {
                d.command_seq += 1;
            }
        }
        let d = state.data.lock().unwrap();
        assert_eq!(d.command_seq, 5);
    }

    #[test]
    fn test_operator_token_default() {
        let state = AppState::new();
        let d = state.data.lock().unwrap();
        assert_eq!(d.operator_token, "DEMO-OPERATOR-TOKEN-2026");
    }
}
