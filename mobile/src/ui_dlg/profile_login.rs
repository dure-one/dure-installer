pub use super::profile_login_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{dialog, MaterialButton};
use std::cell::Cell;

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

        let dialog_action = Cell::new(DialogAction::None);

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog(
            "profile_login_dialog",
            &tr!("profile-login-title"),
            &mut self.open,
        )
        .min_width(350.0)
        .max_width(window_width - 120.0)
        .max_height(window_height - 120.0)
        .scrollable(true)
        .content(|ui, _open| {
            ui.heading(tr!("profile-login-heading"));
            ui.add_space(12.0);

            // Error message display
            if !self.error_message.is_empty() {
                ui.colored_label(
                    egui_material3::theme::get_global_color("error"),
                    &self.error_message,
                );
                ui.add_space(8.0);
            }

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

            ui.add_space(12.0);
        })
        .action(tr!("cancel"), || {
            dialog_action.set(DialogAction::Cancel);
        })
        .primary_action(tr!("ok"), || {
            dialog_action.set(DialogAction::Submit);
        })
        .show(ctx);

        self.process_action(dialog_action.get())
    }
}
