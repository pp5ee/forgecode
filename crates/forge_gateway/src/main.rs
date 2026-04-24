//! Forgecode Gateway - Main entry point

use forge_gateway::{run_server, GatewayConfig};
use tracing::info;

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting Forgecode Gateway...");

    // Load configuration
    let config = GatewayConfig::default();
    info!("Gateway configuration loaded: {:?}", config);

    // Start the HTTP server
    run_server(config).await?;

    Ok(())
}