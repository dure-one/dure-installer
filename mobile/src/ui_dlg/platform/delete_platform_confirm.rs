pub use super::delete_platform_confirm_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::dialog;
use std::cell::Cell;

impl DeletePlatformConfirmDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, platform_name: String, vm_count: usize) {
        self.platform_name = platform_name;
        self.vm_count = vm_count;
        self.delete_vms = false;
        self.delete_project = false;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.platform_name.clear();
        self.vm_count = 0;
        self.delete_vms = false;
        self.delete_project = false;
    }

    /// Show delete platform confirm dialog, returns action result
    pub fn show(&mut self, ctx: &egui::Context) -> DeletePlatformConfirmAction {
        if !self.open {
            return DeletePlatformConfirmAction::None;
        }

        let delete_clicked = Cell::new(false);
        let cancel_clicked = Cell::new(false);

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog(
            "delete_platform_dialog",
            &tr!("delete-platform-title"),
            &mut self.open,
        )
        .min_width(350.0)
        .max_width(window_width - 120.0)
        .max_height(window_height - 120.0)
        .scrollable(true)
        .content(|ui, _open| {
            ui.heading(tr!("delete-platform-heading"));
            ui.add_space(12.0);

            ui.label(format!(
                "Are you sure you want to delete platform '{}'?",
                self.platform_name
            ));
            ui.add_space(8.0);

            if self.vm_count > 0 {
                ui.colored_label(
                    egui_material3::theme::get_global_color("error"),
                    format!(
                        "This will also remove {} VM(s) from config!",
                        self.vm_count
                    ),
                );
                ui.add_space(4.0);
                ui.colored_label(
                    egui::Color32::from_rgb(255, 193, 7), // Warning amber
                    "Note: VMs will be removed from config but NOT deleted from GCP.",
                );
            } else {
                ui.colored_label(
                    egui_material3::theme::get_global_color("onSurfaceVariant"),
                    tr!("platform-drawer-no-vms"),
                );
            }

            ui.add_space(8.0);
            ui.colored_label(
                egui_material3::theme::get_global_color("error"),
                "This action cannot be undone!",
            );

            ui.add_space(12.0);

            // Delete options
            ui.checkbox(&mut self.delete_vms, "Delete VMs from GCP");
            ui.checkbox(
                &mut self.delete_project,
                "Delete GCP project",
            );

            ui.add_space(12.0);
        })
        .action(&tr!("cancel"), || {
            cancel_clicked.set(true);
        })
        .primary_action(&tr!("delete"), || {
            delete_clicked.set(true);
        })
        .show(ctx);

        if delete_clicked.get() {
            let result = DeletePlatformConfirmAction::Delete {
                platform_name: self.platform_name.clone(),
                delete_vms: self.delete_vms,
                delete_project: self.delete_project,
            };
            self.close();
            result
        } else if cancel_clicked.get() || !self.open {
            self.close();
            DeletePlatformConfirmAction::Cancel
        } else {
            DeletePlatformConfirmAction::None
        }
    }
}
