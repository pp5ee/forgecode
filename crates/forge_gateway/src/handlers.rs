//! HTTP request handlers for the web gateway

use actix_web::{web, HttpResponse, Result};
use forge_api::API;
use forge_config::ForgeConfig;
use serde_json::json;

/// Health check endpoint
pub async fn health() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({"status": "healthy"})))
}

/// Handle conversation requests
pub async fn conversation(
    api: web::Data<API>,
    payload: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    // TODO: Implement conversation handling using forge_api
    Ok(HttpResponse::Ok().json(json!({"message": "Conversation endpoint"})))
}

/// List files in the workspace
pub async fn list_files(api: web::Data<API>) -> Result<HttpResponse> {
    // TODO: Implement file listing using forge_api::API::discover()
    Ok(HttpResponse::Ok().json(json!({"files": []})))
}

/// Get file content
pub async fn get_file(
    api: web::Data<API>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    // TODO: Implement file content retrieval
    Ok(HttpResponse::Ok().json(json!({"content": "", "path": path.into_inner()})))
}

/// Update file content
pub async fn update_file(
    api: web::Data<API>,
    path: web::Path<String>,
    payload: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    // TODO: Implement file update
    Ok(HttpResponse::Ok().json(json!({"message": "File updated", "path": path.into_inner()})))
}

/// Execute shell commands
pub async fn execute_command(
    api: web::Data<API>,
    payload: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    // TODO: Implement command execution using forge_api::API::execute_shell_command()
    Ok(HttpResponse::Ok().json(json!({"output": "", "exit_code": 0})))
}

/// Serve the web UI
pub async fn serve_ui() -> Result<HttpResponse> {
    // TODO: Implement static file serving for the web UI
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
        </div>
    </body>
    </html>
    "#;

    Ok(HttpResponse::Ok()
        .content_type("text/html")
        .body(html_content))
}