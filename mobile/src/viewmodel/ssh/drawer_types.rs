//! Drawer types for SSH host tab management and operation logging

use crate::storage::models::opslog::OperationLog;
use serde::{Deserialize, Serialize};

/// Active tab in the SSH drawer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DrawerTab {
    /// Status tab - SSH connection state, Docker/Dure install status
    Status,
    /// Logs tab - stdout logs filtered by SSH host
    Logs,
    /// Operations tab - SSH operation history from SQLite
    Operations,
    /// Host tab - system information (OS, uptime, load, memory, disk)
    Host,
    /// Docker tab - Docker status and containers
    Docker,
    /// Dure tab - Dure WSS status and configuration
    Dure,
}

impl DrawerTab {
    pub fn as_str(&self) -> &'static str {
        match self {
            DrawerTab::Status => "Status",
            DrawerTab::Logs => "Logs",
            DrawerTab::Operations => "Operations",
            DrawerTab::Host => "Host",
            DrawerTab::Docker => "Docker",
            DrawerTab::Dure => "Dure",
        }
    }

    pub fn all() -> [DrawerTab; 6] {
        [
            DrawerTab::Status,
            DrawerTab::Logs,
            DrawerTab::Operations,
            DrawerTab::Host,
            DrawerTab::Docker,
            DrawerTab::Dure,
        ]
    }
}

impl Default for DrawerTab {
    fn default() -> Self {
        DrawerTab::Status
    }
}

/// Host system information
#[derive(Debug, Clone, Default)]
pub struct HostInfo {
    pub os: String,
    pub uptime: String,
    pub external_ip: String,
    pub load_average: String,
    pub memory_usage: String,
    pub disk_usage: String,
    pub top_processes: Vec<String>,
}

/// Docker status
#[derive(Debug, Clone)]
pub struct DockerStatus {
    pub installed: bool,
    pub version: Option<String>,
}

impl Default for DockerStatus {
    fn default() -> Self {
        Self {
            installed: false,
            version: None,
        }
    }
}

/// Docker container information
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub name: String,
    pub image: String,
    pub status: String,
    pub ports: Vec<String>,
}

/// Dure WSS status
#[derive(Debug, Clone)]
pub struct DureStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub running: bool,
}

impl Default for DureStatus {
    fn default() -> Self {
        Self {
            installed: false,
            version: None,
            running: false,
        }
    }
}

/// Commands for drawer operations
#[derive(Debug, Clone)]
pub enum DrawerCommand {
    /// Switch to a different tab
    SwitchTab { tab: DrawerTab },
    /// Load operation logs for an SSH host
    LoadOperations { ssh_host: String, limit: i64 },
    /// Load stdout logs filtered by SSH host
    LoadLogs { ssh_host: String, limit: usize },
    /// Update logs from LogActor response
    UpdateLogs {
        ssh_host: String,
        lines: Vec<String>,
    },
    /// Load host information
    LoadHostInfo { ssh_host: String },
    /// Load Docker status and containers
    LoadDockerInfo { ssh_host: String },
    /// Load Dure status
    LoadDureInfo { ssh_host: String },
    /// Refresh current tab data
    Refresh,
}

/// Events from drawer operations
#[derive(Debug, Clone)]
pub enum DrawerEvent {
    /// Tab switched successfully
    TabSwitched { tab: DrawerTab },
    /// Operation logs loaded
    OperationsLoaded {
        ssh_host: String,
        logs: Vec<OperationLog>,
    },
    /// Stdout logs loaded
    LogsLoaded {
        ssh_host: String,
        lines: Vec<String>,
    },
    /// Host information loaded
    HostInfoLoaded {
        ssh_host: String,
        info: HostInfo,
    },
    /// Docker information loaded
    DockerInfoLoaded {
        ssh_host: String,
        status: DockerStatus,
        containers: Vec<ContainerInfo>,
    },
    /// Dure information loaded
    DureInfoLoaded {
        ssh_host: String,
        status: DureStatus,
    },
    /// Error occurred
    Error { operation: String, error: String },
}

/// Drawer state
#[derive(Debug, Clone)]
pub struct DrawerState {
    /// Currently active tab
    pub active_tab: DrawerTab,
    /// Current SSH host (IP address for filtering)
    pub ssh_host: Option<String>,
    /// Cached operation logs
    pub operations: Vec<OperationLog>,
    /// Cached stdout logs
    pub logs: Vec<String>,
    /// Loading state
    pub loading: bool,
    /// Host information (for Host tab)
    pub host_info: Option<HostInfo>,
    /// Docker status (for Docker tab)
    pub docker_status: Option<DockerStatus>,
    /// Docker containers (for Docker tab)
    pub containers: Vec<ContainerInfo>,
    /// Dure status (for Dure tab)
    pub dure_status: Option<DureStatus>,
}

impl DrawerState {
    pub fn new() -> Self {
        Self {
            active_tab: DrawerTab::default(),
            ssh_host: None,
            operations: Vec::new(),
            logs: Vec::new(),
            loading: false,
            host_info: None,
            docker_status: None,
            containers: Vec::new(),
            dure_status: None,
        }
    }

    pub fn set_ssh_host(&mut self, ssh_host: impl Into<String>) {
        self.ssh_host = Some(ssh_host.into());
        // Clear cached data when SSH host changes
        self.operations.clear();
        self.logs.clear();
        self.host_info = None;
        self.docker_status = None;
        self.containers.clear();
        self.dure_status = None;
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

    pub fn set_host_info(&mut self, info: HostInfo) {
        self.host_info = Some(info);
        self.loading = false;
    }

    pub fn set_docker_info(&mut self, status: DockerStatus, containers: Vec<ContainerInfo>) {
        self.docker_status = Some(status);
        self.containers = containers;
        self.loading = false;
    }

    pub fn set_dure_info(&mut self, status: DureStatus) {
        self.dure_status = Some(status);
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
        assert_eq!(DrawerTab::Host.as_str(), "Host");
        assert_eq!(DrawerTab::Docker.as_str(), "Docker");
        assert_eq!(DrawerTab::Dure.as_str(), "Dure");
    }

    #[test]
    fn test_drawer_tab_all() {
        let tabs = DrawerTab::all();
        assert_eq!(tabs.len(), 6);
        assert_eq!(tabs[0], DrawerTab::Status);
        assert_eq!(tabs[1], DrawerTab::Logs);
        assert_eq!(tabs[2], DrawerTab::Operations);
        assert_eq!(tabs[3], DrawerTab::Host);
        assert_eq!(tabs[4], DrawerTab::Docker);
        assert_eq!(tabs[5], DrawerTab::Dure);
    }

    #[test]
    fn test_drawer_state_new() {
        let state = DrawerState::new();
        assert_eq!(state.active_tab, DrawerTab::Status);
        assert!(state.ssh_host.is_none());
        assert!(state.operations.is_empty());
        assert!(state.logs.is_empty());
        assert!(!state.loading);
        assert!(state.host_info.is_none());
        assert!(state.docker_status.is_none());
        assert!(state.containers.is_empty());
        assert!(state.dure_status.is_none());
    }

    #[test]
    fn test_drawer_state_set_ssh_host() {
        let mut state = DrawerState::new();
        state.operations.push(OperationLog {
            id: 1,
            project_id: "192.168.1.100".to_string(),
            operation_type: "ssh_connect".to_string(),
            external_system: "ssh".to_string(),
            status: "success".to_string(),
            started_at: 0,
            completed_at: Some(0),
            error_message: None,
            details: None,
        });
        state.logs.push("old log".to_string());
        state.host_info = Some(HostInfo::default());
        state.docker_status = Some(DockerStatus::default());
        state.containers.push(ContainerInfo {
            name: "old-container".to_string(),
            image: "nginx".to_string(),
            status: "running".to_string(),
            ports: vec![],
        });
        state.dure_status = Some(DureStatus::default());

        state.set_ssh_host("192.168.1.101");

        assert_eq!(state.ssh_host, Some("192.168.1.101".to_string()));
        assert!(state.operations.is_empty(), "Operations should be cleared");
        assert!(state.logs.is_empty(), "Logs should be cleared");
        assert!(state.host_info.is_none(), "Host info should be cleared");
        assert!(
            state.docker_status.is_none(),
            "Docker status should be cleared"
        );
        assert!(state.containers.is_empty(), "Containers should be cleared");
        assert!(
            state.dure_status.is_none(),
            "Dure status should be cleared"
        );
    }

    #[test]
    fn test_drawer_state_switch_tab() {
        let mut state = DrawerState::new();
        assert_eq!(state.active_tab, DrawerTab::Status);

        state.switch_tab(DrawerTab::Logs);
        assert_eq!(state.active_tab, DrawerTab::Logs);

        state.switch_tab(DrawerTab::Operations);
        assert_eq!(state.active_tab, DrawerTab::Operations);

        state.switch_tab(DrawerTab::Host);
        assert_eq!(state.active_tab, DrawerTab::Host);

        state.switch_tab(DrawerTab::Docker);
        assert_eq!(state.active_tab, DrawerTab::Docker);

        state.switch_tab(DrawerTab::Dure);
        assert_eq!(state.active_tab, DrawerTab::Dure);
    }

    #[test]
    fn test_drawer_state_loading() {
        let mut state = DrawerState::new();
        assert!(!state.loading);

        state.set_loading(true);
        assert!(state.loading);

        state.set_operations(vec![]);
        assert!(!state.loading, "set_operations should clear loading flag");

        state.set_loading(true);
        state.set_logs(vec![]);
        assert!(!state.loading, "set_logs should clear loading flag");

        state.set_loading(true);
        state.set_host_info(HostInfo::default());
        assert!(!state.loading, "set_host_info should clear loading flag");

        state.set_loading(true);
        state.set_docker_info(DockerStatus::default(), vec![]);
        assert!(
            !state.loading,
            "set_docker_info should clear loading flag"
        );

        state.set_loading(true);
        state.set_dure_info(DureStatus::default());
        assert!(!state.loading, "set_dure_info should clear loading flag");
    }

    #[test]
    fn test_docker_status_default() {
        let status = DockerStatus::default();
        assert!(!status.installed);
        assert!(status.version.is_none());
    }

    #[test]
    fn test_dure_status_default() {
        let status = DureStatus::default();
        assert!(!status.installed);
        assert!(status.version.is_none());
        assert!(!status.running);
    }

    #[test]
    fn test_host_info_default() {
        let info = HostInfo::default();
        assert_eq!(info.os, "");
        assert_eq!(info.uptime, "");
        assert_eq!(info.external_ip, "");
        assert_eq!(info.load_average, "");
        assert_eq!(info.memory_usage, "");
        assert_eq!(info.disk_usage, "");
        assert!(info.top_processes.is_empty());
    }
}
