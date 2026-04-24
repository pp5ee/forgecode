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
pub mod middleware;
pub mod websocket;

#[cfg(test)]
mod integration_tests;

/// Re-export commonly used types for convenience
pub use auth::{TokenManager, AuthError};
pub use forgecode_client::{ForgeCodeClient, ExecuteCodeRequest, ExecuteCodeResponse};
pub use handlers::{ExecuteRequest, ExecuteResponse, AuthRequest, AuthResponse};

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