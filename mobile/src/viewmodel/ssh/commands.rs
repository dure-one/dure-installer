//! SSH actor commands

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
    /// Check if host packages are installed
    CheckHost {
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },
    /// Install host packages
    InstallHost {
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
        version: String,
        env_fields: Vec<crate::calc::env_parser::EnvField>,
        image_tag: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },
    /// Remove Dure
    RemoveDure {
        host: String,
        delete_data: bool,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },

    // Drawer commands
    /// Drawer-specific command passthrough
    Drawer(DrawerCommand),
}
