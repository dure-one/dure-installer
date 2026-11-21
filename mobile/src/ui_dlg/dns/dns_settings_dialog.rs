pub use super::dns_settings_dialog_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{dialog, MaterialButton};
use std::cell::Cell;

impl DnsSettingsDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, result: Option<(SettingsCheckResult, Vec<String>)>) {
        self.open = true;
        self.result = result;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.result = None;
        self.operation_log.clear();
    }

    pub fn show(&mut self, ctx: &egui::Context) -> DnsSettingsAction {
        if !self.open {
            return DnsSettingsAction::None;
        }

        let auth_clicked = Cell::new(false);
        let setup_clicked = Cell::new(false);

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog(
            "dns_settings_check_dialog",
            &tr!("dns-settings-dialog-title"),
            &mut self.open,
        )
        .min_width(350.0)
        .max_width(window_width - 120.0)
        .max_height(window_height - 120.0)
        .scrollable(true)
        .content(|ui, _open| {
            if let Some((check_result, display_msgs)) = &self.result {
                ui.heading(&tr!("dns-settings-status-heading"));
                ui.add_space(8.0);

                // Show display messages
                if !display_msgs.is_empty() {
                    ui.group(|ui| {
                        for msg in display_msgs {
                            ui.label(msg);
                        }
                    });
                }

                // Show missing records
                if !check_result.missing_records.is_empty() {
                    ui.add_space(8.0);
                    ui.label(tr!("dns-settings-missing-label"));
                    ui.group(|ui| {
                        for record in &check_result.missing_records {
                            ui.label(format!("  • {}", record));
                        }
                    });
                }

                // Show fetched DNS records
                if !check_result.fetched_records.is_empty() {
                    ui.add_space(8.0);
                    ui.label("Current DNS Records:");
                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                        .max_height(200.0)
                        .show(ui, |ui| {
                            for record in &check_result.fetched_records {
                                ui.monospace(format!(
                                    "{:<30} {:<8} {}",
                                    record.name, record.record_type, record.value
                                ));
                            }
                        });
                }

                // Show error if any
                if let Some(error) = &check_result.error {
                    use egui_material3::theme::get_global_color;
                    ui.add_space(8.0);
                    ui.colored_label(
                        get_global_color("error"),
                        tr!("dns-settings-error", {error: error}),
                    );

                    // Show authentication button if no OAuth token
                    if !self.has_oauth_token {
                        ui.add_space(8.0);
                        if ui
                            .add(MaterialButton::filled(tr!("dns-settings-auth-button")))
                            .clicked()
                        {
                            auth_clicked.set(true);
                        }
                    }
                }

                // Show Setup DNS button if authenticated and records missing
                if self.has_oauth_token
                    && !check_result.records_valid
                    && !check_result.missing_records.is_empty()
                {
                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.label(tr!("dns-settings-setup-prompt"));
                    ui.add_space(4.0);

                    // DNS record preview
                    if let (Some(domain), Some(vm_ip)) = (&self.domain, &self.vm_ip) {
                        ui.group(|ui| {
                            ui.label(format!("A    {}  →  {}", domain, vm_ip));
                            ui.label(format!("CNAME    conference.{}  →  {}", domain, domain));
                            ui.label(format!("SRV    _xmpp-client._tcp.{}  →  {}", domain, domain));
                            ui.label(format!("SRV    _xmpp-server._tcp.{}  →  {}", domain, domain));
                        });
                    }

                    ui.add_space(8.0);

                    if ui
                        .add(MaterialButton::filled(tr!("dns-settings-setup-button")))
                        .clicked()
                    {
                        setup_clicked.set(true);
                    }
                }
            }

            // Show operation log if any
            if !self.operation_log.is_empty() {
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                ui.label(tr!("dns-settings-op-log-label"));
                ui.group(|ui| {
                    for log_entry in &self.operation_log {
                        ui.label(log_entry);
                    }
                });
            }
        })
        .action(tr!("close"), || {})
        .show(ctx);

        // Handle actions
        if auth_clicked.get() {
            if let Some(domain) = self.domain.clone() {
                DnsSettingsAction::Authenticate { domain }
            } else {
                DnsSettingsAction::None
            }
        } else if setup_clicked.get() {
            DnsSettingsAction::SetupDns
        } else if !self.open {
            self.close();
            DnsSettingsAction::Close
        } else {
            DnsSettingsAction::None
        }
    }
}
