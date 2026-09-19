pub use super::profile_login_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::MaterialButton;

impl DlgProfileLogin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.password.clear();
        self.error_message.clear();
        self.confirmed = false;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn reset(&mut self) {
        self.password.clear();
        self.error_message.clear();
        self.confirmed = false;
        self.open = false;
    }

    pub fn set_error(&mut self, error: String) {
        self.error_message = error;
    }

    /// Display the login dialog and return password if confirmed, None if cancelled
    pub fn show(&mut self, ctx: &egui::Context) -> Option<String> {
        if !self.open {
            return None;
        }

        let mut close_clicked = false;
        let mut login_clicked = false;

        egui::Window::new("Login Profile")
            .id(egui::Id::new("profile_login_window"))
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .scroll([false, false])
            .min_width(400.0)
            .min_height(250.0)
            .resize(|r| {
                r.default_size([400.0, 250.0])
            })
            .show(ctx, |ui| {
                ui.heading("Enter Profile Password");
                ui.add_space(12.0);

                // Error message display
                if !self.error_message.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(220, 20, 60),
                        &self.error_message,
                    );
                    ui.add_space(8.0);
                }

                // Password input
                ui.label("Password:");
                ui.text_edit_singleline(&mut self.password);

                ui.add_space(12.0);

                // Action buttons
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(MaterialButton::filled("OK")).clicked() {
                            login_clicked = true;
                        }
                        if ui.add(MaterialButton::outlined("Cancel")).clicked() {
                            close_clicked = true;
                        }
                    });
                });
            });

        if login_clicked {
            self.confirmed = true;
            self.open = false;
            let password = self.password.clone();
            return Some(password);
        }

        if close_clicked {
            self.close();
        }

        None
    }
}
