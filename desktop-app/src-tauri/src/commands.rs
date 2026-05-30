use crate::error::AppError;
use crate::network::pi_client::CommandTx;
use crate::state::{AppState, AppStateView};
use pi_controller::network::protocol::{CommandAction, CommandPayload};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{Manager, State};

pub fn parse_action(s: &str) -> Result<CommandAction, AppError> {
    match s {
        "arm" => Ok(CommandAction::Arm),
        "disarm" => Ok(CommandAction::Disarm),
        "fire" => Ok(CommandAction::Fire),
        "estop" => Ok(CommandAction::Estop),
        _ => Err(AppError::UnknownAction(s.to_string())),
    }
}

#[tauri::command]
pub async fn get_app_state(state: State<'_, Arc<AppState>>) -> Result<AppStateView, AppError> {
    Ok(state.view.read().await.clone())
}

#[tauri::command]
pub async fn send_operator_command(
    action_str: String,
    state: State<'_, Arc<AppState>>,
    cmd_tx: State<'_, CommandTx>,
) -> Result<u64, AppError> {
    let action = parse_action(&action_str)?;

    let auth_token = state.data.read().await.operator_token.clone();

    let timestamp_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let now = Instant::now();
    let seq;

    {
        let mut data = state.data.write().await;
        if let Some(last_time) = data.last_command_time {
            if now < last_time || now.duration_since(last_time) < Duration::from_secs(1) {
                return Err(AppError::RateLimited { wait_ms: 1000 });
            }
        }
        data.command_seq += 1;
        seq = data.command_seq;
        data.last_command_time = Some(now);
    }

    state.log_event(format!(
        "Operator requested action: {:?} (seq={})",
        action, seq
    )).await;

    let payload = CommandPayload {
        protocol_version: pi_controller::network::protocol::PROTOCOL_VERSION,
        seq,
        timestamp_ms,
        action,
        auth_token,
        hmac: String::new(),
    };

    cmd_tx.tx.send(payload).await
        .map_err(|e| AppError::NetworkSend(e.to_string()))?;

    Ok(seq)
}

#[tauri::command]
pub async fn get_settings(state: State<'_, Arc<AppState>>) -> Result<AppStateView, AppError> {
    Ok(state.view.read().await.clone())
}

#[tauri::command]
pub async fn update_settings(
    pi_ip: String,
    pi_port: u16,
    local_port: u16,
    state: State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
) -> Result<(), AppError> {
    let app_data = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Io(e.to_string()))?;
    let existing = crate::settings::Settings::load(&app_data);
    let save = crate::settings::Settings {
        pi_ip: pi_ip.clone(),
        pi_port,
        local_port,
        operator_token: existing.operator_token,
    };
    save.save(&app_data).map_err(|e| AppError::Settings(e))?;

    {
        let mut view = state.view.write().await;
        view.pi_ip = pi_ip;
        view.pi_port = pi_port;
        view.local_port = local_port;
    }
    Ok(())
}

#[tauri::command]
pub async fn export_audit_log(
    state: State<'_, Arc<AppState>>,
    app_handle: tauri::AppHandle,
) -> Result<String, AppError> {
    let app_data = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Io(e.to_string()))?;
    let path = app_data.join("session.log");
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| AppError::Io(e.to_string()))?;
    }

    let log_content = state.data.read().await.audit_log
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    std::fs::write(&path, &log_content).map_err(|e| AppError::Io(e.to_string()))?;
    Ok(path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_parsing_exhaustive() {
        for (input, expected) in [
            ("arm", CommandAction::Arm),
            ("disarm", CommandAction::Disarm),
            ("fire", CommandAction::Fire),
            ("estop", CommandAction::Estop),
        ] {
            let result = parse_action(input).unwrap();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn action_parsing_unknown_returns_error() {
        assert!(parse_action("launch_missiles").is_err());
    }
}
