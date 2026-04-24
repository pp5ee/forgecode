use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, Duration};
use uuid::Uuid;
use thiserror::Error;
use serde::{Deserialize, Serialize};

/// Security configuration for token authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Token expiration time in seconds
    pub token_expiration_seconds: u64,
    /// Maximum number of tokens per user/IP
    pub max_tokens_per_user: usize,
    /// Rate limiting: max requests per minute
    pub rate_limit_per_minute: u32,
    /// Token refresh window (percentage of expiration time)
    pub refresh_window_percent: u8,
    /// Require HTTPS for token transmission
    pub require_https: bool,
    /// Enable audit logging
    pub enable_audit_logging: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            token_expiration_seconds: 3600, // 1 hour
            max_tokens_per_user: 10,
            rate_limit_per_minute: 60,
            refresh_window_percent: 20,
            require_https: true,
            enable_audit_logging: true,
        }
    }
}

/// Enhanced token with security metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureToken {
    pub id: Uuid,
    pub created_at: SystemTime,
    pub expires_at: SystemTime,
    pub last_used_at: SystemTime,
    pub permissions: Vec<String>,
    pub user_agent: Option<String>,
    pub client_ip: Option<String>,
    pub is_revoked: bool,
    pub refresh_count: u32,
    pub max_refresh_count: u32,
}

impl SecureToken {
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }

    pub fn can_refresh(&self) -> bool {
        !self.is_revoked && 
        self.refresh_count < self.max_refresh_count &&
        !self.is_expired()
    }

    pub fn is_within_refresh_window(&self, config: &SecurityConfig) -> bool {
        let now = SystemTime::now();
        let total_lifetime = self.expires_at.duration_since(self.created_at)
            .unwrap_or(Duration::from_secs(config.token_expiration_seconds));
        let refresh_window = total_lifetime * config.refresh_window_percent as u32 / 100;
        
        now > self.expires_at - refresh_window
    }
}

/// Rate limiting and security tracking
#[derive(Debug, Clone)]
pub struct RateLimiter {
    requests: Arc<RwLock<HashMap<String, Vec<SystemTime>>>>,
    config: SecurityConfig,
}

impl RateLimiter {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            requests: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    pub fn check_rate_limit(&self, identifier: &str) -> Result<(), AuthError> {
        let mut requests = self.requests.write().unwrap();
        let now = SystemTime::now();
        let one_minute_ago = now - Duration::from_secs(60);

        let entry = requests.entry(identifier.to_string())
            .or_insert_with(Vec::new);

        // Remove old requests
        entry.retain(|&time| time > one_minute_ago);

        if entry.len() >= self.config.rate_limit_per_minute as usize {
            return Err(AuthError::RateLimitExceeded);
        }

        entry.push(now);
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Token not found")]
    TokenNotFound,
    #[error("Token expired")]
    TokenExpired,
    #[error("Token revoked")]
    TokenRevoked,
    #[error("Invalid token format")]
    InvalidTokenFormat,
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Refresh limit exceeded")]
    RefreshLimitExceeded,
    #[error("Invalid refresh window")]
    InvalidRefreshWindow,
    #[error("Security violation: {0}")]
    SecurityViolation(String),
}

/// Enhanced token manager with security features
#[derive(Clone)]
pub struct SecureTokenManager {
    tokens: Arc<RwLock<HashMap<Uuid, SecureToken>>>,
    rate_limiter: RateLimiter,
    config: SecurityConfig,
}

impl SecureTokenManager {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            rate_limiter: RateLimiter::new(config.clone()),
            config,
        }
    }

    pub fn generate_token(
        &self,
        permissions: Vec<String>,
        user_agent: Option<String>,
        client_ip: Option<String>,
    ) -> Result<(String, u64), AuthError> {
        // Validate permissions
        if permissions.iter().any(|p| p.trim().is_empty()) {
            return Err(AuthError::InvalidTokenFormat);
        }

        // Apply rate limiting
        let identifier = client_ip.clone().unwrap_or_else(|| "unknown".to_string());
        self.rate_limiter.check_rate_limit(&identifier)?;

        let token_id = Uuid::new_v4();
        let now = SystemTime::now();
        let expires_at = now + Duration::from_secs(self.config.token_expiration_seconds);

        let token = SecureToken {
            id: token_id,
            created_at: now,
            expires_at,
            last_used_at: now,
            permissions,
            user_agent,
            client_ip,
            is_revoked: false,
            refresh_count: 0,
            max_refresh_count: 5, // Allow up to 5 refreshes
        };

        {
            let mut tokens = self.tokens.write().unwrap();
            tokens.insert(token_id, token);
        }

        Ok((token_id.to_string(), self.config.token_expiration_seconds))
    }

    pub fn validate_token(&self, token_str: &str) -> Result<SecureToken, AuthError> {
        let token_id = Uuid::parse_str(token_str)
            .map_err(|_| AuthError::InvalidTokenFormat)?;

        let mut tokens = self.tokens.write().unwrap();
        let token = tokens.get_mut(&token_id)
            .ok_or(AuthError::TokenNotFound)?
            .clone();

        if token.is_revoked {
            return Err(AuthError::TokenRevoked);
        }

        if token.is_expired() {
            return Err(AuthError::TokenExpired);
        }

        // Update last used timestamp
        if let Some(active_token) = tokens.get_mut(&token_id) {
            active_token.last_used_at = SystemTime::now();
        }

        Ok(token)
    }

    pub fn refresh_token(&self, token_str: &str) -> Result<(String, u64), AuthError> {
        let token = self.validate_token(token_str)?;

        if !token.can_refresh() {
            return Err(AuthError::RefreshLimitExceeded);
        }

        if !token.is_within_refresh_window(&self.config) {
            return Err(AuthError::InvalidRefreshWindow);
        }

        // Generate new token with same permissions
        let (new_token, expires_in) = self.generate_token(
            token.permissions,
            token.user_agent,
            token.client_ip,
        )?;

        // Revoke old token
        self.revoke_token(token_str)?;

        Ok((new_token, expires_in))
    }

    pub fn revoke_token(&self, token_str: &str) -> Result<(), AuthError> {
        let token_id = Uuid::parse_str(token_str)
            .map_err(|_| AuthError::InvalidTokenFormat)?;

        let mut tokens = self.tokens.write().unwrap();
        if let Some(token) = tokens.get_mut(&token_id) {
            token.is_revoked = true;
            Ok(())
        } else {
            Err(AuthError::TokenNotFound)
        }
    }

    pub fn cleanup_expired_tokens(&self) -> usize {
        let now = SystemTime::now();
        let mut tokens = self.tokens.write().unwrap();
        let initial_count = tokens.len();
        
        tokens.retain(|_, token| !token.is_expired() && !token.is_revoked);
        
        initial_count - tokens.len()
    }
}
