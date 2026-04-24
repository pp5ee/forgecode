//! HTTP request handlers for the gateway

use actix_web::{HttpResponse, Responder};
use serde_json::json;

/// Health check endpoint
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "ok",
        "service": "forgecode-gateway",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

/// Validate token endpoint
pub async fn validate_token() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "valid": true,
        "message": "Token validation endpoint"
    }))
}

/// Renew token endpoint
pub async fn renew_token() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "success": true,
        "message": "Token renewal endpoint"
    }))
}