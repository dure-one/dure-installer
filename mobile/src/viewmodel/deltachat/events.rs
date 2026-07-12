//! DeltaChat actor events

#[derive(Debug, Clone)]
pub struct ContactInfo {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub is_blocked: bool,
}

#[derive(Debug, Clone)]
pub struct ChatInfo {
    pub id: u32,
    pub name: String,
    pub last_message: Option<String>,
    pub unread_count: u32,
    pub timestamp: i64,
}

#[derive(Debug, Clone)]
pub struct MessageInfo {
    pub id: u32,
    pub from_contact_id: u32,
    pub from_name: String,
    pub text: String,
    pub timestamp: i64,
    pub is_outgoing: bool,
    pub is_seen: bool,
}

#[derive(Debug, Clone)]
pub enum DeltaChatEvent {
    // Configuration
    ConfigurationStarted,
    ConfigurationProgress {
        progress: i32,
        comment: Option<String>,
    },
    Configured {
        email: String,
    },
    ConfigurationFailed {
        error: String,
    },

    // Connection
    Connected,
    Disconnected,
    ConnectionStatus {
        connected: bool,
        email: Option<String>,
    },

    // Contacts
    ContactAdded {
        contact: ContactInfo,
    },
    ContactsListed {
        contacts: Vec<ContactInfo>,
    },
    ContactInfo {
        contact: ContactInfo,
    },

    // Chats
    ChatCreated {
        chat: ChatInfo,
    },
    ChatsListed {
        chats: Vec<ChatInfo>,
    },
    ChatSelected {
        chat_id: u32,
    },

    // Messages
    MessageSent {
        msg_id: u32,
        chat_id: u32,
    },
    MessagesListed {
        chat_id: u32,
        messages: Vec<MessageInfo>,
    },
    NewMessageReceived {
        chat_id: u32,
        message: MessageInfo,
    },
    MessagesSeen {
        chat_id: u32,
    },

    // Progress & Errors
    Progress {
        operation: String,
        progress: f32,
    },
    Error {
        operation: String,
        error: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contact_info() {
        let contact = ContactInfo {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            is_blocked: false,
        };

        assert_eq!(contact.id, 1);
        assert_eq!(contact.name, "Alice");
        assert_eq!(contact.email, "alice@example.com");
        assert!(!contact.is_blocked);
    }

    #[test]
    fn test_contact_info_clone() {
        let contact = ContactInfo {
            id: 1,
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
            is_blocked: false,
        };

        let cloned = contact.clone();
        assert_eq!(cloned.id, contact.id);
        assert_eq!(cloned.email, contact.email);
    }

    #[test]
    fn test_message_info() {
        let msg = MessageInfo {
            id: 42,
            from_contact_id: 1,
            from_name: "Bob".to_string(),
            text: "Hello!".to_string(),
            timestamp: 1234567890,
            is_outgoing: false,
            is_seen: false,
        };

        assert_eq!(msg.id, 42);
        assert_eq!(msg.text, "Hello!");
        assert!(!msg.is_outgoing);
    }

    #[test]
    fn test_configured_event() {
        let event = DeltaChatEvent::Configured {
            email: "user@example.com".to_string(),
        };

        match event {
            DeltaChatEvent::Configured { email } => {
                assert_eq!(email, "user@example.com");
            }
            _ => panic!("Expected Configured event"),
        }
    }
}
