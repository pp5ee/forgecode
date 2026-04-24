//! Forge Gateway
//!
//! A gateway service that provides authentication, WebSocket communication,
//! and HTTP API endpoints for interacting with the forgecode service.
//!
//! # Features
//! - Token-based authentication with permissions
//! - WebSocket support for real-time code execution
//! - HTTP API endpoints for code execution and token management
//! - Integration with forgecode service for actual code execution
//! - Comprehensive error handling and logging

pub mod auth;
pub mod forgecode_client;
pub mod handlers;
pub mod server;

#[cfg(test)]
mod integration_tests;

/// Re-export commonly used types for convenience
pub use auth::{TokenManager, AuthError};
pub use forgecode_client::{ForgeCodeClient, ExecuteCodeRequest, ExecuteCodeResponse};
pub use handlers::{ExecuteRequest, ExecuteResponse, AuthRequest, AuthResponse};
pub use server::{create_routes, handle_websocket_connection};

/// Gateway configuration
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// Port to run the gateway server on
    pub port: u16,
    /// Base URL of the forgecode service
    pub forgecode_base_url: String,
    /// Token expiration time in seconds
    pub token_expiration_seconds: u64,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            forgecode_base_url: "http://localhost:8081".to_string(),
            token_expiration_seconds: 3600, // 1 hour
        }
    }
}

/// Start the gateway server with the provided configuration
pub async fn start_gateway(config: GatewayConfig) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("Starting forge_gateway on port {}", config.port);
    println!("Connecting to forgecode service at: {}", config.forgecode_base_url);

    // Initialize components
    let token_manager = TokenManager::new("http://localhost:8081".to_string());
    let forgecode_client = ForgeCodeClient::new(config.forgecode_base_url);

    // Check forgecode service health
    match forgecode_client.health_check().await {
        Ok(true) => println!("Forgecode service is healthy"),
        Ok(false) => println!("Warning: Forgecode service is not responding"),
        Err(e) => println!("Warning: Cannot connect to forgecode service: {}", e),
    }

    // Create routes
    let routes = create_routes(token_manager, forgecode_client);

    // Start server
    warp::serve(routes)
        .run(([0, 0, 0, 0], config.port))
        .await;

    Ok(())
}