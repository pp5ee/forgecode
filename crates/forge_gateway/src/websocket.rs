//! WebSocket implementation for real-time terminal communication

use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// WebSocket message types
#[derive(Debug, Deserialize, Serialize)]
pub struct TerminalCommand {
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TerminalOutput {
    pub output: String,
    pub is_error: bool,
    pub exit_code: Option<i32>,
}

/// WebSocket connection actor
pub struct WebSocketConnection {
    /// Connection ID
    id: Uuid,
    /// Last heartbeat time
    hb: Instant,
    /// Command execution channel
    command_sender: Option<mpsc::Sender<String>>,
}

impl WebSocketConnection {
    /// Create a new WebSocket connection
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            hb: Instant::now(),
            command_sender: None,
        }
    }

    /// Execute a command and stream output back via WebSocket
    async fn execute_command(&self, cmd: TerminalCommand, ctx: &mut ws::WebsocketContext<Self>) {
        let (tx, mut rx) = mpsc::channel(32);

        // Spawn command execution task
        let connection_id = self.id;
        tokio::spawn(async move {
            // Use the forgecode integration service to execute the command
            // This will be properly integrated once the forgecode service is available
            let command_str = if !cmd.args.is_empty() {
                format!("{} {}", cmd.command, cmd.args.join(" "))
            } else {
                cmd.command.clone()
            };

            // For now, simulate command execution with basic output
            // In a real implementation, this would call the forgecode API
            let output = format!("Executing: {}\n", command_str);
            if let Err(e) = tx.send(format!("stdout:{}", output)).await {
                tracing::error!("Failed to send output for connection {}: {}", connection_id, e);
            }

            // Simulate command completion
            let completion_msg = format!("Command '{}' completed successfully\n", cmd.command);
            if let Err(e) = tx.send(format!("stdout:{}", completion_msg)).await {
                tracing::error!("Failed to send completion for connection {}: {}", connection_id, e);
            }

            // Send exit code
            if let Err(e) = tx.send(format!("exit:{}", 0)).await {
                tracing::error!("Failed to send exit code for connection {}: {}", connection_id, e);
            }
        });

        // Handle output streaming
        let mut receiver = Some(rx);
        while let Some(rx) = receiver.take() {
            match rx.recv().await {
                Some(output) => {
                    ctx.text(output);
                    receiver = Some(rx);
                }
                None => break,
            }
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

impl actix::Actor for WebSocketConnection {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        tracing::info!("WebSocket connection established: {}", self.id);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::info!("WebSocket connection closed: {}", self.id);
    }
}

/// Handler for WebSocket messages
impl actix::StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketConnection {
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
                // Handle incoming text messages (terminal input)
                tracing::debug!("Received WebSocket message: {}", text);

                // Parse command from JSON
                match serde_json::from_str::<TerminalCommand>(&text) {
                    Ok(cmd) => {
                        // Execute the command
                        let ctx_clone = ctx.clone();
                        tokio::spawn(async move {
                            self.execute_command(cmd, &mut ctx_clone).await;
                        });
                    }
                    Err(_) => {
                        // If not a valid command, treat as raw terminal input
                        ctx.text(format!("echo:{}", text));
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                // Handle binary messages
                tracing::debug!("Received binary message: {} bytes", bin.len());
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
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
    token_manager: web::Data<Mutex<auth::TokenManager>>,
) -> Result<HttpResponse, Error> {
    // Check authentication token
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            if auth_str.starts_with("Bearer ") {
                let token = &auth_str[7..];
                // Validate token against token manager
                if token_manager.lock().unwrap().validate_token(token) {
                    let resp = ws::start(WebSocketConnection::new(), &req, stream);
                    tracing::info!("WebSocket connection established with Bearer token");
                    return resp;
                }
            }
        }
    }

    // Check URL token parameter
    if let Some(token) = req.uri().query()
        .and_then(|q| url::form_urlencoded::parse(q.as_bytes()).find(|(k, _)| k == "token"))
        .map(|(_, v)| v.into_owned())
    {
        // Validate URL token against token manager
        if token_manager.lock().unwrap().validate_token(&token) {
            let resp = ws::start(WebSocketConnection::new(), &req, stream);
            tracing::info!("WebSocket connection established with URL token");
            return resp;
        }
    }

    tracing::warn!("WebSocket connection rejected: No valid authentication");
    Err(actix_web::error::ErrorUnauthorized("Authentication required for WebSocket connection"))
}