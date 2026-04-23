//! HTTP request handlers for the web gateway

use std::path::PathBuf;

use actix_web::{web, HttpResponse, Result};
use forge_api::API;
use forge_config::ForgeConfig;
use forge_domain::{
    CreateUrlTokenRequest, CreateUrlTokenResponse, File, UrlToken, UrlTokenId,
};
use forge_infra::UrlTokenRepository;
use forge_services::UrlTokenService;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::auth::TokenAuthExt;

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
    req: actix_web::HttpRequest,
    payload: web::Json<ConversationRequest>,
) -> Result<HttpResponse> {
    let _token_id = req.token_id();
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
            Ok(HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to list files"})))
        }
    }
}

/// Response structure for file content
#[derive(Debug, Serialize)]
pub struct FileContentResponse {
    pub path: String,
    pub content: String,
    pub exists: bool,
}

/// Get file content
pub async fn get_file(api: web::Data<API>, path: web::Path<String>) -> Result<HttpResponse> {
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
        message: format!(
            "File updated with {} characters",
            update_request.content.len()
        ),
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

    match api
        .execute_shell_command(&command_request.command, working_dir)
        .await
    {
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

/// Create a new URL token
pub async fn create_token<R: UrlTokenRepository>(
    token_service: web::Data<UrlTokenService<R>>,
    payload: web::Json<CreateUrlTokenRequest>,
) -> Result<HttpResponse> {
    let request = payload.into_inner();

    match token_service.create_token(request).await {
        Ok(response) => Ok(HttpResponse::Created().json(response)),
        Err(e) => {
            log::error!("Failed to create token: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to create token"})))
        }
    }
}

/// Response for listing tokens
#[derive(Debug, Serialize)]
pub struct ListTokensResponse {
    pub tokens: Vec<TokenSummary>,
}

/// Summary of a token (without the full token string)
#[derive(Debug, Serialize)]
pub struct TokenSummary {
    pub id: UrlTokenId,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub description: Option<String>,
    pub revoked: bool,
    pub expired: bool,
}

impl From<UrlToken> for TokenSummary {
    fn from(token: UrlToken) -> Self {
        Self {
            id: token.id,
            created_at: token.created_at,
            expires_at: token.expires_at,
            description: token.description,
            revoked: token.revoked,
            expired: token.is_expired(),
        }
    }
}

/// List all URL tokens
pub async fn list_tokens<R: UrlTokenRepository>(
    token_service: web::Data<UrlTokenService<R>>,
) -> Result<HttpResponse> {
    match token_service.list_tokens().await {
        Ok(tokens) => {
            let summaries: Vec<TokenSummary> = tokens
                .into_values()
                .map(TokenSummary::from)
                .collect();
            Ok(HttpResponse::Ok().json(ListTokensResponse { tokens: summaries }))
        }
        Err(e) => {
            log::error!("Failed to list tokens: {}", e);
            Ok(HttpResponse::InternalServerError()
                .json(json!({"error": "Failed to list tokens"})))
        }
    }
}

/// Revoke a URL token
pub async fn revoke_token<R: UrlTokenRepository>(
    token_service: web::Data<UrlTokenService<R>>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let token_id_str = path.into_inner();
    let token_id = UrlTokenId::from(token_id_str);

    match token_service.revoke_token(&token_id).await {
        Ok(()) => Ok(HttpResponse::NoContent().finish()),
        Err(e) => {
            log::error!("Failed to revoke token: {}", e);
            // Check if it's a not found error
            if e.to_string().contains("Token not found") {
                Ok(HttpResponse::NotFound().json(json!({"error": "Token not found"})))
            } else {
                Ok(HttpResponse::InternalServerError()
                    .json(json!({"error": "Failed to revoke token"})))
            }
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
            .auth-section { margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 4px; }
            .token-example { background: #f8f9fa; padding: 10px; border-radius: 4px; font-family: monospace; }
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
            <div class="auth-section">
                <h3>Authentication</h3>
                <p>This gateway uses URL tokens for authentication. Tokens can be provided:</p>
                <ul>
                    <li>As a query parameter: <code>?token=your_token_here</code></li>
                    <li>In the Authorization header: <code>Bearer your_token_here</code></li>
                </ul>
                <div class="token-example">
                    POST /api/tokens - Create a new token<br>
                    GET /api/tokens - List all tokens<br>
                    DELETE /api/tokens/{id} - Revoke a token
                </div>
            </div>
            <div class="api-status">
                <h3>Protected API Endpoints</h3>
                <div class="api-endpoint">
                    <strong>GET /api/health</strong> - Health check (public)
                </div>
                <div class="api-endpoint">
                    <strong>POST /api/conversation</strong> - AI conversation (protected)
                </div>
                <div class="api-endpoint">
                    <strong>GET /api/files</strong> - List files in workspace (protected)
                </div>
                <div class="api-endpoint">
                    <strong>GET /api/files/{path}</strong> - Get file content (protected)
                </div>
                <div class="api-endpoint">
                    <strong>PUT /api/files/{path}</strong> - Update file content (protected)
                </div>
                <div class="api-endpoint">
                    <strong>POST /api/execute</strong> - Execute shell command (protected)
                </div>
                <div class="api-endpoint">
                    <strong>GET /ws/conversation</strong> - WebSocket conversation (protected)
                </div>
                <div class="api-endpoint">
                    <strong>GET /ws/command</strong> - WebSocket command execution (protected)
                </div>
            </div>
        </div>
    </body>
    </html>
    "#;

    Ok(HttpResponse::Ok().content_type("text/html").body(html_content))
}
