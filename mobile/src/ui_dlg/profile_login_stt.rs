#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Default)]
pub struct DlgProfileLogin {
    pub open: bool,
    pub password: String,
    pub error_message: String,
    pub confirmed: bool,
}

/// Internal representation of dialog button click events.
/// Separated from egui rendering to enable testable logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogAction {
    /// OK button was clicked
    Submit,
    /// Cancel button was clicked
    Cancel,
    /// No button was clicked
    None,
}
