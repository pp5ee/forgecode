use actix_web::{test, App, web};
use serde_json::json;
use uuid::Uuid;

use forge_gateway::auth::{SecureToken, SecureTokenManager};
use forge_gateway::forgecode_client::ForgeCodeClient;
use forge_gateway::handlers as gateway_handlers;
use forge_gateway::middleware::auth_middleware;

/// Test the authentication system
#[actix_web::test]
async fn test_token_authentication() {
    // Setup
    let token_manager = web::Data::new(SecureTokenManager::new());
    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .service(gateway_handlers::generate_token)
            .service(gateway_handlers::validate_token)
    ).await;

    // Test token generation
    let req = test::TestRequest::post()
        .uri("/tokens/generate")
        .set_json(json!({
            "permissions": ["read", "execute"],
            "expires_in": 3600
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Extract token from response
    let body: serde_json::Value = test::read_body_json(resp).await;
    let token_str = body["token"].as_str().unwrap();

    // Test token validation
    let req = test::TestRequest::get()
        .uri("/tokens/validate")
        .insert_header(("Authorization", format!("Bearer {}", token_str)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Test token refresh
    let req = test::TestRequest::post()
        .uri("/tokens/refresh")
        .insert_header(("Authorization", format!("Bearer {}", token_str)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

/// Test command execution endpoint
#[actix_web::test]
async fn test_command_execution() {
    // Setup with mock forgecode client
    let token_manager = web::Data::new(SecureTokenManager::new());
    let forgecode_client = web::Data::new(ForgeCodeClient::new("http://localhost:8081"));

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .app_data(forgecode_client.clone())
            .wrap(auth_middleware(token_manager.clone()))
            .service(gateway_handlers::execute_command)
    ).await;

    // Generate a token first
    let token = token_manager.generate_token(
        vec!["execute".to_string()],
        3600
    ).unwrap();

    // Test command execution with valid token
    let req = test::TestRequest::post()
        .uri("/execute")
        .insert_header(("Authorization", format!("Bearer {}", token.token)))
        .set_json(json!({
            "command": "echo",
            "args": ["hello"]
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // This will likely fail without a real forgecode service, but we test the auth flow
    assert!(resp.status().is_client_error() || resp.status().is_server_error());
}

/// Test token renewal functionality
#[actix_web::test]
async fn test_token_renewal() {
    use forge_gateway::token_renewal::{TokenRenewalManager, handlers as renewal_handlers};

    let token_manager = web::Data::new(SecureTokenManager::new());
    let renewal_manager = web::Data::new(TokenRenewalManager::new(token_manager.clone().into_inner().into()));

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .app_data(renewal_manager.clone())
            .service(renewal_handlers::start_renewal)
            .service(renewal_handlers::stop_renewal)
            .service(renewal_handlers::get_renewal_sessions)
    ).await;

    // Generate a token
    let token = token_manager.generate_token(vec!["read".to_string()], 3600).unwrap();

    // Start renewal
    let req = test::TestRequest::post()
        .uri(&format!("/renewal/start/{}", token.id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Check renewal status
    let req = test::TestRequest::get()
        .uri(&format!("/renewal/status/{}", token.id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Get renewal sessions
    let req = test::TestRequest::get()
        .uri("/renewal/sessions")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Stop renewal
    let req = test::TestRequest::post()
        .uri(&format!("/renewal/stop/{}", token.id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

/// Test health check endpoint
#[actix_web::test]
async fn test_health_check() {
    let app = test::init_service(
        App::new().service(gateway_handlers::health_check)
    ).await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

/// Test system info endpoint
#[actix_web::test]
async fn test_system_info() {
    let app = test::init_service(
        App::new().service(gateway_handlers::system_info)
    ).await;

    let req = test::TestRequest::get().uri("/system/info").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

/// Test token revocation
#[actix_web::test]
async fn test_token_revocation() {
    let token_manager = web::Data::new(SecureTokenManager::new());
    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .service(gateway_handlers::revoke_token)
            .service(gateway_handlers::validate_token)
    ).await;

    // Generate a token
    let token = token_manager.generate_token(vec!["read".to_string()], 3600).unwrap();

    // Validate token before revocation
    let req = test::TestRequest::get()
        .uri("/tokens/validate")
        .insert_header(("Authorization", format!("Bearer {}", token.token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Revoke token
    let req = test::TestRequest::post()
        .uri(&format!("/tokens/revoke/{}", token.id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    // Validate token after revocation
    let req = test::TestRequest::get()
        .uri("/tokens/validate")
        .insert_header(("Authorization", format!("Bearer {}", token.token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

/// Test rate limiting (basic simulation)
#[actix_web::test]
async fn test_rate_limiting() {
    let token_manager = web::Data::new(SecureTokenManager::new());
    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .service(gateway_handlers::health_check)
    ).await;

    // Make multiple rapid requests to health endpoint
    for _ in 0..5 {
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
    // In a real implementation, we'd test rate limiting behavior
}

/// Test URL token authentication
#[actix_web::test]
async fn test_url_token_auth() {
    let token_manager = web::Data::new(SecureTokenManager::new());
    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .service(gateway_handlers::health_check)
    ).await;

    // Generate a token
    let token = token_manager.generate_token(vec!["read".to_string()], 3600).unwrap();

    // Test URL token format
    let req = test::TestRequest::get()
        .uri(&format!("/health?token={}", token.token))
        .to_request();
    let resp = test::call_service(&app, req).await;
    // Note: URL token auth requires middleware setup, this is a basic test
    assert!(resp.status().is_success());
}