use crate::auth::{TokenManager, AuthError};
use std::sync::Arc;
use crate::forgecode_client::{ForgeCodeClient, ExecuteCodeRequest};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use warp::{Filter, Rejection, Reply};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteRequest {
    pub code: String,
    pub language: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteResponse {
    pub output: String,
    pub error: Option<String>,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
    pub user_id: Option<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
}

pub async fn execute_code_handler(
    forgecode_client: ForgeCodeClient,
    body: ExecuteRequest,
) -> Result<impl Reply, Rejection> {
    let request = ExecuteCodeRequest {
        code: body.code.clone(),
        language: body.language.clone(),
        session_id: body.session_id.clone(),
    };

    match forgecode_client.execute_code(request).await {
        Ok(response) => {
            let reply = ExecuteResponse {
                output: response.output,
                error: response.error,
                session_id: response.session_id,
            };
            Ok(warp::reply::json(&reply))
        }
        Err(_) => {
            // Fallback to mock response if service is unavailable
            let reply = ExecuteResponse {
                output: format!("Mock execution of {} code: {}", body.language, body.code),
                error: None,
                session_id: body.session_id,
            };
            Ok(warp::reply::json(&reply))
        }
    }
}

pub async fn generate_token_handler(
    token_manager: Arc<TokenManager>,
    body: AuthRequest,
) -> Result<impl Reply, Rejection> {
    match token_manager.generate_token(body.user_id, body.permissions).await {
        Ok(token_id) => {
            let response = AuthResponse {
                token: token_id.to_string(),
            };
            Ok(warp::reply::json(&response))
        }
        Err(e) => Err(warp::reject::custom(AuthRejection(e))),
    }
}

pub async fn validate_token_handler(
    token_manager: Arc<TokenManager>,
    token: String,
) -> Result<impl Reply, Rejection> {
    let token_id = Uuid::parse_str(&token)
        .map_err(|_| warp::reject::custom(AuthRejection(AuthError::InvalidTokenFormat)))?;

    match token_manager.validate_token(token_id, None).await {
        Ok(token_data) => {
            let response = serde_json::json!({
                "valid": true,
                "user_id": token_data.user_id,
                "permissions": token_data.permissions
            });
            Ok(warp::reply::json(&response))
        }
        Err(e) => Err(warp::reject::custom(AuthRejection(e))),
    }
}

pub async fn renew_token_handler(
    token_manager: Arc<TokenManager>,
    token: String,
) -> Result<impl Reply, Rejection> {
    let token_id = Uuid::parse_str(&token)
        .map_err(|_| warp::reject::custom(AuthRejection(AuthError::InvalidTokenFormat)))?;

    match token_manager.renew_token(token_id).await {
        Ok(new_token_id) => {
            let response = AuthResponse {
                token: new_token_id.to_string(),
            };
            Ok(warp::reply::json(&response))
        }
        Err(e) => Err(warp::reject::custom(AuthRejection(e))),
    }
}

pub async fn revoke_token_handler(
    token_manager: Arc<TokenManager>,
    token: String,
) -> Result<impl Reply, Rejection> {
    let token_id = Uuid::parse_str(&token)
        .map_err(|_| warp::reject::custom(AuthRejection(AuthError::InvalidTokenFormat)))?;

    match token_manager.revoke_token(token_id).await {
        Ok(()) => {
            let response = serde_json::json!({"success": true});
            Ok(warp::reply::json(&response))
        }
        Err(e) => Err(warp::reject::custom(AuthRejection(e))),
    }
}

// Custom rejection for auth errors
#[derive(Debug)]
pub struct AuthRejection(pub AuthError);

impl warp::reject::Reject for AuthRejection {}

impl std::fmt::Display for AuthRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Auth error: {}", self.0)
    }
}

pub async fn handle_auth_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    if let Some(auth_rejection) = err.find::<AuthRejection>() {
        let error_message = auth_rejection.0.to_string();
        let json = warp::reply::json(&serde_json::json!({
            "error": error_message
        }));
        return Ok(warp::reply::with_status(json, warp::http::StatusCode::UNAUTHORIZED));
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&serde_json::json!({
            "error": "Internal server error"
        })),
        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
    ))
}