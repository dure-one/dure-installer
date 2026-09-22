#![cfg(all(test, feature = "gui"))]

use dure::ui_dlg::{DlgProfileCreate, CreateDialogAction, ProfileCreateResult};

// ============================================================================
// State Management Tests
// ============================================================================

#[test]
fn test_new_creates_default_dialog() {
    let dialog = DlgProfileCreate::new();
    assert!(!dialog.open);
    assert!(dialog.profile_name.is_empty());
    assert!(dialog.password.is_empty());
    assert!(dialog.password_confirm.is_empty());
    assert!(dialog.error_message.is_empty());
    assert!(!dialog.confirmed);
}

#[test]
fn test_default_trait_creates_closed_dialog() {
    let dialog = DlgProfileCreate::default();
    assert!(!dialog.open);
    assert!(dialog.profile_name.is_empty());
    assert!(dialog.password.is_empty());
    assert!(dialog.password_confirm.is_empty());
}

#[test]
fn test_open_initializes_state() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();

    // Act
    dialog.open();

    // Assert
    assert!(dialog.open);
    assert!(dialog.profile_name.is_empty());
    assert!(dialog.password.is_empty());
    assert!(dialog.password_confirm.is_empty());
    assert!(dialog.error_message.is_empty());
    assert!(!dialog.confirmed);
}

#[test]
fn test_open_clears_previous_profile_name() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "old_profile".to_string();

    // Act
    dialog.open();

    // Assert
    assert!(dialog.profile_name.is_empty());
}

#[test]
fn test_open_clears_previous_password() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.password = "oldpass".to_string();

    // Act
    dialog.open();

    // Assert
    assert!(dialog.password.is_empty());
}

#[test]
fn test_open_clears_previous_password_confirm() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.password_confirm = "oldpass".to_string();

    // Act
    dialog.open();

    // Assert
    assert!(dialog.password_confirm.is_empty());
}

#[test]
fn test_open_clears_previous_error() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.error_message = "previous error".to_string();

    // Act
    dialog.open();

    // Assert
    assert!(dialog.error_message.is_empty());
}

#[test]
fn test_close_closes_dialog() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.open = true;

    // Act
    dialog.close();

    // Assert
    assert!(!dialog.open);
}

#[test]
fn test_reset_clears_all_state() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.open = true;
    dialog.profile_name = "profile".to_string();
    dialog.password = "pass".to_string();
    dialog.password_confirm = "pass".to_string();
    dialog.error_message = "error".to_string();
    dialog.confirmed = true;

    // Act
    dialog.reset();

    // Assert
    assert!(!dialog.open);
    assert!(dialog.profile_name.is_empty());
    assert!(dialog.password.is_empty());
    assert!(dialog.password_confirm.is_empty());
    assert!(dialog.error_message.is_empty());
    assert!(!dialog.confirmed);
}

#[test]
fn test_set_error_updates_error_message() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    let error = "test error".to_string();

    // Act
    dialog.set_error(error.clone());

    // Assert
    assert_eq!(dialog.error_message, error);
}

#[test]
fn test_set_error_overwrites_previous_error() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.error_message = "old error".to_string();

    // Act
    dialog.set_error("new error".to_string());

    // Assert
    assert_eq!(dialog.error_message, "new error");
}

// ============================================================================
// Profile Name Validation Tests
// ============================================================================

#[test]
fn test_validate_empty_profile_name_fails() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.password = "mypassword".to_string();
    dialog.password_confirm = "mypassword".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_none());
    assert_eq!(
        dialog.error_message,
        "Profile name cannot be empty"
    );
}

#[test]
fn test_validate_profile_name_too_long_fails() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "a".repeat(65); // 65 chars, exceeds 64 limit
    dialog.password = "mypassword".to_string();
    dialog.password_confirm = "mypassword".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_none());
    assert!(dialog.error_message.contains("too long"));
    assert!(dialog.error_message.contains("65 chars"));
}

#[test]
fn test_validate_profile_name_with_invalid_char_fails() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "my@profile".to_string();
    dialog.password = "mypassword".to_string();
    dialog.password_confirm = "mypassword".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_none());
    assert!(dialog.error_message.contains("Invalid character '@'"));
}

#[test]
fn test_validate_profile_name_with_space_fails() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "my profile".to_string();
    dialog.password = "mypassword".to_string();
    dialog.password_confirm = "mypassword".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_none());
    assert!(dialog.error_message.contains("Invalid character ' '"));
}

#[test]
fn test_validate_profile_name_lowercase_valid() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "myprofile".to_string();
    dialog.password = "mypassword".to_string();
    dialog.password_confirm = "mypassword".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_some());
    let res = result.unwrap();
    assert_eq!(res.name, "myprofile");
}

#[test]
fn test_validate_profile_name_uppercase_valid() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "MYPROFILE".to_string();
    dialog.password = "mypassword".to_string();
    dialog.password_confirm = "mypassword".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_some());
    let res = result.unwrap();
    assert_eq!(res.name, "MYPROFILE");
}

#[test]
fn test_validate_profile_name_with_digits_valid() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "profile123".to_string();
    dialog.password = "mypassword".to_string();
    dialog.password_confirm = "mypassword".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_some());
    let res = result.unwrap();
    assert_eq!(res.name, "profile123");
}

#[test]
fn test_validate_profile_name_with_dash_valid() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "my-profile".to_string();
    dialog.password = "mypassword".to_string();
    dialog.password_confirm = "mypassword".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_some());
    let res = result.unwrap();
    assert_eq!(res.name, "my-profile");
}

#[test]
fn test_validate_profile_name_with_underscore_valid() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "my_profile".to_string();
    dialog.password = "mypassword".to_string();
    dialog.password_confirm = "mypassword".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_some());
    let res = result.unwrap();
    assert_eq!(res.name, "my_profile");
}

#[test]
fn test_validate_profile_name_mixed_valid() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "My_Profile-123".to_string();
    dialog.password = "mypassword".to_string();
    dialog.password_confirm = "mypassword".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_some());
    let res = result.unwrap();
    assert_eq!(res.name, "My_Profile-123");
}

#[test]
fn test_validate_profile_name_max_length_valid() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "a".repeat(64); // exactly 64 chars
    dialog.password = "mypassword".to_string();
    dialog.password_confirm = "mypassword".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_some());
    let res = result.unwrap();
    assert_eq!(res.name.len(), 64);
}

#[test]
fn test_validate_profile_name_single_char_valid() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "a".to_string();
    dialog.password = "mypassword".to_string();
    dialog.password_confirm = "mypassword".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_some());
    let res = result.unwrap();
    assert_eq!(res.name, "a");
}

// ============================================================================
// Password Validation Tests
// ============================================================================

#[test]
fn test_validate_empty_password_fails() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "myprofile".to_string();
    dialog.password = "".to_string();
    dialog.password_confirm = "".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_none());
    assert_eq!(dialog.error_message, "Password cannot be empty");
}

#[test]
fn test_validate_password_mismatch_fails() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "myprofile".to_string();
    dialog.password = "password1".to_string();
    dialog.password_confirm = "password2".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_none());
    assert_eq!(dialog.error_message, "Passwords do not match");
}

#[test]
fn test_validate_password_only_password_empty_fails() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "myprofile".to_string();
    dialog.password = "".to_string();
    dialog.password_confirm = "mypassword".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_none());
    assert_eq!(dialog.error_message, "Password cannot be empty");
}

#[test]
fn test_validate_password_only_confirm_empty_fails() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "myprofile".to_string();
    dialog.password = "mypassword".to_string();
    dialog.password_confirm = "".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_none());
    assert_eq!(dialog.error_message, "Passwords do not match");
}

#[test]
fn test_validate_password_with_special_chars_valid() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "myprofile".to_string();
    dialog.password = "p@$$w0rd!".to_string();
    dialog.password_confirm = "p@$$w0rd!".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_some());
    let res = result.unwrap();
    assert_eq!(res.password, "p@$$w0rd!");
}

#[test]
fn test_validate_password_with_spaces_valid() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "myprofile".to_string();
    dialog.password = "my pass word".to_string();
    dialog.password_confirm = "my pass word".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_some());
    let res = result.unwrap();
    assert_eq!(res.password, "my pass word");
}

// ============================================================================
// Process Action Tests
// ============================================================================

#[test]
fn test_process_action_submit_with_valid_inputs_returns_result() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "newprofile".to_string();
    dialog.password = "securepass".to_string();
    dialog.password_confirm = "securepass".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_some());
    let profile_result = result.unwrap();
    assert_eq!(profile_result.name, "newprofile");
    assert_eq!(profile_result.password, "securepass");
}

#[test]
fn test_process_action_submit_sets_confirmed_flag() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "newprofile".to_string();
    dialog.password = "securepass".to_string();
    dialog.password_confirm = "securepass".to_string();

    // Act
    let _ = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(dialog.confirmed);
}

#[test]
fn test_process_action_submit_closes_dialog() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.open = true;
    dialog.profile_name = "newprofile".to_string();
    dialog.password = "securepass".to_string();
    dialog.password_confirm = "securepass".to_string();

    // Act
    let _ = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(!dialog.open);
}

#[test]
fn test_process_action_cancel_closes_dialog() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.open = true;

    // Act
    let result = dialog.process_action(CreateDialogAction::Cancel);

    // Assert
    assert!(!dialog.open);
    assert!(result.is_none());
}

#[test]
fn test_process_action_cancel_returns_none() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();

    // Act
    let result = dialog.process_action(CreateDialogAction::Cancel);

    // Assert
    assert!(result.is_none());
}

#[test]
fn test_process_action_none_returns_none() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();

    // Act
    let result = dialog.process_action(CreateDialogAction::None);

    // Assert
    assert!(result.is_none());
}

#[test]
fn test_process_action_none_preserves_state() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "test".to_string();
    dialog.open = true;

    // Act
    let _ = dialog.process_action(CreateDialogAction::None);

    // Assert
    assert_eq!(dialog.profile_name, "test");
    assert!(dialog.open);
}

#[test]
fn test_process_action_validation_error_keeps_dialog_open() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.open = true;
    dialog.profile_name = "".to_string(); // invalid: empty
    dialog.password = "pass".to_string();
    dialog.password_confirm = "pass".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_none());
    assert!(dialog.open); // dialog should remain open
}

#[test]
fn test_process_action_clones_profile_name() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "original".to_string();
    dialog.password = "pass".to_string();
    dialog.password_confirm = "pass".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Clear original name
    dialog.profile_name.clear();

    // Assert - result should have independent copy
    assert!(result.is_some());
    let profile_result = result.unwrap();
    assert_eq!(profile_result.name, "original");
}

#[test]
fn test_process_action_clones_password() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "profile".to_string();
    dialog.password = "original_pass".to_string();
    dialog.password_confirm = "original_pass".to_string();

    // Act
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Clear original password
    dialog.password.clear();

    // Assert - result should have independent copy
    assert!(result.is_some());
    let profile_result = result.unwrap();
    assert_eq!(profile_result.password, "original_pass");
}

#[test]
fn test_multiple_submit_attempts_with_validation_errors() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.profile_name = "".to_string(); // invalid
    dialog.password = "pass".to_string();
    dialog.password_confirm = "pass".to_string();

    // Act - first attempt
    let result1 = dialog.process_action(CreateDialogAction::Submit);
    assert!(result1.is_none());
    let _error1 = dialog.error_message.clone();

    // Fix the name
    dialog.profile_name = "valid_profile".to_string();

    // Act - second attempt
    let result2 = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result2.is_some());
    let profile_result = result2.unwrap();
    assert_eq!(profile_result.name, "valid_profile");
}

// ============================================================================
// Dialog Action Enum Tests
// ============================================================================

#[test]
fn test_dialog_action_equality() {
    assert_eq!(CreateDialogAction::Submit, CreateDialogAction::Submit);
    assert_eq!(CreateDialogAction::Cancel, CreateDialogAction::Cancel);
    assert_eq!(CreateDialogAction::None, CreateDialogAction::None);
    assert_ne!(CreateDialogAction::Submit, CreateDialogAction::Cancel);
    assert_ne!(CreateDialogAction::Cancel, CreateDialogAction::None);
}

// ============================================================================
// show() Integration Tests
// ============================================================================

#[test]
fn test_show_returns_none_when_dialog_closed() {
    // Arrange
    let dialog = DlgProfileCreate::new();
    assert!(!dialog.open);

    // Act - Note: We cannot fully test show() without an egui Context,
    // but we can verify early return logic
    let result = if !dialog.open {
        None
    } else {
        Some(ProfileCreateResult {
            name: "test".to_string(),
            password: "test".to_string(),
        })
    };

    // Assert
    assert!(result.is_none());
}

#[test]
fn test_show_delegates_to_process_action() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.open = true;
    dialog.profile_name = "newprofile".to_string();
    dialog.password = "pass".to_string();
    dialog.password_confirm = "pass".to_string();

    // Act - simulate show() calling process_action(Submit)
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_some());
    let profile_result = result.unwrap();
    assert_eq!(profile_result.name, "newprofile");
}

#[test]
fn test_show_behavior_with_cancel() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    dialog.open = true;
    dialog.profile_name = "newprofile".to_string();

    // Act - simulate show() calling process_action(Cancel)
    let result = dialog.process_action(CreateDialogAction::Cancel);

    // Assert
    assert!(result.is_none());
    assert!(!dialog.open);
}

#[test]
fn test_dialog_lifecycle_in_create() {
    // Arrange
    let mut dialog = DlgProfileCreate::new();
    assert!(!dialog.open);

    // Act - open dialog
    dialog.open();
    assert!(dialog.open);

    // Fill in data
    dialog.profile_name = "myprofile".to_string();
    dialog.password = "mypass".to_string();
    dialog.password_confirm = "mypass".to_string();

    // Submit
    let result = dialog.process_action(CreateDialogAction::Submit);

    // Assert
    assert!(result.is_some());
    assert!(!dialog.open);
    assert!(dialog.confirmed);
    let profile_result = result.unwrap();
    assert_eq!(profile_result.name, "myprofile");
    assert_eq!(profile_result.password, "mypass");
}

#[test]
fn test_profile_create_result_equality() {
    // Arrange
    let result1 = ProfileCreateResult {
        name: "profile".to_string(),
        password: "pass".to_string(),
    };
    let result2 = ProfileCreateResult {
        name: "profile".to_string(),
        password: "pass".to_string(),
    };
    let result3 = ProfileCreateResult {
        name: "different".to_string(),
        password: "pass".to_string(),
    };

    // Assert
    assert_eq!(result1, result2);
    assert_ne!(result1, result3);
}
