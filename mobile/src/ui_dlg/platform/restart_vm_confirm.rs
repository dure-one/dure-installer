pub use super::restart_vm_confirm_stt::*;
use eframe::egui;
use egui_material3::dialog;
use std::cell::Cell;

impl RestartVmConfirmDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, vm_name: String, platform: String) {
        self.vm_name = vm_name;
        self.platform = platform;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.vm_name.clear();
        self.platform.clear();
    }

    /// Show restart VM confirm dialog, returns action result
    pub fn show(&mut self, ctx: &egui::Context) -> RestartVmConfirmAction {
        if !self.open {
            return RestartVmConfirmAction::None;
        }

        let confirmed = Cell::new(false);
        let cancelled = Cell::new(false);

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog("restart_vm_confirm", "Confirm Restart", &mut self.open)
            .min_width(400.0)
            .max_width(window_width - 120.0)
            .max_height(window_height - 120.0)
            .scrollable(true)
            .content(|ui, _open| {
                ui.label(format!("Restart VM '{}'?", self.vm_name));
                ui.add_space(8.0);
                ui.label("The VM will be temporarily unavailable during restart.");
            })
            .action("Cancel", || {
                cancelled.set(true);
            })
            .primary_action("Restart", || {
                confirmed.set(true);
            })
            .show(ctx);

        if confirmed.get() {
            let result = RestartVmConfirmAction::Restart {
                vm_name: self.vm_name.clone(),
                platform: self.platform.clone(),
            };
            self.close();
            result
        } else if cancelled.get() || !self.open {
            self.close();
            RestartVmConfirmAction::Cancel
        } else {
            RestartVmConfirmAction::None
        }
    }
}
