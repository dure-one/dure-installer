//! WebSocket and HTTP/2 server built on wtx + smol
//!
//! Provides high-performance WebSocket and HTTP/2 server with:
//! - TLS support via rustls
//! - DDoS protection via fail2ban-rs
//! - Middleware chain (CORS, compression, sessions)
//! - Static file serving

pub mod error;

pub use error::{WsError, Result};

#[cfg(test)]
mod tests {
    #[test]
    fn test_crate_compiles() {
        assert!(true);
    }
}
