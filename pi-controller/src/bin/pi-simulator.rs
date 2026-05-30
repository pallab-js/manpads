use std::net::SocketAddr;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tracing::{info, warn};

use pi_controller::network::protocol::{CommandPayload, TelemetryFrame, AckFrame, SystemState, CommandAction};
use pi_controller::hardware::sensors::SensorPoller;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("🚀 STARTING MANPADS TACTICAL STRESS SIMULATOR...");
    info!("Simulating packet drops (5%) and latency spikes (50ms-250ms)...");

    let socket = UdpSocket::bind("127.0.0.1:8080").await?;
    let mut buf = [0; 65535];
    let mut telemetry_seq = 0u64;
    let mut sensors = SensorPoller::new();
    let mut state = SystemState::Safe;
    let mut last_cmd_seq = 0u64;

    // Telemetry tick interval (100ms = 10Hz)
    let mut telemetry_interval = tokio::time::interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            // Receive Command (Client -> Pi)
            res = socket.recv_from(&mut buf) => {
                if let Ok((len, src)) = res {
                    // Simulate random 5% packet loss
                    if rand_nanos() % 20 == 0 {
                        warn!("💥 SIMULATOR: Packet dropped (simulating high-altitude EMI loss).");
                        continue;
                    }

                    let data = &buf[..len];
                    if let Ok(raw_str) = std::str::from_utf8(data) {
                        if let Ok(mut cmd) = CommandPayload::from_json(raw_str) {
                            if !cmd.verify_signature() {
                                warn!("Signature mismatch in simulator. Discarding.");
                                continue;
                            }

                            if cmd.seq <= last_cmd_seq && cmd.action != CommandAction::Estop {
                                continue;
                            }
                            last_cmd_seq = cmd.seq;

                            // Simulate network latency spike (50ms to 250ms RTT)
                            let latency_spike = 50 + (rand_nanos() % 200);
                            info!("📶 SIMULATOR: Injecting network lag of {}ms", latency_spike);
                            tokio::time::sleep(Duration::from_millis(latency_spike)).await;

                            let mut success = true;
                            let mut err_msg = String::new();

                            match cmd.action {
                                CommandAction::Arm => {
                                    state = SystemState::Armed;
                                    info!("Simulator: Armed");
                                }
                                CommandAction::Disarm => {
                                    state = SystemState::Safe;
                                    info!("Simulator: Disarmed");
                                }
                                CommandAction::Fire => {
                                    if state == SystemState::Armed {
                                        state = SystemState::Active;
                                        warn!("🔥 Simulator: FIRE INITIATED!");
                                    } else {
                                        success = false;
                                        err_msg = "Blocked: System not Armed".to_string();
                                    }
                                }
                                CommandAction::Estop => {
                                    state = SystemState::Emergency;
                                    warn!("🚨 Simulator: ESTOP COMPLIED");
                                }
                            }

                            let ack_timestamp = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_millis() as u64;

                            let mut ack = AckFrame {
                                seq: cmd.seq,
                                timestamp_ms: ack_timestamp,
                                command_seq: cmd.seq,
                                success,
                                error_msg: err_msg,
                                hmac: String::new(),
                            };
                            ack.sign();

                            if let Ok(serialized) = ack.to_json() {
                                let dest: SocketAddr = "127.0.0.1:8081".parse().unwrap();
                                let _ = socket.send_to(serialized.as_bytes(), &dest).await;
                            }
                        }
                    }
                }
            }

            // Periodic 10Hz Telemetry Stream
            _ = telemetry_interval.tick() => {
                let (volt, temp, lat, lng) = sensors.poll(state);
                
                // 1% chance to raise a temporary sensor overheat fault for diagnostic demonstration
                let mut fault_mask = 0u32;
                if temp > 70.0 {
                    fault_mask |= 4; // Thermal fault
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
                    let dest: SocketAddr = "127.0.0.1:8081".parse().unwrap();
                    let _ = socket.send_to(serialized.as_bytes(), &dest).await;
                }
            }
        }
    }
}

fn rand_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos() as u64
}
