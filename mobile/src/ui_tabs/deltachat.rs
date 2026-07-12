//! DeltaChat tab - Encrypted messaging

use eframe::egui;
use crate::viewmodel::deltachat::{ChatInfo, ContactInfo, MessageInfo};
use crate::viewmodel::ViewModel;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DeltaChatTab {
    is_configured: bool,
    is_connected: bool,
    configured_email: Option<String>,

    #[cfg_attr(feature = "serde", serde(skip))]
    config_dialog_open: bool,
    config_email: String,
    config_password: String,
    config_in_progress: bool,
    config_progress: i32,
    config_error: Option<String>,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_contact_dialog_open: bool,
    add_contact_email: String,
    contacts: Vec<ContactInfo>,

    chats: Vec<ChatInfo>,
    selected_chat_id: Option<u32>,

    messages: Vec<MessageInfo>,
    compose_text: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    last_fetch: Option<std::time::Instant>,
}

impl Default for DeltaChatTab {
    fn default() -> Self {
        Self {
            is_configured: false,
            is_connected: false,
            configured_email: None,
            config_dialog_open: false,
            config_email: String::new(),
            config_password: String::new(),
            config_in_progress: false,
            config_progress: 0,
            config_error: None,
            add_contact_dialog_open: false,
            add_contact_email: String::new(),
            contacts: Vec::new(),
            chats: Vec::new(),
            selected_chat_id: None,
            messages: Vec::new(),
            compose_text: String::new(),
            last_fetch: None,
        }
    }
}

impl DeltaChatTab {
    pub fn ui(&mut self, ui: &mut egui::Ui, vm: &ViewModel) {
        // Poll for events
        self.handle_events(vm);

        // Auto-refresh messages when connected and chat selected
        if self.is_connected {
            if let Some(chat_id) = self.selected_chat_id {
                let should_fetch = self.last_fetch
                    .map(|last| last.elapsed().as_secs() > 5)
                    .unwrap_or(true);

                if should_fetch {
                    use crate::viewmodel::deltachat::DeltaChatCommand;
                    smol::block_on(async {
                        let _ = vm.deltachat_tx.send(DeltaChatCommand::FetchMessages).await;
                        let _ = vm.deltachat_tx.send(DeltaChatCommand::ListMessages { chat_id }).await;
                    });
                    self.last_fetch = Some(std::time::Instant::now());
                }
            }
        }

        // Show configuration dialog
        if self.config_dialog_open {
            self.render_config_dialog(ui, vm);
        }

        ui.heading("DeltaChat - Encrypted Messaging");

        if !self.is_configured {
            ui.vertical_centered(|ui| {
                ui.label("Configure your email account to start using encrypted messaging.");

                if ui.button("Configure Account").clicked() {
                    self.config_dialog_open = true;
                }
            });
        } else {
            ui.label(format!("Account: {}",
                self.configured_email.as_deref().unwrap_or("Unknown")));

            ui.horizontal(|ui| {
                ui.label(format!("Status: {}",
                    if self.is_connected { "Connected" } else { "Disconnected" }));

                if self.is_connected {
                    if ui.button("Disconnect").clicked() {
                        use crate::viewmodel::deltachat::DeltaChatCommand;
                        smol::block_on(async {
                            let _ = vm.deltachat_tx.send(DeltaChatCommand::Disconnect).await;
                        });
                    }
                } else {
                    if ui.button("Connect").clicked() {
                        use crate::viewmodel::deltachat::DeltaChatCommand;
                        smol::block_on(async {
                            let _ = vm.deltachat_tx.send(DeltaChatCommand::Connect).await;
                        });
                    }
                }

                if ui.button("Reconfigure").clicked() {
                    self.config_dialog_open = true;
                }
            });

            ui.separator();

            // Contacts section
            ui.heading("Contacts");

            if ui.button("Add Contact").clicked() {
                self.add_contact_dialog_open = true;
            }

            // Show add contact dialog
            if self.add_contact_dialog_open {
                self.render_add_contact_dialog(ui, vm);
            }

            // List contacts on first render or when requested
            if self.contacts.is_empty() && self.is_connected {
                use crate::viewmodel::deltachat::DeltaChatCommand;
                smol::block_on(async {
                    let _ = vm.deltachat_tx.send(DeltaChatCommand::ListContacts).await;
                });
            }

            // Display contacts
            egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                for contact in &self.contacts {
                    ui.horizontal(|ui| {
                        ui.label(&contact.name);
                        ui.label(format!("({})", &contact.email));

                        if ui.small_button("Chat").clicked() {
                            use crate::viewmodel::deltachat::DeltaChatCommand;
                            let contact_id = contact.id;
                            smol::block_on(async {
                                let _ = vm.deltachat_tx.send(DeltaChatCommand::CreateChat { contact_id }).await;
                            });
                        }
                    });
                }
            });

            ui.separator();

            // Chats section
            ui.heading("Chats");

            // List chats on first render or when requested
            if self.chats.is_empty() && self.is_connected {
                use crate::viewmodel::deltachat::DeltaChatCommand;
                smol::block_on(async {
                    let _ = vm.deltachat_tx.send(DeltaChatCommand::ListChats).await;
                });
            }

            // Display chats
            egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                for chat in &self.chats {
                    ui.horizontal(|ui| {
                        let is_selected = self.selected_chat_id == Some(chat.id);

                        if ui.selectable_label(is_selected, &chat.name).clicked() {
                            use crate::viewmodel::deltachat::DeltaChatCommand;
                            let chat_id = chat.id;
                            self.selected_chat_id = Some(chat_id);

                            smol::block_on(async {
                                let _ = vm.deltachat_tx.send(DeltaChatCommand::SelectChat { chat_id }).await;
                                let _ = vm.deltachat_tx.send(DeltaChatCommand::ListMessages { chat_id }).await;
                            });
                        }
                    });
                }
            });

            // Messages section
            if let Some(chat_id) = self.selected_chat_id {
                ui.separator();
                ui.heading("Messages");

                // Display messages
                egui::ScrollArea::vertical().max_height(300.0).show(ui, |ui| {
                    for message in &self.messages {
                        ui.horizontal(|ui| {
                            let color = if message.is_outgoing {
                                egui::Color32::LIGHT_BLUE
                            } else {
                                egui::Color32::LIGHT_GRAY
                            };

                            ui.colored_label(color, &message.from_name);
                            ui.label(&message.text);
                        });
                    }
                });

                // Compose message
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.compose_text);

                    let can_send = !self.compose_text.is_empty();

                    if ui.add_enabled(can_send, egui::Button::new("Send")).clicked() {
                        use crate::viewmodel::deltachat::DeltaChatCommand;
                        let text = self.compose_text.clone();

                        smol::block_on(async {
                            let _ = vm.deltachat_tx.send(DeltaChatCommand::SendTextMessage {
                                chat_id,
                                text
                            }).await;
                        });

                        self.compose_text.clear();
                    }
                });
            }
        }
    }

    fn render_add_contact_dialog(&mut self, ui: &mut egui::Ui, vm: &ViewModel) {
        use crate::viewmodel::deltachat::DeltaChatCommand;

        egui::Window::new("Add Contact")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.label("Email Address:");
                    ui.text_edit_singleline(&mut self.add_contact_email);

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.add_contact_dialog_open = false;
                            self.add_contact_email.clear();
                        }

                        let can_submit = !self.add_contact_email.is_empty();

                        if ui.add_enabled(can_submit, egui::Button::new("Add"))
                            .clicked()
                        {
                            let cmd = DeltaChatCommand::AddContact {
                                email: self.add_contact_email.clone(),
                            };

                            smol::block_on(async {
                                let _ = vm.deltachat_tx.send(cmd).await;
                            });

                            self.add_contact_dialog_open = false;
                            self.add_contact_email.clear();
                        }
                    });
                });
            });
    }

    fn handle_events(&mut self, vm: &ViewModel) {
        use crate::viewmodel::common::ViewModelEvent;
        use crate::viewmodel::deltachat::DeltaChatEvent;

        while let Ok(event) = vm.event_rx.try_recv() {
            match event {
                ViewModelEvent::DeltaChat(dc_event) => match dc_event {
                    DeltaChatEvent::ConfigurationFailed { error } => {
                        self.config_in_progress = false;
                        self.config_error = Some(error);
                    }

                    DeltaChatEvent::Configured { email } => {
                        self.is_configured = true;
                        self.configured_email = Some(email);
                        self.config_dialog_open = false;
                        self.config_error = None;
                        self.config_in_progress = false;
                    }

                    DeltaChatEvent::ConfigurationProgress { progress, .. } => {
                        self.config_progress = progress;
                    }

                    DeltaChatEvent::Connected => {
                        self.is_connected = true;
                        log::info!("DeltaChat connected");
                    }

                    DeltaChatEvent::Disconnected => {
                        self.is_connected = false;
                        log::info!("DeltaChat disconnected");
                    }

                    DeltaChatEvent::ConnectionStatus { connected, email } => {
                        self.is_connected = connected;
                        if let Some(email) = email {
                            self.configured_email = Some(email);
                        }
                    }

                    DeltaChatEvent::ContactAdded { contact } => {
                        self.contacts.push(contact);
                        log::info!("Contact added to UI");
                    }

                    DeltaChatEvent::ContactsListed { contacts } => {
                        self.contacts = contacts;
                        log::info!("Contacts list updated: {} contacts", self.contacts.len());
                    }

                    DeltaChatEvent::ChatCreated { chat } => {
                        self.chats.push(chat.clone());
                        self.selected_chat_id = Some(chat.id);
                        log::info!("Chat created and selected");
                    }

                    DeltaChatEvent::ChatsListed { chats } => {
                        self.chats = chats;
                        log::info!("Chats list updated: {} chats", self.chats.len());
                    }

                    DeltaChatEvent::ChatSelected { chat_id } => {
                        self.selected_chat_id = Some(chat_id);
                        log::info!("Chat selected: {}", chat_id);
                    }

                    DeltaChatEvent::MessageSent { msg_id, chat_id } => {
                        log::info!("Message sent: {} in chat {}", msg_id, chat_id);
                        // Request messages refresh
                    }

                    DeltaChatEvent::MessagesListed { chat_id, messages } => {
                        if Some(chat_id) == self.selected_chat_id {
                            self.messages = messages;
                            log::info!("Messages list updated: {} messages", self.messages.len());
                        }
                    }

                    DeltaChatEvent::NewMessageReceived { chat_id, message } => {
                        if Some(chat_id) == self.selected_chat_id {
                            self.messages.push(message);
                        }
                        log::info!("New message received in chat {}", chat_id);
                    }

                    _ => {}
                }
                _ => {}
            }
        }
    }

    fn render_config_dialog(&mut self, ui: &mut egui::Ui, vm: &ViewModel) {
        use crate::viewmodel::deltachat::DeltaChatCommand;

        egui::Window::new("Configure DeltaChat")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.vertical(|ui| {
                    ui.label("Email Address:");
                    ui.text_edit_singleline(&mut self.config_email);

                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.config_password)
                        .password(true));

                    if let Some(error) = &self.config_error {
                        ui.colored_label(egui::Color32::RED, error);
                    }

                    if self.config_in_progress {
                        let progress = self.config_progress as f32 / 1000.0;
                        ui.add(egui::ProgressBar::new(progress).show_percentage());
                        ui.label("Configuring...");
                    }

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() && !self.config_in_progress {
                            self.config_dialog_open = false;
                            self.config_error = None;
                        }

                        let can_submit = !self.config_email.is_empty()
                                      && !self.config_password.is_empty()
                                      && !self.config_in_progress;

                        if ui.add_enabled(can_submit, egui::Button::new("Configure"))
                            .clicked()
                        {
                            self.config_in_progress = true;
                            self.config_error = None;
                            self.config_progress = 0;

                            let cmd = DeltaChatCommand::Configure {
                                email: self.config_email.clone(),
                                password: self.config_password.clone(),
                            };

                            smol::block_on(async {
                                let _ = vm.deltachat_tx.send(cmd).await;
                            });
                        }
                    });
                });
            });
    }
}
