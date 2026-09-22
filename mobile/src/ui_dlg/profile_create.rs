pub use super::profile_create_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::MaterialButton;

impl DlgProfileCreate {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.profile_name.clear();
        self.password.clear();
        self.password_confirm.clear();
        self.error_message.clear();
        self.confirmed = false;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn reset(&mut self) {
        self.profile_name.clear();
        self.password.clear();
        self.password_confirm.clear();
        self.error_message.clear();
        self.confirmed = false;
        self.open = false;
    }

    pub fn set_error(&mut self, error: String) {
        self.error_message = error;
    }

    /// Validate profile name using ProfileManager validation rules
    fn validate_profile_name(name: &str) -> Result<(), String> {
        // Check empty
        if name.is_empty() {
            return Err("Profile name cannot be empty".to_string());
        }

        // Check length
        if name.len() > 64 {
            return Err(format!("Profile name too long ({} chars, max 64)", name.len()));
        }

        // Check valid characters: a-z, A-Z, 0-9, -, _
        for ch in name.chars() {
            if !ch.is_ascii_alphanumeric() && ch != '-' && ch != '_' {
                return Err(format!(
                    "Invalid character '{}' in name (only a-z, A-Z, 0-9, -, _ allowed)",
                    ch
                ));
            }
        }

        Ok(())
    }

    /// Validate password input
    fn validate_password(password: &str, password_confirm: &str) -> Result<(), String> {
        // Check if password is empty
        if password.is_empty() {
            return Err("Password cannot be empty".to_string());
        }

        // Check if passwords match
        if password != password_confirm {
            return Err("Passwords do not match".to_string());
        }

        Ok(())
    }

    /// Process dialog action and return the result.
    /// Separated from UI rendering for testability.
    ///
    /// # Arguments
    /// * `action` - The button that was clicked (CreateDialogAction enum)
    ///
    /// # Returns
    /// * `Some(ProfileCreateResult)` if Submit action with valid name and matching passwords
    /// * `None` if Cancel action, no action, or Submit with validation errors
    pub fn process_action(&mut self, action: CreateDialogAction) -> Option<ProfileCreateResult> {
        match action {
            CreateDialogAction::Submit => {
                // Validate profile name
                if let Err(err) = Self::validate_profile_name(&self.profile_name) {
                    self.error_message = err;
                    return None;
                }

                // Validate password
                if let Err(err) = Self::validate_password(&self.password, &self.password_confirm) {
                    self.error_message = err;
                    return None;
                }

                // All validations passed
                self.confirmed = true;
                self.open = false;
                Some(ProfileCreateResult {
                    name: self.profile_name.clone(),
                    password: self.password.clone(),
                })
            }
            CreateDialogAction::Cancel => {
                self.close();
                None
            }
            CreateDialogAction::None => None,
        }
    }

    /// Display the create profile dialog and return profile info if confirmed, None if cancelled
    pub fn show(&mut self, ctx: &egui::Context) -> Option<ProfileCreateResult> {
        if !self.open {
            return None;
        }

        let mut dialog_action = CreateDialogAction::None;

        egui::Window::new(tr!("profile-create-title"))
            .id(egui::Id::new("profile_create_window"))
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .scroll([false, false])
            .min_width(400.0)
            .min_height(350.0)
            .resize(|r| {
                r.default_size([400.0, 350.0])
            })
            .show(ctx, |ui| {
                ui.heading(tr!("profile-create-heading"));
                ui.add_space(12.0);

                // Error message display
                if !self.error_message.is_empty() {
                    ui.colored_label(
                        egui::Color32::from_rgb(220, 20, 60),
                        &self.error_message,
                    );
                    ui.add_space(8.0);
                }

                // Profile name input
                ui.label(tr!("profile-name"));
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.profile_name)
                        .hint_text(tr!("profile-name-hint"))
                        .desired_width(ui.available_width() - 40.0),
                );

                #[cfg(target_os = "android")]
                {
                    if response.gained_focus() {
                        let _ = crate::android::inputmethod::show_soft_input();
                    }
                    if response.lost_focus() {
                        let _ = crate::android::inputmethod::hide_soft_input();
                    }
                }

                crate::ui_dlg::clipboard_popup::show_clipboard_popup(ui, &response, &mut self.profile_name);

                ui.add_space(8.0);

                // Password input
                ui.label(tr!("password"));
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.password)
                        .password(true)
                        .hint_text(tr!("password-hint"))
                        .desired_width(ui.available_width() - 40.0),
                );

                #[cfg(target_os = "android")]
                {
                    if response.gained_focus() {
                        let _ = crate::android::inputmethod::show_soft_input();
                    }
                    if response.lost_focus() {
                        let _ = crate::android::inputmethod::hide_soft_input();
                    }
                }

                crate::ui_dlg::clipboard_popup::show_clipboard_popup(ui, &response, &mut self.password);

                ui.add_space(8.0);

                // Password confirmation input
                ui.label(tr!("password-confirm"));
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.password_confirm)
                        .password(true)
                        .hint_text(tr!("password-confirm-hint"))
                        .desired_width(ui.available_width() - 40.0),
                );

                #[cfg(target_os = "android")]
                {
                    if response.gained_focus() {
                        let _ = crate::android::inputmethod::show_soft_input();
                    }
                    if response.lost_focus() {
                        let _ = crate::android::inputmethod::hide_soft_input();
                    }
                }

                crate::ui_dlg::clipboard_popup::show_clipboard_popup(ui, &response, &mut self.password_confirm);

                ui.add_space(12.0);

                // Action buttons
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(MaterialButton::filled(tr!("create"))).clicked() {
                            dialog_action = CreateDialogAction::Submit;
                        }
                        if ui.add(MaterialButton::outlined(tr!("cancel"))).clicked() {
                            dialog_action = CreateDialogAction::Cancel;
                        }
                    });
                });
            });

        self.process_action(dialog_action)
    }
}
