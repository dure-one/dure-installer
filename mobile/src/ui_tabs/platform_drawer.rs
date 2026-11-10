//! Platform drawer UI with Status/Logs/Operations tabs

use eframe::egui;
use egui_i18n::tr;
use egui_material3::tabs_primary;

use crate::storage::models::opslog::OperationLog;
use crate::ui_tabs::platform::PlatformRow;
use crate::viewmodel::platform::{DrawerTab, DrawerState};

/// Render the platform drawer with tabs
pub fn render_drawer(
    ui: &mut egui::Ui,
    row: &PlatformRow,
    drawer_state: &DrawerState,
    on_tab_switch: &mut Option<DrawerTab>,
    current_profile_kdbx: &Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
) {
    // Tab bar at top (Material3 compact tabs)
    let mut temp_selected = drawer_state.active_tab.to_index();
    ui.add(
        tabs_primary(&mut temp_selected)
            .id_salt("platform_drawer_tabs")
            .height(32.0)
            .tab(&tr!("platform-drawer-tab-status"))
            .tab(&tr!("platform-drawer-tab-logs"))
            .tab(&tr!("platform-drawer-tab-operations"))
    );
    if temp_selected != drawer_state.active_tab.to_index() {
        *on_tab_switch = Some(DrawerTab::from_index(temp_selected));
    }

    ui.separator();

    // Content area with static height
    egui::ScrollArea::vertical()
        .max_height(500.0)
        .show(ui, |ui| {
            match drawer_state.active_tab {
                DrawerTab::Status => render_status_tab(ui, row, drawer_state, current_profile_kdbx),
                DrawerTab::Logs => render_logs_tab(ui, drawer_state),
                DrawerTab::Operations => render_operations_tab(ui, drawer_state),
            }
        });
}

/// Render Status tab (existing drawer content)
fn render_status_tab(
    ui: &mut egui::Ui,
    row: &PlatformRow,
    _drawer_state: &DrawerState,
    current_profile_kdbx: &Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
) {
    use crate::ui_components::{ActionMenu, drawer::StatusLine};

    ui.add_space(8.0);

    // Connection info
    if let Some(email) = &row.email {
        let count_display = format_project_count_display(row.total_project_count);
        StatusLine::new(&tr!("platform-drawer-status-email"), &format!("{} ({})", email, count_display)).show(ui);
    } else {
        StatusLine::new(&tr!("platform-drawer-status-email"), tr!("platform-drawer-status-not-connected")).show(ui);
    }

    // ui.add_space(4.0);

    // Project info
    if let Some(project_id) = &row.selected_project_id {
        StatusLine::new(&tr!("platform-drawer-status-project"), project_id).show(ui);
        // ui.add_space(4.0);

        // Refresh staleness
        if let Some(last_refresh) = row.last_refresh_time {
            let elapsed = chrono::Utc::now().timestamp() - last_refresh;
            let (time_text, is_warning) = if elapsed < 60 {
                (tr!("platform-drawer-time-just-now"), false)
            } else if elapsed < 3600 {
                (tr!("platform-drawer-time-min-ago", {minutes: elapsed / 60}), false)
            } else if elapsed < 86400 {
                (tr!("platform-drawer-time-hours-ago", {hours: elapsed / 3600}), true)
            } else {
                (tr!("platform-drawer-time-days-ago", {days: elapsed / 86400}), true)
            };

            let mut status_line = StatusLine::new(&tr!("platform-drawer-status-refreshed"), &time_text);
            if is_warning {
                status_line = status_line.warning();
            }
            status_line.show(ui);
            // ui.add_space(4.0);
        }

        // VM details
        if let Some(vm_name) = &row.vm_name {
            StatusLine::new(&tr!("platform-drawer-status-vm"), vm_name).show(ui);
            // ui.add_space(4.0);

            // IP address
            if let Some(ip) = &row.vm_external_ip {
                StatusLine::new(&tr!("platform-drawer-status-ip"), ip).show(ui);
            } else {
                StatusLine::new(&tr!("platform-drawer-status-ip"), &tr!("platform-drawer-status-no-ip"))
                    .warning()
                    .show(ui);
            }
            // ui.add_space(4.0);

            // Firewall status (check operation state)
            let (firewall_text, firewall_type) = match &row.operation_state {
                crate::ui_tabs::platform::OperationState::InProgress { operation, .. }
                    if operation.to_lowercase().contains("firewall") =>
                {
                    (tr!("platform-drawer-status-updating").to_string(), 1) // warning
                }
                crate::ui_tabs::platform::OperationState::Failed { operation, error, .. }
                    if operation.to_lowercase().contains("firewall") =>
                {
                    (error.clone(), 2) // error
                }
                _ => (row.firewall_status.clone(), 0), // normal
            };
            let mut firewall_line = StatusLine::new(&tr!("platform-drawer-status-firewall"), &firewall_text);
            if firewall_type == 1 {
                firewall_line = firewall_line.warning();
            } else if firewall_type == 2 {
                firewall_line = firewall_line.error();
            }
            firewall_line.show(ui);
            // ui.add_space(4.0);

            // SSH status
            StatusLine::new(&tr!("platform-drawer-status-ssh"), &row.ssh_status).show(ui);
        } else {
            StatusLine::new(&tr!("platform-drawer-status-vm"), &tr!("platform-drawer-status-no-vm")).show(ui);
        }
    } else {
        StatusLine::new(&tr!("platform-drawer-status-project"), &tr!("platform-drawer-status-no-project")).show(ui);
    }

    // ui.add_space(8.0);

    // SSH action menu - load key on-demand when drawer is shown
    if let Some(external_ip) = &row.vm_external_ip {
        if let Some(keyring_domain) = &row.ssh_keyring_domain {
            // Cache key per (project_id, keyring_domain) to avoid repeated loads
            let cache_key = format!("ssh_key_{}_{}", row.project_id, keyring_domain);
            let cache_id = egui::Id::new(&cache_key);

            let cached_key: Option<String> = ui.data(|d| d.get_temp(cache_id));

            let private_key = if let Some(key) = cached_key {
                Some(key)
            } else if let Some(handle) = current_profile_kdbx {
                // Load key once when drawer opens (not in load_rows render loop)
                let project_id = row.selected_project_id.as_deref().unwrap_or("__global__");
                let (loaded_key, _) = super::platform::load_ssh_key_from_keyring(
                    project_id,
                    &Some(keyring_domain.clone()),
                    &Some(handle.clone()),
                );

                if let Some(ref key) = loaded_key {
                    ui.data_mut(|d| d.insert_temp(cache_id, key.clone()));
                }

                loaded_key
            } else {
                None
            };

            if let Some(private_key) = private_key {
                // ui.add_space(8.0);

                let ssh_command = format!(
                    "K=$(mktemp) && cat > $K <<'EOF'\n{}\nEOF\nchmod 600 $K && ssh -i $K root@{} && rm $K",
                    private_key.trim(),
                    external_ip
                );

                let mut menu = ActionMenu::new("💻SSH");
                menu.add_action(&tr!("platform-drawer-menu-copy-ssh"));
                menu.add_action(&tr!("platform-drawer-menu-copy-key"));
                menu.add_action(&tr!("platform-drawer-menu-copy-ip"));

                if let Some(action_idx) = menu.show(ui) {
                    let text_to_copy = match action_idx {
                        0 => &ssh_command,
                        1 => &private_key,
                        2 => external_ip,
                        _ => return,
                    };

                    ui.ctx().copy_text(text_to_copy.to_string());
                }
            } else {
                // ui.add_space(8.0);
                StatusLine::new(&tr!("platform-drawer-ssh-key-label"), &tr!("platform-drawer-ssh-not-found"))
                    .warning()
                    .show(ui);
            }
        }
    }
}

/// Helper function to format project count display
fn format_project_count_display(count: Option<usize>) -> String {
    match count {
        Some(n) => format!("{} {}", n, if n == 1 { tr!("platform-drawer-status-project-singular") } else { tr!("platform-drawer-status-project-plural") }),
        None => tr!("platform-drawer-status-project-unknown"),
    }
}

/// Render Logs tab (stdout logs filtered by project)
fn render_logs_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    // Auto-refresh: trigger log reload every 1 second (not every frame)
    if let Some(ref project_id) = drawer_state.project_id {
        let refresh_id = egui::Id::new("drawer_log_last_refresh");
        let now = std::time::Instant::now();
        let should_refresh = ui.data(|d| {
            d.get_temp::<std::time::Instant>(refresh_id)
                .map(|last| now.duration_since(last).as_secs() >= 1)
                .unwrap_or(true)
        });

        if should_refresh {
            ui.data_mut(|d| {
                d.insert_temp(refresh_id, now);
                d.insert_temp(
                    egui::Id::new("drawer_action_refresh_logs"),
                    project_id.clone(),
                );
            });
        }
    }

    ui.horizontal(|ui| {
        ui.heading(tr!("platform-drawer-logs-heading"));
        ui.add_space(8.0);

        // New logs indicator - show badge when logs are added
        let log_count_id = egui::Id::new("drawer_prev_log_count");
        let current_count = drawer_state.logs.len();
        let prev_count = ui.data(|d| d.get_temp::<usize>(log_count_id)).unwrap_or(0);

        if current_count > prev_count {
            // Show "NEW" badge when logs are added
            ui.label(
                egui::RichText::new("🆕 NEW")
                    .color(egui::Color32::from_rgb(76, 175, 80))
                    .strong()
            );

            // Request repaint for animation
            ui.ctx().request_repaint();
        }

        // Update stored count
        ui.data_mut(|d| d.insert_temp(log_count_id, current_count));

        ui.add_space(8.0);

        // Log level filter
        ui.label(tr!("platform-drawer-logs-level"));
        let filter_id = egui::Id::new("log_level_filter");
        let mut log_level_filter = ui.data(|d| d.get_temp::<String>(filter_id))
            .unwrap_or_else(|| "All".to_string());

        egui::ComboBox::from_label("")
            .selected_text(&log_level_filter)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut log_level_filter, "All".to_string(), tr!("platform-drawer-logs-level-all"));
                ui.selectable_value(&mut log_level_filter, "DEBUG".to_string(), tr!("platform-drawer-logs-level-debug"));
                ui.selectable_value(&mut log_level_filter, "INFO".to_string(), tr!("platform-drawer-logs-level-info"));
                ui.selectable_value(&mut log_level_filter, "WARN".to_string(), tr!("platform-drawer-logs-level-warn"));
                ui.selectable_value(&mut log_level_filter, "ERROR".to_string(), tr!("platform-drawer-logs-level-error"));
            });

        ui.data_mut(|d| d.insert_temp(filter_id, log_level_filter.clone()));
    });

    ui.add_space(8.0);

    if drawer_state.loading {
        ui.spinner();
        ui.label(tr!("platform-drawer-logs-loading"));
        return;
    }

    if drawer_state.project_id.is_none() {
        ui.label(tr!("platform-drawer-logs-no-project"));
        return;
    }

    if drawer_state.logs.is_empty() {
        ui.label(tr!("platform-drawer-logs-empty"));
        return;
    }

    // Filter logs by level
    let log_level_filter = ui.data(|d| d.get_temp::<String>(egui::Id::new("log_level_filter")))
        .unwrap_or_else(|| "All".to_string());

    let filtered_logs: Vec<&String> = if log_level_filter == "All" {
        drawer_state.logs.iter().collect()
    } else {
        drawer_state.logs.iter()
            .filter(|line| line.contains(&format!("[{}]", log_level_filter)))
            .collect()
    };

    // Scrollable log view with fixed-width font - always 500px height
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(500.0)
        .show(ui, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.0));

            for line in filtered_logs {
                ui.label(line);
            }
        });
}

/// Render Operations tab (operation logs from SQLite)
fn render_operations_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    // Auto-refresh: trigger operation reload every 2 seconds (not every frame)
    if let Some(ref project_id) = drawer_state.project_id {
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
                    project_id.clone(),
                );
            });
        }
    }

    ui.heading(tr!("platform-drawer-ops-heading"));
    ui.add_space(8.0);

    if drawer_state.loading {
        ui.spinner();
        ui.label(tr!("platform-drawer-ops-loading"));
        return;
    }

    if drawer_state.operations.is_empty() {
        ui.label(tr!("platform-drawer-ops-empty"));
        return;
    }

    // Scrollable operations table - fixed 500px height (matches Logs tab)
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(500.0)
        .show(ui, |ui| {
            render_operations_table(ui, &drawer_state.operations);
        });
}

/// Render operations table with simple grid layout (30px rows)
fn render_operations_table(ui: &mut egui::Ui, operations: &[OperationLog]) {
    use egui_extras::{Column, TableBuilder};

    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(130.0)) // Date
        .column(Column::auto().at_least(120.0)) // Operation
        .column(Column::auto().at_least(80.0))  // System
        .column(Column::auto().at_least(80.0))  // Status
        .column(Column::auto().at_least(60.0))  // Duration
        .column(Column::remainder())             // Error
        .header(20.0, |mut header| {
            header.col(|ui| { ui.heading(tr!("platform-drawer-ops-col-date")); });
            header.col(|ui| { ui.heading(tr!("platform-drawer-ops-col-operation")); });
            header.col(|ui| { ui.heading(tr!("platform-drawer-ops-col-system")); });
            header.col(|ui| { ui.heading(tr!("platform-drawer-ops-col-status")); });
            header.col(|ui| { ui.heading(tr!("platform-drawer-ops-col-duration")); });
            header.col(|ui| { ui.heading(tr!("platform-drawer-ops-col-error")); });
        })
        .body(|mut body| {
            for op in operations {
                body.row(30.0, |mut row| {
                    // Date
                    row.col(|ui| {
                        let dt = chrono::DateTime::from_timestamp(op.started_at, 0)
                            .map(|dt| dt.format("%Y%m%d %H:%M").to_string())
                            .unwrap_or_else(|| "???????? ??:??".to_string());
                        ui.label(dt);
                    });

                    // Operation
                    row.col(|ui| {
                        ui.label(&op.operation_type);
                    });

                    // System
                    row.col(|ui| {
                        ui.label(&op.external_system);
                    });

                    // Status with color
                    row.col(|ui| {
                        let (text, color) = match op.status.as_str() {
                            "running" => (tr!("platform-drawer-ops-status-running"), egui::Color32::from_rgb(100, 150, 255)),
                            "success" => (tr!("platform-drawer-ops-status-success"), egui::Color32::from_rgb(50, 200, 50)),
                            "failed" => (tr!("platform-drawer-ops-status-failed"), egui::Color32::from_rgb(255, 80, 80)),
                            _ => (op.status.clone(), egui::Color32::GRAY),
                        };
                        ui.colored_label(color, text);
                    });

                    // Duration
                    row.col(|ui| {
                        if let Some(duration_ms) = op.duration_ms() {
                            let duration_s = duration_ms as f64 / 1000.0;
                            ui.label(format!("{:.2}s", duration_s));
                        } else {
                            ui.label("-");
                        }
                    });

                    // Error
                    row.col(|ui| {
                        if let Some(err) = &op.error_message {
                            ui.label(err);
                        } else {
                            ui.label("");
                        }
                    });
                });
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawer_tabs_all() {
        let tabs = DrawerTab::all();
        assert_eq!(tabs.len(), 3);
        assert_eq!(tabs[0], DrawerTab::Status);
        assert_eq!(tabs[1], DrawerTab::Logs);
        assert_eq!(tabs[2], DrawerTab::Operations);
    }
}
