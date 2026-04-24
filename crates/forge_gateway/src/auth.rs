//! Token authentication system for the gateway

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Authentication token with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    /// Unique token identifier
    pub id: Uuid,

    /// Token value (used in URL)
    pub token: String,

    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Expiration timestamp
    pub expires_at: chrono::DateTime<chrono::Utc>,

    /// Whether the token has been renewed
    pub renewed: bool,
}

impl AuthToken {
    /// Create a new authentication token
    pub fn new(expiration_seconds: u64) -> Self {
        let now = chrono::Utc::now();
        let expires_at = now + chrono::Duration::seconds(expiration_seconds as i64);

        Self {
            id: Uuid::new_v4(),
            token: Uuid::new_v4().to_string(),
            created_at: now,
            expires_at,
            renewed: false,
        }
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        chrono::Utc::now() > self.expires_at
    }

    /// Renew the token with a new expiration time
    pub fn renew(&mut self, expiration_seconds: u64) {
        self.expires_at = chrono::Utc::now() + chrono::Duration::seconds(expiration_seconds as i64);
        self.renewed = true;
    }
}

/// Token validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenValidation {
    pub valid: bool,
    pub message: String,
    pub token_id: Option<Uuid>,
}

/// Token manager for handling authentication tokens
pub struct TokenManager {
    tokens: std::collections::HashMap<Uuid, AuthToken>,
    expiration_seconds: u64,
}

impl TokenManager {
    /// Create a new token manager
    pub fn new(expiration_seconds: u64) -> Self {
        Self {
            tokens: std::collections::HashMap::new(),
            expiration_seconds,
        }
    }

    /// Generate a new token
    pub fn generate_token(&mut self) -> AuthToken {
        let token = AuthToken::new(self.expiration_seconds);
        self.tokens.insert(token.id, token.clone());
        token
    }

    /// Validate a token
    pub fn validate_token(&self, token_str: &str) -> TokenValidation {
        // Find token by value
        let token = self.tokens.values().find(|t| t.token == token_str);

        match token {
            Some(token) => {
                if token.is_expired() {
                    TokenValidation {
                        valid: false,
                        message: "Token has expired".to_string(),
                        token_id: Some(token.id),
                    }
                } else {
                    TokenValidation {
                        valid: true,
                        message: "Token is valid".to_string(),
                        token_id: Some(token.id),
                    }
                }
            }
            None => TokenValidation {
                valid: false,
                message: "Token not found".to_string(),
                token_id: None,
            },
        }
    }

    /// Renew a token
    pub fn renew_token(&mut self, token_str: &str) -> Option<AuthToken> {
        // Find token by value and update it
        let token_id = self.tokens
            .iter()
            .find(|(_, t)| t.token == token_str)
            .map(|(id, _)| *id);

        if let Some(token_id) = token_id {
            if let Some(token) = self.tokens.get_mut(&token_id) {
                token.renew(self.expiration_seconds);
                return Some(token.clone());
            }
        }

        None
    }
}