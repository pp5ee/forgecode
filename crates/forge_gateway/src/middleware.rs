use actix_web::{dev::ServiceRequest, Error, HttpMessage};
use actix_web_httpauth::extractors::bearer::{BearerAuth, Config};
use actix_web_httpauth::extractors::AuthenticationError;
use crate::auth::{TokenManager, AuthError};

/// Middleware to extract token from URL parameters or Authorization header
pub async fn token_validator(
    req: ServiceRequest,
    credentials: BearerAuth,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    let token_manager = req.app_data::<actix_web::web::Data<TokenManager>>()
        .expect("TokenManager not found in app data");

    // Validate the token
    match token_manager.validate_token(credentials.token()) {
        Ok(token_data) => {
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

/// Extract token from URL parameters if present
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
        if key == "token" && !value.is_empty() {
            return Some(value.to_string());
        }
    }

    None
}

/// Create a wrapper that checks both URL parameters and Authorization header
pub async fn url_token_auth(
    req: ServiceRequest,
    credentials: Option<BearerAuth>,
) -> Result<ServiceRequest, (Error, ServiceRequest)> {
    // First try to extract token from URL parameters
    if let Some(token) = extract_token_from_url(&req) {
        let token_manager = req.app_data::<actix_web::web::Data<TokenManager>>()
            .expect("TokenManager not found in app data");

        match token_manager.validate_token(&token) {
            Ok(token_data) => {
                req.extensions_mut().insert(token_data);
                return Ok(req);
            }
            Err(_) => {
                // URL token is invalid, fall through to check Authorization header
            }
        }
    }

    // If no URL token or it's invalid, check Authorization header
    if let Some(credentials) = credentials {
        token_validator(req, credentials).await
    } else {
        let config = req.app_data::<Config>().cloned().unwrap_or_default();
        Err((AuthenticationError::from(config).into(), req))
    }
}