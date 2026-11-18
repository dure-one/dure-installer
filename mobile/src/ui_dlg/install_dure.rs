//! Install Dure wizard dialog
//!
//! Multi-step wizard for installing dure-mycart to remote hosts

use super::install_dure_stt::*;
use crate::{dure_debug, dure_info};
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{dialog, select, MaterialButton, MaterialInput, SelectVariant};

/// Install Dure wizard dialog
pub struct InstallDure {
    pub state: InstallDureState,
    profile_keyring: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
}

impl InstallDure {
    pub fn new(host: impl Into<String>) -> Self {
        let mut state = InstallDureState::new(host);
        state.start_fetch_tags();

        Self {
            state,
            profile_keyring: None,
        }
    }

    pub fn with_keyring(
        mut self,
        keyring: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> Self {
        self.profile_keyring = keyring;
        self
    }

    pub fn should_close(&self) -> bool {
        self.state.should_close
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        // Poll promises
        self.state.poll_fetch_tags();
        self.state.poll_download_env();
        self.state.poll_install();

        let title = format!("Install Dure - {}", self.state.host);
        let mut open = !self.state.should_close;

        dialog("install_dure_dialog", &title, &mut open)
            .content(|ui, _| {
                ui.set_width(900.0);
                ui.vertical(|ui| {
                    self.render_header(ui);
                    ui.add_space(16.0);

                    match self.state.step.clone() {
                        InstallStep::VersionSelect => self.render_version_select(ui),
                        InstallStep::ConfigEdit => self.render_config_edit(ui),
                        InstallStep::Installing => self.render_installing(ui),
                        InstallStep::Complete => self.render_complete(ui),
                        InstallStep::Error(e) => self.render_error(ui, &e),
                    }

                    ui.add_space(16.0);
                    self.render_buttons(ui);
                });
            })
            .show(ctx);

        self.state.should_close = !open;
    }

    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Version:");

            let selected_text = self
                .state
                .selected_version
                .and_then(|idx| self.state.available_versions.get(idx))
                .map(|s| s.as_str())
                .unwrap_or("Select version");

            let options: Vec<&str> = self
                .state
                .available_versions
                .iter()
                .map(String::as_str)
                .collect();

            if !options.is_empty() {
                egui::ComboBox::from_id_source("version_select")
                    .selected_text(selected_text)
                    .show_ui(ui, |ui| {
                        for (idx, option) in options.iter().enumerate() {
                            ui.selectable_value(
                                &mut self.state.selected_version,
                                Some(idx),
                                *option,
                            );
                        }
                    });

                if ui
                    .add(MaterialButton::filled(tr!("ssh-button-start")).small())
                    .clicked()
                {
                    if self.state.selected_version.is_some() {
                        self.state.start_download_env();
                    }
                }
            } else {
                ui.spinner();
                ui.label("Loading versions...");
            }
        });
    }

    fn render_version_select(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Select Dure Version");
            ui.add_space(20.0);

            if self.state.tags_fetched {
                ui.label("Select a version above and click Start");
            } else {
                ui.spinner();
                ui.label("Fetching available versions from GitHub...");
            }
        });
    }

    fn render_config_edit(&mut self, ui: &mut egui::Ui) {
        // Tabs for Form / Code Editor
        let tab_labels = ["Input Form", "Code Editor"];

        ui.horizontal(|ui| {
            for (idx, &label) in tab_labels.iter().enumerate() {
                let is_selected = self.state.active_tab == idx;
                let button = if is_selected {
                    MaterialButton::filled(label)
                } else {
                    MaterialButton::outlined(label)
                };

                if ui.add(button.small()).clicked() {
                    // Sync before switching
                    if self.state.active_tab == 1 {
                        self.state.sync_editor_to_form();
                    } else {
                        self.state.sync_form_to_editor();
                    }
                    self.state.active_tab = idx;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label("Image Tag:");
                let image_options = ["latest", "test", "dev"];
                let mut selected_idx = image_options
                    .iter()
                    .position(|&s| s == self.state.image_tag.as_str())
                    .unwrap_or(0);

                egui::ComboBox::from_id_source("image_tag_select")
                    .selected_text(self.state.image_tag.as_str())
                    .show_ui(ui, |ui| {
                        for &tag in &image_options {
                            ui.selectable_value(&mut self.state.image_tag, tag.to_string(), tag);
                        }
                    });
            });
        });

        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(450.0)
            .show(ui, |ui| {
                if self.state.active_tab == 0 {
                    self.render_form_tab(ui);
                } else {
                    self.render_editor_tab(ui);
                }
            });
    }

    fn render_form_tab(&mut self, ui: &mut egui::Ui) {
        for field in &mut self.state.env_fields {
            ui.horizontal(|ui| {
                ui.label(&field.key);
                if let Some(comment) = &field.comment {
                    ui.label(egui::RichText::new(comment).weak());
                }
            });

            ui.add_sized(
                [ui.available_width() - 20.0, 40.0],
                MaterialInput::outlined(&mut field.value).placeholder(&field.key),
            );
            ui.add_space(8.0);
        }

        if self.state.env_fields.is_empty() {
            ui.label("No configuration fields found");
        }
    }

    fn render_editor_tab(&mut self, ui: &mut egui::Ui) {
        ui.add(
            egui::TextEdit::multiline(&mut self.state.env_editor_text)
                .code_editor()
                .desired_width(ui.available_width() - 20.0)
                .desired_rows(25),
        );
    }

    fn render_installing(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Installing...");
            ui.add_space(20.0);
            ui.spinner();
            ui.add_space(20.0);

            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for log_line in &self.state.progress_log {
                        ui.label(log_line);
                    }
                });
        });
    }

    fn render_complete(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("✓ Installation Complete");
            ui.add_space(20.0);

            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for log_line in &self.state.progress_log {
                        ui.label(log_line);
                    }
                });
        });
    }

    fn render_error(&mut self, ui: &mut egui::Ui, error: &str) {
        ui.vertical_centered(|ui| {
            ui.heading("⚠ Error");
            ui.add_space(20.0);
            ui.colored_label(egui::Color32::from_rgb(200, 50, 50), error);
        });
    }

    fn render_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Close button (always available)
                if ui.add(MaterialButton::text(tr!("close"))).clicked() {
                    self.state.should_close = true;
                }

                // Reset button
                if matches!(
                    self.state.step,
                    InstallStep::Complete | InstallStep::Error(_)
                ) {
                    if ui.add(MaterialButton::outlined(tr!("reset"))).clicked() {
                        self.state.reset();
                        self.state.start_fetch_tags();
                    }
                }

                // Install button (only in ConfigEdit step)
                if matches!(self.state.step, InstallStep::ConfigEdit) {
                    if ui.add(MaterialButton::filled(tr!("install"))).clicked() {
                        // Sync form to editor before install
                        if self.state.active_tab == 0 {
                            self.state.sync_form_to_editor();
                        }

                        // Create host config from host string
                        let host_config = crate::config::SshHostConfig {
                            host: self.state.host.clone(),
                            password: None,
                            private_key_path: None,
                            keyring_domain: None,
                            port: 22,
                            initialized: false,
                            last_status: None,
                            platform_name: None,
                            docker_containers: Vec::new(),
                            ansible_roles: Vec::new(),
                            dure_wss_config: None,
                            last_ssh_check: None,
                            last_host_check: None,
                            last_docker_check: None,
                            last_dure_check: None,
                        };

                        self.state
                            .start_install(host_config, self.profile_keyring.clone());
                    }
                }
            });
        });
    }
}
