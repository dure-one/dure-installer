#[derive(Default)]
pub struct DeletePlatformConfirmDialog {
    pub open: bool,
    pub platform_name: String,
    pub vm_count: usize,
    pub delete_vms: bool,
    pub delete_project: bool,
}

pub enum DeletePlatformConfirmAction {
    Delete {
        platform_name: String,
        delete_vms: bool,
        delete_project: bool,
    },
    Cancel,
    None,
}
