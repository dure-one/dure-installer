//! Database debugger dialog

pub use super::db_debugger_stt::*;
use diesel::prelude::*;
use diesel::sql_types::Text;
use eframe::egui;

impl DlgDbDebugger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.open = true;
        self.data_loaded = false;
        self.error_message = None;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    /// Reset key sequence (called on timeout)
    pub fn reset_sequence(&mut self) {
        self.key_sequence_count = 0;
        self.last_key_time = 0.0;
    }

    /// Load table list from database
    /// ponytail: Only show tables we actually support
    pub fn load_tables(&mut self, conn: &mut SqliteConnection) {
        // Hardcode supported tables instead of querying - simpler and avoids errors
        self.table_names = vec![
            "authenticated_devices".to_string(),
            "crypt_keys".to_string(),
            "sessions".to_string(),
            "sites".to_string(),
            "operations_log".to_string(),
            "audit_log".to_string(),
        ];

        if !self.table_names.is_empty() && self.selected_table.is_empty() {
            self.selected_table = self.table_names[0].clone();
        }
        self.error_message = None;
    }

    /// Load data from selected table using existing model functions
    /// ponytail: Reuse existing list_* functions from models - no new code needed
    pub fn load_table_data(&mut self, conn: &mut SqliteConnection) {
        if self.selected_table.is_empty() {
            return;
        }

        // Call existing model functions based on table name
        let result = match self.selected_table.as_str() {
            "authenticated_devices" => self.load_device_data(conn),
            "crypt_keys" => self.load_crypt_data(conn),
            "sessions" => self.load_session_data(conn),
            "sites" => self.load_site_data(conn),
            "operations_log" => self.load_opslog_data(conn),
            "audit_log" => self.load_audit_data(conn),
            "dns_records" => self.load_dns_data(conn),
            _ => {
                self.error_message = Some(format!("Unknown table: {}", self.selected_table));
                return;
            }
        };

        match result {
            Ok((cols, rows)) => {
                self.column_names = cols;
                self.table_data = rows;
                self.data_loaded = true;
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(e);
                self.table_data = vec![];
            }
        }
    }

    fn load_device_data(&self, conn: &mut SqliteConnection) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
        use crate::storage::models::device;
        let devices = device::list_authenticated_devices(conn)
            .map_err(|e| format!("Failed to load devices: {}", e))?;

        let cols = vec!["device_id".to_string(), "public_key".to_string(), "session_id".to_string(),
                       "authenticated_at".to_string(), "last_seen".to_string()];
        let rows = devices.into_iter().map(|d| {
            vec![d.device_id, d.public_key, d.session_id,
                 d.authenticated_at.to_string(), d.last_seen.to_string()]
        }).collect();

        Ok((cols, rows))
    }

    fn load_crypt_data(&self, conn: &mut SqliteConnection) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
        use crate::storage::models::crypt;
        let keys = crypt::list_device_keys(conn)
            .map_err(|e| format!("Failed to load crypt keys: {}", e))?;

        let cols = vec!["device_id".to_string(), "private_key".to_string(), "public_key".to_string(),
                       "created_at".to_string()];
        let rows = keys.into_iter().map(|k| {
            vec![k.device_id,
                 format!("{:?}", k.private_key),
                 format!("{:?}", k.public_key),
                 k.created_at.to_string()]
        }).collect();

        Ok((cols, rows))
    }

    fn load_session_data(&self, conn: &mut SqliteConnection) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
        use crate::storage::models::session;
        // Show sessions active in last 24 hours
        let sessions = session::list_active_sessions(conn, 24 * 3600)
            .map_err(|e| format!("Failed to load sessions: {}", e))?;

        let cols = vec!["session_id".to_string(), "domain".to_string(), "session_type".to_string(),
                       "connected_at".to_string(), "last_seen".to_string(), "request_count".to_string()];
        let rows = sessions.into_iter().map(|s| {
            vec![s.session_id, s.domain, s.session_type,
                 s.connected_at.to_string(), s.last_seen.to_string(), s.request_count.to_string()]
        }).collect();

        Ok((cols, rows))
    }

    fn load_site_data(&self, conn: &mut SqliteConnection) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
        use crate::storage::models::site;
        let sites = site::list_sites(conn)
            .map_err(|e| format!("Failed to load sites: {}", e))?;

        let cols = vec!["domain".to_string(), "public_key".to_string(), "status".to_string(),
                       "last_seen".to_string(), "created_at".to_string()];
        let rows = sites.into_iter().map(|s| {
            vec![s.domain, s.public_key, s.status,
                 s.last_seen.map(|t| t.to_string()).unwrap_or_default(),
                 s.created_at.to_string()]
        }).collect();

        Ok((cols, rows))
    }

    fn load_opslog_data(&self, conn: &mut SqliteConnection) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
        use crate::storage::models::opslog;
        let logs = opslog::list_all(conn, 1000)
            .map_err(|e| format!("Failed to load operations: {}", e))?;

        let cols = vec!["id".to_string(), "operation_type".to_string(), "status".to_string(),
                       "project_id".to_string(), "started_at".to_string()];
        let rows = logs.into_iter().map(|op| {
            vec![op.id.to_string(), op.operation_type, op.status,
                 op.project_id, op.started_at.to_string()]
        }).collect();

        Ok((cols, rows))
    }

    fn load_audit_data(&self, conn: &mut SqliteConnection) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
        use crate::storage::models::audit;
        let audits = audit::list_recent(conn, 1000)
            .map_err(|e| format!("Failed to load audit logs: {}", e))?;

        let cols = vec!["id".to_string(), "timestamp".to_string(), "actor_id".to_string(),
                       "action".to_string(), "object".to_string(), "outcome".to_string()];
        let rows = audits.into_iter().map(|a| {
            vec![a.id.to_string(), a.timestamp.to_string(), a.actor_id,
                 a.action, a.object, a.outcome]
        }).collect();

        Ok((cols, rows))
    }

    fn load_dns_data(&self, conn: &mut SqliteConnection) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
        use crate::storage::models::dns;
        use crate::calc::dns::RecordType;
        // Get all A records as a sample (DNS cache is per-domain and type)
        // ponytail: Just show empty result - DNS cache needs specific domain
        let cols = vec!["domain".to_string(), "record_type".to_string(), "value".to_string(),
                       "ttl".to_string()];
        let rows = vec![]; // Empty - DNS cache requires specific lookups

        Ok((cols, rows))
    }

    pub fn show(&mut self, ctx: &egui::Context, conn: Option<&mut SqliteConnection>) {
        if !self.open {
            return;
        }

        let mut close_clicked = false;

        egui::Window::new("🔍 Database Debugger (Hidden)")
            .id(egui::Id::new("db_debugger_window"))
            .resizable(true)
            .collapsible(false)
            .scroll([false, true])
            .default_width(800.0)
            .default_height(600.0)
            .show(ctx, |ui| {
                ui.heading("Database Debugger");
                ui.label("⚠️ Hidden debug tool - Ctrl+Shift+D x3 to open");
                ui.separator();

                if let Some(err) = &self.error_message {
                    ui.colored_label(egui::Color32::RED, format!("Error: {}", err));
                    ui.separator();
                }

                if let Some(conn) = conn {
                    // Load tables if not loaded
                    if self.table_names.is_empty() {
                        self.load_tables(conn);
                    }

                    // Table selector
                    ui.horizontal(|ui| {
                        ui.label("Table:");

                        let prev_table = self.selected_table.clone();
                        egui::ComboBox::from_id_salt("table_selector")
                            .selected_text(&self.selected_table)
                            .show_ui(ui, |ui| {
                                for table in &self.table_names {
                                    ui.selectable_value(
                                        &mut self.selected_table,
                                        table.clone(),
                                        table,
                                    );
                                }
                            });

                        // Reload if table changed
                        if prev_table != self.selected_table {
                            self.data_loaded = false;
                        }

                        if ui.button("🔄 Reload").clicked() {
                            self.data_loaded = false;
                        }
                    });

                    ui.separator();

                    // Load data if needed
                    if !self.data_loaded && !self.selected_table.is_empty() {
                        self.load_table_data(conn);
                    }

                    // Display data
                    if self.data_loaded {
                        ui.label(format!(
                            "Showing {} rows from table: {}",
                            self.table_data.len(),
                            self.selected_table
                        ));

                        egui::ScrollArea::both()
                            .max_height(ui.available_height() - 40.0)
                            .show(ui, |ui| {
                                if self.table_data.is_empty() {
                                    ui.label("No data in table");
                                } else {
                                    // Table header
                                    egui::Grid::new("db_table_grid")
                                        .striped(true)
                                        .show(ui, |ui| {
                                            // Header row
                                            for col in &self.column_names {
                                                ui.strong(col);
                                            }
                                            ui.end_row();

                                            // Data rows
                                            for row in &self.table_data {
                                                for cell in row {
                                                    ui.label(cell);
                                                }
                                                ui.end_row();
                                            }
                                        });
                                }
                            });
                    }
                } else {
                    ui.label("❌ No database connection (no profile loaded)");
                }

                ui.separator();
                if ui.button("Close").clicked() {
                    close_clicked = true;
                }
            });

        if close_clicked {
            self.close();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let dlg = DlgDbDebugger::new();
        assert!(!dlg.open);
    }

    #[test]
    fn test_open_close() {
        let mut dlg = DlgDbDebugger::new();
        assert!(!dlg.open);

        dlg.open();
        assert!(dlg.open);

        dlg.close();
        assert!(!dlg.open);
    }

    #[test]
    fn test_reset_sequence() {
        let mut dlg = DlgDbDebugger::new();
        dlg.key_sequence_count = 3;
        dlg.last_key_time = 100.0;

        dlg.reset_sequence();
        assert_eq!(dlg.key_sequence_count, 0);
        assert_eq!(dlg.last_key_time, 0.0);
    }
}
