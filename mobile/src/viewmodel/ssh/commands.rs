//! SSH actor commands

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
    InstallDockerImage {
        host_name: String,
        container_name: String,
        image: String,
        tag: String,
        ports: Vec<(u16, u16)>,
        env: Vec<(String, String)>,
    },
    RemoveDockerContainer {
        host_name: String,
        container_name: String,
    },
    /// Remove multiple Docker containers (batch operation)
    RemoveDockerContainers {
        host_name: String,
        container_names: Vec<String>,
    },
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
    ValidateAnsibleRole {
        role: String,
    },
    InstallAnsibleRole {
        host_name: String,
        instance_name: String,
        galaxy_name: String,
        variables: Vec<(String, String)>,
        ports: Vec<u16>,
    },
    RemoveAnsibleRole {
        host_name: String,
        instance_name: String,
    },
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

    InstallDocker {
        name: String,
    },
    GetDockerStatus {
        name: String,
    },
    UninstallDocker {
        name: String,
    },

    InstallAnsible {
        name: String,
    },
    GetAnsibleStatus {
        name: String,
    },
    UninstallAnsible {
        name: String,
    },

    /// Check if SSH host is reachable (TCP port check with timeout)
    CheckHostHealth {
        name: String,
        timeout_secs: u8,
    },
}
