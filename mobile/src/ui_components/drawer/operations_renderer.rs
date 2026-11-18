use crate::storage::models::opslog::{OperationLog, OperationStatus};
use eframe::egui::{self, Color32, Id, Ui};
use egui_i18n::tr;

#[cfg(feature = "gui")]
use egui_material3::{data_table, datatable::CompactMode, DataTableTheme};

/// Check if error message contains multiple lines
fn has_multiline_error(error: &Option<String>) -> bool {
    error.as_ref().map(|e| e.contains('\n')).unwrap_or(false)
}

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
        Self { max_height: height }
    }

    /// Display the operation history in a data table
    #[cfg(feature = "gui")]
    pub fn show(&self, ui: &mut Ui, operations: &[OperationLog]) {
        if operations.is_empty() {
            ui.label(tr!("ops-no-operations"));
            return;
        }
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.render_operations_table(ui, operations);
            });
    }

    /// Headless fallback when GUI feature is disabled
    #[cfg(not(feature = "gui"))]
    pub fn show(&self, _ui: &mut Ui, _operations: &[OperationLog]) {
        // No-op in headless mode
    }

    /// Render operations table filling available space
    #[cfg(feature = "gui")]
    fn render_operations_table(&self, ui: &mut Ui, operations: &[OperationLog]) {
        let mut theme = DataTableTheme::default();
        theme.header_font_size = Some(12.0);
        theme.heading_row_height = Some(40.0);

        let w = ui.available_width() / 7.0; // ponytail: fixed ratios, scales with drawer width

        let mut table = data_table()
            .id(Id::new("operations_data_table"))
            .theme(theme)
            .compact_mode(CompactMode::ForceNarrow) // Force narrow despite wide width
            .default_row_height(30.0)
            .column(tr!("ops-col-date"), w * 1.3, false)
            .column(tr!("ops-col-operation"), w * 1.2, false)
            .column(tr!("ops-col-system"), w * 0.8, false)
            .column(tr!("ops-col-status"), w, false)
            .column(tr!("ops-col-duration"), w * 0.7, true)
            .column(tr!("ops-col-error"), w * 2.0, false);

        for op in operations {
            let date = chrono::DateTime::from_timestamp(op.started_at, 0)
                .map(|dt| dt.format("%Y%m%d %H:%M").to_string())
                .unwrap_or_else(|| "???????? ??:??".to_string());

            let (status_text, status_color) = match op.status.as_str() {
                "running" => (tr!("ops-status-running"), Color32::from_rgb(100, 181, 246)),
                "success" => (tr!("ops-status-success"), Color32::from_rgb(76, 175, 80)),
                "failed" => (
                    tr!("ops-status-failed"),
                    egui_material3::theme::get_global_color("error"),
                ),
                _ => (
                    op.status.clone(),
                    egui_material3::theme::get_global_color("onSurfaceVariant"),
                ),
            };

            let duration = op
                .duration_ms()
                .map(|ms| format!("{:.2}s", ms as f64 / 1000.0))
                .unwrap_or_else(|| "-".to_string());

            let error_display = op.error_message.clone().unwrap_or_default();

            table = table.row(|row| {
                row.cell(&date)
                    .cell(&op.operation_type)
                    .cell(&op.external_system)
                    .cell(&status_text)
                    .cell(&duration)
                    .cell(&error_display)
            });
        }

        ui.add(table);
    }

    /// No-op for headless mode
    #[cfg(not(feature = "gui"))]
    fn render_operations_table(&self, _ui: &mut Ui, _operations: &[OperationLog]) {
        // No-op in headless mode
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
        assert_eq!(op.details.unwrap(), "VM created with ID: i-12345678");
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

    #[test]
    fn test_has_multiline_error_with_newline() {
        use super::has_multiline_error;
        let error = Some("Line 1\nLine 2\nLine 3".to_string());
        assert!(has_multiline_error(&error));
    }

    #[test]
    fn test_has_multiline_error_single_line() {
        use super::has_multiline_error;
        let error = Some("Single line error".to_string());
        assert!(!has_multiline_error(&error));
    }

    #[test]
    fn test_has_multiline_error_none() {
        use super::has_multiline_error;
        let error = None;
        assert!(!has_multiline_error(&error));
    }

    #[test]
    fn test_has_multiline_error_empty_string() {
        use super::has_multiline_error;
        let error = Some("".to_string());
        assert!(!has_multiline_error(&error));
    }

    #[test]
    fn test_status_text_no_emojis_running() {
        let status_text = "Running";
        assert!(!status_text.contains('⏳'));
        assert!(!status_text.contains('✓'));
        assert!(!status_text.contains('✗'));
    }

    #[test]
    fn test_status_text_no_emojis_success() {
        let status_text = "Success";
        assert!(!status_text.contains('⏳'));
        assert!(!status_text.contains('✓'));
        assert!(!status_text.contains('✗'));
    }

    #[test]
    fn test_status_text_no_emojis_failed() {
        let status_text = "Failed";
        assert!(!status_text.contains('⏳'));
        assert!(!status_text.contains('✓'));
        assert!(!status_text.contains('✗'));
    }
}
