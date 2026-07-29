//! SSH management functionality
//!
//! Provides SSH connection and server initialization capabilities
//! Uses russh (pure Rust SSH, no OpenSSL dependency)

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::{Context, Result};
use russh::client::{self, Handle};
use russh_keys::key::PublicKey;
use std::net::ToSocketAddrs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use crate::config::SshHostConfig;

/// SSH connection result
#[derive(Debug, Clone)]
pub struct SshConnectionResult {
    pub success: bool,
    pub message: String,
}

/// SSH client handler
struct Client;

#[async_trait::async_trait]
impl client::Handler for Client {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        // Accept all server keys for now
        // TODO: Implement proper host key verification
        Ok(true)
    }
}

/// Connect to SSH host and verify connection
pub async fn test_connection(host_config: &SshHostConfig) -> Result<SshConnectionResult> {
    let (username, hostname) = parse_ssh_host(&host_config.host)?;
    let addr = format!("{}:{}", hostname, host_config.port);

    // Resolve address
    let socket_addr = addr
        .to_socket_addrs()
        .context(format!("Failed to resolve address: {}", addr))?
        .next()
        .ok_or_else(|| anyhow::anyhow!("No address found for {}", addr))?;

    // Connect
    let config = client::Config::default();

    let config = Arc::new(config);
    let sh = Client;

    let mut session = client::connect(config, socket_addr, sh)
        .await
        .context("Failed to connect")?;

    // Authenticate
    authenticate(&mut session, &username, host_config).await?;

    session
        .disconnect(russh::Disconnect::ByApplication, "", "")
        .await?;

    Ok(SshConnectionResult {
        success: true,
        message: format!("Successfully connected to {}", host_config.host),
    })
}

/// Execute SSH command on remote host
pub async fn execute_command(host_config: &SshHostConfig, command: &str) -> Result<String> {
    let (username, hostname) = parse_ssh_host(&host_config.host)?;
    let addr = format!("{}:{}", hostname, host_config.port);

    // Resolve address
    let socket_addr = addr
        .to_socket_addrs()
        .context(format!("Failed to resolve address: {}", addr))?
        .next()
        .ok_or_else(|| anyhow::anyhow!("No address found for {}", addr))?;

    // Connect
    let config = client::Config::default();

    let config = Arc::new(config);
    let sh = Client;

    let mut session = client::connect(config, socket_addr, sh)
        .await
        .context("Failed to connect")?;

    // Authenticate
    authenticate(&mut session, &username, host_config).await?;

    // Execute command
    let mut channel = session.channel_open_session().await?;
    channel.exec(true, command).await?;

    let mut output = String::new();
    let mut code = None;

    loop {
        let Some(msg) = channel.wait().await else {
            break;
        };

        use russh::ChannelMsg::*;
        match msg {
            Data { ref data } => {
                output.push_str(&String::from_utf8_lossy(data));
            }
            ExitStatus { exit_status } => {
                code = Some(exit_status);
            }
            _ => {}
        }
    }

    session
        .disconnect(russh::Disconnect::ByApplication, "", "")
        .await?;

    if let Some(exit_code) = code {
        if exit_code != 0 {
            anyhow::bail!("Command failed with exit code {}: {}", exit_code, output);
        }
    }

    Ok(output)
}

/// Initialize SSH host with required software
pub async fn initialize_host(host_config: &SshHostConfig) -> Result<Vec<String>> {
    let mut progress_log = Vec::new();

    progress_log.push("Starting SSH host initialization...".to_string());

    // Step 1: Test connection
    progress_log.push("Testing SSH connection...".to_string());
    test_connection(host_config).await?;
    progress_log.push("✓ SSH connection successful".to_string());

    // Step 2: Check and install swap if needed
    progress_log.push("Checking swap memory...".to_string());
    let swap_output =
        execute_command(host_config, "free -m | grep Swap | awk '{print $2}'").await?;
    let swap_mb: u32 = swap_output.trim().parse().unwrap_or(0);

    if swap_mb < 8000 {
        progress_log.push(format!(
            "Current swap: {}MB. Installing 8GB swap...",
            swap_mb
        ));

        let swap_commands = vec![
            "sudo fallocate -l 8G /swapfile",
            "sudo chmod 600 /swapfile",
            "sudo mkswap /swapfile",
            "sudo swapon /swapfile",
            "echo '/swapfile none swap sw 0 0' | sudo tee -a /etc/fstab",
        ];

        for cmd in swap_commands {
            execute_command(host_config, cmd)
                .await
                .context(format!("Failed to execute: {}", cmd))?;
        }

        progress_log.push("✓ 8GB swap installed and enabled".to_string());
    } else {
        progress_log.push(format!("✓ Swap already configured: {}MB", swap_mb));
    }

    // Step 3: Install and configure nftables
    progress_log.push("Installing nftables...".to_string());

    let nft_commands = vec![
        "sudo apt-get update",
        "sudo apt-get install -y nftables",
        "sudo systemctl enable nftables",
    ];

    for cmd in nft_commands {
        execute_command(host_config, cmd)
            .await
            .context(format!("Failed to execute: {}", cmd))?;
    }

    progress_log.push("✓ nftables installed".to_string());

    // Configure basic nftables rules
    progress_log.push("Configuring nftables rules...".to_string());

    let nft_rules = r#"#!/usr/sbin/nft -f

flush ruleset

table inet filter {
    chain input {
        type filter hook input priority 0; policy drop;

        # Allow established/related connections
        ct state established,related accept

        # Allow loopback
        iif lo accept

        # Allow SSH
        tcp dport 22 accept

        # Allow HTTP/HTTPS
        tcp dport { 80, 443 } accept

        # Allow ICMP
        ip protocol icmp accept
        ip6 nexthdr icmpv6 accept
    }

    chain forward {
        type filter hook forward priority 0; policy drop;
    }

    chain output {
        type filter hook output priority 0; policy accept;
    }
}
"#;

    let write_nft_config = format!("echo '{}' | sudo tee /etc/nftables.conf", nft_rules);
    execute_command(host_config, &write_nft_config).await?;
    execute_command(host_config, "sudo nft -f /etc/nftables.conf").await?;

    progress_log.push("✓ nftables configured".to_string());

    // Step 4: Install dure server (placeholder - actual implementation needed)
    progress_log.push("Installing dure server...".to_string());

    // TODO: Implement actual dure server installation
    // This would typically involve:
    // - Uploading the binary
    // - Creating systemd service
    // - Starting the service

    progress_log.push("⚠ Dure server installation not yet implemented".to_string());

    // Step 5: Test connection to dure server
    progress_log.push("Testing dure server connection...".to_string());
    progress_log.push("⚠ Dure server connection test not yet implemented".to_string());

    progress_log.push("✓ SSH host initialization completed".to_string());

    Ok(progress_log)
}

/// Parse SSH host string into username and hostname
fn parse_ssh_host(host: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = host.split('@').collect();

    if parts.len() != 2 {
        anyhow::bail!("Invalid SSH host format. Expected: username@hostname");
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Authenticate SSH session
async fn authenticate(
    session: &mut Handle<Client>,
    username: &str,
    host_config: &SshHostConfig,
) -> Result<()> {
    let mut attempted_methods = Vec::new();
    let mut errors = Vec::new();

    // Try keyring authentication first if keyring domain is provided
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(ref keyring_domain) = host_config.keyring_domain {
        attempted_methods.push("keyring".to_string());

        match load_private_key_from_keyring(keyring_domain, username) {
            Ok(private_key_pem) => match russh_keys::decode_secret_key(&private_key_pem, None) {
                Ok(key_pair) => {
                    let auth_res = session
                        .authenticate_publickey(username, Arc::new(key_pair))
                        .await;

                    if auth_res.is_ok() {
                        return Ok(());
                    } else if let Err(e) = auth_res {
                        errors.push(format!("Keyring: {}", e));
                    }
                }
                Err(e) => {
                    errors.push(format!("Keyring key decode: {}", e));
                }
            },
            Err(e) => {
                errors.push(format!("Keyring: {}", e));
            }
        }
    }

    // Try public key authentication if private key file is provided
    if let Some(ref key_path) = host_config.private_key_path {
        attempted_methods.push(format!("private key ({})", key_path));

        let key_path = Path::new(key_path);
        if key_path.exists() {
            match std::fs::read_to_string(key_path) {
                Ok(key_content) => match russh_keys::decode_secret_key(&key_content, None) {
                    Ok(key_pair) => {
                        let auth_res = session
                            .authenticate_publickey(username, Arc::new(key_pair))
                            .await;

                        if auth_res.is_ok() {
                            return Ok(());
                        } else if let Err(e) = auth_res {
                            errors.push(format!("Private key: {}", e));
                        }
                    }
                    Err(e) => {
                        errors.push(format!("Private key decode: {}", e));
                    }
                },
                Err(e) => {
                    errors.push(format!("Private key read: {}", e));
                }
            }
        } else {
            errors.push(format!(
                "Private key: file not found at '{}'",
                key_path.display()
            ));
        }
    }

    // Try password authentication if password is provided
    if let Some(ref password) = host_config.password {
        attempted_methods.push("password".to_string());

        match session.authenticate_password(username, password).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                errors.push(format!("Password: {}", e));
            }
        }
    }

    // Build detailed error message
    let mut error_msg = format!("Authentication failed for {}@host", username);

    if !attempted_methods.is_empty() {
        error_msg.push_str(&format!(
            "\nAttempted methods: {}",
            attempted_methods.join(", ")
        ));
    }

    if !errors.is_empty() {
        error_msg.push_str("\nErrors:");
        for err in errors {
            error_msg.push_str(&format!("\n  - {}", err));
        }
    }

    anyhow::bail!(error_msg)
}

/// Load private key from keyring
#[cfg(not(target_arch = "wasm32"))]
fn load_private_key_from_keyring(domain: &str, username: &str) -> Result<String> {
    use crate::calc::keyring;

    let kdbx_path = keyring::get_default_kdbx_path().context("Failed to get kdbx path")?;
    let kpkey_path = keyring::get_default_kpkey_path().context("Failed to get KPKey path")?;

    let keys = keyring::list_keys(&kdbx_path, Some(&kpkey_path))
        .context("Failed to list keys from keyring")?;

    // Find the key with matching domain and username
    let key_entry = keys
        .iter()
        .find(|k| k.domain == domain && k.username == username)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Key not found in keyring for domain '{}' and username '{}'",
                domain,
                username
            )
        })?;

    // Try to get SSH key from binary attachment first
    if let Some(ssh_key_bytes) = &key_entry.ssh_key {
        // Convert bytes to string (SSH private keys are text)
        let private_key_str =
            String::from_utf8(ssh_key_bytes.clone()).context("SSH key is not valid UTF-8")?;

        Ok(private_key_str)
    } else {
        // Fallback to password field (for backward compatibility or if stored as text)
        if !key_entry.password.is_empty() {
            Ok(key_entry.password.clone())
        } else {
            anyhow::bail!(
                "No SSH key found in keyring entry. Please store the SSH private key as a binary attachment named 'ssh_key'."
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_host() {
        let result = parse_ssh_host("user@example.com");
        assert!(result.is_ok());
        let (username, hostname) = result.unwrap();
        assert_eq!(username, "user");
        assert_eq!(hostname, "example.com");
    }

    #[test]
    fn test_parse_ssh_host_invalid() {
        let result = parse_ssh_host("invalid-host");
        assert!(result.is_err());
    }

    #[test]
    fn test_ssh_config_with_keyring_domain() {
        let config = SshHostConfig {
            host: "user@example.com".to_string(),
            password: None,
            private_key_path: None,
            keyring_domain: Some("gcp.test.vm".to_string()),
            port: 22,
            initialized: false,
            last_status: None,
            platform_name: None,
            docker_containers: Vec::new(),
            ansible_roles: Vec::new(),
            dure_wss_config: None,
        };

        assert_eq!(config.keyring_domain, Some("gcp.test.vm".to_string()));
        assert!(config.private_key_path.is_none());
    }
}

// ============================================================================
// Linux Status and Service Management Functions
// ============================================================================

/// Linux system status (calc layer version)
#[derive(Clone, Debug)]
pub struct LinuxStatus {
    pub uptime: String,
    pub external_ip: String,
    pub load_average: String,
    pub memory_usage: String,
    pub disk_usage: String,
    pub top_processes: Vec<String>,
}

/// Detect OS distribution via SSH
pub async fn detect_os(host_config: &SshHostConfig) -> Result<String> {
    // Try /etc/os-release first (modern standard)
    if let Ok(output) = execute_command(
        host_config,
        "cat /etc/os-release | grep '^ID=' | cut -d= -f2 | tr -d '\"'",
    )
    .await
    {
        let os = output.trim().to_string();
        if !os.is_empty() {
            return Ok(os);
        }
    }

    // Fallback to uname
    if let Ok(output) = execute_command(host_config, "uname -s").await {
        let os = output.trim().to_lowercase();
        if !os.is_empty() {
            return Ok(os);
        }
    }

    Ok("unknown".to_string())
}

/// Get comprehensive Linux system status via SSH
pub async fn get_linux_status(host_config: &SshHostConfig) -> Result<LinuxStatus> {
    // Execute multiple commands - use unwrap_or for resilience
    let uptime = execute_command(host_config, "uptime -p")
        .await
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

    let external_ip = execute_command(host_config, "curl -s ifconfig.me")
        .await
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

    let load = execute_command(host_config, "cat /proc/loadavg | awk '{print $1, $2, $3}'")
        .await
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

    let memory = execute_command(
        host_config,
        "free -h | grep Mem | awk '{print $3 \" / \" $2}'",
    )
    .await
    .unwrap_or_else(|_| "unknown".to_string())
    .trim()
    .to_string();

    let disk = execute_command(
        host_config,
        "df -h / | tail -1 | awk '{print $3 \" / \" $2 \" (\" $5 \")\"}}'",
    )
    .await
    .unwrap_or_else(|_| "unknown".to_string())
    .trim()
    .to_string();

    let processes_output = execute_command(
        host_config,
        "ps aux --sort=-%mem | head -6 | tail -5 | awk '{print $11}'",
    )
    .await
    .unwrap_or_else(|_| "".to_string());

    let top_processes: Vec<String> = processes_output
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    Ok(LinuxStatus {
        uptime,
        external_ip,
        load_average: load,
        memory_usage: memory,
        disk_usage: disk,
        top_processes,
    })
}

/// Check if Docker is installed via SSH
pub async fn check_docker_installed(host_config: &SshHostConfig) -> Result<bool> {
    let result = execute_command(host_config, "command -v docker").await;
    Ok(result.is_ok() && !result.unwrap().trim().is_empty())
}

/// Check if Docker daemon is running via SSH
pub async fn check_docker_running(host_config: &SshHostConfig) -> Result<bool> {
    let result = execute_command(host_config, "systemctl is-active docker").await;
    Ok(result.is_ok() && result.unwrap().trim() == "active")
}

/// Install Docker via convenience script
pub async fn install_docker(host_config: &SshHostConfig) -> Result<()> {
    // Download and execute Docker install script
    execute_command(host_config, "curl -fsSL https://get.docker.com | sh").await?;

    // Enable and start Docker service
    execute_command(host_config, "systemctl enable docker").await?;
    execute_command(host_config, "systemctl start docker").await?;

    Ok(())
}

/// Uninstall Docker
pub async fn uninstall_docker(host_config: &SshHostConfig) -> Result<()> {
    // Stop and disable service
    let _ = execute_command(host_config, "systemctl stop docker").await;
    let _ = execute_command(host_config, "systemctl disable docker").await;

    // Remove packages (Debian/Ubuntu)
    execute_command(host_config,
        "apt-get remove -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin"
    ).await?;

    Ok(())
}

/// Check if Ansible is installed
pub async fn check_ansible_installed(host_config: &SshHostConfig) -> Result<bool> {
    let result = execute_command(host_config, "command -v ansible").await;
    Ok(result.is_ok() && !result.unwrap().trim().is_empty())
}

/// Install Ansible (placeholder)
pub async fn install_ansible(_host_config: &SshHostConfig) -> Result<()> {
    anyhow::bail!("Ansible installation not yet implemented")
}

/// Uninstall Ansible (placeholder)
pub async fn uninstall_ansible(_host_config: &SshHostConfig) -> Result<()> {
    anyhow::bail!("Ansible uninstallation not yet implemented")
}

/// Check if Dure-WSS is installed
pub async fn check_dure_wss_installed(host_config: &SshHostConfig) -> Result<bool> {
    let result = execute_command(host_config, "command -v dure").await;
    Ok(result.is_ok() && !result.unwrap().trim().is_empty())
}

/// Install Dure-WSS (placeholder)
pub async fn install_dure_wss(_host_config: &SshHostConfig) -> Result<()> {
    anyhow::bail!("Dure-WSS installation not yet implemented")
}

/// Uninstall Dure-WSS (placeholder)
pub async fn uninstall_dure_wss(_host_config: &SshHostConfig) -> Result<()> {
    anyhow::bail!("Dure-WSS uninstallation not yet implemented")
}

/// Docker pull (stub implementation)
pub fn docker_pull(_config: &crate::config::SshHostConfig, _image: &str) -> anyhow::Result<()> {
    anyhow::bail!("Docker pull not yet implemented")
}

/// Docker run (stub implementation)
pub fn docker_run(
    _config: &crate::config::SshHostConfig,
    _image: &str,
    _container_name: &str,
    _ports: &[(u16, u16)],
    _env: &[(String, String)],
) -> anyhow::Result<()> {
    anyhow::bail!("Docker run not yet implemented")
}

/// Docker stop (stub implementation)
pub fn docker_stop(
    _config: &crate::config::SshHostConfig,
    _container_name: &str,
) -> anyhow::Result<()> {
    anyhow::bail!("Docker stop not yet implemented")
}

/// Port open (stub implementation)
pub fn port_open(
    _config: &crate::config::SshHostConfig,
    _port: u16,
    _protocol: &str,
) -> anyhow::Result<()> {
    anyhow::bail!("Port open not yet implemented")
}

/// Port close (stub implementation)
pub fn port_close(
    _config: &crate::config::SshHostConfig,
    _port: u16,
    _protocol: &str,
) -> anyhow::Result<()> {
    anyhow::bail!("Port close not yet implemented")
}
