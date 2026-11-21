pub use super::add_firewall_dialog_stt::*;
use crate::ui_tabs::platform::FirewallRule;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{dialog, linear_progress, MaterialButton, MaterialInput};
use std::cell::Cell;

impl AddFirewallDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, platform: String, form_data: FirewallRule, edit_idx: Option<usize>) {
        self.platform = platform;
        self.form_data = form_data;
        self.edit_idx = edit_idx;
        self.save_in_progress = false;
        self.save_error = None;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.platform.clear();
        self.form_data = FirewallRule::default();
        self.edit_idx = None;
        self.save_in_progress = false;
        self.save_error = None;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> AddFirewallAction {
        if !self.open {
            return AddFirewallAction::None;
        }

        let form_data = &mut self.form_data;
        let is_edit = self.edit_idx.is_some();
        let confirmed = Cell::new(false);
        let cancelled = Cell::new(false);

        // Validation error states
        let name_error = Cell::new(false);
        let network_error = Cell::new(false);
        let direction_error = Cell::new(false);
        let allowed_denied_error = Cell::new(false);

        // Persist string state across frames
        let state_id = egui::Id::new("firewall_form_strings");
        let mut strings = ctx
            .data_mut(|d| {
                d.get_persisted::<(
                    String,
                    String,
                    String,
                    String,
                    String,
                    String,
                    String,
                    String,
                    String,
                )>(state_id)
            })
            .unwrap_or_else(|| {
                (
                    form_data.source_ranges.join(", "),
                    form_data.dest_ranges.join(", "),
                    form_data.allowed.join(", "),
                    form_data.denied.join(", "),
                    form_data.source_tags.join(", "),
                    form_data.source_service_accounts.join(", "),
                    form_data.target_tags.join(", "),
                    form_data.target_service_accounts.join(", "),
                    form_data.priority.to_string(),
                )
            });

        let (
            mut src_ranges_str,
            mut dest_ranges_str,
            mut allowed_str,
            mut denied_str,
            mut src_tags_str,
            mut src_svc_str,
            mut target_tags_str,
            mut target_svc_str,
            mut priority_str,
        ) = strings;

        let title = if is_edit {
            "Edit Firewall Rule"
        } else {
            &tr!("firewall-add-dialog-title")
        };

        let window_width = ctx.viewport_rect().width();

        dialog("add_firewall_dialog", title, &mut self.open)
            .min_width(600.0)
            .max_width((window_width * 0.8).min(600.0))
            .scrollable(true)
            .content(|ui, open| {
                ui.heading(title);
                ui.add_space(12.0);

                if self.save_in_progress {
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.add(linear_progress());
                        ui.label(if is_edit {
                            "Updating firewall rule..."
                        } else {
                            "Creating firewall rule..."
                        });
                    });
                    ui.add_space(8.0);
                }

                if let Some(ref error_msg) = self.save_error {
                    ui.add_space(8.0);
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 82, 82),
                        format!("❌ Error: {}", error_msg),
                    );
                    ui.add_space(8.0);
                }

                ui.add(
                    MaterialInput::filled(&mut form_data.name)
                        .label("Name (e.g., allow-ssh-from-office)")
                        .is_error(name_error.get())
                        .error_text("Name is required"),
                );

                ui.add(
                    MaterialInput::filled(&mut form_data.network)
                        .label("Network (e.g., default)")
                        .is_error(network_error.get())
                        .error_text("Network is required"),
                );

                ui.add(
                    MaterialInput::filled(&mut form_data.direction)
                        .label("Direction (INGRESS or EGRESS)")
                        .is_error(direction_error.get())
                        .error_text("Must be INGRESS or EGRESS"),
                );

                ui.add(MaterialInput::filled(&mut priority_str).label("Priority (e.g., 1000 - lower = higher priority)"));

                ui.add(MaterialInput::filled(&mut src_ranges_str).label("Source IP Ranges (e.g., 203.0.113.0/24, 198.51.100.42/32)"));

                ui.add(MaterialInput::filled(&mut dest_ranges_str).label("Destination Ranges (e.g., 0.0.0.0/0)"));

                ui.add(
                    MaterialInput::filled(&mut allowed_str)
                        .label("Allowed Protocols (e.g., tcp:22, tcp:80, tcp:443, icmp)")
                        .is_error(allowed_denied_error.get())
                        .error_text("Either allowed or denied rules required"),
                );

                ui.add(
                    MaterialInput::filled(&mut denied_str)
                        .label("Denied Protocols (e.g., tcp:23 - optional)")
                        .is_error(allowed_denied_error.get()),
                );

                ui.add(MaterialInput::filled(&mut src_tags_str).label("Source Tags (e.g., web-server, api-backend)"));

                ui.add(MaterialInput::filled(&mut src_svc_str).label("Source Service Account (e.g., app@project.iam.gserviceaccount.com)"));

                ui.add(MaterialInput::filled(&mut target_tags_str).label("Target Tags (e.g., http-server, https-server)"));

                ui.add(MaterialInput::filled(&mut target_svc_str).label("Target Service Account (e.g., vm@project.iam.gserviceaccount.com)"));

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let button_text = if is_edit { "Save" } else { "Create" };
                    let save_button = MaterialButton::filled(button_text).enabled(!self.save_in_progress);
                    if ui.add(save_button).clicked() {
                        // Parse string inputs
                        if let Ok(val) = priority_str.parse::<u32>() {
                            form_data.priority = val;
                        }
                        form_data.source_ranges = src_ranges_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        form_data.dest_ranges = dest_ranges_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        form_data.allowed = allowed_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        form_data.denied = denied_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        form_data.source_tags = src_tags_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        form_data.source_service_accounts = src_svc_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        form_data.target_tags = target_tags_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        form_data.target_service_accounts = target_svc_str
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();

                        // Validate
                        let name_valid = !form_data.name.trim().is_empty();
                        let network_valid = !form_data.network.trim().is_empty();
                        let direction_valid =
                            form_data.direction == "INGRESS" || form_data.direction == "EGRESS";
                        let allowed_denied_valid =
                            !form_data.allowed.is_empty() || !form_data.denied.is_empty();

                        name_error.set(!name_valid);
                        network_error.set(!network_valid);
                        direction_error.set(!direction_valid);
                        allowed_denied_error.set(!allowed_denied_valid);

                        if name_valid && network_valid && direction_valid && allowed_denied_valid {
                            confirmed.set(true);
                        }
                    }
                    if ui.add(MaterialButton::outlined("Cancel")).clicked() {
                        cancelled.set(true);
                        *open = false;
                    }
                });

                // Save string state
                ctx.data_mut(|d| {
                    d.insert_persisted(
                        state_id,
                        (
                            src_ranges_str.clone(),
                            dest_ranges_str.clone(),
                            allowed_str.clone(),
                            denied_str.clone(),
                            src_tags_str.clone(),
                            src_svc_str.clone(),
                            target_tags_str.clone(),
                            target_svc_str.clone(),
                            priority_str.clone(),
                        ),
                    );
                });
            })
            .show(ctx);

        if confirmed.get() {
            let result = AddFirewallAction::Save {
                rule_data: self.form_data.clone(),
                edit_idx: self.edit_idx,
            };
            // Don't close yet - wait for save completion event
            result
        } else if cancelled.get() {
            self.close();
            AddFirewallAction::Cancel
        } else if !self.open {
            self.close();
            AddFirewallAction::Cancel
        } else {
            AddFirewallAction::None
        }
    }
}
