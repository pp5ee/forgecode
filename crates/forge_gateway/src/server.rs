//! HTTP server implementation for the web gateway

use std::sync::Arc;
use std::time::Duration;

use actix_cors::Cors;
use actix_web::{middleware::Logger, web, App, HttpServer};
use forge_api::API;
use forge_config::ForgeConfig;
use forge_infra::UrlTokenRepository;
use forge_services::UrlTokenService;

use crate::auth::{PublicWithOptionalAuth, UrlTokenAuth};
use crate::handlers;
use crate::monitoring::MonitoringService;
use crate::websocket::WebSocketHandler;

/// Configuration for the gateway server
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub bind_address: String,
    pub workers: usize,
    pub client_timeout: Duration,
    pub client_shutdown: Duration,
    pub token_storage_path: Option<std::path::PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            bind_address: "0.0.0.0".to_string(),
            workers: num_cpus::get(),
            client_timeout: Duration::from_secs(60),
            client_shutdown: Duration::from_secs(30),
            token_storage_path: None,
        }
    }
}

impl ServerConfig {
    /// Create server config from environment variables and forge config
    pub fn from_env_and_config(config: &ForgeConfig) -> Self {
        let port = std::env::var("FORGE_GATEWAY_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .or_else(|| config.gateway_port)
            .unwrap_or(8080);

        let bind_address = std::env::var("FORGE_GATEWAY_BIND")
            .unwrap_or_else(|_| "0.0.0.0".to_string());

        let workers = std::env::var("FORGE_GATEWAY_WORKERS")
            .ok()
            .and_then(|w| w.parse().ok())
            .unwrap_or_else(num_cpus::get);

        let token_storage_path = std::env::var("FORGE_GATEWAY_TOKEN_STORAGE")
            .ok()
            .map(std::path::PathBuf::from);

        Self {
            port,
            bind_address,
            workers,
            token_storage_path,
            ..Default::default()
        }
    }

    /// Get the full bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.bind_address, self.port)
    }

    /// Get the token storage path with default fallback
    pub fn token_storage_path(&self) -> std::path::PathBuf {
        self.token_storage_path
            .clone()
            .unwrap_or_else(|| {
                let cwd = std::env::current_dir().unwrap_or_default();
                cwd.join(".forge").join("gateway_tokens.json")
            })
    }
}

/// HTTP server for the web gateway
pub struct GatewayServer<R: UrlTokenRepository> {
    api: Arc<API>,
    config: ForgeConfig,
    server_config: ServerConfig,
    token_service: Arc<UrlTokenService<R>>,
    monitoring_service: Arc<MonitoringService>,
    shutdown_signal: Option<tokio::sync::oneshot::Sender<()>>,
}

impl<R: UrlTokenRepository + 'static> GatewayServer<R> {
    /// Create a new gateway server
    pub fn new(
        api: Arc<API>,
        config: ForgeConfig,
        token_service: Arc<UrlTokenService<R>>,
    ) -> Self {
        let server_config = ServerConfig::from_env_and_config(&config);
        let monitoring_service = Arc::new(MonitoringService::new());

        Self {
            api,
            config,
            server_config,
            token_service,
            monitoring_service,
            shutdown_signal: None,
        }
    }

    /// Start the HTTP server
    pub async fn start(&mut self) -> Result<(), anyhow::Error> {
        let api = self.api.clone();
        let config = self.config.clone();
        let server_config = self.server_config.clone();
        let token_service = self.token_service.clone();
        let monitoring_service = self.monitoring_service.clone();

        let bind_address = server_config.bind_address();

        // Initialize monitoring and logging
        crate::monitoring::init_logging();
        monitoring_service.log_startup();

        log::info!("Starting gateway server on {}", bind_address);
        log::info!("Server configuration: {:?}", server_config);

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        self.shutdown_signal = Some(shutdown_tx);

        let server = HttpServer::new(move || {
            // Configure CORS
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600);

            App::new()
                // Add logging middleware
                .wrap(Logger::default())
                .wrap(cors)
                .app_data(web::Data::new(api.clone()))
                .app_data(web::Data::new(config.clone()))
                .app_data(web::Data::from(token_service.clone()))
                .app_data(web::Data::new(monitoring_service.clone()))
                // Public health endpoint (no auth required)
                .service(web::resource("/api/health").route(web::get().to(handlers::health)))
                // Public metrics endpoint (no auth required)
                .service(web::resource("/api/metrics").route(web::get().to(handlers::metrics)))
                // Token management endpoints (public with optional auth)
                .service(
                    web::scope("/api/tokens")
                        .wrap(PublicWithOptionalAuth::new(token_service.clone()))
                        .route("", web::post().to(handlers::create_token))
                        .route("", web::get().to(handlers::list_tokens))
                        .route("/{id}", web::delete().to(handlers::revoke_token)),
                )
                // Protected API routes (require valid token)
                .service(
                    web::scope("/api")
                        .wrap(UrlTokenAuth::new(token_service.clone()))
                        .route("/conversation", web::post().to(handlers::conversation))
                        .route("/files", web::get().to(handlers::list_files))
                        .route("/files/{path:.*}", web::get().to(handlers::get_file))
                        .route("/files/{path:.*}", web::put().to(handlers::update_file))
                        .route("/execute", web::post().to(handlers::execute_command)),
                )
                // Protected WebSocket routes (require valid token)
                .service(
                    web::scope("/ws")
                        .wrap(UrlTokenAuth::new(token_service.clone()))
                        .route(
                            "/conversation",
                            web::get().to(WebSocketHandler::conversation),
                        )
                        .route("/command", web::get().to(WebSocketHandler::command)),
                )
                // Public UI routes (optional auth)
                .service(
                    web::scope("/")
                        .wrap(PublicWithOptionalAuth::new(token_service.clone()))
                        .route("", web::get().to(handlers::serve_ui))
                        .route("/{path:.*}", web::get().to(handlers::serve_ui)),
                )
        })
        .workers(server_config.workers)
        .client_request_timeout(server_config.client_timeout)
        .shutdown_timeout(server_config.client_shutdown.as_secs())
        .bind(&bind_address)?;

        // Start server with graceful shutdown handling
        let server = server.run();

        let graceful = server.handle();
        let monitoring_service_shutdown = monitoring_service.clone();
        tokio::spawn(async move {
            let _ = shutdown_rx.await;
            graceful.stop(true).await;
        });

        server.await?;

        monitoring_service_shutdown.log_shutdown();
        log::info!("Gateway server stopped");
        Ok(())
    }

    /// Stop the HTTP server gracefully
    pub async fn stop(&mut self) -> Result<(), anyhow::Error> {
        if let Some(shutdown_tx) = self.shutdown_signal.take() {
            log::info!("Stopping gateway server gracefully...");
            let _ = shutdown_tx.send(());
            // Give the server time to shut down
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        Ok(())
    }

    /// Get server configuration
    pub fn config(&self) -> &ServerConfig {
        &self.server_config
    }
}
