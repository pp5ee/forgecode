use actix_web::{web, HttpResponse, Responder, HttpRequest};
use serde::{Deserialize, Serialize};
use crate::auth::security::{SecureTokenManager, SecureToken, AuthError, SecurityConfig};

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub expires_in: u64,
    pub token_type: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct RevokeTokenRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct TokenInfoResponse {
    pub token_id: String,
    pub created_at: String,
    pub expires_at: String,
    pub permissions: Vec<String>,
    pub is_expired: bool,
    pub is_revoked: bool,
}

/// Generate a new secure authentication token
#[actix_web::post("/token/generate")]
pub async fn generate_token(
    token_manager: web::Data<SecureTokenManager>,
    request: web::Json<TokenRequest>,
    req: HttpRequest,
) -> impl Responder {
    // Extract client information for security
    let user_agent = req.headers()
        .get("user-agent")
        .and_then(|ua| ua.to_str().ok())
        .map(|s| s.to_string());

    let client_ip = req.connection_info()
        .realip_remote_addr()
        .map(|s| s.to_string());

    match token_manager.generate_token(
        request.permissions.clone(),
        user_agent,
        client_ip,
    ) {
        Ok((token, expires_in)) => {
            log::info!("Generated new token with {} permissions", request.permissions.len());
            HttpResponse::Ok().json(TokenResponse {
                token,
                expires_in,
                token_type: "Bearer".to_string(),
            })
        }
        Err(e) => {
            log::warn!("Failed to generate token: {}", e);
            HttpResponse::BadRequest().body(format!("Failed to generate token: {}", e))
        }
    }
}

/// Refresh an existing token
#[actix_web::post("/token/refresh")]
pub async fn refresh_token(
    token_manager: web::Data<SecureTokenManager>,
    request: web::Json<RefreshTokenRequest>,
) -> impl Responder {
    match token_manager.refresh_token(&request.token) {
        Ok((new_token, expires_in)) => {
            log::info!("Successfully refreshed token");
            HttpResponse::Ok().json(TokenResponse {
                token: new_token,
                expires_in,
                token_type: "Bearer".to_string(),
            })
        }
        Err(AuthError::TokenExpired) => {
            HttpResponse::BadRequest().body("Token has expired and cannot be refreshed")
        }
        Err(AuthError::RefreshLimitExceeded) => {
            HttpResponse::BadRequest().body("Token refresh limit exceeded")
        }
        Err(AuthError::InvalidRefreshWindow) => {
            HttpResponse::BadRequest().body("Token cannot be refreshed yet")
        }
        Err(e) => {
            HttpResponse::BadRequest().body(format!("Failed to refresh token: {}", e))
        }
    }
}

/// Revoke a token
#[actix_web::post("/token/revoke")]
pub async fn revoke_token(
    token_manager: web::Data<SecureTokenManager>,
    request: web::Json<RevokeTokenRequest>,
) -> impl Responder {
    match token_manager.revoke_token(&request.token) {
        Ok(()) => {
            log::info!("Successfully revoked token");
            HttpResponse::Ok().body("Token revoked successfully")
        }
        Err(e) => {
            HttpResponse::BadRequest().body(format!("Failed to revoke token: {}", e))
        }
    }
}

/// Get token information
#[actix_web::get("/token/info")]
pub async fn get_token_info(
    token_manager: web::Data<SecureTokenManager>,
    req: HttpRequest,
) -> impl Responder {
    // Extract token from Authorization header
    let auth_header = req.headers().get("Authorization");
    let token_str = if let Some(header) = auth_header {
        if let Ok(header_str) = header.to_str() {
            if header_str.starts_with("Bearer ") {
                Some(header_str[7..].to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    if token_str.is_none() {
        return HttpResponse::BadRequest().body("Authorization header required");
    }

    match token_manager.validate_token(&token_str.unwrap()) {
        Ok(token) => {
            HttpResponse::Ok().json(TokenInfoResponse {
                token_id: token.id.to_string(),
                created_at: format!("{:?}", token.created_at),
                expires_at: format!("{:?}", token.expires_at),
                permissions: token.permissions,
                is_expired: token.is_expired(),
                is_revoked: token.is_revoked,
            })
        }
        Err(e) => {
            HttpResponse::BadRequest().body(format!("Invalid token: {}", e))
        }
    }
}

/// Cleanup expired tokens (admin endpoint)
#[actix_web::post("/token/cleanup")]
pub async fn cleanup_tokens(
    token_manager: web::Data<SecureTokenManager>,
) -> impl Responder {
    let cleaned_count = token_manager.cleanup_expired_tokens();
    log::info!("Cleaned up {} expired tokens", cleaned_count);
    HttpResponse::Ok().body(format!("Cleaned up {} expired tokens", cleaned_count))
}

/// Get security configuration (admin endpoint)
#[actix_web::get("/security/config")]
pub async fn get_security_config(
    config: web::Data<SecurityConfig>,
) -> impl Responder {
    HttpResponse::Ok().json(&*config)
}
