//! Mycart drawer UI with Status/Logs tabs

use eframe::egui;
use egui_i18n::tr;
use egui_material3::tabs_secondary;
use std::collections::HashMap;

use crate::api::mycart::{CrontabJob, MycartStats, SellerInfo};
use crate::ui_components::drawer::StatusLine;
use crate::viewmodel::mycart::DrawerTab;

/// Action from drawer (passed back to parent)
#[derive(Debug, Clone)]
pub enum DrawerAction {
    ToggleCronjob { job_id: String, enabled: bool },
    SetCronjobInterval { job_id: String, interval: String },
}

/// Render the Mycart drawer with tabs (Phase 4: service-specific tabs)
pub fn render_drawer(
    ui: &mut egui::Ui,
    site: &str,
    stats: &Option<MycartStats>,
    seller: &Option<SellerInfo>,
    active_tab: &DrawerTab,
    on_tab_switch: &mut Option<DrawerTab>,
    crontab_jobs: &[CrontabJob],
    crontab_loading: bool,
    crontab_error: &Option<String>,
    on_action: &mut Option<DrawerAction>,
    mycart_logs: &HashMap<String, String>,
    mycart_loading: bool,
    xmppproxy_logs: &HashMap<String, String>,
    xmppproxy_loading: bool,
    fail2banrs_logs: &HashMap<String, String>,
    fail2banrs_loading: bool,
) {
    // Tab bar (Phase 4: 5 tabs)
    let mut temp_selected = active_tab.to_index();
    ui.add(
        tabs_secondary(&mut temp_selected)
            .id_salt("mycart_drawer_tabs")
            .height(32.0)
            .tab("Status")
            .tab("Operations")
            .tab("Mycart")
            .tab("XMPPProxy")
            .tab("Fail2BanRS"),
    );
    if temp_selected != active_tab.to_index() {
        *on_tab_switch = Some(DrawerTab::from_index(temp_selected));
    }

    ui.separator();

    // Content area
    egui::ScrollArea::vertical()
        .max_height(500.0)
        .show(ui, |ui| match active_tab {
            DrawerTab::Status => render_status_tab(ui, site, stats, seller),
            DrawerTab::Operations => render_operations_tab(ui, crontab_jobs, crontab_loading, crontab_error, on_action),
            DrawerTab::Mycart => render_mycart_tab(ui, mycart_logs, mycart_loading),
            DrawerTab::XMPPProxy => render_xmppproxy_tab(ui, xmppproxy_logs, xmppproxy_loading),
            DrawerTab::Fail2BanRS => render_fail2banrs_tab(ui, fail2banrs_logs, fail2banrs_loading),
        });
}

/// Render Status tab
fn render_status_tab(
    ui: &mut egui::Ui,
    site: &str,
    stats: &Option<MycartStats>,
    seller: &Option<SellerInfo>,
) {
    ui.add_space(8.0);

    // Site address
    StatusLine::new("Site", site).show(ui);

    if let Some(stats) = stats {
        StatusLine::new("Orders", &stats.orders_count.to_string()).show(ui);
        StatusLine::new("Products", &stats.products_count.to_string()).show(ui);
        StatusLine::new("Messages", &stats.messages_count.to_string()).show(ui);
    } else {
        StatusLine::new("Stats", "Not loaded").warning().show(ui);
    }

    ui.add_space(8.0);

    if let Some(seller) = seller {
        ui.label(egui::RichText::new("Seller Info").strong());
        StatusLine::new("Seller ID", &seller.seller_id).show(ui);
        StatusLine::new("Name", &seller.seller_name).show(ui);
        StatusLine::new("Email", &seller.email).show(ui);
        if let Some(phone) = &seller.phone {
            StatusLine::new("Phone", phone).show(ui);
        }
    }
}

/// Render Mycart tab
fn render_mycart_tab(ui: &mut egui::Ui, logs: &HashMap<String, String>, loading: bool) {
    ui.add_space(8.0);

    if loading {
        ui.label("Loading mycart logs...");
        ui.spinner();
        return;
    }

    if logs.is_empty() {
        ui.label("No logs loaded. Click 'Check Mycart' button to load logs.");
        return;
    }

    for log_name in ["mycart-stderr.log", "mycart-stdout.log"] {
        ui.collapsing(log_name, |ui| {
            if let Some(content) = logs.get(log_name) {
                ui.add(
                    egui::TextEdit::multiline(&mut content.as_str())
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(20),
                );
            } else {
                ui.label("Log not found or failed to load");
            }
        });
    }
}

/// Render XMPPProxy tab
fn render_xmppproxy_tab(ui: &mut egui::Ui, logs: &HashMap<String, String>, loading: bool) {
    ui.add_space(8.0);

    if loading {
        ui.label("Loading xmppproxy logs...");
        ui.spinner();
        return;
    }

    if logs.is_empty() {
        ui.label("No logs loaded. Click 'Check XMPPProxy' button to load logs.");
        return;
    }

    for log_name in ["xmpp-proxy-stdout.log", "xmpp-proxy-stderr.log"] {
        ui.collapsing(log_name, |ui| {
            if let Some(content) = logs.get(log_name) {
                ui.add(
                    egui::TextEdit::multiline(&mut content.as_str())
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(20),
                );
            } else {
                ui.label("Log not found or failed to load");
            }
        });
    }
}

/// Render Fail2BanRS tab
fn render_fail2banrs_tab(ui: &mut egui::Ui, logs: &HashMap<String, String>, loading: bool) {
    ui.add_space(8.0);

    if loading {
        ui.label("Loading fail2banrs logs...");
        ui.spinner();
        return;
    }

    if logs.is_empty() {
        ui.label("No logs loaded. Click 'Check Fail2BanRS' button to load logs.");
        return;
    }

    for log_name in ["fail2ban-rs-stderr.log", "fail2ban-rs-stdout.log"] {
        ui.collapsing(log_name, |ui| {
            if let Some(content) = logs.get(log_name) {
                ui.add(
                    egui::TextEdit::multiline(&mut content.as_str())
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(20),
                );
            } else {
                ui.label("Log not found or failed to load");
            }
        });
    }
}

/// Render Operations tab with crontab management (Phase 5: interactive controls)
fn render_operations_tab(
    ui: &mut egui::Ui,
    crontab_jobs: &[CrontabJob],
    crontab_loading: bool,
    crontab_error: &Option<String>,
    on_action: &mut Option<DrawerAction>,
) {
    ui.add_space(8.0);

    ui.heading("Crontab Jobs");
    ui.add_space(4.0);

    // Show error if present
    if let Some(error) = crontab_error {
        ui.colored_label(egui::Color32::from_rgb(255, 0, 0), format!("⚠ Error: {}", error));
        ui.add_space(4.0);
    }

    // Show loading state
    if crontab_loading {
        ui.label("Loading crontab jobs...");
        ui.add_space(4.0);
        ui.spinner();
        return;
    }

    // Show empty state or jobs list
    if crontab_jobs.is_empty() {
        ui.label("No crontab jobs available. Click refresh to load.");
    } else {
        ui.label(format!("{} crontab jobs found", crontab_jobs.len()));
        ui.add_space(8.0);

        // Job list with interactive controls
        for job in crontab_jobs {
            ui.group(|ui| {
                ui.label(egui::RichText::new(&job.name).strong());
                ui.label(format!("ID: {}", job.job_id));

                // Enabled checkbox
                ui.horizontal(|ui| {
                    let mut enabled = job.enabled;
                    if ui.checkbox(&mut enabled, "Enabled").changed() {
                        *on_action = Some(DrawerAction::ToggleCronjob {
                            job_id: job.job_id.clone(),
                            enabled,
                        });
                    }
                });

                // Interval editor with persistent state
                ui.horizontal(|ui| {
                    ui.label("Interval:");

                    // Use ui.data_mut to persist the interval buffer across frames
                    let interval_id = ui.make_persistent_id(format!("interval_{}", job.job_id));
                    let mut interval_buffer = ui.data_mut(|d| {
                        d.get_persisted::<String>(interval_id)
                            .unwrap_or_else(|| job.interval.clone())
                    });

                    ui.text_edit_singleline(&mut interval_buffer);

                    // Store the buffer back
                    ui.data_mut(|d| d.insert_persisted(interval_id, interval_buffer.clone()));

                    // Apply button to submit changes
                    if ui.button("Apply").clicked() && interval_buffer != job.interval {
                        *on_action = Some(DrawerAction::SetCronjobInterval {
                            job_id: job.job_id.clone(),
                            interval: interval_buffer,
                        });
                    }
                });

                // Last run timestamp
                if let Some(last_run) = job.last_run {
                    use chrono::{DateTime, Utc};
                    let dt = DateTime::<Utc>::from_timestamp(last_run, 0).unwrap_or_default();
                    ui.label(format!("Last run: {}", dt.format("%Y-%m-%d %H:%M")));
                } else {
                    ui.label("Last run: never");
                }
            });
            ui.add_space(4.0);
        }
    }
}
