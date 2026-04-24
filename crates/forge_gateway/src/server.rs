//! HTTP server implementation for the gateway

use actix_cors::Cors;
use actix_web::{web, App, HttpServer};
use tracing::info;

use crate::{GatewayConfig, handlers, websocket, Result};

/// Start the HTTP server with the given configuration
pub async fn run_server(config: GatewayConfig) -> Result<()> {
    let bind_addr = format!("{}:{}", config.bind_address, config.port);

    info!("Starting HTTP server on {}", bind_addr);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(tracing_actix_web::TracingLogger::default())
            // Health check endpoint
            .route("/health", web::get().to(handlers::health_check))
            // Authentication endpoint
            .route("/auth/validate", web::post().to(handlers::validate_token))
            // Token renewal endpoint
            .route("/auth/renew", web::post().to(handlers::renew_token))
            // WebSocket endpoint for real-time terminal
            .route("/ws", web::get().to(websocket::websocket_handler))
    })
    .bind(&bind_addr)
    .map_err(|e| crate::GatewayError::Http(e.to_string()))?
    .run()
    .await
    .map_err(|e| crate::GatewayError::Http(e.to_string()))?;

    Ok(())
}