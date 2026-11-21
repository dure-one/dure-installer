//! Mycart crontab management dialog
//! Phase 3: Standalone dialog for crontab job management

use eframe::egui;
use egui_material3::{dialog, MaterialButton};
use std::collections::HashMap;

use crate::api::mycart::CrontabJob;

/// Crontab dialog state
pub struct CrontabDialog {
    pub site_address: String,
    pub jobs: Vec<CrontabJob>,
    pub loading: bool,
    pub error: Option<String>,
    /// Per-job interval edit buffers (job_id -> interval_buffer)
    pub interval_buffers: HashMap<String, String>,
}

impl CrontabDialog {
    pub fn new(site_address: String) -> Self {
        Self {
            site_address,
            jobs: Vec::new(),
            loading: true,
            error: None,
            interval_buffers: HashMap::new(),
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

    /// Render the dialog, returns action
    pub fn render(&mut self, ctx: &egui::Context, open: &mut bool) -> Option<CrontabAction> {
        use std::cell::Cell;
        let action = Cell::new(None);

        dialog("mycart_crontab_dialog", "Manage Crontab Jobs", open)
            .min_width(600.0)
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
                    // Job list with editing controls
                    egui::ScrollArea::vertical()
                        .max_height(400.0)
                        .show(ui, |ui| {
                            for job in &self.jobs {
                                ui.group(|ui| {
                                    ui.set_min_width(550.0);

                                    // Job name and ID
                                    ui.label(egui::RichText::new(&job.name).strong());
                                    ui.label(format!("ID: {}", job.job_id));
                                    ui.add_space(4.0);

                                    // Enabled checkbox
                                    ui.horizontal(|ui| {
                                        let mut enabled = job.enabled;
                                        if ui.checkbox(&mut enabled, "Enabled").changed() {
                                            action.set(Some(CrontabAction::ToggleJob {
                                                job_id: job.job_id.clone(),
                                                enabled,
                                            }));
                                        }
                                    });

                                    // Interval editor
                                    ui.horizontal(|ui| {
                                        ui.label("Interval:");

                                        let buffer = self
                                            .interval_buffers
                                            .entry(job.job_id.clone())
                                            .or_insert_with(|| job.interval.clone());

                                        ui.text_edit_singleline(buffer);

                                        // Apply button (only enabled if changed)
                                        let changed = buffer != &job.interval;
                                        if ui
                                            .add_enabled(changed, MaterialButton::filled("Apply"))
                                            .clicked()
                                        {
                                            action.set(Some(CrontabAction::SetInterval {
                                                job_id: job.job_id.clone(),
                                                interval: buffer.clone(),
                                            }));
                                        }
                                    });

                                    // Last run timestamp
                                    if let Some(last_run) = job.last_run {
                                        use chrono::{DateTime, Utc};
                                        let dt = DateTime::<Utc>::from_timestamp(last_run, 0)
                                            .unwrap_or_default();
                                        ui.label(format!(
                                            "Last run: {}",
                                            dt.format("%Y-%m-%d %H:%M")
                                        ));
                                    } else {
                                        ui.label("Last run: never");
                                    }
                                });
                                ui.add_space(8.0);
                            }
                        });
                }

                ui.add_space(8.0);

                // Close button
                ui.horizontal(|ui| {
                    if ui.add(MaterialButton::outlined("Close")).clicked() {
                        *_open = false;
                    }
                });
            })
            .show(ctx);

        action.into_inner()
    }
}

/// Actions from crontab dialog
#[derive(Debug, Clone)]
pub enum CrontabAction {
    ToggleJob { job_id: String, enabled: bool },
    SetInterval { job_id: String, interval: String },
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
}
