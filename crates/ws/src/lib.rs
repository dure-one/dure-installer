//! # ws - WebSocket and HTTP/2 Server
//!
//! High-performance WebSocket and HTTP/2 server built on wtx + smol.
//!
//! ## Features
//!
//! - HTTP/2 server with wtx (runtime-agnostic)
//! - WebSocket server (RFC 6455)
//! - TLS support via rustls
//! - Static file serving
//! - Middleware chain (CORS, compression, sessions)
//! - DDoS protection via fail2ban-rs
//!
//! ## Example
//!
//! ```rust,ignore
//! use ws::{ServerConfig, WsServer};
//!
//! smol::block_on(async {
//!     let config = ServerConfig::new("example.com");
//!     WsServer::new(config).run().await.unwrap();
//! });
//! ```

#[cfg(feature = "chat")]
pub use service;

pub mod error;
pub mod config;
pub mod tls;
pub mod server;
pub mod static_files;
pub mod http;

pub use error::{WsError, Result};
pub use config::ServerConfig;
pub use server::WsServer;

#[cfg(test)]
mod tests {
    #[test]
    fn test_crate_compiles() {
        assert!(true);
    }
}
