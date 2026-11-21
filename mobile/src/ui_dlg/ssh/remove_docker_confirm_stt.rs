#[derive(Default)]
pub struct RemoveDockerConfirmDialog {
    pub open: bool,
    pub host: String,
}

pub enum RemoveDockerConfirmAction {
    Remove {
        host: String,
    },
    Cancel,
    None,
}
