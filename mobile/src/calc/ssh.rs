//! SSH management functionality
//!
//! Provides SSH connection and server initialization capabilities

use anyhow::{Context, Result};
use ssh2::Session;
use std::io::Read;
use std::net::{TcpStream, ToSocketAddrs};
use std::path::Path;
use std::time::Duration;

use crate::config::SshHostConfig;

/// SSH connection result
#[derive(Debug, Clone)]
pub struct SshConnectionResult {
    pub success: bool,
    pub message: String,
}

/// Connect to SSH host and verify connection
pub fn test_connection(host_config: &SshHostConfig) -> Result<SshConnectionResult> {
    let (username, hostname) = parse_ssh_host(&host_config.host)?;
    let addr = format!("{}:{}", hostname, host_config.port);

    // Connect to TCP stream with 15 second timeout
    let timeout = Duration::from_secs(15);
    let socket_addr = addr
        .to_socket_addrs()
        .context(format!("Failed to resolve address: {}", addr))?
        .next()
        .ok_or_else(|| anyhow::anyhow!("No address found for {}", addr))?;

    let tcp = TcpStream::connect_timeout(&socket_addr, timeout)
        .context(format!("Failed to connect to {} (timeout: 15s)", addr))?;

    // Create SSH session
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // Authenticate
    authenticate(&mut sess, &username, host_config)?;

    Ok(SshConnectionResult {
        success: true,
        message: format!("Successfully connected to {}", host_config.host),
    })
}

/// Execute SSH command on remote host
pub fn execute_command(host_config: &SshHostConfig, command: &str) -> Result<String> {
    let (username, hostname) = parse_ssh_host(&host_config.host)?;
    let addr = format!("{}:{}", hostname, host_config.port);

    // Connect to TCP stream with 15 second timeout
    let timeout = Duration::from_secs(15);
    let socket_addr = addr
        .to_socket_addrs()
        .context(format!("Failed to resolve address: {}", addr))?
        .next()
        .ok_or_else(|| anyhow::anyhow!("No address found for {}", addr))?;

    let tcp = TcpStream::connect_timeout(&socket_addr, timeout)
        .context(format!("Failed to connect to {} (timeout: 15s)", addr))?;

    // Create SSH session
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // Authenticate
    authenticate(&mut sess, &username, host_config)?;

    // Execute command
    let mut channel = sess.channel_session()?;
    channel.exec(command)?;

    let mut output = String::new();
    channel.read_to_string(&mut output)?;

    channel.wait_close()?;
    let exit_status = channel.exit_status()?;

    if exit_status != 0 {
        anyhow::bail!("Command failed with exit code {}: {}", exit_status, output);
    }

    Ok(output)
}

/// Initialize SSH host with required software
pub fn initialize_host(host_config: &SshHostConfig) -> Result<Vec<String>> {
    let mut progress_log = Vec::new();

    progress_log.push("Starting SSH host initialization...".to_string());

    // Step 1: Test connection
    progress_log.push("Testing SSH connection...".to_string());
    test_connection(host_config)?;
    progress_log.push("✓ SSH connection successful".to_string());

    // Step 2: Check and install swap if needed
    progress_log.push("Checking swap memory...".to_string());
    let swap_output = execute_command(host_config, "free -m | grep Swap | awk '{print $2}'")?;
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
            execute_command(host_config, cmd).context(format!("Failed to execute: {}", cmd))?;
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
        execute_command(host_config, cmd).context(format!("Failed to execute: {}", cmd))?;
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
    execute_command(host_config, &write_nft_config)?;
    execute_command(host_config, "sudo nft -f /etc/nftables.conf")?;

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
fn authenticate(sess: &mut Session, username: &str, host_config: &SshHostConfig) -> Result<()> {
    let mut attempted_methods = Vec::new();
    let mut errors = Vec::new();

    // Try keyring authentication first if keyring domain is provided
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(ref keyring_domain) = host_config.keyring_domain {
        attempted_methods.push("keyring".to_string());

        match load_private_key_from_keyring(keyring_domain, username) {
            Ok(private_key) => {
                // Write key to temp file (ssh2 requires file path)
                use std::io::Write;
                let temp_dir = std::env::temp_dir();
                let key_file = temp_dir.join(format!("dure_ssh_{}", uuid::Uuid::new_v4()));

                if let Ok(mut file) = std::fs::File::create(&key_file) {
                    if file.write_all(private_key.as_bytes()).is_ok() {
                        // Try to authenticate with the key
                        let result = sess.userauth_pubkey_file(username, None, &key_file, None);

                        // Clean up temp file
                        let _ = std::fs::remove_file(&key_file);

                        if result.is_ok() {
                            return Ok(());
                        } else if let Err(e) = result {
                            errors.push(format!("Keyring: {}", e));
                        }
                    }
                }
            }
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
            match sess.userauth_pubkey_file(username, None, key_path, None) {
                Ok(_) => return Ok(()),
                Err(e) => {
                    errors.push(format!("Private key: {}", e));
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

        match sess.userauth_password(username, password) {
            Ok(_) => return Ok(()),
            Err(e) => {
                errors.push(format!("Password: {}", e));
            }
        }
    }

    // Try agent authentication as fallback
    attempted_methods.push("SSH agent".to_string());

    match sess.userauth_agent(username) {
        Ok(_) => return Ok(()),
        Err(e) => {
            errors.push(format!("SSH agent: {}", e));
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
        };

        assert_eq!(config.keyring_domain, Some("gcp.test.vm".to_string()));
        assert!(config.private_key_path.is_none());
    }
}
