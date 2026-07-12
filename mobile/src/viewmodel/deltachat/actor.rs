//! DeltaChat actor implementation

use crate::viewmodel::common::ViewModelEvent;
use super::{ChatInfo, ContactInfo, DeltaChatCommand, DeltaChatEvent, MessageInfo};
use smol::channel::{Receiver, Sender};
use std::path::PathBuf;

pub struct DeltaChatActor {
    command_rx: Receiver<DeltaChatCommand>,
    event_tx: Sender<ViewModelEvent>,
    context: Option<deltachat::Context>,
    tokio_runtime: tokio::runtime::Runtime,
    database_path: PathBuf,
    is_configured: bool,
    is_connected: bool,
}

impl DeltaChatActor {
    pub fn new(
        command_rx: Receiver<DeltaChatCommand>,
        event_tx: Sender<ViewModelEvent>,
        database_path: PathBuf,
    ) -> Self {
        let tokio_runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("deltachat-tokio")
            .build()
            .expect("Failed to create tokio runtime");

        Self {
            command_rx,
            event_tx,
            context: None,
            tokio_runtime,
            database_path,
            is_configured: false,
            is_connected: false,
        }
    }

    pub async fn run(mut self) {
        log::info!("DeltaChatActor started");

        while let Ok(cmd) = self.command_rx.recv().await {
            log::debug!("DeltaChatActor received command: {:?}", cmd);

            // Handle command (moves into tokio runtime)
            if let Err(e) = self.handle_command(cmd).await {
                log::error!("DeltaChat command failed: {}", e);
            }
        }

        log::info!("DeltaChatActor stopped");
    }

    async fn initialize_context(&mut self) -> Result<(), String> {
        let db_path = self.database_path.clone();

        // Ensure database directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Cannot create database directory: {}", e))?;
        }

        // Open/create database in tokio runtime
        let context = self.tokio_runtime.block_on(async {
            deltachat::ContextBuilder::new(db_path)
                .with_id(1)
                .open()
                .await
                .map_err(|e| format!("Cannot open DeltaChat database: {}", e))
        })?;

        self.context = Some(context);
        log::info!("DeltaChat context initialized");

        Ok(())
    }

    async fn configure_internal(&mut self, email: &str, password: &str) -> Result<(), String> {
        // Initialize context if not already done
        if self.context.is_none() {
            self.initialize_context().await?;
        }

        let context = self.context.as_ref().ok_or("Context not initialized")?;

        self.tokio_runtime.block_on(async {
            use deltachat::config::Config;

            // Set configuration
            context.set_config(Config::Addr, Some(email))
                .await
                .map_err(|e| format!("Failed to set email: {}", e))?;

            context.set_config(Config::MailPw, Some(password))
                .await
                .map_err(|e| format!("Failed to set password: {}", e))?;

            // Run configuration
            context.configure()
                .await
                .map_err(|e| format!("Configuration failed: {}", e))?;

            Ok::<(), String>(())
        })
    }

    async fn handle_command(&mut self, cmd: DeltaChatCommand) -> Result<(), String> {
        // Wrap tokio calls with runtime
        let result = match cmd {
            DeltaChatCommand::Configure { ref email, ref password } => {
                self.emit_event(DeltaChatEvent::ConfigurationStarted);
                self.configure_internal(email, password).await
            }
            DeltaChatCommand::Connect => {
                if let Some(context) = &self.context {
                    self.tokio_runtime.block_on(async {
                        context.start_io().await;
                    });
                    self.is_connected = true;
                    self.emit_event(DeltaChatEvent::Connected);
                    log::info!("DeltaChat connected");
                } else {
                    let error = "Cannot connect: not configured".to_string();
                    self.emit_event(DeltaChatEvent::Error {
                        operation: "connect".to_string(),
                        error: error.clone(),
                    });
                    return Err(error);
                }
                Ok(())
            }
            DeltaChatCommand::Disconnect => {
                if let Some(context) = &self.context {
                    self.tokio_runtime.block_on(async {
                        context.stop_io().await;
                    });
                    self.is_connected = false;
                    self.emit_event(DeltaChatEvent::Disconnected);
                    log::info!("DeltaChat disconnected");
                }
                Ok(())
            }
            DeltaChatCommand::GetConnectionStatus => {
                self.emit_event(DeltaChatEvent::ConnectionStatus {
                    connected: self.is_connected,
                    email: if self.is_configured {
                        // Get email from context config
                        if let Some(context) = &self.context {
                            self.tokio_runtime.block_on(async {
                                use deltachat::config::Config;
                                context.get_config(Config::Addr).await.ok().flatten()
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    },
                });
                Ok(())
            }
            DeltaChatCommand::AddContact { ref email } => {
                if let Some(context) = &self.context {
                    let contact_id = self.tokio_runtime.block_on(async {
                        deltachat::contact::Contact::create(&context, "", email)
                            .await
                            .map_err(|e| format!("Failed to add contact: {}", e))
                    })?;

                    let contact = self.tokio_runtime.block_on(async {
                        deltachat::contact::Contact::get_by_id(&context, contact_id)
                            .await
                            .map_err(|e| format!("Failed to get contact: {}", e))
                    })?;

                    let contact_info = ContactInfo {
                        id: contact_id.to_u32(),
                        name: contact.get_display_name().to_string(),
                        email: contact.get_addr().to_string(),
                        is_blocked: contact.is_blocked(),
                    };

                    self.emit_event(DeltaChatEvent::ContactAdded {
                        contact: contact_info
                    });
                    log::info!("Contact added: {}", email);
                } else {
                    return Err("Cannot add contact: not configured".to_string());
                }
                Ok(())
            }
            DeltaChatCommand::ListContacts => {
                if let Some(context) = &self.context {
                    let contacts = self.tokio_runtime.block_on(async {
                        let contact_ids = deltachat::contact::Contact::get_all(
                            &context,
                            0, // flags
                            None, // query
                        ).await.map_err(|e| format!("Failed to list contacts: {}", e))?;

                        let mut contacts = Vec::new();
                        for contact_id in contact_ids {
                            if let Ok(contact) = deltachat::contact::Contact::get_by_id(&context, contact_id).await {
                                contacts.push(ContactInfo {
                                    id: contact_id.to_u32(),
                                    name: contact.get_display_name().to_string(),
                                    email: contact.get_addr().to_string(),
                                    is_blocked: contact.is_blocked(),
                                });
                            }
                        }

                        Ok::<Vec<ContactInfo>, String>(contacts)
                    })?;

                    self.emit_event(DeltaChatEvent::ContactsListed { contacts });
                    log::info!("Listed {} contacts", contacts.len());
                } else {
                    return Err("Cannot list contacts: not configured".to_string());
                }
                Ok(())
            }
            DeltaChatCommand::CreateChat { contact_id } => {
                if let Some(context) = &self.context {
                    let chat_id = self.tokio_runtime.block_on(async {
                        use deltachat::chat::ChatId;
                        use deltachat::contact::ContactId;

                        let contact_id = ContactId::new(contact_id);
                        deltachat::chat::create_by_contact_id(&context, contact_id)
                            .await
                            .map_err(|e| format!("Failed to create chat: {}", e))
                    })?;

                    let chat = self.tokio_runtime.block_on(async {
                        deltachat::chat::Chat::load_from_db(&context, chat_id)
                            .await
                            .map_err(|e| format!("Failed to load chat: {}", e))
                    })?;

                    let chat_info = ChatInfo {
                        id: chat_id.to_u32(),
                        name: chat.get_name().to_string(),
                        last_message: None,
                        unread_count: 0,
                        timestamp: 0,
                    };

                    self.emit_event(DeltaChatEvent::ChatCreated { chat: chat_info });
                    log::info!("Chat created: {}", chat_id.to_u32());
                } else {
                    return Err("Cannot create chat: not configured".to_string());
                }
                Ok(())
            }
            DeltaChatCommand::ListChats => {
                if let Some(context) = &self.context {
                    let chats = self.tokio_runtime.block_on(async {
                        let chat_ids = deltachat::chat::get_chat_ids(&context, 0, None, None)
                            .await
                            .map_err(|e| format!("Failed to list chats: {}", e))?;

                        let mut chats = Vec::new();
                        for chat_id in chat_ids {
                            if let Ok(chat) = deltachat::chat::Chat::load_from_db(&context, chat_id).await {
                                chats.push(ChatInfo {
                                    id: chat_id.to_u32(),
                                    name: chat.get_name().to_string(),
                                    last_message: None,
                                    unread_count: 0,
                                    timestamp: 0,
                                });
                            }
                        }

                        Ok::<Vec<ChatInfo>, String>(chats)
                    })?;

                    self.emit_event(DeltaChatEvent::ChatsListed { chats });
                    log::info!("Listed {} chats", chats.len());
                } else {
                    return Err("Cannot list chats: not configured".to_string());
                }
                Ok(())
            }
            DeltaChatCommand::SelectChat { chat_id } => {
                self.emit_event(DeltaChatEvent::ChatSelected { chat_id });
                log::info!("Chat selected: {}", chat_id);
                Ok(())
            }
            DeltaChatCommand::SendTextMessage { chat_id, ref text } => {
                if let Some(context) = &self.context {
                    let msg_id = self.tokio_runtime.block_on(async {
                        use deltachat::chat::{ChatId, send_text_msg};

                        let chat_id = ChatId::new(chat_id);
                        send_text_msg(&context, chat_id, text.clone())
                            .await
                            .map_err(|e| format!("Failed to send message: {}", e))
                    })?;

                    self.emit_event(DeltaChatEvent::MessageSent {
                        msg_id: msg_id.to_u32(),
                        chat_id,
                    });
                    log::info!("Message sent: {}", msg_id.to_u32());
                } else {
                    return Err("Cannot send message: not configured".to_string());
                }
                Ok(())
            }
            DeltaChatCommand::ListMessages { chat_id } => {
                if let Some(context) = &self.context {
                    let messages = self.tokio_runtime.block_on(async {
                        use deltachat::chat::{ChatId, get_chat_msgs};

                        let chat_id_obj = ChatId::new(chat_id);
                        let msg_ids = get_chat_msgs(&context, chat_id_obj)
                            .await
                            .map_err(|e| format!("Failed to list messages: {}", e))?;

                        let mut messages = Vec::new();
                        for msg_id in msg_ids {
                            if let Ok(msg) = deltachat::message::Message::load_from_db(&context, msg_id).await {
                                let from_id = msg.get_from_id();
                                let from_name = if let Ok(contact) = deltachat::contact::Contact::get_by_id(&context, from_id).await {
                                    contact.get_display_name().to_string()
                                } else {
                                    "Unknown".to_string()
                                };

                                messages.push(MessageInfo {
                                    id: msg_id.to_u32(),
                                    from_contact_id: from_id.to_u32(),
                                    from_name,
                                    text: msg.get_text().to_string(),
                                    timestamp: msg.get_timestamp(),
                                    is_outgoing: msg.is_outgoing(),
                                    is_seen: msg.is_seen(),
                                });
                            }
                        }

                        Ok::<Vec<MessageInfo>, String>(messages)
                    })?;

                    self.emit_event(DeltaChatEvent::MessagesListed { chat_id, messages });
                    log::info!("Listed messages for chat {}", chat_id);
                } else {
                    return Err("Cannot list messages: not configured".to_string());
                }
                Ok(())
            }
            DeltaChatCommand::MarkMessagesSeen { chat_id } => {
                if let Some(context) = &self.context {
                    self.tokio_runtime.block_on(async {
                        use deltachat::chat::ChatId;

                        let chat_id_obj = ChatId::new(chat_id);
                        deltachat::chat::marknoticed_chat(&context, chat_id_obj)
                            .await
                            .map_err(|e| format!("Failed to mark messages as seen: {}", e))
                    })?;

                    self.emit_event(DeltaChatEvent::MessagesSeen { chat_id });
                    log::info!("Messages marked as seen for chat {}", chat_id);
                } else {
                    return Err("Cannot mark messages as seen: not configured".to_string());
                }
                Ok(())
            }
            DeltaChatCommand::FetchMessages => {
                if let Some(context) = &self.context {
                    self.tokio_runtime.block_on(async {
                        context.fetch().await
                            .map_err(|e| format!("Failed to fetch messages: {}", e))
                    })?;
                    log::info!("Fetched new messages from server");
                } else {
                    return Err("Cannot fetch messages: not configured".to_string());
                }
                Ok(())
            }
            DeltaChatCommand::GetContactInfo { .. } => {
                log::warn!("GetContactInfo not yet implemented");
                Ok(())
            }
            _ => {
                log::warn!("Command not yet implemented: {:?}", cmd);
                Ok(())
            }
        };

        match result {
            Ok(_) if matches!(cmd, DeltaChatCommand::Configure { .. }) => {
                if let DeltaChatCommand::Configure { email, .. } = cmd {
                    self.is_configured = true;
                    self.emit_event(DeltaChatEvent::Configured { email });

                    // Auto-connect after successful configuration
                    if let Some(context) = &self.context {
                        self.tokio_runtime.block_on(async {
                            context.start_io().await;
                        });
                        self.is_connected = true;
                        self.emit_event(DeltaChatEvent::Connected);
                    }
                }
                Ok(())
            }
            Err(e) => {
                let error_msg = if e.contains("authentication") || e.contains("login") {
                    "Invalid email or password.".to_string()
                } else if e.contains("network") || e.contains("connection") {
                    "Cannot reach email server.".to_string()
                } else {
                    format!("Configuration failed: {}", e)
                };

                self.emit_event(DeltaChatEvent::ConfigurationFailed {
                    error: error_msg.clone()
                });
                Err(error_msg)
            }
            Ok(_) => Ok(()),
        }
    }

    fn emit_event(&self, event: DeltaChatEvent) {
        let event_tx = self.event_tx.clone();
        smol::block_on(async move {
            let _ = event_tx.send(ViewModelEvent::DeltaChat(event)).await;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use smol::channel::unbounded;
    use std::path::PathBuf;

    #[test]
    fn test_actor_creation() {
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let db_path = PathBuf::from("/tmp/test.db");

        let actor = DeltaChatActor::new(cmd_rx, event_tx, db_path.clone());

        assert_eq!(actor.database_path, db_path);
        assert!(!actor.is_configured);
        assert!(!actor.is_connected);
        assert!(actor.context.is_none());
    }

    #[test]
    fn test_actor_receives_commands() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = unbounded();
            let (event_tx, _event_rx) = unbounded();

            cmd_tx.send(DeltaChatCommand::GetConnectionStatus).await.unwrap();

            let received = cmd_rx.recv().await.unwrap();
            assert!(matches!(received, DeltaChatCommand::GetConnectionStatus));
        });
    }

    #[tokio::test]
    async fn test_initialize_context() {
        use tempfile::TempDir;

        let tmpdir = TempDir::new().unwrap();
        let db_path = tmpdir.path().join("test.db");

        let context = deltachat::ContextBuilder::new(db_path.clone())
            .with_id(1)
            .open()
            .await
            .unwrap();

        assert!(context.is_open().await);
    }

    #[tokio::test]
    async fn test_add_contact() {
        use tempfile::TempDir;

        let tmpdir = TempDir::new().unwrap();
        let db_path = tmpdir.path().join("test.db");

        let context = deltachat::ContextBuilder::new(db_path)
            .with_id(1)
            .open()
            .await
            .unwrap();

        let contact_id = deltachat::contact::Contact::create(
            &context,
            "Test User",
            "test@example.com"
        ).await.unwrap();

        let contact = deltachat::contact::Contact::get_by_id(&context, contact_id)
            .await
            .unwrap();

        assert_eq!(contact.get_addr(), "test@example.com");
        assert_eq!(contact.get_display_name(), "Test User");
    }
}
