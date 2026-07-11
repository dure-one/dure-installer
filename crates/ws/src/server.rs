use crate::config::ServerConfig;
use crate::error::Result;
use std::sync::Arc;

/// Main WebSocket/HTTP server
pub struct WsServer {
    config: Arc<ServerConfig>,
}

impl WsServer {
    /// Create a new server with configuration
    pub fn new(config: ServerConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Run the server (blocking)
    pub async fn run(self) -> Result<()> {
        todo!("Server main loop not yet implemented")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let config = ServerConfig::new("example.com");
        let server = WsServer::new(config);
        assert_eq!(server.config.domain, "example.com");
    }
}
