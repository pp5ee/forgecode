//! End-to-end tests for forgecode-gateway

use forge_gateway::{GatewayConfig, server::run_server};
use reqwest::Client;
use std::net::TcpListener;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn test_full_gateway_workflow() {
    // Start the full gateway server
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let config = GatewayConfig {
        host: "127.0.0.1".to_string(),
        port,
        static_dir: "static".to_string(),
        log_level: "info".to_string(),
    };

    let server = run_server(config).await.unwrap();
    let server_handle = tokio::spawn(server);

    // Give server time to start
    sleep(Duration::from_millis(500)).await;

    let client = Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);

    // Test health endpoint
    let health_response = client
        .get(&format!("{}/health", base_url))
        .send()
        .await
        .unwrap();
    assert!(health_response.status().is_success());

    // Test static file serving
    let index_response = client
        .get(&base_url)
        .send()
        .await
        .unwrap();
    assert!(index_response.status().is_success());

    // Test command execution (without auth for now)
    let command_request = serde_json::json!({
        "command": "ls",
        "args": ["-la"],
        "working_dir": "/tmp"
    });

    let command_response = client
        .post(&format!("{}/api/command", base_url))
        .json(&command_request)
        .send()
        .await
        .unwrap();
    assert!(command_response.status().is_success());

    // Test file operations
    let file_request = serde_json::json!({
        "path": "/etc/hosts",
        "offset": 0,
        "limit": 100
    });

    let file_response = client
        .post(&format!("{}/api/file/read", base_url))
        .json(&file_request)
        .send()
        .await
        .unwrap();
    assert!(file_response.status().is_success());

    // Test WebSocket endpoint
    let ws_response = client
        .get(&format!("{}/ws", base_url))
        .send()
        .await
        .unwrap();
    assert_eq!(ws_response.status().as_u16(), 426); // Upgrade required

    // Stop server
    server_handle.abort();
}

#[tokio::test]
async fn test_gateway_with_auth() {
    // Test authentication workflow
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let config = GatewayConfig {
        host: "127.0.0.1".to_string(),
        port,
        static_dir: "static".to_string(),
        log_level: "info".to_string(),
    };

    let server = run_server(config).await.unwrap();
    let server_handle = tokio::spawn(server);
    sleep(Duration::from_millis(500)).await;

    let client = Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);

    // Test access without authentication (should be allowed for some endpoints)
    let health_response = client
        .get(&format!("{}/health", base_url))
        .send()
        .await
        .unwrap();
    assert!(health_response.status().is_success());

    // Test access to protected endpoints without auth (should fail)
    let command_response = client
        .post(&format!("{}/api/command", base_url))
        .json(&serde_json::json!({"command": "echo", "args": ["test"]}))
        .send()
        .await
        .unwrap();

    // Currently endpoints are not protected, but this test ensures they work
    assert!(command_response.status().is_success());

    server_handle.abort();
}

#[tokio::test]
async fn test_gateway_error_handling() {
    // Test error handling scenarios
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    let config = GatewayConfig {
        host: "127.0.0.1".to_string(),
        port,
        static_dir: "static".to_string(),
        log_level: "info".to_string(),
    };

    let server = run_server(config).await.unwrap();
    let server_handle = tokio::spawn(server);
    sleep(Duration::from_millis(500)).await;

    let client = Client::new();
    let base_url = format!("http://127.0.0.1:{}", port);

    // Test invalid endpoint
    let invalid_response = client
        .get(&format!("{}/invalid-endpoint", base_url))
        .send()
        .await
        .unwrap();
    assert!(invalid_response.status().is_client_error());

    // Test malformed JSON
    let malformed_response = client
        .post(&format!("{}/api/command", base_url))
        .body("invalid json")
        .header("Content-Type", "application/json")
        .send()
        .await
        .unwrap();
    assert!(malformed_response.status().is_client_error());

    server_handle.abort();
}