use std::net::{SocketAddr, Ipv4Addr, IpAddr};
use std::time::Duration;
use tokio::net::UdpSocket;
use tracing::{info, warn};

use pi_controller::network::protocol::{CommandPayload, TelemetryFrame, AckFrame, SystemState, CommandAction, SignedMessage};
use pi_controller::hardware::sensors::SensorPoller;
use pi_controller::handler::apply_state_transition;

const DESKTOP_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8081);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    info!("MANPADS Tactical Stress Simulator starting...");
    info!("Simulating packet drops (5%) and latency spikes (50ms-250ms)...");

    let socket = UdpSocket::bind("127.0.0.1:8080").await?;
    let mut buf = [0; 65535];
    let mut telemetry_seq = 0u64;
    let mut sensors = SensorPoller::new();
    let mut state = SystemState::Safe;
    let mut last_cmd_seq = 0u64;

    let mut telemetry_interval = tokio::time::interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            res = socket.recv_from(&mut buf) => {
                if let Ok((len, _src)) = res {
                    if rand::random::<u64>() % 20 == 0 {
                        warn!("SIMULATOR: Packet dropped (simulating EMI loss).");
                        continue;
                    }

                    let data = &buf[..len];
                    if let Ok(raw_str) = std::str::from_utf8(data) {
                        if let Ok(cmd) = CommandPayload::from_json(raw_str) {
                            if !cmd.verify_signature() {
                                warn!("Signature mismatch in simulator. Discarding.");
                                continue;
                            }

                            if cmd.seq <= last_cmd_seq && cmd.action != CommandAction::Estop {
                                continue;
                            }
                            last_cmd_seq = cmd.seq;

                            let latency_ms = 50 + (rand::random::<u64>() % 200);
                            info!("SIMULATOR: Injecting network lag of {}ms", latency_ms);
                            tokio::time::sleep(Duration::from_millis(latency_ms)).await;

                            let (new_state, err_msg) = apply_state_transition(state, cmd.action);
                            let success = err_msg.is_empty();
                            if success {
                                state = new_state;
                                match cmd.action {
                                    CommandAction::Fire => warn!("FIRE INITIATED!"),
                                    CommandAction::Estop => warn!("ESTOP COMPLIED"),
                                    _ => info!("Simulator state: {:?}", state),
                                }
                            }

                            let ack_timestamp = std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
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
                                let _ = socket.send_to(serialized.as_bytes(), &DESKTOP_ADDR).await;
                            }
                        }
                    }
                }
            }

            _ = telemetry_interval.tick() => {
                let (volt, temp, lat, lng) = sensors.poll(state);

                let mut fault_mask = 0u32;
                if temp > 70.0 {
                    fault_mask |= 4;
                }

                telemetry_seq += 1;
                let current_time = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
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
                    let _ = socket.send_to(serialized.as_bytes(), &DESKTOP_ADDR).await;
                }
            }
        }
    }
}
