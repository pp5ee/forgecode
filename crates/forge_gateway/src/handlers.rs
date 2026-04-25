use std::sync::Arc;

use axum::{
    extract::{Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{Html, Json},
    Json as AxumJson,
};
use forge_gateway::{ForgeCodeClient, WebSocketHandler};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthToken;

#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub command: String,
    pub working_dir: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub success: bool,
}

#[derive(Debug, Deserialize)]
pub struct FileListRequest {
    pub path: Option<String>,
}

pub async fn auth_handler() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

pub async fn token_validate_handler(
    Query(token): Query<AuthToken>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if token.validate().await {
        Ok(Json(serde_json::json!({ "valid": true })))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn file_list_handler(
    State(client): State<Arc<ForgeCodeClient>>,
    Query(params): Query<FileListRequest>,
) -> Result<Json<Vec<String>>, StatusCode> {
    match client.list_files(params.path).await {
        Ok(files) => Ok(Json(files)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn command_execute_handler(
    State(client): State<Arc<ForgeCodeClient>>,
    AxumJson(request): AxumJson<CommandRequest>,
) -> Result<Json<CommandResponse>, StatusCode> {
    match client.execute_command(request).await {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn command_list_handler(
    State(client): State<Arc<ForgeCodeClient>>,
) -> Result<Json<forge_app::dto::ToolsOverview>, StatusCode> {
    match client.get_tools().await {
        Ok(tools) => Ok(Json(tools)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}