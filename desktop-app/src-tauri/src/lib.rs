pub mod state;
pub mod network;
pub mod commands;

use std::sync::Arc;
use tauri::Manager;
use state::AppState;
use network::pi_client::{PiClient, CommandTx};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(AppState::new());
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(state.clone())
        .setup(move |app| {
            // Bind local UDP on 8081, send commands to Pi on 8080
            let client = PiClient::new("127.0.0.1:8080".to_string(), "127.0.0.1:8081".to_string());
            let cmd_tx = client.start(app.handle().clone(), state.clone());
            app.manage(CommandTx { tx: cmd_tx });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_state,
            commands::send_operator_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
