pub use super::remove_docker_confirm_stt::*;
use eframe::egui;
use egui_material3::dialog;
use std::cell::Cell;

impl RemoveDockerConfirmDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, host: String) {
        self.host = host;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.host.clear();
    }

    /// Show remove docker confirm dialog, returns action result
    pub fn show(&mut self, ctx: &egui::Context) -> RemoveDockerConfirmAction {
        if !self.open {
            return RemoveDockerConfirmAction::None;
        }

        let confirmed = Cell::new(false);
        let cancelled = Cell::new(false);

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog(
            "remove_docker_confirm",
            "Confirm Remove Docker",
            &mut self.open,
        )
        .min_width(400.0)
        .max_width(window_width - 120.0)
        .max_height(window_height - 120.0)
        .scrollable(true)
        .content(|ui, _open| {
            ui.label(format!("Remove Docker from host '{}'?", self.host));
            ui.add_space(8.0);
            ui.label("This will remove Docker and all containers. This action cannot be undone.");
        })
        .action("Cancel", || {
            cancelled.set(true);
        })
        .primary_action("Remove", || {
            confirmed.set(true);
        })
        .show(ctx);

        if confirmed.get() {
            let result = RemoveDockerConfirmAction::Remove {
                host: self.host.clone(),
            };
            self.close();
            result
        } else if cancelled.get() || !self.open {
            self.close();
            RemoveDockerConfirmAction::Cancel
        } else {
            RemoveDockerConfirmAction::None
        }
    }
}
