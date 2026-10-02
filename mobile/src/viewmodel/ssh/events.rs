//! SSH actor events

use crate::{dure_info, dure_debug, dure_warn, dure_error};
#[cfg(not(target_arch = "wasm32"))]
use crate::calc::ansible::AnsibleRoleMetadata;
#[cfg(not(target_arch = "wasm32"))]
use crate::config::{DockerContainerConfig, AnsibleRoleConfig};

/// Drawer events for SSH operations
#[derive(Debug, Clone)]
pub enum DrawerEvent {
    #[doc(hidden)]
    _Placeholder,
}

#[derive(Debug, Clone)]
pub struct SshHostInfo {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub user: String,
}

#[derive(Debug, Clone)]
pub struct DockerContainer {
    pub name: String,
    pub image: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub enum SshEvent {
    // Host Events
    HostAdded {
        name: String,
    },
    HostDeleted {
        name: String,
    },
    HostsListed {
        hosts: Vec<SshHostInfo>,
    },
    ConnectionTested {
        name: String,
        success: bool,
        latency_ms: Option<u64>,
    },
    HostInitialized {
        name: String,
        success: bool,
    },

    // Docker Events
    DockerImagePulled {
        host_name: String,
        image: String,
    },
    DockerContainerStarted {
        host_name: String,
        container_name: String,
    },
    DockerContainerStopped {
        host_name: String,
        container_name: String,
    },
    DockerContainersListed {
        host_name: String,
        containers: Vec<DockerContainer>,
    },

    // Port Events
    PortOpened {
        host_name: String,
        port: u16,
        protocol: String,
    },
    PortClosed {
        host_name: String,
        port: u16,
        protocol: String,
    },
    PortsListed {
        host_name: String,
        open_ports: Vec<(u16, String)>,
    },

    // Docker Lifecycle Events
    DockerDaemonInstallRequired {
        host_name: String,
    },
    DockerDaemonInstalled {
        host_name: String,
    },
    DockerImageInstalled {
        host_name: String,
        container_name: String,
    },
    DockerContainerRemoved {
        host_name: String,
        container_name: String,
    },
    #[cfg(not(target_arch = "wasm32"))]
    DockerContainersListedNew {
        host_name: String,
        containers: Vec<DockerContainerConfig>,
    },

    // Ansible Lifecycle Events
    #[cfg(not(target_arch = "wasm32"))]
    AnsibleRoleValidated {
        role: String,
        metadata: AnsibleRoleMetadata,
    },
    AnsibleDaemonInstallRequired {
        host_name: String,
    },
    AnsibleDaemonInstalled {
        host_name: String,
    },
    AnsibleRoleInstalled {
        host_name: String,
        instance_name: String,
    },
    AnsibleRoleRemoved {
        host_name: String,
        instance_name: String,
    },
    AnsibleRolesListed {
        host_name: String,
        roles: Vec<String>,
    },

    // Dure-WSS Lifecycle Events
    DureWssServiceInstalled {
        host_name: String,
        domain: String,
    },
    DureWssStarted {
        host_name: String,
    },
    DureWssStopped {
        host_name: String,
    },
    DureWssUninstalled {
        host_name: String,
    },

    // Deployment Events
    DureWssDeployed {
        host_name: String,
        domain: String,
        service_status: String,
    },

    // Service Management Events
    LinuxStatusRetrieved {
        name: String,
        uptime: String,
        external_ip: String,
        load_average: String,
        memory_usage: String,
        disk_usage: String,
        top_processes: Vec<String>,
    },

    DockerInstalled {
        name: String,
    },
    DockerStatusRetrieved {
        name: String,
        installed: bool,
        running: bool,
    },
    DockerUninstalled {
        name: String,
    },

    AnsibleInstalled {
        name: String,
    },
    AnsibleStatusRetrieved {
        name: String,
        installed: bool,
    },
    AnsibleUninstalled {
        name: String,
    },

    // Legacy Dure-WSS events (temporary - kept for backward compatibility)
    DureWssInstalled {
        name: String,
    },
    DureWssStatusRetrieved {
        name: String,
        installed: bool,
    },

    /// Host health check completed (TCP port check result)
    HostHealthChecked {
        name: String,
        is_alive: bool,
        latency_ms: Option<u64>,
    },

    /// Docker image inspection completed
    DockerImageInspected {
        image: String,
        tag: String,
        exposed_ports: Vec<u16>,
        env_vars: Vec<(String, String)>,
    },

    /// Docker containers removed (batch operation)
    DockerContainersRemoved {
        host_name: String,
        removed: Vec<String>,           // successfully removed
        failed: Vec<(String, String)>,  // (container_name, error_message)
    },

    ServiceError {
        name: String,
        service: String,
        operation: String,
        error: String,
    },

    // Progress & Errors
    Progress {
        operation: String,
        progress: f32,
        status: String,
    },
    Error {
        operation: String,
        error: String,
    },

    // Operation events
    /// Refresh completed with status of all components
    RefreshCompleted {
        host: String,
        ssh_connected: bool,
        base_installed: bool,
        docker_installed: bool,
        dure_installed: bool,
    },

    /// SSH connection check completed
    SshCheckCompleted {
        host: String,
        connected: bool,
        error: Option<String>,
    },

    /// Base packages check completed
    BaseCheckCompleted {
        host: String,
        installed: bool,
        missing_packages: Vec<String>,
    },

    /// Base packages installation completed
    BaseInstallCompleted {
        host: String,
        success: bool,
        error: Option<String>,
    },

    /// Docker check completed
    DockerCheckCompleted {
        host: String,
        installed: bool,
        version: Option<String>,
    },

    /// Docker installation completed
    DockerInstallCompleted {
        host: String,
        success: bool,
        error: Option<String>,
    },

    /// Docker removal completed
    DockerRemoveCompleted {
        host: String,
        success: bool,
        error: Option<String>,
    },

    /// Dure check completed
    DureCheckCompleted {
        host: String,
        installed: bool,
        running: bool,
        services: Vec<String>,
    },

    /// Dure installation completed
    DureInstallCompleted {
        host: String,
        success: bool,
        error: Option<String>,
    },

    /// Dure removal completed
    DureRemoveCompleted {
        host: String,
        success: bool,
        error: Option<String>,
    },

    /// Operation failed with error details
    OperationFailed {
        host: String,
        operation: String,
        error: String,
    },

    /// Drawer event passthrough
    Drawer(DrawerEvent),
}
