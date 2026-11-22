//! Drawer types for Mycart tab management

use serde::{Deserialize, Serialize};

use crate::api::mycart::{CrontabJob, MycartStats};
use crate::storage::models::opslog::OperationLog;
use crate::ui_components::drawer::DrawerTabTrait;

/// Active tab in the Mycart drawer (6 tabs: status + logs + ops + 3 service logs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawerTab {
    /// Status tab - Mycart statistics (orders, products, messages, seller info)
    Status,
    /// Logs tab - Dure installer stdout logs filtered by site (like platform)
    Logs,
    /// Operations tab - Operation history from SQLite
    Operations,
    /// Mycart service logs (SSH server logs)
    Mycart,
    /// XMPPProxy service logs (SSH server logs)
    XMPPProxy,
    /// Fail2BanRS service logs (SSH server logs)
    Fail2BanRS,
}

impl DrawerTab {
    pub fn as_str(&self) -> &'static str {
        match self {
            DrawerTab::Status => "Status",
            DrawerTab::Logs => "Logs",
            DrawerTab::Operations => "Operations",
            DrawerTab::Mycart => "Mycart",
            DrawerTab::XMPPProxy => "XMPPProxy",
            DrawerTab::Fail2BanRS => "Fail2BanRS",
        }
    }

    pub fn all() -> [DrawerTab; 6] {
        [
            DrawerTab::Status,
            DrawerTab::Logs,
            DrawerTab::Operations,
            DrawerTab::Mycart,
            DrawerTab::XMPPProxy,
            DrawerTab::Fail2BanRS,
        ]
    }

    pub fn to_index(&self) -> usize {
        match self {
            DrawerTab::Status => 0,
            DrawerTab::Logs => 1,
            DrawerTab::Operations => 2,
            DrawerTab::Mycart => 3,
            DrawerTab::XMPPProxy => 4,
            DrawerTab::Fail2BanRS => 5,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => DrawerTab::Status,
            1 => DrawerTab::Logs,
            2 => DrawerTab::Operations,
            3 => DrawerTab::Mycart,
            4 => DrawerTab::XMPPProxy,
            5 => DrawerTab::Fail2BanRS,
            _ => DrawerTab::default(),
        }
    }
}

impl DrawerTabTrait for DrawerTab {
    fn as_str(&self) -> &'static str {
        DrawerTab::as_str(self)
    }

    fn all() -> &'static [Self] {
        &[
            Self::Status,
            Self::Logs,
            Self::Operations,
            Self::Mycart,
            Self::XMPPProxy,
            Self::Fail2BanRS,
        ]
    }
}

impl Default for DrawerTab {
    fn default() -> Self {
        DrawerTab::Status
    }
}

/// Drawer state for Mycart tab
#[derive(Debug, Clone)]
pub struct DrawerState {
    pub active_tab: DrawerTab,
    pub stats: Option<MycartStats>,
    pub operations: Vec<OperationLog>,
    // Phase 3: Crontab management fields
    pub crontab_jobs: Vec<CrontabJob>,
    pub crontab_loading: bool,
    pub crontab_error: Option<String>,
    // Phase 5: Installer logs (stdout filtered by site, like platform)
    pub logs: Vec<String>,
    pub logs_loading: bool,
    // Phase 5: SSH server logs (3 services) - per-file storage
    pub mycart_logs: std::collections::HashMap<String, Vec<String>>,
    pub mycart_loading: bool,
    pub xmppproxy_logs: std::collections::HashMap<String, Vec<String>>,
    pub xmppproxy_loading: bool,
    pub fail2banrs_logs: std::collections::HashMap<String, Vec<String>>,
    pub fail2banrs_loading: bool,
    // Phase 6: Config files for each service
    pub mycart_config: Option<String>,
    pub mycart_config_loading: bool,
    pub xmppproxy_config: Option<String>,
    pub xmppproxy_config_loading: bool,
    pub fail2banrs_config: Option<String>,
    pub fail2banrs_config_loading: bool,
    pub fail2banrs_nft: Option<String>,
    pub fail2banrs_nft_loading: bool,
}

impl Default for DrawerState {
    fn default() -> Self {
        Self {
            active_tab: DrawerTab::Status,
            stats: None,
            operations: Vec::new(),
            crontab_jobs: Vec::new(),
            crontab_loading: false,
            crontab_error: None,
            logs: Vec::new(),
            logs_loading: false,
            mycart_logs: std::collections::HashMap::new(),
            mycart_loading: false,
            xmppproxy_logs: std::collections::HashMap::new(),
            xmppproxy_loading: false,
            fail2banrs_logs: std::collections::HashMap::new(),
            fail2banrs_loading: false,
            mycart_config: None,
            mycart_config_loading: false,
            xmppproxy_config: None,
            xmppproxy_config_loading: false,
            fail2banrs_config: None,
            fail2banrs_config_loading: false,
            fail2banrs_nft: None,
            fail2banrs_nft_loading: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawer_tab_enum() {
        assert_eq!(DrawerTab::Status.to_index(), 0);
        assert_eq!(DrawerTab::Logs.to_index(), 1);
        assert_eq!(DrawerTab::Operations.to_index(), 2);
        assert_eq!(DrawerTab::Mycart.to_index(), 3);
        assert_eq!(DrawerTab::XMPPProxy.to_index(), 4);
        assert_eq!(DrawerTab::Fail2BanRS.to_index(), 5);
        assert_eq!(DrawerTab::from_index(0), DrawerTab::Status);
        assert_eq!(DrawerTab::from_index(1), DrawerTab::Logs);
        assert_eq!(DrawerTab::from_index(2), DrawerTab::Operations);
        assert_eq!(DrawerTab::from_index(3), DrawerTab::Mycart);
        assert_eq!(DrawerTab::from_index(4), DrawerTab::XMPPProxy);
        assert_eq!(DrawerTab::from_index(5), DrawerTab::Fail2BanRS);
        assert_eq!(DrawerTab::from_index(99), DrawerTab::Status); // Default
    }

    #[test]
    fn test_drawer_state_initialization() {
        let state = DrawerState::default();
        assert_eq!(state.active_tab, DrawerTab::Status);
        assert!(state.stats.is_none());
        assert!(state.operations.is_empty());
        assert!(state.crontab_jobs.is_empty());
        assert!(!state.crontab_loading);
        assert!(state.crontab_error.is_none());
        assert!(state.logs.is_empty());
        assert!(!state.logs_loading);
        assert!(state.mycart_logs.is_empty());
        assert!(!state.mycart_loading);
        assert!(state.xmppproxy_logs.is_empty());
        assert!(!state.xmppproxy_loading);
        assert!(state.fail2banrs_logs.is_empty());
        assert!(!state.fail2banrs_loading);
    }

    #[test]
    fn test_drawer_tab_all() {
        let tabs = DrawerTab::all();
        assert_eq!(tabs.len(), 6);
        assert_eq!(tabs[0], DrawerTab::Status);
        assert_eq!(tabs[1], DrawerTab::Logs);
        assert_eq!(tabs[2], DrawerTab::Operations);
        assert_eq!(tabs[3], DrawerTab::Mycart);
        assert_eq!(tabs[4], DrawerTab::XMPPProxy);
        assert_eq!(tabs[5], DrawerTab::Fail2BanRS);
    }
}
