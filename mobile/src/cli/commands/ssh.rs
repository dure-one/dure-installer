//! SSH host management CLI commands

// Logging provided by standard log crate
use crate::{dure_debug, dure_error, dure_info, dure_warn};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::path::PathBuf;

use crate::calc::audit;
use crate::calc::profile::ProfileContext;
use crate::calc::ssh;
use crate::config::{AppConfig, SshHostConfig};

/// SSH host management commands
#[derive(Debug, Args)]
pub struct SshCommand {
    #[command(subcommand)]
    pub command: SshSubcommand,
}

#[derive(Debug, Subcommand)]
pub enum SshSubcommand {
    /// Show list and status of SSH hosts
    Status,
    /// Add SSH host to configuration
    Add {
        /// SSH connection string (username@hostname)
        host: String,
        /// SSH password
        #[arg(long)]
        pass: Option<String>,
        /// Path to private key file
        #[arg(long)]
        prvkey: Option<String>,
        /// SSH port (default: 22)
        #[arg(long, default_value = "22")]
        port: u16,
    },
    /// Delete SSH host from configuration
    Del {
        /// SSH connection string (username@hostname)
        host: String,
    },
    /// Initialize SSH host (install swap, nftables, dure server)
    Init {
        /// SSH connection string (username@hostname)
        host: String,
    },
}

/// Get config file path
fn get_config_path() -> Result<PathBuf> {
    Ok(crate::get_app_config_dir()?.join("config.yml"))
}

/// Execute SSH status command
pub fn execute_ssh_status(profile: &ProfileContext) -> Result<()> {
    let app_config = AppConfig::load_or_default_for_profile(profile);

    if app_config.ssh_hosts.is_empty() {
        dure_info!("No SSH hosts configured.");
        dure_info!("");
        dure_info!("Run 'dure ssh add username@hostname' to add a host");
        return Ok(());
    }

    // Load profile keyring for SSH authentication
    use crate::calc::keyring::DatabaseHandle;
    use std::sync::Arc;

    let keyring = match DatabaseHandle::open(
        profile.kdbx_path.clone(),
        profile.kpkey_path.clone(),
        None, // No password - using keyfile only
    ) {
        Ok(handle) => Some(Arc::new(handle)),
        Err(e) => {
            dure_warn!("Failed to load keyring: {}", e);
            dure_warn!("SSH authentication may fail for hosts requiring keyring credentials.");
            None
        }
    };

    dure_info!("SSH Hosts:");
    dure_info!("");

    for (idx, host) in app_config.ssh_hosts.iter().enumerate() {
        dure_info!("{}. {}", idx + 1, host.host);
        dure_info!("   Port: {}", host.port);

        if host.private_key_path.is_some() {
            dure_info!(
                "   Auth: Private key ({})",
                host.private_key_path.as_ref().unwrap()
            );
        } else if host.password.is_some() {
            dure_info!("   Auth: Password");
        } else {
            dure_info!("   Auth: SSH agent");
        }

        dure_info!(
            "   Initialized: {}",
            if host.initialized { "Yes" } else { "No" }
        );

        // Test connection with keyring
        eprint!("   Status: ");
        match smol::block_on(async { ssh::test_connection(host, keyring.as_ref()).await }) {
            Ok(result) => {
                if result.success {
                    dure_info!(" Connected");
                } else {
                    dure_error!(" {}", result.message);
                }
            }
            Err(e) => {
                dure_error!(" Connection failed: {}", e);
            }
        }

        dure_info!("");
    }

    Ok(())
}

/// Execute SSH add command
pub fn execute_ssh_add(
    profile: &ProfileContext,
    host: String,
    pass: Option<String>,
    prvkey: Option<String>,
    port: u16,
) -> Result<()> {
    let mut app_config = AppConfig::load_or_default_for_profile(profile);

    // Check if host already exists
    if app_config.ssh_hosts.iter().any(|h| h.host == host) {
        anyhow::bail!("SSH host '{}' already exists", host);
    }

    // Expand private key path if provided
    let private_key_path = prvkey
        .as_ref()
        .map(|key_path| shellexpand::tilde(key_path).to_string());

    // Create new SSH host config
    let ssh_host = SshHostConfig {
        host: host.clone(),
        password: pass,
        private_key_path,
        port,
        ..Default::default()
    };

    // Test connection before adding (russh uses tokio, wrap with async-compat)
    dure_info!("Testing SSH connection to {}...", host);
    match smol::block_on(async { ssh::test_connection(&ssh_host, None).await }) {
        Ok(result) => {
            if result.success {
                dure_info!(" Connection successful");
            } else {
                dure_warn!(" Warning: {}", result.message);
            }
        }
        Err(e) => {
            dure_error!(" Connection test failed: {}", e);
            dure_info!("");
            dure_info!("Host will be added anyway. You can test it later with 'dure ssh status'");
        }
    }

    // Add to config
    app_config.ssh_hosts.push(ssh_host);

    // Save config
    app_config.save_for_profile(profile)?;

    // Record audit event
    let _ = audit::push_cli_for_profile(profile, "system", "cli", "ssh add", &host);

    dure_info!(" SSH host '{}' added successfully", host);

    Ok(())
}

/// Execute SSH del command
pub fn execute_ssh_del(profile: &ProfileContext, host: String) -> Result<()> {
    let mut app_config = AppConfig::load_or_default_for_profile(profile);

    // Find and remove host
    let initial_len = app_config.ssh_hosts.len();
    app_config.ssh_hosts.retain(|h| h.host != host);

    if app_config.ssh_hosts.len() == initial_len {
        anyhow::bail!("SSH host '{}' not found", host);
    }

    // Save config
    app_config.save_for_profile(profile)?;

    // Record audit event
    let _ = audit::push_cli_for_profile(profile, "system", "cli", "ssh del", &host);

    dure_info!(" SSH host '{}' deleted successfully", host);

    Ok(())
}

/// Execute SSH init command
pub fn execute_ssh_init(profile: &ProfileContext, host: String) -> Result<()> {
    let mut app_config = AppConfig::load_or_default_for_profile(profile);

    // Find host
    let host_config = app_config
        .ssh_hosts
        .iter_mut()
        .find(|h| h.host == host)
        .context(format!("SSH host '{}' not found", host))?;

    dure_info!("Initializing SSH host: {}", host);
    dure_info!("");

    // Run initialization (russh uses tokio, wrap with async-compat)
    let progress_log = smol::block_on(async { ssh::initialize_host(host_config).await })?;

    // Print progress
    for line in &progress_log {
        dure_info!("{}", line);
    }

    // Mark as initialized
    host_config.initialized = true;

    // Save config
    app_config.save_for_profile(profile)?;

    dure_info!("");
    dure_info!(" SSH host initialization completed");

    Ok(())
}
