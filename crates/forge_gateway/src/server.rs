use crate::auth::TokenManager;
use crate::forgecode_client::{ForgeCodeClient, ExecuteCodeRequest};
use crate::handlers::{execute_code_handler, generate_token_handler, validate_token_handler, renew_token_handler, revoke_token_handler, handle_auth_rejection, ExecuteRequest, AuthRequest};
use serde::{Deserialize, Serialize};
// Remove unused import
use std::sync::Arc;
use tokio::sync::Mutex;
use warp::{Filter, Rejection, Reply};
use warp::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub action: String,
    pub code: Option<String>,
    pub language: Option<String>,
    pub session_id: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketResponse {
    pub action: String,
    pub output: Option<String>,
    pub error: Option<String>,
    pub session_id: Option<String>,
}

pub async fn handle_websocket_connection(
    ws: WebSocket,
    token_manager: Arc<TokenManager>,
    forgecode_client: Arc<Mutex<ForgeCodeClient>>,
) {
    let (mut ws_tx, mut ws_rx) = ws.split();

    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(message) => {
                if let Ok(text) = message.to_str() {
                    match serde_json::from_str::<WebSocketMessage>(text) {
                        Ok(ws_message) => {
                            let response = match ws_message.action.as_str() {
                                "execute" => {
                                    handle_websocket_execute(
                                        ws_message,
                                        token_manager.clone(),
                                        forgecode_client.clone(),
                                    ).await
                                }
                                "authenticate" => {
                                    handle_websocket_authenticate(
                                        ws_message,
                                        token_manager.clone(),
                                    ).await
                                }
                                "validate" => {
                                    handle_websocket_validate(
                                        ws_message,
                                        token_manager.clone(),
                                    ).await
                                }
                                _ => {
                                    WebSocketResponse {
                                        action: "error".to_string(),
                                        output: None,
                                        error: Some("Unknown action".to_string()),
                                        session_id: None,
                                    }
                                }
                            };

                            if let Ok(response_json) = serde_json::to_string(&response) {
                                if let Err(e) = ws_tx.send(Message::text(response_json)).await {
                                    eprintln!("Failed to send WebSocket message: {}", e);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            let error_response = WebSocketResponse {
                                action: "error".to_string(),
                                output: None,
                                error: Some(format!("Invalid message format: {}", e)),
                                session_id: None,
                            };
                            if let Ok(error_json) = serde_json::to_string(&error_response) {
                                let _ = ws_tx.send(Message::text(error_json)).await;
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
        }
    }
}

async fn handle_websocket_execute(
    message: WebSocketMessage,
    token_manager: Arc<TokenManager>,
    forgecode_client: Arc<Mutex<ForgeCodeClient>>,
) -> WebSocketResponse {
    // Validate token if provided
    if let Some(token) = &message.token {
        if let Ok(token_id) = Uuid::parse_str(token) {
            if let Err(e) = token_manager.validate_token(token_id, Some("execute")).await {
                return WebSocketResponse {
                    action: "error".to_string(),
                    output: None,
                    error: Some(format!("Authentication failed: {}", e)),
                    session_id: message.session_id,
                };
            }
        } else {
            return WebSocketResponse {
                action: "error".to_string(),
                output: None,
                error: Some("Invalid token format".to_string()),
                session_id: message.session_id,
            };
        }
    }

    // Execute code through forgecode service
    if let (Some(code), Some(language)) = (message.code, message.language) {
        let language_clone = language.clone();
        let request = ExecuteCodeRequest {
            code,
            language,
            session_id: message.session_id.clone(),
        };

        let client_guard = forgecode_client.lock().await;
        match client_guard.execute_code(request).await {
            Ok(response) => {
                WebSocketResponse {
                    action: "execution_result".to_string(),
                    output: Some(response.output),
                    error: response.error,
                    session_id: response.session_id,
                }
            }
            Err(_) => {
                // Fallback to mock execution
                WebSocketResponse {
                    action: "execution_result".to_string(),
                    output: Some(format!("Mock WebSocket execution of {} code", language_clone)),
                    error: None,
                    session_id: message.session_id,
                }
            }
        }
    } else {
        WebSocketResponse {
            action: "error".to_string(),
            output: None,
            error: Some("Missing code or language".to_string()),
            session_id: message.session_id,
        }
    }
}

pub async fn handle_websocket_authenticate(
    message: WebSocketMessage,
    token_manager: Arc<TokenManager>,
) -> WebSocketResponse {
    let user_id = message.token.clone(); // Using token field for user_id in this context
    let permissions = vec!["execute".to_string()]; // Default permissions for WebSocket

    match token_manager.generate_token(user_id, permissions).await {
        Ok(token_id) => {
            WebSocketResponse {
                action: "authentication_result".to_string(),
                output: Some(token_id.to_string()),
                error: None,
                session_id: None,
            }
        }
        Err(e) => {
            WebSocketResponse {
                action: "error".to_string(),
                output: None,
                error: Some(format!("Authentication failed: {}", e)),
                session_id: None,
            }
        }
    }
}

pub async fn handle_websocket_validate(
    message: WebSocketMessage,
    token_manager: Arc<TokenManager>,
) -> WebSocketResponse {
    if let Some(token) = message.token {
        if let Ok(token_id) = Uuid::parse_str(&token) {
            match token_manager.validate_token(token_id, None).await {
                Ok(token_data) => {
                    WebSocketResponse {
                        action: "validation_result".to_string(),
                        output: Some(format!("Valid token for user: {:?}", token_data.user_id)),
                        error: None,
                        session_id: None,
                    }
                }
                Err(e) => {
                    WebSocketResponse {
                        action: "error".to_string(),
                        output: None,
                        error: Some(format!("Token validation failed: {}", e)),
                        session_id: None,
                    }
                }
            }
        } else {
            WebSocketResponse {
                action: "error".to_string(),
                output: None,
                error: Some("Invalid token format".to_string()),
                session_id: None,
            }
        }
    } else {
        WebSocketResponse {
            action: "error".to_string(),
            output: None,
            error: Some("Missing token".to_string()),
            session_id: None,
        }
    }
}

pub fn create_routes(
    token_manager: TokenManager,
    forgecode_client: ForgeCodeClient,
) -> impl Filter<Extract = impl Reply, Error = std::convert::Infallible> + Clone {
    let token_manager = Arc::new(token_manager);
    let forgecode_client = Arc::new(Mutex::new(forgecode_client));

    // Static file serving - use a separate route that handles the different error type
    let static_files = warp::path("static")
        .and(warp::fs::dir("crates/forge_gateway/static"));

    // API routes
    let api_routes = warp::path("api")
        .and(
            // Execute code endpoint
            warp::path("execute")
                .and(warp::post())
                .and(warp::body::json())
                .and_then({
                    let forgecode_client = forgecode_client.clone();
                    move |body: ExecuteRequest| {
                        let client = forgecode_client.clone();
                        async move {
                            let client_guard = client.lock().await;
                            execute_code_handler(client_guard.clone(), body).await
                        }
                    }
                })
                // Token management endpoints
                .or(warp::path("auth")
                    .and(warp::path("generate"))
                    .and(warp::post())
                    .and(warp::body::json())
                    .and_then({
                        let token_manager = token_manager.clone();
                        move |body: AuthRequest| {
                            let tm = token_manager.clone();
                            generate_token_handler(tm, body)
                        }
                    }))
                .or(warp::path("auth")
                    .and(warp::path("validate"))
                    .and(warp::post())
                    .and(warp::body::json())
                    .and_then({
                        let token_manager = token_manager.clone();
                        move |body: serde_json::Value| {
                            let tm = token_manager.clone();
                            async move {
                                if let Some(token) = body.get("token").and_then(|t| t.as_str()) {
                                    validate_token_handler(tm, token.to_string()).await
                                } else {
                                    Err(warp::reject::custom(crate::handlers::AuthRejection(
                                        crate::auth::AuthError::InvalidTokenFormat,
                                    )))
                                }
                            }
                        }
                    }))
                .or(warp::path("auth")
                    .and(warp::path("renew"))
                    .and(warp::post())
                    .and(warp::body::json())
                    .and_then({
                        let token_manager = token_manager.clone();
                        move |body: serde_json::Value| {
                            let tm = token_manager.clone();
                            async move {
                                if let Some(token) = body.get("token").and_then(|t| t.as_str()) {
                                    renew_token_handler(tm, token.to_string()).await
                                } else {
                                    Err(warp::reject::custom(crate::handlers::AuthRejection(
                                        crate::auth::AuthError::InvalidTokenFormat,
                                    )))
                                }
                            }
                        }
                    }))
                .or(warp::path("auth")
                    .and(warp::path("revoke"))
                    .and(warp::post())
                    .and(warp::body::json())
                    .and_then({
                        let token_manager = token_manager.clone();
                        move |body: serde_json::Value| {
                            let tm = token_manager.clone();
                            async move {
                                if let Some(token) = body.get("token").and_then(|t| t.as_str()) {
                                    revoke_token_handler(tm, token.to_string()).await
                                } else {
                                    Err(warp::reject::custom(crate::handlers::AuthRejection(
                                        crate::auth::AuthError::InvalidTokenFormat,
                                    )))
                                }
                            }
                        }
                    }))
        );

    // WebSocket route
    let websocket_route = warp::path("ws")
        .and(warp::ws())
        .and_then({
            let token_manager = token_manager.clone();
            let forgecode_client = forgecode_client.clone();
            move |ws: warp::ws::Ws| {
                let tm = token_manager.clone();
                let fc = forgecode_client.clone();
                async move {
                    Ok::<_, Rejection>(ws.on_upgrade(move |socket| {
                        handle_websocket_connection(socket, tm, fc)
                    }))
                }
            }
        });

    // Combine API and WebSocket routes only
    // Static file serving is better handled separately or through a reverse proxy
    let routes = api_routes
        .or(websocket_route)
        .recover(handle_auth_rejection);

    routes
}