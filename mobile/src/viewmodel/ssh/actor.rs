//! SSH actor implementation

use super::{DockerContainer, SshCommand, SshEvent, SshHostInfo};
use crate::viewmodel::{ViewModelEvent, runtime};
use crate::calc::{docker, ansible, dure_wss};
use crate::config::{DockerContainerConfig, AnsibleRoleConfig, DureWssConfig, SshHostConfig};
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

    fn get_config_and_path(&self) -> anyhow::Result<(crate::config::AppConfig, std::path::PathBuf)> {
        let config_path = Self::get_config_path()?;
        let config = crate::config::AppConfig::load_or_default(&config_path);
        Ok((config, config_path))
    }

    fn load_host_config(&self, host_name: &str) -> anyhow::Result<SshHostConfig> {
        let config_path = Self::get_config_path()?;
        let config = crate::config::AppConfig::load_or_default(&config_path);
        config
            .ssh_hosts
            .iter()
            .find(|h| h.host == *host_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Host '{}' not found", host_name))
    }

    fn check_port_conflicts(
        &self,
        host_name: &str,
        new_ports: &[(u16, u16)],
    ) -> Result<(), String> {
        let config_path = Self::get_config_path().map_err(|e| e.to_string())?;
        let config = crate::config::AppConfig::load_or_default(&config_path);
        let host_config = config
            .ssh_hosts
            .iter()
            .find(|h| h.host == *host_name)
            .ok_or_else(|| format!("Host '{}' not found", host_name))?;

        let mut allocated_ports = std::collections::HashSet::new();

        // Collect ports from Docker containers
        for container in &host_config.docker_containers {
            for (host_port, _) in &container.ports {
                allocated_ports.insert(*host_port);
            }
        }

        // Collect ports from Ansible roles
        for role in &host_config.ansible_roles {
            for port in &role.ports {
                allocated_ports.insert(*port);
            }
        }

        // Check new ports against allocated ports
        for (host_port, _) in new_ports {
            if allocated_ports.contains(host_port) {
                return Err(format!(
                    "Port {} already in use on host '{}'",
                    host_port, host_name
                ));
            }
        }

        Ok(())
    }

    fn save_docker_container(
        &self,
        host_name: &str,
        container: DockerContainerConfig,
    ) -> anyhow::Result<()> {
        let config_path = Self::get_config_path()?;
        let mut config = crate::config::AppConfig::load_or_default(&config_path);

        if let Some(host) = config.ssh_hosts.iter_mut().find(|h| h.host == *host_name) {
            host.docker_containers.push(container);
            config.save(&config_path)?;
        }

        Ok(())
    }

    fn save_ansible_role(
        &self,
        host_name: &str,
        role: AnsibleRoleConfig,
    ) -> anyhow::Result<()> {
        let config_path = Self::get_config_path()?;
        let mut config = crate::config::AppConfig::load_or_default(&config_path);

        if let Some(host) = config.ssh_hosts.iter_mut().find(|h| h.host == *host_name) {
            host.ansible_roles.push(role);
            config.save(&config_path)?;
        }

        Ok(())
    }

    fn save_dure_wss_config(
        &self,
        host_name: &str,
        dure_config: DureWssConfig,
    ) -> anyhow::Result<()> {
        let config_path = Self::get_config_path()?;
        let mut config = crate::config::AppConfig::load_or_default(&config_path);

        if let Some(host) = config.ssh_hosts.iter_mut().find(|h| h.host == *host_name) {
            host.dure_wss_config = Some(dure_config);
            config.save(&config_path)?;
        }

        Ok(())
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

            // Docker Lifecycle Commands
            SshCommand::InstallDockerImage {
                host_name,
                container_name,
                image,
                tag,
                ports,
                env,
            } => {
                return self.handle_install_docker_image(host_name, container_name, image, tag, ports, env).await;
            }
            SshCommand::RemoveDockerContainer {
                host_name,
                container_name,
            } => {
                return self.handle_remove_docker_container(host_name, container_name).await;
            }
            SshCommand::ListDockerContainers { host_name } => {
                return self.handle_list_docker_containers(host_name).await;
            }
            SshCommand::InspectDockerImage { host_name, image, tag } => {
                eprintln!("🔍 SSH Actor: inspect_docker_image called for '{}' with {}:{}", host_name, image, tag);

                // Load host config
                let host_config = match self.load_host_config(&host_name) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "InspectDockerImage".to_string(),
                            error: format!("Host not found: {}", e),
                        }).await;
                        return Ok(());
                    }
                };

                let full_image = format!("{}:{}", image, tag);

                // Step 1: Pull the image
                let pull_cmd = format!("docker pull {}", full_image);
                let host_config_clone = host_config.clone();
                match async_compat::Compat::new(crate::calc::ssh::execute_command(&host_config_clone, &pull_cmd)).await {
                    Ok(output) => {
                        eprintln!("🔍 SSH Actor: Image pulled successfully");
                        eprintln!("📋 Pull output: {}", output);
                    }
                    Err(e) => {
                        eprintln!("❌ SSH Actor: Failed to pull image: {}", e);
                        self.send_event(SshEvent::Error {
                            operation: format!("inspect_docker_image({})", full_image),
                            error: format!("Failed to pull image: {}", e),
                        }).await;
                        return Ok(());
                    }
                }

                // Step 2: Get image history
                let history_cmd = format!("docker history {} --no-trunc --format \"{{{{.CreatedBy}}}}\"", full_image);
                eprintln!("🔍 SSH Actor: Running command: {}", history_cmd);
                let history_output = match async_compat::Compat::new(crate::calc::ssh::execute_command(&host_config, &history_cmd)).await {
                    Ok(output) => output,
                    Err(e) => {
                        eprintln!("❌ SSH Actor: Failed to get image history: {}", e);
                        self.send_event(SshEvent::Error {
                            operation: format!("inspect_docker_image({})", full_image),
                            error: format!("Failed to inspect image history: {}", e),
                        }).await;
                        return Ok(());
                    }
                };

                eprintln!("📋 History output ({} bytes, {} lines):", history_output.len(), history_output.lines().count());
                for (i, line) in history_output.lines().take(10).enumerate() {
                    eprintln!("  Line {}: {}", i + 1, line);
                }
                if history_output.lines().count() > 10 {
                    eprintln!("  ... ({} more lines)", history_output.lines().count() - 10);
                }

                // Step 3: Parse history output
                let (exposed_ports, env_vars) = parse_docker_history(&history_output);

                eprintln!("🔍 SSH Actor: Sending DockerImageInspected event");
                eprintln!("  Image: {}:{}", image, tag);
                eprintln!("  Ports: {:?}", exposed_ports);
                eprintln!("  Env vars: {} variables", env_vars.len());

                self.send_event(SshEvent::DockerImageInspected {
                    image: image.clone(),
                    tag: tag.clone(),
                    exposed_ports,
                    env_vars,
                }).await;
                eprintln!("✓ SSH Actor: DockerImageInspected event sent");

                return Ok(());
            }
            SshCommand::RemoveDockerContainers { host_name, container_names } => {
                eprintln!("🔍 SSH Actor: remove_docker_containers called for '{}'", host_name);
                eprintln!("  Containers to remove: {:?}", container_names);

                // Load host config
                let host_config = match self.load_host_config(&host_name) {
                    Ok(cfg) => cfg,
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "RemoveDockerContainers".to_string(),
                            error: format!("Host not found: {}", e),
                        }).await;
                        return Ok(());
                    }
                };

                let mut removed = Vec::new();
                let mut failed = Vec::new();

                for container_name in container_names {
                    let rm_cmd = format!("docker rm {}", container_name);
                    let host_config_clone = host_config.clone();
                    match async_compat::Compat::new(crate::calc::ssh::execute_command(&host_config_clone, &rm_cmd)).await {
                        Ok(_) => {
                            eprintln!("✓ SSH Actor: Removed container '{}'", container_name);
                            removed.push(container_name.clone());
                        }
                        Err(e) => {
                            eprintln!("❌ SSH Actor: Failed to remove '{}': {}", container_name, e);
                            failed.push((container_name.clone(), e.to_string()));
                        }
                    }
                }

                eprintln!("🔍 SSH Actor: Sending DockerContainersRemoved event");
                eprintln!("  Removed: {} containers", removed.len());
                eprintln!("  Failed: {} containers", failed.len());

                self.send_event(SshEvent::DockerContainersRemoved {
                    host_name: host_name.clone(),
                    removed,
                    failed,
                }).await;
                eprintln!("✓ SSH Actor: DockerContainersRemoved event sent");

                return Ok(());
            }

            // Ansible Lifecycle Commands
            SshCommand::ValidateAnsibleRole { role } => {
                return self.handle_validate_ansible_role(role).await;
            }
            SshCommand::InstallAnsibleRole {
                host_name,
                instance_name,
                galaxy_name,
                variables,
                ports,
            } => {
                return self.handle_install_ansible_role(host_name, instance_name, galaxy_name, variables, ports).await;
            }
            SshCommand::RemoveAnsibleRole {
                host_name,
                instance_name,
            } => {
                return self.handle_remove_ansible_role(host_name, instance_name).await;
            }
            SshCommand::ListAnsibleRoles { host_name } => {
                return self.handle_list_ansible_roles(host_name).await;
            }

            // Dure-WSS Lifecycle Commands
            SshCommand::InstallDureWssService {
                host_name,
                domain,
                email,
                channel,
                variant,
            } => {
                return self.handle_install_dure_wss_service(host_name, domain, email, channel, variant).await;
            }
            SshCommand::StartDureWss { host_name } => {
                return self.handle_start_dure_wss(host_name).await;
            }
            SshCommand::StopDureWss { host_name } => {
                return self.handle_stop_dure_wss(host_name).await;
            }
            SshCommand::RestartDureWss { host_name } => {
                return self.handle_restart_dure_wss(host_name).await;
            }
            SshCommand::UninstallDureWss { host_name } => {
                return self.handle_uninstall_dure_wss(host_name).await;
            }
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

    // Docker Lifecycle Handlers

    async fn handle_install_docker_image(
        &self,
        host_name: String,
        container_name: String,
        image: String,
        tag: String,
        ports: Vec<(u16, u16)>,
        env: Vec<(String, String)>,
    ) -> anyhow::Result<()> {
        let host_config = match self.load_host_config(&host_name) {
            Ok(cfg) => cfg,
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "InstallDockerImage".to_string(),
                    error: format!("Host not found: {}", e),
                }).await;
                return Ok(());
            }
        };

        if let Err(conflict) = self.check_port_conflicts(&host_name, &ports) {
            self.send_event(SshEvent::Error {
                operation: "InstallDockerImage".to_string(),
                error: conflict,
            }).await;
            return Ok(());
        }

        match async_compat::Compat::new(docker::is_docker_installed(&host_config)).await
        {
            Ok(true) => {
                // Docker installed, proceed
            }
            Ok(false) => {
                self.send_event(SshEvent::DockerDaemonInstallRequired {
                    host_name: host_name.clone(),
                }).await;

                self.send_event(SshEvent::Progress {
                    operation: "InstallDocker".to_string(),
                    progress: 0.2,
                    status: "Installing Docker daemon...".to_string(),
                }).await;

                match async_compat::Compat::new(docker::install_docker_daemon(&host_config)).await
                {
                    Ok(_) => {
                        self.send_event(SshEvent::DockerDaemonInstalled {
                            host_name: host_name.clone(),
                        }).await;
                    }
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "InstallDocker".to_string(),
                            error: e.to_string(),
                        }).await;
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "CheckDocker".to_string(),
                    error: e.to_string(),
                }).await;
                return Ok(());
            }
        }

        self.send_event(SshEvent::Progress {
            operation: "InstallDockerImage".to_string(),
            progress: 0.5,
            status: format!("Pulling image {}:{}", image, tag),
        }).await;

        let container_config = DockerContainerConfig {
            name: container_name.clone(),
            image: image.clone(),
            tag: tag.clone(),
            ports: ports.clone(),
            env: env.clone(),
            status: "running".to_string(),
        };

        match async_compat::Compat::new(docker::run_docker_container(
            &host_config,
            &container_config,
        ))
        .await
        {
            Ok(_) => {
                if let Err(e) = self.save_docker_container(&host_name, container_config) {
                    self.send_event(SshEvent::Error {
                        operation: "SaveConfig".to_string(),
                        error: e.to_string(),
                    }).await;
                }

                self.send_event(SshEvent::DockerImageInstalled {
                    host_name,
                    container_name,
                }).await;
            }
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "InstallDockerImage".to_string(),
                    error: e.to_string(),
                }).await;
            }
        }
        Ok(())
    }

    async fn handle_remove_docker_container(
        &self,
        host_name: String,
        container_name: String,
    ) -> anyhow::Result<()> {
        let host_config = match self.load_host_config(&host_name) {
            Ok(cfg) => cfg,
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "RemoveDockerContainer".to_string(),
                    error: e.to_string(),
                }).await;
                return Ok(());
            }
        };

        match async_compat::Compat::new(docker::remove_docker_container(
            &host_config,
            &container_name,
        ))
        .await
        {
            Ok(_) => {
                let (mut config, config_path) = match self.get_config_and_path() {
                    Ok(c) => c,
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "LoadConfig".to_string(),
                            error: e.to_string(),
                        }).await;
                        return Ok(());
                    }
                };

                if let Some(host) = config.ssh_hosts.iter_mut().find(|h| h.host == host_name) {
                    host.docker_containers.retain(|c| c.name != container_name);
                    let _ = config.save(&config_path);
                }

                self.send_event(SshEvent::DockerContainerRemoved {
                    host_name,
                    container_name,
                }).await;
            }
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "RemoveDockerContainer".to_string(),
                    error: e.to_string(),
                }).await;
            }
        }
        Ok(())
    }

    async fn handle_list_docker_containers(&self, host_name: String) -> anyhow::Result<()> {
        let host_config = match self.load_host_config(&host_name) {
            Ok(cfg) => cfg,
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "ListDockerContainers".to_string(),
                    error: e.to_string(),
                }).await;
                return Ok(());
            }
        };

        match runtime::unblock(move || {
            smol::block_on(docker::list_docker_containers(&host_config))
        })
        .await
        {
            Ok(containers) => {
                self.send_event(SshEvent::DockerContainersListedNew {
                    host_name,
                    containers,
                }).await;
            }
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "ListDockerContainers".to_string(),
                    error: e.to_string(),
                }).await;
            }
        }
        Ok(())
    }

    // Ansible Lifecycle Handlers

    async fn handle_validate_ansible_role(&self, role: String) -> anyhow::Result<()> {
        self.send_event(SshEvent::Progress {
            operation: "ValidateAnsibleRole".to_string(),
            progress: 0.5,
            status: format!("Fetching metadata for {}", role),
        }).await;

        let role_clone = role.clone();
        match runtime::unblock(move || {
            smol::block_on(ansible::fetch_ansible_role_metadata(&role_clone))
        })
        .await
        {
            Ok(metadata) => {
                self.send_event(SshEvent::AnsibleRoleValidated {
                    role,
                    metadata,
                }).await;
            }
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "ValidateAnsibleRole".to_string(),
                    error: e.to_string(),
                }).await;
            }
        }
        Ok(())
    }

    async fn handle_install_ansible_role(
        &self,
        host_name: String,
        instance_name: String,
        galaxy_name: String,
        variables: Vec<(String, String)>,
        ports: Vec<u16>,
    ) -> anyhow::Result<()> {
        let host_config = match self.load_host_config(&host_name) {
            Ok(cfg) => cfg,
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "InstallAnsibleRole".to_string(),
                    error: e.to_string(),
                }).await;
                return Ok(());
            }
        };

        let ports_for_check: Vec<(u16, u16)> = ports.iter().map(|p| (*p, *p)).collect();
        if let Err(conflict) = self.check_port_conflicts(&host_name, &ports_for_check) {
            self.send_event(SshEvent::Error {
                operation: "InstallAnsibleRole".to_string(),
                error: conflict,
            }).await;
            return Ok(());
        }

        let host_config_clone = host_config.clone();
        match runtime::unblock(move || {
            smol::block_on(ansible::is_ansible_installed(&host_config_clone))
        })
        .await
        {
            Ok(true) => {
                // Ansible installed, proceed
            }
            Ok(false) => {
                self.send_event(SshEvent::AnsibleDaemonInstallRequired {
                    host_name: host_name.clone(),
                }).await;

                let host_config_clone2 = host_config.clone();
                match runtime::unblock(move || {
                    smol::block_on(ansible::install_ansible(&host_config_clone2))
                })
                .await
                {
                    Ok(_) => {
                        self.send_event(SshEvent::AnsibleDaemonInstalled {
                            host_name: host_name.clone(),
                        }).await;
                    }
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "InstallAnsible".to_string(),
                            error: e.to_string(),
                        }).await;
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "CheckAnsible".to_string(),
                    error: e.to_string(),
                }).await;
                return Ok(());
            }
        }

        let role_config = AnsibleRoleConfig {
            name: instance_name.clone(),
            galaxy_name: galaxy_name.clone(),
            variables: variables.clone(),
            ports: ports.clone(),
            installed: true,
        };

        let host_config_clone3 = host_config.clone();
        let role_config_clone = role_config.clone();
        match runtime::unblock(move || {
            smol::block_on(ansible::install_ansible_role(
                &host_config_clone3,
                &role_config_clone,
            ))
        })
        .await
        {
            Ok(_) => {
                if let Err(e) = self.save_ansible_role(&host_name, role_config) {
                    self.send_event(SshEvent::Error {
                        operation: "SaveConfig".to_string(),
                        error: e.to_string(),
                    }).await;
                }

                self.send_event(SshEvent::AnsibleRoleInstalled {
                    host_name,
                    instance_name,
                }).await;
            }
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "InstallAnsibleRole".to_string(),
                    error: e.to_string(),
                }).await;
            }
        }
        Ok(())
    }

    async fn handle_remove_ansible_role(
        &self,
        host_name: String,
        instance_name: String,
    ) -> anyhow::Result<()> {
        let (config, _) = match self.get_config_and_path() {
            Ok(c) => c,
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "RemoveAnsibleRole".to_string(),
                    error: e.to_string(),
                }).await;
                return Ok(());
            }
        };

        let host_config = match config.ssh_hosts.iter().find(|h| h.host == host_name) {
            Some(h) => h.clone(),
            None => {
                self.send_event(SshEvent::Error {
                    operation: "RemoveAnsibleRole".to_string(),
                    error: format!("Host '{}' not found", host_name),
                }).await;
                return Ok(());
            }
        };

        let galaxy_name = host_config
            .ansible_roles
            .iter()
            .find(|r| r.name == instance_name)
            .map(|r| r.galaxy_name.clone());

        if let Some(gname) = galaxy_name {
            match runtime::unblock(move || {
                smol::block_on(ansible::remove_ansible_role(&host_config, &gname))
            })
            .await
            {
                Ok(_) => {
                    let (mut config, config_path) = match self.get_config_and_path() {
                        Ok(c) => c,
                        Err(e) => {
                            self.send_event(SshEvent::Error {
                                operation: "LoadConfig".to_string(),
                                error: e.to_string(),
                            }).await;
                            return Ok(());
                        }
                    };

                    if let Some(host) = config.ssh_hosts.iter_mut().find(|h| h.host == host_name) {
                        host.ansible_roles.retain(|r| r.name != instance_name);
                        let _ = config.save(&config_path);
                    }

                    self.send_event(SshEvent::AnsibleRoleRemoved {
                        host_name,
                        instance_name,
                    }).await;
                }
                Err(e) => {
                    self.send_event(SshEvent::Error {
                        operation: "RemoveAnsibleRole".to_string(),
                        error: e.to_string(),
                    }).await;
                }
            }
        }
        Ok(())
    }

    async fn handle_list_ansible_roles(&self, host_name: String) -> anyhow::Result<()> {
        let host_config = match self.load_host_config(&host_name) {
            Ok(cfg) => cfg,
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "ListAnsibleRoles".to_string(),
                    error: e.to_string(),
                }).await;
                return Ok(());
            }
        };

        match runtime::unblock(move || {
            smol::block_on(ansible::list_ansible_roles(&host_config))
        })
        .await
        {
            Ok(roles) => {
                self.send_event(SshEvent::AnsibleRolesListed {
                    host_name,
                    roles,
                }).await;
            }
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "ListAnsibleRoles".to_string(),
                    error: e.to_string(),
                }).await;
            }
        }
        Ok(())
    }

    // Dure-WSS Lifecycle Handlers

    async fn handle_install_dure_wss_service(
        &self,
        host_name: String,
        domain: String,
        email: String,
        channel: String,
        variant: String,
    ) -> anyhow::Result<()> {
        let host_config = match self.load_host_config(&host_name) {
            Ok(cfg) => cfg,
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "InstallDureWssService".to_string(),
                    error: e.to_string(),
                }).await;
                return Ok(());
            }
        };

        let dure_config = DureWssConfig {
            domain: domain.clone(),
            email: email.clone(),
            channel: channel.clone(),
            variant: variant.clone(),
            status: "running".to_string(),
        };

        let dure_config_clone = dure_config.clone();
        match runtime::unblock(move || {
            smol::block_on(dure_wss::install_dure_wss(&host_config, &dure_config_clone))
        })
        .await
        {
            Ok(_) => {
                if let Err(e) = self.save_dure_wss_config(&host_name, dure_config) {
                    self.send_event(SshEvent::Error {
                        operation: "SaveConfig".to_string(),
                        error: e.to_string(),
                    }).await;
                }

                self.send_event(SshEvent::DureWssServiceInstalled {
                    host_name,
                    domain,
                }).await;
            }
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "InstallDureWssService".to_string(),
                    error: e.to_string(),
                }).await;
            }
        }
        Ok(())
    }

    async fn handle_start_dure_wss(&self, host_name: String) -> anyhow::Result<()> {
        let host_config = match self.load_host_config(&host_name) {
            Ok(cfg) => cfg,
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "StartDureWss".to_string(),
                    error: e.to_string(),
                }).await;
                return Ok(());
            }
        };

        match runtime::unblock(move || {
            smol::block_on(dure_wss::start_dure_wss(&host_config))
        })
        .await
        {
            Ok(_) => {
                self.send_event(SshEvent::DureWssStarted { host_name }).await;
            }
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "StartDureWss".to_string(),
                    error: e.to_string(),
                }).await;
            }
        }
        Ok(())
    }

    async fn handle_stop_dure_wss(&self, host_name: String) -> anyhow::Result<()> {
        let host_config = match self.load_host_config(&host_name) {
            Ok(cfg) => cfg,
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "StopDureWss".to_string(),
                    error: e.to_string(),
                }).await;
                return Ok(());
            }
        };

        match runtime::unblock(move || {
            smol::block_on(dure_wss::stop_dure_wss(&host_config))
        })
        .await
        {
            Ok(_) => {
                self.send_event(SshEvent::DureWssStopped { host_name }).await;
            }
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "StopDureWss".to_string(),
                    error: e.to_string(),
                }).await;
            }
        }
        Ok(())
    }

    async fn handle_restart_dure_wss(&self, host_name: String) -> anyhow::Result<()> {
        let host_config = match self.load_host_config(&host_name) {
            Ok(cfg) => cfg,
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "RestartDureWss".to_string(),
                    error: e.to_string(),
                }).await;
                return Ok(());
            }
        };

        match runtime::unblock(move || {
            smol::block_on(dure_wss::restart_dure_wss(&host_config))
        })
        .await
        {
            Ok(_) => {
                self.send_event(SshEvent::DureWssStopped {
                    host_name: host_name.clone(),
                }).await;
                self.send_event(SshEvent::DureWssStarted { host_name }).await;
            }
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "RestartDureWss".to_string(),
                    error: e.to_string(),
                }).await;
            }
        }
        Ok(())
    }

    async fn handle_uninstall_dure_wss(&self, host_name: String) -> anyhow::Result<()> {
        let host_config = match self.load_host_config(&host_name) {
            Ok(cfg) => cfg,
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "UninstallDureWss".to_string(),
                    error: e.to_string(),
                }).await;
                return Ok(());
            }
        };

        match runtime::unblock(move || {
            smol::block_on(dure_wss::uninstall_dure_wss(&host_config))
        })
        .await
        {
            Ok(_) => {
                let (mut config, config_path) = match self.get_config_and_path() {
                    Ok(c) => c,
                    Err(e) => {
                        self.send_event(SshEvent::Error {
                            operation: "LoadConfig".to_string(),
                            error: e.to_string(),
                        }).await;
                        return Ok(());
                    }
                };

                if let Some(host) = config.ssh_hosts.iter_mut().find(|h| h.host == host_name) {
                    host.dure_wss_config = None;
                    let _ = config.save(&config_path);
                }

                self.send_event(SshEvent::DureWssUninstalled { host_name }).await;
            }
            Err(e) => {
                self.send_event(SshEvent::Error {
                    operation: "UninstallDureWss".to_string(),
                    error: e.to_string(),
                }).await;
            }
        }
        Ok(())
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

/// Parse docker history output to extract EXPOSE and ARG directives
fn parse_docker_history(output: &str) -> (Vec<u16>, Vec<(String, String)>) {
    let mut ports = Vec::new();
    let mut args = Vec::new();

    for line in output.lines() {
        let line = line.trim();

        // Parse EXPOSE directives - handle both old and new buildkit formats

        // Old format: "/bin/sh -c #(nop)  EXPOSE 8080/tcp"
        if let Some(expose_part) = line.strip_prefix("/bin/sh -c #(nop)  EXPOSE ") {
            if let Some(port_str) = expose_part.split('/').next() {
                if let Ok(port) = port_str.parse::<u16>() {
                    ports.push(port);
                }
            }
        }
        // New buildkit format: "EXPOSE [51820/udp]" or "EXPOSE 8080/tcp"
        else if line.starts_with("EXPOSE ") {
            let expose_part = line.strip_prefix("EXPOSE ").unwrap();
            // Remove brackets if present: "[51820/udp]" -> "51820/udp"
            let expose_part = expose_part.trim_start_matches('[').trim_end_matches(']');
            // Extract port number before '/'
            if let Some(port_str) = expose_part.split('/').next() {
                if let Ok(port) = port_str.parse::<u16>() {
                    ports.push(port);
                }
            }
        }

        // Parse ARG directives - handle both old and new formats

        // Old format: "/bin/sh -c #(nop)  ARG KEY=value"
        if let Some(arg_part) = line.strip_prefix("/bin/sh -c #(nop)  ARG ") {
            if let Some((key, value)) = arg_part.split_once('=') {
                args.push((key.trim().to_string(), value.trim().to_string()));
            }
        }
        // New buildkit format: "ARG KEY=value"
        else if line.starts_with("ARG ") {
            let arg_part = line.strip_prefix("ARG ").unwrap();
            if let Some((key, value)) = arg_part.split_once('=') {
                args.push((key.trim().to_string(), value.trim().to_string()));
            }
        }
    }

    eprintln!("🔍 Parsed {} ports and {} args from docker history", ports.len(), args.len());

    // Remove duplicate ports
    ports.sort_unstable();
    ports.dedup();

    // For args, keep last occurrence (layers stack, last wins)
    use std::collections::HashMap;
    let mut arg_map: HashMap<String, String> = HashMap::new();
    for (k, v) in args.iter().rev() {
        arg_map.entry(k.clone()).or_insert_with(|| v.clone());
    }
    args = arg_map.into_iter().collect();
    args.sort_by(|a, b| a.0.cmp(&b.0));

    (ports, args)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_docker_history_empty() {
        let output = "";
        let (ports, args) = parse_docker_history(output);
        assert!(ports.is_empty());
        assert!(args.is_empty());
    }

    #[test]
    fn test_parse_docker_history_ports() {
        let output = r#"/bin/sh -c #(nop)  CMD ["/init"]
/bin/sh -c #(nop)  EXPOSE 51820/udp
/bin/sh -c #(nop)  EXPOSE 8080/tcp
/bin/sh -c #(nop)  EXPOSE 80"#;

        let (ports, args) = parse_docker_history(output);
        assert_eq!(ports, vec![80, 8080, 51820]);
        assert!(args.is_empty());
    }

    #[test]
    fn test_parse_docker_history_args() {
        let output = r#"/bin/sh -c #(nop)  ARG PUID=1000
/bin/sh -c #(nop)  ARG PGID=1000
/bin/sh -c #(nop)  ARG TZ=Etc/UTC"#;

        let (ports, args) = parse_docker_history(output);
        assert!(ports.is_empty());
        assert_eq!(args.len(), 3);

        // Check that arguments exist (order may vary due to HashMap)
        assert!(args.iter().any(|(k, v)| k == "PGID" && v == "1000"));
        assert!(args.iter().any(|(k, v)| k == "PUID" && v == "1000"));
        assert!(args.iter().any(|(k, v)| k == "TZ" && v == "Etc/UTC"));
    }

    #[test]
    fn test_parse_docker_history_deduplication() {
        let output = r#"/bin/sh -c #(nop)  EXPOSE 8080/tcp
/bin/sh -c #(nop)  EXPOSE 8080/tcp
/bin/sh -c #(nop)  ARG BUILD_VERSION=1.0
/bin/sh -c #(nop)  ARG BUILD_VERSION=1.1"#;

        let (ports, args) = parse_docker_history(output);
        assert_eq!(ports, vec![8080]); // deduplicated
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].0, "BUILD_VERSION");
        // Last occurrence wins
        assert_eq!(args[0].1, "1.1");
    }

    #[test]
    fn test_parse_docker_history_buildkit_format() {
        // New buildkit format uses simpler syntax
        let output = r#"EXPOSE [51820/udp]
COPY /root / # buildkit
RUN |3 BUILD_DATE=2026-07-02 ...
ARG WIREGUARD_RELEASE=1.0.20250521-r1
ARG BUILD_DATE=2026-07-02
ENTRYPOINT ["/init"]"#;

        let (ports, args) = parse_docker_history(output);
        assert_eq!(ports, vec![51820]);
        assert_eq!(args.len(), 2);
        assert!(args.iter().any(|(k, v)| k == "WIREGUARD_RELEASE" && v == "1.0.20250521-r1"));
        assert!(args.iter().any(|(k, v)| k == "BUILD_DATE" && v == "2026-07-02"));
    }

    #[test]
    fn test_parse_docker_history_mixed_format() {
        // Mix of old and new formats
        let output = r#"EXPOSE [8080/tcp]
/bin/sh -c #(nop)  EXPOSE 443/tcp
ARG PORT=8080
/bin/sh -c #(nop)  ARG HOST=localhost"#;

        let (ports, args) = parse_docker_history(output);
        assert_eq!(ports, vec![443, 8080]);
        assert_eq!(args.len(), 2);
        assert!(args.iter().any(|(k, v)| k == "PORT" && v == "8080"));
        assert!(args.iter().any(|(k, v)| k == "HOST" && v == "localhost"));
    }
}
