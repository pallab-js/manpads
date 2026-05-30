pub mod audit;
pub mod commands;
pub mod error;
pub mod network;
pub mod settings;
pub mod state;

use network::pi_client::{CommandTx, PiClient};
use settings::Settings;
use state::AppState;
use std::sync::Arc;
use tauri::{Emitter, Manager};

use pi_controller::network::protocol::init_hmac_key;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("RUST_LOG")
                .unwrap_or_else(|_| "desktop_app=info,warn".parse().unwrap()),
        )
        .json()
        .init();

    let operator_token = std::env::var("MANPADS_OPERATOR_TOKEN")
        .expect("MANPADS_OPERATOR_TOKEN env var must be set");

    let hmac_secret =
        std::env::var("MANPADS_HMAC_SECRET").expect("MANPADS_HMAC_SECRET env var must be set");

    init_hmac_key(&hmac_secret);

    let state = Arc::new(AppState::new());

    tauri::Builder::default()
        .manage(state.clone())
        .setup(move |app| {
            // Initialize persistent audit log and settings
            let app_data_dir = app.path().app_data_dir().ok();
            let (mut pi_ip_str, mut pi_port_val, mut local_port_val) =
                ("127.0.0.1".to_string(), 8080u16, 8081u16);

            if let Some(dir) = &app_data_dir {
                let audit_log_dir = dir.join("audit");
                match audit::AuditWriter::open(&audit_log_dir) {
                    Ok(writer) => {
                        let s = state.clone();
                        tauri::async_runtime::spawn(async move {
                            s.data.write().await.audit_writer = Some(writer);
                        });
                    }
                    Err(e) => {
                        tracing::warn!("Failed to open audit log file: {}", e);
                    }
                }

                let s = Settings::load(dir);
                pi_ip_str = s.pi_ip.clone();
                pi_port_val = s.pi_port;
                local_port_val = s.local_port;

                let state_clone = state.clone();
                let ip = s.pi_ip.clone();
                let pp = s.pi_port;
                let lp = s.local_port;
                let tk = s.operator_token.clone();
                tauri::async_runtime::spawn(async move {
                    let mut view = state_clone.view.write().await;
                    view.pi_ip = ip;
                    view.pi_port = pp;
                    view.local_port = lp;
                    let mut data = state_clone.data.write().await;
                    data.operator_token = tk;
                });
            }

            // Override with env var if set
            {
                let state_clone = state.clone();
                let token = operator_token.clone();
                tauri::async_runtime::spawn(async move {
                    if std::env::var("MANPADS_OPERATOR_TOKEN").is_ok() {
                        state_clone.data.write().await.operator_token = token;
                    }
                });
            }

            let pi_addr = format!("{}:{}", pi_ip_str, pi_port_val);
            let local_addr = format!("127.0.0.1:{}", local_port_val);

            let client = PiClient::new(pi_addr, local_addr);
            let cmd_tx = client.start(app.handle().clone(), state.clone());
            app.manage(CommandTx { tx: cmd_tx });

            let state_clone = state.clone();
            if let Some(window) = app.get_webview_window("main") {
                let handle = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        let s = state_clone.clone();
                        let h = handle.clone();
                        tauri::async_runtime::spawn(async move {
                            s.log_event("Operator terminal session ended.".to_string())
                                .await;
                            let _ = h.emit("connection-status", false);
                        });
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
