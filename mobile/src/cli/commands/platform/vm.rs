//! VM operation commands

use crate::config::{AppConfig, CloudPlatformConfig};
use crate::cli::commands::platform::helpers::*;
use crate::cli::commands::platform::runner::PlatformCliRunner;
use crate::viewmodel::platform::{PlatformCommand, PlatformEvent};
use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Get config file path
fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
        .ok_or_else(|| anyhow!("Failed to get project directories"))?;
    Ok(proj_dirs.config_dir().join("config.yml"))
}

/// Load application config
fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path()?;
    Ok(AppConfig::load_or_default(&config_path))
}

/// Select VM from platform (auto-select if one, error if none)
pub fn select_vm(platform: &CloudPlatformConfig, vm_flag: Option<String>) -> Result<(String, String)> {
    if let Some(vm_name) = vm_flag {
        let vm = platform.vms.iter().find(|v| v.name == vm_name)
            .ok_or_else(|| anyhow!("VM '{}' not found", vm_name))?;
        return Ok((vm.name.clone(), vm.zone.clone()));
    }

    match platform.vms.len() {
        0 => Err(anyhow!("No VMs found")),
        1 => {
            let vm = &platform.vms[0];
            Ok((vm.name.clone(), vm.zone.clone()))
        }
        _ => {
            // Multiple VMs - in real CLI would prompt, for now error
            Err(anyhow!(
                "Multiple VMs found. Use --vm <name> to specify:\n{}",
                platform.vms.iter()
                    .map(|v| format!("  • {} ({})", v.name, v.zone))
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
        }
    }
}

/// Execute addvm command (inner function for testing)
#[cfg(test)]
pub async fn execute_addvm_inner(
    runner: &mut crate::cli::commands::platform::tests::MockPlatformRunner,
    platform: &CloudPlatformConfig,
    vm_name: String,
    zone: String,
    machine_type: String,
) -> Result<()> {
    let event = runner.execute_command(PlatformCommand::CreateVM {
        platform_name: platform.name.clone(),
        vm_name: vm_name.clone(),
        zone: zone.clone(),
        machine_type: machine_type.clone(),
    }).await?;

    if let PlatformEvent::VMCreated { vm_name, external_ip, .. } = event {
        println!("✓ VM created successfully");
        println!("  Name: {}", vm_name);
        println!("  Zone: {}", zone);
        println!("  External IP: {}", external_ip);
        Ok(())
    } else {
        Err(anyhow!("Unexpected event: {:?}", event))
    }
}

/// Execute addvm command
pub fn execute_addvm_command(
    name: String,
    vm_name_flag: Option<String>,
    zone_flag: Option<String>,
    machine_type_flag: Option<String>,
) -> Result<()> {
    let config = load_config()?;

    let platform = config.platforms.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;

    // Validate platform is ready
    validate_platform_ready(platform, "addvm")?;

    // Check if VM already exists
    if !platform.vms.is_empty() {
        return Err(anyhow!(
            "Platform '{}' already has a VM: {}\n\n\
             To create a new VM, first delete the existing one:\n  \
             dure platform {} delvm",
            platform.name,
            platform.vms[0].name,
            platform.name
        ));
    }

    // Get VM parameters (use defaults or prompt)
    let vm_name = vm_name_flag.ok_or_else(||
        anyhow!("VM name required. Use --vm-name <name>")
    )?;
    let zone = zone_flag.unwrap_or_else(|| "us-central1-a".to_string());
    let machine_type = machine_type_flag.unwrap_or_else(|| "e2-micro".to_string());

    println!("Creating VM...");
    println!("  Name: {}", vm_name);
    println!("  Zone: {}", zone);
    println!("  Machine Type: {}", machine_type);
    println!();

    smol::block_on(async {
        let mut runner = PlatformCliRunner::new();

        let event = runner.execute_command(PlatformCommand::CreateVM {
            platform_name: platform.name.clone(),
            vm_name: vm_name.clone(),
            zone: zone.clone(),
            machine_type: machine_type.clone(),
        }).await?;

        if let PlatformEvent::VMCreated { vm_name, external_ip, .. } = event {
            println!("✓ VM created successfully");
            println!("  Name: {}", vm_name);
            println!("  Zone: {}", zone);
            println!("  External IP: {}", external_ip);
            Ok(())
        } else {
            Err(anyhow!("Unexpected event: {:?}", event))
        }
    })
}

/// Execute restart command
pub fn execute_restart_command(name: String, vm_flag: Option<String>) -> Result<()> {
    let config = load_config()?;

    let platform = config.platforms.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;

    // Validate platform is ready
    validate_platform_ready(platform, "restart")?;

    // Select VM
    let (vm_name, zone) = select_vm(platform, vm_flag)?;

    println!("Restarting VM '{}'...", vm_name);

    smol::block_on(async {
        let mut runner = PlatformCliRunner::new();

        let event = runner.execute_command(PlatformCommand::RestartVM {
            platform_name: platform.name.clone(),
            vm_name: vm_name.clone(),
            zone: zone.clone(),
        }).await?;

        if let PlatformEvent::VMRestarted { vm_name, .. } = event {
            println!("✓ VM '{}' restarted successfully", vm_name);
            Ok(())
        } else {
            Err(anyhow!("Unexpected event: {:?}", event))
        }
    })
}

/// Execute delvm command
pub fn execute_delvm_command(name: String, vm_flag: Option<String>) -> Result<()> {
    let config = load_config()?;

    let platform = config.platforms.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;

    // Validate platform is ready
    validate_platform_ready(platform, "delvm")?;

    // Select VM
    let (vm_name, zone) = select_vm(platform, vm_flag)?;

    println!("⚠️  Delete VM '{}'? This cannot be undone.", vm_name);
    print!("Type 'yes' to confirm: ");
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim() != "yes" {
        println!("Cancelled");
        return Ok(());
    }

    println!("Deleting VM '{}'...", vm_name);

    smol::block_on(async {
        let mut runner = PlatformCliRunner::new();

        let event = runner.execute_command(PlatformCommand::DeleteVM {
            platform_name: platform.name.clone(),
            vm_name: vm_name.clone(),
            zone: zone.clone(),
        }).await?;

        if let PlatformEvent::VMDeleted { vm_name, .. } = event {
            println!("✓ VM '{}' deleted successfully", vm_name);
            Ok(())
        } else {
            Err(anyhow!("Unexpected event: {:?}", event))
        }
    })
}
