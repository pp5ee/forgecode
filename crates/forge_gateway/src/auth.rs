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
    pub user_id: Option<String>,
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
    #[error("Insufficient permissions: {0}")]
    InsufficientPermissions(String),
    #[error("ForgeCode service unavailable")]
    ServiceUnavailable,
}

#[derive(Clone)]
pub struct TokenManager {
    tokens: Arc<RwLock<HashMap<Uuid, Token>>>,
    forgecode_url: String,
}

impl TokenManager {
    pub fn new(forgecode_url: String) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            forgecode_url,
        }
    }

    pub async fn generate_token(&self, user_id: Option<String>, permissions: Vec<String>) -> Result<Uuid, AuthError> {
        // Validate permissions with forgecode service
        if let Err(_) = self.validate_permissions(&permissions).await {
            return Err(AuthError::ServiceUnavailable);
        }

        let token_id = Uuid::new_v4();
        let now = std::time::SystemTime::now();
        let expires_at = now + std::time::Duration::from_secs(3600); // 1 hour

        let token = Token {
            id: token_id,
            created_at: now,
            expires_at,
            user_id,
            permissions,
        };

        {
            let mut tokens = self.tokens.write().unwrap();
            tokens.insert(token_id, token);
        }

        Ok(token_id)
    }

    pub async fn validate_token(&self, token_id: Uuid, required_permission: Option<&str>) -> Result<Token, AuthError> {
        let tokens = self.tokens.read().unwrap();

        let token = tokens.get(&token_id)
            .ok_or(AuthError::TokenNotFound)?
            .clone();

        if std::time::SystemTime::now() > token.expires_at {
            return Err(AuthError::TokenExpired);
        }

        // Check permissions if required
        if let Some(permission) = required_permission {
            if !token.permissions.contains(&permission.to_string()) {
                return Err(AuthError::InsufficientPermissions(permission.to_string()));
            }
        }

        Ok(token)
    }

    pub async fn renew_token(&self, token_id: Uuid) -> Result<Uuid, AuthError> {
        let mut tokens = self.tokens.write().unwrap();

        let token = tokens.get_mut(&token_id)
            .ok_or(AuthError::TokenNotFound)?;

        let now = std::time::SystemTime::now();
        if now > token.expires_at {
            return Err(AuthError::TokenExpired);
        }

        token.expires_at = now + std::time::Duration::from_secs(3600);
        Ok(token_id)
    }

    pub async fn revoke_token(&self, token_id: Uuid) -> Result<(), AuthError> {
        let mut tokens = self.tokens.write().unwrap();
        tokens.remove(&token_id)
            .map(|_| ())
            .ok_or(AuthError::TokenNotFound)
    }

    async fn validate_permissions(&self, permissions: &[String]) -> Result<(), AuthError> {
        // In a real implementation, this would validate permissions with the forgecode service
        // For now, we'll simulate this with a simple check
        if permissions.iter().any(|p| p.is_empty()) {
            return Err(AuthError::InvalidTokenFormat);
        }
        Ok(())
    }

    pub fn cleanup_expired_tokens(&self) {
        let now = std::time::SystemTime::now();
        let mut tokens = self.tokens.write().unwrap();
        tokens.retain(|_, token| now < token.expires_at);
    }
}