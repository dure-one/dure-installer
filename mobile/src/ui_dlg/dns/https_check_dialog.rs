pub use super::https_check_dialog_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::dialog;

impl HttpsCheckDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, result: Option<HttpsCheckResult>) {
        self.open = true;
        self.result = result;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.result = None;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> HttpsCheckAction {
        if !self.open {
            return HttpsCheckAction::None;
        }

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog(
            "https_connection_check_dialog",
            tr!("dns-dialog-https-title"),
            &mut self.open,
        )
        .min_width(350.0)
        .max_width(window_width - 120.0)
        .max_height(window_height - 120.0)
        .scrollable(true)
        .content(|ui, _open| {
            if let Some(result) = &self.result {
                ui.heading(if result.resolves_to_vm {
                    tr!("dns-dialog-https-ok")
                } else {
                    tr!("dns-dialog-https-mismatch")
                });
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label(tr!("dns-dialog-https-resolved-ip"));
                    if let Some(ip) = &result.resolved_ip {
                        ui.label(ip);
                    } else {
                        ui.weak(tr!("dns-dialog-https-na"));
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(tr!("dns-dialog-https-expected-ip"));
                    if let Some(ip) = &result.expected_ip {
                        ui.label(ip);
                    } else {
                        ui.weak(tr!("dns-dialog-https-na"));
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(tr!("dns-dialog-https-matches-vm"));
                    if result.resolves_to_vm {
                        ui.label(tr!("dns-dialog-https-yes"));
                    } else {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 193, 7),
                            tr!("dns-dialog-https-no"),
                        );
                    }
                });

                if let Some(err_msg) = &result.error {
                    use egui_material3::theme::get_global_color;
                    ui.add_space(10.0);
                    ui.colored_label(
                        get_global_color("error"),
                        format!("{} {}", tr!("dns-dialog-https-error-prefix"), err_msg),
                    );
                }
            } else {
                ui.spinner();
                ui.label(tr!("dns-dialog-https-checking"));
            }
        })
        .action(tr!("close"), || {})
        .show(ctx);

        if !self.open {
            self.close();
            HttpsCheckAction::Close
        } else {
            HttpsCheckAction::None
        }
    }
}
