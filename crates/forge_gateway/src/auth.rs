use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use actix_web::{dev::ServiceRequest, Error, HttpMessage};

#[derive(Debug, Clone)]
pub struct TokenManager {
    tokens: Arc<RwLock<HashMap<String, bool>>>,
}

impl TokenManager {
    pub fn new() -> Self {
        let initial_token = Uuid::new_v4().to_string();
        let mut tokens = HashMap::new();
        tokens.insert(initial_token.clone(), true);

        Self {
            tokens: Arc::new(RwLock::new(tokens)),
        }
    }

    pub fn validate_token(&self, token: &str) -> bool {
        let tokens = self.tokens.read().unwrap();
        tokens.get(token).copied().unwrap_or(false)
    }

    pub fn generate_token(&self) -> String {
        let new_token = Uuid::new_v4().to_string();
        let mut tokens = self.tokens.write().unwrap();
        tokens.insert(new_token.clone(), true);
        new_token
    }

    pub fn revoke_token(&self, token: &str) -> bool {
        let mut tokens = self.tokens.write().unwrap();
        tokens.remove(token).is_some()
    }

    pub fn get_initial_token(&self) -> Option<String> {
        let tokens = self.tokens.read().unwrap();
        tokens.keys().next().cloned()
    }
}

// URL Token Middleware
pub struct UrlTokenMiddleware {
    token_manager: Arc<TokenManager>,
}

impl UrlTokenMiddleware {
    pub fn new(token_manager: Arc<TokenManager>) -> Self {
        Self { token_manager }
    }
}

impl actix_web::dev::Transform<actix_web::dev::Service, actix_web::dev::ServiceRequest> for UrlTokenMiddleware {
    type Response = actix_web::dev::ServiceResponse;
    type Error = Error;
    type InitError = ();
    type Transform = UrlTokenMiddlewareService;
    type Future = std::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: actix_web::dev::Service) -> Self::Future {
        std::future::ready(Ok(UrlTokenMiddlewareService {
            service,
            token_manager: self.token_manager.clone(),
        }))
    }
}

pub struct UrlTokenMiddlewareService {
    service: actix_web::dev::Service,
    token_manager: Arc<TokenManager>,
}

impl actix_web::dev::Service<actix_web::dev::ServiceRequest> for UrlTokenMiddlewareService {
    type Response = actix_web::dev::ServiceResponse;
    type Error = Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: actix_web::dev::ServiceRequest) -> Self::Future {
        let token_manager = self.token_manager.clone();
        let service = self.service.clone();

        Box::pin(async move {
            // Skip authentication for certain paths
            let path = req.path();
            if path == "/auth/health" || path.starts_with("/auth/") {
                return service.call(req).await;
            }

            // Check for URL token parameter
            let query_string = req.query_string();
            let token = if let Some(token_start) = query_string.find("token=") {
                let token_value = &query_string[token_start + 6..];
                if let Some(token_end) = token_value.find('&') {
                    &token_value[..token_end]
                } else {
                    token_value
                }
            } else {
                ""
            };

            // Validate token
            if token.is_empty() || !token_manager.validate_token(token) {
                let redirect_url = format!("/auth/error?message={}", urlencoding::encode("Invalid or missing token"));
                let response = actix_web::HttpResponse::Found()
                    .append_header(("Location", redirect_url))
                    .finish();
                return Ok(req.into_response(response));
            }

            service.call(req).await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_validation() {
        let manager = TokenManager::new();

        // Test that a valid token exists
        let tokens = manager.tokens.read().unwrap();
        let valid_token = tokens.keys().next().unwrap().clone();
        drop(tokens);

        assert!(manager.validate_token(&valid_token));
        assert!(!manager.validate_token("invalid_token"));
    }

    #[test]
    fn test_token_generation() {
        let manager = TokenManager::new();

        let new_token = manager.generate_token();
        assert!(manager.validate_token(&new_token));
    }

    #[test]
    fn test_token_revocation() {
        let manager = TokenManager::new();

        let tokens = manager.tokens.read().unwrap();
        let valid_token = tokens.keys().next().unwrap().clone();
        drop(tokens);

        assert!(manager.revoke_token(&valid_token));
        assert!(!manager.validate_token(&valid_token));
    }

    #[test]
    fn test_get_initial_token() {
        let manager = TokenManager::new();
        assert!(manager.get_initial_token().is_some());
    }
}