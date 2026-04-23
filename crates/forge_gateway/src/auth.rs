//! Authentication and token management for the web gateway

use actix_web::{dev::ServiceRequest, Error, web};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use actix_web_httpauth::middleware::HttpAuthentication;
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

/// Authentication middleware for token validation
pub struct TokenAuth;

impl TokenAuth {
    /// Create new authentication middleware
    pub fn new() -> HttpAuthentication<BearerAuth, fn(ServiceRequest, BearerAuth) -> Result<ServiceRequest, Error>> {
        HttpAuthentication::bearer(validator)
    }
}

/// Token validator function
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