//! Integration tests for the forgecode gateway

use actix_web::{test, web, App};
use forge_gateway::{handlers, auth::TokenManager, forgecode_client::ForgeCodeClient};
use serde_json::json;

#[actix_web::test]
async fn test_generate_token() {
    let token_manager = web::Data::new(TokenManager::new());
    let forgecode_client = web::Data::new(ForgeCodeClient::new("http://localhost:8081".to_string()));

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .app_data(forgecode_client.clone())
            .service(handlers::generate_token)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/token/generate")
        .set_json(json!({ "permissions": ["execute", "read"] }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["token"].is_string());
    assert!(body["expires_in"].is_number());
}

#[actix_web::test]
async fn test_renew_token() {
    let token_manager = web::Data::new(TokenManager::new());
    let forgecode_client = web::Data::new(ForgeCodeClient::new("http://localhost:8081".to_string()));

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .app_data(forgecode_client.clone())
            .service(handlers::renew_token)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/token/renew")
        .set_json(json!({ "permissions": ["execute", "read"] }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["token"].is_string());
    assert!(body["expires_in"].is_number());
}

#[actix_web::test]
async fn test_execute_command_with_valid_token() {
    let token_manager = web::Data::new(TokenManager::new());
    let forgecode_client = web::Data::new(ForgeCodeClient::new("http://localhost:8081".to_string()));

    // Generate a token first
    let (token, _) = token_manager.generate_token(vec!["execute".to_string()]).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .app_data(forgecode_client.clone())
            .service(handlers::execute_command)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/command/execute")
        .set_json(json!({
            "command": "echo",
            "args": ["hello"]
        }))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["output"].is_string());
    assert!(body["success"].is_boolean());
}

#[actix_web::test]
async fn test_execute_command_with_invalid_token() {
    let token_manager = web::Data::new(TokenManager::new());
    let forgecode_client = web::Data::new(ForgeCodeClient::new("http://localhost:8081".to_string()));

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .app_data(forgecode_client.clone())
            .service(handlers::execute_command)
    ).await;

    let req = test::TestRequest::post()
        .uri("/api/command/execute")
        .set_json(json!({
            "command": "echo",
            "args": ["hello"]
        }))
        .insert_header(("Authorization", "Bearer invalid-token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // This should fail with authentication error
    assert!(!resp.status().is_success());
}

#[actix_web::test]
async fn test_token_validation() {
    let token_manager = TokenManager::new();

    // Generate a token
    let (token, _) = token_manager.generate_token(vec!["execute".to_string()]).unwrap();

    // Validate the token
    let validation_result = token_manager.validate_token(&token);
    assert!(validation_result.is_ok());

    // Try to validate an invalid token
    let invalid_result = token_manager.validate_token("invalid-token");
    assert!(invalid_result.is_err());
}

#[actix_web::test]
async fn test_static_file_serving() {
    let token_manager = web::Data::new(TokenManager::new());
    let forgecode_client = web::Data::new(ForgeCodeClient::new("http://localhost:8081".to_string()));

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .app_data(forgecode_client.clone())
            .service(actix_files::Files::new("/", "static").index_file("index.html"))
    ).await;

    // Test index.html
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}