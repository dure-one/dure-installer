//! DNS drawer UI with Status/Logs/Operations tabs

use eframe::egui;

use crate::ui_components::drawer::{StatusLine, TabBar, LogsRenderer, OperationsRenderer};
use crate::ui_tabs::dns::{DnsDomainRow, DnsDrawerState, DnsDrawerTab};

/// Render the DNS drawer with tabs
pub fn render_drawer(
    ui: &mut egui::Ui,
    row: &DnsDomainRow,
    drawer_state: &DnsDrawerState,
    on_tab_switch: &mut Option<DnsDrawerTab>,
) {
    // Tab bar
    let tab_bar = TabBar::<DnsDrawerTab>::new();
    if let Some(new_tab) = tab_bar.show(ui, drawer_state.active_tab) {
        *on_tab_switch = Some(new_tab);
    }

    ui.separator();

    // Content area
    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            match drawer_state.active_tab {
                DnsDrawerTab::Status => render_status_tab(ui, row),
                DnsDrawerTab::Logs => render_logs_tab(ui, drawer_state),
                DnsDrawerTab::Operations => render_operations_tab(ui, drawer_state),
            }
        });
}

/// Render Status tab
fn render_status_tab(ui: &mut egui::Ui, row: &DnsDomainRow) {
    ui.add_space(8.0);

    // Domain
    StatusLine::new("Domain", &row.domain).show(ui);

    // Provider (with account_id if available)
    let provider_display = if let Some(account_id) = &row.account_id {
        format!("{}({})", row.provider, account_id)
    } else {
        row.provider.clone()
    };
    StatusLine::new("Provider", &provider_display).show(ui);

    ui.add_space(8.0);
    ui.label(egui::RichText::new("Platform Binding").strong());
    ui.separator();

    // Platform info
    if let Some(platform_id) = &row.platform_id {
        StatusLine::new("Platform ID", platform_id).show(ui);
    } else {
        StatusLine::new("Platform ID", "Not bound")
            .warning()
            .show(ui);
    }

    if let Some(platform_name) = &row.platform_name {
        StatusLine::new("Platform", platform_name).show(ui);
    }

    if let Some(vm_name) = &row.vm_name {
        StatusLine::new("VM Name", vm_name).show(ui);
    }

    if let Some(vm_ip) = &row.vm_external_ip {
        StatusLine::new("VM IP", vm_ip).show(ui);
    }

    ui.add_space(8.0);
    ui.label(egui::RichText::new("Check Status").strong());
    ui.separator();

    // NS Check Status
    if let Some(check) = &row.last_ns_check {
        if check.uses_cloudflare {
            StatusLine::new("NS Check", "✓ Using Cloudflare")
                .show(ui);
            for ns in &check.nameservers {
                ui.label(format!("  {}", ns));
            }
        } else {
            let mut line = StatusLine::new("NS Check", "✗ Not using Cloudflare");
            if check.error.is_some() {
                line = line.error();
            } else {
                line = line.warning();
            }
            line.show(ui);
        }
        if let Some(err) = &check.error {
            ui.label(format!("  Error: {}", err));
        }
    } else {
        StatusLine::new("NS Check", "Not checked").show(ui);
    }

    // Settings Check Status
    if let Some(check) = &row.last_settings_check {
        if check.records_valid {
            StatusLine::new("Settings Check", "✓ All records configured")
                .show(ui);
        } else {
            StatusLine::new("Settings Check", &format!("✗ {} missing records", check.missing_records.len()))
                .warning()
                .show(ui);
            for record in &check.missing_records {
                ui.label(format!("  {}", record));
            }
        }
        if let Some(err) = &check.error {
            ui.label(format!("  Error: {}", err));
        }
    } else {
        StatusLine::new("Settings Check", "Not checked").show(ui);
    }

    // HTTPS Check Status
    if let Some(check) = &row.last_https_check {
        if check.resolves_to_vm {
            let msg = if let Some(ip) = &check.resolved_ip {
                format!("✓ Resolves to {} (VM IP)", ip)
            } else {
                "✓ Resolves correctly".to_string()
            };
            StatusLine::new("HTTPS Check", &msg).show(ui);
        } else {
            let msg = if let (Some(resolved), Some(expected)) = (&check.resolved_ip, &check.expected_ip) {
                format!("✗ Resolves to {} (expected {})", resolved, expected)
            } else {
                "✗ Does not resolve to VM".to_string()
            };
            StatusLine::new("HTTPS Check", &msg)
                .warning()
                .show(ui);
        }
        if let Some(err) = &check.error {
            ui.label(format!("  Error: {}", err));
        }
    } else {
        StatusLine::new("HTTPS Check", "Not checked").show(ui);
    }
}

/// Render Logs tab
fn render_logs_tab(ui: &mut egui::Ui, drawer_state: &DnsDrawerState) {
    let renderer = LogsRenderer::new("DNS Operation Logs")
        .max_height(400.0)
        .refresh_interval(2);

    let refresh_id = ui.id().with("dns_logs_refresh");
    renderer.show(ui, &drawer_state.logs, refresh_id);
}

/// Render Operations tab
fn render_operations_tab(ui: &mut egui::Ui, drawer_state: &DnsDrawerState) {
    let renderer = OperationsRenderer::new()
        .max_height(400.0);

    renderer.show(ui, &drawer_state.operations);
}
