//! Integration with forgecode core services

use std::sync::Arc;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;

use forge_services::{ForgeServices, Services};
use forge_api::ApiError;

/// Request for command execution
#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<String>,
}

/// Response from command execution
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub success: bool,
    pub output: String,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
}

/// Request for file operations
#[derive(Debug, Deserialize)]
pub struct FileReadRequest {
    pub path: String,
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

/// Response from file operations
#[derive(Debug, Serialize)]
pub struct FileReadResponse {
    pub success: bool,
    pub content: String,
    pub path: String,
    pub error: Option<String>,
}

/// Gateway service integration
pub struct GatewayIntegration<F>
where
    F: Services + Send + Sync + 'static,
{
    services: Arc<F>,
}

impl<F> GatewayIntegration<F>
where
    F: Services + Send + Sync + 'static,
{
    /// Create new gateway integration
    pub fn new(services: Arc<F>) -> Self {
        Self { services }
    }

    /// Execute a shell command through forgecode
    pub async fn execute_command(&self, request: CommandRequest) -> Result<CommandResponse, ApiError> {
        // Use forge_services to execute the actual command
        let command_result = self.services.execute_command(
            &request.command,
            &request.args,
            request.working_dir.as_deref()
        ).await;

        match command_result {
            Ok(output) => Ok(CommandResponse {
                success: output.exit_code == 0,
                output: output.stdout,
                exit_code: Some(output.exit_code),
                error: if output.exit_code != 0 { Some(output.stderr) } else { None },
            }),
            Err(error) => Ok(CommandResponse {
                success: false,
                output: String::new(),
                exit_code: None,
                error: Some(error.to_string()),
            }),
        }
    }

    /// Read file content through forgecode
    pub async fn read_file(&self, request: FileReadRequest) -> Result<FileReadResponse, ApiError> {
        // Use forge_services to read the actual file
        let file_content = self.services.read_file(
            &request.path,
            request.offset,
            request.limit
        ).await;

        match file_content {
            Ok(content) => Ok(FileReadResponse {
                success: true,
                content,
                path: request.path,
                error: None,
            }),
            Err(error) => Ok(FileReadResponse {
                success: false,
                content: String::new(),
                path: request.path,
                error: Some(error.to_string()),
            }),
        }
    }

    /// Get system information
    pub async fn get_system_info(&self) -> Result<serde_json::Value, ApiError> {
        // Use forge_services to get actual system information
        let system_info = self.services.get_system_info().await;

        match system_info {
            Ok(info) => Ok(json!({
                "service": "forgecode-gateway",
                "version": env!("CARGO_PKG_VERSION"),
                "forgecode_version": "0.1.0",
                "status": "connected",
                "system_info": info
            })),
            Err(error) => Ok(json!({
                "service": "forgecode-gateway",
                "version": env!("CARGO_PKG_VERSION"),
                "forgecode_version": "0.1.0",
                "status": "error",
                "error": error.to_string()
            })),
        }
    }
}

/// Handler for command execution endpoint
pub async fn execute_command_handler<F>(
    request: web::Json<CommandRequest>,
    integration: web::Data<GatewayIntegration<F>>,
) -> impl Responder
where
    F: Services + Send + Sync + 'static,
{
    match integration.execute_command(request.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(error) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": error.to_string()
        })),
    }
}

/// Handler for file reading endpoint
pub async fn read_file_handler<F>(
    request: web::Json<FileReadRequest>,
    integration: web::Data<GatewayIntegration<F>>,
) -> impl Responder
where
    F: Services + Send + Sync + 'static,
{
    match integration.read_file(request.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(error) => HttpResponse::InternalServerError().json(json!({
            "success": false,
            "error": error.to_string()
        })),
    }
}

/// Handler for system info endpoint
pub async fn system_info_handler<F>(
    integration: web::Data<GatewayIntegration<F>>,
) -> impl Responder
where
    F: Services + Send + Sync + 'static,
{
    match integration.get_system_info().await {
        Ok(info) => HttpResponse::Ok().json(info),
        Err(error) => HttpResponse::InternalServerError().json(json!({
            "error": error.to_string()
        })),
    }
}