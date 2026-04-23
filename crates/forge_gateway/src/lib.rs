//! ForgeCode Web Gateway
//!
//! Provides a web interface for remote access to ForgeCode with URL token authentication.

pub mod gateway;
pub mod server;
pub mod auth;
pub mod handlers;
pub mod websocket;
pub mod monitoring;
pub mod rate_limiting;

pub use gateway::Gateway;
pub use server::*;
pub use auth::*;
pub use handlers::*;
pub use websocket::*;
pub use monitoring::*;
pub use rate_limiting::*;

#[cfg(test)]
mod tests;