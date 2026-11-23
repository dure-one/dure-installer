//! Mycart crontab management dialog
//! Phase 3: Standalone dialog for crontab job management

use eframe::egui;
use egui_material3::{dialog, ListItem, MaterialButton, MaterialList};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::api::mycart::CrontabJob;

/// Crontab dialog state
pub struct CrontabDialog {
    pub site_address: String,
    pub jobs: Vec<CrontabJob>,
    pub loading: bool,
    pub error: Option<String>,
    /// Per-job interval edit buffers (job_id -> interval_buffer)
    pub interval_buffers: HashMap<String, String>,
    /// Edit dialog state
    edit_dialog_open: bool,
    editing_job_id: Option<String>,
    edit_enabled: bool,
    edit_interval: String,
}

impl CrontabDialog {
    pub fn new(site_address: String) -> Self {
        Self {
            site_address,
            jobs: Vec::new(),
            loading: true,
            error: None,
            interval_buffers: HashMap::new(),
            edit_dialog_open: false,
            editing_job_id: None,
            edit_enabled: false,
            edit_interval: String::new(),
        }
    }

    /// Derive job category from job_id pattern
    fn get_category(job_id: &str) -> &'static str {
        if job_id.contains("cleanup") || job_id.contains("maintenance") {
            "Maintenance"
        } else if job_id.contains("check") || job_id.contains("monitor") {
            "Monitoring"
        } else if job_id.contains("backup") {
            "Backup"
        } else {
            "System"
        }
    }

    pub fn set_jobs(&mut self, jobs: Vec<CrontabJob>) {
        // Initialize interval buffers from job data
        for job in &jobs {
            self.interval_buffers
                .insert(job.job_id.clone(), job.interval.clone());
        }
        self.jobs = jobs;
        self.loading = false;
        self.error = None;
    }

    pub fn set_error(&mut self, error: String) {
        self.error = Some(error);
        self.loading = false;
    }

    /// Open edit dialog for a specific job
    pub fn open_edit_dialog(&mut self, job_id: &str) {
        if let Some(job) = self.jobs.iter().find(|j| j.job_id == job_id) {
            self.editing_job_id = Some(job_id.to_string());
            self.edit_enabled = job.enabled;
            self.edit_interval = job.interval.clone();
            self.edit_dialog_open = true;
        }
    }

    /// Render the dialog, returns action
    pub fn render(&mut self, ctx: &egui::Context, open: &mut bool) -> Option<CrontabAction> {
        use std::cell::Cell;
        let action = Cell::new(None);

        dialog("mycart_crontab_dialog", "Manage Crontab Jobs", open)
            .min_width(700.0)
            .scrollable(true)
            .content(|ui, _open| {
                // Site header
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Site:").strong());
                    ui.label(&self.site_address);
                });
                ui.add_space(8.0);

                // Error display
                if let Some(ref error) = self.error {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 0, 0),
                        format!("⚠ Error: {}", error),
                    );
                    ui.add_space(8.0);
                }

                // Loading state
                if self.loading {
                    ui.label("Loading crontab jobs...");
                    ui.add_space(4.0);
                    ui.spinner();
                } else if self.jobs.is_empty() {
                    ui.label("No crontab jobs found.");
                } else {
                    // Editable drawer list with 5-line items
                    let edit_signal = self.render_job_list(ui);
                    if let Some(job_id) = edit_signal {
                        self.open_edit_dialog(&job_id);
                    }
                }
            })
            .primary_action("Close", || {})
            .show(ctx);

        // Show edit dialog if open
        if let Some(dialog_action) = self.show_edit_dialog(ctx) {
            action.set(Some(dialog_action));
        }

        action.into_inner()
    }

    fn render_job_list(&self, ui: &mut egui::Ui) -> Option<String> {
        let list_id = "crontab_jobs_list";
        let state = ui
            .data_mut(|d| {
                d.get_persisted::<egui_material3::MaterialListState>(egui::Id::new(list_id))
            })
            .unwrap_or_default();

        #[derive(Clone)]
        enum ListAction {
            Edit(String),
            ToggleEnabled(String, bool),
        }

        let action_signal: Arc<Mutex<Option<ListAction>>> = Arc::new(Mutex::new(None));

        let mut list = MaterialList::new().id(list_id).allow_drawer(true);

        for (index, job) in self.jobs.iter().enumerate() {
            let action_clone = Arc::clone(&action_signal);
            let job_id = job.job_id.clone();
            let is_open = state.drawer_open_items.contains(&index);
            let indicator = if is_open {
                "Collapse ▼"
            } else {
                "Expand ▶"
            };

            let category = Self::get_category(&job.job_id);
            let enabled = job.enabled;
            let interval = job.interval.clone();
            let last_run = job.last_run;

            let list_item = ListItem::new(&job.name)
                .overline(category)
                .secondary_text(format!("ID: {}\nSchedule: {}", job.job_id, job.interval))
                .leading_icon("schedule")
                .leading_icon_label(category)
                .drawer(move |ui| {
                    ui.label("Job Details:");
                    ui.label(format!("• Enabled: {}", if enabled { "Yes" } else { "No" }));
                    ui.label(format!("• Interval: {}", interval));
                    if let Some(ts) = last_run {
                        use chrono::{DateTime, Utc};
                        let dt = DateTime::<Utc>::from_timestamp(ts, 0).unwrap_or_default();
                        ui.label(format!("• Last run: {}", dt.format("%Y-%m-%d %H:%M:%S")));
                    } else {
                        ui.label("• Last run: never");
                    }
                })
                .trailing_widget(move |ui| {
                    ui.vertical(|ui| {
                        // Indicator at top
                        ui.add_space(2.0);
                        ui.label(indicator);

                        ui.add_space(8.0);

                        // Edit button
                        if ui.add(MaterialButton::filled("Edit").small()).clicked() {
                            if let Ok(mut signal) = action_clone.lock() {
                                *signal = Some(ListAction::Edit(job_id.clone()));
                            }
                        }
                    });
                });

            list = list.item(list_item);
        }

        ui.add(list);

        // Handle action after render
        let list_action = {
            if let Ok(mut signal) = action_signal.lock() {
                signal.take()
            } else {
                None
            }
        };

        if let Some(list_action) = list_action {
            match list_action {
                ListAction::Edit(job_id) => {
                    return Some(job_id);
                }
                ListAction::ToggleEnabled(_job_id, _enabled) => {
                    // Not used currently, kept for future
                    return None;
                }
            }
        }

        None
    }

    fn show_edit_dialog(&mut self, ctx: &egui::Context) -> Option<CrontabAction> {
        if !self.edit_dialog_open {
            return None;
        }

        let was_open = self.edit_dialog_open;
        let save_clicked = Arc::new(Mutex::new(false));
        let save_clone = Arc::clone(&save_clicked);

        let job_name = self
            .editing_job_id
            .as_ref()
            .and_then(|id| self.jobs.iter().find(|j| &j.job_id == id))
            .map(|j| j.name.as_str())
            .unwrap_or("Unknown");

        dialog(
            "edit_crontab_dialog",
            "Edit Crontab Job",
            &mut self.edit_dialog_open,
        )
        .max_width(560.0)
        .content(|ui, _open| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Job:");
                    ui.label(egui::RichText::new(job_name).strong());
                });
                ui.add_space(8.0);

                ui.checkbox(&mut self.edit_enabled, "Enabled");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Interval:");
                    ui.text_edit_singleline(&mut self.edit_interval);
                });
                ui.label("Example: 0 0 * * * (daily at midnight)");
            });
        })
        .action("Cancel", || {})
        .primary_action("Save", move || {
            if let Ok(mut clicked) = save_clone.lock() {
                *clicked = true;
            }
        })
        .show(ctx);

        // Commit changes only if Save was clicked
        if was_open && !self.edit_dialog_open {
            let should_save = save_clicked.lock().map(|v| *v).unwrap_or(false);
            if should_save {
                if let Some(job_id) = self.editing_job_id.take() {
                    log::info!(
                        "Crontab edit dialog: Returning UpdateJob action for job_id={}, enabled={}, interval={}",
                        job_id, self.edit_enabled, self.edit_interval
                    );
                    return Some(CrontabAction::UpdateJob {
                        job_id,
                        patch: crate::api::mycart::CrontabPatch {
                            enabled: Some(self.edit_enabled),
                            interval: Some(self.edit_interval.clone()),
                        },
                    });
                }
            }
        }

        None
    }
}

/// Actions from crontab dialog
#[derive(Debug, Clone)]
pub enum CrontabAction {
    ToggleJob {
        job_id: String,
        enabled: bool,
    },
    SetInterval {
        job_id: String,
        interval: String,
    },
    UpdateJob {
        job_id: String,
        patch: crate::api::mycart::CrontabPatch,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crontab_dialog_initial_state() {
        let dialog = CrontabDialog::new("site.example.com".to_string());
        assert_eq!(dialog.site_address, "site.example.com");
        assert!(dialog.jobs.is_empty());
        assert!(dialog.loading);
        assert!(dialog.error.is_none());
    }

    #[test]
    fn test_crontab_dialog_set_jobs() {
        let mut dialog = CrontabDialog::new("site.com".to_string());

        let jobs = vec![CrontabJob {
            job_id: "cleanup".to_string(),
            name: "Cleanup Task".to_string(),
            enabled: true,
            interval: "0 0 * * *".to_string(),
            last_run: None,
        }];

        dialog.set_jobs(jobs.clone());

        assert_eq!(dialog.jobs.len(), 1);
        assert!(!dialog.loading);
        assert!(dialog.error.is_none());
        assert_eq!(dialog.interval_buffers.get("cleanup").unwrap(), "0 0 * * *");
    }

    #[test]
    fn test_crontab_dialog_set_error() {
        let mut dialog = CrontabDialog::new("site.com".to_string());
        dialog.set_error("Connection failed".to_string());

        assert!(!dialog.loading);
        assert_eq!(dialog.error, Some("Connection failed".to_string()));
    }

    #[test]
    fn test_job_category_derivation() {
        assert_eq!(CrontabDialog::get_category("cleanup_job"), "Maintenance");
        assert_eq!(CrontabDialog::get_category("xmpp_check_job"), "Monitoring");
        assert_eq!(CrontabDialog::get_category("backup_daily"), "Backup");
        assert_eq!(CrontabDialog::get_category("other_task"), "System");
        assert_eq!(
            CrontabDialog::get_category("maintenance_weekly"),
            "Maintenance"
        );
        assert_eq!(CrontabDialog::get_category("monitor_health"), "Monitoring");
    }

    #[test]
    fn test_crontab_dialog_edit_state() {
        let mut dialog = CrontabDialog::new("site.com".to_string());
        assert!(!dialog.edit_dialog_open);
        assert!(dialog.editing_job_id.is_none());

        // Set jobs
        let jobs = vec![CrontabJob {
            job_id: "test_job".to_string(),
            name: "Test Job".to_string(),
            enabled: true,
            interval: "0 0 * * *".to_string(),
            last_run: None,
        }];
        dialog.set_jobs(jobs);

        // Open edit dialog
        dialog.open_edit_dialog("test_job");
        assert!(dialog.edit_dialog_open);
        assert_eq!(dialog.editing_job_id, Some("test_job".to_string()));
        assert!(dialog.edit_enabled);
        assert_eq!(dialog.edit_interval, "0 0 * * *");
    }

    #[test]
    fn test_crontab_action_update_job() {
        // Verify CrontabAction::UpdateJob creates correct patch
        let patch = crate::api::mycart::CrontabPatch {
            enabled: Some(false),
            interval: Some("0 1 * * *".to_string()),
        };

        let action = CrontabAction::UpdateJob {
            job_id: "test_job".to_string(),
            patch: patch.clone(),
        };

        match action {
            CrontabAction::UpdateJob { job_id, patch } => {
                assert_eq!(job_id, "test_job");
                assert_eq!(patch.enabled, Some(false));
                assert_eq!(patch.interval, Some("0 1 * * *".to_string()));
            }
            _ => panic!("Wrong action type"),
        }
    }
}
