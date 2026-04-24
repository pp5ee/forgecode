use std::sync::Arc;
use actix_web::web::Data;
use forge_services::ForgeServices;
use crate::auth::TokenManager;

/// Integration layer between the gateway and forgecode services
pub struct ForgeIntegration<F>
where
    F: forge_services::forge_services::ForgeServicesInfrastructure,
{
    /// Core forge services
    pub forge_services: Arc<ForgeServices<F>>,
    /// Gateway's token manager
    pub token_manager: Arc<TokenManager>,
}

impl<F> ForgeIntegration<F>
where
    F: forge_services::forge_services::ForgeServicesInfrastructure,
{
    /// Create a new integration layer
    pub fn new(forge_services: Arc<ForgeServices<F>>, token_manager: Arc<TokenManager>) -> Self {
        Self {
            forge_services,
            token_manager,
        }
    }

    /// Get the shell service for command execution
    pub fn shell_service(&self) -> &forge_services::ForgeShell<F> {
        self.forge_services.shell_service()
    }

    /// Get the authentication service
    pub fn auth_service(&self) -> &forge_services::ForgeAuthService<F> {
        self.forge_services.auth_service()
    }

    /// Get the file discovery service
    pub fn file_discovery_service(&self) -> &forge_services::ForgeDiscoveryService<F> {
        self.forge_services.file_discovery_service()
    }

    /// Get the conversation service
    pub fn conversation_service(&self) -> &forge_services::ForgeConversationService<F> {
        self.forge_services.conversation_service()
    }

    /// Execute a command through the forgecode shell service
    pub async fn execute_command(
        &self,
        command: &str,
        working_directory: Option<&str>,
    ) -> Result<String, forge_services::error::ForgeError> {
        use forge_domain::shell::ShellCommand;

        let shell_cmd = ShellCommand {
            command: command.to_string(),
            working_directory: working_directory.map(|s| s.to_string()),
            env_vars: Default::default(),
        };

        let result = self.shell_service().execute(&shell_cmd).await?;
        Ok(result.output)
    }

    /// Validate gateway token against forgecode authentication
    pub async fn validate_token_against_forgecode(
        &self,
        token_id: &uuid::Uuid,
    ) -> Result<bool, forge_services::error::ForgeError> {
        // For now, we'll use the gateway's token manager
        // In the future, we can integrate with forgecode's auth service
        Ok(self.token_manager.validate_token_by_id(token_id).is_ok())
    }

    /// Get system information from forgecode services
    pub async fn get_system_info(&self) -> Result<SystemInfo, forge_services::error::ForgeError> {
        let config = self.forge_services.get_config()?;
        let environment = self.forge_services.get_environment();

        Ok(SystemInfo {
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: format!("{:?}", environment),
            config_hash: format!("{:x}", md5::compute(format!("{:?}", config))),
            active_tokens: self.token_manager.get_active_token_count(),
            uptime: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        })
    }
}

/// System information structure
#[derive(serde::Serialize)]
pub struct SystemInfo {
    pub version: String,
    pub environment: String,
    pub config_hash: String,
    pub active_tokens: usize,
    pub uptime: u64,
}

/// Actix-web data wrapper for forge integration
pub type ForgeIntegrationData<F> = Data<ForgeIntegration<F>>;

/// Helper function to create forge integration data
pub fn create_forge_integration_data<F>(
    forge_services: Arc<ForgeServices<F>>,
    token_manager: Arc<TokenManager>,
) -> ForgeIntegrationData<F>
where
    F: forge_services::forge_services::ForgeServicesInfrastructure,
{
    Data::new(ForgeIntegration::new(forge_services, token_manager))
}