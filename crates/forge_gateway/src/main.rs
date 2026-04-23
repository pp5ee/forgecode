//! ForgeCode Gateway Binary Entry Point

use forge_gateway::Gateway;
use forge_api::API;
use forge_config::ForgeConfig;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    log::info!("Starting ForgeCode Gateway...");

    // Load configuration
    let config = ForgeConfig::load_default()
        .map_err(|e| {
            log::error!("Failed to load configuration: {}", e);
            e
        })?;

    // Initialize API
    let api = Arc::new(API::new(config.clone())
        .map_err(|e| {
            log::error!("Failed to initialize API: {}", e);
            e
        })?);

    // Create and start gateway
    let mut gateway = Gateway::new(api, config);

    // Generate a sample token for development
    let sample_token = gateway.generate_token();
    log::info!("Sample token for testing: {}", sample_token);
    log::info!("Gateway URL: http://localhost:8080/?token={}", sample_token);

    // Start the gateway server
    gateway.start().await
        .map_err(|e| {
            log::error!("Failed to start gateway: {}", e);
            e
        })?;

    log::info!("ForgeCode Gateway is running!");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await
        .map_err(|e| {
            log::error!("Failed to wait for shutdown signal: {}", e);
            e
        })?;

    log::info!("Shutting down ForgeCode Gateway...");

    // Stop the gateway gracefully
    gateway.stop().await
        .map_err(|e| {
            log::error!("Failed to stop gateway: {}", e);
            e
        })?;

    log::info!("ForgeCode Gateway stopped successfully");
    Ok(())
}