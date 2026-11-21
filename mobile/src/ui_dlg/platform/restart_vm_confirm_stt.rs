#[derive(Default)]
pub struct RestartVmConfirmDialog {
    pub open: bool,
    pub vm_name: String,
    pub platform: String,
}

pub enum RestartVmConfirmAction {
    Restart {
        vm_name: String,
        platform: String,
    },
    Cancel,
    None,
}
