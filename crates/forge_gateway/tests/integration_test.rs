//! Integration tests for the forge_gateway crate

use forge_gateway::{GatewayServer, ServerConfig, TokenManager};
use forge_api::API;
use forge_config::ForgeConfig;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn test_gateway_server_initialization() {
    // Create a mock API instance
    let api = Arc::new(API::default());
    let config = ServerConfig::default();

    // Create gateway server
    let gateway_server = GatewayServer::new(api, config);

    // Verify server configuration
    assert_eq!(gateway_server.config.port, 8080);
    assert_eq!(gateway_server.config.bind_address, "0.0.0.0");
}

#[tokio::test]
async fn test_token_manager_functionality() {
    let token_manager = TokenManager::new();

    // Test token generation
    let token1 = token_manager.generate_token();
    let token2 = token_manager.generate_token();

    assert!(!token1.is_empty(), "Generated token should not be empty");
    assert!(!token2.is_empty(), "Generated token should not be empty");
    assert_ne!(token1, token2, "Generated tokens should be unique");

    // Test token validation
    assert!(token_manager.validate_token(&token1), "Generated token should be valid");
    assert!(token_manager.validate_token(&token2), "Generated token should be valid");
    assert!(!token_manager.validate_token("invalid_token"), "Invalid token should not validate");

    // Test token cleanup
    token_manager.cleanup_expired_tokens();
    // Should still validate after cleanup (tokens are not expired yet)
    assert!(token_manager.validate_token(&token1), "Token should still be valid after cleanup");
}