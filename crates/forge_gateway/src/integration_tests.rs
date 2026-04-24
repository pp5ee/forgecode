#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::TokenManager;
    use crate::forgecode_client::{ForgeCodeClient, ExecuteCodeRequest, ValidateTokenRequest};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    use warp::test::request;
    use warp::Filter;

    #[tokio::test]
    async fn test_token_generation_and_validation() {
        let token_manager = TokenManager::new("http://localhost:8081".to_string());

        // Generate token
        let user_id = Some("test_user".to_string());
        let permissions = vec!["execute".to_string(), "read".to_string()];

        let token_id = token_manager.generate_token(user_id.clone(), permissions.clone())
            .await
            .expect("Failed to generate token");

        // Validate token
        let token_data = token_manager.validate_token(token_id, None)
            .await
            .expect("Failed to validate token");

        assert_eq!(token_data.user_id, user_id);
        assert_eq!(token_data.permissions, permissions);
        assert!(token_data.expires_at > std::time::SystemTime::now());
    }

    #[tokio::test]
    async fn test_token_renewal() {
        let token_manager = TokenManager::new("http://localhost:8081".to_string());

        // Generate initial token
        let initial_token = token_manager.generate_token(None, vec!["test".to_string()])
            .await
            .expect("Failed to generate token");

        // Renew token
        let renewed_token = token_manager.renew_token(initial_token)
            .await
            .expect("Failed to renew token");

        // Verify new token is valid
        let renewed_data = token_manager.validate_token(renewed_token, None)
            .await
            .expect("Renewed token should be valid");

        assert!(renewed_data.expires_at > std::time::SystemTime::now());
    }

    #[tokio::test]
    async fn test_token_revocation() {
        let token_manager = TokenManager::new("http://localhost:8081".to_string());

        // Generate and revoke token
        let token_id = token_manager.generate_token(None, vec!["test".to_string()])
            .await
            .expect("Failed to generate token");

        token_manager.revoke_token(token_id)
            .await
            .expect("Failed to revoke token");

        // Verify token is no longer valid
        let result = token_manager.validate_token(token_id, None).await;
        assert!(result.is_err(), "Revoked token should not be valid");
    }

    #[tokio::test]
    async fn test_permission_validation() {
        let token_manager = TokenManager::new("http://localhost:8081".to_string());

        // Generate token with specific permissions
        let token_id = token_manager.generate_token(
            Some("test_user".to_string()),
            vec!["execute".to_string(), "read".to_string()]
        )
        .await
        .expect("Failed to generate token");

        // Test with required permission
        let result = token_manager.validate_token(token_id, Some("execute")).await;
        assert!(result.is_ok(), "Token should have execute permission");

        // Test with missing permission
        let result = token_manager.validate_token(token_id, Some("admin")).await;
        assert!(result.is_err(), "Token should not have admin permission");
    }

    #[tokio::test]
    async fn test_http_api_endpoints() {
        let token_manager = TokenManager::new("http://localhost:8081".to_string());
        let forgecode_client = ForgeCodeClient::new("http://localhost:8081".to_string());
        let routes = crate::server::create_routes(token_manager, forgecode_client);

        // Test execute endpoint
        let resp = request()
            .method("POST")
            .path("/api/execute")
            .json(&serde_json::json!({
                "code": "print('hello')",
                "language": "python",
                "session_id": "test-session"
            }))
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);

        // Test token generation endpoint
        let resp = request()
            .method("POST")
            .path("/api/auth/generate")
            .json(&serde_json::json!({
                "user_id": "test_user",
                "permissions": ["execute"]
            }))
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);

        // Extract token from response for validation test
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        let token = body["token"].as_str().unwrap();

        // Test token validation endpoint
        let resp = request()
            .method("POST")
            .path("/api/auth/validate")
            .json(&serde_json::json!({
                "token": token
            }))
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_static_file_serving() {
        let token_manager = TokenManager::new("http://localhost:8081".to_string());
        let forgecode_client = ForgeCodeClient::new("http://localhost:8081".to_string());
        let routes = crate::server::create_routes(token_manager, forgecode_client);

        // Test static file serving
        let resp = request()
            .method("GET")
            .path("/static/index.html")
            .reply(&routes)
            .await;

        // Static files should be served (200) or not found (404), but not server error
        assert!(resp.status() == 200 || resp.status() == 404);
    }

    #[tokio::test]
    async fn test_websocket_message_handling() {
        let token_manager = Arc::new(TokenManager::new("http://localhost:8080".to_string()));
        let forgecode_client = Arc::new(Mutex::new(ForgeCodeClient::new("http://localhost:8081".to_string())));

        // Test authentication message
        let auth_message = crate::server::WebSocketMessage {
            action: "authenticate".to_string(),
            code: None,
            language: None,
            session_id: None,
            token: Some("test_user".to_string()),
        };

        let response = crate::server::handle_websocket_authenticate(
            auth_message,
            token_manager.clone(),
        ).await;

        assert_eq!(response.action, "authentication_result");
        assert!(response.output.is_some());
        assert!(response.error.is_none());

        // Test validation message
        let validate_message = crate::server::WebSocketMessage {
            action: "validate".to_string(),
            code: None,
            language: None,
            session_id: None,
            token: response.output,
        };

        let response = crate::server::handle_websocket_validate(
            validate_message,
            token_manager.clone(),
        ).await;

        assert_eq!(response.action, "validation_result");
        assert!(response.output.is_some());
        assert!(response.error.is_none());
    }

    #[tokio::test]
    async fn test_error_handling() {
        let token_manager = TokenManager::new("http://localhost:8081".to_string());
        let forgecode_client = ForgeCodeClient::new("http://localhost:8081".to_string());
        let routes = crate::server::create_routes(token_manager, forgecode_client);

        // Test invalid JSON
        let resp = request()
            .method("POST")
            .path("/api/execute")
            .body("invalid json")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 400);

        // Test missing required fields
        let resp = request()
            .method("POST")
            .path("/api/execute")
            .json(&serde_json::json!({
                "language": "python"
                // missing code field
            }))
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 400);
    }

    #[tokio::test]
    async fn test_concurrent_token_operations() {
        let token_manager = Arc::new(TokenManager::new("http://localhost:8080".to_string()));

        let mut handles = vec![];

        // Spawn multiple concurrent operations
        for i in 0..10 {
            let tm = token_manager.clone();
            let handle = tokio::spawn(async move {
                let user_id = format!("user_{}", i);
                let token_id = tm.generate_token(Some(user_id), vec!["test".to_string()])
                    .await
                    .expect("Failed to generate token");

                // Validate immediately
                let data = tm.validate_token(token_id, None)
                    .await
                    .expect("Failed to validate token");

                data.user_id
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        let results = futures::future::join_all(handles).await;

        for result in results {
            let user_id = result.expect("Task failed");
            assert!(user_id.expect("User ID should be present").starts_with("user_"));
        }
    }
}