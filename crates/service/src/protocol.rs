use serde::{Deserialize, Serialize};

/// Chat events from deltachat
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ChatEvent {
    IncomingMessage {
        chat_id: u32,
        msg_id: u32,
    },
    MessageRead {
        chat_id: u32,
        msg_id: u32,
    },
    ChatModified {
        chat_id: u32,
    },
}

/// Chat information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Chat {
    pub id: u32,
    pub name: String,
    pub is_group: bool,
    pub unread_count: usize,
}

/// Message information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: u32,
    pub chat_id: u32,
    pub from_id: u32,
    pub text: String,
    pub timestamp: i64,
    pub is_outgoing: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_event_serialization() {
        let event = ChatEvent::IncomingMessage {
            chat_id: 1,
            msg_id: 42,
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: ChatEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_chat_creation() {
        let chat = Chat {
            id: 1,
            name: "Test Chat".into(),
            is_group: false,
            unread_count: 5,
        };

        assert_eq!(chat.id, 1);
        assert_eq!(chat.name, "Test Chat");
    }

    #[test]
    fn test_message_creation() {
        let msg = Message {
            id: 1,
            chat_id: 1,
            from_id: 42,
            text: "Hello".into(),
            timestamp: 1234567890,
            is_outgoing: true,
        };

        assert_eq!(msg.text, "Hello");
        assert!(msg.is_outgoing);
    }
}
