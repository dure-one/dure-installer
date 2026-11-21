pub use super::remove_dure_dialog_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::dialog;
use std::cell::Cell;

impl RemoveDureDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, host: String) {
        self.host = host;
        self.delete_data = false;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.host.clear();
        self.delete_data = false;
    }

    /// Show remove dure dialog, returns action result
    pub fn show(&mut self, ctx: &egui::Context) -> RemoveDureAction {
        if !self.open {
            return RemoveDureAction::None;
        }

        let remove_requested = Cell::new(false);
        let cancel_requested = Cell::new(false);

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog(
            "remove_dure_dialog",
            "Remove Dure",
            &mut self.open,
        )
        .min_width(350.0)
        .max_width(window_width - 120.0)
        .max_height(window_height - 120.0)
        .scrollable(true)
        .content(|ui, _open| {
            ui.label(format!("Remove Dure from host: {}", self.host));
            ui.add_space(12.0);

            ui.checkbox(
                &mut self.delete_data,
                "Delete data directories (/srv/dure-mycart and /srv/data)",
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Warning: Deleting data cannot be undone!")
                    .color(egui::Color32::from_rgb(211, 47, 47)),
            );
        })
        .action(tr!("button-cancel"), || {
            cancel_requested.set(true);
        })
        .primary_action("Remove", || {
            remove_requested.set(true);
        })
        .show(ctx);

        if remove_requested.get() {
            let result = RemoveDureAction::Remove {
                host: self.host.clone(),
                delete_data: self.delete_data,
            };
            self.close();
            result
        } else if cancel_requested.get() || !self.open {
            self.close();
            RemoveDureAction::Cancel
        } else {
            RemoveDureAction::None
        }
    }
}
