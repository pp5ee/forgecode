mod auth;
mod forgecode_client;
mod handlers;
mod server;

use crate::auth::TokenManager;
use crate::forgecode_client::ForgeCodeClient;
use crate::server::create_routes;
use std::env;
use warp::Filter;

#[tokio::main]
async fn main() {
    // Initialize logging
    env_logger::init();

    // Get configuration from environment variables
    let forgecode_base_url = env::var("FORGECODE_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8081".to_string());

    let gateway_port = env::var("GATEWAY_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("Invalid GATEWAY_PORT");

    println!("Starting forge_gateway on port {}", gateway_port);
    println!("Connecting to forgecode service at: {}", forgecode_base_url);

    // Initialize components
    let token_manager = TokenManager::new("http://localhost:8081".to_string());
    let forgecode_client = ForgeCodeClient::new(forgecode_base_url);

    // Check forgecode service health
    match forgecode_client.health_check().await {
        Ok(true) => println!("Forgecode service is healthy"),
        Ok(false) => println!("Warning: Forgecode service is not responding"),
        Err(e) => println!("Warning: Cannot connect to forgecode service: {}", e),
    }

    // Create routes
    let routes = create_routes(token_manager, forgecode_client);

    // Start server
    warp::serve(routes)
        .run(([0, 0, 0, 0], gateway_port))
        .await;
}