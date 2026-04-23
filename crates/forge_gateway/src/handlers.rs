//! HTTP request handlers for the web gateway

use actix_web::{web, HttpResponse, Result};
use forge_api::API;
use forge_config::ForgeConfig;
use forge_domain::File;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;

/// Health check endpoint
pub async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({"status": "healthy"})))
}

/// Request structure for conversation
#[derive(Debug, Deserialize)]
pub struct ConversationRequest {
    pub message: String,
    pub conversation_id: Option<String>,
}

/// Response structure for conversation
#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    pub response: String,
    pub conversation_id: String,
}

/// Handle conversation requests
pub async fn conversation(
    api: web::Data<API>,
    payload: web::Json<ConversationRequest>,
) -> Result<HttpResponse> {
    let request = payload.into_inner();

    // For now, return a placeholder response
    // In a real implementation, this would use forge_api::API::chat()
    let response = ConversationResponse {
        response: format!("Received message: {}", request.message),
        conversation_id: request.conversation_id.unwrap_or_else(|| "default".to_string()),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Response structure for file listing
#[derive(Debug, Serialize)]
pub struct FileListResponse {
    pub files: Vec<FileInfo>,
}

/// File information structure
#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub path: String,
    pub is_dir: bool,
}

/// List files in the workspace
pub async fn list_files(api: web::Data<API>) -> Result<HttpResponse> {
    match api.discover().await {
        Ok(files) => {
            let file_info: Vec<FileInfo> = files
                .into_iter()
                .map(|file| FileInfo {
                    path: file.path,
                    is_dir: file.is_dir,
                })
                .collect();

            let response = FileListResponse { files: file_info };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            log::error!("Failed to list files: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({"error": "Failed to list files"})))
        }
    }
}

/// Request structure for file content
#[derive(Debug, Deserialize)]
pub struct FileContentRequest {
    pub path: String,
}

/// Response structure for file content
#[derive(Debug, Serialize)]
pub struct FileContentResponse {
    pub path: String,
    pub content: String,
    pub exists: bool,
}

/// Get file content
pub async fn get_file(
    api: web::Data<API>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let file_path = path.into_inner();

    // For now, return a placeholder response
    // In a real implementation, this would read the actual file content
    let response = FileContentResponse {
        path: file_path.clone(),
        content: "File content placeholder".to_string(),
        exists: true,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Request structure for file update
#[derive(Debug, Deserialize)]
pub struct FileUpdateRequest {
    pub content: String,
}

/// Response structure for file update
#[derive(Debug, Serialize)]
pub struct FileUpdateResponse {
    pub path: String,
    pub success: bool,
    pub message: String,
}

/// Update file content
pub async fn update_file(
    api: web::Data<API>,
    path: web::Path<String>,
    payload: web::Json<FileUpdateRequest>,
) -> Result<HttpResponse> {
    let file_path = path.into_inner();
    let update_request = payload.into_inner();

    // For now, return a placeholder response
    // In a real implementation, this would write the actual file content
    let response = FileUpdateResponse {
        path: file_path,
        success: true,
        message: format!("File updated with {} characters", update_request.content.len()),
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Request structure for command execution
#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub command: String,
    pub working_dir: Option<String>,
}

/// Response structure for command execution
#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub output: String,
    pub exit_code: i32,
    pub success: bool,
}

/// Execute shell commands
pub async fn execute_command(
    api: web::Data<API>,
    payload: web::Json<CommandRequest>,
) -> Result<HttpResponse> {
    let command_request = payload.into_inner();

    // Determine working directory
    let working_dir = command_request
        .working_dir
        .map(PathBuf::from)
        .unwrap_or_else(|| api.environment().cwd.clone());

    match api.execute_shell_command(&command_request.command, working_dir).await {
        Ok(output) => {
            let response = CommandResponse {
                output: output.stdout,
                exit_code: output.exit_code,
                success: output.exit_code == 0,
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            log::error!("Failed to execute command: {}", e);
            let response = CommandResponse {
                output: format!("Error: {}", e),
                exit_code: 1,
                success: false,
            };
            Ok(HttpResponse::Ok().json(response))
        }
    }
}

/// Serve the web UI
pub async fn serve_ui() -> Result<HttpResponse> {
    let html_content = r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>ForgeCode Gateway</title>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <style>
            body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }
            .container { max-width: 1200px; margin: 0 auto; background: white; padding: 20px; border-radius: 8px; }
            .header { text-align: center; margin-bottom: 30px; }
            .status { padding: 10px; border-radius: 4px; margin: 10px 0; }
            .status.healthy { background: #d4edda; color: #155724; }
            .status.error { background: #f8d7da; color: #721c24; }
            .api-status { margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 4px; }
            .api-endpoint { margin: 10px 0; padding: 10px; background: #f8f9fa; border-radius: 4px; }
        </style>
    </head>
    <body>
        <div class="container">
            <div class="header">
                <h1>ForgeCode Gateway</h1>
                <p>Web interface for remote access to ForgeCode</p>
            </div>
            <div class="status healthy">
                Gateway is running. Web UI implementation in progress.
            </div>
            <div class="api-status">
                <h3>API Endpoints</h3>
                <div class="api-endpoint">
                    <strong>GET /api/health</strong> - Health check
                </div>
                <div class="api-endpoint">
                    <strong>POST /api/conversation</strong> - AI conversation
                </div>
                <div class="api-endpoint">
                    <strong>GET /api/files</strong> - List files in workspace
                </div>
                <div class="api-endpoint">
                    <strong>GET /api/files/{path}</strong> - Get file content
                </div>
                <div class="api-endpoint">
                    <strong>PUT /api/files/{path}</strong> - Update file content
                </div>
                <div class="api-endpoint">
                    <strong>POST /api/execute</strong> - Execute shell command
                </div>
                <div class="api-endpoint">
                    <strong>GET /ws/conversation</strong> - WebSocket conversation
                </div>
                <div class="api-endpoint">
                    <strong>GET /ws/command</strong> - WebSocket command execution
                </div>
            </div>
        </div>
    </body>
    </html>
    "#;

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content))
}