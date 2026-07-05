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

    async fn add_host(&mut self, name: String, host: String, port: u16, user: String, ssh_key_path: String) -> anyhow::Result<()> {
        self.send_progress("add_host", 0.5, "Adding SSH host...").await;

        runtime::unblock({
            let name = name.clone();
            move || crate::calc::db::save_ssh_host(&name, &host, port, &user, &ssh_key_path)
        }).await?;

        self.send_event(SshEvent::HostAdded { name }).await;
        Ok(())
    }

    async fn delete_host(&mut self, name: String) -> anyhow::Result<()> {
        self.send_progress("delete_host", 0.5, "Deleting SSH host...").await;

        runtime::unblock({
            let name = name.clone();
            move || crate::calc::db::delete_ssh_host(&name)
        }).await?;

        self.send_event(SshEvent::HostDeleted { name }).await;
        Ok(())
    }

    async fn list_hosts(&mut self) -> anyhow::Result<()> {
        self.send_progress("list_hosts", 0.5, "Loading SSH hosts...").await;

        let hosts = runtime::unblock(|| {
            crate::calc::db::load_ssh_hosts()
        }).await?;

        let host_infos: Vec<SshHostInfo> = hosts.into_iter().map(|h| SshHostInfo {
            name: h.name,
            host: h.host,
            port: h.port,
            user: h.user,
        }).collect();

        self.send_event(SshEvent::HostsListed { hosts: host_infos }).await;
        Ok(())
    }

    async fn test_connection(&mut self, name: String) -> anyhow::Result<()> {
        self.send_progress("test_connection", 0.5, "Testing SSH connection...").await;

        let start = std::time::Instant::now();
        let result = runtime::unblock({
            let name = name.clone();
            move || crate::calc::ssh::test_connection(&name)
        }).await;

        let latency_ms = start.elapsed().as_millis() as u64;

        match result {
            Ok(_) => {
                self.send_event(SshEvent::ConnectionTested {
                    name,
                    success: true,
                    latency_ms: Some(latency_ms),
                }).await;
            }
            Err(e) => {
                self.send_event(SshEvent::ConnectionTested {
                    name,
                    success: false,
                    latency_ms: None,
                }).await;
                return Err(e);
            }
        }

        Ok(())
    }

    async fn docker_pull(&mut self, host_name: String, image: String) -> anyhow::Result<()> {
        self.send_progress("docker_pull", 0.0, "Pulling Docker image...").await;

        runtime::unblock({
            let host_name = host_name.clone();
            let image = image.clone();
            move || crate::calc::ssh::docker_pull(&host_name, &image)
        }).await?;

        self.send_event(SshEvent::DockerImagePulled { host_name, image }).await;
        Ok(())
    }

    async fn docker_run(&mut self, host_name: String, image: String, container_name: String, ports: Vec<(u16, u16)>, env: Vec<(String, String)>) -> anyhow::Result<()> {
        self.send_progress("docker_run", 0.0, "Starting Docker container...").await;

        runtime::unblock({
            let host_name = host_name.clone();
            let container_name = container_name.clone();
            move || crate::calc::ssh::docker_run(&host_name, &image, &container_name, &ports, &env)
        }).await?;

        self.send_event(SshEvent::DockerContainerStarted { host_name, container_name }).await;
        Ok(())
    }

    async fn docker_stop(&mut self, host_name: String, container_name: String) -> anyhow::Result<()> {
        self.send_progress("docker_stop", 0.5, "Stopping Docker container...").await;

        runtime::unblock({
            let host_name = host_name.clone();
            let container_name = container_name.clone();
            move || crate::calc::ssh::docker_stop(&host_name, &container_name)
        }).await?;

        self.send_event(SshEvent::DockerContainerStopped { host_name, container_name }).await;
        Ok(())
    }

    async fn docker_list(&mut self, host_name: String) -> anyhow::Result<()> {
        self.send_progress("docker_list", 0.5, "Listing Docker containers...").await;

        let containers = runtime::unblock({
            let host_name = host_name.clone();
            move || crate::calc::ssh::docker_list(&host_name)
        }).await?;

        let container_infos: Vec<DockerContainer> = containers.into_iter().map(|c| DockerContainer {
            name: c.name,
            image: c.image,
            status: c.status,
        }).collect();

        self.send_event(SshEvent::DockerContainersListed {
            host_name,
            containers: container_infos,
        }).await;
        Ok(())
    }

    async fn port_open(&mut self, host_name: String, port: u16, protocol: String) -> anyhow::Result<()> {
        self.send_progress("port_open", 0.5, "Opening firewall port...").await;

        runtime::unblock({
            let host_name = host_name.clone();
            let protocol = protocol.clone();
            move || crate::calc::nft::port_open(&host_name, port, &protocol)
        }).await?;

        self.send_event(SshEvent::PortOpened { host_name, port, protocol }).await;
        Ok(())
    }

    async fn port_close(&mut self, host_name: String, port: u16, protocol: String) -> anyhow::Result<()> {
        self.send_progress("port_close", 0.5, "Closing firewall port...").await;

        runtime::unblock({
            let host_name = host_name.clone();
            let protocol = protocol.clone();
            move || crate::calc::nft::port_close(&host_name, port, &protocol)
        }).await?;

        self.send_event(SshEvent::PortClosed { host_name, port, protocol }).await;
        Ok(())
    }

    async fn port_list(&mut self, host_name: String) -> anyhow::Result<()> {
        self.send_progress("port_list", 0.5, "Listing open ports...").await;

        let open_ports = runtime::unblock({
            let host_name = host_name.clone();
            move || crate::calc::nft::port_list(&host_name)
        }).await?;

        self.send_event(SshEvent::PortsListed { host_name, open_ports }).await;
        Ok(())
    }

    async fn deploy_dure_wss(&mut self, host_name: String, domain: String, acme_email: String) -> anyhow::Result<()> {
        self.send_progress("deploy_dure_wss", 0.0, "Deploying Dure WSS service...").await;

        runtime::unblock({
            let host_name = host_name.clone();
            let domain = domain.clone();
            move || crate::calc::wss::deploy_wss(&host_name, &domain, &acme_email)
        }).await?;

        self.send_progress("deploy_dure_wss", 0.9, "Checking service status...").await;

        let status = runtime::unblock({
            let host_name = host_name.clone();
            move || crate::calc::wss::check_service_status(&host_name)
        }).await?;

        self.send_event(SshEvent::DureWssDeployed {
            host_name,
            domain,
            service_status: status,
        }).await;

        Ok(())
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
        let _ = self.event_tx.send(ViewModelEvent::Ssh(event)).await;
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
