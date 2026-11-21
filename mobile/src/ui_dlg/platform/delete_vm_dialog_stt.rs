#[derive(Default)]
pub struct DeleteVmDialog {
    pub open: bool,
    pub platform: String,
    pub vm_list: Vec<(String, String, String)>, // (name, zone, status)
    pub selected: Option<usize>,
    pub confirming: bool,
    pub hard_delete: bool,
    pub has_static_ip: bool,
    pub ip_address: Option<String>,
    pub ip_name: Option<String>,
    pub release_ip: bool,
}

pub enum DeleteVmAction {
    Delete {
        vm_name: String,
        zone: String,
        force: bool,
        release_ip: Option<(String, String)>, // (region, address_name)
    },
    Cancel,
    None,
}
