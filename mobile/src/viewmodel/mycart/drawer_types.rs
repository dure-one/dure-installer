//! Drawer types for Mycart tab management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::api::mycart::{CrontabJob, MycartStats};
use crate::storage::models::opslog::OperationLog;
use crate::ui_components::drawer::DrawerTabTrait;

/// Active tab in the Mycart drawer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawerTab {
    /// Status tab - Mycart statistics (orders, products, messages, seller info)
    Status,
    /// Operations tab - Crontab management
    Operations,
    /// Mycart service logs
    Mycart,
    /// XMPPProxy service logs
    XMPPProxy,
    /// Fail2BanRS service logs
    Fail2BanRS,
}

impl DrawerTab {
    pub fn as_str(&self) -> &'static str {
        match self {
            DrawerTab::Status => "Status",
            DrawerTab::Operations => "Operations",
            DrawerTab::Mycart => "Mycart",
            DrawerTab::XMPPProxy => "XMPPProxy",
            DrawerTab::Fail2BanRS => "Fail2BanRS",
        }
    }

    pub fn all() -> [DrawerTab; 5] {
        [
            DrawerTab::Status,
            DrawerTab::Operations,
            DrawerTab::Mycart,
            DrawerTab::XMPPProxy,
            DrawerTab::Fail2BanRS,
        ]
    }

    pub fn to_index(&self) -> usize {
        match self {
            DrawerTab::Status => 0,
            DrawerTab::Operations => 1,
            DrawerTab::Mycart => 2,
            DrawerTab::XMPPProxy => 3,
            DrawerTab::Fail2BanRS => 4,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => DrawerTab::Status,
            1 => DrawerTab::Operations,
            2 => DrawerTab::Mycart,
            3 => DrawerTab::XMPPProxy,
            4 => DrawerTab::Fail2BanRS,
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
    // Phase 4: Service-specific logs
    pub mycart_logs: HashMap<String, String>,
    pub mycart_loading: bool,
    pub xmppproxy_logs: HashMap<String, String>,
    pub xmppproxy_loading: bool,
    pub fail2banrs_logs: HashMap<String, String>,
    pub fail2banrs_loading: bool,
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
            mycart_logs: HashMap::new(),
            mycart_loading: false,
            xmppproxy_logs: HashMap::new(),
            xmppproxy_loading: false,
            fail2banrs_logs: HashMap::new(),
            fail2banrs_loading: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawer_tab_enum() {
        assert_eq!(DrawerTab::Status.to_index(), 0);
        assert_eq!(DrawerTab::Operations.to_index(), 1);
        assert_eq!(DrawerTab::Mycart.to_index(), 2);
        assert_eq!(DrawerTab::XMPPProxy.to_index(), 3);
        assert_eq!(DrawerTab::Fail2BanRS.to_index(), 4);
        assert_eq!(DrawerTab::from_index(0), DrawerTab::Status);
        assert_eq!(DrawerTab::from_index(1), DrawerTab::Operations);
        assert_eq!(DrawerTab::from_index(2), DrawerTab::Mycart);
        assert_eq!(DrawerTab::from_index(3), DrawerTab::XMPPProxy);
        assert_eq!(DrawerTab::from_index(4), DrawerTab::Fail2BanRS);
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
        assert_eq!(tabs.len(), 5);
        assert_eq!(tabs[0], DrawerTab::Status);
        assert_eq!(tabs[1], DrawerTab::Operations);
        assert_eq!(tabs[2], DrawerTab::Mycart);
        assert_eq!(tabs[3], DrawerTab::XMPPProxy);
        assert_eq!(tabs[4], DrawerTab::Fail2BanRS);
    }
}
