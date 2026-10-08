//! SSH actor commands

use std::collections::HashMap;

/// Drawer commands for SSH operations
#[derive(Debug, Clone)]
pub enum DrawerCommand {
    #[doc(hidden)]
    _Placeholder,
}

#[derive(Debug, Clone)]
pub enum SshCommand {
    // Host Management
    AddHost {
        name: String,
        host: String,
        port: u16,
        user: String,
        ssh_key_path: String,
    },
    DeleteHost {
        name: String,
    },
    ListHosts,
    TestConnection {
        name: String,
    },
    InitHost {
        name: String,
    },

    // Docker Operations
    DockerPull {
        host_name: String,
        image: String,
    },
    DockerRun {
        host_name: String,
        image: String,
        container_name: String,
        ports: Vec<(u16, u16)>,
        env: Vec<(String, String)>,
    },
    DockerStop {
        host_name: String,
        container_name: String,
    },
    DockerList {
        host_name: String,
    },

    // Docker Lifecycle
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    InstallDockerImage {
        host_name: String,
        container_name: String,
        image: String,
        tag: String,
        ports: Vec<(u16, u16)>,
        env: Vec<(String, String)>,
    },
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    RemoveDockerContainer {
        host_name: String,
        container_name: String,
    },
    /// Remove multiple Docker containers (batch operation)
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    RemoveDockerContainers {
        host_name: String,
        container_names: Vec<String>,
    },
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    ListDockerContainers {
        host_name: String,
    },
    /// Inspect Docker image by pulling and analyzing history
    InspectDockerImage {
        host_name: String,
        image: String,
        tag: String,
    },

    // Ansible Lifecycle
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    ValidateAnsibleRole {
        role: String,
    },
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    InstallAnsibleRole {
        host_name: String,
        instance_name: String,
        galaxy_name: String,
        variables: Vec<(String, String)>,
        ports: Vec<u16>,
    },
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    RemoveAnsibleRole {
        host_name: String,
        instance_name: String,
    },
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    ListAnsibleRoles {
        host_name: String,
    },

    // Dure-WSS Lifecycle
    InstallDureWssService {
        host_name: String,
        domain: String,
        email: String,
        channel: String,
        variant: String,
    },
    StartDureWss {
        host_name: String,
    },
    StopDureWss {
        host_name: String,
    },
    RestartDureWss {
        host_name: String,
    },
    UninstallDureWss {
        host_name: String,
    },

    // Port Management
    PortOpen {
        host_name: String,
        port: u16,
        protocol: String,
    },
    PortClose {
        host_name: String,
        port: u16,
        protocol: String,
    },
    PortList {
        host_name: String,
    },

    // Dure WSS Deployment
    DeployDureWss {
        host_name: String,
        domain: String,
        acme_email: String,
    },

    // Service Management
    GetLinuxStatus {
        name: String,
    },

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    InstallDocker {
        name: String,
    },
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    GetDockerStatus {
        name: String,
    },
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    UninstallDocker {
        name: String,
    },

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    InstallAnsible {
        name: String,
    },
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    GetAnsibleStatus {
        name: String,
    },
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    UninstallAnsible {
        name: String,
    },

    /// Check if SSH host is reachable (TCP port check with timeout)
    CheckHostHealth {
        name: String,
        timeout_secs: u8,
    },

    // Operation commands
    /// Refresh host status (SSH, Base, Docker, Dure)
    Refresh {
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },
    /// Check if SSH connection is available
    SshCheck {
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },
    /// Check if base packages are installed
    CheckBase {
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },
    /// Install base packages
    InstallBase {
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },
    /// Check if Docker is installed
    CheckDocker {
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },
    /// Install Docker daemon
    InstallDockerDaemon {
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },
    /// Remove Docker daemon
    RemoveDocker {
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },
    /// Check if Dure is installed
    CheckDure {
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },
    /// Install Dure with environment configuration
    InstallDure {
        host: String,
        env_config: HashMap<String, String>,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },
    /// Remove Dure
    RemoveDure {
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },

    // Drawer commands
    /// Drawer-specific command passthrough
    Drawer(DrawerCommand),
}
