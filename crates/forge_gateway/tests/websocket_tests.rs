use actix_web::{test, App, web};
use actix_web_actors::ws;
use serde_json::json;
use std::time::Duration;

use forge_gateway::auth::SecureTokenManager;
use forge_gateway::handlers as gateway_handlers;
use forge_gateway::websocket::WebSocketConnection;

/// Test WebSocket connection establishment
#[actix_web::test]
async fn test_websocket_connection() {
    let token_manager = web::Data::new(SecureTokenManager::new());

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .service(gateway_handlers::websocket_handler)
    ).await;

    // Generate a token for authentication
    let token = token_manager.generate_token(vec!["execute".to_string()], 3600).unwrap();

    // Create WebSocket test request
    let req = test::TestRequest::get()
        .uri(&format!("/ws?token={}", token.token))
        .to_request();

    // Test WebSocket handshake
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

/// Test WebSocket message handling
#[actix_web::test]
async fn test_websocket_message_flow() {
    // This test would require more complex setup with mock forgecode client
    // For now, we'll test the basic WebSocket actor functionality

    let token_manager = web::Data::new(SecureTokenManager::new());

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .service(gateway_handlers::websocket_handler)
    ).await;

    // Generate a token
    let token = token_manager.generate_token(vec!["execute".to_string()], 3600).unwrap();

    // Test WebSocket connection with valid token
    let req = test::TestRequest::get()
        .uri(&format!("/ws?token={}", token.token))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

/// Test WebSocket authentication failure
#[actix_web::test]
async fn test_websocket_auth_failure() {
    let token_manager = web::Data::new(SecureTokenManager::new());

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .service(gateway_handlers::websocket_handler)
    ).await;

    // Test with invalid token
    let req = test::TestRequest::get()
        .uri("/ws?token=invalid_token")
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should fail authentication
    assert!(resp.status().is_client_error());

    // Test with expired token
    let expired_token = token_manager.generate_token(vec!["execute".to_string()], 1).unwrap();

    // Manually expire the token
    let mut tokens = token_manager.tokens.write().unwrap();
    if let Some(token) = tokens.get_mut(&expired_token.id) {
        token.expires_at = std::time::SystemTime::now() - std::time::Duration::from_secs(10);
    }
    drop(tokens);

    let req = test::TestRequest::get()
        .uri(&format!("/ws?token={}", expired_token.token))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should fail with expired token
    assert!(resp.status().is_client_error());
}

/// Test WebSocket session management
#[actix_web::test]
async fn test_websocket_session_management() {
    // This would test session creation, tracking, and cleanup
    // For now, we'll verify the WebSocket actor can be created

    let token_manager = web::Data::new(SecureTokenManager::new());
    let token = token_manager.generate_token(vec!["execute".to_string()], 3600).unwrap();

    // Test WebSocket actor creation
    let ws = WebSocketConnection::new(token.id, token_manager.clone());

    // Verify actor has proper state
    assert_eq!(ws.token_id, token.id);
    // Additional state verification would go here
}

/// Test WebSocket heartbeat functionality
#[actix_web::test]
async fn test_websocket_heartbeat() {
    // Test that heartbeat messages are properly handled
    // This would require more complex WebSocket testing infrastructure

    // For now, we'll test the basic WebSocket flow
    let token_manager = web::Data::new(SecureTokenManager::new());

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .service(gateway_handlers::websocket_handler)
    ).await;

    let token = token_manager.generate_token(vec!["execute".to_string()], 3600).unwrap();

    let req = test::TestRequest::get()
        .uri(&format!("/ws?token={}", token.token))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

/// Test WebSocket command execution flow
#[actix_web::test]
async fn test_websocket_command_execution() {
    // This test would simulate the full command execution flow via WebSocket
    // It would require mocking the forgecode client and command execution

    // For now, we'll test the authentication and connection setup
    let token_manager = web::Data::new(SecureTokenManager::new());

    let app = test::init_service(
        App::new()
            .app_data(token_manager.clone())
            .service(gateway_handlers::websocket_handler)
    ).await;

    let token = token_manager.generate_token(vec!["execute".to_string()], 3600).unwrap();

    let req = test::TestRequest::get()
        .uri(&format!("/ws?token={}", token.token))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}