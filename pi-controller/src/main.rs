use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tracing::{error, info, warn};

use pi_controller::handler::apply_state_transition;
use pi_controller::hardware::actuators::Actuators;
use pi_controller::hardware::interlock::HardwareInterlock;
use pi_controller::hardware::sensors::SensorPoller;
use pi_controller::network::protocol::{
    AckFrame, CommandAction, CommandPayload, SignedMessage, SystemState, TelemetryFrame,
};

const DESKTOP_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize subscriber for clear log traces
    tracing_subscriber::fmt::init();
    info!("MANPADS Edge Controller daemon booting up...");

    // Bind listener UDP socket on 8080 (Pi receiver)
    let socket = Arc::new(UdpSocket::bind("127.0.0.1:8080").await?);
    let socket_send = socket.clone();

    // Setup state and hardware layers
    let system_state = Arc::new(Mutex::new(SystemState::Safe));
    let system_state_recv = system_state.clone();
    let system_state_watchdog = system_state.clone();

    let sensors = Arc::new(Mutex::new(SensorPoller::new()));
    let sensors_send = sensors.clone();

    let actuators = Arc::new(Mutex::new(Actuators::new()));
    let actuators_recv = actuators.clone();
    let actuators_watchdog = actuators.clone();

    let interlock = Arc::new(Mutex::new(HardwareInterlock::new()));
    let interlock_recv = interlock.clone();
    let interlock_send = interlock.clone();
    let interlock_watchdog = interlock.clone();

    // Track sequence validation
    let last_command_seq = Arc::new(Mutex::new(0u64));
    let last_command_seq_recv = last_command_seq.clone();

    let operator_token = std::env::var("MANPADS_OPERATOR_TOKEN")
        .unwrap_or_else(|_| "DEMO-OPERATOR-TOKEN-2026".to_string());

    // Keep track of the last time a valid command was received (watchdog system)
    let last_command_time = Arc::new(Mutex::new(Instant::now()));
    let last_command_time_recv = last_command_time.clone();
    let last_command_time_watchdog = last_command_time.clone();

    // Task A: Incoming Command Listener UDP socket on 8080
    tokio::spawn(async move {
        let mut buf = [0; 65535];
        loop {
            match socket.recv_from(&mut buf).await {
                Ok((len, _src)) => {
                    let data = &buf[..len];
                    let raw_str = match std::str::from_utf8(data) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };

                    let cmd = match CommandPayload::from_json(raw_str) {
                        Ok(c) => c,
                        Err(e) => {
                            warn!("Failed to parse command frame JSON: {}", e);
                            continue;
                        }
                    };

                    if !cmd.verify_signature() {
                        warn!("Signature verification failed! Discarding packet.");
                        continue;
                    }

                    if cmd.auth_token != operator_token {
                        warn!("Access Denied: Invalid operator token! seq={}", cmd.seq);
                        let ack_timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;
                        let mut ack = AckFrame {
                            seq: cmd.seq,
                            timestamp_ms: ack_timestamp,
                            command_seq: cmd.seq,
                            success: false,
                            error_msg: "ACCESS DENIED: Unauthorized operator session token"
                                .to_string(),
                            hmac: String::new(),
                        };
                        ack.sign();
                        if let Ok(serialized) = ack.to_json() {
                            let _ = socket.send_to(serialized.as_bytes(), &DESKTOP_ADDR).await;
                        }
                        continue;
                    }

                    let mut seq_ok = false;
                    {
                        let mut last_seq = last_command_seq_recv.lock().await;
                        if cmd.seq > *last_seq || cmd.action == CommandAction::Estop {
                            *last_seq = cmd.seq;
                            seq_ok = true;
                        }
                    }

                    if !seq_ok {
                        warn!(
                            "Command sequence sequence older than processed. Ignoring seq={}",
                            cmd.seq
                        );
                        continue;
                    }

                    *last_command_time_recv.lock().await = Instant::now();

                    let mut err_msg = String::new();
                    let mut transition_success = true;

                    let is_estop = interlock_recv.lock().await.is_estop_active();
                    if is_estop
                        && cmd.action != CommandAction::Estop
                        && cmd.action != CommandAction::Disarm
                    {
                        err_msg = "BLOCKED: Safety E-STOP interlock active".to_string();
                        transition_success = false;
                    } else {
                        let mut state = system_state_recv.lock().await;
                        let (new_state, transition_err) =
                            apply_state_transition(*state, cmd.action);
                        if transition_err.is_empty() {
                            *state = new_state;
                            let mut act = actuators_recv.lock().await;
                            act.set_state(new_state);
                            match cmd.action {
                                CommandAction::Fire => act.trigger_fire(),
                                CommandAction::Disarm => {
                                    interlock_recv.lock().await.software_estop = false;
                                }
                                CommandAction::Estop => {
                                    interlock_recv.lock().await.software_estop = true;
                                }
                                _ => {}
                            }
                            drop(act);
                        } else {
                            err_msg = transition_err;
                            transition_success = false;
                        }
                    }

                    // Send ACK back
                    let ack_timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;

                    let mut ack = AckFrame {
                        seq: cmd.seq,
                        timestamp_ms: ack_timestamp,
                        command_seq: cmd.seq,
                        success: transition_success,
                        error_msg: err_msg,
                        hmac: String::new(),
                    };
                    ack.sign();

                    if let Ok(serialized) = ack.to_json() {
                        let _ = socket.send_to(serialized.as_bytes(), &DESKTOP_ADDR).await;
                    }
                }
                Err(e) => {
                    error!("UDP receive error in Pi server: {}", e);
                }
            }
        }
    });

    // Task B: Periodic Telemetry Broadcast loop at 10Hz (100ms) to Laptop 8081
    let local_socket_send = socket_send.clone();
    tokio::spawn(async move {
        let mut telemetry_seq = 0u64;

        loop {
            tokio::time::sleep(Duration::from_millis(100)).await;

            let state = *system_state.lock().await;
            let (volt, temp, lat, lng) = sensors_send.lock().await.poll(state);

            // Build Fault Mask
            let mut fault_mask = 0u32;
            if interlock_send.lock().await.is_estop_active() {
                fault_mask |= 2; // GPIO_INTERLOCK_ERR
            }
            if temp > 75.0 {
                fault_mask |= 4; // THERMAL_CRITICAL
            }

            telemetry_seq += 1;
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64;

            let mut frame = TelemetryFrame {
                seq: telemetry_seq,
                timestamp_ms: current_time,
                system_state: state,
                battery_voltage: volt,
                temperature: temp,
                gps_latitude: lat,
                gps_longitude: lng,
                fault_mask,
                hmac: String::new(),
            };
            frame.sign();

            if let Ok(serialized) = frame.to_json() {
                let _ = local_socket_send
                    .send_to(serialized.as_bytes(), &DESKTOP_ADDR)
                    .await;
            }
        }
    });

    // Task C: Hardware watchdog connection monitor
    // If armed or active, and we lose connection (no telemetry poll/heartbeats) from the controller, we disarm immediately to safe state.
    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;

        let last_time = *last_command_time_watchdog.lock().await;
        let mut state = system_state_watchdog.lock().await;

        if (*state == SystemState::Armed || *state == SystemState::Active)
            && last_time.elapsed() > Duration::from_millis(3000)
        {
            warn!("WATCHDOG INTERLOCK: Heartbeat to controller lost for 3000ms. Disarming system to SAFE.");
            *state = SystemState::Safe;
            actuators_watchdog.lock().await.set_state(SystemState::Safe);
        }

        // If physical button Estop triggers, transition to Emergency state
        if interlock_watchdog.lock().await.is_estop_active() && *state != SystemState::Emergency {
            *state = SystemState::Emergency;
            actuators_watchdog
                .lock()
                .await
                .set_state(SystemState::Emergency);
        }
    }
}
