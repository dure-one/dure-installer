//! PlatformCliRunner - ViewModel wrapper for CLI commands

use crate::viewmodel::platform::{PlatformCommand, PlatformEvent};
use crate::viewmodel::{ViewModel, ViewModelEvent};
use anyhow::{Result, anyhow};
use std::time::{Duration, Instant};

/// CLI-specific ViewModel runner
pub struct PlatformCliRunner {
    vm: ViewModel,
}

impl PlatformCliRunner {
    /// Create a new runner with headless ViewModel
    pub fn new() -> Self {
        Self {
            vm: ViewModel::new_headless(),
        }
    }

    /// Execute a platform command and wait for result
    pub async fn execute_command(&mut self, cmd: PlatformCommand) -> Result<PlatformEvent> {
        // Send command to ViewModel via platform_tx
        // Note: We use the individual methods for now since platform_tx is not public
        // This is a simplified approach - in production we might want a generic send method

        // For now, we'll need to match on the command type and call the appropriate method
        // However, for the CLI runner, we can use a helper pattern
        match &cmd {
            PlatformCommand::CreateVM {
                platform_name,
                vm_name,
                zone,
                machine_type,
            } => {
                self.vm.create_vm(
                    platform_name.clone(),
                    vm_name.clone(),
                    zone.clone(),
                    machine_type.clone(),
                )?;
            }
            PlatformCommand::DeleteVM {
                platform_name,
                vm_name,
                zone,
            } => {
                self.vm
                    .delete_vm(platform_name.clone(), vm_name.clone(), zone.clone())?;
            }
            PlatformCommand::RestartVM {
                platform_name,
                vm_name,
                zone,
            } => {
                self.vm
                    .restart_vm(platform_name.clone(), vm_name.clone(), zone.clone())?;
            }
            PlatformCommand::RegenerateVM {
                platform_name,
                vm_name,
                zone,
            } => {
                self.vm
                    .regenerate_vm(platform_name.clone(), vm_name.clone(), zone.clone())?;
            }
            PlatformCommand::UpdateFirewall {
                platform_name,
                allow_ip,
            } => {
                self.vm
                    .update_firewall(platform_name.clone(), allow_ip.clone())?;
            }
            PlatformCommand::FetchBilling {
                platform_name,
                project_id,
                dataset,
                table,
            } => {
                self.vm.fetch_billing(
                    platform_name.clone(),
                    project_id.clone(),
                    dataset.clone(),
                    table.clone(),
                )?;
            }
            PlatformCommand::ListProjects { platform_name } => {
                self.vm.list_projects(platform_name.clone())?;
            }
            PlatformCommand::ListVMs { platform_name } => {
                self.vm.list_vms(platform_name.clone())?;
            }
            _ => {
                return Err(anyhow!("Command not yet supported: {:?}", cmd));
            }
        }

        // Poll for result event with timeout
        let timeout = Duration::from_secs(60);
        let start = Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(anyhow!("Operation timed out after 60 seconds"));
            }

            // Check for events using headless polling
            let events = self.vm.poll_events_headless();
            for event in events {
                if let ViewModelEvent::Platform(platform_event) = event {
                    match platform_event {
                        PlatformEvent::Error { error, .. } => {
                            return Err(anyhow!("{}", error));
                        }
                        _ => {
                            return Ok(platform_event);
                        }
                    }
                }
            }

            // Sleep before next poll
            smol::Timer::after(Duration::from_millis(100)).await;
        }
    }
}
