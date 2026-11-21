pub use super::error_dialog_stt::*;
use eframe::egui;
use egui_i18n::tr;
use std::cell::Cell;

impl SshErrorDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, error: String) {
        self.error_message = error;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.error_message.clear();
    }

    /// Show error dialog, returns true if OK was clicked
    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        if !self.open {
            return false;
        }

        let ok_clicked = Cell::new(false);
        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        egui_material3::dialog("ssh_error_dialog", "Error", &mut self.open)
            .min_width(350.0)
            .max_width(window_width - 120.0)
            .max_height(window_height - 120.0)
            .scrollable(true)
            .content(|ui, _open| {
                ui.add_space(8.0);
                ui.colored_label(
                    egui_material3::theme::get_global_color("error"),
                    &self.error_message,
                );
                ui.add_space(12.0);
            })
            .primary_action(&tr!("ok"), || {
                ok_clicked.set(true);
            })
            .show(ctx);

        if ok_clicked.get() || !self.open {
            self.close();
            true
        } else {
            false
        }
    }
}
