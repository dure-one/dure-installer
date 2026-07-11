use crate::config::ServerConfig;
use crate::error::Result;
use std::sync::Arc;

/// Main WebSocket/HTTP server
pub struct WsServer {
    pub config: Arc<ServerConfig>,
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

#[cfg(feature = "chat")]
impl WsServer {
    /// Attach chat service
    pub fn with_chat_service(mut self, _service: service::ChatService) -> Self {
        // TODO: Store service in WsServer once we add WebSocket handler
        self
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

#[cfg(test)]
mod integration_tests {
    use super::*;
    use tempfile::TempDir;

    #[cfg(feature = "chat")]
    #[smol_potat::test]
    async fn test_server_with_chat_service() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("chat.db");

        let service = service::ChatService::new(db_path).await.unwrap();
        let config = ServerConfig::new("example.com");
        let server = WsServer::new(config).with_chat_service(service);

        assert_eq!(server.config.domain, "example.com");
    }
}
