//! Drawer types for Mycart tab management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::api::mycart::MycartStats;
use crate::storage::models::opslog::OperationLog;
use crate::ui_components::drawer::DrawerTabTrait;

/// Active tab in the Mycart drawer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawerTab {
    /// Status tab - Mycart statistics (orders, products, messages, seller info)
    Status,
    /// Logs tab - Log files from /srv/data/logs
    Logs,
}

impl DrawerTab {
    pub fn as_str(&self) -> &'static str {
        match self {
            DrawerTab::Status => "Status",
            DrawerTab::Logs => "Logs",
        }
    }

    pub fn all() -> [DrawerTab; 2] {
        [DrawerTab::Status, DrawerTab::Logs]
    }

    pub fn to_index(&self) -> usize {
        match self {
            DrawerTab::Status => 0,
            DrawerTab::Logs => 1,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => DrawerTab::Status,
            1 => DrawerTab::Logs,
            _ => DrawerTab::default(),
        }
    }
}

impl DrawerTabTrait for DrawerTab {
    fn as_str(&self) -> &'static str {
        DrawerTab::as_str(self)
    }

    fn all() -> &'static [Self] {
        &[Self::Status, Self::Logs]
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
    pub logs: HashMap<String, String>,
    pub operations: Vec<OperationLog>,
}

impl Default for DrawerState {
    fn default() -> Self {
        Self {
            active_tab: DrawerTab::Status,
            stats: None,
            logs: HashMap::new(),
            operations: Vec::new(),
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
        assert_eq!(DrawerTab::from_index(0), DrawerTab::Status);
        assert_eq!(DrawerTab::from_index(1), DrawerTab::Logs);
        assert_eq!(DrawerTab::from_index(99), DrawerTab::Status); // Default
    }

    #[test]
    fn test_drawer_state_initialization() {
        let state = DrawerState::default();
        assert_eq!(state.active_tab, DrawerTab::Status);
        assert!(state.stats.is_none());
        assert!(state.logs.is_empty());
        assert!(state.operations.is_empty());
    }

    #[test]
    fn test_drawer_tab_all() {
        let tabs = DrawerTab::all();
        assert_eq!(tabs.len(), 2);
        assert_eq!(tabs[0], DrawerTab::Status);
        assert_eq!(tabs[1], DrawerTab::Logs);
    }
}
