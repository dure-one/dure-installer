#[derive(Default)]
pub struct SshErrorDialog {
    pub open: bool,
    pub error_message: String,
}

pub enum ErrorDialogAction {
    Ok,
    None,
}
