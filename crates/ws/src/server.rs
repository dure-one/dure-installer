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

#[cfg(feature = "chat-service")]
impl WsServer {
    /// Attach chat chat
    pub fn with_chat_chat(mut self, _chat: chat::ChatService) -> Self {
        // TODO: Store chat in WsServer once we add WebSocket handler
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

    #[cfg(feature = "chat-service")]
    #[smol_potat::test]
    async fn test_server_with_chat_chat() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("chat.db");

        let chat = chat::ChatService::new(db_path).await.unwrap();
        let config = ServerConfig::new("example.com");
        let server = WsServer::new(config).with_chat_chat(chat);

        assert_eq!(server.config.domain, "example.com");
    }
}
