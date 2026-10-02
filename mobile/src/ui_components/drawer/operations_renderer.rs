use crate::storage::models::opslog::{OperationLog, OperationStatus};
use eframe::egui::{self, Color32, RichText, ScrollArea, Ui};
use egui_extras::{Column, TableBuilder};

/// Renders operation history from the operation log database
pub struct OperationsRenderer {
    max_height: f32,
}

impl OperationsRenderer {
    /// Create a new OperationsRenderer with default settings
    pub fn new() -> Self {
        Self { max_height: 400.0 }
    }

    /// Set the maximum height of the scrollable area
    pub fn max_height(self, height: f32) -> Self {
        Self {
            max_height: height,
        }
    }

    /// Display the operation history in a data table
    pub fn show(&self, ui: &mut Ui, operations: &[OperationLog]) {
        if operations.is_empty() {
            ui.heading("Operation History");
            ui.separator();
            ui.label("No operations yet");
            return;
        }

        ScrollArea::vertical()
            .max_height(self.max_height)
            .show(ui, |ui| {
                self.render_operations_table(ui, operations);
            });
    }

    /// Render operations table with 6 columns
    fn render_operations_table(&self, ui: &mut Ui, operations: &[OperationLog]) {
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
                                "running" => ("⏳ Running", Color32::from_rgb(100, 150, 255)),
                                "success" => ("✓ Success", Color32::from_rgb(50, 200, 50)),
                                "failed" => ("✗ Failed", Color32::from_rgb(255, 80, 80)),
                                _ => (&op.status[..], Color32::GRAY),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a test OperationLog
    fn create_test_op(
        id: i64,
        operation_type: &str,
        status: OperationStatus,
        external_system: &str,
    ) -> OperationLog {
        OperationLog {
            id,
            operation_source: "test".to_string(),
            relevant_id: "test-id".to_string(),
            project_id: "test-project".to_string(),
            operation_type: operation_type.to_string(),
            external_system: external_system.to_string(),
            status: status.as_str().to_string(),
            started_at: 1000,
            completed_at: Some(1100),
            error_message: None,
            details: None,
        }
    }

    #[test]
    fn test_operations_renderer_new() {
        let renderer = OperationsRenderer::new();
        assert_eq!(renderer.max_height, 400.0);
    }

    #[test]
    fn test_operations_renderer_default() {
        let renderer = OperationsRenderer::default();
        assert_eq!(renderer.max_height, 400.0);
    }

    #[test]
    fn test_operations_renderer_max_height() {
        let renderer = OperationsRenderer::new().max_height(600.0);
        assert_eq!(renderer.max_height, 600.0);
    }

    #[test]
    fn test_operations_renderer_max_height_chain() {
        let renderer = OperationsRenderer::new()
            .max_height(500.0)
            .max_height(800.0);
        assert_eq!(renderer.max_height, 800.0);
    }

    #[test]
    fn test_operation_log_success_status() {
        let op = create_test_op(1, "create_vm", OperationStatus::Success, "gcp");
        assert_eq!(op.status(), OperationStatus::Success);
    }

    #[test]
    fn test_operation_log_running_status() {
        let mut op = create_test_op(1, "create_vm", OperationStatus::Running, "gcp");
        op.status = OperationStatus::Running.as_str().to_string();
        assert_eq!(op.status(), OperationStatus::Running);
    }

    #[test]
    fn test_operation_log_failed_status() {
        let op = create_test_op(1, "create_vm", OperationStatus::Failed, "gcp");
        assert_eq!(op.status(), OperationStatus::Failed);
    }

    #[test]
    fn test_operation_log_duration_calculation() {
        let op = OperationLog {
            id: 1,
            operation_source: "test".to_string(),
            relevant_id: "test-id".to_string(),
            project_id: "test-project".to_string(),
            operation_type: "create_vm".to_string(),
            external_system: "gcp".to_string(),
            status: "success".to_string(),
            started_at: 1000,
            completed_at: Some(1100),
            error_message: None,
            details: None,
        };

        let duration = op.duration_ms();
        assert!(duration.is_some());
        assert_eq!(duration.unwrap(), 100000);
    }

    #[test]
    fn test_operation_log_duration_none_when_running() {
        let mut op = create_test_op(1, "create_vm", OperationStatus::Running, "gcp");
        op.completed_at = None;
        assert_eq!(op.duration_ms(), None);
    }

    #[test]
    fn test_operation_log_with_error() {
        let mut op = create_test_op(1, "create_vm", OperationStatus::Failed, "gcp");
        op.error_message = Some("Connection timeout".to_string());
        assert!(op.error_message.is_some());
        assert_eq!(op.error_message.unwrap(), "Connection timeout");
    }

    #[test]
    fn test_operation_log_with_details() {
        let mut op = create_test_op(1, "create_vm", OperationStatus::Success, "gcp");
        op.details = Some("VM created with ID: i-12345678".to_string());
        assert!(op.details.is_some());
        assert_eq!(
            op.details.unwrap(),
            "VM created with ID: i-12345678"
        );
    }

    #[test]
    fn test_operation_status_as_str_running() {
        assert_eq!(OperationStatus::Running.as_str(), "running");
    }

    #[test]
    fn test_operation_status_as_str_success() {
        assert_eq!(OperationStatus::Success.as_str(), "success");
    }

    #[test]
    fn test_operation_status_as_str_failed() {
        assert_eq!(OperationStatus::Failed.as_str(), "failed");
    }

    #[test]
    fn test_operation_status_from_str_running() {
        let status = OperationStatus::from_str("running");
        assert_eq!(status, OperationStatus::Running);
    }

    #[test]
    fn test_operation_status_from_str_success() {
        let status = OperationStatus::from_str("success");
        assert_eq!(status, OperationStatus::Success);
    }

    #[test]
    fn test_operation_status_from_str_failed() {
        let status = OperationStatus::from_str("failed");
        assert_eq!(status, OperationStatus::Failed);
    }

    #[test]
    fn test_operation_status_from_str_invalid_defaults_to_failed() {
        let status = OperationStatus::from_str("invalid_status");
        assert_eq!(status, OperationStatus::Failed);
    }

    #[test]
    fn test_multiple_operations() {
        let ops = vec![
            create_test_op(1, "create_vm", OperationStatus::Success, "gcp"),
            create_test_op(2, "update_firewall", OperationStatus::Running, "gcp"),
            create_test_op(3, "delete_instance", OperationStatus::Failed, "gcp"),
        ];

        assert_eq!(ops.len(), 3);
        assert_eq!(ops[0].operation_type, "create_vm");
        assert_eq!(ops[1].operation_type, "update_firewall");
        assert_eq!(ops[2].operation_type, "delete_instance");
    }

    #[test]
    fn test_operation_with_all_fields() {
        let op = OperationLog {
            id: 42,
            operation_source: "platform".to_string(),
            relevant_id: "proj-123".to_string(),
            project_id: "proj-123".to_string(),
            operation_type: "create_vm".to_string(),
            external_system: "gcp".to_string(),
            status: "success".to_string(),
            started_at: 1000,
            completed_at: Some(1500),
            error_message: Some("Some warning".to_string()),
            details: Some("Details here".to_string()),
        };

        assert_eq!(op.id, 42);
        assert_eq!(op.operation_source, "platform");
        assert_eq!(op.relevant_id, "proj-123");
        assert_eq!(op.project_id, "proj-123");
        assert_eq!(op.operation_type, "create_vm");
        assert_eq!(op.external_system, "gcp");
        assert_eq!(op.status, "success");
        assert_eq!(op.started_at, 1000);
        assert_eq!(op.completed_at, Some(1500));
        assert!(op.error_message.is_some());
        assert!(op.details.is_some());
    }
}
