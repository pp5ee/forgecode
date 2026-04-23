//! Infrastructure abstractions for URL token management
//!
//! This module defines repository and generator traits for URL token
//! operations. Concrete implementations should be provided by the
//! infrastructure layer.

use async_trait::async_trait;
use forge_domain::{UrlToken, UrlTokenId, TokenValidationResult};
use std::collections::HashMap;

/// Repository trait for URL token persistence
///
/// Implementations should handle storage and retrieval of URL tokens
/// from a persistent store (e.g., file system, database).
#[async_trait]
pub trait UrlTokenRepository: Send + Sync {
    /// Save a URL token to the repository
    ///
    /// # Arguments
    /// * `token` - The token to save
    ///
    /// # Errors
    /// Returns an error if the save operation fails
    async fn save(&self, token: &UrlToken) -> anyhow::Result<()>;

    /// Find a token by its ID
    ///
    /// # Arguments
    /// * `id` - The token ID to search for
    ///
    /// # Returns
    /// * `Ok(Some(token))` if found
    /// * `Ok(None)` if not found
    /// * `Err(_)` if the lookup fails
    async fn find_by_id(&self, id: &UrlTokenId) -> anyhow::Result<Option<UrlToken>>;

    /// Find a token by its token string
    ///
    /// # Arguments
    /// * `token_str` - The token string to search for
    ///
    /// # Returns
    /// * `Ok(Some(token))` if found
    /// * `Ok(None)` if not found
    /// * `Err(_)` if the lookup fails
    async fn find_by_token(&self, token_str: &str) -> anyhow::Result<Option<UrlToken>>;

    /// List all tokens (including expired/revoked)
    ///
    /// # Returns
    /// A map of token IDs to tokens
    async fn list_all(&self) -> anyhow::Result<HashMap<UrlTokenId, UrlToken>>;

    /// Delete a token from the repository
    ///
    /// # Arguments
    /// * `id` - The token ID to delete
    ///
    /// # Errors
    /// Returns an error if the delete operation fails
    async fn delete(&self, id: &UrlTokenId) -> anyhow::Result<()>;

    /// Update an existing token (e.g., to revoke it)
    ///
    /// # Arguments
    /// * `token` - The token to update
    ///
    /// # Errors
    /// Returns an error if the update operation fails
    async fn update(&self, token: &UrlToken) -> anyhow::Result<()>;
}

/// Generator trait for creating cryptographically secure tokens
///
/// This trait abstracts the token generation logic, allowing for
/// different implementations (e.g., random strings, JWT-based, etc.)
pub trait UrlTokenGenerator: Send + Sync {
    /// Generate a new token string
    ///
    /// # Returns
    /// A cryptographically secure random token string
    fn generate_token_string(&self) -> String;

    /// Generate a token ID
    ///
    /// # Returns
    /// A unique token identifier
    fn generate_token_id(&self) -> UrlTokenId;
}

/// Standard implementation using UUID and random string generation
#[derive(Default)]
pub struct StandardTokenGenerator;

impl UrlTokenGenerator for StandardTokenGenerator {
    fn generate_token_string(&self) -> String {
        use rand::distributions::Alphanumeric;
        use rand::{thread_rng, Rng};

        thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }

    fn generate_token_id(&self) -> UrlTokenId {
        UrlTokenId::generate()
    }
}

/// Token encoder/decoder for JWT-style tokens
///
/// This trait abstracts the encoding and decoding of tokens,
/// allowing for different implementations (JWT, simple hashes, etc.)
pub trait TokenCodec: Send + Sync {
    /// Encode a token ID into a token string
    ///
    /// # Arguments
    /// * `token_id` - The token ID to encode
    /// * `secret` - The secret key for signing
    ///
    /// # Returns
    /// The encoded token string
    fn encode(&self, token_id: &UrlTokenId, secret: &str) -> anyhow::Result<String>;

    /// Decode and validate a token string
    ///
    /// # Arguments
    /// * `token_str` - The token string to decode
    /// * `secret` - The secret key for validation
    ///
    /// # Returns
    /// The validation result containing the token ID if valid
    fn decode(&self, token_str: &str, secret: &str) -> TokenValidationResult;
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_standard_token_generator_generate_token_string() {
        let fixture = StandardTokenGenerator::default();
        
        let actual = fixture.generate_token_string();
        
        assert_eq!(actual.len(), 32);
    }

    #[test]
    fn test_standard_token_generator_generate_token_id() {
        let fixture = StandardTokenGenerator::default();
        
        let actual = fixture.generate_token_id();
        
        assert!(!actual.to_string().is_empty());
    }
}
