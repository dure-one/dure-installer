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

    /// Process dialog action and return the result.
    /// Separated from UI rendering for testability.
    ///
    /// # Arguments
    /// * `action` - The button that was clicked (DialogAction enum)
    ///
    /// # Returns
    /// * `Some(password)` if Submit action with non-empty password
    /// * `None` if Cancel action, no action, or Submit with empty password
    pub fn process_action(&mut self, action: DialogAction) -> Option<String> {
        match action {
            DialogAction::Submit => {
                if !self.password.is_empty() {
                    self.confirmed = true;
                    self.open = false;
                    return Some(self.password.clone());
                }
                // Empty password: keep dialog open, no action
                None
            }
            DialogAction::Cancel => {
                self.close();
                None
            }
            DialogAction::None => None,
        }
    }

    /// Display the login dialog and return password if confirmed, None if cancelled
    pub fn show(&mut self, ctx: &egui::Context) -> Option<String> {
        if !self.open {
            return None;
        }

        let mut dialog_action = DialogAction::None;

        egui::Window::new(tr!("profile-login-title"))
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
                ui.heading(tr!("profile-login-heading"));
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
                ui.label(tr!("password"));
                ui.add(
                    egui::TextEdit::singleline(&mut self.password)
                        .password(true)
                        .hint_text(tr!("password-hint")),
                );

                ui.add_space(12.0);

                // Action buttons
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(MaterialButton::filled(tr!("ok"))).clicked() {
                            dialog_action = DialogAction::Submit;
                        }
                        if ui.add(MaterialButton::outlined(tr!("cancel"))).clicked() {
                            dialog_action = DialogAction::Cancel;
                        }
                    });
                });
            });

        self.process_action(dialog_action)
    }
}
