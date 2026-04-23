//! Main gateway module that orchestrates the web gateway functionality

use crate::server::{GatewayServer, ServerConfig};
use crate::auth::TokenManager;
use forge_api::API;
use forge_config::ForgeConfig;
use std::sync::Arc;

/// Main gateway struct that manages the web gateway service
pub struct Gateway {
    server: Option<GatewayServer>,
    token_manager: TokenManager,
    api: Arc<API>,
    config: ForgeConfig,
}

impl Gateway {
    /// Create a new gateway instance
    pub fn new(api: Arc<API>, config: ForgeConfig) -> Self {
        let token_manager = TokenManager::new();

        Self {
            server: None,
            token_manager,
            api,
            config,
        }
    }

    /// Start the gateway server
    pub async fn start(&mut self) -> Result<(), anyhow::Error> {
        if self.server.is_some() {
            return Err(anyhow::anyhow!("Server is already running"));
        }

        let mut server = GatewayServer::new(self.api.clone(), self.config.clone());

        log::info!("Starting ForgeCode gateway...");
        log::info!("Server configuration: {:?}", server.config());

        server.start().await?;
        self.server = Some(server);

        log::info!("ForgeCode gateway started successfully");
        Ok(())
    }

    /// Stop the gateway server
    pub async fn stop(&mut self) -> Result<(), anyhow::Error> {
        if let Some(mut server) = self.server.take() {
            log::info!("Stopping ForgeCode gateway...");
            server.stop().await?;
            log::info!("ForgeCode gateway stopped successfully");
        }
        Ok(())
    }

    /// Generate a new authentication token
    pub fn generate_token(&self) -> String {
        self.token_manager.generate_token()
    }

    /// Validate an authentication token
    pub fn validate_token(&self, token: &str) -> bool {
        self.token_manager.validate_token(token)
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