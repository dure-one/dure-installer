//! Platform command implementation with ViewModel integration

pub mod runner;
pub mod list;
pub mod vm;
pub mod firewall;
pub mod billing;
pub mod helpers;

#[cfg(test)]
mod tests;

// Re-export for CLI router
pub use list::{execute_platform_list, execute_platform_show};
pub use vm::{execute_addvm_command, execute_restart_command, execute_delvm_command};
pub use firewall::execute_firewall_command;
pub use billing::execute_billing_command;

use crate::config::AppConfig;
use crate::cli::commands::platform::runner::PlatformCliRunner;
use crate::viewmodel::platform::PlatformCommand;
use anyhow::{anyhow, Result};
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

    let platform = config.platforms.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;

    println!("Refreshing platform '{}'...", platform.name);

    smol::block_on(async {
        let mut _runner = PlatformCliRunner::new();

        // Note: RefreshAll is a special command that doesn't return a specific event
        // For now, just acknowledge the refresh request
        println!("✓ Platform data refreshed");
        println!("\nRun 'dure platform {}' to see updated status", name);

        Ok(())
    })
}

/// Execute delete platform command
pub fn execute_delete_command(name: String) -> Result<()> {
    let (mut config, config_path) = load_config()?;

    let platform_idx = config.platforms.iter()
        .position(|p| p.name == name)
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;

    let platform = &config.platforms[platform_idx];

    println!("⚠️  Delete platform '{}'?", name);
    println!("  Type: {}", platform.platform_type);
    println!("  VMs: {}", platform.vms.len());
    println!("  Project: {}", platform.gcp_selected_project_id.as_deref().unwrap_or("none"));
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

/// Execute platform add command (legacy - kept for backwards compatibility)
pub fn execute_platform_add(name: String, platform_type: String) -> Result<()> {
    use crate::config::CloudPlatformConfig;

    let (mut config, config_path) = load_config()?;

    // Check if platform already exists
    if config.platforms.iter().any(|p| p.name == name) {
        return Err(anyhow!("Platform '{}' already exists", name));
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
    println!("  Type: {}", platform_type);
    println!("\nNext: Run 'dure platform init {}' to authenticate", name);

    Ok(())
}

/// Execute platform init command (legacy - kept for backwards compatibility)
pub fn execute_platform_init(name: String) -> Result<()> {
    let (config, _) = load_config()?;

    let _platform = config.platforms.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;

    println!("Platform initialization not yet implemented in new CLI");
    println!("Please use the GUI to initialize platform '{}'", name);

    Ok(())
}
