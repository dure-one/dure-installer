#[derive(Default)]
pub struct EditHostDialog {
    pub open: bool,
    pub original_host: String,
    pub host: String,
    pub port: String,
    pub use_password: bool,
    pub password: String,
    pub use_private_key: bool,
    pub private_key_path: String,
}

pub enum EditHostAction {
    Save {
        original_host: String,
        host: String,
        port: String,
        use_password: bool,
        password: String,
        use_private_key: bool,
        private_key_path: String,
    },
    Cancel,
    None,
}
