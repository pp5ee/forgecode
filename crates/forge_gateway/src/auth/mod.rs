//! Secure Token Authentication System
//! 
//! Provides comprehensive token-based authentication with security features:
//! - URL-based token authentication with security validation
//! - Token renewal with refresh window and limits
//! - Rate limiting and brute force protection
//! - Token revocation and cleanup mechanisms
//! - Security headers and CORS configuration
//! - Audit logging and security monitoring

pub mod security;
pub mod middleware;
pub mod handlers;

// Re-export commonly used types
pub use security::{SecureTokenManager, SecureToken, SecurityConfig, AuthError};
pub use middleware::{secure_url_token_auth, security_headers, cors_config};
pub use handlers::{
    generate_token, refresh_token, revoke_token, get_token_info, cleanup_tokens, get_security_config,
    TokenRequest, TokenResponse, RefreshTokenRequest, RevokeTokenRequest, TokenInfoResponse
};

/// Initialize the secure authentication system
pub fn init_auth_system(config: SecurityConfig) -> SecureTokenManager {
    SecureTokenManager::new(config)
}

/// Default security configuration for development
pub fn default_security_config() -> SecurityConfig {
    SecurityConfig {
        token_expiration_seconds: 3600, // 1 hour
        max_tokens_per_user: 10,
        rate_limit_per_minute: 60,
        refresh_window_percent: 20,
        require_https: false, // Allow HTTP for development
        enable_audit_logging: true,
    }
}

/// Production security configuration
pub fn production_security_config() -> SecurityConfig {
    SecurityConfig {
        token_expiration_seconds: 900, // 15 minutes for production
        max_tokens_per_user: 5,
        rate_limit_per_minute: 30,
        refresh_window_percent: 10,
        require_https: true,
        enable_audit_logging: true,
    }
}
