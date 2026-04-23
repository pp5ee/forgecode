//! URL Token authentication domain types for gateway access
//!
//! This module provides types for time-limited, URL-based authentication tokens
//! that can be passed as query parameters for web gateway access.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a URL token
#[derive(
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    derive_more::From,
    derive_more::Deref,
    derive_more::Display,
)]
#[serde(transparent)]
pub struct UrlTokenId(String);

impl UrlTokenId {
    /// Generate a new random token ID
    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

/// A time-limited URL token for gateway authentication
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UrlToken {
    /// Unique identifier for this token
    pub id: UrlTokenId,
    /// The actual token string used in URLs
    pub token: String,
    /// When the token was created
    pub created_at: DateTime<Utc>,
    /// When the token expires
    pub expires_at: DateTime<Utc>,
    /// Optional description or label for this token
    pub description: Option<String>,
    /// Whether the token has been revoked
    pub revoked: bool,
}

impl UrlToken {
    /// Create a new URL token with specified duration
    ///
    /// # Arguments
    /// * `duration` - How long the token should be valid
    /// * `description` - Optional description for the token
    ///
    /// # Returns
    /// A new URL token with a generated ID and random token string
    pub fn new(duration: Duration, description: Option<String>) -> Self {
        let id = UrlTokenId::generate();
        let token = Self::generate_token_string();
        let created_at = Utc::now();
        let expires_at = created_at + duration;

        Self {
            id,
            token,
            created_at,
            expires_at,
            description,
            revoked: false,
        }
    }

    /// Create a URL token with default 24-hour expiration
    pub fn default_expiration() -> Self {
        Self::new(Duration::hours(24), None)
    }

    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if the token is valid (not expired and not revoked)
    pub fn is_valid(&self) -> bool {
        !self.revoked && !self.is_expired()
    }

    /// Revoke the token
    pub fn revoke(&mut self) {
        self.revoked = true;
    }

    /// Generate a secure random token string
    fn generate_token_string() -> String {
        use rand::distributions::Alphanumeric;
        use rand::{thread_rng, Rng};

        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }
}

impl fmt::Display for UrlToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "UrlToken(id={}, expires_at={})",
            self.id, self.expires_at
        )
    }
}

/// JWT claims for URL token authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlTokenClaims {
    /// Subject (token ID)
    pub sub: String,
    /// Expiration timestamp
    pub exp: usize,
    /// Issued at timestamp
    pub iat: usize,
    /// Token type
    pub typ: String,
}

impl UrlTokenClaims {
    /// Create new claims for a URL token
    ///
    /// # Arguments
    /// * `token_id` - The token ID to encode
    /// * `expires_at` - When the token expires
    ///
    /// # Returns
    /// New JWT claims
    pub fn new(token_id: &UrlTokenId, expires_at: DateTime<Utc>) -> Self {
        let iat = Utc::now().timestamp() as usize;
        let exp = expires_at.timestamp() as usize;

        Self {
            sub: token_id.to_string(),
            exp,
            iat,
            typ: "url_token".to_string(),
        }
    }
}

/// Request to create a new URL token
#[derive(Debug, Clone, Serialize, Deserialize, derive_setters::Setters)]
#[setters(into, strip_option)]
pub struct CreateUrlTokenRequest {
    /// Duration in hours for token validity
    pub duration_hours: i64,
    /// Optional description
    pub description: Option<String>,
}

impl Default for CreateUrlTokenRequest {
    fn default() -> Self {
        Self {
            duration_hours: 24,
            description: None,
        }
    }
}

/// Response after creating a URL token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUrlTokenResponse {
    /// The token ID
    pub id: UrlTokenId,
    /// The full token string (shown only once)
    pub token: String,
    /// When the token expires
    pub expires_at: DateTime<Utc>,
}

/// Token validation result
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenValidationResult {
    /// Token is valid
    Valid(UrlTokenId),
    /// Token has expired
    Expired,
    /// Token has been revoked
    Revoked,
    /// Token not found
    NotFound,
    /// Invalid token format
    InvalidFormat,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_url_token_new() {
        let fixture = UrlToken::new(Duration::hours(1), Some("Test token".to_string()));
        
        assert!(!fixture.token.is_empty());
        assert_eq!(fixture.description, Some("Test token".to_string()));
        assert!(!fixture.revoked);
    }

    #[test]
    fn test_url_token_is_expired() {
        let mut fixture = UrlToken::default_expiration();
        fixture.expires_at = Utc::now() - Duration::hours(1);
        
        assert!(fixture.is_expired());
    }

    #[test]
    fn test_url_token_is_valid() {
        let fixture = UrlToken::default_expiration();
        
        assert!(fixture.is_valid());
    }

    #[test]
    fn test_url_token_revoked_is_not_valid() {
        let mut fixture = UrlToken::default_expiration();
        fixture.revoke();
        
        assert!(!fixture.is_valid());
    }

    #[test]
    fn test_url_token_id_generate() {
        let actual = UrlTokenId::generate();
        
        assert!(!actual.to_string().is_empty());
    }

    #[test]
    fn test_url_token_claims_new() {
        let token_id = UrlTokenId::generate();
        let expires_at = Utc::now() + Duration::hours(1);
        
        let actual = UrlTokenClaims::new(&token_id, expires_at);
        
        assert_eq!(actual.sub, token_id.to_string());
        assert_eq!(actual.typ, "url_token");
    }

    #[test]
    fn test_create_url_token_request_default() {
        let fixture = CreateUrlTokenRequest::default();
        
        assert_eq!(fixture.duration_hours, 24);
        assert_eq!(fixture.description, None);
    }

    #[test]
    fn test_create_url_token_request_setters() {
        let actual = CreateUrlTokenRequest::default()
            .duration_hours(48)
            .description("API access");
        
        let expected = CreateUrlTokenRequest {
            duration_hours: 48,
            description: Some("API access".to_string()),
        };
        assert_eq!(actual, expected);
    }
}
