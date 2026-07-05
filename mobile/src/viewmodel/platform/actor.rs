//! Platform actor implementation

use super::{PlatformCommand, PlatformEvent, VmInfo};
use crate::viewmodel::{ViewModelEvent, runtime};
use smol::channel::{Receiver, Sender};

pub struct PlatformActor {
    command_rx: Receiver<PlatformCommand>,
    event_tx: Sender<ViewModelEvent>,
}

impl PlatformActor {
    pub fn new(command_rx: Receiver<PlatformCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self { command_rx, event_tx }
    }

    pub async fn run(mut self) {
        log::info!("PlatformActor started");

        loop {
            match self.command_rx.recv().await {
                Ok(cmd) => {
                    if let Err(e) = self.handle_command(cmd).await {
                        log::error!("PlatformActor command failed: {}", e);
                    }
                }
                Err(_) => {
                    log::info!("PlatformActor: channel closed, shutting down");
                    break;
                }
            }
        }
    }

    async fn handle_command(&mut self, cmd: PlatformCommand) -> anyhow::Result<()> {
        let operation = format!("{:?}", cmd);

        let result = match cmd {
            PlatformCommand::ListVMs { platform_name } => {
                self.list_vms(platform_name).await
            }
            PlatformCommand::CreateVM { platform_name, vm_name, zone, machine_type } => {
                self.create_vm(platform_name, vm_name, zone, machine_type).await
            }
            PlatformCommand::DeleteVM { platform_name, vm_name, zone } => {
                self.delete_vm(platform_name, vm_name, zone).await
            }
            PlatformCommand::RestartVM { platform_name, vm_name, zone } => {
                self.restart_vm(platform_name, vm_name, zone).await
            }
            PlatformCommand::UpdateFirewall { platform_name, allow_ip } => {
                self.update_firewall(platform_name, allow_ip).await
            }
            PlatformCommand::FetchBilling { platform_name, project_id, dataset, table } => {
                self.fetch_billing(platform_name, project_id, dataset, table).await
            }
            _ => {
                // Unimplemented commands
                Err(anyhow::anyhow!("Command not implemented: {:?}", cmd))
            }
        };

        if let Err(e) = result {
            self.send_error(&operation, e).await;
        }

        Ok(())
    }

    async fn list_vms(&mut self, platform_name: String) -> anyhow::Result<()> {
        self.send_progress("list_vms", 0.1, "Loading platform config...").await;

        // Load platform config from DB
        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || crate::calc::db::load_platform(&platform_name)
        }).await?;

        self.send_progress("list_vms", 0.5, "Fetching VMs from GCP...").await;

        // Call GCP API
        let vms = runtime::unblock({
            let project_id = platform.project_id.clone();
            move || crate::calc::gcp_rest::list_vms(&project_id)
        }).await?;

        // Convert to VmInfo
        let vm_infos: Vec<VmInfo> = vms.into_iter().map(|vm| VmInfo {
            name: vm.name,
            zone: vm.zone,
            external_ip: vm.external_ip,
            status: vm.status,
        }).collect();

        self.send_event(PlatformEvent::VMsListed {
            platform_name,
            vms: vm_infos,
        }).await;

        Ok(())
    }

    async fn create_vm(&mut self, platform_name: String, vm_name: String, zone: String, machine_type: String) -> anyhow::Result<()> {
        self.send_progress("create_vm", 0.0, "Starting VM creation...").await;

        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || crate::calc::db::load_platform(&platform_name)
        }).await?;

        self.send_progress("create_vm", 0.3, "Creating VM instance...").await;

        let vm = runtime::unblock({
            let project_id = platform.project_id.clone();
            move || crate::calc::gcp_rest::create_vm(&project_id, &vm_name, &zone, &machine_type)
        }).await?;

        self.send_progress("create_vm", 0.7, "Waiting for external IP...").await;

        // Wait for VM to get external IP
        let external_ip = vm.external_ip.unwrap_or_else(|| "pending".to_string());

        self.send_event(PlatformEvent::VMCreated {
            platform_name,
            vm_name,
            external_ip,
        }).await;

        Ok(())
    }

    async fn delete_vm(&mut self, platform_name: String, vm_name: String, zone: String) -> anyhow::Result<()> {
        self.send_progress("delete_vm", 0.5, "Deleting VM...").await;

        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || crate::calc::db::load_platform(&platform_name)
        }).await?;

        runtime::unblock({
            let project_id = platform.project_id.clone();
            move || crate::calc::gcp_rest::delete_vm(&project_id, &vm_name, &zone)
        }).await?;

        self.send_event(PlatformEvent::VMDeleted {
            platform_name,
            vm_name,
        }).await;

        Ok(())
    }

    async fn restart_vm(&mut self, platform_name: String, vm_name: String, zone: String) -> anyhow::Result<()> {
        self.send_progress("restart_vm", 0.5, "Restarting VM...").await;

        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || crate::calc::db::load_platform(&platform_name)
        }).await?;

        runtime::unblock({
            let project_id = platform.project_id.clone();
            move || crate::calc::gcp_rest::restart_vm(&project_id, &vm_name, &zone)
        }).await?;

        self.send_event(PlatformEvent::VMRestarted {
            platform_name,
            vm_name,
        }).await;

        Ok(())
    }

    async fn update_firewall(&mut self, platform_name: String, allow_ip: String) -> anyhow::Result<()> {
        self.send_progress("update_firewall", 0.5, "Updating firewall rules...").await;

        let platform = runtime::unblock({
            let platform_name = platform_name.clone();
            move || crate::calc::db::load_platform(&platform_name)
        }).await?;

        runtime::unblock({
            let project_id = platform.project_id.clone();
            let allow_ip = allow_ip.clone();
            move || crate::calc::gcp_rest::update_firewall(&project_id, &allow_ip)
        }).await?;

        self.send_event(PlatformEvent::FirewallUpdated {
            platform_name,
            whitelisted_ip: allow_ip,
        }).await;

        Ok(())
    }

    async fn fetch_billing(&mut self, platform_name: String, project_id: String, dataset: String, table: String) -> anyhow::Result<()> {
        self.send_progress("fetch_billing", 0.5, "Fetching billing data...").await;

        let records = runtime::unblock(move || {
            crate::calc::gcp_rest::fetch_billing(&project_id, &dataset, &table)
        }).await?;

        self.send_event(PlatformEvent::BillingFetched {
            platform_name,
            records,
        }).await;

        Ok(())
    }

    async fn send_progress(&self, operation: &str, progress: f32, status: &str) {
        let _ = self.event_tx.send(ViewModelEvent::Platform(
            PlatformEvent::Progress {
                operation: operation.to_string(),
                progress,
                status: status.to_string(),
            }
        )).await;
    }

    async fn send_event(&self, event: PlatformEvent) {
        let _ = self.event_tx.send(ViewModelEvent::Platform(event)).await;
    }

    async fn send_error(&self, operation: &str, error: anyhow::Error) {
        let _ = self.event_tx.send(ViewModelEvent::Platform(
            PlatformEvent::Error {
                operation: operation.to_string(),
                error: format!("{:#}", error),
            }
        )).await;
    }
}
