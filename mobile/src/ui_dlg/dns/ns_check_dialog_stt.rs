#[derive(Default)]
pub struct NsCheckDialog {
    pub open: bool,
    pub results: Vec<String>,
}

pub enum NsCheckAction {
    Close,
    None,
}
