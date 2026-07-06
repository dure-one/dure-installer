//! Firewall command implementation

use crate::cli::commands::platform::helpers::*;
use crate::cli::commands::platform::runner::PlatformCliRunner;
use crate::config::AppConfig;
use crate::viewmodel::platform::{PlatformCommand, PlatformEvent};
use anyhow::{Result, anyhow};
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

/// Auto-detect current IP address
async fn get_current_ip(ip_flag: Option<String>) -> Result<String> {
    if let Some(ip) = ip_flag {
        return Ok(ip);
    }

    // Try ipify API
    match ureq::get("https://api.ipify.org").call() {
        Ok(resp) => Ok(resp.into_string()?),
        Err(e) => Err(anyhow!(
            "Failed to auto-detect IP: {}\n\
                 Use --ip <address> to specify manually",
            e
        )),
    }
}

/// Execute firewall command (inner function for testing)
#[cfg(test)]
pub async fn execute_firewall_inner(
    runner: &mut crate::cli::commands::platform::tests::MockPlatformRunner,
    platform: &crate::config::CloudPlatformConfig,
    ip: Option<String>,
) -> Result<()> {
    let allow_ip = get_current_ip(ip).await?;

    let event = runner
        .execute_command(PlatformCommand::UpdateFirewall {
            platform_name: platform.name.clone(),
            allow_ip: allow_ip.clone(),
        })
        .await?;

    if let PlatformEvent::FirewallUpdated { whitelisted_ip, .. } = event {
        println!("✓ Updated firewall rules");
        println!("✓ Whitelisted IP: {}", whitelisted_ip);
        Ok(())
    } else {
        Err(anyhow!("Unexpected event: {:?}", event))
    }
}

/// Execute firewall command
pub fn execute_firewall_command(name: String, ip: Option<String>) -> Result<()> {
    let config = load_config()?;

    let platform = config
        .platforms
        .iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;

    // Validate platform is ready
    validate_platform_ready(platform, "firewall")?;

    smol::block_on(async {
        let mut runner = PlatformCliRunner::new();
        let allow_ip = get_current_ip(ip).await?;

        println!("✓ Detected current IP: {}", allow_ip);

        let event = runner
            .execute_command(PlatformCommand::UpdateFirewall {
                platform_name: platform.name.clone(),
                allow_ip: allow_ip.clone(),
            })
            .await?;

        if let PlatformEvent::FirewallUpdated { whitelisted_ip, .. } = event {
            println!(
                "✓ Updated firewall rules for project '{}'",
                platform
                    .gcp_selected_project_id
                    .as_deref()
                    .unwrap_or("unknown")
            );
            println!("✓ Whitelisted IP: {}", whitelisted_ip);
            Ok(())
        } else {
            Err(anyhow!("Unexpected event: {:?}", event))
        }
    })
}
