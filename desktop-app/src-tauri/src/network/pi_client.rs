use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tauri::Emitter;
use tracing::{info, warn, error};

use pi_controller::network::protocol::{CommandPayload, TelemetryFrame, AckFrame, SignedMessage};
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

        tauri::async_runtime::spawn(async move {
            let socket = match Self::bind_retry(&local_addr).await {
                Some(s) => Arc::new(s),
                None => {
                    error!("Fatal: could not bind UDP socket on {}", local_addr);
                    return;
                }
            };

            info!("Starting Pi UDP client on local {}", local_addr);

            let socket_recv = socket.clone();
            let socket_send = socket.clone();
            let state_recv = state.clone();
            let app_handle_recv = app_handle.clone();
            let last_telemetry_time = Arc::new(Mutex::new(Instant::now()));
            let last_telemetry_time_checker = last_telemetry_time.clone();
            let state_checker = state.clone();
            let app_handle_checker = app_handle.clone();

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

                            if let Ok(frame) = TelemetryFrame::from_json(raw_str) {
                                if !frame.verify_signature() {
                                    warn!("Signature mismatch on Telemetry frame! Discarding packet.");
                                    continue;
                                }

                                if let Ok(mut lock) = last_telemetry_time.lock() {
                                    *lock = Instant::now();
                                }

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

                                let _ = app_handle_recv.emit("telemetry-update", frame);
                            } else if let Ok(ack) = AckFrame::from_json(raw_str) {
                                if !ack.verify_signature() {
                                    warn!("Signature mismatch on ACK frame! Discarding.");
                                    continue;
                                }

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
                        if let Ok(mut pending) = pending_commands_send.lock() {
                            pending.insert(cmd.seq, Instant::now());
                        }

                        match socket_send.send_to(serialized.as_bytes(), &pi_addr).await {
                            Ok(_) => {
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
                let elapsed = last_telemetry_time_checker
                    .lock()
                    .map(|l| l.elapsed())
                    .unwrap_or(Duration::from_secs(10));
                let mut was_connected = false;

                if let Ok(d) = state_checker.data.lock() {
                    was_connected = d.is_connected;
                }

                if was_connected && elapsed > Duration::from_millis(1500) {
                    warn!("Heartbeat lost! No telemetry for {}ms.", elapsed.as_millis());
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

    async fn bind_retry(local_addr: &str) -> Option<UdpSocket> {
        let mut delay = Duration::from_millis(500);
        for attempt in 1..=5 {
            match UdpSocket::bind(local_addr).await {
                Ok(socket) => return Some(socket),
                Err(e) => {
                    warn!("UDP bind attempt {}/5 failed (retry in {:?}): {}", attempt, delay, e);
                    tokio::time::sleep(delay).await;
                    let jitter = Duration::from_millis(rand::random::<u64>() % 250);
                    delay = ((delay * 2).min(Duration::from_secs(10))) + jitter;
                }
            }
        }
        None
    }
}
