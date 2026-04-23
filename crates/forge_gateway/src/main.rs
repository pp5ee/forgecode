//! ForgeCode Gateway Binary Entry Point

use std::sync::Arc;

use forge_api::API;
use forge_config::ForgeConfig;

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
    let mut gateway = forge_gateway::Gateway::new(api, config).await
        .map_err(|e| {
            log::error!("Failed to create gateway: {}", e);
            e
        })?;

    // Generate a sample token for development
    match gateway.generate_token(24, Some("Development token".to_string())).await {
        Ok(sample_token) => {
            log::info!("Sample token for testing: {}", sample_token);
            log::info!("Gateway URL: http://localhost:8080/?token={}", sample_token);
        }
        Err(e) => {
            log::error!("Failed to generate sample token: {}", e);
        }
    }

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
