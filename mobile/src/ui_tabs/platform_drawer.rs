//! Platform drawer UI with Status/Logs/Operations tabs

use eframe::egui;

use crate::storage::models::opslog::OperationLog;
use crate::viewmodel::platform::{DrawerTab, DrawerState};

/// Render the platform drawer with tabs
pub fn render_drawer(
    ui: &mut egui::Ui,
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
        DrawerTab::Status => render_status_tab(ui, drawer_state),
        DrawerTab::Logs => render_logs_tab(ui, drawer_state),
        DrawerTab::Operations => render_operations_tab(ui, drawer_state),
    }
}

/// Render Status tab (existing drawer content)
fn render_status_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    ui.heading("Platform Status");
    ui.add_space(8.0);

    if let Some(project_id) = &drawer_state.project_id {
        ui.label(format!("Project: {}", project_id));
    } else {
        ui.label("No project selected");
    }

    ui.add_space(8.0);

    // Placeholder for actual status content
    // This will be replaced with the existing drawer content from platform.rs
    ui.label("Status information will appear here");
}

/// Render Logs tab (stdout logs filtered by project)
fn render_logs_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    ui.heading("Stdout Logs");
    ui.add_space(8.0);

    if drawer_state.loading {
        ui.spinner();
        ui.label("Loading logs...");
        return;
    }

    if drawer_state.logs.is_empty() {
        ui.label("No logs available");
        return;
    }

    // Scrollable log view with fixed-width font
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.0));

            for line in &drawer_state.logs {
                ui.label(line);
            }
        });
}

/// Render Operations tab (operation logs from SQLite)
fn render_operations_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
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

    // Render operations table with 30px row height
    render_operations_table(ui, &drawer_state.operations);
}

/// Render operations table with simple grid layout (30px rows)
fn render_operations_table(ui: &mut egui::Ui, operations: &[OperationLog]) {
    use egui_extras::{Column, TableBuilder};

    TableBuilder::new(ui)
        .striped(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(80.0))  // Time
        .column(Column::auto().at_least(120.0)) // Operation
        .column(Column::auto().at_least(80.0))  // System
        .column(Column::auto().at_least(80.0))  // Status
        .column(Column::auto().at_least(60.0))  // Duration
        .column(Column::remainder())             // Error
        .header(20.0, |mut header| {
            header.col(|ui| { ui.heading("Time"); });
            header.col(|ui| { ui.heading("Operation"); });
            header.col(|ui| { ui.heading("System"); });
            header.col(|ui| { ui.heading("Status"); });
            header.col(|ui| { ui.heading("Duration"); });
            header.col(|ui| { ui.heading("Error"); });
        })
        .body(|mut body| {
            for op in operations {
                body.row(30.0, |mut row| {
                    // Time
                    row.col(|ui| {
                        let dt = chrono::DateTime::from_timestamp(op.started_at, 0)
                            .map(|dt| dt.format("%H:%M:%S").to_string())
                            .unwrap_or_else(|| "??:??:??".to_string());
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
