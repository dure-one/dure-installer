use crate::ui_tabs::platform::FirewallRule;
use crate::viewmodel::platform::FullFirewallRule;

#[derive(Default)]
pub struct FirewallDialog {
    pub open: bool,
    pub platform: String,
    pub rule_list: Vec<FullFirewallRule>,
    pub rules: Vec<FirewallRule>,
    pub selected: Option<usize>,
    pub selected_rows: Vec<bool>,
    pub edit_data: std::collections::HashMap<usize, FirewallRule>,
    pub last_refresh_time: Option<std::time::Instant>,
    pub current_ip: Option<String>,
    pub show_remove_ip_dialog: bool,
    pub remove_ip_list: Vec<String>,
    pub remove_ip_selected: Vec<bool>,
    pub show_flush_confirm: bool,
}

pub enum FirewallAction {
    AddIp,
    RemoveIp {
        ip_list: Vec<String>,
    },
    Flush,
    AddRule,
    EditRule {
        rule_data: FirewallRule,
        rule_idx: usize,
    },
    DeleteRule {
        rule_name: String,
        rule_idx: Option<usize>,
    },
    Reset,
    Close,
    None,
}
