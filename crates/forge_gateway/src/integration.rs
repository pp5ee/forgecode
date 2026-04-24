//! Forgecode service integration for the gateway

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

use forge_api::ForgeAPI;
use forge_config::ForgeConfig;
use forge_domain::TitleFormat;

/// Forgecode service client wrapper
pub struct ForgeService {
    api: Arc<Mutex<Option<ForgeAPI>>>,
}

impl ForgeService {
    pub fn new() -> Self {
        Self {
            api: Arc::new(Mutex::new(None)),
        }
    }

    /// Initialize the forgecode API with configuration
    pub async fn initialize(&self) -> Result<(), String> {
        let mut api_guard = self.api.lock().await;

        if api_guard.is_some() {
            return Ok(()); // Already initialized
        }

        match ForgeConfig::read() {
            Ok(config) => {
                let cwd = std::env::current_dir()
                    .map_err(|e| format!("Failed to get current directory: {}", e))?;

                match ForgeAPI::init(cwd, config) {
                    Ok(api) => {
                        *api_guard = Some(api);
                        Ok(())
                    }
                    Err(e) => Err(format!("Failed to initialize ForgeAPI: {}", e)),
                }
            }
            Err(e) => Err(format!("Failed to read ForgeConfig: {}", e)),
        }
    }

    /// Execute a forgecode command
    pub async fn execute_command(&self, command: &str) -> Result<String, String> {
        let api_guard = self.api.lock().await;

        match api_guard.as_ref() {
            Some(api) => {
                // Use real forgecode API methods based on command type
                if command.starts_with("help") || command.trim().is_empty() {
                    Ok(self.get_help_text().await)
                } else if command.starts_with("status") {
                    self.get_system_status(api).await
                } else if command.starts_with("version") {
                    Ok(format!("Forgecode Gateway v{}", env!("CARGO_PKG_VERSION")))
                } else if command.starts_with("config") {
                    self.get_config_info(api).await
                } else if command.starts_with("agents") {
                    self.list_agents(api).await
                } else if command.starts_with("tools") {
                    self.list_tools(api).await
                } else if command.starts_with("models") {
                    self.list_models(api).await
                } else if command.starts_with("conversations") {
                    self.list_conversations(api).await
                } else if command.starts_with("workspaces") {
                    self.list_workspaces(api).await
                } else if command.starts_with("discover") {
                    self.discover_files(api).await
                } else {
                    // Try to execute as a shell command
                    self.execute_shell_command(api, command).await
                }
            }
            None => Err("Forgecode API not initialized".to_string()),
        }
    }

    /// Read a file using forgecode services
    pub async fn read_file(&self, path: &str) -> Result<String, String> {
        let file_path = PathBuf::from(path);

        if !file_path.exists() {
            return Err(format!("File not found: {}", path));
        }

        match std::fs::read_to_string(&file_path) {
            Ok(content) => Ok(content),
            Err(e) => Err(format!("Failed to read file {}: {}", path, e)),
        }
    }

    // Helper methods for real forgecode integration

    async fn get_help_text(&self) -> String {
        r#"Available ForgeCode Gateway Commands:

Basic Commands:
  help              - Show this help message
  status            - Show system status and services
  version           - Show gateway version
  config            - Show configuration information

ForgeCode Services:
  agents            - List available agents
  tools             - List available tools
  models            - List available models
  conversations     - List recent conversations
  workspaces        - List available workspaces

System Commands:
  <shell command>   - Execute shell commands in the workspace

Use 'help <command>' for more information on a specific command."#.to_string()
    }

    async fn get_system_status(&self, api: &ForgeAPI) -> Result<String, String> {
        match api.get_agents().await {
            Ok(agents) => {
                let agent_count = agents.len();
                match api.get_tools().await {
                    Ok(tools) => {
                        let tool_count = tools.tools.len();
                        match api.get_models().await {
                            Ok(models) => {
                                let model_count = models.len();
                                Ok(format!(
                                    "ForgeCode Gateway Status:\n\n".to_string() +
                                    "Services: RUNNING\n" +
                                    "Agents: {} available\n" +
                                    "Tools: {} available\n" +
                                    "Models: {} available\n" +
                                    "\nAll systems operational.",
                                    agent_count, tool_count, model_count
                                ))
                            }
                            Err(e) => Err(format!("Failed to get models: {}", e)),
                        }
                    }
                    Err(e) => Err(format!("Failed to get tools: {}", e)),
                }
            }
            Err(e) => Err(format!("Failed to get agents: {}", e)),
        }
    }

    async fn get_config_info(&self, api: &ForgeAPI) -> Result<String, String> {
        match api.get_session_config().await {
            Some(config) => {
                Ok(format!(
                    "Current Session Configuration:\n\n".to_string() +
                    "Provider: {}\n" +
                    "Model: {}\n" +
                    "\nConfiguration loaded successfully.",
                    config.provider, config.model
                ))
            }
            None => Ok("No active session configuration found.".to_string()),
        }
    }

    async fn list_agents(&self, api: &ForgeAPI) -> Result<String, String> {
        match api.get_agents().await {
            Ok(agents) => {
                if agents.is_empty() {
                    Ok("No agents available.".to_string())
                } else {
                    let mut output = "Available Agents:\n\n".to_string();
                    for agent in agents {
                        output.push_str(&format!("- {}: {}\n", agent.id, agent.name));
                    }
                    Ok(output)
                }
            }
            Err(e) => Err(format!("Failed to list agents: {}", e)),
        }
    }

    async fn list_tools(&self, api: &ForgeAPI) -> Result<String, String> {
        match api.get_tools().await {
            Ok(tools) => {
                if tools.tools.is_empty() {
                    Ok("No tools available.".to_string())
                } else {
                    let mut output = "Available Tools:\n\n".to_string();
                    for tool in tools.tools {
                        output.push_str(&format!("- {}: {}\n", tool.name, tool.description));
                    }
                    Ok(output)
                }
            }
            Err(e) => Err(format!("Failed to list tools: {}", e)),
        }
    }

    async fn list_models(&self, api: &ForgeAPI) -> Result<String, String> {
        match api.get_models().await {
            Ok(models) => {
                if models.is_empty() {
                    Ok("No models available.".to_string())
                } else {
                    let mut output = "Available Models:\n\n".to_string();
                    for model in models {
                        output.push_str(&format!("- {}\n", model.id));
                    }
                    Ok(output)
                }
            }
            Err(e) => Err(format!("Failed to list models: {}", e)),
        }
    }

    async fn list_conversations(&self, api: &ForgeAPI) -> Result<String, String> {
        match api.get_conversations(Some(10)).await {
            Ok(conversations) => {
                if conversations.is_empty() {
                    Ok("No recent conversations found.".to_string())
                } else {
                    let mut output = "Recent Conversations:\n\n".to_string();
                    for conv in conversations {
                        let title = conv.title.unwrap_or_else(|| "Untitled".to_string());
                        output.push_str(&format!("- {}: {}\n", conv.id, title));
                    }
                    Ok(output)
                }
            }
            Err(e) => Err(format!("Failed to list conversations: {}", e)),
        }
    }

    async fn list_workspaces(&self, api: &ForgeAPI) -> Result<String, String> {
        match api.list_workspaces().await {
            Ok(workspaces) => {
                if workspaces.is_empty() {
                    Ok("No workspaces available.".to_string())
                } else {
                    let mut output = "Available Workspaces:\n\n".to_string();
                    for workspace in workspaces {
                        output.push_str(&format!("- {}: {}\n", workspace.id, workspace.name));
                    }
                    Ok(output)
                }
            }
            Err(e) => Err(format!("Failed to list workspaces: {}", e)),
        }
    }

    async fn discover_files(&self, api: &ForgeAPI) -> Result<String, String> {
        match api.discover().await {
            Ok(files) => {
                if files.is_empty() {
                    Ok("No files discovered.".to_string())
                } else {
                    let mut output = "Discovered Files:\n\n".to_string();
                    for file in files.iter().take(20) { // Limit to first 20 files
                        output.push_str(&format!("- {}\n", file.name));
                    }
                    if files.len() > 20 {
                        output.push_str(&format!("... and {} more files\n", files.len() - 20));
                    }
                    Ok(output)
                }
            }
            Err(e) => Err(format!("Failed to discover files: {}", e)),
        }
    }

    async fn execute_shell_command(&self, api: &ForgeAPI, command: &str) -> Result<String, String> {
        // Get the current working directory from the environment
        let cwd = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        match api.execute_shell_command(command, cwd).await {
            Ok(output) => {
                let mut result = format!("Command executed successfully.\n\n");
                if !output.stdout.is_empty() {
                    result.push_str(&format!("STDOUT:\n{}\n", output.stdout));
                }
                if !output.stderr.is_empty() {
                    result.push_str(&format!("STDERR:\n{}\n", output.stderr));
                }
                if let Some(exit_code) = output.exit_code {
                    result.push_str(&format!("Exit Code: {}\n", exit_code));
                }
                Ok(result)
            }
            Err(e) => Err(format!("Failed to execute command: {}", e)),
        }
    }

    /// Get system information
    pub async fn get_system_info(&self) -> Result<SystemInfo, String> {
        let api_guard = self.api.lock().await;

        match api_guard.as_ref() {
            Some(_api) => {
                Ok(SystemInfo {
                    service: "forgecode-gateway".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    status: "running".to_string(),
                    forgecode_integrated: true,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                })
            }
            None => Ok(SystemInfo {
                service: "forgecode-gateway".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                status: "running".to_string(),
                forgecode_integrated: false,
                timestamp: chrono::Utc::now().to_rfc3339(),
            }),
        }
    }
}

/// System information response
#[derive(Debug, Serialize)]
pub struct SystemInfo {
    pub service: String,
    pub version: String,
    pub status: String,
    pub forgecode_integrated: bool,
    pub timestamp: String,
}

/// Command execution request
#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub command: String,
}

/// File read request
#[derive(Debug, Deserialize)]
pub struct FileReadRequest {
    pub path: String,
}

/// Global forge service instance
lazy_static::lazy_static! {
    static ref FORGE_SERVICE: ForgeService = ForgeService::new();
}

/// Initialize the forgecode service
pub async fn initialize_forge_service() -> Result<(), String> {
    FORGE_SERVICE.initialize().await
}

/// Execute a command handler
pub async fn execute_command_handler(
    command_request: web::Json<CommandRequest>,
) -> impl Responder {
    match FORGE_SERVICE.execute_command(&command_request.command).await {
        Ok(output) => HttpResponse::Ok().json(json!({
            "success": true,
            "output": output
        })),
        Err(error) => HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": error
        })),
    }
}

/// Read file handler
pub async fn read_file_handler(
    file_request: web::Json<FileReadRequest>,
) -> impl Responder {
    match FORGE_SERVICE.read_file(&file_request.path).await {
        Ok(content) => HttpResponse::Ok().json(json!({
            "success": true,
            "content": content
        })),
        Err(error) => HttpResponse::BadRequest().json(json!({
            "success": false,
            "error": error
        })),
    }
}

/// System info handler
pub async fn system_info_handler() -> impl Responder {
    match FORGE_SERVICE.get_system_info().await {
        Ok(info) => HttpResponse::Ok().json(json!(info)),
        Err(error) => HttpResponse::InternalServerError().json(json!({
            "error": error
        })),
    }
}