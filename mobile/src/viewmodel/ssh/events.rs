//! SSH actor events

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use crate::calc::ansible::AnsibleRoleMetadata;
use crate::config::{DockerContainerConfig, AnsibleRoleConfig};

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
    DockerContainersListedNew {
        host_name: String,
        containers: Vec<DockerContainerConfig>,
    },

    // Ansible Lifecycle Events
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
}
