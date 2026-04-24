//! Gateway configuration management

use serde::{Deserialize, Serialize};

/// Gateway configuration settings
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
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            token_expiration_seconds: 3600, // 1 hour
            allowed_origins: vec!["*".to_string()],
            static_files_dir: "static".to_string(),
        }
    }
}