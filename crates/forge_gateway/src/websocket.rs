//! WebSocket handlers for real-time communication

use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use forge_api::API;
use forge_stream::MpscStream;
use std::sync::Arc;

/// WebSocket actor for real-time conversation streaming
pub struct ConversationWebSocket {
    api: Arc<API>,
}

impl ConversationWebSocket {
    /// Create a new WebSocket actor
    pub fn new(api: Arc<API>) -> Self {
        Self { api }
    }
}

impl actix::Actor for ConversationWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        log::info!("Conversation WebSocket connection established");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        log::info!("Conversation WebSocket connection closed");
    }
}

/// WebSocket message handler
impl actix::StreamHandler<Result<ws::Message, ws::ProtocolError>> for ConversationWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // Handle incoming conversation messages
                log::debug!("Received WebSocket message: {}", text);

                // Parse the message as JSON
                match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(data) => {
                        if let (Some(message_type), Some(message_content)) = (
                            data.get("type").and_then(|v| v.as_str()),
                            data.get("message").and_then(|v| v.as_str()),
                        ) {
                            match message_type {
                                "conversation_message" => {
                                    let conversation_id = data.get("conversation_id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("default");

                                    // Process the conversation message using ForgeCode API
                                    let api = self.api.clone();
                                    let message = message_content.to_string();
                                    let conversation_id = conversation_id.to_string();

                                    // Spawn async task to handle the conversation
                                    actix::spawn(async move {
                                        match api.chat(&message, Some(&conversation_id)).await {
                                            Ok(response) => {
                                                // Send the response back via WebSocket
                                                let response_data = serde_json::json!({
                                                    "type": "conversation_response",
                                                    "message": response.text,
                                                    "conversation_id": response.conversation_id.unwrap_or(conversation_id)
                                                });
                                                ctx.text(response_data.to_string());
                                            }
                                            Err(e) => {
                                                log::error!("Failed to process conversation: {}", e);
                                                let error_data = serde_json::json!({
                                                    "type": "error",
                                                    "message": format!("Failed to process conversation: {}", e)
                                                });
                                                ctx.text(error_data.to_string());
                                            }
                                        }
                                    });
                                }
                                _ => {
                                    // Echo other message types back
                                    ctx.text(text);
                                }
                            }
                        } else {
                            // Echo malformed messages back
                            ctx.text(text);
                        }
                    }
                    Err(_) => {
                        // Echo non-JSON messages back
                        ctx.text(text);
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                log::debug!("Received binary message: {} bytes", bin.len());
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Ping(data)) => {
                ctx.pong(&data);
            }
            Ok(ws::Message::Pong(_)) => {
                // Ignore pong messages
            }
            Err(e) => {
                log::error!("WebSocket error: {}", e);
                ctx.stop();
            }
            _ => (),
        }
    }
}

/// WebSocket handler for command execution streaming
pub struct CommandWebSocket {
    api: Arc<API>,
}

impl CommandWebSocket {
    /// Create a new command WebSocket actor
    pub fn new(api: Arc<API>) -> Self {
        Self { api }
    }
}

impl actix::Actor for CommandWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        log::info!("Command WebSocket connection established");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        log::info!("Command WebSocket connection closed");
    }
}

/// WebSocket message handler for commands
impl actix::StreamHandler<Result<ws::Message, ws::ProtocolError>> for CommandWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // Handle incoming command execution requests
                log::debug!("Received command WebSocket message: {}", text);

                // Parse the message as JSON
                match serde_json::from_str::<serde_json::Value>(&text) {
                    Ok(data) => {
                        if let (Some(message_type), Some(command)) = (
                            data.get("type").and_then(|v| v.as_str()),
                            data.get("command").and_then(|v| v.as_str()),
                        ) {
                            match message_type {
                                "execute_command" => {
                                    let working_dir = data.get("working_dir")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("/");

                                    // Execute the command using ForgeCode API
                                    let api = self.api.clone();
                                    let command_str = command.to_string();
                                    let working_dir_str = working_dir.to_string();

                                    // Spawn async task to handle command execution
                                    actix::spawn(async move {
                                        match api.execute_shell_command(&command_str, std::path::PathBuf::from(&working_dir_str)).await {
                                            Ok(output) => {
                                                // Send the command output back via WebSocket
                                                let response_data = serde_json::json!({
                                                    "type": "command_output",
                                                    "output": output.stdout,
                                                    "exit_code": output.exit_code,
                                                    "success": output.exit_code == 0
                                                });
                                                ctx.text(response_data.to_string());
                                            }
                                            Err(e) => {
                                                log::error!("Failed to execute command: {}", e);
                                                let error_data = serde_json::json!({
                                                    "type": "error",
                                                    "message": format!("Failed to execute command: {}", e)
                                                });
                                                ctx.text(error_data.to_string());
                                            }
                                        }
                                    });
                                }
                                _ => {
                                    // Echo other message types back
                                    ctx.text(text);
                                }
                            }
                        } else {
                            // Echo malformed messages back
                            ctx.text(text);
                        }
                    }
                    Err(_) => {
                        // Echo non-JSON messages back
                        ctx.text(text);
                    }
                }
            }
            Ok(ws::Message::Binary(bin)) => {
                log::debug!("Received binary command message: {} bytes", bin.len());
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Ping(data)) => {
                ctx.pong(&data);
            }
            Ok(ws::Message::Pong(_)) => {
                // Ignore pong messages
            }
            Err(e) => {
                log::error!("Command WebSocket error: {}", e);
                ctx.stop();
            }
            _ => (),
        }
    }
}

/// WebSocket handler for conversation streaming
pub async fn conversation(
    req: HttpRequest,
    stream: web::Payload,
    api: web::Data<API>,
) -> Result<HttpResponse, actix_web::Error> {
    let api = Arc::new(api.get_ref().clone());
    let ws = ConversationWebSocket::new(api);

    ws::start(ws, &req, stream)
}

/// WebSocket handler for command execution streaming
pub async fn command(
    req: HttpRequest,
    stream: web::Payload,
    api: web::Data<API>,
) -> Result<HttpResponse, actix_web::Error> {
    let api = Arc::new(api.get_ref().clone());
    let ws = CommandWebSocket::new(api);

    ws::start(ws, &req, stream)
}

/// WebSocket handler struct
pub struct WebSocketHandler;

impl WebSocketHandler {
    /// Conversation WebSocket endpoint
    pub async fn conversation(
        req: HttpRequest,
        stream: web::Payload,
        api: web::Data<API>,
    ) -> Result<HttpResponse, actix_web::Error> {
        conversation(req, stream, api).await
    }

    /// Command execution WebSocket endpoint
    pub async fn command(
        req: HttpRequest,
        stream: web::Payload,
        api: web::Data<API>,
    ) -> Result<HttpResponse, actix_web::Error> {
        command(req, stream, api).await
    }
}