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

                // Echo the message back for now
                ctx.text(text);
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

                // Echo the message back for now
                ctx.text(text);
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