use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::Emitter;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::state::AppState;
use pi_controller::network::protocol::{AckFrame, CommandAction, CommandPayload, SignedMessage, TelemetryFrame};

const CRITICAL_RETRY_COUNT: usize = 3;
const CRITICAL_RETRY_INTERVAL_MS: u64 = 200;

struct PendingCritical {
    payload: CommandPayload,
    sent_at: Instant,
    retries_remaining: usize,
}

impl PendingCritical {
    fn needs_retry(&self, now: Instant) -> bool {
        now.duration_since(self.sent_at) >= Duration::from_millis(CRITICAL_RETRY_INTERVAL_MS)
            && self.retries_remaining > 0
    }
}

pub struct PiClient {
    pub pi_addr: String,
    pub local_addr: String,
}

pub struct CommandTx {
    pub tx: mpsc::Sender<CommandPayload>,
}

impl PiClient {
    pub fn new(pi_addr: String, local_addr: String) -> Self {
        Self {
            pi_addr,
            local_addr,
        }
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

            let last_telemetry_time = Arc::new(std::sync::Mutex::new(Instant::now()));

            let pending_critical: Arc<std::sync::Mutex<Vec<PendingCritical>>> = Arc::new(std::sync::Mutex::new(Vec::new()));

            // Task A: Recv loop
            {
                let socket_recv = socket.clone();
                let state_recv = state.clone();
                let app_handle_recv = app_handle.clone();
                let telemetry_time = last_telemetry_time.clone();
                let pending = pending_critical.clone();
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
                                        warn!("Signature mismatch on Telemetry frame");
                                        continue;
                                    }

                                    if let Ok(mut lock) = telemetry_time.lock() {
                                        *lock = Instant::now();
                                    }

                                    let mut is_new_conn = false;
                                    {
                                        let mut d = state_recv.view.write().await;
                                        if !d.is_connected {
                                            d.is_connected = true;
                                            is_new_conn = true;
                                        }
                                        d.last_telemetry = Some(frame.clone());
                                    }

                                    if is_new_conn {
                                        state_recv.log_event(format!(
                                            "Connected to Edge Controller at {}",
                                            src
                                        )).await;
                                        let _ = app_handle_recv.emit("connection-status", true);
                                    }

                                    let _ = app_handle_recv.emit("telemetry-update", frame);
                                } else if let Ok(ack) = AckFrame::from_json(raw_str) {
                                    if !ack.verify_signature() {
                                        warn!("Signature mismatch on ACK frame");
                                        continue;
                                    }

                                    {
                                        let mut pc = pending.lock().unwrap();
                                        pc.retain(|p| p.payload.seq != ack.command_seq);
                                    }

                                    {
                                        let mut view = state_recv.view.write().await;
                                        view.latency_ms = 0;
                                    }

                                    state_recv.log_event(format!(
                                        "ACK received for Cmd seq={}: success={}, latency={}ms",
                                        ack.command_seq, ack.success, 0u64
                                    )).await;

                                    let _ = app_handle_recv.emit("ack-update", ack);
                                }
                            }
                            Err(e) => {
                                error!(error = %e, "UDP socket receive error");
                                tokio::time::sleep(Duration::from_millis(100)).await;
                            }
                        }
                    }
                });
            }

            // Task B: Send loop with critical command retry
            {
                let socket_send = socket.clone();
                let state_send = state.clone();
                let app_handle_send = app_handle.clone();
                let pending = pending_critical.clone();
                tokio::spawn(async move {
                    loop {
                        // Check for critical command retries
                        let retry_cmds: Vec<CommandPayload> = {
                            let mut pc = pending.lock().unwrap();
                            let now = Instant::now();
                            let mut to_retry = Vec::new();
                            for p in pc.iter_mut() {
                                if p.needs_retry(now) {
                                    let mut cmd = p.payload.clone();
                                    cmd.sign();
                                    to_retry.push(cmd);
                                    p.retries_remaining -= 1;
                                    p.sent_at = now;
                                    warn!(seq = p.payload.seq, retries_left = p.retries_remaining, "Retrying critical command");
                                }
                            }
                            to_retry
                        };
                        for cmd in retry_cmds {
                            if let Ok(serialized) = cmd.to_json() {
                                let _ = socket_send.send_to(serialized.as_bytes(), &pi_addr).await;
                            }
                        }

                        tokio::select! {
                            cmd = cmd_rx.recv() => {
                                let mut cmd = match cmd {
                                    Some(c) => c,
                                    None => break,
                                };

                                let is_critical = matches!(cmd.action, CommandAction::Fire | CommandAction::Estop);

                                cmd.sign();
                                if let Ok(serialized) = cmd.to_json() {
                                    match socket_send.send_to(serialized.as_bytes(), &pi_addr).await {
                                        Ok(_) => {
                                            tracing::debug!("Sent command seq={} to {}", cmd.seq, pi_addr);
                                            if is_critical {
                                                pending.lock().unwrap().push(PendingCritical {
                                                    payload: cmd,
                                                    sent_at: Instant::now(),
                                                    retries_remaining: CRITICAL_RETRY_COUNT,
                                                });
                                            }
                                        }
                                        Err(e) => {
                                            error!(error = %e, "Failed to send command over UDP");
                                        }
                                    }
                                }
                            }
                            _ = tokio::time::sleep(Duration::from_millis(50)) => {}
                        }

                        // Emit timeout for commands that exhausted all retries
                        let timed_out_seqs: Vec<u64> = {
                            let mut pc = pending.lock().unwrap();
                            let now = Instant::now();
                            let mut seqs = Vec::new();
                            pc.retain(|p| {
                                let total_timeout = Duration::from_millis(
                                    (CRITICAL_RETRY_COUNT as u64) * CRITICAL_RETRY_INTERVAL_MS
                                );
                                if now.duration_since(p.sent_at) > total_timeout && p.retries_remaining == 0 {
                                    seqs.push(p.payload.seq);
                                    return false;
                                }
                                true
                            });
                            seqs
                        };
                        for seq in timed_out_seqs {
                            warn!(seq = seq, "CRITICAL COMMAND TIMEOUT — no ACK received after all retries");
                            state_send.log_event(format!(
                                "CRITICAL: Command seq={} not acknowledged after {} retries",
                                seq, CRITICAL_RETRY_COUNT
                            )).await;
                            let _ = app_handle_send.emit("command-ack-timeout", seq);
                        }
                    }
                });
            }

            // Task C: Watchdog Connection Checker
            {
                let state_checker = state.clone();
                let app_handle_checker = app_handle.clone();
                let telemetry_time_checker = last_telemetry_time.clone();
                loop {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    let elapsed = telemetry_time_checker
                        .lock()
                        .map(|l| l.elapsed())
                        .unwrap_or(Duration::from_secs(10));
                    let was_connected = state_checker.view.read().await.is_connected;

                    if was_connected && elapsed > Duration::from_millis(1500) {
                        warn!(elapsed_ms = elapsed.as_millis(), "Heartbeat lost! No telemetry.");
                        {
                            let mut view = state_checker.view.write().await;
                            view.is_connected = false;
                            view.latency_ms = 0;
                        }
                        state_checker.log_event(
                            "WARNING: Connection to edge controller lost (heartbeat timeout)".to_string()
                        ).await;
                        let _ = app_handle_checker.emit("connection-status", false);
                    }
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
                    warn!(
                        "UDP bind attempt {}/5 failed (retry in {:?}): {}",
                        attempt, delay, e
                    );
                    tokio::time::sleep(delay).await;
                    let jitter = Duration::from_millis(rand::random::<u64>() % 250);
                    delay = ((delay * 2).min(Duration::from_secs(10))) + jitter;
                }
            }
        }
        None
    }
}
