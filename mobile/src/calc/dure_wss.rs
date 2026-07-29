//! Dure-WSS service management functionality
//!
//! Provides installation and lifecycle management for Dure-WSS service

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::Result;
use crate::config::{SshHostConfig, DureWssConfig};
use crate::calc::ssh;

/// Install Dure-WSS via official install script
pub async fn install_dure_wss(
    host_config: &SshHostConfig,
    config: &DureWssConfig,
) -> Result<Vec<String>> {
    let mut progress = Vec::new();

    // Download and run installer
    progress.push("Downloading Dure-WSS installer...".to_string());
    let install_cmd = format!(
        "curl --proto '=https' --tlsv1.2 -sSf https://run.dure.one | \
         DURE_CHANNEL={} DURE_VARIANT={} sh",
        config.channel, config.variant
    );
    ssh::execute_command(host_config, &install_cmd).await?;

    // Configure domain and email
    progress.push("Configuring Dure-WSS...".to_string());
    let config_cmd = format!(
        "dure wss config --domain {} --email {}",
        config.domain, config.email
    );
    ssh::execute_command(host_config, &config_cmd).await?;

    // Start service
    progress.push("Starting Dure-WSS service...".to_string());
    ssh::execute_command(host_config, "dure wss start").await?;

    progress.push("Dure-WSS installed and started".to_string());

    Ok(progress)
}

/// Get Dure-WSS service status
pub async fn get_dure_wss_status(host_config: &SshHostConfig) -> Result<String> {
    ssh::execute_command(host_config, "dure wss status").await
}

/// Start Dure-WSS service
pub async fn start_dure_wss(host_config: &SshHostConfig) -> Result<String> {
    ssh::execute_command(host_config, "dure wss start").await
}

/// Stop Dure-WSS service
pub async fn stop_dure_wss(host_config: &SshHostConfig) -> Result<String> {
    ssh::execute_command(host_config, "dure wss stop").await
}

/// Restart Dure-WSS service
pub async fn restart_dure_wss(host_config: &SshHostConfig) -> Result<String> {
    ssh::execute_command(host_config, "dure wss restart").await
}

/// Uninstall Dure-WSS
pub async fn uninstall_dure_wss(host_config: &SshHostConfig) -> Result<String> {
    // Stop service (ignore errors if already stopped)
    let _ = ssh::execute_command(host_config, "dure wss stop").await;

    // Remove binary and config
    ssh::execute_command(host_config, "sudo rm -f /usr/local/bin/dure").await?;
    ssh::execute_command(host_config, "sudo rm -rf ~/.config/dure").await?;

    Ok("Dure-WSS uninstalled".to_string())
}
