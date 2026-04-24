//! HTTP request handlers for the gateway

use actix_web::{HttpResponse, Responder, web};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Mutex;
use crate::auth::{TokenManager, TokenValidation};

/// Request body for token validation
#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub token: String,
}

/// Request body for token renewal
#[derive(Debug, Deserialize)]
pub struct RenewTokenRequest {
    pub token: String,
}

/// Response for token operations
#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_token: Option<String>,
}

/// Health check endpoint
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "forgecode-gateway",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Validate token endpoint
pub async fn validate_token(
    token_request: web::Json<TokenRequest>,
    token_manager: web::Data<Mutex<TokenManager>>,
) -> impl Responder {
    let token = &token_request.token;

    let validation = token_manager.lock().unwrap().validate_token(token);

    HttpResponse::Ok().json(json!({
        "valid": validation.valid,
        "message": validation.message,
        "token_id": validation.token_id
    }))
}

/// Renew token endpoint
pub async fn renew_token(
    renew_request: web::Json<RenewTokenRequest>,
    token_manager: web::Data<Mutex<TokenManager>>,
) -> impl Responder {
    let token = &renew_request.token;

    let mut manager = token_manager.lock().unwrap();
    if let Some(renewed_token) = manager.renew_token(token) {
        HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Token renewed successfully",
            "new_token": renewed_token.token
        }))
    } else {
        HttpResponse::BadRequest().json(json!({
            "success": false,
            "message": "Token not found or invalid",
            "new_token": null
        }))
    }
}

/// Generate new token endpoint
pub async fn generate_token(
    token_manager: web::Data<Mutex<TokenManager>>,
) -> impl Responder {
    let token = token_manager.lock().unwrap().generate_token();

    HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Token generated successfully",
        "token": token.token,
        "expires_at": token.expires_at.to_rfc3339()
    }))
}

/// Serve the main index page with token validation
pub async fn serve_index() -> impl Responder {
    // This handler is called after middleware validation
    // Redirect to the static index.html file
    HttpResponse::TemporaryRedirect()
        .append_header(("Location", "/static/index.html"))
        .finish()
}