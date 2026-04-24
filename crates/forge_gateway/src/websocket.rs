//! WebSocket implementation for real-time terminal communication

use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use std::time::{Duration, Instant};
use uuid::Uuid;

/// WebSocket connection actor
pub struct WebSocketConnection {
    /// Connection ID
    id: Uuid,
    /// Last heartbeat time
    hb: Instant,
}

impl WebSocketConnection {
    /// Create a new WebSocket connection
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            hb: Instant::now(),
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

                // Echo back for testing
                ctx.text(format!("Echo: {}", text));
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
) -> Result<HttpResponse, Error> {
    let resp = ws::start(WebSocketConnection::new(), &req, stream);
    tracing::info!("WebSocket connection attempted");
    resp
}