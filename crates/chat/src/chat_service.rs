use async_broadcast::{broadcast, Sender, Receiver};
use std::path::PathBuf;
use std::sync::Arc;

use crate::deltachat_bridge::DeltachatBridge;
use crate::error::Result;
use crate::protocol::ChatEvent;

/// Chat chat with event broadcasting
pub struct ChatService {
    _bridge: Arc<DeltachatBridge>,
    #[cfg(test)]
    pub(crate) event_tx: Sender<ChatEvent>,
    #[cfg(not(test))]
    event_tx: Sender<ChatEvent>,
    _initial_rx: Receiver<ChatEvent>,  // Keep channel alive
}

impl ChatService {
    /// Create new chat chat
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        let bridge = DeltachatBridge::init(db_path).await?;
        let (event_tx, initial_rx) = broadcast(1024);

        Ok(Self {
            _bridge: Arc::new(bridge),
            event_tx,
            _initial_rx: initial_rx,
        })
    }

    /// Subscribe to chat events
    pub fn subscribe_events(&self) -> Receiver<ChatEvent> {
        self.event_tx.new_receiver()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[smol_potat::test]
    async fn test_chat_chat_creation() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("chat.db");

        let chat = ChatService::new(db_path).await;
        assert!(chat.is_ok());
    }

    #[smol_potat::test]
    async fn test_event_subscription() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("chat.db");

        let chat = ChatService::new(db_path).await.unwrap();
        let mut rx = chat.subscribe_events();

        // Send test event
        let test_event = ChatEvent::IncomingMessage {
            chat_id: 1,
            msg_id: 42,
        };

        chat.event_tx.broadcast(test_event.clone()).await.unwrap();

        // Receive event
        let received = rx.recv().await.unwrap();
        assert_eq!(received, test_event);
    }
}
