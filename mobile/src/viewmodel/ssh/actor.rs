//! SSH actor implementation

use super::{SshCommand, SshEvent, SshHostInfo, DockerContainer};
use crate::viewmodel::{ViewModelEvent, runtime};
use smol::channel::{Receiver, Sender};

pub struct SshActor {
    command_rx: Receiver<SshCommand>,
    event_tx: Sender<ViewModelEvent>,
}

impl SshActor {
    pub fn new(command_rx: Receiver<SshCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self { command_rx, event_tx }
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
            SshCommand::AddHost { name, host, port, user, ssh_key_path } => {
                self.add_host(name, host, port, user, ssh_key_path).await
            }
            SshCommand::DeleteHost { name } => {
                self.delete_host(name).await
            }
            SshCommand::ListHosts => {
                self.list_hosts().await
            }
            SshCommand::TestConnection { name } => {
                eprintln!("🔍 SSH Actor: Handling TestConnection for '{}'", name);
                self.test_connection(name).await
            }
            SshCommand::DockerPull { host_name, image } => {
                self.docker_pull(host_name, image).await
            }
            SshCommand::DockerRun { host_name, image, container_name, ports, env } => {
                self.docker_run(host_name, image, container_name, ports, env).await
            }
            SshCommand::DockerStop { host_name, container_name } => {
                self.docker_stop(host_name, container_name).await
            }
            SshCommand::DockerList { host_name } => {
                self.docker_list(host_name).await
            }
            SshCommand::PortOpen { host_name, port, protocol } => {
                self.port_open(host_name, port, protocol).await
            }
            SshCommand::PortClose { host_name, port, protocol } => {
                self.port_close(host_name, port, protocol).await
            }
            SshCommand::PortList { host_name } => {
                self.port_list(host_name).await
            }
            SshCommand::DeployDureWss { host_name, domain, acme_email } => {
                self.deploy_dure_wss(host_name, domain, acme_email).await
            }
        };

        if let Err(e) = result {
            self.send_error(&operation, e).await;
        }

        Ok(())
    }

    async fn add_host(&mut self, name: String, host: String, port: u16, _user: String, ssh_key_path: String) -> anyhow::Result<()> {
        self.send_progress("add_host", 0.5, "Adding SSH host...").await;

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
                    private_key_path: if ssh_key_path.is_empty() { None } else { Some(ssh_key_path) },
                    keyring_domain: None,
                    port,
                    initialized: false,
                    last_status: None,
                };

                app_config.ssh_hosts.push(ssh_host);
                app_config.save(&config_path)?;
                Ok(())
            }
        }).await?;

        self.send_event(SshEvent::HostAdded { name }).await;
        Ok(())
    }

    async fn delete_host(&mut self, name: String) -> anyhow::Result<()> {
        self.send_progress("delete_host", 0.5, "Deleting SSH host...").await;

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
        }).await?;

        self.send_event(SshEvent::HostDeleted { name }).await;
        Ok(())
    }

    async fn list_hosts(&mut self) -> anyhow::Result<()> {
        self.send_progress("list_hosts", 0.5, "Loading SSH hosts...").await;

        let host_infos = runtime::unblock(|| -> anyhow::Result<Vec<SshHostInfo>> {
            let config_path = Self::get_config_path()?;
            let app_config = crate::config::AppConfig::load_or_default(&config_path);

            let hosts = app_config.ssh_hosts.into_iter().map(|h| SshHostInfo {
                name: h.host.clone(),
                host: h.host,
                port: h.port,
                user: String::new(), // Not stored separately
            }).collect();

            Ok(hosts)
        }).await?;

        self.send_event(SshEvent::HostsListed { hosts: host_infos }).await;
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
        self.send_progress("test_connection", 0.5, "Testing SSH connection...").await;

        let start = std::time::Instant::now();

        // Load host config first (blocking operation)
        eprintln!("🔍 SSH Actor: Loading host config...");
        let host_config = runtime::unblock({
            let name = name.clone();
            move || -> anyhow::Result<crate::config::SshHostConfig> {
                let config_path = Self::get_config_path()?;
                let app_config = crate::config::AppConfig::load_or_default(&config_path);

                let host_config = app_config.ssh_hosts.into_iter()
                    .find(|h| h.host == name)
                    .ok_or_else(|| anyhow::anyhow!("SSH host '{}' not found", name))?;

                eprintln!("🔍 SSH Actor: Found host config for '{}'", host_config.host);
                Ok(host_config)
            }
        }).await?;

        // Test connection (async operation - russh uses tokio internally)
        eprintln!("🔍 SSH Actor: Starting SSH connection test to {}:{}...", host_config.host, host_config.port);
        let result = async_compat::Compat::new(crate::calc::ssh::test_connection(&host_config))
            .await;

        let latency_ms = start.elapsed().as_millis() as u64;
        eprintln!("🔍 SSH Actor: Connection test completed in {}ms", latency_ms);

        match result {
            Ok(conn_result) => {
                eprintln!("✓ SSH Actor: Connection test succeeded: {}", conn_result.success);
                self.send_event(SshEvent::ConnectionTested {
                    name,
                    success: conn_result.success,
                    latency_ms: Some(latency_ms),
                }).await;
                Ok(())
            }
            Err(e) => {
                eprintln!("✗ SSH Actor: Connection test failed: {}", e);
                self.send_event(SshEvent::ConnectionTested {
                    name,
                    success: false,
                    latency_ms: None,
                }).await;
                Err(e)
            }
        }
    }

    async fn docker_pull(&mut self, _host_name: String, _image: String) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Docker management not yet implemented in ViewModel"))
    }

    async fn docker_run(&mut self, _host_name: String, _image: String, _container_name: String, _ports: Vec<(u16, u16)>, _env: Vec<(String, String)>) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Docker management not yet implemented in ViewModel"))
    }

    async fn docker_stop(&mut self, _host_name: String, _container_name: String) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Docker management not yet implemented in ViewModel"))
    }

    async fn docker_list(&mut self, _host_name: String) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Docker management not yet implemented in ViewModel"))
    }

    async fn port_open(&mut self, _host_name: String, _port: u16, _protocol: String) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Port management not yet implemented in ViewModel"))
    }

    async fn port_close(&mut self, _host_name: String, _port: u16, _protocol: String) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Port management not yet implemented in ViewModel"))
    }

    async fn port_list(&mut self, _host_name: String) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Port management not yet implemented in ViewModel"))
    }

    async fn deploy_dure_wss(&mut self, _host_name: String, _domain: String, _acme_email: String) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("Dure WSS deployment not yet implemented in ViewModel"))
    }

    async fn send_progress(&self, operation: &str, progress: f32, status: &str) {
        let _ = self.event_tx.send(ViewModelEvent::Ssh(
            SshEvent::Progress {
                operation: operation.to_string(),
                progress,
                status: status.to_string(),
            }
        )).await;
    }

    async fn send_event(&self, event: SshEvent) {
        eprintln!("🔍 SSH Actor: Sending event: {:?}", event);
        match self.event_tx.send(ViewModelEvent::Ssh(event.clone())).await {
            Ok(_) => eprintln!("✓ SSH Actor: Event sent successfully"),
            Err(e) => eprintln!("✗ SSH Actor: Failed to send event: {}", e),
        }
    }

    async fn send_error(&self, operation: &str, error: anyhow::Error) {
        let _ = self.event_tx.send(ViewModelEvent::Ssh(
            SshEvent::Error {
                operation: operation.to_string(),
                error: format!("{:#}", error),
            }
        )).await;
    }
}
