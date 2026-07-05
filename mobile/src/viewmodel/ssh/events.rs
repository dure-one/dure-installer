//! SSH actor events

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
    HostAdded { name: String },
    HostDeleted { name: String },
    HostsListed { hosts: Vec<SshHostInfo> },
    ConnectionTested { name: String, success: bool, latency_ms: Option<u64> },

    // Docker Events
    DockerImagePulled { host_name: String, image: String },
    DockerContainerStarted { host_name: String, container_name: String },
    DockerContainerStopped { host_name: String, container_name: String },
    DockerContainersListed { host_name: String, containers: Vec<DockerContainer> },

    // Port Events
    PortOpened { host_name: String, port: u16, protocol: String },
    PortClosed { host_name: String, port: u16, protocol: String },
    PortsListed { host_name: String, open_ports: Vec<(u16, String)> },

    // Deployment Events
    DureWssDeployed { host_name: String, domain: String, service_status: String },

    // Progress & Errors
    Progress { operation: String, progress: f32, status: String },
    Error { operation: String, error: String },
}
