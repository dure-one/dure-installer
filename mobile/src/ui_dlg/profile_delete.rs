pub use super::profile_delete_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::MaterialButton;

impl DlgProfileDelete {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, profile_name: String) {
        self.profile_name = profile_name;
        self.confirmed = false;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
    }

    pub fn reset(&mut self) {
        self.profile_name.clear();
        self.confirmed = false;
        self.open = false;
    }

    /// Process dialog action and return the result.
    /// Separated from UI rendering for testability.
    ///
    /// # Arguments
    /// * `action` - The button that was clicked (DeleteDialogAction enum)
    ///
    /// # Returns
    /// * `Some(true)` if Submit action (user confirmed deletion)
    /// * `Some(false)` if Cancel action (user cancelled)
    /// * `None` if no action
    pub fn process_action(&mut self, action: DeleteDialogAction) -> Option<bool> {
        match action {
            DeleteDialogAction::Submit => {
                self.confirmed = true;
                self.open = false;
                Some(true)
            }
            DeleteDialogAction::Cancel => {
                self.close();
                Some(false)
            }
            DeleteDialogAction::None => None,
        }
    }

    /// Display the delete confirmation dialog and return user choice
    /// Returns Some(true) if user confirmed deletion, Some(false) if cancelled
    pub fn show(&mut self, ctx: &egui::Context) -> Option<bool> {
        if !self.open {
            return None;
        }

        let mut dialog_action = DeleteDialogAction::None;

        egui::Window::new(tr!("profile-delete-title"))
            .id(egui::Id::new("profile_delete_window"))
            .title_bar(false)
            .resizable(false)
            .collapsible(false)
            .scroll([false, false])
            .min_width(400.0)
            .min_height(200.0)
            .resize(|r| {
                r.default_size([400.0, 200.0])
            })
            .show(ctx, |ui| {
                ui.heading(tr!("profile-delete-heading"));
                ui.add_space(12.0);

                // Deletion confirmation message
                ui.label(tr!("profile-delete-message", { name: self.profile_name.clone() }));
                ui.add_space(12.0);

                // Warning message
                ui.colored_label(
                    egui::Color32::from_rgb(255, 165, 0),
                    tr!("profile-delete-warning"),
                );
                ui.add_space(12.0);

                // Action buttons
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(MaterialButton::filled(tr!("delete"))).clicked() {
                            dialog_action = DeleteDialogAction::Submit;
                        }
                        if ui.add(MaterialButton::outlined(tr!("cancel"))).clicked() {
                            dialog_action = DeleteDialogAction::Cancel;
                        }
                    });
                });
            });

        self.process_action(dialog_action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_default_dialog() {
        let dialog = DlgProfileDelete::new();
        assert!(!dialog.open);
        assert!(dialog.profile_name.is_empty());
        assert!(!dialog.confirmed);
    }

    #[test]
    fn test_open_sets_profile_name_and_state() {
        let mut dialog = DlgProfileDelete::new();
        let profile_name = "test_profile".to_string();

        dialog.open(profile_name.clone());

        assert!(dialog.open);
        assert_eq!(dialog.profile_name, profile_name);
        assert!(!dialog.confirmed);
    }

    #[test]
    fn test_close_closes_dialog() {
        let mut dialog = DlgProfileDelete::new();
        dialog.open = true;

        dialog.close();

        assert!(!dialog.open);
    }

    #[test]
    fn test_reset_clears_all_fields() {
        let mut dialog = DlgProfileDelete {
            open: true,
            profile_name: "test".to_string(),
            confirmed: true,
        };

        dialog.reset();

        assert!(!dialog.open);
        assert!(dialog.profile_name.is_empty());
        assert!(!dialog.confirmed);
    }

    #[test]
    fn test_process_action_submit_returns_true() {
        let mut dialog = DlgProfileDelete {
            open: true,
            profile_name: "test".to_string(),
            confirmed: false,
        };

        let result = dialog.process_action(DeleteDialogAction::Submit);

        assert_eq!(result, Some(true));
        assert!(dialog.confirmed);
        assert!(!dialog.open);
    }

    #[test]
    fn test_process_action_cancel_returns_false() {
        let mut dialog = DlgProfileDelete {
            open: true,
            profile_name: "test".to_string(),
            confirmed: false,
        };

        let result = dialog.process_action(DeleteDialogAction::Cancel);

        assert_eq!(result, Some(false));
        assert!(!dialog.open);
    }

    #[test]
    fn test_process_action_none_returns_none() {
        let mut dialog = DlgProfileDelete {
            open: true,
            profile_name: "test".to_string(),
            confirmed: false,
        };

        let result = dialog.process_action(DeleteDialogAction::None);

        assert_eq!(result, None);
        assert!(dialog.open);
        assert!(!dialog.confirmed);
    }

    #[test]
    fn test_show_returns_none_when_not_open() {
        let mut dialog = DlgProfileDelete {
            open: false,
            profile_name: "test".to_string(),
            confirmed: false,
        };

        let ctx = egui::Context::default();
        let result = dialog.show(&ctx);

        assert_eq!(result, None);
    }

    #[test]
    fn test_multiple_opens_clears_previous_state() {
        let mut dialog = DlgProfileDelete::new();

        dialog.open("first_profile".to_string());
        assert_eq!(dialog.profile_name, "first_profile");

        dialog.open("second_profile".to_string());
        assert_eq!(dialog.profile_name, "second_profile");
        assert!(!dialog.confirmed);
    }

    #[test]
    fn test_confirmed_flag_set_on_submit() {
        let mut dialog = DlgProfileDelete {
            open: true,
            profile_name: "test".to_string(),
            confirmed: false,
        };

        dialog.process_action(DeleteDialogAction::Submit);

        assert!(dialog.confirmed);
    }

    #[test]
    fn test_dialog_closes_after_submit() {
        let mut dialog = DlgProfileDelete {
            open: true,
            profile_name: "test".to_string(),
            confirmed: false,
        };

        dialog.process_action(DeleteDialogAction::Submit);

        assert!(!dialog.open);
    }

    #[test]
    fn test_dialog_closes_after_cancel() {
        let mut dialog = DlgProfileDelete {
            open: true,
            profile_name: "test".to_string(),
            confirmed: false,
        };

        dialog.process_action(DeleteDialogAction::Cancel);

        assert!(!dialog.open);
    }

    #[test]
    fn test_empty_profile_name() {
        let mut dialog = DlgProfileDelete::new();
        dialog.open("".to_string());

        assert_eq!(dialog.profile_name, "");
        assert!(dialog.open);
    }

    #[test]
    fn test_special_characters_in_profile_name() {
        let mut dialog = DlgProfileDelete::new();
        let profile_name = "test-profile_123!@#".to_string();

        dialog.open(profile_name.clone());

        assert_eq!(dialog.profile_name, profile_name);
    }
}

