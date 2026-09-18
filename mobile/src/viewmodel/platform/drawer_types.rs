//! Drawer types for tab management and operation logging

use crate::storage::models::opslog::OperationLog;
use serde::{Deserialize, Serialize};

/// Active tab in the platform drawer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawerTab {
    /// Status tab - current platform state
    Status,
    /// Logs tab - stdout logs filtered by project
    Logs,
    /// Operations tab - API operation history from SQLite
    Operations,
}

impl DrawerTab {
    pub fn as_str(&self) -> &'static str {
        match self {
            DrawerTab::Status => "Status",
            DrawerTab::Logs => "Logs",
            DrawerTab::Operations => "Operations",
        }
    }

    pub fn all() -> [DrawerTab; 3] {
        [DrawerTab::Status, DrawerTab::Logs, DrawerTab::Operations]
    }
}

impl Default for DrawerTab {
    fn default() -> Self {
        DrawerTab::Status
    }
}

/// Commands for drawer operations
#[derive(Debug, Clone)]
pub enum DrawerCommand {
    /// Switch to a different tab
    SwitchTab {
        tab: DrawerTab,
    },
    /// Load operation logs for a project
    LoadOperations {
        project_id: String,
        limit: i64,
    },
    /// Load stdout logs filtered by project
    LoadLogs {
        project_id: String,
        limit: usize,
    },
    /// Refresh current tab data
    Refresh,
}

/// Events from drawer operations
#[derive(Debug, Clone)]
pub enum DrawerEvent {
    /// Tab switched successfully
    TabSwitched {
        tab: DrawerTab,
    },
    /// Operation logs loaded
    OperationsLoaded {
        project_id: String,
        logs: Vec<OperationLog>,
    },
    /// Stdout logs loaded
    LogsLoaded {
        project_id: String,
        lines: Vec<String>,
    },
    /// Error occurred
    Error {
        operation: String,
        error: String,
    },
}

/// Drawer state
#[derive(Debug, Clone)]
pub struct DrawerState {
    /// Currently active tab
    pub active_tab: DrawerTab,
    /// Current project ID (for filtering logs/operations)
    pub project_id: Option<String>,
    /// Cached operation logs
    pub operations: Vec<OperationLog>,
    /// Cached stdout logs
    pub logs: Vec<String>,
    /// Loading state
    pub loading: bool,
}

impl DrawerState {
    pub fn new() -> Self {
        Self {
            active_tab: DrawerTab::default(),
            project_id: None,
            operations: Vec::new(),
            logs: Vec::new(),
            loading: false,
        }
    }

    pub fn set_project(&mut self, project_id: impl Into<String>) {
        self.project_id = Some(project_id.into());
        // Clear cached data when project changes
        self.operations.clear();
        self.logs.clear();
    }

    pub fn switch_tab(&mut self, tab: DrawerTab) {
        self.active_tab = tab;
    }

    pub fn set_operations(&mut self, ops: Vec<OperationLog>) {
        self.operations = ops;
        self.loading = false;
    }

    pub fn set_logs(&mut self, logs: Vec<String>) {
        self.logs = logs;
        self.loading = false;
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }
}

impl Default for DrawerState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drawer_tab_default() {
        assert_eq!(DrawerTab::default(), DrawerTab::Status);
    }

    #[test]
    fn test_drawer_tab_as_str() {
        assert_eq!(DrawerTab::Status.as_str(), "Status");
        assert_eq!(DrawerTab::Logs.as_str(), "Logs");
        assert_eq!(DrawerTab::Operations.as_str(), "Operations");
    }

    #[test]
    fn test_drawer_tab_all() {
        let tabs = DrawerTab::all();
        assert_eq!(tabs.len(), 3);
        assert_eq!(tabs[0], DrawerTab::Status);
        assert_eq!(tabs[1], DrawerTab::Logs);
        assert_eq!(tabs[2], DrawerTab::Operations);
    }

    #[test]
    fn test_drawer_state_new() {
        let state = DrawerState::new();
        assert_eq!(state.active_tab, DrawerTab::Status);
        assert!(state.project_id.is_none());
        assert!(state.operations.is_empty());
        assert!(state.logs.is_empty());
        assert!(!state.loading);
    }

    #[test]
    fn test_drawer_state_set_project() {
        let mut state = DrawerState::new();
        state.operations.push(OperationLog {
            id: 1,
            project_id: "old-project".to_string(),
            operation_type: "test".to_string(),
            external_system: "gcp".to_string(),
            status: "success".to_string(),
            started_at: 0,
            completed_at: Some(0),
            error_message: None,
            details: None,
        });
        state.logs.push("old log".to_string());

        state.set_project("new-project");

        assert_eq!(state.project_id, Some("new-project".to_string()));
        assert!(state.operations.is_empty(), "Operations should be cleared");
        assert!(state.logs.is_empty(), "Logs should be cleared");
    }

    #[test]
    fn test_drawer_state_switch_tab() {
        let mut state = DrawerState::new();
        assert_eq!(state.active_tab, DrawerTab::Status);

        state.switch_tab(DrawerTab::Logs);
        assert_eq!(state.active_tab, DrawerTab::Logs);

        state.switch_tab(DrawerTab::Operations);
        assert_eq!(state.active_tab, DrawerTab::Operations);
    }

    #[test]
    fn test_drawer_state_loading() {
        let mut state = DrawerState::new();
        assert!(!state.loading);

        state.set_loading(true);
        assert!(state.loading);

        state.set_operations(vec![]);
        assert!(!state.loading, "set_operations should clear loading flag");
    }

    #[test]
    fn test_drawer_command_variants() {
        let cmd1 = DrawerCommand::SwitchTab {
            tab: DrawerTab::Logs,
        };
        let cmd2 = DrawerCommand::LoadOperations {
            project_id: "test-123".to_string(),
            limit: 50,
        };
        let cmd3 = DrawerCommand::LoadLogs {
            project_id: "test-456".to_string(),
            limit: 100,
        };
        let cmd4 = DrawerCommand::Refresh;

        // Just verify they compile and match correctly
        match cmd1 {
            DrawerCommand::SwitchTab { tab } => assert_eq!(tab, DrawerTab::Logs),
            _ => panic!("Expected SwitchTab"),
        }
        match cmd2 {
            DrawerCommand::LoadOperations { project_id, limit } => {
                assert_eq!(project_id, "test-123");
                assert_eq!(limit, 50);
            }
            _ => panic!("Expected LoadOperations"),
        }
        match cmd3 {
            DrawerCommand::LoadLogs { project_id, limit } => {
                assert_eq!(project_id, "test-456");
                assert_eq!(limit, 100);
            }
            _ => panic!("Expected LoadLogs"),
        }
        match cmd4 {
            DrawerCommand::Refresh => {}
            _ => panic!("Expected Refresh"),
        }
    }

    #[test]
    fn test_drawer_event_variants() {
        let evt1 = DrawerEvent::TabSwitched {
            tab: DrawerTab::Operations,
        };
        let evt2 = DrawerEvent::OperationsLoaded {
            project_id: "proj-789".to_string(),
            logs: vec![],
        };
        let evt3 = DrawerEvent::LogsLoaded {
            project_id: "proj-abc".to_string(),
            lines: vec!["line1".to_string()],
        };
        let evt4 = DrawerEvent::Error {
            operation: "load_ops".to_string(),
            error: "DB error".to_string(),
        };

        // Verify they compile and match
        match evt1 {
            DrawerEvent::TabSwitched { tab } => assert_eq!(tab, DrawerTab::Operations),
            _ => panic!("Expected TabSwitched"),
        }
        match evt2 {
            DrawerEvent::OperationsLoaded { project_id, logs } => {
                assert_eq!(project_id, "proj-789");
                assert_eq!(logs.len(), 0);
            }
            _ => panic!("Expected OperationsLoaded"),
        }
        match evt3 {
            DrawerEvent::LogsLoaded { project_id, lines } => {
                assert_eq!(project_id, "proj-abc");
                assert_eq!(lines.len(), 1);
            }
            _ => panic!("Expected LogsLoaded"),
        }
        match evt4 {
            DrawerEvent::Error { operation, error } => {
                assert_eq!(operation, "load_ops");
                assert_eq!(error, "DB error");
            }
            _ => panic!("Expected Error"),
        }
    }
}
