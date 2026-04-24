//! Gateway configuration management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Gateway-specific configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// HTTP server bind address
    pub bind_address: String,

    /// HTTP server port
    pub port: u16,

    /// Token expiration time in seconds
    pub token_expiration_seconds: u64,

    /// Allowed CORS origins
    pub allowed_origins: Vec<String>,

    /// Static files directory
    pub static_files_dir: String,

    /// Enable/disable gateway functionality
    pub enabled: bool,

    /// Maximum concurrent connections
    pub max_connections: usize,

    /// Request timeout in seconds
    pub request_timeout_seconds: u64,

    /// WebSocket ping interval in seconds
    pub websocket_ping_interval: u64,

    /// WebSocket timeout in seconds
    pub websocket_timeout: u64,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            token_expiration_seconds: 3600, // 1 hour
            allowed_origins: vec!["*".to_string()],
            static_files_dir: "static".to_string(),
            enabled: true,
            max_connections: 100,
            request_timeout_seconds: 30,
            websocket_ping_interval: 30,
            websocket_timeout: 60,
        }
    }
}

/// Extension methods for forge_config integration
impl GatewayConfig {
    /// Create gateway config from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(bind_addr) = std::env::var("FORGECODE_GATEWAY_BIND_ADDRESS") {
            config.bind_address = bind_addr;
        }

        if let Ok(port) = std::env::var("FORGECODE_GATEWAY_PORT") {
            if let Ok(port_num) = port.parse() {
                config.port = port_num;
            }
        }

        if let Ok(enabled) = std::env::var("FORGECODE_GATEWAY_ENABLED") {
            config.enabled = enabled.to_lowercase() == "true";
        }

        config
    }

    /// Validate configuration settings
    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("Port cannot be 0".to_string());
        }

        if self.token_expiration_seconds == 0 {
            return Err("Token expiration must be greater than 0".to_string());
        }

        if self.max_connections == 0 {
            return Err("Max connections must be greater than 0".to_string());
        }

        Ok(())
    }
}