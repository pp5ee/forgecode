use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::auth::{extract_token_from_extensions, SecureTokenManager};
use crate::forgecode_client::ForgeCodeClient;

/// WebSocket connection for real-time terminal streaming
pub struct TerminalWebSocket {
    /// Unique session ID
    session_id: Uuid,
    /// Last heartbeat time
    hb: Instant,
    /// ForgeCode client for command execution
    forgecode_client: web::Data<ForgeCodeClient>,
    /// Token manager for authentication
    token_manager: web::Data<SecureTokenManager>,
}

impl TerminalWebSocket {
    /// Create a new WebSocket connection
    pub fn new(
        forgecode_client: web::Data<ForgeCodeClient>,
        token_manager: web::Data<SecureTokenManager>,
    ) -> Self {
        Self {
            session_id: Uuid::new_v4(),
            hb: Instant::now(),
            forgecode_client,
            token_manager,
        }
    }

    /// Send heartbeat to keep connection alive
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(Duration::from_secs(30), |act, ctx| {
            if Instant::now().duration_since(act.hb) > Duration::from_secs(60) {
                // Connection timeout
                ctx.stop();
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl actix::Actor for TerminalWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        log::info!("WebSocket connection started for session: {}", self.session_id);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        log::info!("WebSocket connection stopped for session: {}", self.session_id);
        actix::Running::Stop
    }
}

/// WebSocket message types
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    /// Execute a command
    ExecuteCommand { command: String, args: Vec<String> },
    /// Send input to running command
    SendInput { input: String },
    /// Heartbeat response
    Heartbeat,
    /// Error message
    Error { message: String },
    /// Command output
    Output { data: String, is_stderr: bool },
    /// Command completed
    CommandCompleted { exit_code: i32 },
}

impl actix::StreamHandler<Result<ws::Message, ws::ProtocolError>> for TerminalWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                match serde_json::from_str::<WebSocketMessage>(&text) {
                    Ok(WebSocketMessage::ExecuteCommand { command, args }) => {
                        // Execute command through forgecode client
                        let client = self.forgecode_client.clone();
                        let session_id = self.session_id;

                        actix::spawn(async move {
                            match client.execute_command(&command, &args).await {
                                Ok(output) => {
                                    // Send output in chunks to simulate real-time streaming
                                    let message = WebSocketMessage::Output {
                                        data: output,
                                        is_stderr: false,
                                    };
                                    // In real implementation, we'd send this back to the WebSocket
                                    // For now, we'll log it
                                    log::info!("Command output for session {}: {:?}", session_id, message);
                                }
                                Err(e) => {
                                    let message = WebSocketMessage::Error {
                                        message: format!("Command execution failed: {}", e),
                                    };
                                    log::error!("Command error for session {}: {:?}", session_id, message);
                                }
                            }
                        });
                    }
                    Ok(WebSocketMessage::SendInput { input }) => {
                        // Handle user input for interactive commands
                        log::info!("Received input for session {}: {}", self.session_id, input);
                        // In a real implementation, we'd send this to the running process
                    }
                    Ok(WebSocketMessage::Heartbeat) => {
                        // Respond to heartbeat
                        let response = WebSocketMessage::Heartbeat;
                        if let Ok(json) = serde_json::to_string(&response) {
                            ctx.text(json);
                        }
                    }
                    Err(e) => {
                        log::warn!("Invalid WebSocket message: {}", e);
                        let error_msg = WebSocketMessage::Error {
                            message: format!("Invalid message format: {}", e),
                        };
                        if let Ok(json) = serde_json::to_string(&error_msg) {
                            ctx.text(json);
                        }
                    }
                    _ => {
                        log::warn!("Unsupported WebSocket message type");
                    }
                }
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Continuation(_)) => {
                ctx.stop();
            }
            _ => (),
        }
    }
}

/// WebSocket endpoint handler
pub async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    forgecode_client: web::Data<ForgeCodeClient>,
    token_manager: web::Data<SecureTokenManager>,
) -> Result<HttpResponse, actix_web::Error> {
    // Extract token from query string (for WebSocket connections)
    let token_str = req
        .query_string()
        .split('&')
        .find(|param| param.starts_with("token="))
        .and_then(|param| param.split('=').nth(1))
        .or_else(|| {
            // Fallback to extensions (for HTTP upgrade requests processed by middleware)
            extract_token_from_extensions(&req).map(|token| token.token.as_str())
        });

    let token_str = match token_str {
        Some(token) => token,
        None => {
            return Ok(HttpResponse::Unauthorized().body("Authentication required"));
        }
    };

    // Validate token using token manager
    let token = match token_manager.validate_token(token_str).await {
        Ok(Some(token)) => token,
        Ok(None) => {
            return Ok(HttpResponse::Unauthorized().body("Invalid token"));
        }
        Err(e) => {
            log::error!("Token validation error: {}", e);
            return Ok(HttpResponse::InternalServerError().body("Authentication error"));
        }
    };

    // Check if token has execute permission
    if !token.permissions.contains(&"execute".to_string()) {
        return Ok(HttpResponse::Forbidden().body("Token does not have execute permission"));
    }

    // Create WebSocket connection
    let ws = TerminalWebSocket::new(forgecode_client, token_manager);
    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}

/// Terminal session management
pub struct TerminalSessionManager {
    /// Active terminal sessions
    sessions: std::sync::RwLock<std::collections::HashMap<Uuid, TerminalSession>>,
}

impl TerminalSessionManager {
    pub fn new() -> Self {
        Self {
            sessions: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Create a new terminal session
    pub fn create_session(&self) -> Uuid {
        let session_id = Uuid::new_v4();
        let session = TerminalSession::new(session_id);
        self.sessions.write().unwrap().insert(session_id, session);
        session_id
    }

    /// Get a terminal session
    pub fn get_session(&self, session_id: &Uuid) -> Option<TerminalSession> {
        self.sessions.read().unwrap().get(session_id).cloned()
    }

    /// Remove a terminal session
    pub fn remove_session(&self, session_id: &Uuid) {
        self.sessions.write().unwrap().remove(session_id);
    }
}

/// Individual terminal session
#[derive(Clone)]
pub struct TerminalSession {
    pub session_id: Uuid,
    pub created_at: std::time::SystemTime,
    pub last_activity: std::time::SystemTime,
}

impl TerminalSession {
    pub fn new(session_id: Uuid) -> Self {
        let now = std::time::SystemTime::now();
        Self {
            session_id,
            created_at: now,
            last_activity: now,
        }
    }

    pub fn update_activity(&mut self) {
        self.last_activity = std::time::SystemTime::now();
    }
}