use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use crate::auth::TokenManager;
use crate::forgecode_client::ForgeCodeClient;

#[derive(Debug, Deserialize)]
pub struct TokenRequest {
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
    pub expires_in: u64,
}

#[derive(Debug, Deserialize)]
pub struct CommandRequest {
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub output: String,
    pub success: bool,
}

/// Generate a new authentication token
#[actix_web::post("/token/generate")]
pub async fn generate_token(
    token_manager: web::Data<TokenManager>,
    request: web::Json<TokenRequest>,
) -> impl Responder {
    match token_manager.generate_token(request.permissions.clone()) {
        Ok((token, expires_in)) => {
            HttpResponse::Ok().json(TokenResponse {
                token,
                expires_in,
            })
        }
        Err(e) => HttpResponse::BadRequest().body(format!("Failed to generate token: {}", e)),
    }
}

/// Renew an existing token
#[actix_web::post("/token/renew")]
pub async fn renew_token(
    token_manager: web::Data<TokenManager>,
    request: web::Json<TokenRequest>,
) -> impl Responder {
    // For now, this is the same as generate_token
    // In a real implementation, this would validate the existing token first
    match token_manager.generate_token(request.permissions.clone()) {
        Ok((token, expires_in)) => {
            HttpResponse::Ok().json(TokenResponse {
                token,
                expires_in,
            })
        }
        Err(e) => HttpResponse::BadRequest().body(format!("Failed to renew token: {}", e)),
    }
}

/// Execute a forgecode command
#[actix_web::post("/command/execute")]
pub async fn execute_command(
    forgecode_client: web::Data<ForgeCodeClient>,
    request: web::Json<CommandRequest>,
) -> impl Responder {
    match forgecode_client.execute_command(&request.command, &request.args).await {
        Ok(output) => HttpResponse::Ok().json(CommandResponse {
            output,
            success: true,
        }),
        Err(e) => HttpResponse::InternalServerError().json(CommandResponse {
            output: format!("Error: {}", e),
            success: false,
        }),
    }
}