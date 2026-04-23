//! Main gateway module that orchestrates the web gateway functionality

use crate::server::GatewayServer;
use crate::auth::TokenManager;
use forge_api::API;
use forge_config::ForgeConfig;
use std::sync::Arc;

/// Main gateway struct that manages the web gateway service
pub struct Gateway {
    server: GatewayServer,
    token_manager: TokenManager,
    api: Arc<API>,
    config: ForgeConfig,
}

impl Gateway {
    /// Create a new gateway instance
    pub fn new(api: Arc<API>, config: ForgeConfig) -> Self {
        let token_manager = TokenManager::new();
        let server = GatewayServer::new(api.clone(), config.clone());

        Self {
            server,
            token_manager,
            api,
            config,
        }
    }

    /// Start the gateway server
    pub async fn start(&self) -> Result<(), anyhow::Error> {
        self.server.start().await
    }

    /// Stop the gateway server
    pub async fn stop(&self) -> Result<(), anyhow::Error> {
        self.server.stop().await
    }

    /// Generate a new authentication token
    pub fn generate_token(&self) -> String {
        self.token_manager.generate_token()
    }

    /// Validate an authentication token
    pub fn validate_token(&self, token: &str) -> bool {
        self.token_manager.validate_token(token)
    }
}