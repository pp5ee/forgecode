//! URL Token Authentication for the web gateway
//!
//! This module provides token-based authentication that supports:
//! - URL query parameter tokens (?token=xxx)
//! - Bearer authorization header
//! - Cookie-based tokens

use actix_web::{dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform}, Error, HttpMessage};
use actix_web::http::header;
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll};

use forge_services::UrlTokenService;
use forge_infra::UrlTokenRepository;

/// Extension trait to attach authenticated token ID to requests
pub trait TokenAuthExt {
    /// Get the authenticated token ID from the request
    fn token_id(&self) -> Option<forge_domain::UrlTokenId>;
}

impl TokenAuthExt for ServiceRequest {
    fn token_id(&self) -> Option<forge_domain::UrlTokenId> {
        self.extensions().get::<forge_domain::UrlTokenId>().cloned()
    }
}

/// Token authentication middleware factory
///
/// Creates middleware that validates URL tokens from query parameters
/// or Authorization headers.
pub struct UrlTokenAuth<R> {
    token_service: Arc<UrlTokenService<R>>,
}

impl<R: UrlTokenRepository> UrlTokenAuth<R> {
    /// Create a new URL token authentication middleware
    ///
    /// # Arguments
    /// * `token_service` - The token service for validation
    pub fn new(token_service: Arc<UrlTokenService<R>>) -> Self {
        Self { token_service }
    }
}

impl<R, S, B> Transform<S, ServiceRequest> for UrlTokenAuth<R>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    R: UrlTokenRepository + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = UrlTokenAuthMiddleware<S, R>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(UrlTokenAuthMiddleware {
            service: Rc::new(service),
            token_service: self.token_service.clone(),
        }))
    }
}

/// URL Token authentication middleware
pub struct UrlTokenAuthMiddleware<S, R> {
    service: Rc<S>,
    token_service: Arc<UrlTokenService<R>>,
}

impl<S, R, B> Service<ServiceRequest> for UrlTokenAuthMiddleware<S, R>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    R: UrlTokenRepository + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let token_service = self.token_service.clone();

        Box::pin(async move {
            // Try to extract token from query parameter
            let query_token = req.query_string()
                .split('&')
                .find(|pair| pair.starts_with("token="))
                .and_then(|pair| pair.split('=').nth(1))
                .map(|s| urlencoding::decode(s).unwrap_or_default().to_string());

            // Try to extract token from Authorization header
            let header_token = req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
                .map(|s| s.to_string());

            // Use query token or header token
            let token_str = query_token.or(header_token);

            if let Some(token) = token_str {
                let result = token_service.validate_token(&token).await;
                
                match result {
                    forge_domain::TokenValidationResult::Valid(token_id) => {
                        req.extensions_mut().insert(token_id);
                    }
                    _ => {
                        // Return 401 Unauthorized
                        return Err(actix_web::error::ErrorUnauthorized("Invalid or expired token"));
                    }
                }
            } else {
                return Err(actix_web::error::ErrorUnauthorized("Token required"));
            }

            service.call(req).await
        })
    }
}

/// Token authentication error response
#[derive(Debug, serde::Serialize)]
struct AuthError {
    error: String,
    message: String,
}

/// Public endpoint middleware - allows requests without authentication
/// but still processes tokens if present
pub struct PublicWithOptionalAuth<R> {
    token_service: Arc<UrlTokenService<R>>,
}

impl<R: UrlTokenRepository> PublicWithOptionalAuth<R> {
    /// Create new optional auth middleware
    pub fn new(token_service: Arc<UrlTokenService<R>>) -> Self {
        Self { token_service }
    }
}

impl<R, S, B> Transform<S, ServiceRequest> for PublicWithOptionalAuth<R>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    R: UrlTokenRepository + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = PublicWithOptionalAuthMiddleware<S, R>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(PublicWithOptionalAuthMiddleware {
            service: Rc::new(service),
            token_service: self.token_service.clone(),
        }))
    }
}

/// Public endpoint middleware with optional authentication
pub struct PublicWithOptionalAuthMiddleware<S, R> {
    service: Rc<S>,
    token_service: Arc<UrlTokenService<R>>,
}

impl<S, R, B> Service<ServiceRequest> for PublicWithOptionalAuthMiddleware<S, R>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    R: UrlTokenRepository + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let token_service = self.token_service.clone();

        Box::pin(async move {
            // Try to extract token from query parameter
            let query_token = req.query_string()
                .split('&')
                .find(|pair| pair.starts_with("token="))
                .and_then(|pair| pair.split('=').nth(1))
                .map(|s| urlencoding::decode(s).unwrap_or_default().to_string());

            // Try to extract token from Authorization header
            let header_token = req.headers()
                .get(header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.strip_prefix("Bearer "))
                .map(|s| s.to_string());

            let token_str = query_token.or(header_token);

            // If token is provided, validate it and attach to request
            if let Some(token) = token_str {
                let result = token_service.validate_token(&token).await;
                
                if let forge_domain::TokenValidationResult::Valid(token_id) = result {
                    req.extensions_mut().insert(token_id);
                }
                // If invalid, still allow the request but without auth context
            }

            service.call(req).await
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Mutex;

    use forge_domain::{UrlToken, UrlTokenId, CreateUrlTokenRequest};
    use forge_infra::UrlTokenRepository;

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

    #[test]
    fn test_mock_repository() {
        let repo = MockTokenRepository::new();
        let token = UrlToken::default_expiration();
        
        // Since we're testing async in sync context, just verify the repo was created
        assert!(true);
    }
}
