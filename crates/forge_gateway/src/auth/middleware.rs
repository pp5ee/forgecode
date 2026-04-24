use actix_web::{dev::ServiceRequest, Error, HttpMessage, http::header};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use std::net::SocketAddr;
use crate::auth::security::{SecureTokenManager, SecureToken, AuthError};

/// Enhanced URL token authentication with security features
pub async fn secure_url_token_auth(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // Extract client information for security auditing
    let client_ip = extract_client_ip(&req);
    let user_agent = extract_user_agent(&req);

    // First try to extract token from URL parameters with security validation
    if let Some(token_str) = extract_token_from_url(&req) {
        let token_manager = req.app_data::<actix_web::web::Data<SecureTokenManager>>()
            .expect("SecureTokenManager not found in app data");

        match token_manager.validate_token(&token_str) {
            Ok(token_data) => {
                // Perform additional security checks
                if let Err(e) = validate_token_usage(&token_data, &client_ip, &user_agent) {
                    log::warn!("Security violation for token {}: {}", token_data.id, e);
                    // Continue to check Authorization header
                } else {
                    // Token is valid and secure
                    req.extensions_mut().insert(token_data);
                    return Ok(req);
                }
            }
            Err(_) => {
                // URL token is invalid, fall through to check Authorization header
            }
        }
    }

    // If no URL token or it's invalid, check Authorization header
    let token_manager = req.app_data::<actix_web::web::Data<SecureTokenManager>>()
        .expect("SecureTokenManager not found in app data");

    // Validate the token from Authorization header
    match token_manager.validate_token(credentials.token()) {
        Ok(token_data) => {
            // Perform security checks for Authorization header token
            if let Err(e) = validate_token_usage(&token_data, &client_ip, &user_agent) {
                log::error!("Security violation for Authorization token {}: {}", token_data.id, e);
                let config = req.app_data::<Config>().cloned().unwrap_or_default();
                return Err((AuthenticationError::from(config).into(), req));
            }

            // Add token data to request extensions for use in handlers
            req.extensions_mut().insert(token_data);
            Ok(req)
        }
        Err(e) => {
            let config = req.app_data::<Config>().cloned().unwrap_or_default();
            Err((AuthenticationError::from(config).into(), req))
        }
    }
}

/// Extract token from URL parameters with security validation
pub fn extract_token_from_url(req: &ServiceRequest) -> Option<String> {
    let query_string = req.query_string();
    if query_string.is_empty() {
        return None;
    }

    // Parse URL parameters to find token
    let params: Vec<(&str, &str)> = url::form_urlencoded::parse(query_string.as_bytes())
        .into_owned()
        .collect();

    for (key, value) in params {
        if (key == "token" || key == "auth_token" || key == "access_token") && !value.is_empty() {
            // Basic security check: token should be a valid UUID format
            if uuid::Uuid::parse_str(value).is_ok() {
                return Some(value.to_string());
            }
        }
    }

    None
}

/// Extract client IP address from request
fn extract_client_ip(req: &ServiceRequest) -> Option<String> {
    req.connection_info().realip_remote_addr()
        .map(|addr| addr.to_string())
        .or_else(|| {
            req.peer_addr()
                .map(|socket_addr: SocketAddr| socket_addr.ip().to_string())
        })
}

/// Extract User-Agent header from request
fn extract_user_agent(req: &ServiceRequest) -> Option<String> {
    req.headers()
        .get(header::USER_AGENT)
        .and_then(|ua| ua.to_str().ok())
        .map(|s| s.to_string())
}

/// Validate token usage for security
fn validate_token_usage(
    token: &SecureToken,
    client_ip: &Option<String>,
    user_agent: &Option<String>,
) -> Result<(), AuthError> {
    // Check if token is being used from the same client/IP (optional)
    if let (Some(stored_ip), Some(current_ip)) = (&token.client_ip, client_ip) {
        if stored_ip != current_ip {
            return Err(AuthError::SecurityViolation(
                format!("Token used from different IP: {} -> {}", stored_ip, current_ip)
            ));
        }
    }

    // Check if token is being used with the same User-Agent (optional)
    if let (Some(stored_ua), Some(current_ua)) = (&token.user_agent, user_agent) {
        if stored_ua != current_ua {
            return Err(AuthError::SecurityViolation(
                format!("Token used with different User-Agent")
            ));
        }
    }

    Ok(())
}

/// Security headers middleware for authentication endpoints
pub fn security_headers() -> actix_web::middleware::DefaultHeaders {
    actix_web::middleware::DefaultHeaders::new()
        .add((header::STRICT_TRANSPORT_SECURITY, "max-age=31536000; includeSubDomains"))
        .add((header::X_CONTENT_TYPE_OPTIONS, "nosniff"))
        .add((header::X_FRAME_OPTIONS, "DENY"))
        .add((header::X_XSS_PROTECTION, "1; mode=block"))
        .add((header::REFERRER_POLICY, "strict-origin-when-cross-origin"))
}

/// CORS configuration for authentication endpoints
pub fn cors_config() -> actix_cors::Cors {
    actix_cors::Cors::default()
        .allowed_origin_fn(|origin, _req_head| {
            // Allow specific origins or localhost for development
            origin.as_bytes().starts_with(b"http://localhost") ||
            origin.as_bytes().starts_with(b"https://localhost") ||
            origin.as_bytes().starts_with(b"https://") // Require HTTPS for production
        })
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
        .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE])
        .max_age(3600)
}
