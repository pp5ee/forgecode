//! HTTP server implementation for the ForgeCode gateway

use std::path::PathBuf;
use std::sync::Arc;

use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use forge_api::API;
use forge_config::ForgeConfig;
use forge_domain::UrlTokenRepository;
use forge_services::UrlTokenService;

use crate::handlers::{chat_stream, get_file, health_check, list_files, update_file};

/// Configuration for the gateway server
#[derive(Clone, Debug)]
pub struct ServerConfig {
    /// Host to bind the server to
    host: String,
    /// Port to listen on
    port: u16,
    /// Base directory for token storage
    token_storage_base: PathBuf,
}

impl ServerConfig {
    /// Create a new server configuration
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            token_storage_base: PathBuf::from(".forge"),
        }
    }

    /// Get the server bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Get the token storage path
    pub fn token_storage_path(&self) -> PathBuf {
        self.token_storage_base.join("tokens")
    }

    /// Set the token storage base directory
    pub fn with_token_storage_base(mut self, base: PathBuf) -> Self {
        self.token_storage_base = base;
        self
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 8080,
            token_storage_base: PathBuf::from(".forge"),
        }
    }
}

/// The gateway HTTP server
pub struct GatewayServer<R: UrlTokenRepository> {
    config: ServerConfig,
    api: Arc<API>,
    token_service: Arc<UrlTokenService<R>>,
    server_handle: Option<actix_web::dev::ServerHandle>,
}

impl<R: UrlTokenRepository + Send + Sync + 'static> GatewayServer<R> {
    /// Create a new gateway server
    pub fn new(
        api: Arc<API>,
        config: ForgeConfig,
        token_service: Arc<UrlTokenService<R>>,
    ) -> Self {
        let server_config = ServerConfig::default();

        Self {
            config: server_config,
            api,
            token_service,
            server_handle: None,
        }
    }

    /// Get the server configuration
    pub fn config(&self) -> &ServerConfig {
        &self.config
    }

    /// Start the gateway server
    pub async fn start(&mut self) -> anyhow::Result<()> {
        let api = self.api.clone();
        let token_service = self.token_service.clone();
        let config = self.config.clone();

        log::info!(
            "Starting server on {}:{}",
            self.config.host,
            self.config.port
        );

        let server = HttpServer::new(move || {
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .supports_credentials();

            App::new()
                .wrap(middleware::Logger::default())
                .wrap(cors)
                .app_data(web::Data::new(api.clone()))
                .app_data(web::Data::new(token_service.clone()))
                .service(
                    web::resource("/health")
                        .route(web::get().to(health_check)),
                )
                .service(
                    web::resource("/api/files")
                        .route(web::get().to(list_files)),
                )
                .service(
                    web::resource("/api/files/{path:.*}")
                        .route(web::get().to(get_file))
                        .route(web::post().to(update_file)),
                )
                .service(
                    web::resource("/api/chat")
                        .route(web::post().to(chat_stream)),
                )
        })
        .bind(&config.bind_address())?;

        let handle = server.run();
        self.server_handle = Some(handle);

        log::info!(
            "Gateway server listening on {}",
            config.bind_address()
        );

        Ok(())
    }

    /// Stop the gateway server
    pub async fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(handle) = self.server_handle.take() {
            handle.stop(true).await;
            log::info!("Server stopped successfully");
        }
        Ok(())
    }
}
