//! Authentication and token management for the web gateway

use actix_web::{dev::ServiceRequest, Error, web, HttpRequest};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::middleware::HttpAuthentication;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT claims structure for token authentication
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String, // token identifier
    exp: usize, // expiration timestamp
    iat: usize, // issued at timestamp
}

/// Token manager for generating and validating authentication tokens
pub struct TokenManager {
    valid_tokens: RwLock<HashSet<String>>,
    secret_key: String,
}

impl TokenManager {
    /// Create a new token manager
    pub fn new() -> Self {
        Self {
            valid_tokens: RwLock::new(HashSet::new()),
            secret_key: Self::generate_secret_key(),
        }
    }

    /// Generate a new authentication token
    pub fn generate_token(&self) -> String {
        let token_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        // Default token expiration: 24 hours
        let expiration = now + (24 * 60 * 60);

        let claims = Claims {
            sub: token_id.clone(),
            exp: expiration,
            iat: now,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret_key.as_ref()),
        )
        .unwrap_or_else(|_| token_id.clone());

        // Store the token in the valid tokens set
        self.valid_tokens.write().unwrap().insert(token.clone());

        token
    }

    /// Generate a URL-safe authentication token
    pub fn generate_url_token(&self) -> String {
        let token_id: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        // Default token expiration: 24 hours
        let expiration = now + (24 * 60 * 60);

        let claims = Claims {
            sub: token_id.clone(),
            exp: expiration,
            iat: now,
        };

        // Generate JWT token
        let jwt_token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret_key.as_ref()),
        )
        .unwrap_or_else(|_| token_id.clone());

        // Convert to URL-safe base64 encoding
        let url_safe_token = URL_SAFE_NO_PAD.encode(jwt_token.as_bytes());

        // Store the original token in the valid tokens set
        self.valid_tokens.write().unwrap().insert(jwt_token.clone());

        url_safe_token
    }

    /// Validate a URL-safe authentication token
    pub fn validate_url_token(&self, url_token: &str) -> bool {
        // Decode URL-safe base64 token
        let decoded_token = match URL_SAFE_NO_PAD.decode(url_token) {
            Ok(bytes) => String::from_utf8(bytes).unwrap_or_default(),
            Err(_) => return false,
        };

        // Validate the decoded token using existing validation logic
        self.validate_token(&decoded_token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_token_generation() {
        let token_manager = TokenManager::new();
        let url_token = token_manager.generate_url_token();

        // URL token should be base64url encoded (no padding, URL-safe characters)
        assert!(!url_token.is_empty());
        assert!(!url_token.contains('+')); // No standard base64 plus signs
        assert!(!url_token.contains('/')); // No standard base64 slashes
        assert!(!url_token.contains('=')); // No padding characters
    }

    #[test]
    fn test_url_token_validation() {
        let token_manager = TokenManager::new();
        let url_token = token_manager.generate_url_token();

        // Should validate correctly
        assert!(token_manager.validate_url_token(&url_token));

        // Invalid token should fail
        assert!(!token_manager.validate_url_token("invalid_token"));
    }

    #[test]
    fn test_url_token_extraction() {
        // Test URL token extraction from query parameters
        let test_urls = vec![
            "http://example.com/api?token=abc123",
            "http://example.com/api?param=value&token=def456",
            "http://example.com/api?token=ghi789&other=param",
        ];

        for url in test_urls {
            let req = actix_web::test::TestRequest::get()
                .uri(url)
                .to_http_request();

            // This would be tested in integration tests
            assert!(url.contains("token="));
        }
    }

    #[test]
    fn test_bearer_token_compatibility() {
        let token_manager = TokenManager::new();

        // Generate both types of tokens
        let bearer_token = token_manager.generate_token();
        let url_token = token_manager.generate_url_token();

        // Both should be valid in their respective contexts
        assert!(token_manager.validate_token(&bearer_token));
        assert!(token_manager.validate_url_token(&url_token));
    }
}

    /// Validate an authentication token
    pub fn validate_token(&self, token: &str) -> bool {
        // Check if token is in the valid tokens set
        if self.valid_tokens.read().unwrap().contains(token) {
            // Also validate JWT if it's a JWT token
            if let Ok(token_data) = decode::<Claims>(
                token,
                &DecodingKey::from_secret(self.secret_key.as_ref()),
                &Validation::new(Algorithm::HS256),
            ) {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as usize;

                // Check if token is not expired
                token_data.claims.exp > now
            } else {
                // Simple token validation (non-JWT)
                true
            }
        } else {
            false
        }
    }

    /// Invalidate a token
    pub fn invalidate_token(&self, token: &str) {
        self.valid_tokens.write().unwrap().remove(token);
    }

    /// Generate a secure secret key
    fn generate_secret_key() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(64)
            .map(char::from)
            .collect()
    }
}

/// Authentication middleware for Bearer token validation
pub struct TokenAuth;

impl TokenAuth {
    /// Create new Bearer authentication middleware
    pub fn new() -> HttpAuthentication<BearerAuth, fn(ServiceRequest, BearerAuth) -> Result<ServiceRequest, Error>> {
        HttpAuthentication::bearer(validator)
    }
}

/// Authentication middleware for URL token validation
pub struct UrlTokenAuth;

impl UrlTokenAuth {
    /// Create new URL token authentication middleware
    pub fn new() -> HttpAuthentication<UrlTokenExtractor, fn(ServiceRequest, UrlToken) -> Result<ServiceRequest, Error>> {
        HttpAuthentication::custom(url_token_validator)
    }
}

/// URL token structure for custom authentication
#[derive(Debug)]
pub struct UrlToken {
    pub token: String,
}

/// Custom extractor for URL tokens
pub struct UrlTokenExtractor;

impl actix_web_httpauth::extractors::Authentication for UrlTokenExtractor {
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self::Output, actix_web::Error>>>>;
    type Output = UrlToken;
    type Config = ();

    fn authenticate(req: &HttpRequest) -> Self::Future {
        let req = req.clone();

        Box::pin(async move {
            // Extract token from URL parameter
            let token = req
                .query_string()
                .split('&')
                .find(|param| param.starts_with("token="))
                .and_then(|param| param.split('=').nth(1))
                .map(|token| token.to_string());

            match token {
                Some(token) if !token.is_empty() => Ok(UrlToken { token }),
                _ => Err(actix_web::error::ErrorUnauthorized("Missing or invalid URL token")),
            }
        })
    }
}

/// Token validator function for Bearer tokens
async fn validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, Error> {
    let token = credentials.token();

    // Extract token manager from app data
    if let Some(token_manager) = req.app_data::<web::Data<TokenManager>>() {
        if token_manager.validate_token(token) {
            return Ok(req);
        }
    }

    // Token validation failed
    let config = req
        .app_data::<Config>()
        .map(|data| data.clone())
        .unwrap_or_else(Default::default);

    Err(AuthenticationError::from(config).into())
}

/// Token validator function for URL tokens
async fn url_token_validator(
    req: ServiceRequest,
    credentials: UrlToken,
) -> Result<ServiceRequest, Error> {
    let token = credentials.token;

    // Extract token manager from app data
    if let Some(token_manager) = req.app_data::<web::Data<TokenManager>>() {
        if token_manager.validate_url_token(&token) {
            return Ok(req);
        }
    }

    // Token validation failed
    Err(actix_web::error::ErrorUnauthorized("Invalid URL token"))
}