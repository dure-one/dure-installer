//! ViewModel layer for actor-based MVVM architecture

pub mod common;
pub mod dns;
pub mod io;
pub mod logs;
pub mod mycart;
pub mod platform;
pub mod runtime;
pub mod ssh;

#[cfg(test)]
mod tests;

pub use common::*;

use crate::calc::profile::ProfileContext;
use crate::{dure_debug, dure_error, dure_info, dure_warn};
use smol::channel::{Receiver, Sender};
use std::collections::{HashMap, VecDeque};

/// ViewModel coordinates actors and exposes unified API for UI/CLI
pub struct ViewModel {
    // Actor communication channels
    platform_tx: Sender<platform::PlatformCommand>,
    drawer_tx: Sender<platform::DrawerCommand>,
    ssh_tx: Sender<ssh::SshCommand>,
    logs_tx: Sender<logs::LogCommand>,
    opslog_tx: Sender<logs::LogsCommand>,
    dns_tx: Sender<dns::DnsCommand>,
    mycart_tx: Sender<mycart::MycartCommand>,

    // Unified event receiver
    event_rx: Receiver<ViewModelEvent>,

    // Transient state
    state: ViewModelState,

    // Runtime handle
    runtime_handle: Option<RuntimeHandle>,

    // Optional egui context (for GUI mode)
    #[cfg(feature = "gui")]
    egui_ctx: Option<egui::Context>,

    // Profile context (for profile-aware operations)
    profile: Option<ProfileContext>,
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


impl ViewModel {
    /// Create ViewModel for GUI mode (Desktop/Android)
    #[cfg(all(feature = "gui", not(target_arch = "wasm32")))]
    pub fn new(ctx: egui::Context, profile: Option<ProfileContext>) -> Self {
        let (platform_tx, platform_rx) = smol::channel::unbounded();
        let (drawer_tx, drawer_rx) = smol::channel::unbounded();
        let (ssh_tx, ssh_rx) = smol::channel::unbounded();
        let (logs_tx, logs_rx) = smol::channel::unbounded();
        let (opslog_tx, opslog_rx) = smol::channel::unbounded();
        let (dns_tx, dns_rx) = smol::channel::unbounded();
        let (mycart_tx, mycart_rx) = smol::channel::unbounded();
        let (event_tx, event_rx) = smol::channel::unbounded();

        // Initialize global log sender
        logs::init_log_sender(logs_tx.clone());

        // Clone for closure (logs_tx is moved into Self at the end)
        let logs_tx_for_drawer = logs_tx.clone();
        let profile_for_actor = profile.clone();

        // Spawn background thread with smol executor
        let runtime_handle = std::thread::spawn(move || {
            smol::block_on(async {
                dure_info!("ViewModel runtime started");

                // Create actors
                let platform_actor = platform::PlatformActor::new(platform_rx, event_tx.clone(), profile_for_actor.clone());
                let drawer_actor =
                    platform::DrawerActor::new(drawer_rx, event_tx.clone(), logs_tx_for_drawer, profile_for_actor.clone());
                let ssh_actor =
                    ssh::SshActor::new(ssh_rx, event_tx.clone(), profile_for_actor.clone());
                let opslog_actor = logs::OpsLogActor::new(opslog_rx, event_tx.clone());
                let dns_actor = dns::DnsActor::new(dns_rx, event_tx.clone());
                let mycart_actor = mycart::MycartActor::new(mycart_rx, event_tx.clone());

                // Run all actors concurrently
                smol::spawn(platform_actor.run()).detach();
                smol::spawn(drawer_actor.run()).detach();
                smol::spawn(ssh_actor.run()).detach();
                smol::spawn(logs::log_actor_loop(logs_rx, event_tx.clone())).detach();
                smol::spawn(opslog_actor.run()).detach();
                smol::spawn(dns_actor.run()).detach();
                smol::spawn(mycart_actor.run()).detach();

                // Keep thread alive
                std::future::pending::<()>().await
            })
        });

        Self {
            platform_tx,
            drawer_tx,
            ssh_tx,
            logs_tx,
            opslog_tx,
            dns_tx,
            mycart_tx,
            event_rx,
            state: ViewModelState::default(),
            runtime_handle: Some(RuntimeHandle::Native(runtime_handle)),
            egui_ctx: Some(ctx),
            profile,
        }
    }

    /// Create ViewModel for CLI mode (headless)
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_headless(profile: Option<ProfileContext>) -> Self {
        let (platform_tx, platform_rx) = smol::channel::unbounded();
        let (drawer_tx, drawer_rx) = smol::channel::unbounded();
        let (ssh_tx, ssh_rx) = smol::channel::unbounded();
        let (logs_tx, logs_rx) = smol::channel::unbounded();
        let (opslog_tx, opslog_rx) = smol::channel::unbounded();
        let (dns_tx, dns_rx) = smol::channel::unbounded();
        let (mycart_tx, mycart_rx) = smol::channel::unbounded();
        let (event_tx, event_rx) = smol::channel::unbounded();

        // Initialize global log sender
        logs::init_log_sender(logs_tx.clone());

        // Clone for closure (logs_tx is moved into Self at the end)
        let logs_tx_for_drawer = logs_tx.clone();
        let profile_for_actor = profile.clone();

        let runtime_handle = std::thread::spawn(move || {
            smol::block_on(async {
                dure_info!("ViewModel runtime started (headless)");

                let platform_actor = platform::PlatformActor::new(platform_rx, event_tx.clone(), profile_for_actor.clone());
                let drawer_actor =
                    platform::DrawerActor::new(drawer_rx, event_tx.clone(), logs_tx_for_drawer, profile_for_actor.clone());
                let ssh_actor =
                    ssh::SshActor::new(ssh_rx, event_tx.clone(), profile_for_actor.clone());
                let opslog_actor = logs::OpsLogActor::new(opslog_rx, event_tx.clone());
                let dns_actor = dns::DnsActor::new(dns_rx, event_tx.clone());
                let mycart_actor = mycart::MycartActor::new(mycart_rx, event_tx.clone());

                smol::spawn(platform_actor.run()).detach();
                smol::spawn(drawer_actor.run()).detach();
                smol::spawn(ssh_actor.run()).detach();
                smol::spawn(logs::log_actor_loop(logs_rx, event_tx.clone())).detach();
                smol::spawn(opslog_actor.run()).detach();
                smol::spawn(dns_actor.run()).detach();
                smol::spawn(mycart_actor.run()).detach();

                std::future::pending::<()>().await
            })
        });

        Self {
            platform_tx,
            drawer_tx,
            ssh_tx,
            logs_tx,
            opslog_tx,
            dns_tx,
            mycart_tx,
            event_rx,
            state: ViewModelState::default(),
            runtime_handle: Some(RuntimeHandle::Native(runtime_handle)),
            #[cfg(feature = "gui")]
            egui_ctx: None,
            profile,
        }
    }

    /// Create ViewModel for WASM (browser)
    #[cfg(target_arch = "wasm32")]
    pub fn new_wasm(profile: Option<ProfileContext>) -> Self {
        use wasm_bindgen_futures::spawn_local;

        let (platform_tx, platform_rx) = smol::channel::unbounded();
        let (drawer_tx, drawer_rx) = smol::channel::unbounded();
        let (ssh_tx, _ssh_rx) = smol::channel::unbounded(); // Unused in WASM
        let (ns_tx, ns_rx) = smol::channel::unbounded();
        let (logs_tx, logs_rx) = smol::channel::unbounded();
        let (opslog_tx, opslog_rx) = smol::channel::unbounded();
        let (dns_tx, dns_rx) = smol::channel::unbounded();
        let (mycart_tx, mycart_rx) = smol::channel::unbounded();
        let (event_tx, event_rx) = smol::channel::unbounded();

        // Initialize global log sender
        logs::init_log_sender(logs_tx.clone());

        // Clone for closure (logs_tx is moved into Self at the end)
        let logs_tx_for_drawer = logs_tx.clone();
        let profile_for_actor = profile.clone();

        // Spawn actors in Web Worker context
        spawn_local(async move {
            dure_info!("ViewModel runtime started (WASM)");

            let platform_actor = platform::PlatformActor::new(platform_rx, event_tx.clone(), profile_for_actor.clone());
            let drawer_actor =
                platform::DrawerActor::new(drawer_rx, event_tx.clone(), logs_tx_for_drawer, profile_for_actor.clone());
            let ns_actor = ns::NsActor::new(ns_rx, event_tx.clone(), profile_for_actor.clone());
            let opslog_actor = logs::OpsLogActor::new(opslog_rx, event_tx.clone());
            let dns_actor = dns::DnsActor::new(dns_rx, event_tx.clone());
            let mycart_actor = mycart::MycartActor::new(mycart_rx, event_tx.clone());

            // SSH disabled in WASM (no native SSH in browser) - gated at compile time

            // Run actors concurrently
            let logs_actor = logs::log_actor_loop(logs_rx, event_tx.clone());
            futures::join!(
                platform_actor.run(),
                drawer_actor.run(),
                ns_actor.run(),
                logs_actor,
                opslog_actor.run(),
                dns_actor.run(),
                mycart_actor.run(),
            );
        });

        Self {
            platform_tx,
            drawer_tx,
            ssh_tx,
            logs_tx,
            opslog_tx,
            dns_tx,
            mycart_tx,
            event_rx,
            state: ViewModelState::default(),
            runtime_handle: None,
            #[cfg(feature = "gui")]
            egui_ctx: None,
            profile,
        }
    }

    /// Poll for events and update state (GUI mode)
    #[cfg(feature = "gui")]
    pub fn poll_events(&mut self, ctx: &egui::Context) -> Vec<ViewModelEvent> {
        let mut events = Vec::new();

        while let Ok(event) = self.event_rx.try_recv() {
            dure_debug!(" ViewModel: Received event: {:?}", event);
            self.apply_event(&event, Some(ctx));
            events.push(event);
        }

        if !events.is_empty() {
            dure_debug!(
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
    fn apply_event(&mut self, event: &ViewModelEvent, _ctx: Option<&egui::Context>) {
        // Forward LogEvent::LogsRetrieved to DrawerActor
        if let ViewModelEvent::Logs(logs::LogEvent::LogsRetrieved { filter_id, lines }) = event {
            dure_debug!(
                "[VM_FORWARD] Received LogsRetrieved - filter_id='{}', {} lines",
                filter_id,
                lines.len()
            );

            if !lines.is_empty() {
                dure_debug!("[VM_FORWARD] First line: {}", lines[0]);
            }

            let cmd = platform::DrawerCommand::UpdateLogs {
                project_id: filter_id.clone(),
                lines: lines.clone(),
            };

            dure_debug!("[VM_FORWARD] Sending UpdateLogs to DrawerActor");
            match self.drawer_tx.try_send(cmd) {
                Ok(_) => dure_debug!("[VM_FORWARD] ✓ UpdateLogs sent successfully"),
                Err(e) => dure_error!("[VM_FORWARD] ✗ Failed to forward logs to drawer: {}", e),
            }
        }
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

    pub fn list_vms(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::ListVMs {
                profile_config_path,
                platform_name,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn delete_vm(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        vm_name: String,
        zone: String,
        force: bool,
        release_ip: Option<(String, String)>,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::DeleteVM {
                profile_config_path,
                platform_name,
                vm_name,
                zone,
                force,
                release_ip,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn check_vm_static_ip(
        &self,
        platform_name: String,
        vm_name: String,
        zone: String,
        external_ip: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::CheckVmStaticIp {
                platform_name,
                vm_name,
                zone,
                external_ip,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn restart_vm(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        vm_name: String,
        zone: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::RestartVM {
                profile_config_path,
                platform_name,
                vm_name,
                zone,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn regenerate_vm(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        vm_name: String,
        zone: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::RegenerateVM {
                profile_config_path,
                platform_name,
                vm_name,
                zone,
                profile_kdbx,
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

    pub fn fetch_firewall_rules(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::FetchFirewallRules {
                profile_config_path,
                platform_name,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn check_ip_whitelist(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::CheckIpWhitelist {
                profile_config_path,
                platform_name,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn update_firewall_rule(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        rule_name: String,
        add_ip: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::UpdateFirewallRule {
                profile_config_path,
                platform_name,
                rule_name,
                add_ip,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn remove_ip_from_firewall(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        rule_name: String,
        remove_ips: Vec<String>,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::RemoveIpFromFirewall {
                profile_config_path,
                platform_name,
                rule_name,
                remove_ips,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn flush_firewall_rule(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        rule_name: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::FlushFirewallRule {
                profile_config_path,
                platform_name,
                rule_name,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn create_firewall_rule(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        rule_data: platform::FirewallRuleData,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::CreateFirewallRule {
                profile_config_path,
                platform_name,
                rule_data,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn delete_firewall_rule(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        rule_name: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::DeleteFirewallRule {
                profile_config_path,
                platform_name,
                rule_name,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn fetch_billing(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        project_id: String,
        dataset: String,
        table: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::FetchBilling {
                profile_config_path,
                platform_name,
                project_id,
                dataset,
                table,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn discover_billing_table(
        &self,
        platform_name: String,
        project_id: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::DiscoverBillingTable {
                platform_name,
                project_id,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn refresh_platform(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::RefreshPlatform {
                profile_config_path,
                platform_name,
                profile_kdbx,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn list_projects(&self, platform_name: String) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::ListProjects { platform_name })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn select_project(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        project_id: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::SelectProject {
                profile_config_path,
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
        profile_config_path: std::path::PathBuf,
        platform_type: String,
        oauth_access_token: Option<String>,
        oauth_refresh_token: Option<String>,
        oauth_token_expiry: Option<i64>,
        connected_email: Option<String>,
        selected_project_id: Option<String>,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::AddPlatform {
                profile_config_path,
                platform_type,
                oauth_access_token,
                oauth_refresh_token,
                oauth_token_expiry,
                connected_email,
                selected_project_id,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn update_platform_oauth(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        oauth_access_token: String,
        oauth_refresh_token: String,
        oauth_token_expiry: i64,
        connected_email: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::UpdatePlatformOAuth {
                profile_config_path,
                platform_name,
                oauth_access_token,
                oauth_refresh_token,
                oauth_token_expiry,
                connected_email,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn delete_platform(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        delete_options: platform::DeleteOptions,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::DeletePlatform {
                profile_config_path,
                platform_name,
                delete_options,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn scan_existing_vms(
        &self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::ScanExistingVMs {
                profile_config_path,
                platform_name,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    // Drawer commands
    pub fn send_drawer_command(&self, cmd: platform::DrawerCommand) -> anyhow::Result<()> {
        self.drawer_tx
            .send_blocking(cmd)
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    /// Create operation log entry
    pub fn create_operation_log(
        &self,
        platform_name: String,
        operation_type: String,
        operation_subtype: String,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::CreateOperationLog {
                platform_name,
                operation_type,
                operation_subtype,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    /// Update operation log status
    pub fn update_operation_log(
        &self,
        log_id: i64,
        status: String,
        error_message: Option<String>,
    ) -> anyhow::Result<()> {
        self.platform_tx
            .send_blocking(platform::PlatformCommand::UpdateOperationLog {
                log_id,
                status,
                error_message,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    // SSH commands (desktop-only: Android/WASM don't support SSH operations)
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


    // Service management commands
    pub fn get_linux_status(&self, host: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::GetLinuxStatus { name: host })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    pub fn install_docker(&self, host: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::InstallDocker { name: host })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    pub fn get_docker_status(&self, host: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::GetDockerStatus { name: host })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    pub fn uninstall_docker(&self, host: String) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::UninstallDocker { name: host })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }


    /// Check if SSH host is reachable (TCP port check with timeout)
    pub fn check_host_health(&self, name: String, timeout_secs: u8) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::CheckHostHealth { name, timeout_secs })
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
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
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
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    pub fn remove_docker_container(&self, host: String, container_name: String) {
        let _ = self
            .ssh_tx
            .send_blocking(ssh::SshCommand::RemoveDockerContainer {
                host_name: host,
                container_name,
            });
    }

    /// Remove multiple Docker containers in batch
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
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
    #[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
    pub fn list_docker_containers(&self, host: String) {
        let _ = self
            .ssh_tx
            .send_blocking(ssh::SshCommand::ListDockerContainers { host_name: host });
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

    // SSH Operation Commands

    /// Refresh SSH host status (SSH, Base, Docker, Dure)
    pub fn refresh_ssh_host(
        &self,
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::Refresh { host, profile_kdbx })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    /// Check SSH connection
    pub fn ssh_check(
        &self,
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::SshCheck { host, profile_kdbx })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    /// Check if base packages are installed
    pub fn check_host(
        &self,
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::CheckHost { host, profile_kdbx })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    /// Install host packages
    pub fn install_host(
        &self,
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::InstallHost { host, profile_kdbx })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    /// Check if Docker is installed
    pub fn check_docker(
        &self,
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::CheckDocker { host, profile_kdbx })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    /// Install Docker daemon
    pub fn install_docker_daemon(
        &self,
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::InstallDockerDaemon { host, profile_kdbx })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    /// Remove Docker daemon
    pub fn remove_docker(
        &self,
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::RemoveDocker { host, profile_kdbx })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    /// Check if Dure is installed
    pub fn check_dure(
        &self,
        host: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::CheckDure { host, profile_kdbx })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    /// Install Dure with environment configuration
    pub fn install_dure(
        &self,
        host: String,
        version: String,
        env_fields: Vec<crate::calc::env_parser::EnvField>,
        image_tag: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::InstallDure {
                host,
                version,
                env_fields,
                image_tag,
                profile_kdbx,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    /// Remove Dure
    pub fn remove_dure(
        &self,
        host: String,
        delete_data: bool,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::RemoveDure {
                host,
                delete_data,
                profile_kdbx,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    /// Get logs for SSH host (uses host_id)
    pub fn get_ssh_logs(&self, host: String) -> anyhow::Result<()> {
        self.logs_tx
            .send_blocking(logs::LogCommand::GetLogs { host_id: host })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    /// Get operation logs for SSH host
    pub fn get_operation_logs(&self, host: String, limit: usize) -> anyhow::Result<()> {
        self.ssh_tx
            .send_blocking(ssh::SshCommand::GetOperationLogs { host, limit })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    // OpsLog commands
    pub fn create_opslog_entry(
        &self,
        project_id: String,
        operation_type: String,
        operation_subtype: String,
        status: logs::LogStatus,
        metadata: Option<String>,
    ) -> anyhow::Result<()> {
        self.opslog_tx
            .send_blocking(logs::LogsCommand::CreateLog {
                project_id,
                operation_type,
                operation_subtype,
                status,
                metadata,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn update_opslog_entry(
        &self,
        log_id: i64,
        status: logs::LogStatus,
        error_message: Option<String>,
        metadata: Option<String>,
    ) -> anyhow::Result<()> {
        self.opslog_tx
            .send_blocking(logs::LogsCommand::UpdateLog {
                log_id,
                status,
                error_message,
                metadata,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn list_opslog_entries(
        &self,
        project_id: Option<String>,
        operation_type: Option<String>,
        status: Option<logs::LogStatus>,
        limit: usize,
        offset: usize,
    ) -> anyhow::Result<()> {
        self.opslog_tx
            .send_blocking(logs::LogsCommand::ListLogs {
                project_id,
                operation_type,
                status,
                limit,
                offset,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn get_opslog_entry(&self, log_id: i64) -> anyhow::Result<()> {
        self.opslog_tx
            .send_blocking(logs::LogsCommand::GetLog { log_id })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn get_opslog_stats(
        &self,
        project_id: String,
        period: logs::StatsPeriod,
    ) -> anyhow::Result<()> {
        self.opslog_tx
            .send_blocking(logs::LogsCommand::GetLogStats {
                project_id,
                period,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn purge_opslog_entries(&self, older_than_days: i64) -> anyhow::Result<()> {
        self.opslog_tx
            .send_blocking(logs::LogsCommand::PurgeLogs { older_than_days })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    // DNS commands
    pub fn set_dns_access_token(
        &self,
        access_token: String,
        refresh_token: Option<String>,
    ) -> anyhow::Result<()> {
        self.dns_tx
            .send_blocking(dns::DnsCommand::SetAccessToken {
                access_token,
                refresh_token,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn list_dns_zones(&self) -> anyhow::Result<()> {
        self.dns_tx
            .send_blocking(dns::DnsCommand::ListZones)
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn find_dns_zone(&self, domain: String) -> anyhow::Result<()> {
        self.dns_tx
            .send_blocking(dns::DnsCommand::FindZone { domain })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn create_dns_zone(&self, domain: String, account_id: String) -> anyhow::Result<()> {
        self.dns_tx
            .send_blocking(dns::DnsCommand::CreateZone { domain, account_id })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn list_dns_records(&self, zone_id: String) -> anyhow::Result<()> {
        self.dns_tx
            .send_blocking(dns::DnsCommand::ListRecords { zone_id })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn find_dns_record(
        &self,
        zone_id: String,
        record_type: dns::RecordType,
        name: String,
    ) -> anyhow::Result<()> {
        self.dns_tx
            .send_blocking(dns::DnsCommand::FindRecord {
                zone_id,
                record_type,
                name,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn create_dns_record(
        &self,
        zone_id: String,
        record_type: dns::RecordType,
        name: String,
        content: String,
        ttl: Option<u32>,
        proxied: Option<bool>,
    ) -> anyhow::Result<()> {
        self.dns_tx
            .send_blocking(dns::DnsCommand::CreateRecord {
                zone_id,
                record_type,
                name,
                content,
                ttl,
                proxied,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn update_dns_record(
        &self,
        zone_id: String,
        record_id: String,
        name: String,
        record_type: dns::RecordType,
        content: String,
        ttl: Option<u32>,
        proxied: Option<bool>,
    ) -> anyhow::Result<()> {
        self.dns_tx
            .send_blocking(dns::DnsCommand::UpdateRecord {
                zone_id,
                record_id,
                name,
                record_type,
                content,
                ttl,
                proxied,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn delete_dns_record(&self, zone_id: String, record_id: String) -> anyhow::Result<()> {
        self.dns_tx
            .send_blocking(dns::DnsCommand::DeleteRecord { zone_id, record_id })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn upsert_dns_record(
        &self,
        zone_id: String,
        record_type: dns::RecordType,
        name: String,
        content: String,
        ttl: Option<u32>,
        proxied: Option<bool>,
    ) -> anyhow::Result<()> {
        self.dns_tx
            .send_blocking(dns::DnsCommand::UpsertRecord {
                zone_id,
                record_type,
                name,
                content,
                ttl,
                proxied,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn create_dns_srv_record(
        &self,
        zone_id: String,
        name: String,
        priority: u16,
        weight: u16,
        port: u16,
        target: String,
    ) -> anyhow::Result<()> {
        self.dns_tx
            .send_blocking(dns::DnsCommand::CreateSrvRecord {
                zone_id,
                name,
                priority,
                weight,
                port,
                target,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn upsert_dns_srv_record(
        &self,
        zone_id: String,
        name: String,
        priority: u16,
        weight: u16,
        port: u16,
        target: String,
    ) -> anyhow::Result<()> {
        self.dns_tx
            .send_blocking(dns::DnsCommand::UpsertSrvRecord {
                zone_id,
                name,
                priority,
                weight,
                port,
                target,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn create_default_xmpp_records(
        &self,
        zone_id: String,
        domain: String,
        vm_ip: String,
    ) -> anyhow::Result<()> {
        self.dns_tx
            .send_blocking(dns::DnsCommand::CreateDefaultXmppRecords {
                zone_id,
                domain,
                vm_ip,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn verify_default_xmpp_records(
        &self,
        zone_id: String,
        domain: String,
        vm_ip: String,
    ) -> anyhow::Result<()> {
        self.dns_tx
            .send_blocking(dns::DnsCommand::VerifyDefaultXmppRecords {
                zone_id,
                domain,
                vm_ip,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    // Phase 4: Mycart commands (cleaned up)
    pub fn load_mycart_stats(&self, site_address: String) -> anyhow::Result<()> {
        self.mycart_tx
            .send_blocking(mycart::MycartCommand::LoadStats { site_address })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    // Phase 5: Removed old LoadLogs - now use service-specific commands via drawer auto-refresh

    pub fn list_mycart_crontab(&self, site_address: String) -> anyhow::Result<()> {
        self.mycart_tx
            .send_blocking(mycart::MycartCommand::ListCrontab { site_address })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn enable_mycart_cronjob(&self, site_address: String, job_id: String) -> anyhow::Result<()> {
        self.mycart_tx
            .send_blocking(mycart::MycartCommand::EnableCronjob { site_address, job_id })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn disable_mycart_cronjob(&self, site_address: String, job_id: String) -> anyhow::Result<()> {
        self.mycart_tx
            .send_blocking(mycart::MycartCommand::DisableCronjob { site_address, job_id })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

    pub fn set_mycart_cronjob_interval(
        &self,
        site_address: String,
        job_id: String,
        interval: String,
    ) -> anyhow::Result<()> {
        self.mycart_tx
            .send_blocking(mycart::MycartCommand::SetInterval {
                site_address,
                job_id,
                interval,
            })
            .map_err(|e| anyhow::anyhow!("Send failed: {}", e))
    }

}
