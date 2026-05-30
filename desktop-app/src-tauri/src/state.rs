use pi_controller::network::protocol::TelemetryFrame;
use std::collections::VecDeque;
use tokio::sync::RwLock;

use crate::audit::AuditWriter;

pub struct AppStateInternal {
    pub operator_token: String,
    pub command_seq: u64,
    pub last_command_time: Option<std::time::Instant>,
    pub audit_log: VecDeque<String>,
    pub audit_writer: Option<AuditWriter>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppStateView {
    pub is_connected: bool,
    pub pi_ip: String,
    pub pi_port: u16,
    pub local_port: u16,
    pub latency_ms: u64,
    pub last_telemetry: Option<TelemetryFrame>,
    pub audit_log: Vec<String>,
}

pub struct AppState {
    pub data: RwLock<AppStateInternal>,
    pub view: RwLock<AppStateView>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(AppStateInternal {
                operator_token: String::new(),
                command_seq: 0,
                last_command_time: None,
                audit_log: VecDeque::from(["System Initialized".to_string()]),
                audit_writer: None,
            }),
            view: RwLock::new(AppStateView {
                is_connected: false,
                pi_ip: "127.0.0.1".to_string(),
                pi_port: 8080,
                local_port: 8081,
                latency_ms: 0,
                last_telemetry: None,
                audit_log: vec!["System Initialized".to_string()],
            }),
        }
    }

    pub async fn log_event(&self, message: String) {
        let mut data = self.data.write().await;
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let formatted = format!("[{}] {}", timestamp, message);
        tracing::info!(event = %message, timestamp = timestamp, "Audit event");
        data.audit_log.push_back(formatted.clone());
        if data.audit_log.len() > 100 {
            data.audit_log.pop_front();
        }
        if let Some(writer) = &mut data.audit_writer {
            writer.write_event(&formatted);
        }
        // Sync audit log to view
        let mut view = self.view.write().await;
        view.audit_log = data.audit_log.iter().cloned().collect();
    }

    pub async fn sync_view(&self) {
        let data = self.data.read().await;
        let mut view = self.view.write().await;
        view.audit_log = data.audit_log.iter().cloned().collect();
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
