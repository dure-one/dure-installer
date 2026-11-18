//! SSH actor implementation

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use super::{DockerContainer, SshCommand, SshEvent, SshHostInfo};
use crate::viewmodel::{ViewModelEvent, runtime};
use crate::calc::dure_wss;
#[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
use crate::calc::{docker, ansible};
use crate::config::{DureWssConfig, SshHostConfig};
#[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
use crate::config::{DockerContainerConfig, AnsibleRoleConfig};
use smol::channel::{Receiver, Sender};

pub struct SshActor {
    command_rx: Receiver<SshCommand>,
    event_tx: Sender<ViewModelEvent>,
    profile_config_path: Option<std::path::PathBuf>,
    repository: crate::viewmodel::platform::DrawerRepository,
}

impl SshActor {
    pub fn new(
        command_rx: Receiver<SshCommand>,
        event_tx: Sender<ViewModelEvent>,
        profile_config_path: Option<std::path::PathBuf>,
    ) -> Self {
        Self {
            command_rx,
            event_tx,
            profile_config_path,
            repository: crate::viewmodel::platform::DrawerRepository::new(),
        }
    }

    fn get_config_and_path(&self) -> anyhow::Result<(crate::config::AppConfig, std::path::PathBuf)> {
        let config_path = self.profile_config_path
            .clone()
            .unwrap_or_else(|| Self::get_config_path().unwrap());
        let config = crate::config::AppConfig::load_or_default(&config_path);
        Ok((config, config_path))
    }

    fn load_host_config(&self, host_name: &str) -> anyhow::Result<SshHostConfig> {
        let config_path = self.profile_config_path
            .clone()
            .unwrap_or_else(|| Self::get_config_path().unwrap());
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

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
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

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
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

    /// Check if SSH host is reachable via TCP port check with timeout
    async fn handle_check_host_health(&self, name: String, timeout_secs: u8) {
        let start = std::time::Instant::now();

        // Load host config to get host:port
        let host_config = match self.load_host_config(&name) {
            Ok(config) => config,
            Err(e) => {
                dure_debug!("Failed to load host config for {}: {}", name, e);
                let _ = self.event_tx.send(ViewModelEvent::Ssh(
                    SshEvent::HostHealthChecked {
                        name,
                        is_alive: false,
                        latency_ms: None,
                    }
                )).await;
                return;
            }
        };

        // Extract hostname/IP from "user@host" format
        let hostname = if let Some(at_pos) = host_config.host.rfind('@') {
            &host_config.host[at_pos + 1..]
        } else {
            &host_config.host
        };

        let address = format!("{}:{}", hostname, host_config.port);
        dure_debug!("Health check: connecting to {}", address);

        // TCP connection attempt with timeout
        let timeout = std::time::Duration::from_secs(timeout_secs as u64);
        let result = smol::future::or(
            async {
                smol::net::TcpStream::connect(&address).await
            },
            async {
                smol::Timer::after(timeout).await;
                Err(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Health check timeout"
                ))
            }
        ).await;

        let elapsed = start.elapsed();
        let (is_alive, latency_ms) = match result {
            Ok(_stream) => {
                dure_debug!("Health check succeeded for {} in {:?}", name, elapsed);
                (true, Some(elapsed.as_millis() as u64))
            }
            Err(e) => {
                dure_debug!("Health check failed for {}: {}", name, e);
                (false, None)
            }
        };

        let _ = self.event_tx.send(ViewModelEvent::Ssh(
            SshEvent::HostHealthChecked {
                name,
                is_alive,
                latency_ms,
            }
        )).await;
    }

    pub async fn run(mut self) {
        dure_info!("SshActor started");

        loop {
            match self.command_rx.recv().await {
                Ok(cmd) => {
                    if let Err(e) = self.handle_command(cmd).await {
                        dure_error!("SshActor command failed: {}", e);
                    }
                }
                Err(_) => {
                    dure_info!("SshActor: channel closed, shutting down");
                    break;
                }
            }
        }
    }

    fn extract_host_id(cmd: &SshCommand) -> Option<String> {
        match cmd {
            SshCommand::AddHost { name, .. } => Some(name.clone()),
            SshCommand::DeleteHost { name } => Some(name.clone()),
            SshCommand::TestConnection { name } => Some(name.clone()),
            SshCommand::InitHost { name } => Some(name.clone()),
            SshCommand::DockerPull { host_name, .. } => Some(host_name.clone()),
            SshCommand::DockerRun { host_name, .. } => Some(host_name.clone()),
            SshCommand::DockerStop { host_name, .. } => Some(host_name.clone()),
            SshCommand::DockerList { host_name } => Some(host_name.clone()),
            SshCommand::PortOpen { host_name, .. } => Some(host_name.clone()),
            SshCommand::PortClose { host_name, .. } => Some(host_name.clone()),
            SshCommand::PortList { host_name } => Some(host_name.clone()),
            SshCommand::DeployDureWss { host_name, .. } => Some(host_name.clone()),
            SshCommand::GetLinuxStatus { name } => Some(name.clone()),
            SshCommand::CheckHostHealth { name, .. } => Some(name.clone()),
            SshCommand::InspectDockerImage { host_name, .. } => Some(host_name.clone()),
            SshCommand::Refresh { host, .. } => Some(host.clone()),
            SshCommand::SshCheck { host, .. } => Some(host.clone()),
            SshCommand::CheckHost { host, .. } => Some(host.clone()),
            SshCommand::InstallHost { host, .. } => Some(host.clone()),
            SshCommand::CheckDocker { host, .. } => Some(host.clone()),
            SshCommand::InstallDockerDaemon { host, .. } => Some(host.clone()),
            SshCommand::RemoveDocker { host, .. } => Some(host.clone()),
            SshCommand::CheckDure { host, .. } => Some(host.clone()),
            SshCommand::InstallDure { host, .. } => Some(host.clone()),
            SshCommand::RemoveDure { host, .. } => Some(host.clone()),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::InstallDockerImage { host_name, .. } => Some(host_name.clone()),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::RemoveDockerContainer { host_name, .. } => Some(host_name.clone()),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::InstallDocker { name } => Some(name.clone()),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::GetDockerStatus { name } => Some(name.clone()),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::UninstallDocker { name } => Some(name.clone()),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::InstallAnsible { name } => Some(name.clone()),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::GetAnsibleStatus { name } => Some(name.clone()),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::UninstallAnsible { name } => Some(name.clone()),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::ListDockerContainers { host_name } => Some(host_name.clone()),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::RemoveDockerContainers { host_name, .. } => Some(host_name.clone()),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::ValidateAnsibleRole { .. } => None,
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::InstallAnsibleRole { host_name, .. } => Some(host_name.clone()),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::RemoveAnsibleRole { host_name, .. } => Some(host_name.clone()),
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::ListAnsibleRoles { host_name } => Some(host_name.clone()),
            SshCommand::InstallDureWssService { host_name, .. } => Some(host_name.clone()),
            SshCommand::StartDureWss { host_name } => Some(host_name.clone()),
            SshCommand::StopDureWss { host_name } => Some(host_name.clone()),
            SshCommand::RestartDureWss { host_name } => Some(host_name.clone()),
            SshCommand::UninstallDureWss { host_name } => Some(host_name.clone()),
            SshCommand::ListHosts => None,
            SshCommand::Drawer(_) => None,
        }
    }

    async fn handle_command(&mut self, cmd: SshCommand) -> anyhow::Result<()> {
        let operation = format!("{:?}", cmd);
        dure_debug!(" SSH Actor: Received command: {}", operation);

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
                dure_debug!(" SSH Actor: Handling TestConnection for '{}'", name);
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
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
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
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::RemoveDockerContainer {
                host_name,
                container_name,
            } => {
                return self.handle_remove_docker_container(host_name, container_name).await;
            }
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::InstallDocker { name } => self.install_docker(name).await,
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::GetDockerStatus { name } => self.get_docker_status(name).await,
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::UninstallDocker { name } => self.uninstall_docker(name).await,
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::InstallAnsible { name } => self.install_ansible(name).await,
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::GetAnsibleStatus { name } => self.get_ansible_status(name).await,
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::UninstallAnsible { name } => self.uninstall_ansible(name).await,
            SshCommand::CheckHostHealth { name, timeout_secs } => {
                self.handle_check_host_health(name, timeout_secs).await;
                Ok(())
            }
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::ListDockerContainers { host_name } => {
                return self.handle_list_docker_containers(host_name).await;
            }
            SshCommand::InspectDockerImage { host_name, image, tag } => {
                dure_debug!(" SSH Actor: inspect_docker_image called for '{}' with {}:{}", host_name, image, tag);

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
                match crate::calc::ssh::execute_command(&host_config_clone, &pull_cmd, None).await {
                    Ok(output) => {
                        dure_debug!(" SSH Actor: Image pulled successfully");
                        dure_debug!(" Pull output: {}", output);
                    }
                    Err(e) => {
                        dure_error!(" SSH Actor: Failed to pull image: {}", e);
                        self.send_event(SshEvent::Error {
                            operation: format!("inspect_docker_image({})", full_image),
                            error: format!("Failed to pull image: {}", e),
                        }).await;
                        return Ok(());
                    }
                }

                // Step 2: Get image history
                let history_cmd = format!("docker history {} --no-trunc --format \"{{{{.CreatedBy}}}}\"", full_image);
                dure_debug!(" SSH Actor: Running command: {}", history_cmd);
                let history_output = match crate::calc::ssh::execute_command(&host_config, &history_cmd, None).await {
                    Ok(output) => output,
                    Err(e) => {
                        dure_error!(" SSH Actor: Failed to get image history: {}", e);
                        self.send_event(SshEvent::Error {
                            operation: format!("inspect_docker_image({})", full_image),
                            error: format!("Failed to inspect image history: {}", e),
                        }).await;
                        return Ok(());
                    }
                };

                dure_debug!(" History output ({} bytes, {} lines):", history_output.len(), history_output.lines().count());
                for (i, line) in history_output.lines().take(10).enumerate() {
                    dure_debug!("  Line {}: {}", i + 1, line);
                }
                if history_output.lines().count() > 10 {
                    dure_debug!("  ... ({} more lines)", history_output.lines().count() - 10);
                }

                // Step 3: Parse history output
                let (exposed_ports, env_vars) = parse_docker_history(&history_output);

                dure_debug!(" SSH Actor: Sending DockerImageInspected event");
                dure_debug!("  Image: {}:{}", image, tag);
                dure_debug!("  Ports: {:?}", exposed_ports);
                dure_debug!("  Env vars: {} variables", env_vars.len());

                self.send_event(SshEvent::DockerImageInspected {
                    image: image.clone(),
                    tag: tag.clone(),
                    exposed_ports,
                    env_vars,
                }).await;
                dure_info!(" SSH Actor: DockerImageInspected event sent");

                return Ok(());
            }
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::RemoveDockerContainers { host_name, container_names } => {
                dure_debug!(" SSH Actor: remove_docker_containers called for '{}'", host_name);
                dure_debug!("  Containers to remove: {:?}", container_names);

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
                    match docker::remove_docker_container(&host_config, &container_name).await {
                        Ok(_) => {
                            dure_info!(" SSH Actor: Removed container '{}'", container_name);
                            removed.push(container_name.clone());
                        }
                        Err(e) => {
                            dure_error!(" SSH Actor: Failed to remove '{}': {}", container_name, e);
                            failed.push((container_name.clone(), e.to_string()));
                        }
                    }
                }

                dure_debug!(" SSH Actor: Sending DockerContainersRemoved event");
                dure_debug!("  Removed: {} containers", removed.len());
                dure_debug!("  Failed: {} containers", failed.len());

                self.send_event(SshEvent::DockerContainersRemoved {
                    host_name: host_name.clone(),
                    removed,
                    failed,
                }).await;
                dure_info!(" SSH Actor: DockerContainersRemoved event sent");

                return Ok(());
            }

            // Ansible Lifecycle Commands
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::ValidateAnsibleRole { role } => {
                return self.handle_validate_ansible_role(role).await;
            }
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::InstallAnsibleRole {
                host_name,
                instance_name,
                galaxy_name,
                variables,
                ports,
            } => {
                return self.handle_install_ansible_role(host_name, instance_name, galaxy_name, variables, ports).await;
            }
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
            SshCommand::RemoveAnsibleRole {
                host_name,
                instance_name,
            } => {
                return self.handle_remove_ansible_role(host_name, instance_name).await;
            }
            #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
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

            // Operation commands (new)
            SshCommand::Refresh { host, profile_kdbx } => {
                let event = self.handle_refresh(host, profile_kdbx).await;
                self.send_event(event).await;
                Ok(())
            }
            SshCommand::SshCheck { host, profile_kdbx } => {
                self.handle_ssh_check(host, profile_kdbx).await
            }
            SshCommand::CheckHost { host, profile_kdbx } => {
                self.handle_check_host(host, profile_kdbx).await
            }
            SshCommand::InstallHost { host, profile_kdbx: _ } => {
                self.send_event(SshEvent::HostInstallCompleted {
                    host: host.clone(),
                    success: false,
                    error: Some("Not implemented".to_string()),
                }).await;
                Ok(())
            }
            SshCommand::CheckDocker { host, profile_kdbx } => {
                self.handle_check_docker(host, profile_kdbx).await
            }
            SshCommand::InstallDockerDaemon { host, profile_kdbx: _ } => {
                self.send_event(SshEvent::DockerInstallCompleted {
                    host: host.clone(),
                    success: false,
                    error: Some("Not implemented".to_string()),
                }).await;
                Ok(())
            }
            SshCommand::RemoveDocker { host, profile_kdbx: _ } => {
                self.send_event(SshEvent::DockerRemoveCompleted {
                    host: host.clone(),
                    success: false,
                    error: Some("Not implemented".to_string()),
                }).await;
                Ok(())
            }
            SshCommand::CheckDure { host, profile_kdbx } => {
                self.handle_check_dure(host, profile_kdbx).await
            }
            SshCommand::InstallDure { host, env_config, profile_kdbx } => {
                self.handle_install_dure(host, env_config, profile_kdbx).await
            }
            SshCommand::RemoveDure { host, delete_data, profile_kdbx } => {
                self.handle_remove_dure(host, delete_data, profile_kdbx).await
            }
            SshCommand::Drawer(_drawer_cmd) => {
                Err(anyhow::anyhow!("Drawer command not yet implemented"))
            }

            // Catch-all for cfg-gated commands on Android/WASM
            #[cfg(any(target_os = "android", target_arch = "wasm32"))]
            _ => {
                Err(anyhow::anyhow!("Command not available on this platform"))
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
        let log_id = self.start_operation("add_host", &name, true).await.unwrap_or(None);
        self.log_to_tab(&name, crate::viewmodel::logs::LogLevel::Info, "Adding SSH host...");

        let result = runtime::unblock({
            let host_clone = host.clone();
            move || -> anyhow::Result<()> {
                let config_path = Self::get_config_path()?;
                let mut app_config = crate::config::AppConfig::load_or_default(&config_path);

                if app_config.ssh_hosts.iter().any(|h| h.host == host_clone) {
                    anyhow::bail!("SSH host '{}' already exists", host_clone);
                }

                let ssh_host = crate::config::SshHostConfig {
                    host: host_clone.clone(),
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
                    last_ssh_check: None,
                    last_host_check: None,
                    last_docker_check: None,
                    last_dure_check: None,
                };

                app_config.ssh_hosts.push(ssh_host);
                app_config.save(&config_path)?;
                Ok(())
            }
        })
        .await;

        match result {
            Ok(_) => {
                let _ = self.end_operation_success("add_host", &name, log_id).await;
                self.send_event(SshEvent::HostAdded { name }).await;
                Ok(())
            }
            Err(e) => {
                let _ = self.end_operation_failed("add_host", &name, log_id, &e).await;
                Err(e)
            }
        }
    }

    async fn delete_host(&mut self, name: String) -> anyhow::Result<()> {
        let log_id = self.start_operation("delete_host", &name, true).await.unwrap_or(None);
        self.log_to_tab(&name, crate::viewmodel::logs::LogLevel::Info, "Deleting SSH host...");

        let result = runtime::unblock({
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
        .await;

        match result {
            Ok(_) => {
                let _ = self.end_operation_success("delete_host", &name, log_id).await;
                self.send_event(SshEvent::HostDeleted { name }).await;
                Ok(())
            }
            Err(e) => {
                let _ = self.end_operation_failed("delete_host", &name, log_id, &e).await;
                Err(e)
            }
        }
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

    /// Helper to get config file path (Desktop)
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    fn get_config_path() -> anyhow::Result<std::path::PathBuf> {
        Ok(crate::get_app_config_dir()?.join("config.yml"))
    }

    /// Helper to get config file path (Android)
    #[cfg(target_os = "android")]
    fn get_config_path() -> anyhow::Result<std::path::PathBuf> {
        Ok(std::path::PathBuf::from("/data/data/app.dure.installer/files/config.yml"))
    }

    /// Helper to get config file path (WASM)
    #[cfg(target_arch = "wasm32")]
    fn get_config_path() -> anyhow::Result<std::path::PathBuf> {
        Ok(std::path::PathBuf::from(".dure/config.yml"))
    }

    async fn test_connection(&mut self, name: String) -> anyhow::Result<()> {
        dure_debug!(" SSH Actor: test_connection called for '{}'", name);
        self.send_progress("test_connection", 0.5, "Testing SSH connection...")
            .await;

        let start = std::time::Instant::now();

        // Load host config first (blocking operation)
        dure_debug!(" SSH Actor: Loading host config...");
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

                dure_debug!(" SSH Actor: Found host config for '{}'", host_config.host);
                Ok(host_config)
            }
        })
        .await?;

        // Test connection (async operation - russh uses tokio internally)
        dure_debug!("🔍 SSH Actor: Starting SSH connection test to {}:{}...", host_config.host, host_config.port
        );
        // TODO: Pass profile keyring handle when SSH Actor has profile context
        let result =
            crate::calc::ssh::test_connection(&host_config, None).await;

        let latency_ms = start.elapsed().as_millis() as u64;
        dure_debug!("🔍 SSH Actor: Connection test completed in {}ms", latency_ms
        );

        match result {
            Ok(conn_result) => {
                dure_debug!("✓ SSH Actor: Connection test succeeded: {}", conn_result.success
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
                dure_warn!(" SSH Actor: Connection test failed: {}", e);
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
        dure_debug!(" SSH Actor: init_host called for '{}'", name);
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
            crate::calc::ssh::initialize_host(&host_config).await;

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
                dure_warn!(" SSH Actor: Host initialization failed: {}", e);
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
        dure_debug!(" SSH Actor: get_linux_status called for '{}'", name);
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
            crate::calc::ssh::get_linux_status(&host_config).await;

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
                dure_warn!(" SSH Actor: Linux status retrieval failed: {}", e);
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
        dure_debug!(" SSH Actor: install_docker called for '{}'", name);
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
            crate::calc::ssh::install_docker(&host_config).await;

        match result {
            Ok(_) => {
                self.send_progress("install_docker", 1.0, "Docker installed")
                    .await;
                self.send_event(SshEvent::DockerInstalled { name }).await;
                Ok(())
            }
            Err(e) => {
                dure_warn!(" SSH Actor: Docker installation failed: {}", e);
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
        dure_debug!(" SSH Actor: get_docker_status called for '{}'", name);
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
            crate::calc::ssh::check_docker_installed(&host_config)
                .await?;

        let running = if installed {
            crate::calc::ssh::check_docker_running(&host_config).await?
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
        dure_debug!(" SSH Actor: uninstall_docker called for '{}'", name);
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
            crate::calc::ssh::uninstall_docker(&host_config).await;

        match result {
            Ok(_) => {
                self.send_progress("uninstall_docker", 1.0, "Docker uninstalled")
                    .await;
                self.send_event(SshEvent::DockerUninstalled { name }).await;
                Ok(())
            }
            Err(e) => {
                dure_warn!(" SSH Actor: Docker uninstallation failed: {}", e);
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

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    async fn install_ansible(&mut self, name: String) -> anyhow::Result<()> {
        dure_debug!(" SSH Actor: install_ansible called for '{}'", name);
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
            crate::calc::ssh::install_ansible(&host_config).await;

        match result {
            Ok(_) => {
                self.send_progress("install_ansible", 1.0, "Ansible installed")
                    .await;
                self.send_event(SshEvent::AnsibleInstalled { name }).await;
                Ok(())
            }
            Err(e) => {
                dure_warn!(" SSH Actor: Ansible installation failed: {}", e);
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

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    async fn get_ansible_status(&mut self, name: String) -> anyhow::Result<()> {
        dure_debug!(" SSH Actor: get_ansible_status called for '{}'", name);
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
            crate::calc::ssh::check_ansible_installed(&host_config)
                .await?;

        self.send_progress("get_ansible_status", 1.0, "Status retrieved")
            .await;
        self.send_event(SshEvent::AnsibleStatusRetrieved { name, installed })
            .await;
        Ok(())
    }

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    async fn uninstall_ansible(&mut self, name: String) -> anyhow::Result<()> {
        dure_debug!(" SSH Actor: uninstall_ansible called for '{}'", name);
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
            crate::calc::ssh::uninstall_ansible(&host_config).await;

        match result {
            Ok(_) => {
                self.send_progress("uninstall_ansible", 1.0, "Ansible uninstalled")
                    .await;
                self.send_event(SshEvent::AnsibleUninstalled { name }).await;
                Ok(())
            }
            Err(e) => {
                dure_warn!(" SSH Actor: Ansible uninstallation failed: {}", e);
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

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
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

        match docker::is_docker_installed(&host_config).await
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

                match docker::install_docker_daemon(&host_config).await
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

        match docker::run_docker_container(
            &host_config,
            &container_config,
        )
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

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
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

        match docker::remove_docker_container(
            &host_config,
            &container_name,
        )
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

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
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

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
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

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
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

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
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

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
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

    fn get_event_description(event: &SshEvent) -> String {
        match event {
            SshEvent::HostAdded { name } => format!("HostAdded({})", name),
            SshEvent::HostDeleted { name } => format!("HostDeleted({})", name),
            SshEvent::HostsListed { hosts } => format!("HostsListed({} hosts)", hosts.len()),
            SshEvent::ConnectionTested { name, success, .. } => {
                format!("ConnectionTested({}, success={})", name, success)
            }
            SshEvent::HostInitialized { name, success } => {
                format!("HostInitialized({}, success={})", name, success)
            }
            SshEvent::DockerImagePulled { host_name, image } => {
                format!("DockerImagePulled({}, {})", host_name, image)
            }
            SshEvent::DockerContainerStarted { host_name, container_name } => {
                format!("DockerContainerStarted({}, {})", host_name, container_name)
            }
            SshEvent::DockerContainerStopped { host_name, container_name } => {
                format!("DockerContainerStopped({}, {})", host_name, container_name)
            }
            SshEvent::DockerContainersListed { host_name, containers } => {
                format!("DockerContainersListed({}, {} containers)", host_name, containers.len())
            }
            SshEvent::PortOpened { host_name, port, protocol } => {
                format!("PortOpened({}, {}:{})", host_name, port, protocol)
            }
            SshEvent::PortClosed { host_name, port, protocol } => {
                format!("PortClosed({}, {}:{})", host_name, port, protocol)
            }
            SshEvent::PortsListed { host_name, open_ports } => {
                format!("PortsListed({}, {} ports)", host_name, open_ports.len())
            }
            SshEvent::DockerDaemonInstallRequired { host_name } => {
                format!("DockerDaemonInstallRequired({})", host_name)
            }
            SshEvent::DockerDaemonInstalled { host_name } => {
                format!("DockerDaemonInstalled({})", host_name)
            }
            SshEvent::DockerImageInstalled { host_name, container_name } => {
                format!("DockerImageInstalled({}, {})", host_name, container_name)
            }
            SshEvent::DockerContainerRemoved { host_name, container_name } => {
                format!("DockerContainerRemoved({}, {})", host_name, container_name)
            }
            #[cfg(not(target_arch = "wasm32"))]
            SshEvent::DockerContainersListedNew { host_name, containers } => {
                format!("DockerContainersListedNew({}, {} containers)", host_name, containers.len())
            }
            #[cfg(not(target_arch = "wasm32"))]
            SshEvent::AnsibleRoleValidated { role, .. } => {
                format!("AnsibleRoleValidated({})", role)
            }
            SshEvent::AnsibleDaemonInstallRequired { host_name } => {
                format!("AnsibleDaemonInstallRequired({})", host_name)
            }
            SshEvent::AnsibleDaemonInstalled { host_name } => {
                format!("AnsibleDaemonInstalled({})", host_name)
            }
            SshEvent::AnsibleRoleInstalled { host_name, instance_name } => {
                format!("AnsibleRoleInstalled({}, {})", host_name, instance_name)
            }
            SshEvent::AnsibleRoleRemoved { host_name, instance_name } => {
                format!("AnsibleRoleRemoved({}, {})", host_name, instance_name)
            }
            SshEvent::AnsibleRolesListed { host_name, roles } => {
                format!("AnsibleRolesListed({}, {} roles)", host_name, roles.len())
            }
            SshEvent::DureWssServiceInstalled { host_name, domain } => {
                format!("DureWssServiceInstalled({}, {})", host_name, domain)
            }
            SshEvent::DureWssStarted { host_name } => {
                format!("DureWssStarted({})", host_name)
            }
            SshEvent::DureWssStopped { host_name } => {
                format!("DureWssStopped({})", host_name)
            }
            SshEvent::DureWssUninstalled { host_name } => {
                format!("DureWssUninstalled({})", host_name)
            }
            SshEvent::DureWssDeployed { host_name, domain, .. } => {
                format!("DureWssDeployed({}, {})", host_name, domain)
            }
            SshEvent::LinuxStatusRetrieved { name, .. } => {
                format!("LinuxStatusRetrieved({})", name)
            }
            SshEvent::DockerInstalled { name } => {
                format!("DockerInstalled({})", name)
            }
            SshEvent::DockerStatusRetrieved { name, installed, running } => {
                format!("DockerStatusRetrieved({}, installed={}, running={})", name, installed, running)
            }
            SshEvent::DockerUninstalled { name } => {
                format!("DockerUninstalled({})", name)
            }
            SshEvent::AnsibleInstalled { name } => {
                format!("AnsibleInstalled({})", name)
            }
            SshEvent::AnsibleStatusRetrieved { name, installed } => {
                format!("AnsibleStatusRetrieved({}, installed={})", name, installed)
            }
            SshEvent::AnsibleUninstalled { name } => {
                format!("AnsibleUninstalled({})", name)
            }
            SshEvent::DureWssInstalled { name } => {
                format!("DureWssInstalled({})", name)
            }
            SshEvent::DureWssStatusRetrieved { name, installed } => {
                format!("DureWssStatusRetrieved({}, installed={})", name, installed)
            }
            SshEvent::HostHealthChecked { name, is_alive, latency_ms } => {
                if let Some(ms) = latency_ms {
                    format!("HostHealthChecked({}, alive={}, latency={}ms)", name, is_alive, ms)
                } else {
                    format!("HostHealthChecked({}, alive={})", name, is_alive)
                }
            }
            SshEvent::DockerImageInspected { image, tag, .. } => {
                format!("DockerImageInspected({}:{})", image, tag)
            }
            SshEvent::DockerContainersRemoved { host_name, removed, failed } => {
                format!("DockerContainersRemoved({}, removed={}, failed={})",
                    host_name, removed.len(), failed.len())
            }
            SshEvent::ServiceError { name, service, operation, .. } => {
                format!("ServiceError({}, service={}, op={})", name, service, operation)
            }
            SshEvent::Progress { operation, progress, .. } => {
                format!("Progress({}, {:.0}%)", operation, progress * 100.0)
            }
            SshEvent::Error { operation, .. } => {
                format!("Error({})", operation)
            }

            // Operation events (new)
            SshEvent::RefreshCompleted { host, .. } => {
                format!("RefreshCompleted({})", host)
            }
            SshEvent::SshCheckCompleted { host, connected, .. } => {
                format!("SshCheckCompleted({}, connected={})", host, connected)
            }
            SshEvent::HostCheckCompleted { host, installed, .. } => {
                format!("HostCheckCompleted({}, installed={})", host, installed)
            }
            SshEvent::HostInstallCompleted { host, success, .. } => {
                format!("HostInstallCompleted({}, success={})", host, success)
            }
            SshEvent::DockerCheckCompleted { host, installed, .. } => {
                format!("DockerCheckCompleted({}, installed={})", host, installed)
            }
            SshEvent::DockerInstallCompleted { host, success, .. } => {
                format!("DockerInstallCompleted({}, success={})", host, success)
            }
            SshEvent::DockerRemoveCompleted { host, success, .. } => {
                format!("DockerRemoveCompleted({}, success={})", host, success)
            }
            SshEvent::DureCheckCompleted { host, installed, running, .. } => {
                format!("DureCheckCompleted({}, installed={}, running={})", host, installed, running)
            }
            SshEvent::DureInstallCompleted { host, success, .. } => {
                format!("DureInstallCompleted({}, success={})", host, success)
            }
            SshEvent::DureRemoveCompleted { host, success, .. } => {
                format!("DureRemoveCompleted({}, success={})", host, success)
            }
            SshEvent::OperationFailed { host, operation, .. } => {
                format!("OperationFailed({}, {})", host, operation)
            }
            SshEvent::HostStatusRetrieved { host, raw_output } => {
                format!("HostStatusRetrieved({}, {} bytes)", host, raw_output.len())
            }
            SshEvent::DockerStatusRawRetrieved { host, raw_output } => {
                format!("DockerStatusRawRetrieved({}, {} bytes)", host, raw_output.len())
            }
            SshEvent::DureComposeLogsRetrieved { host, logs } => {
                format!("DureComposeLogsRetrieved({}, {} bytes)", host, logs.len())
            }
            SshEvent::DureContainerLogsRetrieved { host, container_logs } => {
                format!("DureContainerLogsRetrieved({}, {} containers)", host, container_logs.len())
            }
            SshEvent::Drawer(_) => {
                format!("Drawer(...)")
            }
        }
    }

    async fn start_operation_log(&self, host: &str, operation: &str) -> Result<i64, String> {
        use crate::storage::models::opslog::NewOperationLog;
        use crate::calc::db;

        let log = NewOperationLog::ssh(host, operation);

        runtime::unblock({
            move || -> Result<i64, String> {
                let mut conn = db::establish_connection();
                crate::storage::models::opslog::create_log(&mut conn, log)
                    .map_err(|e| format!("Failed to start operation log: {}", e))
            }
        })
        .await
    }

    async fn complete_operation_log(
        &self,
        op_id: i64,
        success: bool,
        error: Option<String>,
    ) -> Result<(), String> {
        use crate::calc::db;
        use crate::storage::models::opslog::OperationStatus;

        runtime::unblock({
            move || -> Result<(), String> {
                let mut conn = db::establish_connection();
                let status = if success {
                    OperationStatus::Success
                } else {
                    OperationStatus::Failed
                };

                crate::storage::models::opslog::update_log_status(&mut conn, op_id, status, error)
                    .map_err(|e| format!("Failed to update operation log: {}", e))
            }
        })
        .await
    }

    async fn handle_refresh(
        &mut self,
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> SshEvent {
        // Start refresh operation (Read operation, no history)
        let log_id = match self.start_operation("refresh", &host, false).await {
            Ok(id) => id,
            Err(e) => {
                dure_warn!("Failed to start operation: {}", e);
                None
            }
        };

        // Load host config
        let host_config = match self.load_host_config(&host) {
            Ok(cfg) => cfg,
            Err(_) => {
                self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Error, "Host config not found");
                return SshEvent::RefreshCompleted {
                    host,
                    ssh_connected: false,
                    host_installed: false,
                    docker_installed: false,
                    dure_installed: false,
                    ssh_error: Some("Host config not found".to_string()),
                    host_missing_packages: None,
                    docker_version: None,
                    dure_running: None,
                    dure_services: None,
                };
            }
        };

        // Run 4 checks with progress logging
        self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Info, "Testing SSH connection...");
        let ssh_result = crate::calc::ssh::test_connection(&host_config, profile_kdbx.as_ref()).await;
        let ssh_connected = ssh_result.as_ref().map(|r| r.success).unwrap_or(false);
        let ssh_error = if !ssh_connected {
            Some(ssh_result.as_ref().map(|r| r.message.clone()).unwrap_or_else(|_| "Connection failed".to_string()))
        } else {
            None
        };

        self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Info, "Checking host packages...");
        let host_missing_packages = if ssh_connected {
            crate::calc::ssh::check_host_packages(&host_config, profile_kdbx.as_ref()).await.ok()
        } else {
            None
        };
        let host_installed = host_missing_packages.as_ref().map(|p| p.is_empty()).unwrap_or(false);

        self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Info, "Checking Docker version...");
        let docker_version = if ssh_connected {
            crate::calc::ssh::check_docker_version(&host_config, profile_kdbx.as_ref()).await.ok().flatten()
        } else {
            None
        };
        let docker_installed = docker_version.is_some();

        self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Info, "Checking Dure status...");
        let dure_check = if ssh_connected {
            crate::calc::ssh::check_dure_mycart(&host_config, profile_kdbx.as_ref()).await.ok()
        } else {
            None
        };
        let dure_installed = dure_check.as_ref().map(|(installed, _, _)| *installed).unwrap_or(false);
        let dure_running = dure_check.as_ref().map(|(_, running, _)| *running);
        let dure_services = dure_check.map(|(_, _, services)| services);

        // End operation successfully
        if let Err(e) = self.end_operation_success("refresh", &host, log_id).await {
            dure_warn!("Failed to end operation: {}", e);
        }

        SshEvent::RefreshCompleted {
            host,
            ssh_connected,
            host_installed,
            docker_installed,
            dure_installed,
            ssh_error,
            host_missing_packages,
            docker_version,
            dure_running,
            dure_services,
        }
    }

    async fn handle_ssh_check(
        &mut self,
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        // Start ssh_check operation (Read operation, no history)
        let log_id = self.start_operation("ssh_check", &host, false).await.unwrap_or(None);

        dure_info!(project_id = host.clone(), "Checking SSH connection to {}", host);
        self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Info, "Checking SSH connection...");

        // Check SSH connection
        let host_config = self.load_host_config(&host)?;

        let result = crate::calc::ssh::test_connection(&host_config, profile_kdbx.as_ref()).await;

        match result {
            Ok(conn_result) => {
                let connected = conn_result.success;
                let error = if !connected { Some(conn_result.message.clone()) } else { None };

                dure_info!(project_id = host.clone(), "SSH connection check: {}", if connected { "✓ Connected" } else { "✗ Failed" });

                // End operation
                let _ = self.end_operation_success("ssh_check", &host, log_id).await;

                self.send_event(SshEvent::SshCheckCompleted {
                    host,
                    connected,
                    error,
                }).await;
                Ok(())
            }
            Err(e) => {
                dure_error!(project_id = host.clone(), "SSH connection check failed: {}", e);

                // End operation with error
                let _ = self.end_operation_failed("ssh_check", &host, log_id, &e).await;

                self.send_event(SshEvent::SshCheckCompleted {
                    host,
                    connected: false,
                    error: Some(e.to_string()),
                }).await;
                Ok(())
            }
        }
    }

    async fn send_event(&self, event: SshEvent) {
        let event_desc = Self::get_event_description(&event);
        dure_debug!(" SSH Actor: Sending event: {}", event_desc);
        match self.event_tx.send(ViewModelEvent::Ssh(event.clone())).await {
            Ok(_) => dure_info!(" SSH Actor: {}", event_desc),
            Err(e) => dure_warn!(" SSH Actor: Failed to send event ({}): {}", event_desc, e),
        }
    }

    async fn handle_check_host(
        &mut self,
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        let log_id = self.start_operation("check_host", &host, false).await.unwrap_or(None);
        self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Info, "Checking host packages...");

        let host_config = match self.load_host_config(&host) {
            Ok(cfg) => cfg,
            Err(e) => {
                let _ = self.end_operation_failed("check_host", &host, log_id, &e).await;
                return Err(e);
            }
        };

        let result = crate::calc::ssh::check_host_packages(&host_config, profile_kdbx.as_ref()).await;

        match result {
            Ok(missing) => {
                let installed = missing.is_empty();
                let status = if installed {
                    "✓ All packages installed".to_string()
                } else {
                    format!("✗ Missing: {:?}", missing)
                };
                self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Info, &status);

                self.send_event(SshEvent::HostCheckCompleted {
                    host: host.clone(),
                    installed,
                    missing_packages: missing,
                }).await;
            }
            Err(e) => {
                let _ = self.end_operation_failed("check_host", &host, log_id, &e).await;
                self.send_event(SshEvent::HostCheckCompleted {
                    host: host.clone(),
                    installed: false,
                    missing_packages: vec![],
                }).await;
                return Err(e);
            }
        }

        self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Info, "Retrieving system status...");
        match crate::calc::ssh::get_comprehensive_system_status(&host_config, profile_kdbx.as_ref()).await {
            Ok(raw_output) => {
                let _ = self.end_operation_success("check_host", &host, log_id).await;
                self.send_event(SshEvent::HostStatusRetrieved {
                    host,
                    raw_output,
                }).await;
                Ok(())
            }
            Err(e) => {
                self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Error, format!("System status failed: {}", e));
                let _ = self.end_operation_success("check_host", &host, log_id).await;
                Ok(())
            }
        }
    }

    async fn handle_check_docker(
        &mut self,
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        let log_id = self.start_operation("check_docker", &host, false).await.unwrap_or(None);
        self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Info, "Checking Docker version...");

        let host_config = match self.load_host_config(&host) {
            Ok(cfg) => cfg,
            Err(e) => {
                let _ = self.end_operation_failed("check_docker", &host, log_id, &e).await;
                return Err(e);
            }
        };

        let result = crate::calc::ssh::check_docker_version(&host_config, profile_kdbx.as_ref()).await;

        match result {
            Ok(version) => {
                let installed = version.is_some();
                let status = if installed {
                    format!("✓ Docker installed ({})", version.as_ref().unwrap())
                } else {
                    "✗ Docker not installed".to_string()
                };
                self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Info, &status);

                self.send_event(SshEvent::DockerCheckCompleted {
                    host: host.clone(),
                    installed,
                    version,
                }).await;
            }
            Err(e) => {
                let _ = self.end_operation_failed("check_docker", &host, log_id, &e).await;
                self.send_event(SshEvent::DockerCheckCompleted {
                    host: host.clone(),
                    installed: false,
                    version: None,
                }).await;
                return Err(e);
            }
        }

        self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Info, "Retrieving Docker status...");
        match crate::calc::ssh::get_docker_status(&host_config, profile_kdbx.as_ref()).await {
            Ok(raw_output) => {
                let _ = self.end_operation_success("check_docker", &host, log_id).await;
                self.send_event(SshEvent::DockerStatusRawRetrieved {
                    host,
                    raw_output,
                }).await;
                Ok(())
            }
            Err(e) => {
                self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Error, format!("Docker status failed: {}", e));
                let _ = self.end_operation_success("check_docker", &host, log_id).await;
                Ok(())
            }
        }
    }

    async fn handle_install_dure(
        &mut self,
        host: String,
        env_config: std::collections::HashMap<String, String>,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        let log_id = self.start_operation("install_dure", &host, true).await.unwrap_or(None);
        self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Info, "Installing Dure mycart...");

        let host_config = match self.load_host_config(&host) {
            Ok(cfg) => cfg,
            Err(e) => {
                let _ = self.end_operation_failed("install_dure", &host, log_id, &e).await;
                self.send_event(SshEvent::DureInstallCompleted {
                    host,
                    success: false,
                    error: Some(e.to_string()),
                }).await;
                return Ok(());
            }
        };

        let domain = env_config.get("DOMAIN").cloned().unwrap_or_else(|| "example.com".to_string());

        match crate::calc::dure_mycart::install_dure_mycart(&host_config, &domain, &env_config, profile_kdbx.as_ref()).await {
            Ok(_progress) => {
                let _ = self.end_operation_success("install_dure", &host, log_id).await;
                self.send_event(SshEvent::DureInstallCompleted {
                    host,
                    success: true,
                    error: None,
                }).await;
            }
            Err(e) => {
                let _ = self.end_operation_failed("install_dure", &host, log_id, &e).await;
                self.send_event(SshEvent::DureInstallCompleted {
                    host,
                    success: false,
                    error: Some(e.to_string()),
                }).await;
            }
        }

        Ok(())
    }

    async fn handle_remove_dure(
        &mut self,
        host: String,
        delete_data: bool,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        let log_id = self.start_operation("remove_dure", &host, true).await.unwrap_or(None);
        self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Info, "Removing Dure mycart...");

        let host_config = match self.load_host_config(&host) {
            Ok(cfg) => cfg,
            Err(e) => {
                let _ = self.end_operation_failed("remove_dure", &host, log_id, &e).await;
                self.send_event(SshEvent::DureRemoveCompleted {
                    host,
                    success: false,
                    error: Some(e.to_string()),
                }).await;
                return Ok(());
            }
        };

        match crate::calc::dure_mycart::remove_dure_mycart(&host_config, delete_data, profile_kdbx.as_ref()).await {
            Ok(_progress) => {
                let _ = self.end_operation_success("remove_dure", &host, log_id).await;
                self.send_event(SshEvent::DureRemoveCompleted {
                    host,
                    success: true,
                    error: None,
                }).await;
            }
            Err(e) => {
                let _ = self.end_operation_failed("remove_dure", &host, log_id, &e).await;
                self.send_event(SshEvent::DureRemoveCompleted {
                    host,
                    success: false,
                    error: Some(e.to_string()),
                }).await;
            }
        }

        Ok(())
    }

    async fn handle_check_dure(
        &mut self,
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        let log_id = self.start_operation("check_dure", &host, false).await.unwrap_or(None);
        self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Info, "Checking Dure status...");

        let host_config = match self.load_host_config(&host) {
            Ok(cfg) => cfg,
            Err(e) => {
                let _ = self.end_operation_failed("check_dure", &host, log_id, &e).await;
                return Err(e);
            }
        };

        let result = crate::calc::dure_mycart::check_dure_mycart(&host_config, profile_kdbx.as_ref()).await;

        let (installed, running, containers) = match result {
            Ok((inst, cont)) => {
                let run = cont.iter().any(|c| c.status.contains("Up"));
                let services: Vec<String> = cont.iter().map(|c| c.name.clone()).collect();

                let status = if inst && run {
                    "✓ Dure installed and running"
                } else if inst {
                    "⚠ Dure installed but not running"
                } else {
                    "✗ Dure not installed"
                };
                self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Info, status);

                self.send_event(SshEvent::DureCheckCompleted {
                    host: host.clone(),
                    installed: inst,
                    running: run,
                    services,
                    containers: cont.clone(),
                }).await;

                (inst, run, cont)
            }
            Err(e) => {
                let _ = self.end_operation_failed("check_dure", &host, log_id, &e).await;
                self.send_event(SshEvent::DureCheckCompleted {
                    host: host.clone(),
                    installed: false,
                    running: false,
                    services: vec![],
                    containers: vec![],
                }).await;
                return Err(e);
            }
        };

        if installed && running {
            self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Info, format!("Retrieving logs for {} containers...", containers.len()));

            let mut container_logs = std::collections::HashMap::new();
            for container in &containers {
                match crate::calc::dure_mycart::get_container_logs(
                    &host_config,
                    &container.name,
                    100,
                    profile_kdbx.as_ref(),
                ).await {
                    Ok(logs) => {
                        container_logs.insert(container.name.clone(), logs);
                    }
                    Err(e) => {
                        self.log_to_tab(&host, crate::viewmodel::logs::LogLevel::Error, format!("Failed to get logs for {}: {}", container.name, e));
                        container_logs.insert(container.name.clone(), vec![format!("Failed to get logs: {}", e)]);
                    }
                }
            }

            self.send_event(SshEvent::DureContainerLogsRetrieved {
                host: host.clone(),
                container_logs,
            }).await;
        }

        let _ = self.end_operation_success("check_dure", &host, log_id).await;
        Ok(())
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

    dure_debug!(" Parsed {} ports and {} args from docker history", ports.len(), args.len());

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

impl SshActor {
    /// Write log to Logs tab (classified by hostname)
    fn log_to_tab(&self, hostname: &str, level: crate::viewmodel::logs::LogLevel, message: impl Into<String>) {
        crate::viewmodel::logs::append_log(Some(hostname.to_string()), level, message.into());
    }

    /// Start operation with optional history logging
    /// Returns log_id for CUD operations, None for Read operations
    async fn start_operation(
        &mut self,
        op_name: &str,
        hostname: &str,
        is_cud: bool,
    ) -> anyhow::Result<Option<i64>> {
        // Always log to Logs tab
        self.log_to_tab(hostname, crate::viewmodel::logs::LogLevel::Info, format!("▶ Starting {}", op_name));

        // Only log to Operations history if CUD operation
        if is_cud {
            let log_id = self.repository.log_operation(
                hostname,
                op_name,
                "ssh",
                None,
            ).await?;
            Ok(Some(log_id))
        } else {
            Ok(None)
        }
    }

    /// End operation successfully
    async fn end_operation_success(
        &mut self,
        op_name: &str,
        hostname: &str,
        log_id: Option<i64>,
    ) -> anyhow::Result<()> {
        // Always log to Logs tab
        self.log_to_tab(hostname, crate::viewmodel::logs::LogLevel::Info, format!("✓ {} completed", op_name));

        // Mark history if CUD operation
        if let Some(id) = log_id {
            self.repository.mark_success(id).await?;
        }
        Ok(())
    }

    /// End operation with error
    async fn end_operation_failed(
        &mut self,
        op_name: &str,
        hostname: &str,
        log_id: Option<i64>,
        error: &anyhow::Error,
    ) -> anyhow::Result<()> {
        let error_msg = format!("{:#}", error);

        // Always log to Logs tab
        self.log_to_tab(hostname, crate::viewmodel::logs::LogLevel::Error, format!("✗ {} failed: {}", op_name, error_msg));

        // Mark history if CUD operation
        if let Some(id) = log_id {
            self.repository.mark_failed(id, &error_msg).await?;
        }
        Ok(())
    }
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

    #[test]
    fn test_handle_refresh_creates_refresh_event() {
        // Test that handle_refresh returns a RefreshCompleted event
        smol::block_on(async {
            let (tx, _rx) = smol::channel::unbounded();
            let mut actor = SshActor::new(smol::channel::unbounded::<SshCommand>().1, tx);

            // Call handle_refresh with a test host
            let event = actor.handle_refresh("test-host".to_string()).await;

            // Verify event type
            match event {
                SshEvent::RefreshCompleted { host, .. } => {
                    assert_eq!(host, "test-host");
                }
                _ => panic!("Expected RefreshCompleted event"),
            }
        });
    }

    #[test]
    fn test_handle_refresh_event_structure() {
        // Test that RefreshCompleted event has all required fields
        smol::block_on(async {
            let (tx, _rx) = smol::channel::unbounded();
            let mut actor = SshActor::new(smol::channel::unbounded::<SshCommand>().1, tx, None);

            let event = actor.handle_refresh("localhost".to_string(), None).await;

            match event {
                SshEvent::RefreshCompleted {
                    host,
                    ssh_connected,
                    host_installed,
                    docker_installed,
                    dure_installed,
                    ..
                } => {
                    assert_eq!(host, "localhost");
                    // Fields should be initialized (placeholder values)
                    let _ = (ssh_connected, host_installed, docker_installed, dure_installed);
                }
                _ => panic!("Expected RefreshCompleted event"),
            }
        });
    }
}
