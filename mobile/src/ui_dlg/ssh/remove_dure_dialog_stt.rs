#[derive(Default)]
pub struct RemoveDureDialog {
    pub open: bool,
    pub host: String,
    pub delete_data: bool,
}

pub enum RemoveDureAction {
    Remove { host: String, delete_data: bool },
    Cancel,
    None,
}
