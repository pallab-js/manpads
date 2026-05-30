use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{Manager, State};
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
    state: State<'_, Arc<AppState>>,
    cmd_tx: State<'_, CommandTx>,
) -> Result<u64, String> {
    let auth_token;
    if let Ok(d) = state.data.lock() {
        auth_token = d.operator_token.clone();
    } else {
        return Err("Failed to lock AppState".to_string());
    }

    let action = match action_str.as_str() {
        "arm" => CommandAction::Arm,
        "disarm" => CommandAction::Disarm,
        "fire" => CommandAction::Fire,
        "estop" => CommandAction::Estop,
        _ => return Err(format!("Unknown action: {}", action_str)),
    };

    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let now = Instant::now();

    let seq;
    if let Ok(mut d) = state.data.lock() {
        if let Some(last_time) = d.last_command_time {
            if now < last_time || now.duration_since(last_time) < Duration::from_secs(1) {
                state.log_event("BLOCKED: Operator command rate-limited (limit 1Hz)".to_string());
                return Err("BLOCKED: Command frequency restricted to 1Hz.".to_string());
            }
        }
        d.command_seq += 1;
        seq = d.command_seq;
        d.last_command_time = Some(now);
    } else {
        return Err("Failed to lock AppState".to_string());
    }

    state.log_event(format!("Operator requested action: {:?} (seq={})", action, seq));

    let payload = CommandPayload {
        seq,
        timestamp_ms,
        action,
        auth_token,
        hmac: String::new(),
    };

    if let Err(e) = cmd_tx.tx.send(payload).await {
        let err_msg = format!("Failed to send command to network queue: {}", e);
        state.log_event(format!("ERROR: {}", err_msg));
        Err(err_msg)
    } else {
        Ok(seq)
    }
}

#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> Result<AppStateData, String> {
    if let Ok(d) = state.data.lock() {
        Ok(d.clone())
    } else {
        Err("Failed to lock AppState".to_string())
    }
}

#[tauri::command]
pub fn update_settings(
    pi_ip: String,
    pi_port: u16,
    local_port: u16,
    state: State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let app_data = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let existing = crate::settings::Settings::load(&app_data);
    let save = crate::settings::Settings {
        pi_ip: pi_ip.clone(),
        pi_port,
        local_port,
        operator_token: existing.operator_token,
    };
    save.save(&app_data)?;

    if let Ok(mut d) = state.data.lock() {
        d.pi_ip = save.pi_addr();
        d.pi_port = pi_port;
        d.local_port = local_port;
    }
    Ok(())
}

#[tauri::command]
pub fn export_audit_log(
    state: State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let app_data = app_handle.path().app_data_dir().map_err(|e| e.to_string())?;
    let path = app_data.join("session.log");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let log_content = if let Ok(d) = state.data.lock() {
        d.audit_log.iter().map(|s| s.as_str()).collect::<Vec<_>>().join("\n")
    } else {
        return Err("Failed to lock AppState".to_string());
    };

    std::fs::write(&path, &log_content).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}
