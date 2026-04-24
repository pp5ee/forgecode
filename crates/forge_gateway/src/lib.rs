//! Forge Gateway
//!
//! A gateway service that provides authentication, WebSocket communication,
//! and HTTP API endpoints for interacting with the forgecode service.
//!
//! # Features
//! - Secure token-based authentication with permissions
//! - URL-based token authentication with security validation
//! - Token renewal with refresh window and limits
//! - Rate limiting and brute force protection
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
pub use auth::{
    SecureTokenManager, SecureToken, SecurityConfig, AuthError,
    secure_url_token_auth, security_headers, cors_config,
    generate_token, refresh_token, revoke_token, get_token_info, cleanup_tokens, get_security_config,
    TokenRequest, TokenResponse, RefreshTokenRequest, RevokeTokenRequest, TokenInfoResponse
};

/// Gateway configuration
#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// Port to run the gateway server on
    pub port: u16,
    /// Base URL of the forgecode service
    pub forgecode_base_url: String,
    /// Security configuration for token authentication
    pub security_config: SecurityConfig,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            forgecode_base_url: "http://localhost:8081".to_string(),
            security_config: SecurityConfig::default(),
        }
    }
}
