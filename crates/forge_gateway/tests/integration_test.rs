//! Integration tests for the forge_gateway crate

use forge_gateway::Gateway;
use forge_api::API;
use forge_config::ForgeConfig;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[tokio::test]
async fn test_gateway_start_stop() {
    // Create a mock API instance
    let api = Arc::new(API::default());
    let config = ForgeConfig::default();

    // Create gateway
    let mut gateway = Gateway::new(api, config);

    // Try to start the gateway (this should fail gracefully in test environment)
    // since we don't have a real API implementation yet
    let result = gateway.start().await;

    // The start should fail gracefully rather than panic
    assert!(result.is_err(), "Gateway start should fail gracefully in test environment");

    // Stop should work even if start failed
    let stop_result = gateway.stop().await;
    assert!(stop_result.is_ok(), "Gateway stop should work even if start failed");
}

#[tokio::test]
async fn test_gateway_token_management() {
    let api = Arc::new(API::default());
    let config = ForgeConfig::default();
    let gateway = Gateway::new(api, config);

    // Test token generation
    let token1 = gateway.generate_token();
    let token2 = gateway.generate_token();

    assert!(!token1.is_empty(), "Generated token should not be empty");
    assert!(!token2.is_empty(), "Generated token should not be empty");
    assert_ne!(token1, token2, "Generated tokens should be unique");

    // Test token validation
    assert!(gateway.validate_token(&token1), "Generated token should be valid");
    assert!(gateway.validate_token(&token2), "Generated token should be valid");
    assert!(!gateway.validate_token("invalid_token"), "Invalid token should not validate");
}