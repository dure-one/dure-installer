#[derive(Default)]
pub struct PlatformErrorDialog {
    pub open: bool,
    pub error_message: String,
}

pub enum ErrorDialogAction {
    Ok,
    None,
}
