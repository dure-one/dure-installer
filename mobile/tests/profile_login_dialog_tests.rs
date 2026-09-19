//! Tests for ProfileLoginDialog
//!
//! These tests verify the state management and behavior of the ProfileLoginDialog widget.
//! Note: Full egui rendering is tested in e2e tests; these focus on logic.

#[cfg(all(test, feature = "gui"))]
mod profile_login_dialog_tests {
    use dure::ui_dlg::DlgProfileLogin;

    #[test]
    fn test_new_creates_default_dialog() {
        let dialog = DlgProfileLogin::new();
        assert!(!dialog.open);
        assert!(dialog.password.is_empty());
        assert!(dialog.error_message.is_empty());
        assert!(!dialog.confirmed);
    }

    #[test]
    fn test_open_initializes_state() {
        let mut dialog = DlgProfileLogin::new();
        dialog.open();

        assert!(dialog.open);
        assert!(dialog.password.is_empty());
        assert!(dialog.error_message.is_empty());
        assert!(!dialog.confirmed);
    }

    #[test]
    fn test_open_clears_previous_password() {
        let mut dialog = DlgProfileLogin::new();
        dialog.password = "old_password".to_string();

        dialog.open();

        assert!(dialog.password.is_empty());
    }

    #[test]
    fn test_open_clears_previous_error() {
        let mut dialog = DlgProfileLogin::new();
        dialog.error_message = "previous error".to_string();

        dialog.open();

        assert!(dialog.error_message.is_empty());
    }

    #[test]
    fn test_close_closes_dialog() {
        let mut dialog = DlgProfileLogin::new();
        dialog.open = true;

        dialog.close();

        assert!(!dialog.open);
    }

    #[test]
    fn test_reset_clears_all_state() {
        let mut dialog = DlgProfileLogin::new();
        dialog.open = true;
        dialog.password = "test_password".to_string();
        dialog.error_message = "test error".to_string();
        dialog.confirmed = true;

        dialog.reset();

        assert!(!dialog.open);
        assert!(dialog.password.is_empty());
        assert!(dialog.error_message.is_empty());
        assert!(!dialog.confirmed);
    }

    #[test]
    fn test_set_error_updates_error_message() {
        let mut dialog = DlgProfileLogin::new();
        let error_msg = "Incorrect password".to_string();

        dialog.set_error(error_msg.clone());

        assert_eq!(dialog.error_message, error_msg);
    }

    #[test]
    fn test_set_error_overwrites_previous_error() {
        let mut dialog = DlgProfileLogin::new();
        dialog.error_message = "old error".to_string();

        dialog.set_error("new error".to_string());

        assert_eq!(dialog.error_message, "new error");
    }

    #[test]
    fn test_default_trait_creates_closed_dialog() {
        let dialog = DlgProfileLogin::default();
        assert!(!dialog.open);
        assert!(dialog.password.is_empty());
        assert!(dialog.error_message.is_empty());
        assert!(!dialog.confirmed);
    }

    #[test]
    fn test_password_field_stores_input() {
        let mut dialog = DlgProfileLogin::new();
        dialog.password = "my_secret_password".to_string();

        assert_eq!(dialog.password, "my_secret_password");
    }

    #[test]
    fn test_multiple_set_error_calls() {
        let mut dialog = DlgProfileLogin::new();

        dialog.set_error("error 1".to_string());
        assert_eq!(dialog.error_message, "error 1");

        dialog.set_error("error 2".to_string());
        assert_eq!(dialog.error_message, "error 2");

        dialog.set_error("error 3".to_string());
        assert_eq!(dialog.error_message, "error 3");
    }
}
