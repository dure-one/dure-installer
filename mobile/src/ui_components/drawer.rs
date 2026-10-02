//! Reusable drawer components for logs and operations tabs

use eframe::egui;
use crate::storage::models::opslog::OperationLog;

/// Logs renderer with auto-refresh, level filtering, scrollable view
pub struct LogsRenderer {
    title: String,
    refresh_interval_secs: u64,
}

impl LogsRenderer {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            refresh_interval_secs: 1,
        }
    }

    pub fn show(
        &self,
        ui: &mut egui::Ui,
        logs: &[String],
        refresh_action_id: egui::Id,
        entity_id: &Option<String>,
    ) {
        // Auto-refresh trigger
        if let Some(ref id) = entity_id {
            let refresh_id = egui::Id::new(format!("{}_log_last_refresh", refresh_action_id));
            let now = std::time::Instant::now();
            let should_refresh = ui.data(|d| {
                d.get_temp::<std::time::Instant>(refresh_id)
                    .map(|last| now.duration_since(last).as_secs() >= self.refresh_interval_secs)
                    .unwrap_or(true)
            });

            if should_refresh {
                ui.data_mut(|d| {
                    d.insert_temp(refresh_id, now);
                    d.insert_temp(refresh_action_id, id.clone());
                });
            }
        }

        ui.horizontal(|ui| {
            ui.heading(&self.title);
            ui.add_space(8.0);

            // New logs indicator
            let log_count_id = egui::Id::new(format!("{}_prev_log_count", refresh_action_id));
            let current_count = logs.len();
            let prev_count = ui.data(|d| d.get_temp::<usize>(log_count_id)).unwrap_or(0);

            if current_count > prev_count {
                ui.label(
                    egui::RichText::new("NEW")
                        .color(egui::Color32::from_rgb(76, 175, 80))
                        .strong(),
                );
                ui.ctx().request_repaint();
            }

            ui.data_mut(|d| d.insert_temp(log_count_id, current_count));

            ui.add_space(8.0);

            // Log level filter
            ui.label("Level:");
            let filter_id = egui::Id::new(format!("{}_log_level_filter", refresh_action_id));
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

        if entity_id.is_none() {
            ui.label("No entity selected");
            return;
        }

        if logs.is_empty() {
            ui.label("No logs available");
            return;
        }

        // Filter logs by level
        let log_level_filter = ui
            .data(|d| d.get_temp::<String>(egui::Id::new(format!("{}_log_level_filter", refresh_action_id))))
            .unwrap_or_else(|| "All".to_string());

        let filtered_logs: Vec<&String> = if log_level_filter == "All" {
            logs.iter().collect()
        } else {
            let filter_str = format!("[{}]", log_level_filter);
            logs.iter().filter(|line| line.contains(&filter_str)).collect()
        };

        // Limit to latest 100 lines
        let start_idx = filtered_logs.len().saturating_sub(100);
        let latest_logs = &filtered_logs[start_idx..];

        // Scrollable log view
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(500.0)
            .show(ui, |ui| {
                ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.0));

                for line in latest_logs {
                    ui.label(*line);
                }
            });
    }
}

/// Operations renderer with auto-refresh, table view
pub struct OperationsRenderer {
    refresh_interval_secs: u64,
}

impl OperationsRenderer {
    pub fn new() -> Self {
        Self {
            refresh_interval_secs: 2,
        }
    }

    pub fn show(
        &self,
        ui: &mut egui::Ui,
        operations: &[OperationLog],
        refresh_action_id: egui::Id,
        entity_id: &Option<String>,
    ) {
        // Auto-refresh trigger
        if let Some(ref id) = entity_id {
            let refresh_id = egui::Id::new(format!("{}_ops_last_refresh", refresh_action_id));
            let now = std::time::Instant::now();
            let should_refresh = ui.data(|d| {
                d.get_temp::<std::time::Instant>(refresh_id)
                    .map(|last| now.duration_since(last).as_secs() >= self.refresh_interval_secs)
                    .unwrap_or(true)
            });

            if should_refresh {
                ui.data_mut(|d| {
                    d.insert_temp(refresh_id, now);
                    d.insert_temp(refresh_action_id, id.clone());
                });
            }
        }

        ui.heading("Operations History");
        ui.add_space(8.0);

        if entity_id.is_none() {
            ui.label("No entity selected");
            return;
        }

        if operations.is_empty() {
            ui.label("No operations recorded");
            return;
        }

        // Scrollable operations table
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .max_height(500.0)
            .show(ui, |ui| {
                self.render_operations_table(ui, operations);
            });
    }

    fn render_operations_table(&self, ui: &mut egui::Ui, operations: &[OperationLog]) {
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
                header.col(|ui| {
                    ui.heading("Date");
                });
                header.col(|ui| {
                    ui.heading("Operation");
                });
                header.col(|ui| {
                    ui.heading("System");
                });
                header.col(|ui| {
                    ui.heading("Status");
                });
                header.col(|ui| {
                    ui.heading("Duration");
                });
                header.col(|ui| {
                    ui.heading("Error");
                });
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
}

impl Default for OperationsRenderer {
    fn default() -> Self {
        Self::new()
    }
}
