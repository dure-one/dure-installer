//! WebSocket and HTTP/2 server built on wtx + smol
//!
//! Provides high-performance WebSocket and HTTP/2 server with:
//! - TLS support via rustls
//! - DDoS protection via fail2ban-rs
//! - Middleware chain (CORS, compression, sessions)
//! - Static file serving

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
