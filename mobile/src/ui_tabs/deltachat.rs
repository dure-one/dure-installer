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
            ui.label(format!("Status: {}",
                if self.is_connected { "Connected" } else { "Disconnected" }));
        }
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
