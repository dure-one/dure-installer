#[derive(Default)]
pub struct BindVmDialog {
    pub open: bool,
    pub platform: String,
    pub vm_list: Vec<(String, String, String, String)>, // (project_id, vm_name, zone, external_ip)
    pub selected: Option<usize>,
    pub loading: bool,
}

pub enum BindVmAction {
    Bind {
        platform: String,
    },
    Cancel,
    None,
}
