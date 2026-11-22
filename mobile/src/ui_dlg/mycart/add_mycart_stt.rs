#[derive(Default)]
pub struct AddMycartDialog {
    pub open: bool,
    pub site_address: String,
    pub admin_email: String,
    pub password: String,
    pub ssh_host: String,
    pub platform_id: String,
    pub status_message: String,
    pub in_progress: bool,
}

pub enum AddMycartAction {
    Add {
        site_address: String,
        email: String,
        password: String,
        ssh_host: String,
        platform_id: String,
    },
    Cancel,
    None,
}
