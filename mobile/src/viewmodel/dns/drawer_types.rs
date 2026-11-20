//! Drawer types for DNS tab management

use serde::{Deserialize, Serialize};

/// Active tab in the DNS drawer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawerTab {
    /// Operations tab - DNS zone and record operations
    Operations,
    /// Logs tab - Operation logs
    Logs,
}

impl DrawerTab {
    pub fn as_str(&self) -> &'static str {
        match self {
            DrawerTab::Operations => "Operations",
            DrawerTab::Logs => "Logs",
        }
    }

    pub fn all() -> [DrawerTab; 2] {
        [DrawerTab::Operations, DrawerTab::Logs]
    }

    pub fn to_index(&self) -> usize {
        match self {
            DrawerTab::Operations => 0,
            DrawerTab::Logs => 1,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => DrawerTab::Operations,
            1 => DrawerTab::Logs,
            _ => DrawerTab::default(),
        }
    }
}

impl Default for DrawerTab {
    fn default() -> Self {
        DrawerTab::Operations
    }
}

/// Drawer state for DNS tab
#[derive(Debug, Clone)]
pub struct DrawerState {
    pub open: bool,
    pub active_tab: DrawerTab,
    pub width: f32,
}

impl Default for DrawerState {
    fn default() -> Self {
        Self {
            open: false,
            active_tab: DrawerTab::Operations,
            width: 400.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawer_tab_enum() {
        assert_eq!(DrawerTab::Operations.to_index(), 0);
        assert_eq!(DrawerTab::Logs.to_index(), 1);
        assert_eq!(DrawerTab::from_index(0), DrawerTab::Operations);
        assert_eq!(DrawerTab::from_index(1), DrawerTab::Logs);
        assert_eq!(DrawerTab::from_index(99), DrawerTab::Operations);
    }

    #[test]
    fn test_drawer_state_initialization() {
        let state = DrawerState::default();
        assert!(!state.open);
        assert_eq!(state.active_tab, DrawerTab::Operations);
        assert_eq!(state.width, 400.0);
    }

    #[test]
    fn test_drawer_tab_all() {
        let tabs = DrawerTab::all();
        assert_eq!(tabs.len(), 2);
        assert_eq!(tabs[0], DrawerTab::Operations);
        assert_eq!(tabs[1], DrawerTab::Logs);
    }
}
