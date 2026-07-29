//! Site tab - Site-to-site communication management

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use eframe::egui;
use egui_material3::MaterialButton;
use egui_material3::spreadsheet::{MaterialSpreadsheet, text_column};

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use crate::calc::audit;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use crate::calc::site;

/// Site tab state
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SiteTab {
    selected_row: Option<usize>,
    /// Cached site rows (domain, public_key, status, last_seen)
    #[cfg_attr(feature = "serde", serde(skip))]
    rows: Vec<[String; 4]>,
    #[cfg_attr(feature = "serde", serde(skip))]
    loaded: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    load_error: Option<String>,

    // Spreadsheet
    #[cfg_attr(feature = "serde", serde(skip))]
    spreadsheet: Option<MaterialSpreadsheet>,
    row_selection_enabled: bool,

    // Add dialog state
    #[cfg_attr(feature = "serde", serde(skip))]
    show_add_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_domain: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_public_key: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_status_message: String,
}

impl Default for SiteTab {
    fn default() -> Self {
        let spreadsheet = {
            let columns = vec![
                text_column("Domain", 250.0),
                text_column("Public Key", 300.0),
                text_column("Status", 120.0),
                text_column("Last Seen", 180.0),
            ];

            // Create spreadsheet with theme-aware settings
            MaterialSpreadsheet::new("site_spreadsheet", columns)
                .ok()
                .map(|mut s| {
                    // Enable striped rows for Material Design theme colors
                    s.set_striped(true);
                    // Enable row selection
                    s.set_row_selection_enabled(true);
                    // Enable selection for better visual feedback
                    s.set_allow_selection(true);
                    s
                })
        };

        Self {
            selected_row: None,
            rows: Vec::new(),
            loaded: false,
            load_error: None,
            spreadsheet,
            row_selection_enabled: true,
            show_add_dialog: false,
            add_domain: String::new(),
            add_public_key: String::new(),
            add_status_message: String::new(),
        }
    }
}

impl SiteTab {
    /// Render the site tab UI
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Site Management");
        ui.add_space(4.0);
        ui.label("Manage sites for site-to-site communication via WebSocket.");
        ui.add_space(8.0);

        // Get selected row for action buttons
        let selected_row_idx = self.spreadsheet.as_ref().and_then(|s| s.get_selected_row());
        let has_selection = selected_row_idx.is_some();

        // Action buttons
        ui.horizontal(|ui| {
            if ui.add(MaterialButton::filled("Add Site")).clicked() {
                self.show_add_dialog = true;
                self.add_domain.clear();
                self.add_public_key.clear();
                self.add_status_message.clear();
            }

            // Delete button - enabled only when a row is selected
            let delete_button = MaterialButton::outlined("Delete Selected");
            let delete_button = if has_selection {
                delete_button
            } else {
                delete_button.enabled(false)
            };

            if ui.add(delete_button).clicked() {
                if let Some(idx) = selected_row_idx {
                    if idx < self.rows.len() {
                        let domain = self.rows[idx][0].clone();
                        self.execute_delete_site(domain);
                    }
                }
            }

            if ui.add(MaterialButton::outlined("Refresh")).clicked() {
                self.loaded = false;
                self.load_error = None;
            }

            // Show selected site info
            if let Some(idx) = selected_row_idx {
                if idx < self.rows.len() {
                    ui.label(format!("│ Selected: {}", self.rows[idx][0]));
                }
            }
        });
        ui.add_space(8.0);

        // Lazy-load from database on first render or after refresh
        if !self.loaded {
            self.load_rows();
            self.loaded = true;
        }

        if let Some(err) = &self.load_error {
            ui.colored_label(egui::Color32::RED, format!("⚠ {err}"));
            ui.add_space(4.0);
        }

        // Sites spreadsheet - fill remaining space
        if let Some(spreadsheet) = &mut self.spreadsheet {
            let available_height = ui.available_height();

            ui.group(|ui| {
                // Set the group to fill available space
                ui.set_min_height(available_height - 20.0); // Leave some padding
                ui.set_width(ui.available_width());

                egui::ScrollArea::vertical()
                    .max_height(available_height - 20.0)
                    .show(ui, |ui| {
                        spreadsheet.show(ui);
                    });
            });
        }

        // Add site dialog
        if self.show_add_dialog {
            self.render_add_dialog(ui.ctx());
        }
    }

    fn load_rows(&mut self) {
        self.rows.clear();
        self.load_error = None;

        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            match site::list_sites() {
                Ok(sites) => {
                    let mut data_rows = Vec::new();

                    for site_info in &sites {
                        let last_seen = match site_info.last_seen {
                            Some(ts) => chrono::DateTime::from_timestamp(ts as i64, 0)
                                .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                                .unwrap_or_else(|| "Unknown".to_string()),
                            None => "Never".to_string(),
                        };

                        // Truncate public key for display
                        let display_key = if site_info.public_key.len() > 40 {
                            format!("{}...", &site_info.public_key[..40])
                        } else {
                            site_info.public_key.clone()
                        };

                        self.rows.push([
                            site_info.domain.clone(),
                            display_key.clone(),
                            site_info.status.clone(),
                            last_seen.clone(),
                        ]);

                        data_rows.push(vec![
                            site_info.domain.clone(),
                            display_key,
                            site_info.status.clone(),
                            last_seen,
                        ]);
                    }

                    // Clear and update spreadsheet with fresh data
                    if let Some(spreadsheet) = &mut self.spreadsheet {
                        let columns = vec![
                            text_column("Domain", 250.0),
                            text_column("Public Key", 300.0),
                            text_column("Status", 120.0),
                            text_column("Last Seen", 180.0),
                        ];

                        match MaterialSpreadsheet::new("site_spreadsheet", columns) {
                            Ok(mut new_spreadsheet) => {
                                new_spreadsheet.set_striped(true);
                                new_spreadsheet.set_row_selection_enabled(true);
                                new_spreadsheet.set_allow_selection(true);
                                new_spreadsheet.init_with_data(data_rows);
                                *spreadsheet = new_spreadsheet;
                            }
                            Err(e) => {
                                self.load_error =
                                    Some(format!("Failed to create spreadsheet: {e}"));
                            }
                        }
                    }
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load sites: {e}"));
                }
            }
        }

        #[cfg(any(target_os = "android", target_arch = "wasm32"))]
        {
            self.load_error = Some("Site management not available on this platform".to_string());
        }
    }

    fn render_add_dialog(&mut self, ctx: &egui::Context) {
        let mut open = true;

        egui::Window::new("Add Site")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Configure a new site for site-to-site communication:");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Domain:");
                    ui.text_edit_singleline(&mut self.add_domain)
                        .on_hover_text("Site domain (e.g., example.com)");
                });

                ui.horizontal(|ui| {
                    ui.label("Public Key:");
                    ui.text_edit_singleline(&mut self.add_public_key)
                        .on_hover_text("Site's public key for authentication");
                });

                ui.add_space(8.0);
                ui.label(
                    egui::RichText::new(
                        "Note: Ensure the public key is published in DNS TXT record",
                    )
                    .color(ui.visuals().weak_text_color()),
                );

                if !self.add_status_message.is_empty() {
                    ui.add_space(8.0);
                    ui.colored_label(egui::Color32::RED, &self.add_status_message);
                }

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.add(MaterialButton::filled("Add")).clicked() {
                        self.execute_add_site();
                    }

                    if ui.add(MaterialButton::text("Cancel")).clicked() {
                        self.show_add_dialog = false;
                        self.add_status_message.clear();
                    }
                });
            });

        if !open {
            self.show_add_dialog = false;
            self.add_status_message.clear();
        }
    }

    fn execute_add_site(&mut self) {
        self.add_status_message.clear();

        // Validation
        if self.add_domain.trim().is_empty() {
            self.add_status_message = "Domain is required".to_string();
            return;
        }

        if self.add_public_key.trim().is_empty() {
            self.add_status_message = "Public key is required".to_string();
            return;
        }

        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            match site::add_site(
                self.add_domain.trim().to_string(),
                self.add_public_key.trim().to_string(),
            ) {
                Ok(_) => {
                    // Record audit event
                    let _ =
                        audit::push_gui("system", "desktop", "site add", self.add_domain.trim());

                    self.show_add_dialog = false;
                    self.loaded = false; // Trigger reload
                }
                Err(e) => {
                    self.add_status_message = format!("Failed to add site: {e}");
                }
            }
        }

        #[cfg(any(target_os = "android", target_arch = "wasm32"))]
        {
            self.add_status_message = "Not available on this platform".to_string();
        }
    }

    fn execute_delete_site(&mut self, domain: String) {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            match site::delete_site(&domain) {
                Ok(_) => {
                    // Record audit event
                    let _ = audit::push_gui("system", "desktop", "site del", &domain);

                    self.loaded = false; // Trigger reload
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to delete site: {e}"));
                }
            }
        }

        #[cfg(any(target_os = "android", target_arch = "wasm32"))]
        {
            self.load_error = Some("Not available on this platform".to_string());
        }
    }
}
