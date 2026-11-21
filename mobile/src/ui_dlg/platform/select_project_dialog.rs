pub use super::select_project_dialog_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::dialog;
use std::cell::Cell;

impl SelectProjectDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, platform_name: String, project_list: Vec<(String, String)>) {
        self.platform_name = platform_name;
        self.project_list = project_list;
        self.selected = None;
        self.loading = false;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.platform_name.clear();
        self.project_list.clear();
        self.selected = None;
        self.loading = false;
    }

    /// Show select project dialog, returns action result
    pub fn show(&mut self, ctx: &egui::Context) -> SelectProjectAction {
        if !self.open {
            return SelectProjectAction::None;
        }

        let select_clicked = Cell::new(false);
        let cancel_clicked = Cell::new(false);

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog(
            "select_project_dialog",
            "Select GCP Project",
            &mut self.open,
        )
        .min_width(350.0)
        .max_width(window_width - 120.0)
        .max_height(window_height - 120.0)
        .scrollable(true)
        .content(|ui, _open| {
            ui.heading(tr!("platform-select-project-heading"));
            ui.add_space(8.0);

            if self.project_list.is_empty() {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 193, 7),
                    tr!("platform-select-project-none"),
                ); // Warning amber
            } else {
                ui.label(format!(
                    "Found {} projects:",
                    self.project_list.len()
                ));
                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for (idx, (project_id, project_name)) in
                            self.project_list.iter().enumerate()
                        {
                            let is_selected = self.selected == Some(idx);
                            if ui
                                .selectable_label(
                                    is_selected,
                                    format!("{} ({})", project_name, project_id),
                                )
                                .clicked()
                            {
                                self.selected = Some(idx);
                            }
                        }
                    });
            }

            ui.add_space(12.0);
        })
        .action("Cancel", || {
            cancel_clicked.set(true);
        })
        .primary_action(&tr!("platform-select-project-button"), || {
            select_clicked.set(true);
        })
        .show(ctx);

        if select_clicked.get() && self.selected.is_some() {
            if let Some(idx) = self.selected {
                if idx < self.project_list.len() {
                    let (project_id, _) = &self.project_list[idx];
                    let result = SelectProjectAction::Select {
                        platform_name: self.platform_name.clone(),
                        project_id: project_id.clone(),
                    };
                    self.close();
                    return result;
                }
            }
        }

        if cancel_clicked.get() || !self.open {
            self.close();
            SelectProjectAction::Cancel
        } else {
            SelectProjectAction::None
        }
    }
}
