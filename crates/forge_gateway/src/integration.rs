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
                // Simulate command execution - in a real implementation, this would
                // call the appropriate forgecode service methods
                let output = format!("Executing command: {}\n", command);

                // Simulate different command responses
                if command.contains("help") {
                    Ok(format!("{}Available commands: help, status, version, config", output))
                } else if command.contains("status") {
                    Ok(format!("{}Forgecode gateway is running. Services: OK", output))
                } else if command.contains("version") {
                    Ok(format!("{}Forgecode Gateway v{}", output, env!("CARGO_PKG_VERSION")))
                } else if command.contains("config") {
                    Ok(format!("{}Gateway configuration loaded successfully", output))
                } else {
                    Ok(format!("{}Unknown command: {}", output, command))
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