use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use forge_api::ForgeAPI;
use forge_gateway::handlers::{
    auth_handler, command_execute_handler, command_list_handler, file_list_handler,
    token_validate_handler,
};
use forge_gateway::server::Server;
use forge_gateway::ForgeCodeClient;
use forge_gateway::WebSocketHandler;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // Initialize the forgecode API
    let api = Arc::new(forge_api::ForgeAPI::new()?);

    // Initialize the forgecode client with the actual API
    let forge_client = Arc::new(ForgeCodeClient::new(api));

    // Initialize WebSocket handler
    let ws_handler = Arc::new(WebSocketHandler::new(forge_client.clone()));

    // Build our application with a route
    let app = Router::new()
        .route("/auth", get(auth_handler))
        .route("/validate", get(token_validate_handler))
        .route("/files", get(file_list_handler))
        .route("/command/execute", post(command_execute_handler))
        .route("/command/list", get(command_list_handler))
        .route("/ws", get(WebSocketHandler::handle_websocket))
        .with_state(ws_handler.clone());

    // Start server
    let server = Server::new("0.0.0.0:8080".parse().unwrap());
    server.run(app).await?;

    Ok(())
}