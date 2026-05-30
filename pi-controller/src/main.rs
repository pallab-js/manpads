use tracing::info;

use pi_controller::config::ControllerConfig;
use pi_controller::daemon::ControllerDaemon;
use pi_controller::hardware::simulated::SimulatedHardware;
use pi_controller::network::protocol::init_hmac_key;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_env("RUST_LOG")
                .unwrap_or_else(|_| "pi_controller=info,warn".parse().unwrap()),
        )
        .json()
        .init();

    info!(subsystem = "controller", "MANPADS Edge Controller daemon booting up...");

    let config = ControllerConfig::from_env()?;
    init_hmac_key(&config.hmac_secret);

    #[cfg(not(debug_assertions))]
    info!("Release build — running with production settings");

    info!(
        listen_addr = %config.listen_addr,
        desktop_addr = %config.desktop_addr,
        watchdog_ms = config.watchdog_timeout_ms,
        telemetry_ms = config.telemetry_interval_ms,
        "Controller configuration loaded"
    );

    let hardware = SimulatedHardware::new();
    let daemon = ControllerDaemon::new(config, hardware);
    daemon.run().await?;

    Ok(())
}
