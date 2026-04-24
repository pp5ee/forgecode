//! Integration tests for the forgecode gateway

use actix_web::{test, web, App};
use forge_gateway::{handlers, auth, GatewayConfig, run_server};
use serde_json::json;
use std::sync::{Arc, Mutex};

#[actix_rt::test]
async fn test_health_check() {
    let app = test::init_service(
        App::new()
            .route("/health", web::get().to(handlers::health_check))
    ).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
    assert_eq!(json["service"], "forgecode-gateway");
}

#[actix_rt::test]
async fn test_token_generation() {
    let token_manager = web::Data::new(Mutex::new(auth::TokenManager::new()));

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .route("/auth/token/generate", web::post().to(handlers::generate_token))
    ).await;

    let req = test::TestRequest::post().uri("/auth/token/generate").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);
    assert!(json["token"].is_string());
}

#[actix_rt::test]
async fn test_token_validation() {
    let token_manager = web::Data::new(Mutex::new(auth::TokenManager::new()));

    // Generate a token first
    let token = token_manager.lock().unwrap().generate_token();

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .route("/auth/token/validate", web::post().to(handlers::validate_token))
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/token/validate")
        .set_json(&json!({ "token": token }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["valid"], true);
}

#[actix_rt::test]
async fn test_token_renewal() {
    let token_manager = web::Data::new(Mutex::new(auth::TokenManager::new()));

    // Generate a token first
    let token = token_manager.lock().unwrap().generate_token();

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .route("/auth/token/renew", web::post().to(handlers::renew_token))
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/token/renew")
        .set_json(&json!({ "token": token }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["success"], true);
    assert!(json["new_token"].is_string());
}

#[actix_rt::test]
async fn test_invalid_token_validation() {
    let token_manager = web::Data::new(Mutex::new(auth::TokenManager::new()));

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .route("/auth/token/validate", web::post().to(handlers::validate_token))
    ).await;

    let req = test::TestRequest::post()
        .uri("/auth/token/validate")
        .set_json(&json!({ "token": "invalid_token" }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body = test::read_body(resp).await;
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["valid"], false);
}

#[actix_rt::test]
async fn test_middleware_protection() {
    let token_manager = web::Data::new(Mutex::new(auth::TokenManager::new()));
    let middleware = auth::UrlTokenMiddleware::new(token_manager.clone());

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .wrap(middleware)
            .route("/protected", web::get().to(|| async { "Protected content" }))
            .route("/health", web::get().to(handlers::health_check))
    ).await;

    // Test that health endpoint is accessible without token
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Test that protected endpoint requires token
    let req = test::TestRequest::get().uri("/protected").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_redirection()); // Should redirect to auth error

    // Test that protected endpoint works with valid token
    let token = token_manager.lock().unwrap().generate_token();
    let req = test::TestRequest::get()
        .uri(&format!("/protected?token={}", token))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}