//! SSH actor implementation

use super::{DockerContainer, SshCommand, SshEvent, SshHostInfo};
use crate::viewmodel::{ViewModelEvent, runtime};
use smol::channel::{Receiver, Sender};

pub struct SshActor {
    command_rx: Receiver<SshCommand>,
    event_tx: Sender<ViewModelEvent>,
}

impl SshActor {
    pub fn new(command_rx: Receiver<SshCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self {
            command_rx,
            event_tx,
        }
    }

    pub async fn run(mut self) {
        log::info!("SshActor started");

        loop {
            match self.command_rx.recv().await {
                Ok(cmd) => {
                    if let Err(e) = self.handle_command(cmd).await {
                        log::error!("SshActor command failed: {}", e);
                    }
                }
                Err(_) => {
                    log::info!("SshActor: channel closed, shutting down");
                    break;
                }
            }
        }
    }

    async fn handle_command(&mut self, cmd: SshCommand) -> anyhow::Result<()> {
        let operation = format!("{:?}", cmd);
        eprintln!("🔍 SSH Actor: Received command: {}", operation);

        let result = match cmd {
            SshCommand::AddHost {
                name,
                host,
                port,
                user,
                ssh_key_path,
            } => self.add_host(name, host, port, user, ssh_key_path).await,
            SshCommand::DeleteHost { name } => self.delete_host(name).await,
            SshCommand::ListHosts => self.list_hosts().await,
            SshCommand::TestConnection { name } => {
                eprintln!("🔍 SSH Actor: Handling TestConnection for '{}'", name);
                self.test_connection(name).await
            }
            SshCommand::InitHost { name } => self.init_host(name).await,
            SshCommand::DockerPull { host_name, image } => self.docker_pull(host_name, image).await,
            SshCommand::DockerRun {
                host_name,
                image,
                container_name,
                ports,
                env,
            } => {
                self.docker_run(host_name, image, container_name, ports, env)
                    .await
            }
            SshCommand::DockerStop {
                host_name,
                container_name,
            } => self.docker_stop(host_name, container_name).await,
            SshCommand::DockerList { host_name } => self.docker_list(host_name).await,
            SshCommand::PortOpen {
                host_name,
                port,
                protocol,
            } => self.port_open(host_name, port, protocol).await,
            SshCommand::PortClose {
                host_name,
                port,
                protocol,
            } => self.port_close(host_name, port, protocol).await,
            SshCommand::PortList { host_name } => self.port_list(host_name).await,
            SshCommand::DeployDureWss {
                host_name,
                domain,
                acme_email,
            } => self.deploy_dure_wss(host_name, domain, acme_email).await,
            SshCommand::GetLinuxStatus { name } => self.get_linux_status(name).await,
            SshCommand::InstallDocker { name } => self.install_docker(name).await,
            SshCommand::GetDockerStatus { name } => self.get_docker_status(name).await,
            SshCommand::UninstallDocker { name } => self.uninstall_docker(name).await,
            SshCommand::InstallAnsible { name } => self.install_ansible(name).await,
            SshCommand::GetAnsibleStatus { name } => self.get_ansible_status(name).await,
            SshCommand::UninstallAnsible { name } => self.uninstall_ansible(name).await,
            SshCommand::InstallDureWss { name } => self.install_dure_wss(name).await,
            SshCommand::GetDureWssStatus { name } => self.get_dure_wss_status(name).await,
            SshCommand::UninstallDureWss { name } => self.uninstall_dure_wss(name).await,
        };

        if let Err(e) = result {
            self.send_error(&operation, e).await;
        }

        Ok(())
    }

    async fn add_host(
        &mut self,
        name: String,
        host: String,
        port: u16,
        _user: String,
        ssh_key_path: String,
    ) -> anyhow::Result<()> {
        self.send_progress("add_host", 0.5, "Adding SSH host...")
            .await;

        runtime::unblock({
            move || -> anyhow::Result<()> {
                let config_path = Self::get_config_path()?;
                let mut app_config = crate::config::AppConfig::load_or_default(&config_path);

                // Check if host already exists
                if app_config.ssh_hosts.iter().any(|h| h.host == host) {
                    anyhow::bail!("SSH host '{}' already exists", host);
                }

                // Create new SSH host config
                let ssh_host = crate::config::SshHostConfig {
                    host: host.clone(),
                    password: None,
                    private_key_path: if ssh_key_path.is_empty() {
                        None
                    } else {
                        Some(ssh_key_path)
                    },
                    keyring_domain: None,
                    port,
                    initialized: false,
                    last_status: None,
                    platform_name: None,
                    docker_containers: Vec::new(),
                    ansible_roles: Vec::new(),
                    dure_wss_config: None,
                };

                app_config.ssh_hosts.push(ssh_host);
                app_config.save(&config_path)?;
                Ok(())
            }
        })
        .await?;

        self.send_event(SshEvent::HostAdded { name }).await;
        Ok(())
    }

    async fn delete_host(&mut self, name: String) -> anyhow::Result<()> {
        self.send_progress("delete_host", 0.5, "Deleting SSH host...")
            .await;

        runtime::unblock({
            let name_clone = name.clone();
            move || -> anyhow::Result<()> {
                let config_path = Self::get_config_path()?;
                let mut app_config = crate::config::AppConfig::load_or_default(&config_path);

                let initial_len = app_config.ssh_hosts.len();
                app_config.ssh_hosts.retain(|h| h.host != name_clone);

                if app_config.ssh_hosts.len() == initial_len {
                    anyhow::bail!("SSH host '{}' not found", name_clone);
                }

                app_config.save(&config_path)?;
                Ok(())
            }
        })
        .await?;

        self.send_event(SshEvent::HostDeleted { name }).await;
        Ok(())
    }

    async fn list_hosts(&mut self) -> anyhow::Result<()> {
        self.send_progress("list_hosts", 0.5, "Loading SSH hosts...")
            .await;

        let host_infos = runtime::unblock(|| -> anyhow::Result<Vec<SshHostInfo>> {
            let config_path = Self::get_config_path()?;
            let app_config = crate::config::AppConfig::load_or_default(&config_path);

            let hosts = app_config
                .ssh_hosts
                .into_iter()
                .map(|h| SshHostInfo {
                    name: h.host.clone(),
                    host: h.host,
                    port: h.port,
                    user: String::new(), // Not stored separately
                })
                .collect();

            Ok(hosts)
        })
        .await?;

        self.send_event(SshEvent::HostsListed { hosts: host_infos })
            .await;
        Ok(())
    }

    /// Helper to get config file path
    #[cfg(not(target_arch = "wasm32"))]
    fn get_config_path() -> anyhow::Result<std::path::PathBuf> {
        let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
            .ok_or_else(|| anyhow::anyhow!("Failed to get project directories"))?;
        Ok(proj_dirs.config_dir().join("config.yml"))
    }

    async fn test_connection(&mut self, name: String) -> anyhow::Result<()> {
        eprintln!("🔍 SSH Actor: test_connection called for '{}'", name);
        self.send_progress("test_connection", 0.5, "Testing SSH connection...")
            .await;

        let start = std::time::Instant::now();

        // Load host config first (blocking operation)
        eprintln!("🔍 SSH Actor: Loading host config...");
        let host_config = runtime::unblock({
            let name = name.clone();
            move || -> anyhow::Result<crate::config::SshHostConfig> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .into_iter()
                    .find(|h| h.host == name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", name))?;

                eprintln!("🔍 SSH Actor: Found host config for '{}'", host_config.host);
                Ok(host_config)
            }
        })
        .await?;

        // Test connection (async operation - russh uses tokio internally)
        eprintln!(
            "🔍 SSH Actor: Starting SSH connection test to {}:{}...",
            host_config.host, host_config.port
        );
        let result =
            async_compat::Compat::new(crate::calc::ssh::test_connection(&host_config)).await;

        let latency_ms = start.elapsed().as_millis() as u64;
        eprintln!(
            "🔍 SSH Actor: Connection test completed in {}ms",
            latency_ms
        );

        match result {
            Ok(conn_result) => {
                eprintln!(
                    "✓ SSH Actor: Connection test succeeded: {}",
                    conn_result.success
                );
                self.send_event(SshEvent::ConnectionTested {
                    name,
                    success: conn_result.success,
                    latency_ms: Some(latency_ms),
                })
                .await;
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ SSH Actor: Connection test failed: {}", e);
                self.send_event(SshEvent::ConnectionTested {
                    name,
                    success: false,
                    latency_ms: None,
                })
                .await;
                Err(e)
            }
        }
    }

    async fn init_host(&mut self, name: String) -> anyhow::Result<()> {
        eprintln!("🔍 SSH Actor: init_host called for '{}'", name);
        self.send_progress("init_host", 0.1, "Loading host configuration...")
            .await;

        // Load host config
        let host_config = runtime::unblock({
            let name = name.clone();
            move || -> anyhow::Result<crate::config::SshHostConfig> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .into_iter()
                    .find(|h| h.host == name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", name))?;

                Ok(host_config)
            }
        })
        .await?;

        self.send_progress("init_host", 0.3, "Initializing SSH host...")
            .await;

        // Initialize host (async operation - russh uses tokio internally)
        let result =
            async_compat::Compat::new(crate::calc::ssh::initialize_host(&host_config)).await;

        match result {
            Ok(_) => {
                self.send_progress("init_host", 1.0, "Host initialized")
                    .await;
                self.send_event(SshEvent::HostInitialized {
                    name,
                    success: true,
                })
                .await;
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ SSH Actor: Host initialization failed: {}", e);
                self.send_event(SshEvent::HostInitialized {
                    name,
                    success: false,
                })
                .await;
                Err(e)
            }
        }
    }

    async fn docker_pull(&mut self, host_name: String, image: String) -> anyhow::Result<()> {
        self.send_progress(
            "docker_pull",
            0.3,
            &format!("Pulling Docker image {}...", image),
        )
        .await;

        // Execute docker pull via SSH
        runtime::unblock({
            let host_name = host_name.clone();
            let image = image.clone();
            move || -> anyhow::Result<()> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .iter()
                    .find(|h| h.host == host_name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", host_name))?;

                // Execute docker pull via SSH using calc::ssh
                crate::calc::ssh::docker_pull(host_config, &image)?;
                Ok(())
            }
        })
        .await?;

        self.send_progress("docker_pull", 1.0, "Image pulled").await;

        self.send_event(SshEvent::DockerImagePulled { host_name, image })
            .await;

        Ok(())
    }

    async fn docker_run(
        &mut self,
        host_name: String,
        image: String,
        container_name: String,
        ports: Vec<(u16, u16)>,
        env: Vec<(String, String)>,
    ) -> anyhow::Result<()> {
        self.send_progress(
            "docker_run",
            0.3,
            &format!("Starting container {}...", container_name),
        )
        .await;

        // Execute docker run via SSH
        runtime::unblock({
            let host_name = host_name.clone();
            let container_name = container_name.clone();
            move || -> anyhow::Result<()> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .iter()
                    .find(|h| h.host == host_name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", host_name))?;

                // Execute docker run via SSH
                crate::calc::ssh::docker_run(host_config, &image, &container_name, &ports, &env)?;
                Ok(())
            }
        })
        .await?;

        self.send_progress("docker_run", 1.0, "Container started")
            .await;

        self.send_event(SshEvent::DockerContainerStarted {
            host_name,
            container_name,
        })
        .await;

        Ok(())
    }

    async fn docker_stop(
        &mut self,
        host_name: String,
        container_name: String,
    ) -> anyhow::Result<()> {
        self.send_progress(
            "docker_stop",
            0.3,
            &format!("Stopping container {}...", container_name),
        )
        .await;

        // Execute docker stop via SSH
        runtime::unblock({
            let host_name = host_name.clone();
            let container_name = container_name.clone();
            move || -> anyhow::Result<()> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .iter()
                    .find(|h| h.host == host_name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", host_name))?;

                // Execute docker stop via SSH
                crate::calc::ssh::docker_stop(host_config, &container_name)?;
                Ok(())
            }
        })
        .await?;

        self.send_progress("docker_stop", 1.0, "Container stopped")
            .await;

        self.send_event(SshEvent::DockerContainerStopped {
            host_name,
            container_name,
        })
        .await;

        Ok(())
    }

    async fn docker_list(&mut self, _host_name: String) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(
            "Docker management not yet implemented in ViewModel"
        ))
    }

    async fn port_open(
        &mut self,
        host_name: String,
        port: u16,
        protocol: String,
    ) -> anyhow::Result<()> {
        self.send_progress("port_open", 0.3, &format!("Opening port {}...", port))
            .await;

        // Execute port open via SSH (nftables)
        runtime::unblock({
            let host_name = host_name.clone();
            let protocol = protocol.clone();
            move || -> anyhow::Result<()> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .iter()
                    .find(|h| h.host == host_name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", host_name))?;

                // Execute nftables command via SSH
                crate::calc::ssh::port_open(host_config, port, &protocol)?;
                Ok(())
            }
        })
        .await?;

        self.send_progress("port_open", 1.0, "Port opened").await;

        self.send_event(SshEvent::PortOpened {
            host_name,
            port,
            protocol,
        })
        .await;

        Ok(())
    }

    async fn port_close(
        &mut self,
        host_name: String,
        port: u16,
        protocol: String,
    ) -> anyhow::Result<()> {
        self.send_progress("port_close", 0.3, &format!("Closing port {}...", port))
            .await;

        // Execute port close via SSH (nftables)
        runtime::unblock({
            let host_name = host_name.clone();
            let protocol = protocol.clone();
            move || -> anyhow::Result<()> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .iter()
                    .find(|h| h.host == host_name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", host_name))?;

                // Execute nftables command via SSH
                crate::calc::ssh::port_close(host_config, port, &protocol)?;
                Ok(())
            }
        })
        .await?;

        self.send_progress("port_close", 1.0, "Port closed").await;

        self.send_event(SshEvent::PortClosed {
            host_name,
            port,
            protocol,
        })
        .await;

        Ok(())
    }

    async fn port_list(&mut self, _host_name: String) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(
            "Port management not yet implemented in ViewModel"
        ))
    }

    async fn deploy_dure_wss(
        &mut self,
        _host_name: String,
        _domain: String,
        _acme_email: String,
    ) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(
            "Dure WSS deployment not yet implemented in ViewModel"
        ))
    }

    async fn get_linux_status(&mut self, name: String) -> anyhow::Result<()> {
        eprintln!("🔍 SSH Actor: get_linux_status called for '{}'", name);
        self.send_progress("get_linux_status", 0.1, "Loading host configuration...")
            .await;

        // Load host config
        let host_config = runtime::unblock({
            let name = name.clone();
            move || -> anyhow::Result<crate::config::SshHostConfig> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .into_iter()
                    .find(|h| h.host == name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", name))?;

                Ok(host_config)
            }
        })
        .await?;

        self.send_progress("get_linux_status", 0.3, "Retrieving Linux status...")
            .await;

        // Get status (async operation - russh uses tokio internally)
        let result =
            async_compat::Compat::new(crate::calc::ssh::get_linux_status(&host_config)).await;

        match result {
            Ok(status) => {
                self.send_progress("get_linux_status", 1.0, "Status retrieved")
                    .await;
                self.send_event(SshEvent::LinuxStatusRetrieved {
                    name,
                    uptime: status.uptime,
                    external_ip: status.external_ip,
                    load_average: status.load_average,
                    memory_usage: status.memory_usage,
                    disk_usage: status.disk_usage,
                    top_processes: status.top_processes,
                })
                .await;
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ SSH Actor: Linux status retrieval failed: {}", e);
                self.send_event(SshEvent::ServiceError {
                    name,
                    service: "linux".to_string(),
                    operation: "get_status".to_string(),
                    error: format!("{:#}", e),
                })
                .await;
                Err(e)
            }
        }
    }

    async fn install_docker(&mut self, name: String) -> anyhow::Result<()> {
        eprintln!("🔍 SSH Actor: install_docker called for '{}'", name);
        self.send_progress("install_docker", 0.1, "Loading host configuration...")
            .await;

        // Load host config
        let host_config = runtime::unblock({
            let name = name.clone();
            move || -> anyhow::Result<crate::config::SshHostConfig> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .into_iter()
                    .find(|h| h.host == name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", name))?;

                Ok(host_config)
            }
        })
        .await?;

        self.send_progress("install_docker", 0.3, "Installing Docker...")
            .await;

        // Install Docker (async operation)
        let result =
            async_compat::Compat::new(crate::calc::ssh::install_docker(&host_config)).await;

        match result {
            Ok(_) => {
                self.send_progress("install_docker", 1.0, "Docker installed")
                    .await;
                self.send_event(SshEvent::DockerInstalled { name }).await;
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ SSH Actor: Docker installation failed: {}", e);
                self.send_event(SshEvent::ServiceError {
                    name,
                    service: "docker".to_string(),
                    operation: "install".to_string(),
                    error: format!("{:#}", e),
                })
                .await;
                Err(e)
            }
        }
    }

    async fn get_docker_status(&mut self, name: String) -> anyhow::Result<()> {
        eprintln!("🔍 SSH Actor: get_docker_status called for '{}'", name);
        self.send_progress("get_docker_status", 0.1, "Loading host configuration...")
            .await;

        // Load host config
        let host_config = runtime::unblock({
            let name = name.clone();
            move || -> anyhow::Result<crate::config::SshHostConfig> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .into_iter()
                    .find(|h| h.host == name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", name))?;

                Ok(host_config)
            }
        })
        .await?;

        self.send_progress("get_docker_status", 0.3, "Checking Docker status...")
            .await;

        // Check Docker status (async operations)
        let installed =
            async_compat::Compat::new(crate::calc::ssh::check_docker_installed(&host_config))
                .await?;

        let running = if installed {
            async_compat::Compat::new(crate::calc::ssh::check_docker_running(&host_config)).await?
        } else {
            false
        };

        self.send_progress("get_docker_status", 1.0, "Status retrieved")
            .await;
        self.send_event(SshEvent::DockerStatusRetrieved {
            name,
            installed,
            running,
        })
        .await;
        Ok(())
    }

    async fn uninstall_docker(&mut self, name: String) -> anyhow::Result<()> {
        eprintln!("🔍 SSH Actor: uninstall_docker called for '{}'", name);
        self.send_progress("uninstall_docker", 0.1, "Loading host configuration...")
            .await;

        // Load host config
        let host_config = runtime::unblock({
            let name = name.clone();
            move || -> anyhow::Result<crate::config::SshHostConfig> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .into_iter()
                    .find(|h| h.host == name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", name))?;

                Ok(host_config)
            }
        })
        .await?;

        self.send_progress("uninstall_docker", 0.3, "Uninstalling Docker...")
            .await;

        // Uninstall Docker (async operation)
        let result =
            async_compat::Compat::new(crate::calc::ssh::uninstall_docker(&host_config)).await;

        match result {
            Ok(_) => {
                self.send_progress("uninstall_docker", 1.0, "Docker uninstalled")
                    .await;
                self.send_event(SshEvent::DockerUninstalled { name }).await;
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ SSH Actor: Docker uninstallation failed: {}", e);
                self.send_event(SshEvent::ServiceError {
                    name,
                    service: "docker".to_string(),
                    operation: "uninstall".to_string(),
                    error: format!("{:#}", e),
                })
                .await;
                Err(e)
            }
        }
    }

    async fn install_ansible(&mut self, name: String) -> anyhow::Result<()> {
        eprintln!("🔍 SSH Actor: install_ansible called for '{}'", name);
        self.send_progress("install_ansible", 0.1, "Loading host configuration...")
            .await;

        // Load host config
        let host_config = runtime::unblock({
            let name = name.clone();
            move || -> anyhow::Result<crate::config::SshHostConfig> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .into_iter()
                    .find(|h| h.host == name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", name))?;

                Ok(host_config)
            }
        })
        .await?;

        self.send_progress("install_ansible", 0.3, "Installing Ansible...")
            .await;

        // Install Ansible (async operation)
        let result =
            async_compat::Compat::new(crate::calc::ssh::install_ansible(&host_config)).await;

        match result {
            Ok(_) => {
                self.send_progress("install_ansible", 1.0, "Ansible installed")
                    .await;
                self.send_event(SshEvent::AnsibleInstalled { name }).await;
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ SSH Actor: Ansible installation failed: {}", e);
                self.send_event(SshEvent::ServiceError {
                    name,
                    service: "ansible".to_string(),
                    operation: "install".to_string(),
                    error: format!("{:#}", e),
                })
                .await;
                Err(e)
            }
        }
    }

    async fn get_ansible_status(&mut self, name: String) -> anyhow::Result<()> {
        eprintln!("🔍 SSH Actor: get_ansible_status called for '{}'", name);
        self.send_progress("get_ansible_status", 0.1, "Loading host configuration...")
            .await;

        // Load host config
        let host_config = runtime::unblock({
            let name = name.clone();
            move || -> anyhow::Result<crate::config::SshHostConfig> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .into_iter()
                    .find(|h| h.host == name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", name))?;

                Ok(host_config)
            }
        })
        .await?;

        self.send_progress("get_ansible_status", 0.3, "Checking Ansible status...")
            .await;

        // Check Ansible status (async operation)
        let installed =
            async_compat::Compat::new(crate::calc::ssh::check_ansible_installed(&host_config))
                .await?;

        self.send_progress("get_ansible_status", 1.0, "Status retrieved")
            .await;
        self.send_event(SshEvent::AnsibleStatusRetrieved { name, installed })
            .await;
        Ok(())
    }

    async fn uninstall_ansible(&mut self, name: String) -> anyhow::Result<()> {
        eprintln!("🔍 SSH Actor: uninstall_ansible called for '{}'", name);
        self.send_progress("uninstall_ansible", 0.1, "Loading host configuration...")
            .await;

        // Load host config
        let host_config = runtime::unblock({
            let name = name.clone();
            move || -> anyhow::Result<crate::config::SshHostConfig> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .into_iter()
                    .find(|h| h.host == name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", name))?;

                Ok(host_config)
            }
        })
        .await?;

        self.send_progress("uninstall_ansible", 0.3, "Uninstalling Ansible...")
            .await;

        // Uninstall Ansible (async operation)
        let result =
            async_compat::Compat::new(crate::calc::ssh::uninstall_ansible(&host_config)).await;

        match result {
            Ok(_) => {
                self.send_progress("uninstall_ansible", 1.0, "Ansible uninstalled")
                    .await;
                self.send_event(SshEvent::AnsibleUninstalled { name }).await;
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ SSH Actor: Ansible uninstallation failed: {}", e);
                self.send_event(SshEvent::ServiceError {
                    name,
                    service: "ansible".to_string(),
                    operation: "uninstall".to_string(),
                    error: format!("{:#}", e),
                })
                .await;
                Err(e)
            }
        }
    }

    async fn install_dure_wss(&mut self, name: String) -> anyhow::Result<()> {
        eprintln!("🔍 SSH Actor: install_dure_wss called for '{}'", name);
        self.send_progress("install_dure_wss", 0.1, "Loading host configuration...")
            .await;

        // Load host config
        let host_config = runtime::unblock({
            let name = name.clone();
            move || -> anyhow::Result<crate::config::SshHostConfig> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .into_iter()
                    .find(|h| h.host == name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", name))?;

                Ok(host_config)
            }
        })
        .await?;

        self.send_progress("install_dure_wss", 0.3, "Installing Dure-WSS...")
            .await;

        // Install Dure-WSS (async operation)
        let result =
            async_compat::Compat::new(crate::calc::ssh::install_dure_wss(&host_config)).await;

        match result {
            Ok(_) => {
                self.send_progress("install_dure_wss", 1.0, "Dure-WSS installed")
                    .await;
                self.send_event(SshEvent::DureWssInstalled { name }).await;
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ SSH Actor: Dure-WSS installation failed: {}", e);
                self.send_event(SshEvent::ServiceError {
                    name,
                    service: "dure-wss".to_string(),
                    operation: "install".to_string(),
                    error: format!("{:#}", e),
                })
                .await;
                Err(e)
            }
        }
    }

    async fn get_dure_wss_status(&mut self, name: String) -> anyhow::Result<()> {
        eprintln!("🔍 SSH Actor: get_dure_wss_status called for '{}'", name);
        self.send_progress("get_dure_wss_status", 0.1, "Loading host configuration...")
            .await;

        // Load host config
        let host_config = runtime::unblock({
            let name = name.clone();
            move || -> anyhow::Result<crate::config::SshHostConfig> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .into_iter()
                    .find(|h| h.host == name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", name))?;

                Ok(host_config)
            }
        })
        .await?;

        self.send_progress("get_dure_wss_status", 0.3, "Checking Dure-WSS status...")
            .await;

        // Check Dure-WSS status (async operation)
        let installed =
            async_compat::Compat::new(crate::calc::ssh::check_dure_wss_installed(&host_config))
                .await?;

        self.send_progress("get_dure_wss_status", 1.0, "Status retrieved")
            .await;
        self.send_event(SshEvent::DureWssStatusRetrieved { name, installed })
            .await;
        Ok(())
    }

    async fn uninstall_dure_wss(&mut self, name: String) -> anyhow::Result<()> {
        eprintln!("🔍 SSH Actor: uninstall_dure_wss called for '{}'", name);
        self.send_progress("uninstall_dure_wss", 0.1, "Loading host configuration...")
            .await;

        // Load host config
        let host_config = runtime::unblock({
            let name = name.clone();
            move || -> anyhow::Result<crate::config::SshHostConfig> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config
                    .ssh_hosts
                    .into_iter()
                    .find(|h| h.host == name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", name))?;

                Ok(host_config)
            }
        })
        .await?;

        self.send_progress("uninstall_dure_wss", 0.3, "Uninstalling Dure-WSS...")
            .await;

        // Uninstall Dure-WSS (async operation)
        let result =
            async_compat::Compat::new(crate::calc::ssh::uninstall_dure_wss(&host_config)).await;

        match result {
            Ok(_) => {
                self.send_progress("uninstall_dure_wss", 1.0, "Dure-WSS uninstalled")
                    .await;
                self.send_event(SshEvent::DureWssUninstalled { name }).await;
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ SSH Actor: Dure-WSS uninstallation failed: {}", e);
                self.send_event(SshEvent::ServiceError {
                    name,
                    service: "dure-wss".to_string(),
                    operation: "uninstall".to_string(),
                    error: format!("{:#}", e),
                })
                .await;
                Err(e)
            }
        }
    }

    async fn send_progress(&self, operation: &str, progress: f32, status: &str) {
        let _ = self
            .event_tx
            .send(ViewModelEvent::Ssh(SshEvent::Progress {
                operation: operation.to_string(),
                progress,
                status: status.to_string(),
            }))
            .await;
    }

    async fn send_event(&self, event: SshEvent) {
        eprintln!("🔍 SSH Actor: Sending event: {:?}", event);
        match self.event_tx.send(ViewModelEvent::Ssh(event.clone())).await {
            Ok(_) => eprintln!("✓ SSH Actor: Event sent successfully"),
            Err(e) => eprintln!("✗ SSH Actor: Failed to send event: {}", e),
        }
    }

    async fn send_error(&self, operation: &str, error: anyhow::Error) {
        let _ = self
            .event_tx
            .send(ViewModelEvent::Ssh(SshEvent::Error {
                operation: operation.to_string(),
                error: format!("{:#}", error),
            }))
            .await;
    }
}
