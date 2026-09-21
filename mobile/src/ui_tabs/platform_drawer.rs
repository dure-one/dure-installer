//! Platform drawer UI with Status/Logs/Operations tabs

use eframe::egui;

use crate::storage::models::opslog::OperationLog;
use crate::ui_tabs::platform::PlatformRow;
use crate::viewmodel::platform::{DrawerTab, DrawerState};

/// Render the platform drawer with tabs
pub fn render_drawer(
    ui: &mut egui::Ui,
    row: &PlatformRow,
    drawer_state: &DrawerState,
    on_tab_switch: &mut Option<DrawerTab>,
) {
    // Tab bar at top
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        for tab in DrawerTab::all() {
            let is_selected = drawer_state.active_tab == tab;

            if ui.selectable_label(is_selected, tab.as_str()).clicked() && !is_selected {
                *on_tab_switch = Some(tab);
            }
        }
    });

    ui.separator();

    // Content area
    match drawer_state.active_tab {
        DrawerTab::Status => render_status_tab(ui, row, drawer_state),
        DrawerTab::Logs => render_logs_tab(ui, drawer_state),
        DrawerTab::Operations => render_operations_tab(ui, drawer_state),
    }
}

/// Render Status tab (existing drawer content)
fn render_status_tab(ui: &mut egui::Ui, row: &PlatformRow, _drawer_state: &DrawerState) {
    use crate::ui_components::ActionMenu;
    use egui_twemoji::EmojiLabel as TwemojiLabel;

    ui.add_space(8.0);

    // Connection info
    if let Some(email) = &row.email {
        let count_display = format_project_count_display(row.total_project_count);
        TwemojiLabel::new(format!("📧 {} ({})", email, count_display)).show(ui);
    } else {
        TwemojiLabel::new("📧 Not connected").show(ui);
    }

    ui.add_space(4.0);

    // Project info
    if let Some(project_id) = &row.selected_project_id {
        TwemojiLabel::new(format!("📁 Project: {}", project_id)).show(ui);
        ui.add_space(4.0);

        // Refresh staleness
        if let Some(last_refresh) = row.last_refresh_time {
            let elapsed = chrono::Utc::now().timestamp() - last_refresh;
            let time_text = if elapsed < 60 {
                "🕐 Refreshed: just now".to_string()
            } else if elapsed < 3600 {
                format!("🕐 Refreshed: {} min ago", elapsed / 60)
            } else if elapsed < 86400 {
                format!("⚠️ Refreshed: {} hours ago", elapsed / 3600)
            } else {
                format!("⚠️ Refreshed: {} days ago", elapsed / 86400)
            };

            let color = if elapsed < 3600 {
                ui.style().visuals.text_color()
            } else {
                egui::Color32::from_rgb(255, 193, 7) // Warning color
            };

            TwemojiLabel::new(egui::RichText::new(time_text).color(color)).show(ui);
            ui.add_space(4.0);
        }

        // VM details
        if let Some(vm_name) = &row.vm_name {
            TwemojiLabel::new(format!("💻 VM: {}", vm_name)).show(ui);
            ui.add_space(4.0);

            // IP address
            if let Some(ip) = &row.vm_external_ip {
                TwemojiLabel::new(format!("🌐 IP: {}", ip)).show(ui);
            } else {
                TwemojiLabel::new(
                    egui::RichText::new("⚠️ IP: No external IP")
                        .color(egui::Color32::from_rgb(255, 193, 7))
                ).show(ui);
            }
            ui.add_space(4.0);

            // Firewall status (check operation state)
            let firewall_text = match &row.operation_state {
                crate::ui_tabs::platform::OperationState::InProgress { operation, .. }
                    if operation.to_lowercase().contains("firewall") =>
                {
                    ("🔄 Updating...".to_string(), egui::Color32::from_rgb(255, 152, 0))
                }
                crate::ui_tabs::platform::OperationState::Failed { operation, error, .. }
                    if operation.to_lowercase().contains("firewall") =>
                {
                    (format!("🔥 {}", error), egui::Color32::from_rgb(244, 67, 54))
                }
                _ => (format!("🔥 {}", row.firewall_status), ui.style().visuals.text_color()),
            };
            TwemojiLabel::new(egui::RichText::new(firewall_text.0).color(firewall_text.1)).show(ui);
            ui.add_space(4.0);

            // SSH status
            TwemojiLabel::new(format!("🔑 SSH: {}", row.ssh_status)).show(ui);
        } else {
            TwemojiLabel::new("💻 VM: — No VM created").show(ui);
        }
    } else {
        TwemojiLabel::new("📁 Project: — No project selected").show(ui);
    }

    ui.add_space(8.0);

    // SSH action menu (if available)
    if let (Some(external_ip), Some(private_key)) =
        (&row.vm_external_ip, &row.ssh_private_key)
    {
        ui.add_space(8.0);

        let ssh_command = format!(
            "K=$(mktemp) && cat > $K <<'EOF'\n{}\nEOF\nchmod 600 $K && ssh -i $K root@{} && rm $K",
            private_key.trim(),
            external_ip
        );

        let mut menu = ActionMenu::new("💻SSH");
        menu.add_action("Copy SSH Command");
        menu.add_action("Copy Private Key");
        menu.add_action("Copy IP Address");

        if let Some(action_idx) = menu.show(ui) {
            let text_to_copy = match action_idx {
                0 => &ssh_command,
                1 => private_key,
                2 => external_ip,
                _ => return,
            };

            ui.ctx().copy_text(text_to_copy.to_string());
        }
    } else if row.vm_external_ip.is_some() && row.ssh_keyring_domain.is_some() {
        ui.add_space(8.0);
        TwemojiLabel::new(
            egui::RichText::new("⚠️ SSH key not found in keyring")
                .color(egui::Color32::from_rgb(255, 152, 0))
        ).show(ui);
    }
}

/// Helper function to format project count display
fn format_project_count_display(count: Option<usize>) -> String {
    match count {
        Some(n) => format!("{} project{}", n, if n == 1 { "" } else { "s" }),
        None => "? projects".to_string(),
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
        ui.heading("Project Logs");
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
        ui.label("Level:");
        let filter_id = egui::Id::new("log_level_filter");
        let mut log_level_filter = ui.data(|d| d.get_temp::<String>(filter_id))
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

    if drawer_state.loading {
        ui.spinner();
        ui.label("Loading logs...");
        return;
    }

    if drawer_state.project_id.is_none() {
        ui.label("No project selected");
        return;
    }

    if drawer_state.logs.is_empty() {
        ui.label("No logs available");
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

    ui.heading("Operation History");
    ui.add_space(8.0);

    if drawer_state.loading {
        ui.spinner();
        ui.label("Loading operations...");
        return;
    }

    if drawer_state.operations.is_empty() {
        ui.label("No operations logged");
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
            header.col(|ui| { ui.heading("Date"); });
            header.col(|ui| { ui.heading("Operation"); });
            header.col(|ui| { ui.heading("System"); });
            header.col(|ui| { ui.heading("Status"); });
            header.col(|ui| { ui.heading("Duration"); });
            header.col(|ui| { ui.heading("Error"); });
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
                            "running" => ("⏳ Running", egui::Color32::from_rgb(100, 150, 255)),
                            "success" => ("✓ Success", egui::Color32::from_rgb(50, 200, 50)),
                            "failed" => ("✗ Failed", egui::Color32::from_rgb(255, 80, 80)),
                            _ => (&op.status[..], egui::Color32::GRAY),
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
