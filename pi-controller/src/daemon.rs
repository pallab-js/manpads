use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::config::ControllerConfig;
use crate::error::ControllerError;
use crate::handler::apply_state_transition;
use crate::hardware::{HardwareBackend, SensorReading};
use crate::network::protocol::{
    fault_flags, AckFrame, CommandAction, CommandPayload, SignedMessage, SystemState,
    TelemetryFrame,
};

pub struct ControllerDaemon<H: HardwareBackend> {
    config: ControllerConfig,
    hardware: Arc<Mutex<H>>,
    system_state: Arc<RwLock<SystemState>>,
    last_command_time: Arc<Mutex<Instant>>,
    shutdown: CancellationToken,
}

impl<H: HardwareBackend> ControllerDaemon<H> {
    pub fn new(config: ControllerConfig, hardware: H) -> Self {
        Self {
            config,
            hardware: Arc::new(Mutex::new(hardware)),
            system_state: Arc::new(RwLock::new(SystemState::Safe)),
            last_command_time: Arc::new(Mutex::new(Instant::now())),
            shutdown: CancellationToken::new(),
        }
    }

    pub fn cancellation_token(&self) -> CancellationToken {
        self.shutdown.clone()
    }

    pub async fn run(self) -> Result<(), ControllerError> {
        let socket = Arc::new(UdpSocket::bind(self.config.listen_addr).await?);
        let shutdown = self.shutdown.clone();

        let daemon = Arc::new(self);

        let recv_handle = {
            let daemon = daemon.clone();
            let socket = socket.clone();
            let shutdown = shutdown.clone();
            tokio::spawn(async move {
                daemon.command_receiver_loop(socket, shutdown).await;
            })
        };

        let telemetry_handle = {
            let daemon = daemon.clone();
            let socket = socket.clone();
            let shutdown = shutdown.clone();
            tokio::spawn(async move {
                daemon.telemetry_sender_loop(socket, shutdown).await;
            })
        };

        let watchdog_handle = {
            let daemon = daemon.clone();
            let shutdown = shutdown.clone();
            tokio::spawn(async move {
                daemon.watchdog_loop(shutdown).await;
            })
        };

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Shutdown signal received, stopping daemon...");
            }
            _ = shutdown.cancelled() => {}
        }

        daemon.shutdown.cancel();

        let _ = recv_handle.await;
        let _ = telemetry_handle.await;
        let _ = watchdog_handle.await;

        info!("Controller daemon shut down cleanly");
        Ok(())
    }

    async fn command_receiver_loop(
        self: Arc<Self>,
        socket: Arc<UdpSocket>,
        shutdown: CancellationToken,
    ) {
        let operator_token = self.config.operator_token.clone();
        let desktop_addr = self.config.desktop_addr;
        let last_command_seq = Arc::new(Mutex::new(0u64));

        let mut buf = [0; 65535];
        loop {
            tokio::select! {
                _ = shutdown.cancelled() => break,
                result = socket.recv_from(&mut buf) => {
                    let (len, _src) = match result {
                        Ok(v) => v,
                        Err(e) => {
                            error!(error = %e, "UDP receive error");
                            continue;
                        }
                    };

                    let raw_str = match std::str::from_utf8(&buf[..len]) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };

                    let cmd = match CommandPayload::from_json(raw_str) {
                        Ok(c) => c,
                        Err(e) => {
                            warn!(error = %e, "Failed to parse command frame");
                            continue;
                        }
                    };

                    if cmd.protocol_version != crate::network::protocol::PROTOCOL_VERSION {
                        warn!(received = cmd.protocol_version, expected = crate::network::protocol::PROTOCOL_VERSION, "Protocol version mismatch");
                        continue;
                    }

                    if !cmd.verify_signature() {
                        warn!(seq = cmd.seq, "Signature verification failed");
                        continue;
                    }

                    if cmd.auth_token != operator_token {
                        warn!(seq = cmd.seq, "Access Denied: Invalid operator token");
                        let ack_timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_millis() as u64;
                        let mut ack = AckFrame {
                            protocol_version: crate::network::protocol::PROTOCOL_VERSION,
                            seq: cmd.seq,
                            timestamp_ms: ack_timestamp,
                            command_seq: cmd.seq,
                            success: false,
                            error_msg: "ACCESS DENIED: Unauthorized operator session token".to_string(),
                            hmac: String::new(),
                        };
                        ack.sign();
                        if let Ok(serialized) = ack.to_json() {
                            let _ = socket.send_to(serialized.as_bytes(), &desktop_addr).await;
                        }
                        continue;
                    }

                    let now_ms = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;
                    const MAX_CLOCK_SKEW_MS: u64 = 5_000;
                    if now_ms.abs_diff(cmd.timestamp_ms) > MAX_CLOCK_SKEW_MS {
                        warn!(
                            delta = now_ms.abs_diff(cmd.timestamp_ms),
                            seq = cmd.seq,
                            "Command timestamp out of freshness window"
                        );
                        continue;
                    }

                    {
                        let mut last_seq = last_command_seq.lock().await;
                        if cmd.seq <= *last_seq && cmd.action != CommandAction::Estop {
                            warn!(seq = cmd.seq, "Duplicate/older sequence ignored");
                            continue;
                        }
                        *last_seq = cmd.seq;
                    }

                    *self.last_command_time.lock().await = Instant::now();

                    let mut err_msg = String::new();
                    let mut transition_success = true;
                    let is_estop = self.hardware.lock().await.is_estop_active();

                    if is_estop && cmd.action != CommandAction::Estop && cmd.action != CommandAction::Disarm {
                        err_msg = "BLOCKED: Safety E-STOP interlock active".to_string();
                        transition_success = false;
                    } else {
                        let mut state = self.system_state.write().await;
                        let (new_state, transition_err) = apply_state_transition(*state, cmd.action);
                        if transition_err.is_empty() {
                            *state = new_state;
                            let mut hw = self.hardware.lock().await;
                            hw.set_actuator_state(new_state);
                            match cmd.action {
                                CommandAction::Fire => hw.trigger_fire(),
                                CommandAction::Disarm => hw.clear_software_estop(),
                                CommandAction::Estop => hw.activate_software_estop(),
                                _ => {}
                            }
                        } else {
                            err_msg = transition_err;
                            transition_success = false;
                        }
                    }

                    let ack_timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;

                    let mut ack = AckFrame {
                        protocol_version: crate::network::protocol::PROTOCOL_VERSION,
                        seq: cmd.seq,
                        timestamp_ms: ack_timestamp,
                        command_seq: cmd.seq,
                        success: transition_success,
                        error_msg: err_msg,
                        hmac: String::new(),
                    };
                    ack.sign();

                    if let Ok(serialized) = ack.to_json() {
                        let _ = socket.send_to(serialized.as_bytes(), &desktop_addr).await;
                    }
                }
            }
        }
    }

    async fn telemetry_sender_loop(
        self: Arc<Self>,
        socket: Arc<UdpSocket>,
        shutdown: CancellationToken,
    ) {
        let desktop_addr = self.config.desktop_addr;
        let interval = Duration::from_millis(self.config.telemetry_interval_ms);
        let mut telemetry_seq = 0u64;

        loop {
            tokio::select! {
                _ = shutdown.cancelled() => break,
                _ = tokio::time::sleep(interval) => {
                    let state = *self.system_state.read().await;
                    let reading: SensorReading;
                    let is_estop: bool;
                    {
                        let mut hw = self.hardware.lock().await;
                        reading = hw.poll_sensors(state);
                        is_estop = hw.is_estop_active();
                    }

                    let mut fault_mask = 0u32;
                    if is_estop {
                        fault_mask |= fault_flags::GPIO_INTERLOCK_ERR;
                    }
                    if reading.temperature > 75.0 {
                        fault_mask |= fault_flags::THERMAL_CRITICAL;
                    }
                    if reading.battery_voltage < 10.5 {
                        fault_mask |= fault_flags::BATTERY_LOW;
                    }

                    telemetry_seq += 1;
                    let current_time = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64;

                    let mut frame = TelemetryFrame {
                        protocol_version: crate::network::protocol::PROTOCOL_VERSION,
                        seq: telemetry_seq,
                        timestamp_ms: current_time,
                        system_state: state,
                        battery_voltage: reading.battery_voltage,
                        temperature: reading.temperature,
                        gps_latitude: reading.gps_latitude,
                        gps_longitude: reading.gps_longitude,
                        fault_mask,
                        hmac: String::new(),
                    };
                    frame.sign();

                    if let Ok(serialized) = frame.to_json() {
                        let _ = socket.send_to(serialized.as_bytes(), &desktop_addr).await;
                    }
                }
            }
        }
    }

    async fn watchdog_loop(self: Arc<Self>, shutdown: CancellationToken) {
        loop {
            tokio::select! {
                _ = shutdown.cancelled() => break,
                _ = tokio::time::sleep(Duration::from_millis(500)) => {
                    let last_time = *self.last_command_time.lock().await;
                    let mut state = self.system_state.write().await;

                    if (*state == SystemState::Armed || *state == SystemState::Active)
                        && last_time.elapsed() > Duration::from_millis(self.config.watchdog_timeout_ms)
                    {
                        error!(
                            elapsed_ms = last_time.elapsed().as_millis(),
                            "WATCHDOG: Heartbeat lost, disarming to SAFE"
                        );
                        *state = SystemState::Safe;
                        self.hardware.lock().await.set_actuator_state(SystemState::Safe);
                    }

                    if self.hardware.lock().await.is_estop_active() && *state != SystemState::Emergency {
                        *state = SystemState::Emergency;
                        self.hardware.lock().await.set_actuator_state(SystemState::Emergency);
                    }
                }
            }
        }
    }
}
