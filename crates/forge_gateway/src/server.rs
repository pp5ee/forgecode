//! HTTP server implementation for the gateway

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{web, App, HttpServer};
use std::sync::Mutex;
use tracing::info;

use crate::{GatewayConfig, handlers, websocket, auth, integration, Result};

/// Start the HTTP server with the given configuration
pub async fn run_server(config: GatewayConfig) -> Result<()> {
    let bind_addr = format!("{}:{}", config.bind_address, config.port);

    info!("Starting HTTP server on {}", bind_addr);

    // Create token manager with configuration
    let token_manager = web::Data::new(Mutex::new(auth::TokenManager::new(
        config.token_expiration_seconds,
    )));

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .app_data(token_manager.clone())
            .wrap(cors)
            .wrap(tracing_actix_web::TracingLogger::default())
            // Health check endpoint
            .route("/health", web::get().to(handlers::health_check))
            // Authentication endpoints
            .route("/auth/validate", web::post().to(handlers::validate_token))
            .route("/auth/renew", web::post().to(handlers::renew_token))
            .route("/auth/generate", web::post().to(handlers::generate_token))
            // WebSocket endpoint for real-time terminal
            .route("/ws", web::get().to(websocket::websocket_handler))
            // Forgecode integration endpoints
            .route("/api/command", web::post().to(integration::execute_command_handler::<dyn forge_services::Services>))
            .route("/api/file/read", web::post().to(integration::read_file_handler::<dyn forge_services::Services>))
            .route("/api/system/info", web::get().to(integration::system_info_handler::<dyn forge_services::Services>))
            // Static file serving for web UI
            .service(Files::new("/", "static/").index_file("index.html"))
    })
    .bind(&bind_addr)
    .map_err(|e| crate::GatewayError::Http(e.to_string()))?
    .run()
    .await
    .map_err(|e| crate::GatewayError::Http(e.to_string()))?;

    Ok(())
}