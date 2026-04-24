//! HTTP server implementation for the gateway

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{web, App, HttpServer};
use std::sync::{Arc, Mutex};
use tracing::info;

use crate::{GatewayConfig, handlers, websocket, auth, integration, Result};

/// Start the HTTP server with the given configuration
pub async fn run_server(config: GatewayConfig) -> Result<()> {
    let bind_addr = format!("{}:{}", config.bind_address, config.port);

    info!("Starting HTTP server on {}", bind_addr);

    // Create token manager
    let token_manager = web::Data::new(Mutex::new(auth::TokenManager::new()));

    // Initialize forgecode service
    info!("Initializing forgecode service...");
    if let Err(e) = integration::initialize_forge_service().await {
        info!("Forgecode service initialization failed: {}, continuing without full integration", e);
    } else {
        info!("Forgecode service initialized successfully");
    }

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        // Create the token middleware
        let token_middleware = auth::UrlTokenMiddleware::new(token_manager.clone());

        App::new()
            .app_data(token_manager.clone())
            .wrap(cors)
            .wrap(token_middleware)
            .wrap(tracing_actix_web::TracingLogger::default())
            // Health check endpoint (public - exempt from auth)
            .route("/health", web::get().to(handlers::health_check))
            // Authentication endpoints (public - exempt from auth)
            .route("/auth/token/validate", web::post().to(handlers::validate_token))
            .route("/auth/token/renew", web::post().to(handlers::renew_token))
            .route("/auth/token/generate", web::post().to(handlers::generate_token))
            // WebSocket endpoint for real-time terminal (protected)
            .route("/ws", web::get().to(|req, stream, token_manager: web::Data<Mutex<auth::TokenManager>>| {
                websocket::websocket_handler(req, stream, token_manager)
            }))
            // Forgecode integration endpoints (protected)
            .route("/api/command", web::post().to(integration::execute_command_handler))
            .route("/api/file/read", web::post().to(integration::read_file_handler))
            .route("/api/system/info", web::get().to(integration::system_info_handler))
            // Static file serving for web UI (public access, auth handled in UI)
            .service(Files::new("/static", "static/").index_file("index.html"))
            // Root path redirects to static index with proper token handling
            .route("/", web::get().to(handlers::serve_index))
    })
    .bind(&bind_addr)
    .map_err(|e| crate::GatewayError::Http(e.to_string()))?
    .run()
    .await
    .map_err(|e| crate::GatewayError::Http(e.to_string()))?;

    Ok(())
}