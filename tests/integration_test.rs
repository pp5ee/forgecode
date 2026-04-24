//! Integration tests for forgecode-gateway

use actix_web::{test, web, App};
use forge_gateway::{handlers::*, server::run_server, GatewayConfig};
use std::net::TcpListener;
use tokio::time::{sleep, Duration};

#[actix_web::test]
async fn test_server_startup() {
    // Test basic server startup
    let config = GatewayConfig {
        host: "127.0.0.1".to_string(),
        port: 0, // Use port 0 to get random available port
        static_dir: "static".to_string(),
        log_level: "info".to_string(),
    };

    let server = run_server(config).await;
    assert!(server.is_ok());
}

#[actix_web::test]
async fn test_health_endpoint_integration() {
    // Start a test server
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = actix_web::HttpServer::new(|| {
        App::new()
            .service(health_handler)
    })
    .listen(listener)
    .unwrap()
    .run();

    // Run server in background
    let server_handle = tokio::spawn(server);

    // Give server time to start
    sleep(Duration::from_millis(100)).await;

    // Test health endpoint
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://127.0.0.1:{}/health", port))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    // Stop server
    server_handle.abort();
}

#[actix_web::test]
async fn test_command_execution_integration() {
    // Test command execution with authentication
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = actix_web::HttpServer::new(|| {
        App::new()
            .service(command_handler)
    })
    .listen(listener)
    .unwrap()
    .run();

    let server_handle = tokio::spawn(server);
    sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let command_request = serde_json::json!({
        "command": "echo",
        "args": ["integration-test"],
        "working_dir": "/tmp"
    });

    let response = client
        .post(&format!("http://127.0.0.1:{}/api/command", port))
        .json(&command_request)
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    server_handle.abort();
}

#[actix_web::test]
async fn test_static_file_serving_integration() {
    // Test static file serving
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = actix_web::HttpServer::new(|| {
        App::new()
            .service(static_files_handler)
    })
    .listen(listener)
    .unwrap()
    .run();

    let server_handle = tokio::spawn(server);
    sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://127.0.0.1:{}/", port))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());

    server_handle.abort();
}

#[actix_web::test]
async fn test_websocket_connection() {
    // Test WebSocket connection
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = actix_web::HttpServer::new(|| {
        App::new()
            .service(web::resource("/ws").to(forge_gateway::websocket::websocket_handler))
    })
    .listen(listener)
    .unwrap()
    .run();

    let server_handle = tokio::spawn(server);
    sleep(Duration::from_millis(100)).await;

    // Test WebSocket connection (this would require a WebSocket client)
    // For now, just verify the endpoint exists
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://127.0.0.1:{}/ws", port))
        .send()
        .await
        .unwrap();

    // WebSocket endpoint should return upgrade required
    assert_eq!(response.status().as_u16(), 426);

    server_handle.abort();
}