use actix_web::{web, App, HttpServer, middleware::Logger};
use actix_web_httpauth::middleware::HttpAuthentication;
use std::env;

mod handlers;
mod auth;
mod forgecode_client;
mod middleware;

use crate::auth::TokenManager;
use crate::forgecode_client::ForgeCodeClient;
use crate::middleware::url_token_auth;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
    let token_manager = TokenManager::new();
    let forgecode_client = ForgeCodeClient::new(forgecode_base_url);

    // Check forgecode service health
    match forgecode_client.health_check().await {
        Ok(true) => println!("Forgecode service is healthy"),
        Ok(false) => println!("Warning: Forgecode service is not responding"),
        Err(e) => println!("Warning: Cannot connect to forgecode service: {}", e),
    }

    HttpServer::new(move || {
        let auth = HttpAuthentication::with_fn(url_token_auth);

        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(token_manager.clone()))
            .app_data(web::Data::new(forgecode_client.clone()))
            .service(
                web::scope("/api")
                    .service(handlers::generate_token)
                    .service(handlers::renew_token)
                    .wrap(auth.clone())
                    .service(handlers::execute_command)
            )
            .service(
                actix_files::Files::new("/", "static")
                    .index_file("index.html")
                    .use_last_modified(true)
            )
    })
    .bind(("0.0.0.0", gateway_port))?
    .run()
    .await
}