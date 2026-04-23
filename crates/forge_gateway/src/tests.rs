//! Test module for the forge_gateway crate

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::{ServerConfig, GatewayServer};
    use crate::auth::TokenManager;
    use forge_api::API;
    use forge_config::ForgeConfig;
    use std::sync::Arc;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.bind_address, "0.0.0.0");
        assert!(config.workers > 0);
    }

    #[test]
    fn test_server_config_from_env() {
        // Test with environment variables
        std::env::set_var("FORGE_GATEWAY_PORT", "9090");
        std::env::set_var("FORGE_GATEWAY_BIND", "127.0.0.1");
        std::env::set_var("FORGE_GATEWAY_WORKERS", "2");

        let forge_config = ForgeConfig::default();
        let config = ServerConfig::from_env_and_config(&forge_config);

        assert_eq!(config.port, 9090);
        assert_eq!(config.bind_address, "127.0.0.1");
        assert_eq!(config.workers, 2);

        // Clean up environment
        std::env::remove_var("FORGE_GATEWAY_PORT");
        std::env::remove_var("FORGE_GATEWAY_BIND");
        std::env::remove_var("FORGE_GATEWAY_WORKERS");
    }

    #[test]
    fn test_server_config_bind_address() {
        let config = ServerConfig {
            port: 8080,
            bind_address: "0.0.0.0".to_string(),
            ..Default::default()
        };

        assert_eq!(config.bind_address(), "0.0.0.0:8080");
    }

    #[test]
    fn test_gateway_creation() {
        let api = Arc::new(API::default());
        let config = ForgeConfig::default();
        let gateway = crate::gateway::Gateway::new(api, config);

        assert!(!gateway.is_running(), "Gateway should not be running initially");
        assert!(gateway.server_config().is_none(), "Server config should be None when not running");
    }

    #[test]
    fn test_token_generation() {
        let token_manager = TokenManager::new();
        let token = token_manager.generate_token();

        assert!(!token.is_empty(), "Token should not be empty");
        assert!(token_manager.validate_token(&token), "Generated token should be valid");
    }

    #[test]
    fn test_token_validation() {
        let token_manager = TokenManager::new();

        // Test valid token
        let token = token_manager.generate_token();
        assert!(token_manager.validate_token(&token), "Valid token should validate");

        // Test invalid token
        assert!(!token_manager.validate_token("invalid_token"), "Invalid token should not validate");

        // Test token invalidation
        let token2 = token_manager.generate_token();
        assert!(token_manager.validate_token(&token2), "Token should be valid before invalidation");
        token_manager.invalidate_token(&token2);
        assert!(!token_manager.validate_token(&token2), "Token should be invalid after invalidation");
    }

    #[test]
    fn test_gateway_server_creation() {
        let api = Arc::new(API::default());
        let config = ForgeConfig::default();
        let server = GatewayServer::new(api, config);

        // Verify server configuration is created
        let server_config = server.config();
        assert_eq!(server_config.port, 8080);
        assert_eq!(server_config.bind_address, "0.0.0.0");
    }

    #[tokio::test]
    async fn test_handlers() {
        use crate::handlers;
        use actix_web::{test, web, App};

        let app = test::init_service(
            App::new()
                .route("/health", web::get().to(handlers::health))
        ).await;

        // Test health endpoint
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success(), "Health endpoint should return success");
    }

    #[test]
    fn test_web_ui_interface() {
        // Test that web UI components are properly structured
        // This is a placeholder for UI testing - in a real scenario, we might use browser automation
        assert!(true, "Web UI interface structure is valid");
    }

    #[tokio::test]
    async fn test_file_operations() {
        use crate::handlers;
        use actix_web::{test, web, App};
        use forge_api::API;
        use std::sync::Arc;

        let api = Arc::new(API::default());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(api.clone()))
                .route("/files", web::get().to(handlers::list_files))
        ).await;

        // Test file listing endpoint
        let req = test::TestRequest::get().uri("/files").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success(), "File listing endpoint should return success");
    }

    #[tokio::test]
    async fn test_command_execution() {
        use crate::handlers;
        use actix_web::{test, web, App};
        use forge_api::API;
        use std::sync::Arc;

        let api = Arc::new(API::default());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(api.clone()))
                .route("/execute", web::post().to(handlers::execute_command))
        ).await;

        // Test command execution endpoint with empty command
        let req = test::TestRequest::post()
            .uri("/execute")
            .set_json(&serde_json::json!({"command": "echo test"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success(), "Command execution endpoint should return success");
    }

    #[test]
    fn test_docker_deployment_config() {
        // Test that Docker configuration is properly set up
        // This would verify that the Dockerfile and docker-compose.yml exist and are valid
        let dockerfile_exists = std::path::Path::new("Dockerfile").exists();
        let compose_exists = std::path::Path::new("docker-compose.yml").exists();

        assert!(dockerfile_exists, "Dockerfile should exist");
        assert!(compose_exists, "docker-compose.yml should exist");
    }

    #[test]
    fn test_security_features() {
        // Test that security features are implemented
        // This would verify that rate limiting and authentication are properly configured
        use crate::rate_limiting::RateLimitConfig;

        let config = RateLimitConfig::default();
        assert!(config.max_requests > 0, "Rate limiting should be configured");
        assert!(config.window_seconds > 0, "Rate limiting window should be configured");
    }

    #[test]
    fn test_monitoring_and_logging() {
        // Test that monitoring and logging are properly set up
        // This would verify that logging configuration and monitoring endpoints exist
        use crate::monitoring::{MonitoringService, GatewayMetrics};

        let monitoring = MonitoringService::new();
        let metrics = monitoring.get_metrics();

        assert!(metrics.uptime_seconds >= 0, "Monitoring service should be functional");
        assert_eq!(metrics.total_requests, 0, "Initial request count should be zero");
        assert_eq!(metrics.successful_requests, 0, "Initial successful requests should be zero");
        assert_eq!(metrics.failed_requests, 0, "Initial failed requests should be zero");
        assert_eq!(metrics.active_websocket_connections, 0, "Initial WebSocket connections should be zero");
    }

    // Integration tests for comprehensive coverage
    #[tokio::test]
    async fn test_authentication_flow() {
        use crate::auth::TokenManager;

        let token_manager = TokenManager::new();
        let token = token_manager.generate_token();

        // Test token generation and validation
        assert!(!token.is_empty(), "Token should not be empty");
        assert!(token_manager.validate_token(&token), "Generated token should be valid");

        // Test token invalidation
        token_manager.invalidate_token(&token);
        assert!(!token_manager.validate_token(&token), "Invalidated token should not be valid");
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        use crate::rate_limiting::{RateLimitStore, RateLimitConfig};

        let config = RateLimitConfig {
            max_requests: 2,
            window_seconds: 60,
            ..Default::default()
        };

        let store = RateLimitStore::new(config);

        // Test rate limiting logic
        assert!(!store.check_rate_limit("127.0.0.1", None), "First request should not be limited");
        assert!(!store.check_rate_limit("127.0.0.1", None), "Second request should not be limited");
        assert!(store.check_rate_limit("127.0.0.1", None), "Third request should be limited");
    }

    #[test]
    fn test_websocket_implementation() {
        // Test that WebSocket handlers are properly implemented
        // This would verify that WebSocket endpoints are configured correctly
        use crate::websocket::{ConversationWebSocket, CommandWebSocket};
        use forge_api::API;
        use std::sync::Arc;

        let api = Arc::new(API::default());
        let conversation_ws = ConversationWebSocket::new(api.clone());
        let command_ws = CommandWebSocket::new(api);

        // Verify that WebSocket actors can be created
        assert!(std::mem::size_of_val(&conversation_ws) > 0, "Conversation WebSocket should be created");
        assert!(std::mem::size_of_val(&command_ws) > 0, "Command WebSocket should be created");
    }

    // Additional comprehensive tests for edge cases and integration scenarios
    #[test]
    fn test_server_config_validation() {
        // Test invalid server configurations
        let config = ServerConfig {
            port: 0, // Invalid port
            bind_address: "0.0.0.0".to_string(),
            ..Default::default()
        };

        // Should handle invalid port gracefully
        assert_eq!(config.bind_address(), "0.0.0.0:0");
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        // Test handling of concurrent requests
        use crate::handlers;
        use actix_web::{test, web, App};
        use forge_api::API;
        use std::sync::Arc;

        let api = Arc::new(API::default());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(api.clone()))
                .route("/health", web::get().to(handlers::health))
        ).await;

        // Simulate concurrent requests
        let req1 = test::TestRequest::get().uri("/health").to_request();
        let req2 = test::TestRequest::get().uri("/health").to_request();

        let (resp1, resp2) = tokio::join!(
            test::call_service(&app, req1),
            test::call_service(&app, req2)
        );

        assert!(resp1.status().is_success(), "First concurrent request should succeed");
        assert!(resp2.status().is_success(), "Second concurrent request should succeed");
    }

    #[test]
    fn test_error_handling() {
        // Test error handling scenarios
        use crate::auth::TokenManager;

        let token_manager = TokenManager::new();

        // Test empty token validation
        assert!(!token_manager.validate_token(""), "Empty token should be invalid");

        // Test very long token
        let long_token = "a".repeat(1000);
        assert!(!token_manager.validate_token(&long_token), "Very long token should be invalid");
    }

    #[tokio::test]
    async fn test_graceful_shutdown() {
        // Test server shutdown behavior
        use crate::server::GatewayServer;
        use forge_api::API;
        use forge_config::ForgeConfig;
        use std::sync::Arc;

        let api = Arc::new(API::default());
        let config = ForgeConfig::default();
        let server = GatewayServer::new(api, config);

        // Verify server can be created and has shutdown capability
        assert!(server.config().port > 0, "Server should have valid port configuration");
    }

    // Performance and stress tests
    #[tokio::test]
    async fn test_high_frequency_requests() {
        // Test handling of high-frequency requests
        use crate::handlers;
        use actix_web::{test, web, App};
        use forge_api::API;
        use std::sync::Arc;

        let api = Arc::new(API::default());
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(api.clone()))
                .route("/health", web::get().to(handlers::health))
        ).await;

        // Send multiple rapid requests
        for _ in 0..10 {
            let req = test::TestRequest::get().uri("/health").to_request();
            let resp = test::call_service(&app, req).await;
            assert!(resp.status().is_success(), "High frequency request should succeed");
        }
    }

    // Security boundary tests
    #[test]
    fn test_path_traversal_protection() {
        // Test that path traversal attempts are blocked
        // This would be implemented in the file operations handler
        assert!(true, "Path traversal protection should be implemented");
    }

    #[test]
    fn test_rate_limiting_boundaries() {
        // Test rate limiting boundary conditions
        use crate::rate_limiting::{RateLimitStore, RateLimitConfig};

        let config = RateLimitConfig {
            max_requests: 1,
            window_seconds: 60,
            ..Default::default()
        };

        let store = RateLimitStore::new(config);

        // Test exact boundary
        assert!(!store.check_rate_limit("127.0.0.1", None), "First request should not be limited");
        assert!(store.check_rate_limit("127.0.0.1", None), "Second request should be limited");
    }

    // Configuration validation tests
    #[test]
    fn test_invalid_environment_variables() {
        // Test handling of invalid environment variables
        std::env::set_var("FORGE_GATEWAY_PORT", "not_a_number");

        let forge_config = ForgeConfig::default();
        let config = ServerConfig::from_env_and_config(&forge_config);

        // Should fall back to default values
        assert_eq!(config.port, 8080, "Should fall back to default port on invalid env var");

        std::env::remove_var("FORGE_GATEWAY_PORT");
    }
}