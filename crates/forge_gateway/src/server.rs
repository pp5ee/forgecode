//! HTTP server implementation for the web gateway

use crate::handlers;
use crate::auth::TokenAuth;
use crate::websocket::WebSocketHandler;
use forge_api::API;
use forge_config::ForgeConfig;
use actix_web::{web, App, HttpServer};
use std::sync::Arc;

/// HTTP server for the web gateway
pub struct GatewayServer {
    api: Arc<API>,
    config: ForgeConfig,
}

impl GatewayServer {
    /// Create a new gateway server
    pub fn new(api: Arc<API>, config: ForgeConfig) -> Self {
        Self { api, config }
    }

    /// Start the HTTP server
    pub async fn start(&self) -> Result<(), anyhow::Error> {
        let api = self.api.clone();
        let config = self.config.clone();

        let port = config.gateway_port.unwrap_or(8080);
        let bind_address = format!("0.0.0.0:{}", port);

        log::info!("Starting gateway server on {}", bind_address);

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(api.clone()))
                .app_data(web::Data::new(config.clone()))
                .service(
                    web::scope("/api")
                        .wrap(TokenAuth::new())
                        .route("/health", web::get().to(handlers::health))
                        .route("/conversation", web::post().to(handlers::conversation))
                        .route("/files", web::get().to(handlers::list_files))
                        .route("/files/{path:.*}", web::get().to(handlers::get_file))
                        .route("/files/{path:.*}", web::put().to(handlers::update_file))
                        .route("/execute", web::post().to(handlers::execute_command))
                )
                .service(
                    web::scope("/ws")
                        .wrap(TokenAuth::new())
                        .route("/conversation", web::get().to(WebSocketHandler::conversation))
                        .route("/command", web::get().to(WebSocketHandler::command))
                )
                .service(
                    web::scope("/")
                        .route("", web::get().to(handlers::serve_ui))
                        .route("/{path:.*}", web::get().to(handlers::serve_ui))
                )
        })
        .bind(&bind_address)?
        .run()
        .await?;

        Ok(())
    }

    /// Stop the HTTP server
    pub async fn stop(&self) -> Result<(), anyhow::Error> {
        // Implementation for graceful shutdown
        Ok(())
    }
}