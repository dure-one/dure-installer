pub use super::add_mycart_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{dialog, select, MaterialButton, MaterialInput, SelectVariant};
use std::cell::Cell;

#[cfg(not(target_arch = "wasm32"))]
use crate::config::SshHostConfig;

impl AddMycartDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.site_address.clear();
        self.admin_email.clear();
        self.password.clear();
        self.ssh_host.clear();
        self.platform_id.clear();
        self.status_message.clear();
        self.in_progress = false;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn set_status(&mut self, message: String) {
        self.status_message = message;
    }

    pub fn set_in_progress(&mut self, in_progress: bool) {
        self.in_progress = in_progress;
    }

    /// Show add mycart site dialog, returns action result
    #[cfg(not(target_arch = "wasm32"))]
    pub fn show(&mut self, ctx: &egui::Context, ssh_hosts: &[SshHostConfig]) -> AddMycartAction {
        if !self.open {
            return AddMycartAction::None;
        }

        let add_clicked = Cell::new(false);
        let cancel_clicked = Cell::new(false);

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        // Track selected SSH host index
        let mut selected_ssh_idx: Option<usize> =
            ssh_hosts.iter().position(|h| h.host == self.ssh_host);

        let prev_ssh_host = self.ssh_host.clone();

        dialog(
            "add_mycart_dialog",
            &tr!("mycart-add-title"),
            &mut self.open,
        )
        .min_width(400.0)
        .max_width(window_width - 120.0)
        .max_height(window_height - 120.0)
        .scrollable(true)
        .content(|ui, _open| {
            ui.label(tr!("mycart-add-heading"));
            ui.add_space(12.0);

            // Site Address
            ui.add(
                MaterialInput::filled(&mut self.site_address).label(&tr!("mycart-add-site-label")),
            );
            ui.add_space(8.0);

            // Admin Email
            ui.add(
                MaterialInput::filled(&mut self.admin_email).label(&tr!("mycart-add-email-label")),
            );
            ui.add_space(8.0);

            // Password
            ui.add(
                MaterialInput::filled(&mut self.password)
                    .label(&tr!("mycart-add-password-label"))
                    .password(true),
            );
            ui.add_space(8.0);

            // SSH Host dropdown
            if ssh_hosts.is_empty() {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 193, 7),
                    "No SSH hosts configured. Add one in the SSH tab first.",
                );
            } else {
                let mut select_widget = select(&mut selected_ssh_idx)
                    .variant(SelectVariant::Outlined)
                    .label(&tr!("mycart-add-ssh-label"))
                    .helper_text(&tr!("mycart-add-ssh-placeholder"))
                    .width(ui.available_width() - 40.0);

                // Add each SSH host as an option
                for (idx, host) in ssh_hosts.iter().enumerate() {
                    select_widget = select_widget.option(idx, &host.host);
                }

                ui.add(select_widget);

                // Update ssh_host from selection
                if let Some(idx) = selected_ssh_idx {
                    if idx < ssh_hosts.len() {
                        self.ssh_host = ssh_hosts[idx].host.clone();
                    }
                }
            }

            ui.add_space(8.0);

            // Auto-fill platform_id when ssh_host changes
            if self.ssh_host != prev_ssh_host && !self.ssh_host.is_empty() {
                if let Some(selected_host) = ssh_hosts.iter().find(|h| h.host == self.ssh_host) {
                    self.platform_id = selected_host.platform_id.clone().unwrap_or_default();
                }
            }

            // Platform ID (read-only, auto-filled)
            ui.horizontal(|ui| {
                ui.label(format!("{}:", tr!("mycart-add-platform-label")));
                if self.platform_id.is_empty() {
                    ui.label(
                        egui::RichText::new("(auto-filled from SSH host)")
                            .italics()
                            .color(egui::Color32::GRAY),
                    );
                } else {
                    ui.label(egui::RichText::new(&self.platform_id).strong());
                }
            });

            // Status message
            if !self.status_message.is_empty() {
                ui.add_space(12.0);
                let color = if self.status_message.contains('✓') {
                    egui::Color32::from_rgb(0, 200, 0)
                } else {
                    egui::Color32::from_rgb(255, 0, 0)
                };
                ui.colored_label(color, &self.status_message);
            }

            // Progress indicator
            if self.in_progress {
                ui.add_space(12.0);
                ui.spinner();
                ui.label(tr!("mycart-add-verifying"));
            }
        })
        .action(&tr!("cancel"), || {
            cancel_clicked.set(true);
        })
        .primary_action(&tr!("button-add"), || {
            if !self.in_progress {
                add_clicked.set(true);
            }
        })
        .show(ctx);

        if add_clicked.get() && !self.in_progress {
            // Validate required fields
            if self.site_address.is_empty()
                || self.admin_email.is_empty()
                || self.password.is_empty()
            {
                self.status_message = tr!("mycart-add-status-required").to_string();
                return AddMycartAction::None;
            }

            if self.ssh_host.is_empty() {
                self.status_message = tr!("mycart-add-status-ssh-required").to_string();
                return AddMycartAction::None;
            }

            let result = AddMycartAction::Add {
                site_address: self.site_address.clone(),
                email: self.admin_email.clone(),
                password: self.password.clone(),
                ssh_host: self.ssh_host.clone(),
                platform_id: self.platform_id.clone(),
            };
            return result;
        }

        if cancel_clicked.get() || !self.open {
            self.close();
            AddMycartAction::Cancel
        } else {
            AddMycartAction::None
        }
    }

    /// WASM stub (no SSH hosts)
    #[cfg(target_arch = "wasm32")]
    pub fn show(&mut self, ctx: &egui::Context) -> AddMycartAction {
        AddMycartAction::None
    }
}
