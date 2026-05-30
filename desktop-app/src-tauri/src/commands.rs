use std::sync::Arc;
use tauri::State;
use pi_controller::network::protocol::{CommandPayload, CommandAction};
use crate::state::{AppState, AppStateData};
use crate::network::pi_client::CommandTx;

#[tauri::command]
pub fn get_app_state(state: State<'_, Arc<AppState>>) -> Result<AppStateData, String> {
    if let Ok(d) = state.data.lock() {
        Ok(d.clone())
    } else {
        Err("Failed to lock AppState".to_string())
    }
}

#[tauri::command]
pub async fn send_operator_command(
    action_str: String,
    auth_token: String,
    state: State<'_, Arc<AppState>>,
    cmd_tx: State<'_, CommandTx>,
) -> Result<u64, String> {
    // 1. Session Token Validation
    if auth_token != "DEMO-OPERATOR-TOKEN-2026" {
        return Err("ACCESS DENIED: Unauthorized operator session token".to_string());
    }

    let action = match action_str.as_str() {
        "arm" => CommandAction::Arm,
        "disarm" => CommandAction::Disarm,
        "fire" => CommandAction::Fire,
        "estop" => CommandAction::Estop,
        _ => return Err(format!("Unknown action: {}", action_str)),
    };

    let timestamp_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    // 2. Command Debounce / Rate Limiter Check (1Hz frequency limit)
    let mut seq = 0;
    let mut rate_limit_ok = true;
    if let Ok(mut d) = state.data.lock() {
        if let Some(last_time) = d.last_command_timestamp {
            if timestamp_ms - last_time < 1000 {
                rate_limit_ok = false;
            }
        }
        if rate_limit_ok {
            d.command_seq += 1;
            seq = d.command_seq;
            d.last_command_timestamp = Some(timestamp_ms);
        }
    } else {
        return Err("Failed to lock AppState".to_string());
    }

    if !rate_limit_ok {
        state.log_event("BLOCKED: Operator command rate-limited (limit 1Hz)".to_string());
        return Err("BLOCKED: Command frequency restricted to 1Hz.".to_string());
    }

    state.log_event(format!("Operator requested action: {:?} (seq={})", action, seq));

    let payload = CommandPayload {
        seq,
        timestamp_ms,
        action,
        auth_token,
        hmac: String::new(), // Signed dynamically in send loop
    };

    if let Err(e) = cmd_tx.tx.send(payload).await {
        let err_msg = format!("Failed to send command to network queue: {}", e);
        state.log_event(format!("ERROR: {}", err_msg));
        Err(err_msg)
    } else {
        Ok(seq)
    }
}
