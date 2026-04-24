//! Test suite for forgecode-gateway

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, http::StatusCode};
    use serde_json::json;
    use std::sync::Mutex;

    use crate::auth::TokenManager;
    use crate::handlers;
    use crate::integration;

    #[actix_web::test]
    async fn test_health_endpoint() {
        let app = test::init_service(
            App::new()
                .route("/health", web::get().to(handlers::health_check))
        ).await;

        let req = test::TestRequest::get()
            .uri("/health")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("forgecode-gateway"));
    }

    #[actix_web::test]
    async fn test_token_generation() {
        let token_manager = web::Data::new(Mutex::new(TokenManager::new()));

        let app = test::init_service(
            App::new()
                .app_data(token_manager.clone())
                .route("/auth/token/generate", web::post().to(handlers::generate_token))
        ).await;

        let req = test::TestRequest::post()
            .uri("/auth/token/generate")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        let json: serde_json::Value = serde_json::from_str(&body_str).unwrap();

        assert!(json["success"].as_bool().unwrap());
        assert!(json["token"].as_str().is_some());
    }

    #[actix_web::test]
    async fn test_token_validation() {
        let token_manager = web::Data::new(Mutex::new(TokenManager::new()));

        // First generate a token
        let generate_app = test::init_service(
            App::new()
                .app_data(token_manager.clone())
                .route("/auth/token/generate", web::post().to(handlers::generate_token))
        ).await;

        let generate_req = test::TestRequest::post()
            .uri("/auth/token/generate")
            .to_request();
        let generate_resp = test::call_service(&generate_app, generate_req).await;

        let generate_body = test::read_body(generate_resp).await;
        let generate_json: serde_json::Value = serde_json::from_slice(&generate_body).unwrap();
        let token = generate_json["token"].as_str().unwrap();

        // Now validate the token
        let validate_app = test::init_service(
            App::new()
                .app_data(token_manager.clone())
                .route("/auth/token/validate", web::post().to(handlers::validate_token))
        ).await;

        let validate_req = test::TestRequest::post()
            .uri("/auth/token/validate")
            .set_json(&json!({ "token": token }))
            .to_request();
        let validate_resp = test::call_service(&validate_app, validate_req).await;

        assert!(validate_resp.status().is_success());

        let validate_body = test::read_body(validate_resp).await;
        let validate_json: serde_json::Value = serde_json::from_slice(&validate_body).unwrap();

        assert!(validate_json["valid"].as_bool().unwrap());
    }

    #[actix_web::test]
    async fn test_invalid_token_validation() {
        let token_manager = web::Data::new(Mutex::new(TokenManager::new()));

        let app = test::init_service(
            App::new()
                .app_data(token_manager.clone())
                .route("/auth/token/validate", web::post().to(handlers::validate_token))
        ).await;

        let req = test::TestRequest::post()
            .uri("/auth/token/validate")
            .set_json(&json!({ "token": "invalid-token" }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(!json["valid"].as_bool().unwrap());
    }

    #[actix_web::test]
    async fn test_token_renewal() {
        let token_manager = web::Data::new(Mutex::new(TokenManager::new()));

        // First generate a token
        let generate_app = test::init_service(
            App::new()
                .app_data(token_manager.clone())
                .route("/auth/token/generate", web::post().to(handlers::generate_token))
        ).await;

        let generate_req = test::TestRequest::post()
            .uri("/auth/token/generate")
            .to_request();
        let generate_resp = test::call_service(&generate_app, generate_req).await;

        let generate_body = test::read_body(generate_resp).await;
        let generate_json: serde_json::Value = serde_json::from_slice(&generate_body).unwrap();
        let token = generate_json["token"].as_str().unwrap();

        // Now renew the token
        let renew_app = test::init_service(
            App::new()
                .app_data(token_manager.clone())
                .route("/auth/token/renew", web::post().to(handlers::renew_token))
        ).await;

        let renew_req = test::TestRequest::post()
            .uri("/auth/token/renew")
            .set_json(&json!({ "token": token }))
            .to_request();
        let renew_resp = test::call_service(&renew_app, renew_req).await;

        assert!(renew_resp.status().is_success());

        let renew_body = test::read_body(renew_resp).await;
        let renew_json: serde_json::Value = serde_json::from_slice(&renew_body).unwrap();

        assert!(renew_json["success"].as_bool().unwrap());
        assert!(renew_json["new_token"].as_str().is_some());
    }

    #[actix_web::test]
    async fn test_system_info_endpoint() {
        let app = test::init_service(
            App::new()
                .route("/api/system/info", web::get().to(integration::system_info_handler))
        ).await;

        let req = test::TestRequest::get()
            .uri("/api/system/info")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(json["service"].as_str().unwrap(), "forgecode-gateway");
        assert!(json["version"].as_str().is_some());
        assert_eq!(json["status"].as_str().unwrap(), "running");
    }

    #[actix_web::test]
    async fn test_command_execution() {
        let app = test::init_service(
            App::new()
                .route("/api/command", web::post().to(integration::execute_command_handler))
        ).await;

        let req = test::TestRequest::post()
            .uri("/api/command")
            .set_json(&json!({ "command": "help" }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json["success"].as_bool().unwrap());
        assert!(json["output"].as_str().unwrap().contains("help"));
    }

    #[actix_web::test]
    async fn test_file_read_endpoint() {
        let app = test::init_service(
            App::new()
                .route("/api/file/read", web::post().to(integration::read_file_handler))
        ).await;

        // Test reading a file that exists
        let req = test::TestRequest::post()
            .uri("/api/file/read")
            .set_json(&json!({ "path": "Cargo.toml" }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(json["success"].as_bool().unwrap());
        assert!(json["content"].as_str().unwrap().contains("forge_gateway"));
    }

    #[actix_web::test]
    async fn test_file_read_nonexistent() {
        let app = test::init_service(
            App::new()
                .route("/api/file/read", web::post().to(integration::read_file_handler))
        ).await;

        // Test reading a file that doesn't exist
        let req = test::TestRequest::post()
            .uri("/api/file/read")
            .set_json(&json!({ "path": "/nonexistent/file.txt" }))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        let body = test::read_body(resp).await;
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert!(!json["success"].as_bool().unwrap());
        assert!(json["error"].as_str().unwrap().contains("not found"));
    }

    #[actix_web::test]
    async fn test_static_file_serving() {
        // Create a temporary static file for testing
        std::fs::create_dir_all("static").unwrap();
        std::fs::write("static/test.html", "<html><body>Test</body></html>").unwrap();

        let app = test::init_service(
            App::new()
                .service(actix_files::Files::new("/static", "static"))
        ).await;

        let req = test::TestRequest::get()
            .uri("/static/test.html")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert!(resp.status().is_success());

        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("Test"));

        // Clean up
        std::fs::remove_file("static/test.html").unwrap();
        std::fs::remove_dir("static").unwrap();
    }

    #[actix_web::test]
    async fn test_auth_token_expiry() {
        let token_manager = web::Data::new(Mutex::new(TokenManager::new()));

        // Generate a token
        let generate_app = test::init_service(
            App::new()
                .app_data(token_manager.clone())
                .route("/auth/token/generate", web::post().to(handlers::generate_token))
        ).await;

        let generate_req = test::TestRequest::post()
            .uri("/auth/token/generate")
            .to_request();
        let generate_resp = test::call_service(&generate_app, generate_req).await;

        let generate_body = test::read_body(generate_resp).await;
        let generate_json: serde_json::Value = serde_json::from_slice(&generate_body).unwrap();
        let token = generate_json["token"].as_str().unwrap();

        // Revoke the token
        {
            let mut manager = token_manager.lock().unwrap();
            manager.revoke_token(token);
        }

        // Try to validate the revoked token
        let validate_app = test::init_service(
            App::new()
                .app_data(token_manager.clone())
                .route("/auth/token/validate", web::post().to(handlers::validate_token))
        ).await;

        let validate_req = test::TestRequest::post()
            .uri("/auth/token/validate")
            .set_json(&json!({ "token": token }))
            .to_request();
        let validate_resp = test::call_service(&validate_app, validate_req).await;

        assert!(validate_resp.status().is_success());

        let validate_body = test::read_body(validate_resp).await;
        let validate_json: serde_json::Value = serde_json::from_slice(&validate_body).unwrap();

        assert!(!validate_json["valid"].as_bool().unwrap());
    }
}