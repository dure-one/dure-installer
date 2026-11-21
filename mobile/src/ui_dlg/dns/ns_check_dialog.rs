pub use super::ns_check_dialog_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::dialog;

impl NsCheckDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, results: Vec<String>) {
        self.open = true;
        self.results = results;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.results.clear();
    }

    pub fn show(&mut self, ctx: &egui::Context) -> NsCheckAction {
        if !self.open {
            return NsCheckAction::None;
        }

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog(
            "ns_server_check_dialog",
            tr!("dns-dialog-ns-title"),
            &mut self.open,
        )
        .min_width(350.0)
        .max_width(window_width - 120.0)
        .max_height(window_height - 120.0)
        .scrollable(true)
        .content(|ui, _open| {
            if self.results.is_empty() {
                ui.spinner();
                ui.label(tr!("dns-dialog-ns-querying"));
            } else {
                for result in &self.results {
                    ui.label(result);
                }
            }
        })
        .action(tr!("close"), || {})
        .show(ctx);

        if !self.open {
            self.close();
            NsCheckAction::Close
        } else {
            NsCheckAction::None
        }
    }
}
