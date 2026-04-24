//! Test suite for forgecode-gateway

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App};
    use crate::auth::token_manager::TokenManager;
    use crate::handlers::*;
    use crate::middleware::auth_middleware;

    #[actix_web::test]
    async fn test_token_generation() {
        let token_manager = TokenManager::new();

        // Test generating a token
        let token = token_manager.generate_token("test-user", None).await;
        assert!(token.is_ok());

        let token_value = token.unwrap();
        assert!(!token_value.is_empty());
    }

    #[actix_web::test]
    async fn test_token_validation() {
        let token_manager = TokenManager::new();

        // Generate and validate a token
        let token = token_manager.generate_token("test-user", None).await.unwrap();
        let validation = token_manager.validate_token(&token).await;

        assert!(validation.is_ok());
        let claims = validation.unwrap();
        assert_eq!(claims.user_id, "test-user");
    }

    #[actix_web::test]
    async fn test_invalid_token() {
        let token_manager = TokenManager::new();

        // Test invalid token
        let validation = token_manager.validate_token("invalid-token").await;
        assert!(validation.is_err());
    }

    #[actix_web::test]
    async fn test_expired_token() {
        let token_manager = TokenManager::new();

        // Generate token with very short expiration
        let token = token_manager.generate_token_with_expiry("test-user", std::time::Duration::from_secs(1)).await.unwrap();

        // Wait for token to expire
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

        let validation = token_manager.validate_token(&token).await;
        assert!(validation.is_err());
    }

    #[actix_web::test]
    async fn test_health_endpoint() {
        let app = test::init_service(
            App::new()
                .service(health_handler)
        ).await;

        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_static_file_serving() {
        let app = test::init_service(
            App::new()
                .service(static_files_handler)
        ).await;

        // Test serving index.html
        let req = test::TestRequest::get()
            .uri("/")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_command_execution() {
        let app = test::init_service(
            App::new()
                .service(command_handler)
        ).await;

        let command_request = serde_json::json!({
            "command": "echo",
            "args": ["hello"],
            "working_dir": "/tmp"
        });

        let req = test::TestRequest::post()
            .uri("/api/command")
            .set_json(&command_request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_file_operations() {
        let app = test::init_service(
            App::new()
                .service(file_read_handler)
        ).await;

        let file_request = serde_json::json!({
            "path": "/etc/hosts",
            "offset": 0,
            "limit": 100
        });

        let req = test::TestRequest::post()
            .uri("/api/file/read")
            .set_json(&file_request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_authentication_middleware() {
        let token_manager = TokenManager::new();
        let token = token_manager.generate_token("test-user", None).await.unwrap();

        let app = test::init_service(
            App::new()
                .wrap(auth_middleware::AuthMiddleware::new(token_manager))
                .service(health_handler)
        ).await;

        // Test with valid token
        let req = test::TestRequest::get()
            .uri("/health")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());

        // Test without token
        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }

    #[actix_web::test]
    async fn test_url_token_authentication() {
        let token_manager = TokenManager::new();
        let token = token_manager.generate_token("test-user", None).await.unwrap();

        let app = test::init_service(
            App::new()
                .wrap(auth_middleware::AuthMiddleware::new(token_manager))
                .service(health_handler)
        ).await;

        // Test with URL token
        let req = test::TestRequest::get()
            .uri(&format!("/health?token={}", token))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}