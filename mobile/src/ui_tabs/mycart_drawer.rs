//! Mycart drawer UI with Status/Logs tabs

use eframe::egui;
use egui_i18n::tr;
use egui_material3::tabs_secondary;
use std::collections::HashMap;

use crate::api::mycart::{MycartStats, SellerInfo};
use crate::ui_components::drawer::StatusLine;
use crate::viewmodel::mycart::DrawerTab;

/// Render the Mycart drawer with tabs
pub fn render_drawer(
    ui: &mut egui::Ui,
    site: &str,
    stats: &Option<MycartStats>,
    seller: &Option<SellerInfo>,
    logs: &HashMap<String, String>,
    active_tab: &DrawerTab,
    on_tab_switch: &mut Option<DrawerTab>,
) {
    // Tab bar
    let mut temp_selected = active_tab.to_index();
    ui.add(
        tabs_secondary(&mut temp_selected)
            .id_salt("mycart_drawer_tabs")
            .height(32.0)
            .tab("Status")
            .tab("Logs"),
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
            DrawerTab::Logs => render_logs_tab(ui, logs),
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

/// Render Logs tab with sub-tabs for each log file
fn render_logs_tab(ui: &mut egui::Ui, logs: &HashMap<String, String>) {
    ui.add_space(8.0);

    if logs.is_empty() {
        ui.label("No logs loaded. Click 'Logs' button to load logs.");
        return;
    }

    // Sub-tabs for each log file
    let log_files = vec![
        "mycart-stderr.log",
        "mycart-stdout.log",
        "fail2ban-rs-stderr.log",
        "fail2ban-rs-stdout.log",
        "xmpp-proxy-stdout.log",
        "xmpp-proxy-stderr.log",
        "prosody.log",
    ];

    egui::ScrollArea::vertical().show(ui, |ui| {
        for log_name in log_files {
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
    });
}
