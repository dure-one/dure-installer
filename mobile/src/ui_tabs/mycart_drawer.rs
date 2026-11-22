//! Mycart drawer UI with Status/Logs tabs

use eframe::egui;
use egui_i18n::tr;
use egui_material3::{tabs_primary, tabs_secondary};

use crate::api::mycart::{CrontabJob, MycartStats, SellerInfo};
use crate::ui_components::drawer::StatusLine;
use crate::viewmodel::mycart::DrawerTab;

/// Action from drawer (passed back to parent)
#[derive(Debug, Clone)]
pub enum DrawerAction {
    ToggleCronjob { job_id: String, enabled: bool },
    SetCronjobInterval { job_id: String, interval: String },
}

/// Render the Mycart drawer with tabs (Phase 5: 6 tabs)
pub fn render_drawer(
    ui: &mut egui::Ui,
    site: &str,
    stats: &Option<MycartStats>,
    seller: &Option<SellerInfo>,
    active_tab: &DrawerTab,
    on_tab_switch: &mut Option<DrawerTab>,
    logs: &Vec<String>,
    logs_loading: bool,
    operations: &[crate::storage::models::opslog::OperationLog],
    operations_loading: bool,
    mycart_logs: &std::collections::HashMap<String, Vec<String>>,
    mycart_loading: bool,
    xmppproxy_logs: &std::collections::HashMap<String, Vec<String>>,
    xmppproxy_loading: bool,
    fail2banrs_logs: &std::collections::HashMap<String, Vec<String>>,
    fail2banrs_loading: bool,
) {
    // Tab bar (Phase 5: 6 tabs)
    let mut temp_selected = active_tab.to_index();
    ui.add(
        tabs_primary(&mut temp_selected)
            .id_salt("mycart_drawer_tabs")
            .height(32.0)
            .tab("Status")
            .tab("Logs")
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
            DrawerTab::Logs => render_logs_tab(ui, logs, logs_loading, site),
            DrawerTab::Operations => render_operations_tab(ui, operations, operations_loading, site),
            DrawerTab::Mycart => render_mycart_tab(ui, mycart_logs, mycart_loading),
            DrawerTab::XMPPProxy => render_xmppproxy_tab(ui, xmppproxy_logs, xmppproxy_loading),
            DrawerTab::Fail2BanRS => render_fail2banrs_tab(ui, fail2banrs_logs, fail2banrs_loading),
        });
}

/// Render Logs tab (installer stdout logs filtered by site, mirroring platform)
fn render_logs_tab(ui: &mut egui::Ui, logs: &Vec<String>, loading: bool, site: &str) {
    // Auto-refresh every 2s
    let refresh_id = egui::Id::new("drawer_logs_last_refresh");
    let now = std::time::Instant::now();
    let should_refresh = ui.data(|d| {
        d.get_temp::<std::time::Instant>(refresh_id)
            .map(|last| now.duration_since(last).as_secs() >= 2)
            .unwrap_or(true)
    });

    if should_refresh {
        ui.data_mut(|d| {
            d.insert_temp(refresh_id, now);
            d.insert_temp(
                egui::Id::new("drawer_action_refresh_logs"),
                site.to_string(),
            );
        });
    }

    ui.horizontal(|ui| {
        ui.heading("Installer Logs");
        ui.add_space(8.0);

        // Log level filter
        ui.label("Level:");
        let filter_id = egui::Id::new("installer_log_level_filter");
        let mut log_level_filter = ui
            .data(|d| d.get_temp::<String>(filter_id))
            .unwrap_or_else(|| "All".to_string());

        egui::ComboBox::from_label("")
            .selected_text(&log_level_filter)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut log_level_filter, "All".to_string(), "All");
                ui.selectable_value(&mut log_level_filter, "DEBUG".to_string(), "DEBUG");
                ui.selectable_value(&mut log_level_filter, "INFO".to_string(), "INFO");
                ui.selectable_value(&mut log_level_filter, "WARN".to_string(), "WARN");
                ui.selectable_value(&mut log_level_filter, "ERROR".to_string(), "ERROR");
            });

        ui.data_mut(|d| d.insert_temp(filter_id, log_level_filter.clone()));
    });

    ui.add_space(8.0);

    if loading {
        ui.spinner();
        ui.label("Loading logs...");
        return;
    }

    if logs.is_empty() {
        ui.label("No logs available.");
        return;
    }

    // Filter logs by level
    let log_level_filter = ui
        .data(|d| d.get_temp::<String>(egui::Id::new("installer_log_level_filter")))
        .unwrap_or_else(|| "All".to_string());

    let filtered_logs: Vec<&String> = if log_level_filter == "All" {
        logs.iter().collect()
    } else {
        logs.iter()
            .filter(|line| line.contains(&format!("[{}]", log_level_filter)))
            .collect()
    };

    // Scrollable log view
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(500.0)
        .show(ui, |ui| {
            for line in filtered_logs {
                ui.label(egui::RichText::new(line).monospace().small());
            }
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

/// Render Mycart tab with sub-tabs for each log file
fn render_mycart_tab(
    ui: &mut egui::Ui,
    logs: &std::collections::HashMap<String, Vec<String>>,
    loading: bool,
) {
    ui.heading("Mycart Logs");

    if loading {
        ui.spinner();
        ui.label("Loading logs...");
        return;
    }

    if logs.is_empty() {
        ui.label("No logs loaded. Click 'Check Mycart' button to load logs.");
        return;
    }

    // Sub-tabs: stderr | stdout
    let mut active_subtab = ui
        .data(|d| d.get_temp::<usize>(egui::Id::new("mycart_log_subtab")))
        .unwrap_or(0);

    ui.add(
        tabs_secondary(&mut active_subtab)
            .id_salt("mycart_log_subtabs")
            .height(32.0)
            .tab("stderr")
            .tab("stdout"),
    );
    ui.data_mut(|d| d.insert_temp(egui::Id::new("mycart_log_subtab"), active_subtab));

    ui.separator();

    // Show selected file based on tab index
    let selected_file = match active_subtab {
        0 => "stderr",
        1 => "stdout",
        _ => "stderr",
    };

    if let Some(lines) = logs.get(selected_file) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(500.0)
            .show(ui, |ui| {
                for line in lines {
                    ui.label(egui::RichText::new(line).monospace().small());
                }
            });
    } else {
        ui.label(format!("No logs for {}", selected_file));
    }
}

/// Render XMPPProxy tab with sub-tabs for each log file
fn render_xmppproxy_tab(
    ui: &mut egui::Ui,
    logs: &std::collections::HashMap<String, Vec<String>>,
    loading: bool,
) {
    ui.heading("XMPPProxy Logs");

    if loading {
        ui.spinner();
        ui.label("Loading logs...");
        return;
    }

    if logs.is_empty() {
        ui.label("No logs loaded. Click 'Check XMPPProxy' button to load logs.");
        return;
    }

    // Sub-tabs: stdout | stderr
    let mut active_subtab = ui
        .data(|d| d.get_temp::<usize>(egui::Id::new("xmppproxy_log_subtab")))
        .unwrap_or(0);

    ui.add(
        tabs_secondary(&mut active_subtab)
            .id_salt("xmppproxy_log_subtabs")
            .height(32.0)
            .tab("stdout")
            .tab("stderr"),
    );
    ui.data_mut(|d| d.insert_temp(egui::Id::new("xmppproxy_log_subtab"), active_subtab));

    ui.separator();

    // Show selected file based on tab index
    let selected_file = match active_subtab {
        0 => "stdout",
        1 => "stderr",
        _ => "stdout",
    };

    if let Some(lines) = logs.get(selected_file) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(500.0)
            .show(ui, |ui| {
                for line in lines {
                    ui.label(egui::RichText::new(line).monospace().small());
                }
            });
    } else {
        ui.label(format!("No logs for {}", selected_file));
    }
}

/// Render Fail2BanRS tab with sub-tabs for each log file
fn render_fail2banrs_tab(
    ui: &mut egui::Ui,
    logs: &std::collections::HashMap<String, Vec<String>>,
    loading: bool,
) {
    ui.heading("Fail2BanRS Logs");

    if loading {
        ui.spinner();
        ui.label("Loading logs...");
        return;
    }

    if logs.is_empty() {
        ui.label("No logs loaded. Click 'Check Fail2BanRS' button to load logs.");
        return;
    }

    // Sub-tabs: stderr | stdout
    let mut active_subtab = ui
        .data(|d| d.get_temp::<usize>(egui::Id::new("fail2banrs_log_subtab")))
        .unwrap_or(0);

    ui.add(
        tabs_secondary(&mut active_subtab)
            .id_salt("fail2banrs_log_subtabs")
            .height(32.0)
            .tab("stderr")
            .tab("stdout"),
    );
    ui.data_mut(|d| d.insert_temp(egui::Id::new("fail2banrs_log_subtab"), active_subtab));

    ui.separator();

    // Show selected file based on tab index
    let selected_file = match active_subtab {
        0 => "stderr",
        1 => "stdout",
        _ => "stderr",
    };

    if let Some(lines) = logs.get(selected_file) {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(500.0)
            .show(ui, |ui| {
                for line in lines {
                    ui.label(egui::RichText::new(line).monospace().small());
                }
            });
    } else {
        ui.label(format!("No logs for {}", selected_file));
    }
}

/// Render Operations tab showing operation history (Phase 5: mirroring platform pattern)
fn render_operations_tab(
    ui: &mut egui::Ui,
    operations: &[crate::storage::models::opslog::OperationLog],
    loading: bool,
    site: &str,
) {
    // Auto-refresh every 2s
    let refresh_id = egui::Id::new("drawer_ops_last_refresh");
    let now = std::time::Instant::now();
    let should_refresh = ui.data(|d| {
        d.get_temp::<std::time::Instant>(refresh_id)
            .map(|last| now.duration_since(last).as_secs() >= 2)
            .unwrap_or(true)
    });

    if should_refresh {
        ui.data_mut(|d| {
            d.insert_temp(refresh_id, now);
            d.insert_temp(
                egui::Id::new("drawer_action_refresh_operations"),
                site.to_string(),
            );
        });
    }

    ui.add_space(8.0);

    if loading {
        ui.spinner();
        ui.label("Loading operations...");
        return;
    }

    ui.heading("Operation History");
    ui.add_space(8.0);

    let renderer = crate::ui_components::drawer::OperationsRenderer::new().max_height(500.0);
    renderer.show(ui, operations);
}
