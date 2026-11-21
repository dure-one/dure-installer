use crate::ui_tabs::platform::FirewallRule;

#[derive(Default)]
pub struct AddFirewallDialog {
    pub open: bool,
    pub platform: String,
    pub form_data: FirewallRule,
    pub edit_idx: Option<usize>,
    pub save_in_progress: bool,
    pub save_error: Option<String>,
}

pub enum AddFirewallAction {
    Save {
        rule_data: FirewallRule,
        edit_idx: Option<usize>,
    },
    Cancel,
    None,
}
