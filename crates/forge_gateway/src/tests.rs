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
}