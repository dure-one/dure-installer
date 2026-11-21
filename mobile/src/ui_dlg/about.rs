pub use super::about_stt::*;
use crate::{dure_debug, dure_error, dure_info, dure_warn};
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{dialog, MaterialButton};

impl DlgAbout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.do_check_update = false;
        self.do_perform_update = false;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        update_checking: bool,
        update_available: bool,
        update_status: &str,
    ) {
        if !self.open {
            return;
        }

        self.do_check_update = false;
        self.do_perform_update = false;

        let version = env!("CARGO_PKG_VERSION");
        let description = tr!("about-description");
        let website_label = tr!("about-website");
        let credits_label = tr!("about-credits");

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog("about_dialog", &tr!("about"), &mut self.open)
            .min_width(350.0)
            .max_width(window_width - 120.0)
            .max_height(window_height - 120.0)
            .scrollable(true)
            .content(|ui, _open| {
                ui.heading(tr!("app-title"));
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("{}: {}", tr!("about-version"), version));

                    ui.add_space(8.0);

                    if update_checking {
                        ui.spinner();
                        ui.label(tr!("checking-update"));
                    } else if update_available {
                        ui.label(update_status);
                        if ui.button(tr!("update-now")).clicked() {
                            self.do_perform_update = true;
                        }
                    } else if !update_status.is_empty() {
                        ui.label(update_status);
                    } else if ui.button(tr!("check-update")).clicked() {
                        self.do_check_update = true;
                    }
                });

                ui.add_space(12.0);

                // Description
                ui.add(egui::Label::new(&description).wrap());

                ui.add_space(12.0);

                // Website
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("{}: ", website_label));
                    if ui.button("https://dure.pages.dev").clicked() {
                        if let Err(e) = webbrowser::open("https://dure.pages.dev") {
                            dure_error!("Failed to open website URL: {}", e);
                        }
                    }
                });

                ui.add_space(12.0);

                // Credits section
                use egui_material3::theme::get_global_color;
                ui.label(
                    egui::RichText::new(&credits_label)
                        .strong()
                        .color(get_global_color("onSurface")),
                );
                ui.add_space(4.0);

                ui.heading(tr!("about-reference-projects"));
                ui.add_space(4.0);
                ui.label("• bevy_game_template");
                ui.label("  Template for Bevy game projects");
                ui.label("  License: MIT/Apache-2.0");
                ui.add_space(2.0);
                ui.label("• chatGPTBox");
                ui.label("  ChatGPT browser extension");
                ui.label("  License: MIT");
                ui.add_space(2.0);
                ui.label("• android-activity");
                ui.label("  Android activity glue crate");
                ui.label("  License: MIT/Apache-2.0");

                ui.add_space(12.0);
                ui.heading(tr!("about-rust-libraries"));
                ui.add_space(4.0);
                ui.add(
                    egui::Label::new("• egui - Immediate mode GUI library (MIT/Apache-2.0)").wrap(),
                );
                ui.add(
                    egui::Label::new("• diesel - Type-safe ORM and query builder (MIT/Apache-2.0)")
                        .wrap(),
                );
                ui.add(
                    egui::Label::new(
                        "• tray-icon - Cross-platform system tray library (MIT/Apache-2.0)",
                    )
                    .wrap(),
                );
                ui.add(
                    egui::Label::new(
                        "• winit - Cross-platform window creation and management (Apache-2.0)",
                    )
                    .wrap(),
                );

                ui.add_space(12.0);
                ui.heading("Assets");
                ui.add_space(4.0);
                ui.label("• Solar Icons (CC Attribution)");
                ui.label("• Noto Emoji (Apache-2.0)");
                ui.label("• Twemoji (CC-BY 4.0)");

                ui.add_space(8.0);
            })
            .primary_action(tr!("ok"), || {})
            .show(ctx);
    }
}
