#[derive(Default)]
pub struct DeleteFirewallConfirmDialog {
    pub open: bool,
    pub rule_name: String,
    pub platform: String,
    pub rule_idx: Option<usize>,
}

pub enum DeleteFirewallConfirmAction {
    Delete {
        rule_name: String,
        platform: String,
        rule_idx: Option<usize>,
    },
    Cancel,
    None,
}
