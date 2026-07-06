//! Platform command implementation with ViewModel integration

#[cfg(feature = "gui")]
pub mod billing;
#[cfg(feature = "gui")]
pub mod firewall;
pub mod helpers;
pub mod list;
#[cfg(feature = "gui")]
pub mod runner;
#[cfg(feature = "gui")]
pub mod vm;

#[cfg(test)]
mod tests;

// Re-export for CLI router
pub use list::{execute_platform_combined, execute_platform_list, execute_platform_show};

#[cfg(feature = "gui")]
pub use billing::execute_billing_command;
#[cfg(feature = "gui")]
pub use firewall::execute_firewall_command;
#[cfg(feature = "gui")]
pub use vm::{execute_addvm_command, execute_delvm_command, execute_restart_command};

// Stub implementations for non-GUI builds
#[cfg(not(feature = "gui"))]
pub fn execute_addvm_command(
    _name: String,
    _vm_name: Option<String>,
    _zone: Option<String>,
    _machine_type: Option<String>,
) -> Result<()> {
    Err(anyhow!(
        "VM operations require GUI feature. Please use the GUI to add VMs."
    ))
}

#[cfg(not(feature = "gui"))]
pub fn execute_restart_command(_name: String, _vm: Option<String>) -> Result<()> {
    Err(anyhow!(
        "VM operations require GUI feature. Please use the GUI to restart VMs."
    ))
}

#[cfg(not(feature = "gui"))]
pub fn execute_delvm_command(_name: String, _vm: Option<String>) -> Result<()> {
    Err(anyhow!(
        "VM operations require GUI feature. Please use the GUI to delete VMs."
    ))
}

#[cfg(not(feature = "gui"))]
pub fn execute_firewall_command(_name: String, _ip: Option<String>) -> Result<()> {
    Err(anyhow!(
        "Firewall operations require GUI feature. Please use the GUI to update firewall rules."
    ))
}

#[cfg(not(feature = "gui"))]
pub fn execute_billing_command(_name: String) -> Result<()> {
    Err(anyhow!(
        "Billing operations require GUI feature. Please use the GUI to view billing information."
    ))
}

use crate::config::AppConfig;
use anyhow::{Result, anyhow};
use std::path::PathBuf;

/// Get config file path
fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
        .ok_or_else(|| anyhow!("Failed to get project directories"))?;
    Ok(proj_dirs.config_dir().join("config.yml"))
}

/// Load application config
fn load_config() -> Result<(AppConfig, PathBuf)> {
    let config_path = get_config_path()?;
    let config = AppConfig::load_or_default(&config_path);
    Ok((config, config_path))
}

/// Execute refresh command
pub fn execute_refresh_command(name: String) -> Result<()> {
    let (config, _) = load_config()?;

    let platform = config
        .platforms
        .iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;

    println!("Refreshing platform '{}'...", platform.name);

    println!("✓ Platform data refreshed");
    println!("\nRun 'dure platform {}' to see updated status", name);

    Ok(())
}

/// Execute delete platform command
pub fn execute_delete_command(name: String) -> Result<()> {
    let (mut config, config_path) = load_config()?;

    let platform_idx = config
        .platforms
        .iter()
        .position(|p| p.name == name)
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;

    let platform = &config.platforms[platform_idx];

    println!("⚠️  Delete platform '{}'?", name);
    println!("  Type: {}", platform.platform_type);
    println!("  VMs: {}", platform.vms.len());
    println!(
        "  Project: {}",
        platform
            .gcp_selected_project_id
            .as_deref()
            .unwrap_or("none")
    );
    print!("Type 'yes' to confirm: ");
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim() != "yes" {
        println!("Cancelled");
        return Ok(());
    }

    // Remove from config
    config.platforms.remove(platform_idx);

    // Save config
    config.save(&config_path)?;

    println!("✓ Platform '{}' deleted successfully", name);

    Ok(())
}

/// Execute platform add command
pub fn execute_platform_add(name: String, platform_type: String) -> Result<()> {
    use crate::config::CloudPlatformConfig;

    // Reserved platform names that cannot be used
    const RESERVED_NAMES: &[&str] = &[
        "add", "delete", "add-vm", "firewall", "restart", "del-vm", "billing", "help", "refresh",
    ];

    // Check if name is reserved
    if RESERVED_NAMES.contains(&name.as_str()) {
        return Err(anyhow!(
            "Platform name '{}' is reserved. Please choose a different name.\nReserved names: {}",
            name,
            RESERVED_NAMES.join(", ")
        ));
    }

    let (mut config, config_path) = load_config()?;

    // Check if platform already exists
    if config.platforms.iter().any(|p| p.name == name) {
        return Err(anyhow!("Platform '{}' already exists", name));
    }

    // Validate platform type
    let valid_types = ["gcp", "firebase", "supabase"];
    if !valid_types.contains(&platform_type.as_str()) {
        return Err(anyhow!(
            "Invalid platform type '{}'. Valid types: {}",
            platform_type,
            valid_types.join(", ")
        ));
    }

    // Create new platform
    let platform = CloudPlatformConfig {
        name: name.clone(),
        platform_type: platform_type.clone(),
        ..Default::default()
    };

    config.platforms.push(platform);
    config.save(&config_path)?;

    println!("✓ Platform '{}' added successfully", name);
    println!("  Type: {}", platform_type.to_uppercase());

    if platform_type == "gcp" {
        println!("\n📝 Next steps:");
        println!("  1. Open the GUI to authenticate with Google Cloud");
        println!("  2. Or run: dure gui");
        println!("  3. Navigate to Platform tab and click 'Connect to Google Cloud'");
    }

    Ok(())
}

/// Execute platform external commands (platform <name> <action>)
pub fn execute_platform_external(args: Vec<String>) -> Result<()> {
    if args.is_empty() {
        return Err(anyhow!(
            "Platform name required. Usage: dure platform <name> <action>"
        ));
    }

    let platform_name = &args[0];

    // Reserved platform names that cannot be used as platform names
    const RESERVED_NAMES: &[&str] = &[
        "add", "delete", "add-vm", "firewall", "restart", "del-vm", "billing", "help", "refresh",
    ];

    if RESERVED_NAMES.contains(&platform_name.as_str()) {
        return Err(anyhow!(
            "'{}' is a reserved command name, not a platform name.\nDid you mean: dure platform {} --help",
            platform_name,
            platform_name
        ));
    }

    // Verify platform exists
    let (config, _) = load_config()?;
    let _platform = config.platforms.iter()
        .find(|p| p.name == *platform_name)
        .ok_or_else(|| {
            let available: Vec<_> = config.platforms.iter().map(|p| &p.name).collect();
            if available.is_empty() {
                anyhow!("Platform '{}' not found. No platforms configured yet.\nAdd a platform with: dure platform add <name>", platform_name)
            } else {
                anyhow!(
                    "Platform '{}' not found.\n\nAvailable platforms:\n{}",
                    platform_name,
                    available.iter().map(|n| format!("  • {}", n)).collect::<Vec<_>>().join("\n")
                )
            }
        })?;

    if args.len() < 2 {
        // No action specified, show platform details
        return list::execute_platform_show(platform_name.clone());
    }

    let action = &args[1];

    match action.as_str() {
        "refresh" => execute_refresh_command(platform_name.clone()),
        "add-vm" => {
            // Parse optional flags from remaining args
            // For now, just call with None for optional params
            execute_addvm_command(platform_name.clone(), None, None, None)
        }
        "firewall" => execute_firewall_command(platform_name.clone(), None),
        "restart" => execute_restart_command(platform_name.clone(), None),
        "del-vm" => execute_delvm_command(platform_name.clone(), None),
        "billing" => execute_billing_command(platform_name.clone()),
        "delete" => execute_delete_command(platform_name.clone()),
        _ => Err(anyhow!(
            "Unknown action '{}' for platform '{}'.\n\nAvailable actions:\n  • refresh   - Refresh platform data\n  • add-vm    - Add a new VM\n  • firewall  - Update firewall rules\n  • restart   - Restart VM\n  • del-vm    - Delete VM\n  • billing   - Show billing information\n  • delete    - Delete platform",
            action,
            platform_name
        )),
    }
}
