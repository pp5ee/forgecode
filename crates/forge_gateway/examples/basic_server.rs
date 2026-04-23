//! Basic example of starting the ForgeCode gateway server

use forge_gateway::Gateway;
use forge_api::API;
use forge_config::ForgeConfig;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize logging
    env_logger::init();

    // Create a mock API instance (in a real implementation, this would be the actual ForgeCode API)
    let api = Arc::new(API::default());

    // Create configuration
    let config = ForgeConfig::default();

    // Create and start the gateway
    let mut gateway = Gateway::new(api, config);

    println!("Starting ForgeCode gateway...");

    // Start the server
    if let Err(e) = gateway.start().await {
        eprintln!("Failed to start gateway: {}", e);
        return Err(e);
    }

    println!("Gateway started successfully!");
    println!("Press Ctrl+C to stop the server...");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;

    println!("Shutting down gateway...");
    gateway.stop().await?;

    println!("Gateway stopped successfully!");
    Ok(())
}