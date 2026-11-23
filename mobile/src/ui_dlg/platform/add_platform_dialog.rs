pub use super::add_platform_dialog_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{dialog, MaterialButton};
use std::cell::Cell;

impl AddPlatformDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, platform_type: String) {
        self.open = true;
        self.platform_type = platform_type;
        self.reauth_mode = false;
        self.existing_project_id.clear();
    }

    pub fn open_reauth(&mut self, existing_project_id: String) {
        self.open = true;
        self.reauth_mode = true;
        self.existing_project_id = existing_project_id;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.oauth_url = None;
        self.oauth_result = None;
        self.oauth_promise = None;
        self.connected_email = None;
        self.project_list.clear();
        self.selected_project = None;
        self.create_new = false;
        self.new_project_id.clear();
        self.reauth_mode = false;
        self.existing_project_id.clear();
    }

    pub fn show(&mut self, ctx: &egui::Context) -> AddPlatformAction {
        if !self.open {
            return AddPlatformAction::None;
        }

        let add_clicked = Cell::new(false);
        let cancel_clicked = Cell::new(false);
        let connect_clicked = Cell::new(false);

        let title = if self.reauth_mode {
            tr!("reauth-platform-title")
        } else {
            tr!("add-platform-title")
        };

        let heading = if self.reauth_mode {
            tr!("reauth-platform-heading")
        } else {
            tr!("add-platform-heading")
        };

        let button_text = if self.reauth_mode {
            tr!("reauth")
        } else {
            tr!("add")
        };

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog("add_platform_dialog", &title, &mut self.open)
            .min_width(350.0)
            .max_width(window_width - 120.0)
            .max_height(window_height - 120.0)
            .scrollable(true)
            .content(|ui, _open| {
                ui.heading(&heading);
                ui.add_space(12.0);

                // Show existing project in re-auth mode
                if self.reauth_mode {
                    ui.label(format!("Project: {}", self.existing_project_id));
                    ui.add_space(8.0);
                } else {
                    ui.label(tr!("platform-add-configure"));
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label(tr!("platform-add-type"));
                        egui::ComboBox::from_id_salt("platform_type_combo")
                            .selected_text(&self.platform_type)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.platform_type,
                                    "gcp".to_string(),
                                    "GCP (Google Cloud Platform)",
                                );
                            });
                    });

                    ui.add_space(12.0);
                }

                // OAuth section
                if self.platform_type == "gcp" {
                    ui.separator();
                    ui.add_space(8.0);

                    if let Some(email) = &self.connected_email {
                        ui.colored_label(
                            egui::Color32::from_rgb(76, 175, 80),
                            format!("Connected as: {}", email),
                        );

                        ui.add_space(8.0);

                        // Project selection (skip in reauth mode)
                        if !self.reauth_mode {
                            ui.label(tr!("platform-add-project"));
                            ui.add_space(4.0);

                            // Radio buttons: select vs create
                            ui.horizontal(|ui| {
                                ui.radio_value(&mut self.create_new, false, "Select Existing");
                                ui.radio_value(&mut self.create_new, true, "Create New");
                            });
                            ui.add_space(8.0);

                            if self.create_new {
                                // Create new project
                                ui.label(tr!("platform-add-project-id"));
                                ui.add_space(4.0);
                                ui.text_edit_singleline(&mut self.new_project_id);
                                ui.add_space(4.0);
                                ui.colored_label(
                                    egui_material3::theme::get_global_color("onSurfaceVariant"),
                                    "Project ID must be 6-30 characters, lowercase letters, digits, hyphens",
                                );
                            } else {
                                // Select existing project
                                ui.label(format!(
                                    "Select Project ({} available):",
                                    self.project_list.len()
                                ));
                                ui.add_space(4.0);

                                egui::ScrollArea::vertical()
                                    .max_height(200.0)
                                    .show(ui, |ui| {
                                        for (idx, (project_id, project_name)) in
                                            self.project_list.iter().enumerate()
                                        {
                                            let is_selected = self.selected_project == Some(idx);
                                            if ui
                                                .selectable_label(
                                                    is_selected,
                                                    format!("{} ({})", project_name, project_id),
                                                )
                                                .clicked()
                                            {
                                                self.selected_project = Some(idx);
                                            }
                                        }
                                    });
                            }
                        }
                    } else if self.oauth_promise.is_some() {
                        ui.spinner();
                        ui.label(tr!("platform-add-oauth-waiting"));
                        ui.label(tr!("platform-add-oauth-complete"));
                    } else {
                        ui.label("OAuth connection required");
                        ui.add_space(4.0);

                        if ui
                            .add(MaterialButton::outlined(tr!("platform-add-oauth-connect")))
                            .clicked()
                        {
                            connect_clicked.set(true);
                        }

                        // Show OAuth URL if available
                        if let Some(ref oauth_url) = self.oauth_url {
                            ui.add_space(8.0);
                            ui.label(tr!("platform-add-oauth-url"));
                            ui.add_space(4.0);
                            egui::ScrollArea::vertical()
                                .max_height(60.0)
                                .show(ui, |ui| {
                                    ui.monospace(oauth_url);
                                });
                        }
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                }
            })
            .action(&tr!("cancel"), || {
                cancel_clicked.set(true);
            })
            .primary_action(&button_text, || {
                add_clicked.set(true);
            })
            .show(ctx);

        // Handle actions
        if cancel_clicked.get() {
            self.close();
            AddPlatformAction::Cancel
        } else if connect_clicked.get() {
            AddPlatformAction::Connect
        } else if add_clicked.get() {
            let can_add = if self.reauth_mode {
                self.connected_email.is_some()
            } else {
                self.platform_type != "gcp"
                    || (self.connected_email.is_some()
                        && (self.selected_project.is_some() || !self.new_project_id.is_empty()))
            };

            if can_add {
                if let Some(oauth_result) = self.oauth_result.clone() {
                    let project_id = if self.reauth_mode {
                        self.existing_project_id.clone()
                    } else if self.create_new {
                        self.new_project_id.clone()
                    } else if let Some(idx) = self.selected_project {
                        self.project_list
                            .get(idx)
                            .map(|(id, _)| id.clone())
                            .unwrap_or_default()
                    } else {
                        String::new()
                    };

                    self.close();
                    AddPlatformAction::Add {
                        platform_type: self.platform_type.clone(),
                        project_id,
                        oauth_result,
                    }
                } else {
                    AddPlatformAction::None
                }
            } else {
                AddPlatformAction::None
            }
        } else if !self.open {
            self.close();
            AddPlatformAction::Cancel
        } else {
            AddPlatformAction::None
        }
    }
}
