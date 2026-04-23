//! URL Token Service
//!
//! This service provides business logic for URL token authentication
//! in the web gateway. It handles token creation, validation, revocation,
//! and lifecycle management.

use std::sync::Arc;

use chrono::Duration;
use forge_domain::{
    CreateUrlTokenRequest, CreateUrlTokenResponse, TokenValidationResult, UrlToken,
    UrlTokenId,
};
use forge_infra::UrlTokenRepository;

/// Service for managing URL tokens used for web gateway authentication
///
/// This service provides high-level operations for token lifecycle
/// management while delegating persistence to the repository layer.
#[derive(Clone)]
pub struct UrlTokenService<R> {
    repository: Arc<R>,
}

impl<R: UrlTokenRepository> UrlTokenService<R> {
    /// Create a new URL token service
    ///
    /// # Arguments
    /// * `repository` - The repository for token persistence
    ///
    /// # Returns
    /// A new URL token service instance
    pub fn new(repository: Arc<R>) -> Self {
        Self { repository }
    }

    /// Create a new URL token
    ///
    /// # Arguments
    /// * `request` - The request containing token parameters
    ///
    /// # Returns
    /// A response containing the created token details
    ///
    /// # Errors
    /// Returns an error if the repository save fails
    pub async fn create_token(
        &self,
        request: CreateUrlTokenRequest,
    ) -> anyhow::Result<CreateUrlTokenResponse> {
        let duration = Duration::hours(request.duration_hours);
        let token = UrlToken::new(duration, request.description);

        self.repository.save(&token).await?;

        Ok(CreateUrlTokenResponse {
            id: token.id.clone(),
            token: token.token.clone(),
            expires_at: token.expires_at,
        })
    }

    /// Validate a token string
    ///
    /// Checks if the token exists, is not expired, and is not revoked.
    ///
    /// # Arguments
    /// * `token_str` - The token string to validate
    ///
    /// # Returns
    /// A validation result indicating success or failure reason
    pub async fn validate_token(&self, token_str: &str) -> TokenValidationResult {
        let token = match self.repository.find_by_token(token_str).await {
            Ok(Some(t)) => t,
            Ok(None) => return TokenValidationResult::NotFound,
            Err(_) => return TokenValidationResult::InvalidFormat,
        };

        if token.revoked {
            return TokenValidationResult::Revoked;
        }

        if token.is_expired() {
            return TokenValidationResult::Expired;
        }

        TokenValidationResult::Valid(token.id)
    }

    /// Revoke a token by ID
    ///
    /// # Arguments
    /// * `token_id` - The ID of the token to revoke
    ///
    /// # Errors
    /// Returns an error if the token is not found or update fails
    pub async fn revoke_token(&self, token_id: &UrlTokenId) -> anyhow::Result<()> {
        let mut token = self
            .repository
            .find_by_id(token_id)
            .await?
            .ok_or_else(|| TokenError::TokenNotFound(token_id.clone()))?;

        token.revoke();
        self.repository.update(&token).await?;

        Ok(())
    }

    /// List all tokens
    ///
    /// Returns all tokens including expired and revoked ones.
    ///
    /// # Returns
    /// A map of token IDs to tokens
    pub async fn list_tokens(&self) -> anyhow::Result<std::collections::HashMap<UrlTokenId, UrlToken>> {
        self.repository.list_all().await
    }

    /// Delete a token
    ///
    /// # Arguments
    /// * `token_id` - The ID of the token to delete
    ///
    /// # Errors
    /// Returns an error if the delete operation fails
    pub async fn delete_token(&self, token_id: &UrlTokenId) -> anyhow::Result<()> {
        self.repository.delete(token_id).await
    }

    /// Clean up expired tokens
    ///
    /// Removes all tokens that have expired from the repository.
    ///
    /// # Returns
    /// The number of tokens removed
    pub async fn cleanup_expired_tokens(&self) -> anyhow::Result<usize> {
        let tokens = self.repository.list_all().await?;
        let mut removed = 0;

        for (id, token) in tokens {
            if token.is_expired() {
                self.repository.delete(&id).await?;
                removed += 1;
            }
        }

        Ok(removed)
    }
}

/// Error type for URL token operations
#[derive(thiserror::Error, Debug)]
pub enum TokenError {
    /// Token not found
    #[error("Token not found: {0}")]
    TokenNotFound(UrlTokenId),
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use forge_infra::UrlTokenRepository;
    use pretty_assertions::assert_eq;

    use super::*;

    /// Mock repository for testing
    struct MockTokenRepository {
        tokens: Mutex<HashMap<UrlTokenId, UrlToken>>,
    }

    impl MockTokenRepository {
        fn new() -> Self {
            Self {
                tokens: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl UrlTokenRepository for MockTokenRepository {
        async fn save(&self, token: &UrlToken) -> anyhow::Result<()> {
            self.tokens.lock().unwrap().insert(token.id.clone(), token.clone());
            Ok(())
        }

        async fn find_by_id(&self, id: &UrlTokenId) -> anyhow::Result<Option<UrlToken>> {
            Ok(self.tokens.lock().unwrap().get(id).cloned())
        }

        async fn find_by_token(&self, token_str: &str) -> anyhow::Result<Option<UrlToken>> {
            Ok(self
                .tokens
                .lock()
                .unwrap()
                .values()
                .find(|t| t.token == token_str)
                .cloned())
        }

        async fn list_all(&self) -> anyhow::Result<HashMap<UrlTokenId, UrlToken>> {
            Ok(self.tokens.lock().unwrap().clone())
        }

        async fn delete(&self, id: &UrlTokenId) -> anyhow::Result<()> {
            self.tokens.lock().unwrap().remove(id);
            Ok(())
        }

        async fn update(&self, token: &UrlToken) -> anyhow::Result<()> {
            self.tokens.lock().unwrap().insert(token.id.clone(), token.clone());
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_create_token() {
        let fixture = MockTokenRepository::new();
        let service = UrlTokenService::new(Arc::new(fixture));

        let request = CreateUrlTokenRequest::default()
            .duration_hours(24)
            .description("Test token");

        let actual = service.create_token(request).await.unwrap();

        assert!(!actual.token.is_empty());
        assert!(!actual.id.to_string().is_empty());
    }

    #[tokio::test]
    async fn test_validate_token_valid() {
        let fixture = MockTokenRepository::new();
        let service = UrlTokenService::new(Arc::new(fixture));

        let request = CreateUrlTokenRequest::default();
        let created = service.create_token(request).await.unwrap();

        let actual = service.validate_token(&created.token).await;

        assert!(matches!(actual, TokenValidationResult::Valid(_)));
    }

    #[tokio::test]
    async fn test_validate_token_not_found() {
        let fixture = MockTokenRepository::new();
        let service = UrlTokenService::new(Arc::new(fixture));

        let actual = service.validate_token("invalid_token").await;

        assert_eq!(actual, TokenValidationResult::NotFound);
    }

    #[tokio::test]
    async fn test_revoke_token() {
        let fixture = MockTokenRepository::new();
        let service = UrlTokenService::new(Arc::new(fixture));

        let request = CreateUrlTokenRequest::default();
        let created = service.create_token(request).await.unwrap();

        service.revoke_token(&created.id).await.unwrap();

        let actual = service.validate_token(&created.token).await;
        assert_eq!(actual, TokenValidationResult::Revoked);
    }

    #[tokio::test]
    async fn test_list_tokens() {
        let fixture = MockTokenRepository::new();
        let service = UrlTokenService::new(Arc::new(fixture));

        let request = CreateUrlTokenRequest::default().description("Token 1");
        let _ = service.create_token(request).await.unwrap();

        let actual = service.list_tokens().await.unwrap();

        assert_eq!(actual.len(), 1);
    }

    #[tokio::test]
    async fn test_delete_token() {
        let fixture = MockTokenRepository::new();
        let service = UrlTokenService::new(Arc::new(fixture));

        let request = CreateUrlTokenRequest::default();
        let created = service.create_token(request).await.unwrap();

        service.delete_token(&created.id).await.unwrap();

        let actual = service.validate_token(&created.token).await;
        assert_eq!(actual, TokenValidationResult::NotFound);
    }

    #[tokio::test]
    async fn test_cleanup_expired_tokens() {
        let fixture = MockTokenRepository::new();
        let service = UrlTokenService::new(Arc::new(fixture));

        // Create an expired token directly
        let mut token = UrlToken::new(Duration::hours(-1), Some("Expired".to_string()));
        token.expires_at = chrono::Utc::now() - Duration::hours(1);
        Arc::clone(&service.repository)
            .save(&token)
            .await
            .unwrap();

        let actual = service.cleanup_expired_tokens().await.unwrap();

        assert_eq!(actual, 1);
    }
}
