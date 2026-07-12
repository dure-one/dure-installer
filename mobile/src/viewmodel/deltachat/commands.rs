//! DeltaChat actor commands

#[derive(Debug, Clone)]
pub enum DeltaChatCommand {
    // Configuration & Connection
    Configure {
        email: String,
        password: String,
    },
    Connect,
    Disconnect,
    GetConnectionStatus,

    // Contact Management
    AddContact {
        email: String,
    },
    ListContacts,
    GetContactInfo {
        contact_id: u32,
    },

    // Chat Management
    CreateChat {
        contact_id: u32,
    },
    ListChats,
    SelectChat {
        chat_id: u32,
    },

    // Messaging
    SendTextMessage {
        chat_id: u32,
        text: String,
    },
    ListMessages {
        chat_id: u32,
    },
    MarkMessagesSeen {
        chat_id: u32,
    },

    // Background sync
    FetchMessages,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_configure_command() {
        let cmd = DeltaChatCommand::Configure {
            email: "test@example.com".to_string(),
            password: "secret".to_string(),
        };

        match cmd {
            DeltaChatCommand::Configure { email, password } => {
                assert_eq!(email, "test@example.com");
                assert_eq!(password, "secret");
            }
            _ => panic!("Expected Configure command"),
        }
    }

    #[test]
    fn test_command_clone() {
        let cmd = DeltaChatCommand::AddContact {
            email: "alice@example.com".to_string(),
        };
        let cloned = cmd.clone();
        assert!(matches!(cloned, DeltaChatCommand::AddContact { .. }));
    }

    #[test]
    fn test_send_message_command() {
        let cmd = DeltaChatCommand::SendTextMessage {
            chat_id: 42,
            text: "Hello!".to_string(),
        };

        match cmd {
            DeltaChatCommand::SendTextMessage { chat_id, text } => {
                assert_eq!(chat_id, 42);
                assert_eq!(text, "Hello!");
            }
            _ => panic!("Expected SendTextMessage command"),
        }
    }
}
