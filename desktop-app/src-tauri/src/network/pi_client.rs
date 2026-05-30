use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tauri::Emitter; // Tauri v2 event trait
use tracing::{info, warn, error};

use pi_controller::network::protocol::{CommandPayload, TelemetryFrame, AckFrame, CommandAction};
use crate::state::AppState;

pub struct PiClient {
    pub pi_addr: String,
    pub local_addr: String,
}

pub struct CommandTx {
    pub tx: mpsc::Sender<CommandPayload>,
}

impl PiClient {
    pub fn new(pi_addr: String, local_addr: String) -> Self {
        Self { pi_addr, local_addr }
    }

    pub fn start(
        self,
        app_handle: tauri::AppHandle,
        state: Arc<AppState>,
    ) -> mpsc::Sender<CommandPayload> {
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<CommandPayload>(100);
        let pi_addr = self.pi_addr.clone();
        let local_addr = self.local_addr.clone();

        // Spawn main network loop
        tokio::spawn(async move {
            info!("Starting Pi UDP client on local {}", local_addr);
            let socket = match UdpSocket::bind(&local_addr).await {
                Ok(s) => Arc::new(s),
                Err(e) => {
                    error!("Failed to bind local UDP socket: {}", e);
                    return;
                }
            };

            let socket_recv = socket.clone();
            let socket_send = socket.clone();
            let state_recv = state.clone();
            let app_handle_recv = app_handle.clone();
            let last_telemetry_time = Arc::new(Mutex::new(Instant::now()));
            let last_telemetry_time_checker = last_telemetry_time.clone();
            let state_checker = state.clone();
            let app_handle_checker = app_handle.clone();

            // Track sent commands for latency calculation (Rtt)
            let pending_commands = Arc::new(Mutex::new(std::collections::HashMap::<u64, Instant>::new()));
            let pending_commands_send = pending_commands.clone();
            let pending_commands_recv = pending_commands.clone();

            // Task A: Recv loop
            tokio::spawn(async move {
                let mut buf = [0; 65535];
                loop {
                    match socket_recv.recv_from(&mut buf).await {
                        Ok((len, src)) => {
                            let data = &buf[..len];
                            let raw_str = match std::str::from_utf8(data) {
                                Ok(s) => s,
                                Err(_) => continue,
                            };

                            // Check if Telemetry Frame
                            if raw_str.contains("systemState") {
                                if let Ok(frame) = TelemetryFrame::from_json(raw_str) {
                                    if !frame.verify_signature() {
                                        warn!("Signature mismatch on Telemetry frame! Discarding packet.");
                                        continue;
                                    }

                                    *last_telemetry_time.lock().unwrap() = Instant::now();

                                    let mut is_new_conn = false;
                                    if let Ok(mut d) = state_recv.data.lock() {
                                        if !d.is_connected {
                                            d.is_connected = true;
                                            is_new_conn = true;
                                        }
                                        d.last_telemetry = Some(frame.clone());
                                    }

                                    if is_new_conn {
                                        state_recv.log_event(format!("Connected to Edge Controller at {}", src));
                                        let _ = app_handle_recv.emit("connection-status", true);
                                    }

                                    // Stream telemetry at 10Hz
                                    let _ = app_handle_recv.emit("telemetry-update", frame);
                                }
                            }
                            // Check if ACK Frame
                            else if raw_str.contains("commandSeq") {
                                if let Ok(ack) = AckFrame::from_json(raw_str) {
                                    if !ack.verify_signature() {
                                        warn!("Signature mismatch on ACK frame! Discarding.");
                                        continue;
                                    }

                                    // Calculate Latency
                                    let mut rtt = 0;
                                    if let Ok(mut pending) = pending_commands_recv.lock() {
                                        if let Some(sent_time) = pending.remove(&ack.command_seq) {
                                            rtt = sent_time.elapsed().as_millis() as u64;
                                        }
                                    }

                                    if let Ok(mut d) = state_recv.data.lock() {
                                        d.latency_ms = rtt;
                                    }

                                    state_recv.log_event(format!(
                                        "ACK received for Cmd seq={}: success={}, latency={}ms",
                                        ack.command_seq, ack.success, rtt
                                    ));

                                    let _ = app_handle_recv.emit("ack-update", ack);
                                }
                            }
                        }
                        Err(e) => {
                            error!("UDP socket receive error: {}", e);
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        }
                    }
                }
            });

            // Task B: Send loop
            tokio::spawn(async move {
                while let Some(mut cmd) = cmd_rx.recv().await {
                    cmd.sign();
                    if let Ok(serialized) = cmd.to_json() {
                        // Store sent timestamp to compute Rtt later
                        if let Ok(mut pending) = pending_commands_send.lock() {
                            pending.insert(cmd.seq, Instant::now());
                        }

                        match socket_send.send_to(serialized.as_bytes(), &pi_addr).await {
                            Ok(_) => {
                                // TD-ONLY log
                                tracing::debug!("Sent command seq={} to {}", cmd.seq, pi_addr);
                            }
                            Err(e) => {
                                error!("Failed to send command over UDP: {}", e);
                            }
                        }
                    }
                }
            });

            // Task C: Watchdog Connection Checker (Heartbeat monitor)
            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;
                let elapsed = last_telemetry_time_checker.lock().unwrap().elapsed();
                let mut was_connected = false;
                
                if let Ok(d) = state_checker.data.lock() {
                    was_connected = d.is_connected;
                }

                if was_connected && elapsed > Duration::from_millis(1500) {
                    warn!("Heartbeat lost! No telemetry received for {}ms. Disconnecting.", elapsed.as_millis());
                    if let Ok(mut d) = state_checker.data.lock() {
                        d.is_connected = false;
                        d.latency_ms = 0;
                    }
                    state_checker.log_event("WARNING: Connection to edge controller lost (heartbeat timeout)".to_string());
                    let _ = app_handle_checker.emit("connection-status", false);
                }
            }
        });

        cmd_tx
    }
}
