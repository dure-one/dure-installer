#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Default)]
pub struct DlgProfileCreate {
    pub open: bool,
    pub profile_name: String,
    pub password: String,
    pub password_confirm: String,
    pub error_message: String,
    pub confirmed: bool,
}

/// Result of profile creation dialog when user confirms
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileCreateResult {
    pub name: String,
    pub password: String,
}

/// Internal representation of dialog button click events.
/// Separated from egui rendering to enable testable logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreateDialogAction {
    /// Create button was clicked
    Submit,
    /// Cancel button was clicked
    Cancel,
    /// No button was clicked
    None,
}
