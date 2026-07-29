//! SSH host management CLI commands

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::path::PathBuf;

use crate::calc::audit;
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
    let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
        .context("Failed to get project directories")?;
    Ok(proj_dirs.config_dir().join("config.yml"))
}

/// Execute SSH status command
pub fn execute_ssh_status() -> Result<()> {
    let config_path = get_config_path()?;
    let app_config = AppConfig::load_or_default(&config_path);

    if app_config.ssh_hosts.is_empty() {
        dure_info!("No SSH hosts configured.");
        dure_info!("");
        dure_info!("Run 'dure ssh add username@hostname' to add a host");
        return Ok(());
    }

    dure_info!("SSH Hosts:");
    dure_info!("");

    for (idx, host) in app_config.ssh_hosts.iter().enumerate() {
        dure_info!("{}. {}", idx + 1, host.host);
        dure_info!("   Port: {}", host.port);

        if host.private_key_path.is_some() {
            dure_info!("   Auth: Private key ({})", host.private_key_path.as_ref().unwrap()
            );
        } else if host.password.is_some() {
            dure_info!("   Auth: Password");
        } else {
            dure_info!("   Auth: SSH agent");
        }

        dure_info!("   Initialized: {}", if host.initialized { "Yes" } else { "No" }
        );

        // Test connection (russh uses tokio, wrap with async-compat)
        eprint!("   Status: ");
        match smol::block_on(async { async_compat::Compat::new(ssh::test_connection(host)).await })
        {
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
    host: String,
    pass: Option<String>,
    prvkey: Option<String>,
    port: u16,
) -> Result<()> {
    let config_path = get_config_path()?;
    let mut app_config = AppConfig::load_or_default(&config_path);

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
        keyring_domain: None,
        port,
        initialized: false,
        last_status: None,
        platform_name: None,
        docker_containers: Vec::new(),
        ansible_roles: Vec::new(),
        dure_wss_config: None,
    };

    // Test connection before adding (russh uses tokio, wrap with async-compat)
    dure_info!("Testing SSH connection to {}...", host);
    match smol::block_on(async { async_compat::Compat::new(ssh::test_connection(&ssh_host)).await })
    {
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
    app_config.save(&config_path)?;

    // Record audit event
    let _ = audit::push_cli("system", "cli", "ssh add", &host);

    dure_info!(" SSH host '{}' added successfully", host);

    Ok(())
}

/// Execute SSH del command
pub fn execute_ssh_del(host: String) -> Result<()> {
    let config_path = get_config_path()?;
    let mut app_config = AppConfig::load_or_default(&config_path);

    // Find and remove host
    let initial_len = app_config.ssh_hosts.len();
    app_config.ssh_hosts.retain(|h| h.host != host);

    if app_config.ssh_hosts.len() == initial_len {
        anyhow::bail!("SSH host '{}' not found", host);
    }

    // Save config
    app_config.save(&config_path)?;

    // Record audit event
    let _ = audit::push_cli("system", "cli", "ssh del", &host);

    dure_info!(" SSH host '{}' deleted successfully", host);

    Ok(())
}

/// Execute SSH init command
pub fn execute_ssh_init(host: String) -> Result<()> {
    let config_path = get_config_path()?;
    let mut app_config = AppConfig::load_or_default(&config_path);

    // Find host
    let host_config = app_config
        .ssh_hosts
        .iter_mut()
        .find(|h| h.host == host)
        .context(format!("SSH host '{}' not found", host))?;

    dure_info!("Initializing SSH host: {}", host);
    dure_info!("");

    // Run initialization (russh uses tokio, wrap with async-compat)
    let progress_log = smol::block_on(async {
        async_compat::Compat::new(ssh::initialize_host(host_config)).await
    })?;

    // Print progress
    for line in &progress_log {
        dure_info!("{}", line);
    }

    // Mark as initialized
    host_config.initialized = true;

    // Save config
    app_config.save(&config_path)?;

    dure_info!("");
    dure_info!(" SSH host initialization completed");

    Ok(())
}
