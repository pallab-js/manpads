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

    info!("MANPADS Tactical Stress Simulator starting...");
    info!("Simulating packet drops (5%) and latency spikes (50ms-250ms)...");

    let config = ControllerConfig::from_env()?;
    init_hmac_key(&config.hmac_secret);

    let hardware = SimulatedHardware::new()
        .with_packet_drop_rate(0.05)
        .with_latency_range(50..=250);

    let daemon = ControllerDaemon::new(config, hardware);
    daemon.run().await?;

    Ok(())
}
