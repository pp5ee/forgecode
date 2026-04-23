//! ForgeCode Web Gateway
//!
//! Provides a web interface for remote access to ForgeCode with URL token authentication.

mod gateway;
mod server;
mod auth;
mod handlers;
mod websocket;
mod monitoring;

pub use gateway::*;
pub use server::*;
pub use auth::*;
pub use handlers::*;
pub use websocket::*;
pub use monitoring::*;

#[cfg(test)]
mod tests;