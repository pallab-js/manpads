use pi_controller::config::ControllerConfig;
use pi_controller::daemon::ControllerDaemon;
use pi_controller::hardware::simulated::SimulatedHardware;
use pi_controller::network::protocol::{
    init_hmac_key, AckFrame, CommandAction, CommandPayload, SignedMessage, PROTOCOL_VERSION,
};
use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::net::UdpSocket;

use pi_controller::network::protocol::TelemetryFrame;

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Bind a temporary socket to discover a free port, then return it.
async fn find_free_port() -> (SocketAddr, UdpSocket) {
    let s = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    let addr = s.local_addr().unwrap();
    (addr, s)
}

/// Receive until we get an AckFrame, skipping TelemetryFrames.
async fn recv_ack(sock: &UdpSocket, timeout: Duration) -> Result<AckFrame, &'static str> {
    let deadline = tokio::time::Instant::now() + timeout;
    let mut buf = [0u8; 65535];
    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            return Err("timeout waiting for ACK");
        }
        let (len, _) = tokio::time::timeout(remaining, sock.recv_from(&mut buf))
            .await
            .map_err(|_| "timeout")?
            .map_err(|_| "recv error")?;
        let raw = std::str::from_utf8(&buf[..len]).map_err(|_| "invalid utf8")?;
        // Try ACK first
        if let Ok(ack) = AckFrame::from_json(raw) {
            return Ok(ack);
        }
        // Skip telemetry
        if TelemetryFrame::from_json(raw).is_ok() {
            continue;
        }
        // Unknown frame
        tracing::warn!("Unknown frame: {}", raw);
    }
}

#[tokio::test]
async fn test_arm_disarm_sequence_via_udp() {
    init_hmac_key("test-secret-key-32byteslongenough");

    // Desktop socket — receives ACKs from the daemon
    let (desktop_addr, desktop_sock) = find_free_port().await;

    // Daemon listens here — use a temp socket to get a free port, then let it go
    let (daemon_addr, _probe) = find_free_port().await;
    drop(_probe);

    let config = ControllerConfig {
        listen_addr: daemon_addr,
        desktop_addr,
        hmac_secret: "test-secret-key-32byteslongenough".into(),
        operator_token: "test-op-token".into(),
        watchdog_timeout_ms: 500,
        telemetry_interval_ms: 50,
    };

    let hardware = SimulatedHardware::new();
    let daemon = ControllerDaemon::new(config, hardware);
    let shutdown = daemon.cancellation_token();
    tokio::spawn(daemon.run());

    // Give daemon time to bind
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Send Arm command to daemon's listen port
    let mut cmd = CommandPayload {
        protocol_version: PROTOCOL_VERSION,
        seq: 1,
        timestamp_ms: current_timestamp_ms(),
        action: CommandAction::Arm,
        auth_token: "test-op-token".into(),
        hmac: String::new(),
    };
    cmd.sign();
    let serialized = cmd.to_json().unwrap();
    desktop_sock
        .send_to(serialized.as_bytes(), daemon_addr)
        .await
        .unwrap();

    // Expect ACK with success=true (telemetry arrives first — skip it)
    let ack = recv_ack(&desktop_sock, Duration::from_secs(3))
        .await
        .expect("Should receive ACK within timeout");
    assert!(ack.verify_signature(), "ACK signature should be valid");
    assert!(ack.success, "Arm command should succeed");
    assert_eq!(ack.command_seq, 1);

    shutdown.cancel();
}

#[tokio::test]
async fn test_unauthorized_token_rejected() {
    init_hmac_key("test-secret-key-32byteslongenough");

    let (desktop_addr, desktop_sock) = find_free_port().await;
    let (daemon_addr, _probe) = find_free_port().await;
    drop(_probe);

    let config = ControllerConfig {
        listen_addr: daemon_addr,
        desktop_addr,
        hmac_secret: "test-secret-key-32byteslongenough".into(),
        operator_token: "real-op-token".into(),
        watchdog_timeout_ms: 500,
        telemetry_interval_ms: 50,
    };

    let hardware = SimulatedHardware::new();
    let daemon = ControllerDaemon::new(config, hardware);
    let shutdown = daemon.cancellation_token();
    tokio::spawn(daemon.run());

    tokio::time::sleep(Duration::from_millis(200)).await;

    // Send command with wrong token
    let mut cmd = CommandPayload {
        protocol_version: PROTOCOL_VERSION,
        seq: 1,
        timestamp_ms: current_timestamp_ms(),
        action: CommandAction::Fire,
        auth_token: "wrong-token".into(),
        hmac: String::new(),
    };
    cmd.sign();
    let serialized = cmd.to_json().unwrap();
    desktop_sock
        .send_to(serialized.as_bytes(), daemon_addr)
        .await
        .unwrap();

    let ack = recv_ack(&desktop_sock, Duration::from_secs(3))
        .await
        .expect("Should receive ACK within timeout");
    assert!(!ack.success, "Unauthorized command should fail");
    assert!(ack.error_msg.contains("ACCESS DENIED"));

    shutdown.cancel();
}
