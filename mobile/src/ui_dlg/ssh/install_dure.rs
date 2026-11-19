//! Install Dure wizard dialog
//!
//! Multi-step wizard for installing dure-mycart to remote hosts

use super::install_dure_stt::*;
use crate::{dure_debug, dure_error, dure_info};
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

    pub fn with_domain(mut self, domain: Option<String>) -> Self {
        if let Some(ref d) = domain {
            dure_info!("[InstallDure] Setting replacement_domain: {}", d);
        } else {
            dure_info!("[InstallDure] No domain provided (replacement_domain = None)");
        }
        self.state.replacement_domain = domain;
        self
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

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        mut vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        dure_debug!(
            "show() called: step={:?}, vm={}",
            self.state.step,
            if vm.is_some() { "Some" } else { "None" }
        );

        // Poll promises
        self.state.poll_fetch_tags();
        self.state.poll_download_env();

        // Poll EAB key fetch
        if let Some(result) = self.state.poll_fetch_eab() {
            match result {
                Ok((key_id, hmac_key)) => {
                    dure_info!(
                        "[InstallDure] Received EAB keys: key_id={}, hmac_key_len={}",
                        key_id,
                        hmac_key.len()
                    );
                    self.update_env_fields_for_gts(&key_id, &hmac_key);
                }
                Err(e) => {
                    dure_error!("[InstallDure] Failed to fetch EAB keys: {}", e);
                    self.state.step = InstallStep::Error(format!("Failed to fetch EAB keys: {}", e));
                }
            }
        }

        // Trigger EAB fetch when provider changes to GTS (and we have credentials)
        if self.state.acme_provider == super::install_dure_stt::AcmeProvider::GoogleTrustServices
            && self.state.fetch_eab_promise.is_none()
            && !self.state.eab_fetch_attempted
            && self.state.gcp_project_id.is_some()
            && self.state.gcp_access_token.is_some()
        {
            let project_id = self.state.gcp_project_id.clone().unwrap();
            let access_token = self.state.gcp_access_token.clone().unwrap();
            dure_info!(
                "[InstallDure] Triggering EAB key fetch for project: {}",
                project_id
            );
            self.state.start_fetch_eab_keys(project_id, access_token);
        }

        // Always poll events (not just when Installing) to avoid missing completion event
        if let Some(ref mut vm_ref) = vm {
            let events = vm_ref.poll_events(ctx);
            dure_debug!(
                "Polling events: got {} events, current step={:?}",
                events.len(),
                self.state.step
            );

            for event in events {
                if let crate::viewmodel::ViewModelEvent::Ssh(ssh_event) = event {
                    if let crate::viewmodel::ssh::SshEvent::DureInstallCompleted {
                        host,
                        success,
                        error,
                        progress,
                        ..
                    } = ssh_event
                    {
                        dure_info!(
                            "[UiDlg::Ins] Dialog received DureInstallCompleted: host={}, success={}, current_step={:?}",
                            host,
                            success,
                            self.state.step
                        );

                        // Only process if we're the target host and currently installing
                        let host_matches = host == self.state.host;
                        let is_installing = matches!(self.state.step, InstallStep::Installing);

                        dure_info!(
                            "[UiDlg::Ins] Match check: host_matches={} ('{}' vs '{}'), is_installing={}",
                            host_matches,
                            host,
                            self.state.host,
                            is_installing
                        );

                        if host_matches && is_installing {
                            dure_info!("[UiDlg::Ins] Processing event - transitioning state");

                            dure_debug!(
                                "[UiDlg::Ins] Before update - progress_log has {} entries",
                                self.state.progress_log.len()
                            );

                            // Update progress log with actual installation progress
                            self.state.progress_log = progress;
                            ctx.request_repaint(); // Force UI update after progress change

                            dure_info!(
                                "[UiDlg::Ins] After update - progress_log has {} entries",
                                self.state.progress_log.len()
                            );
                            if !self.state.progress_log.is_empty() {
                                dure_debug!(
                                    "[UiDlg::Ins] Last progress message: {}",
                                    self.state.progress_log.last().unwrap()
                                );
                            }

                            if success {
                                self.state.step = InstallStep::Complete;
                                self.state
                                    .progress_log
                                    .push("✓ Installation completed successfully".to_string());
                                dure_info!("[UiDlg::Ins] Transitioned to Complete state");
                                ctx.request_repaint();
                            } else {
                                let err_msg = error
                                    .unwrap_or_else(|| "Installation failed".to_string());
                                self.state.step = InstallStep::Error(err_msg.clone());
                                self.state
                                    .progress_log
                                    .push(format!("✗ Error: {}", err_msg));
                                dure_error!("[UiDlg::Ins] Transitioned to Error state: {}", err_msg);
                                ctx.request_repaint();
                            }
                        } else {
                            dure_info!(
                                "[UiDlg::Ins] Skipping event - host_matches={}, is_installing={}",
                                host_matches,
                                is_installing
                            );
                        }
                    }
                }
            }
        }

        let title = format!("Install Dure - {}", self.state.host);
        let mut open = !self.state.should_close;

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog("install_dure_dialog", &title, &mut open)
            .min_width(350.0)
            .max_width(window_width - 40.0)
            .max_height(window_height - 40.0)
            .scrollable(true)
            .content(|ui, _| {
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
            })
            .show(ctx);

        self.state.should_close = !open;

        // Handle deferred install action (after dialog to avoid closure borrow issues)
        if self.state.pending_install {
            self.state.pending_install = false;

            if let Some(ref mut vm_ref) = vm {
                // Pass env_fields directly (preserves comments and order)
                let env_fields = self.state.env_fields.clone();

                dure_info!(
                    "[InstallDure] Sending install command with {} env fields",
                    env_fields.len()
                );
                dure_debug!(
                    "[InstallDure] env_fields keys: {:?}",
                    env_fields.iter().map(|f| &f.key).collect::<Vec<_>>()
                );

                let version = self.state.available_versions[self.state.selected_version.unwrap()]
                    .clone();
                let image_tag = self.state.image_tag.clone();

                match vm_ref.install_dure(
                    self.state.host.clone(),
                    version,
                    env_fields,
                    image_tag,
                    self.profile_keyring.clone(),
                ) {
                    Ok(_) => {
                        self.state.step = InstallStep::Installing;
                        self.state
                            .progress_log
                            .push("Installation started...".to_string());
                        dure_info!("Install Dure command sent to ViewModel");
                    }
                    Err(e) => {
                        self.state.step = InstallStep::Error(format!(
                            "Failed to start installation: {}",
                            e
                        ));
                        dure_error!("Failed to send install command: {}", e);
                    }
                }
            } else {
                self.state.step =
                    InstallStep::Error("ViewModel not available - cannot install".to_string());
                dure_error!("ViewModel not available for install");
            }
        }
    }

    fn render_header(&mut self, ui: &mut egui::Ui) {
        // Show domain if present
        if let Some(domain) = &self.state.replacement_domain {
            dure_debug!("[InstallDure] Rendering domain label: {}", domain);
            ui.horizontal(|ui| {
                ui.label("Domain:");
                ui.strong(domain); // Bold for emphasis
            });
            ui.add_space(4.0);
        } else {
            dure_debug!("[InstallDure] No replacement_domain in state, not showing domain label");
        }

        // Version selector
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

                if ui.add(MaterialButton::filled("Download").small()).clicked() {
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
        // ACME Provider selector
        ui.horizontal(|ui| {
            ui.label("ACME Provider:");

            let provider_text = match self.state.acme_provider {
                super::install_dure_stt::AcmeProvider::LetsEncrypt => "Let's Encrypt",
                super::install_dure_stt::AcmeProvider::GoogleTrustServices => "Google Trust Services",
            };

            egui::ComboBox::from_id_source("acme_provider_select")
                .selected_text(provider_text)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(
                            &mut self.state.acme_provider,
                            super::install_dure_stt::AcmeProvider::LetsEncrypt,
                            "Let's Encrypt",
                        )
                        .changed()
                    {
                        self.update_env_fields_for_provider();
                    }
                    if ui
                        .selectable_value(
                            &mut self.state.acme_provider,
                            super::install_dure_stt::AcmeProvider::GoogleTrustServices,
                            "Google Trust Services",
                        )
                        .changed()
                    {
                        self.update_env_fields_for_provider();
                    }
                });
        });

        ui.add_space(8.0);

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


        });

        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(600.0)
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
            // Render all comment lines above the field
            if !field.comments.is_empty() {
                for comment in &field.comments {
                    if comment.is_empty() {
                        // Empty comment line → add spacing
                        ui.add_space(4.0);
                    } else if comment.contains("====") || comment.contains("----") {
                        // Section header → use stronger style
                        ui.strong(comment);
                    } else {
                        // Regular comment
                        ui.label(comment);
                    }
                }
                ui.add_space(4.0);
            }

            // Render input field
            let input = MaterialInput::outlined(&mut field.value).label(&field.key);
            ui.add(input);
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
            ui.heading("Installation Complete");
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
            ui.heading("Error");
            ui.add_space(20.0);
            ui.colored_label(egui::Color32::from_rgb(200, 50, 50), error);
        });
    }

    fn render_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Close button (always available, always works)
                if ui.add(MaterialButton::filled(tr!("close"))).clicked() {
                    self.state.should_close = true;
                    ui.ctx().request_repaint();
                }

                // Done button (prominent in Complete state)
                if matches!(self.state.step, InstallStep::Complete) {
                    if ui.add(MaterialButton::filled("Done")).clicked() {
                        self.state.should_close = true;
                        ui.ctx().request_repaint();
                    }
                }

                // Reset button
                if matches!(
                    self.state.step,
                    InstallStep::Complete | InstallStep::Error(_)
                ) {
                    if ui.add(MaterialButton::filled("Reset")).clicked() {
                        self.state.reset();
                        self.state.start_fetch_tags();
                        ui.ctx().request_repaint();
                    }
                }

                // Install button (only in ConfigEdit step)
                if matches!(self.state.step, InstallStep::ConfigEdit) {
                    if ui.add(MaterialButton::filled(tr!("install"))).clicked() {
                        // Sync form to editor before install
                        if self.state.active_tab == 0 {
                            self.state.sync_form_to_editor();
                        }

                        // Set flag to trigger install after dialog (avoids closure borrow issues)
                        self.state.pending_install = true;
                    }
                }
            });
        });
    }

    /// Update env fields when ACME provider changes
    fn update_env_fields_for_provider(&mut self) {
        use super::install_dure_stt::AcmeProvider;

        match self.state.acme_provider {
            AcmeProvider::LetsEncrypt => {
                self.set_env_field("ACME_PROVIDER", "letsencrypt");
                self.set_env_field(
                    "ACME_DIRECTORY_URL",
                    "https://acme-v02.api.letsencrypt.org/directory",
                );
                self.remove_env_field("GTS_EAB_KID");
                self.remove_env_field("GTS_EAB_HMAC_KEY");
            }
            AcmeProvider::GoogleTrustServices => {
                self.set_env_field("ACME_PROVIDER", "google");
                self.set_env_field(
                    "ACME_DIRECTORY_URL",
                    "https://dv.acme-v02.api.pki.goog/directory",
                );
                // Reset fetch flag to allow re-fetch when switching back to GTS
                self.state.eab_fetch_attempted = false;
                // Keys will be filled by update_env_fields_for_gts when fetched
            }
        }
        self.state.sync_form_to_editor();
    }

    /// Update env fields with fetched GTS EAB keys
    fn update_env_fields_for_gts(&mut self, key_id: &str, hmac_key: &str) {
        self.set_env_field("GTS_EAB_KID", key_id);
        self.set_env_field("GTS_EAB_HMAC_KEY", hmac_key);
        self.state.sync_form_to_editor();
    }

    /// Set or update an env field value
    fn set_env_field(&mut self, key: &str, value: &str) {
        if let Some(field) = self.state.env_fields.iter_mut().find(|f| f.key == key) {
            field.value = value.to_string();
        } else {
            self.state
                .env_fields
                .push(crate::calc::env_parser::EnvField::new(key, value));
        }
    }

    /// Remove an env field if it exists
    fn remove_env_field(&mut self, key: &str) {
        self.state.env_fields.retain(|f| f.key != key);
    }
}
