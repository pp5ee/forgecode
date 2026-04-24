//! Forgecode Gateway - Web UI gateway for forgecode with URL token authentication
//!
//! This crate provides a web interface to access forgecode functionality through
//! a browser with URL-based token authentication, similar to OpenClaw Gateway.

pub mod auth;
pub mod server;
pub mod handlers;
pub mod websocket;
pub mod config;
pub mod integration;

pub use config::GatewayConfig;
pub use server::run_server;
pub use integration::GatewayIntegration;

/// Gateway-specific errors
#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("HTTP server error: {0}")]
    Http(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Forgecode integration error: {0}")]
    Integration(String),
}

pub type Result<T> = std::result::Result<T, GatewayError>;