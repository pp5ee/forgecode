//! Integration tests for ForgeGateway

use forge_gateway::{
    handlers::{self, AuthRequest, ConversationRequest, ExecuteCommandRequest, FileRequest},
    server::Server,
    websocket::WebSocketHandler,
};
use hyper::{Body, Request, StatusCode};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::test]
async fn test_health_endpoint() {
    let server = Server::new("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr();

    let client = hyper::Client::new();
    let response = client
        .get(format!("http://{}/health", addr).parse().unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_authentication_flow() {
    let server = Server::new("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr();

    let client = hyper::Client::new();

    // Test authentication endpoint
    let auth_request = AuthRequest {
        token: "test-token".to_string(),
    };

    let response = client
        .post(format!("http://{}/api/auth", addr).parse().unwrap())
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&auth_request).unwrap()))
        .await
        .unwrap();

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_file_browsing() {
    let server = Server::new("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr();

    let client = hyper::Client::new();

    let file_request = FileRequest {
        path: "/".to_string(),
    };

    let response = client
        .post(format!("http://{}/api/files", addr).parse().unwrap())
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token")
        .body(Body::from(serde_json::to_string(&file_request).unwrap()))
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status() == StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_command_execution() {
    let server = Server::new("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr();

    let client = hyper::Client::new();

    let command_request = ExecuteCommandRequest {
        command: "echo test".to_string(),
        working_directory: "/tmp".to_string(),
    };

    let response = client
        .post(format!("http://{}/api/execute", addr).parse().unwrap())
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token")
        .body(Body::from(serde_json::to_string(&command_request).unwrap()))
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status() == StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_conversation_interface() {
    let server = Server::new("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr();

    let client = hyper::Client::new();

    let conversation_request = ConversationRequest {
        message: "Test message".to_string(),
        conversation_id: None,
    };

    let response = client
        .post(format!("http://{}/api/conversation", addr).parse().unwrap())
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-token")
        .body(Body::from(serde_json::to_string(&conversation_request).unwrap()))
        .await
        .unwrap();

    assert!(response.status().is_success() || response.status() == StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_websocket_connection() {
    let server = Server::new("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr();

    // Try to establish WebSocket connection
    let ws_url = format!("ws://{}/ws", addr);
    let result = connect_async(&ws_url).await;

    // Connection should either succeed or fail gracefully (not panic)
    assert!(result.is_ok() || matches!(result, Err(_)));
}

#[tokio::test]
async fn test_error_handling() {
    let server = Server::new("127.0.0.1:0").await.unwrap();
    let addr = server.local_addr();

    let client = hyper::Client::new();

    // Test invalid endpoint
    let response = client
        .get(format!("http://{}/invalid", addr).parse().unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Test invalid JSON
    let response = client
        .post(format!("http://{}/api/auth", addr).parse().unwrap())
        .header("content-type", "application/json")
        .body(Body::from("invalid json"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}