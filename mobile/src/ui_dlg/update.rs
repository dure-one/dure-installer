pub use super::update_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{dialog, MaterialButton};

impl DlgUpdate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.do_update = false;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.open {
            return;
        }

        self.do_update = false;

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog("update_dialog", &tr!("update-available-title"), &mut self.open)
            .min_width(350.0)
            .max_width(window_width - 40.0)
            .max_height(window_height - 40.0)
            .scrollable(true)
            .content(|ui, _open| {
                ui.heading(tr!("update-available-title"));
                ui.add_space(8.0);

                // Show version info
                ui.add(
                    egui::Label::new(format!(
                        "{} {} → {}",
                        tr!("update-available-message"),
                        self.current_version,
                        self.latest_version
                    ))
                    .wrap(),
                );

                ui.add_space(12.0);

                // Show release notes if available
                if !self.release_notes.is_empty() {
                    ui.label(tr!("release-notes"));
                    ui.add_space(4.0);

                    egui::ScrollArea::vertical()
                        .id_salt("update_release_notes_scroll")
                        .max_height(200.0)
                        .show(ui, |ui| {
                            ui.add(egui::Label::new(&self.release_notes).wrap());
                        });

                    ui.add_space(12.0);
                }

                // Platform-specific instructions
                #[cfg(target_os = "android")]
                {
                    ui.add(egui::Label::new(tr!("update-android-instruction")).wrap());
                }

                #[cfg(not(target_os = "android"))]
                {
                    ui.add(egui::Label::new(tr!("update-desktop-instruction")).wrap());
                }

                ui.add_space(8.0);
            })
            .action(tr!("later"), || {})
            .primary_action(tr!("update-now"), || {
                #[cfg(not(target_os = "android"))]
                {
                    self.do_update = true;
                }
            })
            .show(ctx);
    }
}
