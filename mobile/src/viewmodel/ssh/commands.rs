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
    DeleteHost { name: String },
    ListHosts,
    TestConnection { name: String },
    InitHost { name: String },

    // Docker Operations
    DockerPull { host_name: String, image: String },
    DockerRun {
        host_name: String,
        image: String,
        container_name: String,
        ports: Vec<(u16, u16)>,
        env: Vec<(String, String)>,
    },
    DockerStop { host_name: String, container_name: String },
    DockerList { host_name: String },

    // Port Management
    PortOpen { host_name: String, port: u16, protocol: String },
    PortClose { host_name: String, port: u16, protocol: String },
    PortList { host_name: String },

    // Dure WSS Deployment
    DeployDureWss {
        host_name: String,
        domain: String,
        acme_email: String,
    },
}
