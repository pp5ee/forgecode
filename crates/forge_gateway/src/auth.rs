use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use thiserror::Error;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub id: Uuid,
    pub created_at: std::time::SystemTime,
    pub expires_at: std::time::SystemTime,
    pub permissions: Vec<String>,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("Token not found")]
    TokenNotFound,
    #[error("Token expired")]
    TokenExpired,
    #[error("Invalid token format")]
    InvalidTokenFormat,
}

#[derive(Clone)]
pub struct TokenManager {
    tokens: Arc<RwLock<HashMap<Uuid, Token>>>,
}

impl TokenManager {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn generate_token(&self, permissions: Vec<String>) -> Result<(String, u64), AuthError> {
        // Simple validation - ensure no empty permissions
        if permissions.iter().any(|p| p.is_empty()) {
            return Err(AuthError::InvalidTokenFormat);
        }

        let token_id = Uuid::new_v4();
        let now = std::time::SystemTime::now();
        let expires_in = 3600; // 1 hour in seconds
        let expires_at = now + std::time::Duration::from_secs(expires_in);

        let token = Token {
            id: token_id,
            created_at: now,
            expires_at,
            permissions,
        };

        {
            let mut tokens = self.tokens.write().unwrap();
            tokens.insert(token_id, token);
        }

        Ok((token_id.to_string(), expires_in))
    }

    pub fn validate_token(&self, token_str: &str) -> Result<Token, AuthError> {
        let token_id = Uuid::parse_str(token_str)
            .map_err(|_| AuthError::InvalidTokenFormat)?;

        let tokens = self.tokens.read().unwrap();
        let token = tokens.get(&token_id)
            .ok_or(AuthError::TokenNotFound)?
            .clone();

        if std::time::SystemTime::now() > token.expires_at {
            return Err(AuthError::TokenExpired);
        }

        Ok(token)
    }

    pub fn cleanup_expired_tokens(&self) {
        let now = std::time::SystemTime::now();
        let mut tokens = self.tokens.write().unwrap();
        tokens.retain(|_, token| now < token.expires_at);
    }
}