//! ViewModel layer for actor-based MVVM architecture

pub mod common;
pub mod deltachat;
pub mod io;
pub mod ns;
pub mod platform;
pub mod runtime;
pub mod ssh;
pub mod wss;

#[cfg(test)]
mod tests;

pub use common::*;

use smol::channel::{Receiver, Sender};
use std::collections::{HashMap, VecDeque};

/// ViewModel coordinates actors and exposes unified API for UI/CLI
pub struct ViewModel {
    // Actor communication channels
    platform_tx: Sender<platform::PlatformCommand>,
    ssh_tx: Sender<ssh::SshCommand>,
    ns_tx: Sender<ns::NsCommand>,
    wss_tx: Sender<wss::WssCommand>,
    deltachat_tx: Sender<deltachat::DeltaChatCommand>,

    // Unified event receiver
    event_rx: Receiver<ViewModelEvent>,

    // Transient state
    state: ViewModelState,

    // Runtime handle
    runtime_handle: Option<RuntimeHandle>,

    // Optional egui context (for GUI mode)
    #[cfg(feature = "gui")]
    egui_ctx: Option<egui::Context>,
}

enum RuntimeHandle {
    #[cfg(not(target_arch = "wasm32"))]
    Native(std::thread::JoinHandle<()>),

    #[cfg(target_arch = "wasm32")]
    Wasm(WasmExecutorHandle),
}

#[cfg(target_arch = "wasm32")]
struct WasmExecutorHandle;

#[derive(Default)]
pub struct ViewModelState {
    pub active_operations: HashMap<String, OperationProgress>,
    pub recent_errors: VecDeque<ErrorRecord>,
    pub wss_connections: HashMap<String, WssConnectionInfo>,
    #[cfg(feature = "gui")]
    pub textures: HashMap<String, egui::TextureHandle>,
}

pub struct OperationProgress {
    pub operation: String,
    pub progress: f32,
    pub status: String,
    pub started_at: std::time::Instant,
}

pub struct ErrorRecord {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation: String,
    pub error: String,
    pub actor: String,
}

#[derive(Clone, Debug)]
pub struct WssConnectionInfo {
    pub connection_id: String,
    pub url: String,
    pub connected_at: chrono::DateTime<chrono::Utc>,
    pub status: String,
}

impl ViewModel {
    /// Create ViewModel for GUI mode (Desktop/Android)
    #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
    pub fn new(ctx: egui::Context) -> Self {
        let (platform_tx, platform_rx) = smol::channel::unbounded();
        let (ssh_tx, ssh_rx) = smol::channel::unbounded();
        let (ns_tx, ns_rx) = smol::channel::unbounded();
        let (wss_tx, wss_rx) = smol::channel::unbounded();
        let (deltachat_tx, deltachat_rx) = smol::channel::unbounded();
        let (event_tx, event_rx) = smol::channel::unbounded();

        // Spawn background thread with smol executor
        let runtime_handle = std::thread::spawn(move || {
            smol::block_on(async {
                log::info!("ViewModel runtime started");

                // Create actors
                let platform_actor = platform::PlatformActor::new(platform_rx, event_tx.clone());
                let ssh_actor = ssh::SshActor::new(ssh_rx, event_tx.clone());
                let ns_actor = ns::NsActor::new(ns_rx, event_tx.clone());
                let wss_actor = wss::WssActor::new(wss_rx, event_tx.clone());

                // DeltaChat actor (uses netwatch stub on OpenBSD)
                let db_path = std::env::var("HOME")
                    .map(|h| std::path::PathBuf::from(h).join(".local/share/dure/deltachat-default.db"))
                    .unwrap_or_else(|_| std::path::PathBuf::from("deltachat-default.db"));
                let deltachat_actor = deltachat::DeltaChatActor::new(deltachat_rx, event_tx.clone(), db_path);

                // Run all actors concurrently
                smol::spawn(platform_actor.run()).detach();
                smol::spawn(ssh_actor.run()).detach();
                smol::spawn(ns_actor.run()).detach();
                smol::spawn(wss_actor.run()).detach();
                smol::spawn(deltachat_actor.run()).detach();

                // Keep thread alive
                std::future::pending::<()>().await
            })
        });

        Self {
            platform_tx,
            ssh_tx,
            ns_tx,
            wss_tx,
            deltachat_tx,
            event_rx,
            state: ViewModelState::default(),
            runtime_handle: Some(RuntimeHandle::Native(runtime_handle)),
            egui_ctx: Some(ctx),
        }
    }

    /// Create ViewModel for CLI mode (headless)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_headless() -> Self {
        let (platform_tx, platform_rx) = smol::channel::unbounded();
        let (ssh_tx, ssh_rx) = smol::channel::unbounded();
        let (ns_tx, ns_rx) = smol::channel::unbounded();
        let (wss_tx, wss_rx) = smol::channel::unbounded();
        let (deltachat_tx, deltachat_rx) = smol::channel::unbounded();
        let (event_tx, event_rx) = smol::channel::unbounded();

        let runtime_handle = std::thread::spawn(move || {
            smol::block_on(async {
                log::info!("ViewModel runtime started (headless)");

                let platform_actor = platform::PlatformActor::new(platform_rx, event_tx.clone());
                let ssh_actor = ssh::SshActor::new(ssh_rx, event_tx.clone());
                let ns_actor = ns::NsActor::new(ns_rx, event_tx.clone());
                let wss_actor = wss::WssActor::new(wss_rx, event_tx.clone());

                // DeltaChat actor (uses netwatch stub on OpenBSD)
                let db_path = std::env::var("HOME")
                    .map(|h| std::path::PathBuf::from(h).join(".local/share/dure/deltachat-default.db"))
                    .unwrap_or_else(|_| std::path::PathBuf::from("deltachat-default.db"));
                let deltachat_actor = deltachat::DeltaChatActor::new(deltachat_rx, event_tx.clone(), db_path);

                smol::spawn(platform_actor.run()).detach();
                smol::spawn(ssh_actor.run()).detach();
                smol::spawn(ns_actor.run()).detach();
                smol::spawn(wss_actor.run()).detach();
                smol::spawn(deltachat_actor.run()).detach();

                std::future::pending::<()>().await
            })
        });

        Self {
            platform_tx,
            ssh_tx,
            ns_tx,
            wss_tx,
            deltachat_tx,
            event_rx,
            state: ViewModelState::default(),
            runtime_handle: Some(RuntimeHandle::Native(runtime_handle)),
            #[cfg(feature = "gui")]
            egui_ctx: None,
        }
    }

    /// Create ViewModel for WASM (browser)
    #[cfg(target_arch = "wasm32")]
    pub fn new_wasm() -> Self {
        use wasm_bindgen_futures::spawn_local;

        let (platform_tx, platform_rx) = smol::channel::unbounded();
        let (ssh_tx, ssh_rx) = smol::channel::unbounded();
        let (ns_tx, ns_rx) = smol::channel::unbounded();
        let (wss_tx, wss_rx) = smol::channel::unbounded();
        let (deltachat_tx, deltachat_rx) = smol::channel::unbounded();
        let (event_tx, event_rx) = smol::channel::unbounded();

        // Spawn actors in Web Worker context
        spawn_local(async move {
            log::info!("ViewModel runtime started (WASM)");

            let platform_actor = platform::PlatformActor::new(platform_rx, event_tx.clone());
            let ns_actor = ns::NsActor::new(ns_rx, event_tx.clone());
            let wss_actor = wss::WssActor::new(wss_rx, event_tx.clone());

            // DeltaChat actor
            #[cfg(not(target_os = "openbsd"))]
            let db_path = std::path::PathBuf::from("deltachat-default.db");
            #[cfg(not(target_os = "openbsd"))]
            let deltachat_actor = deltachat::DeltaChatActor::new(deltachat_rx, event_tx.clone(), db_path);

            // SSH disabled in WASM (no native SSH in browser)
            drop(ssh_rx);

            // Run actors concurrently
            futures::join!(
                platform_actor.run(),
                ns_actor.run(),
                wss_actor.run(),
                deltachat_actor.run(),
            );
        });

        Self {
            platform_tx,
            ssh_tx,
            ns_tx,
            wss_tx,
            deltachat_tx,
            event_rx,
            state: ViewModelState::default(),
            runtime_handle: None,
            #[cfg(feature = "gui")]
            egui_ctx: None,
        }
    }

    /// Poll for events and update state (GUI mode)
    #[cfg(feature = "gui")]
    pub fn poll_events(&mut self, ctx: &egui::Context) -> Vec<ViewModelEvent> {
        let mut events = Vec::new();

        while let Ok(event) = self.event_rx.try_recv() {
            eprintln!("🔍 ViewModel: Received event: {:?}", event);
            self.apply_event(&event, Some(ctx));
            events.push(event);
        }

        if !events.is_empty() {
            eprintln!(
                "🔍 ViewModel: Collected {} events, requesting repaint",
                events.len()
            );
            ctx.request_repaint();
        }

        events
    }

    /// Poll events without egui context (CLI mode)
    pub fn poll_events_headless(&mut self) -> Vec<ViewModelEvent> {
        let mut events = Vec::new();

        while let Ok(event) = self.event_rx.try_recv() {
            #[cfg(feature = "gui")]
            self.apply_event(&event, None);
            #[cfg(not(feature = "gui"))]
            {
                // Minimal event processing for headless mode
                let _ = event; // Suppress unused variable warning
            }
            events.push(event);
        }

        events
    }

    #[cfg(feature = "gui")]
    fn apply_event(&mut self, _event: &ViewModelEvent, _ctx: Option<&egui::Context>) {
        // TODO: implement in Week 2
    }

    // State accessors
    pub fn active_operations(&self) -> &HashMap<String, OperationProgress> {
        &self.state.active_operations
    }

    pub fn recent_errors(&self) -> &VecDeque<ErrorRecord> {
        &self.state.recent_errors
    }

    // Platform commands
    pub fn create_vm(
        &self,
        platform_name: String,
        vm_name: String,
        zone: String,
        machine_type: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::CreateVM {
                platform_name,
                vm_name,
                zone,
                machine_type,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn list_vms(&self, platform_name: String) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::ListVMs { platform_name })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn delete_vm(
        &self,
        platform_name: String,
        vm_name: String,
        zone: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::DeleteVM {
                platform_name,
                vm_name,
                zone,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn restart_vm(
        &self,
        platform_name: String,
        vm_name: String,
        zone: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::RestartVM {
                platform_name,
                vm_name,
                zone,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn regenerate_vm(
        &self,
        platform_name: String,
        vm_name: String,
        zone: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::RegenerateVM {
                platform_name,
                vm_name,
                zone,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn update_firewall(&self, platform_name: String, allow_ip: String) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::UpdateFirewall {
                platform_name,
                allow_ip,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn fetch_billing(
        &self,
        platform_name: String,
        project_id: String,
        dataset: String,
        table: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::FetchBilling {
                platform_name,
                project_id,
                dataset,
                table,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn list_projects(&self, platform_name: String) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::ListProjects { platform_name })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn select_project(&self, platform_name: String, project_id: String) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::SelectProject {
                platform_name,
                project_id,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn start_oauth(&self, platform_name: String) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::StartOAuth { platform_name })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn complete_oauth(&self, platform_name: String, auth_code: String) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::CompleteOAuth {
                platform_name,
                auth_code,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn add_platform(
        &self,
        platform_type: String,
        oauth_access_token: Option<String>,
        oauth_refresh_token: Option<String>,
        oauth_token_expiry: Option<i64>,
        connected_email: Option<String>,
        selected_project_id: Option<String>,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::AddPlatform {
                platform_type,
                oauth_access_token,
                oauth_refresh_token,
                oauth_token_expiry,
                connected_email,
                selected_project_id,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn delete_platform(
        &self,
        platform_name: String,
        delete_options: platform::DeleteOptions,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::DeletePlatform {
                platform_name,
                delete_options,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    // SSH commands
    pub fn add_ssh_host(
        &self,
        name: String,
        host: String,
        port: u16,
        user: String,
        ssh_key_path: String,
    ) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::AddHost {
                name,
                host,
                port,
                user,
                ssh_key_path,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn delete_ssh_host(&self, name: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::DeleteHost { name })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn list_ssh_hosts(&self) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::ListHosts)
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn test_ssh_connection(&self, name: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::TestConnection { name })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn init_ssh_host(&self, name: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::InitHost { name })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn docker_pull(&self, host_name: String, image: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::DockerPull { host_name, image })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn docker_run(
        &self,
        host_name: String,
        image: String,
        container_name: String,
        ports: Vec<(u16, u16)>,
        env: Vec<(String, String)>,
    ) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::DockerRun {
                host_name,
                image,
                container_name,
                ports,
                env,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn docker_stop(&self, host_name: String, container_name: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::DockerStop {
                host_name,
                container_name,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn port_open(&self, host_name: String, port: u16, protocol: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::PortOpen {
                host_name,
                port,
                protocol,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn port_close(&self, host_name: String, port: u16, protocol: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::PortClose {
                host_name,
                port,
                protocol,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    // Service management commands
    pub fn get_linux_status(&self, host: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::GetLinuxStatus { name: host })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn install_docker(&self, host: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::InstallDocker { name: host })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn get_docker_status(&self, host: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::GetDockerStatus { name: host })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn uninstall_docker(&self, host: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::UninstallDocker { name: host })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn install_ansible(&self, host: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::InstallAnsible { name: host })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn get_ansible_status(&self, host: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::GetAnsibleStatus { name: host })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn uninstall_ansible(&self, host: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::UninstallAnsible { name: host })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    /// Check if SSH host is reachable (TCP port check with timeout)
    pub fn check_host_health(&self, name: String, timeout_secs: u8) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::CheckHostHealth {
                name,
                timeout_secs,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    // Docker Lifecycle Management

    /// Inspect Docker image by pulling and analyzing history
    pub fn inspect_docker_image(
        &self,
        host: String,
        image: String,
        tag: String,
    ) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::InspectDockerImage {
                host_name: host,
                image,
                tag,
            })?;
        Ok(())
    }

    /// Install Docker image on host (auto-installs Docker if needed)
    pub fn install_docker_image(
        &self,
        host: String,
        container_name: String,
        image: String,
        tag: String,
        ports: Vec<(u16, u16)>,
        env: Vec<(String, String)>,
    ) {
        let _ = self
            .ssh_tx
            .send_blocking(ssh::SshCommand::InstallDockerImage {
                host_name: host,
                container_name,
                image,
                tag,
                ports,
                env,
            });
    }

    /// Remove Docker container from host
    pub fn remove_docker_container(&self, host: String, container_name: String) {
        let _ = self
            .ssh_tx
            .send_blocking(ssh::SshCommand::RemoveDockerContainer {
                host_name: host,
                container_name,
            });
    }

    /// Remove multiple Docker containers in batch
    pub fn remove_docker_containers(
        &self,
        host: String,
        container_names: Vec<String>,
    ) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::RemoveDockerContainers {
                host_name: host,
                container_names,
            })?;
        Ok(())
    }

    /// List Docker containers on host
    pub fn list_docker_containers(&self, host: String) {
        let _ = self
            .ssh_tx
            .send_blocking(ssh::SshCommand::ListDockerContainers { host_name: host });
    }

    // Ansible Lifecycle Management

    /// Validate Ansible role and fetch metadata from Galaxy
    pub fn validate_ansible_role(&self, role: String) {
        let _ = self
            .ssh_tx
            .send_blocking(ssh::SshCommand::ValidateAnsibleRole { role });
    }

    /// Install Ansible role on host (auto-installs Ansible if needed)
    pub fn install_ansible_role(
        &self,
        host: String,
        instance_name: String,
        galaxy_name: String,
        variables: Vec<(String, String)>,
        ports: Vec<u16>,
    ) {
        let _ = self
            .ssh_tx
            .send_blocking(ssh::SshCommand::InstallAnsibleRole {
                host_name: host,
                instance_name,
                galaxy_name,
                variables,
                ports,
            });
    }

    /// Remove Ansible role from host
    pub fn remove_ansible_role(&self, host: String, instance_name: String) {
        let _ = self
            .ssh_tx
            .send_blocking(ssh::SshCommand::RemoveAnsibleRole {
                host_name: host,
                instance_name,
            });
    }

    /// List Ansible roles installed on host
    pub fn list_ansible_roles(&self, host: String) {
        let _ = self
            .ssh_tx
            .send_blocking(ssh::SshCommand::ListAnsibleRoles { host_name: host });
    }

    // Dure-WSS Lifecycle Management

    /// Install Dure-WSS service on host
    pub fn install_dure_wss(
        &self,
        host: String,
        domain: String,
        email: String,
        channel: String,
        variant: String,
    ) {
        let _ = self
            .ssh_tx
            .send_blocking(ssh::SshCommand::InstallDureWssService {
                host_name: host,
                domain,
                email,
                channel,
                variant,
            });
    }

    /// Start Dure-WSS service
    pub fn start_dure_wss(&self, host: String) {
        let _ = self
            .ssh_tx
            .send_blocking(ssh::SshCommand::StartDureWss { host_name: host });
    }

    /// Stop Dure-WSS service
    pub fn stop_dure_wss(&self, host: String) {
        let _ = self
            .ssh_tx
            .send_blocking(ssh::SshCommand::StopDureWss { host_name: host });
    }

    /// Restart Dure-WSS service
    pub fn restart_dure_wss(&self, host: String) {
        let _ = self
            .ssh_tx
            .send_blocking(ssh::SshCommand::RestartDureWss { host_name: host });
    }

    /// Uninstall Dure-WSS service
    pub fn uninstall_dure_wss(&self, host: String) {
        let _ = self
            .ssh_tx
            .send_blocking(ssh::SshCommand::UninstallDureWss { host_name: host });
    }

    // Legacy compatibility stubs for old UI code (will be removed in Tasks 5-7)
    #[deprecated(note = "Use install_dure_wss with full parameters")]
    pub fn install_dure_wss_legacy(&self, _host: String) -> anyhow::Result<()> {
        Ok(())
    }

    #[deprecated(note = "Use start_dure_wss/stop_dure_wss methods instead")]
    pub fn get_dure_wss_status(&self, _host: String) -> anyhow::Result<()> {
        Ok(())
    }

    // NS commands
    pub fn add_dns_provider(
        &self,
        name: String,
        provider_type: String,
        api_token: String,
    ) -> anyhow::Result<()> {
        self.ns_tx
            .send_blocking(ns::NsCommand::AddProvider {
                name,
                provider_type,
                api_token,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn delete_dns_provider(&self, name: String) -> anyhow::Result<()> {
        self.ns_tx
            .send_blocking(ns::NsCommand::DeleteProvider { name })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn list_dns_providers(&self) -> anyhow::Result<()> {
        self.ns_tx
            .send_blocking(ns::NsCommand::ListProviders)
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn add_dns_domain(&self, provider_name: String, domain: String) -> anyhow::Result<()> {
        self.ns_tx
            .send_blocking(ns::NsCommand::AddDomain {
                provider_name,
                domain,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn delete_dns_domain(&self, provider_name: String, domain: String) -> anyhow::Result<()> {
        self.ns_tx
            .send_blocking(ns::NsCommand::DeleteDomain {
                provider_name,
                domain,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn list_dns_domains(&self, provider_name: String) -> anyhow::Result<()> {
        self.ns_tx
            .send_blocking(ns::NsCommand::ListDomains { provider_name })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn add_dns_record(
        &self,
        provider_name: String,
        domain: String,
        record_type: String,
        name: String,
        value: String,
        ttl: u32,
    ) -> anyhow::Result<()> {
        self.ns_tx
            .send_blocking(ns::NsCommand::AddRecord {
                provider_name,
                domain,
                record_type,
                name,
                value,
                ttl,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn delete_dns_record(
        &self,
        provider_name: String,
        domain: String,
        name: String,
        record_type: String,
    ) -> anyhow::Result<()> {
        self.ns_tx
            .send_blocking(ns::NsCommand::DeleteRecord {
                provider_name,
                domain,
                name,
                record_type,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn list_dns_records(&self, provider_name: String, domain: String) -> anyhow::Result<()> {
        self.ns_tx
            .send_blocking(ns::NsCommand::ListRecords {
                provider_name,
                domain,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }
}
