//! Main gateway module that orchestrates the web gateway functionality

use std::sync::Arc;

use forge_api::API;
use forge_config::ForgeConfig;
use forge_infra::FsUrlTokenRepository;
use forge_services::UrlTokenService;

use crate::server::{GatewayServer, ServerConfig};

/// Main gateway struct that manages the web gateway service
pub struct Gateway {
    server: Option<GatewayServer<FsUrlTokenRepository>>,
    api: Arc<API>,
    config: ForgeConfig,
    token_service: Arc<UrlTokenService<FsUrlTokenRepository>>,
}

impl Gateway {
    /// Create a new gateway instance
    pub async fn new(api: Arc<API>, config: ForgeConfig) -> anyhow::Result<Self> {
        // Initialize token repository with default storage path
        let storage_path = ServerConfig::default().token_storage_path();
        let repository = FsUrlTokenRepository::new(storage_path).await?;
        let token_service = Arc::new(UrlTokenService::new(Arc::new(repository)));

        Ok(Self {
            server: None,
            api,
            config,
            token_service,
        })
    }

    /// Start the gateway server
    pub async fn start(&mut self) -> anyhow::Result<()> {
        if self.server.is_some() {
            return Err(anyhow::anyhow!("Server is already running"));
        }

        let mut server = GatewayServer::new(
            self.api.clone(),
            self.config.clone(),
            self.token_service.clone(),
        );

        log::info!("Starting ForgeCode gateway...");
        log::info!("Server configuration: {:?}", server.config());

        server.start().await?;
        self.server = Some(server);

        log::info!("ForgeCode gateway started successfully");
        Ok(())
    }

    /// Stop the gateway server
    pub async fn stop(&mut self) -> anyhow::Result<()> {
        if let Some(mut server) = self.server.take() {
            log::info!("Stopping ForgeCode gateway...");
            server.stop().await?;
            log::info!("ForgeCode gateway stopped successfully");
        }
        Ok(())
    }

    /// Generate a new authentication token
    pub async fn generate_token(&self, duration_hours: i64, description: Option<String>) -> anyhow::Result<String> {
        use forge_domain::CreateUrlTokenRequest;

        let request = CreateUrlTokenRequest::default()
            .duration_hours(duration_hours)
            .description(description.unwrap_or_default());

        let response = self.token_service.create_token(request).await?;
        Ok(response.token)
    }

    /// Validate an authentication token
    pub async fn validate_token(&self, token: &str) -> bool {
        matches!(
            self.token_service.validate_token(token).await,
            forge_domain::TokenValidationResult::Valid(_)
        )
    }

    /// Check if the server is running
    pub fn is_running(&self) -> bool {
        self.server.is_some()
    }

    /// Get server configuration
    pub fn server_config(&self) -> Option<ServerConfig> {
        self.server.as_ref().map(|s| s.config().clone())
    }
}
