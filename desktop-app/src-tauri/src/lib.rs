pub mod state;
pub mod network;
pub mod commands;
pub mod settings;

use std::sync::Arc;
use tauri::{Emitter, Manager};
use state::AppState;
use settings::Settings;
use network::pi_client::{PiClient, CommandTx};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt::init();
    let state = Arc::new(AppState::new());

    let operator_token = std::env::var("MANPADS_OPERATOR_TOKEN")
        .unwrap_or_else(|_| "DEMO-OPERATOR-TOKEN-2026".to_string());

    tauri::Builder::default()
        .manage(state.clone())
        .setup(move |app| {
            let app_data = app.path().app_data_dir().ok();
            if let Some(dir) = &app_data {
                let s = Settings::load(dir);
                if let Ok(mut d) = state.data.lock() {
                    d.pi_ip = s.pi_addr();
                    d.pi_port = s.pi_port;
                    d.local_port = s.local_port;
                    d.operator_token = s.operator_token.clone();
                }
            }

            // Override with env var if set
            if let Ok(mut d) = state.data.lock() {
                if std::env::var("MANPADS_OPERATOR_TOKEN").is_ok() {
                    d.operator_token = operator_token.clone();
                }
            }

            let pi_addr;
            let local_addr;
            if let Ok(d) = state.data.lock() {
                pi_addr = format!("{}:{}", &d.pi_ip, d.pi_port);
                local_addr = format!("127.0.0.1:{}", d.local_port);
            } else {
                pi_addr = "127.0.0.1:8080".to_string();
                local_addr = "127.0.0.1:8081".to_string();
            }

            let client = PiClient::new(pi_addr, local_addr);
            let cmd_tx = client.start(app.handle().clone(), state.clone());
            app.manage(CommandTx { tx: cmd_tx });

            let state_clone = state.clone();
            if let Some(window) = app.get_webview_window("main") {
                let handle = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        state_clone.log_event("Operator terminal session ended.".to_string());
                        let _ = handle.emit("connection-status", false);
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::send_operator_command,
            commands::get_settings,
            commands::update_settings,
            commands::export_audit_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
