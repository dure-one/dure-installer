//! SSH management functionality
//!
//! Provides SSH connection and server initialization capabilities
//! Uses russh (pure Rust SSH, no OpenSSL dependency)
//! Desktop-only feature (not available on Android/WASM)

use crate::dure_info;
use anyhow::Result;

use crate::config::SshHostConfig;

/// SSH connection result
#[derive(Debug, Clone)]
pub struct SshConnectionResult {
    pub success: bool,
    pub message: String,
}

/// Linux system status (from SSH queries)
#[derive(Debug, Clone)]
pub struct LinuxStatus {
    pub uptime: String,
    pub external_ip: String,
    pub load_average: String,
    pub memory_usage: String,
    pub disk_usage: String,
    pub top_processes: Vec<String>,
}

// Desktop implementation (requires russh, async-compat, async-trait)
#[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
mod desktop_impl {
    use super::*;
    use crate::{dure_debug, dure_warn, dure_error};
    use anyhow::Context;
    use russh::client::{self, Handle};
    use russh_keys::key::PublicKey;
    use std::net::ToSocketAddrs;
    use std::path::Path;
    use std::sync::Arc;

    /// SSH client handler
    struct Client;

    #[async_trait::async_trait]
    impl client::Handler for Client {
        type Error = anyhow::Error;

        async fn check_server_key(
            &mut self,
            _server_public_key: &PublicKey,
        ) -> Result<bool, Self::Error> {
            Ok(true)
        }
    }

    /// Test SSH connection (simple version with raw private key PEM)
    pub async fn test_connection_simple(
        host_ip: &str,
        private_key_pem: &str,
        port: u16,
        timeout_ms: u64,
    ) -> Result<()> {
        let address = format!("{}:{}", host_ip, port);
        let socket_addrs: Vec<_> = address.to_socket_addrs()?.collect();
        let socket_addr = socket_addrs.first()
            .ok_or_else(|| anyhow::anyhow!("Could not resolve address: {}", address))?;

        let config = Arc::new(russh::client::Config::default());
        let mut session = russh::client::connect(config, socket_addr, Client).await?;

        let key_pair = russh_keys::decode_secret_key(private_key_pem, None)?;

        session
            .authenticate_publickey("root", Arc::new(key_pair))
            .await?;

        session
            .disconnect(russh::Disconnect::ByApplication, "", "")
            .await?;

        Ok(())
    }

    /// Test SSH connection with profile keyring
    pub async fn test_connection(
        host_config: &SshHostConfig,
        profile_keyring: Option<&Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> Result<SshConnectionResult> {
        let (username, hostname) = parse_ssh_host(&host_config.host)?;

        let address = format!("{}:{}", hostname, host_config.port);
        let socket_addrs: Vec<_> = address
            .to_socket_addrs()
            .context("Failed to resolve SSH hostname")?
            .collect();
        let socket_addr = socket_addrs
            .first()
            .ok_or_else(|| anyhow::anyhow!("Could not resolve address: {}", address))?;

        let config = Arc::new(russh::client::Config::default());
        let mut session = russh::client::connect(config, socket_addr, Client)
            .await
            .context("Failed to connect to SSH server")?;

        match authenticate(&mut session, &username, host_config, profile_keyring).await {
            Ok(_) => {
                session
                    .disconnect(russh::Disconnect::ByApplication, "", "")
                    .await
                    .ok();
                Ok(SshConnectionResult {
                    success: true,
                    message: format!("Successfully connected to {}", host_config.host),
                })
            }
            Err(e) => Ok(SshConnectionResult {
                success: false,
                message: format!("{}", e),
            }),
        }
    }

    /// Execute command over SSH
    pub async fn execute_command(
        host_config: &SshHostConfig,
        command: &str,
        profile_keyring: Option<&Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> Result<String> {
        let (username, hostname) = parse_ssh_host(&host_config.host)?;

        let address = format!("{}:{}", hostname, host_config.port);
        let socket_addrs: Vec<_> = address.to_socket_addrs()?.collect();
        let socket_addr = socket_addrs
            .first()
            .ok_or_else(|| anyhow::anyhow!("Could not resolve address: {}", address))?;

        let config = Arc::new(russh::client::Config::default());
        let mut session = russh::client::connect(config, socket_addr, Client).await?;

        authenticate(&mut session, &username, host_config, profile_keyring).await?;

        let mut channel = session.channel_open_session().await?;
        channel.exec(true, command).await?;

        let mut output = String::new();
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
                    if exit_status != 0 {
                        anyhow::bail!("Command failed with exit status {}", exit_status);
                    }
                    break;
                }
                _ => {}
            }
        }

        session
            .disconnect(russh::Disconnect::ByApplication, "", "")
            .await?;

        Ok(output)
    }

    /// Initialize SSH host (install swap, nftables, dure server)
    pub async fn initialize_host(host_config: &SshHostConfig) -> Result<Vec<String>> {
        let mut progress_log = Vec::new();

        progress_log.push("Starting SSH host initialization...".to_string());

        // Step 1: Test connection
        progress_log.push("Testing SSH connection...".to_string());
        test_connection(host_config, None).await?;
        progress_log.push("✓ SSH connection successful".to_string());

        // Step 2: Check and install swap if needed
        progress_log.push("Checking swap memory...".to_string());
        let swap_output =
            execute_command(host_config, "free -m | grep Swap | awk '{print $2}'", None).await?;
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
                execute_command(host_config, cmd, None)
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
            execute_command(host_config, cmd, None)
                .await
                .context(format!("Failed to execute: {}", cmd))?;
        }

        progress_log.push("Configuring nftables firewall...".to_string());

        let nft_rules = r#"#!/usr/sbin/nft -f

flush ruleset

table inet filter {
    chain input {
        type filter hook input priority 0; policy drop;

        # Accept loopback
        iif "lo" accept

        # Accept established/related connections
        ct state established,related accept

        # Accept SSH (port 22)
        tcp dport 22 accept

        # Accept ICMP (ping)
        ip protocol icmp accept
        ip6 nexthdr ipv6-icmp accept
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
        execute_command(host_config, &write_nft_config, None).await?;
        execute_command(host_config, "sudo nft -f /etc/nftables.conf", None).await?;

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
        profile_keyring: Option<&Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> Result<()> {
        let mut attempted_methods = Vec::new();
        let mut errors = Vec::new();

        // Try keyring authentication first if keyring domain is provided
        if let Some(ref keyring_domain) = host_config.keyring_domain {
            attempted_methods.push("keyring".to_string());

            match load_private_key_from_keyring(keyring_domain, username, profile_keyring) {
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
                                errors.push(format!("Private key file: {}", e));
                            }
                        }
                        Err(e) => {
                            errors.push(format!("Private key decode: {}", e));
                        }
                    },
                    Err(e) => {
                        errors.push(format!("Failed to read key file: {}", e));
                    }
                }
            } else {
                errors.push(format!("Private key file not found: {}", key_path.display()));
            }
        }

        // Try password authentication if provided
        if let Some(ref password) = host_config.password {
            attempted_methods.push("password".to_string());

            let auth_res = session.authenticate_password(username, password).await;
            if auth_res.is_ok() {
                return Ok(());
            } else if let Err(e) = auth_res {
                errors.push(format!("Password: {}", e));
            }
        }

        // Try SSH agent if available (fallback)
        attempted_methods.push("SSH agent".to_string());
        let keys = russh_keys::agent::client::AgentClient::connect_env()
            .await
            .ok()
            .and_then(|agent| futures::executor::block_on(async { agent.request_identities().await.ok() }));

        if let Some(keys) = keys {
            for key in keys {
                let auth_res = session
                    .authenticate_publickey(username, key)
                    .await;
                if auth_res.is_ok() {
                    return Ok(());
                }
            }
            errors.push("SSH agent: No valid keys".to_string());
        } else {
            errors.push("SSH agent: Not available".to_string());
        }

        // All methods failed
        anyhow::bail!(
            "Authentication failed for {}@host\nAttempted methods: {}\nErrors:\n  - {}",
            username,
            attempted_methods.join(", "),
            errors.join("\n  - ")
        )
    }

    /// Load private key from keyring (either profile keyring or global keyring)
    fn load_private_key_from_keyring(
        domain: &str,
        username: &str,
        profile_keyring: Option<&Arc<crate::calc::keyring::DatabaseHandle>>,
    ) -> Result<String> {
        use crate::calc::keyring;

        // Use profile keyring if provided, otherwise fall back to global keyring
        let keys = if let Some(handle) = profile_keyring {
            keyring::list_keys_from_handle(handle)
                .context("Failed to list keys from profile keyring")?
        } else {
            // Fall back to global keyring
            let kdbx_path = keyring::get_default_kdbx_path()?;
            let kpkey_path = keyring::get_default_kpkey_path()?;
            keyring::ensure_kdbx_exists(&kdbx_path, &kpkey_path)?;
            keyring::list_keys(&kdbx_path, Some(&kpkey_path), None)
                .context("Failed to list keys from keyring")?
        };

        // Find key by domain
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

        Ok(key_entry.password.clone())
    }

    /// Detect OS distribution via SSH
    pub async fn detect_os(host_config: &SshHostConfig) -> Result<String> {
        // Try /etc/os-release first (modern standard)
        if let Ok(output) = execute_command(
            host_config,
            "cat /etc/os-release | grep '^ID=' | cut -d= -f2 | tr -d '\"'",
            None,
        )
        .await
        {
            let os = output.trim().to_string();
            if !os.is_empty() {
                return Ok(os);
            }
        }

        // Fallback to uname
        if let Ok(output) = execute_command(host_config, "uname -s", None).await {
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
        let uptime = execute_command(host_config, "uptime -p", None)
            .await
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string();

        let external_ip = execute_command(host_config, "curl -s ifconfig.me", None)
            .await
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string();

        let load = execute_command(host_config, "cat /proc/loadavg | awk '{print $1, $2, $3}'", None)
            .await
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_string();

        let memory = execute_command(
            host_config,
            "free -h | grep Mem | awk '{print $3 \" / \" $2}'",
            None,
        )
        .await
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

        let disk = execute_command(
            host_config,
            "df -h / | tail -1 | awk '{print $3 \" / \" $2 \" (\" $5 \")\"}}'",
            None,
        )
        .await
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();

        let processes_output = execute_command(
            host_config,
            "ps aux --sort=-%mem | head -6 | tail -5 | awk '{print $11}'",
            None,
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
        let result = execute_command(host_config, "command -v docker", None).await;
        Ok(result.is_ok() && !result.unwrap().trim().is_empty())
    }

    /// Check if Docker daemon is running via SSH
    pub async fn check_docker_running(host_config: &SshHostConfig) -> Result<bool> {
        let result = execute_command(host_config, "systemctl is-active docker", None).await;
        Ok(result.is_ok() && result.unwrap().trim() == "active")
    }

    /// Install Docker via convenience script
    pub async fn install_docker(host_config: &SshHostConfig) -> Result<()> {
        // Download and execute Docker install script
        execute_command(host_config, "curl -fsSL https://get.docker.com | sh", None).await?;

        // Enable and start Docker service
        execute_command(host_config, "systemctl enable docker", None).await?;
        execute_command(host_config, "systemctl start docker", None).await?;

        Ok(())
    }

    /// Uninstall Docker
    pub async fn uninstall_docker(host_config: &SshHostConfig) -> Result<()> {
        // Stop and disable service
        let _ = execute_command(host_config, "systemctl stop docker", None).await;
        let _ = execute_command(host_config, "systemctl disable docker", None).await;

        // Remove packages (Debian/Ubuntu)
        execute_command(host_config,
            "apt-get remove -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin",
            None
        ).await?;

        Ok(())
    }

    /// Check if Ansible is installed
    pub async fn check_ansible_installed(host_config: &SshHostConfig) -> Result<bool> {
        let result = execute_command(host_config, "command -v ansible", None).await;
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
        let result = execute_command(host_config, "command -v dure", None).await;
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
}

// Re-export desktop implementation
#[cfg(all(not(target_os = "android"), not(target_arch = "wasm32")))]
pub use desktop_impl::*;

// Android/WASM stub implementation
#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub async fn test_connection_simple(
    _host_ip: &str,
    _private_key_pem: &str,
    _port: u16,
    _timeout_ms: u64,
) -> Result<()> {
    anyhow::bail!("SSH not supported on this platform")
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub async fn test_connection(
    _host_config: &SshHostConfig,
    _profile_keyring: Option<&std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
) -> Result<SshConnectionResult> {
    Ok(SshConnectionResult {
        success: false,
        message: "SSH not supported on this platform".to_string(),
    })
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub async fn execute_command(
    _host_config: &SshHostConfig,
    _command: &str,
    _profile_keyring: Option<&std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
) -> Result<String> {
    anyhow::bail!("SSH not supported on this platform")
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub async fn initialize_host(_host_config: &SshHostConfig) -> Result<Vec<String>> {
    anyhow::bail!("SSH not supported on this platform")
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub async fn detect_os(_host_config: &SshHostConfig) -> Result<String> {
    Ok("unknown".to_string())
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub async fn get_linux_status(_host_config: &SshHostConfig) -> Result<LinuxStatus> {
    Ok(LinuxStatus {
        uptime: "N/A".to_string(),
        external_ip: "N/A".to_string(),
        load_average: "N/A".to_string(),
        memory_usage: "N/A".to_string(),
        disk_usage: "N/A".to_string(),
        top_processes: vec![],
    })
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub async fn check_docker_installed(_host_config: &SshHostConfig) -> Result<bool> {
    Ok(false)
}

#[cfg(any(target_os = "android"), target_arch = "wasm32"))]
pub async fn check_docker_running(_host_config: &SshHostConfig) -> Result<bool> {
    Ok(false)
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub async fn install_docker(_host_config: &SshHostConfig) -> Result<()> {
    anyhow::bail!("Docker installation not supported on this platform")
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub async fn uninstall_docker(_host_config: &SshHostConfig) -> Result<()> {
    anyhow::bail!("Docker uninstallation not supported on this platform")
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub async fn check_ansible_installed(_host_config: &SshHostConfig) -> Result<bool> {
    Ok(false)
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub async fn install_ansible(_host_config: &SshHostConfig) -> Result<()> {
    anyhow::bail!("Ansible installation not supported on this platform")
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub async fn uninstall_ansible(_host_config: &SshHostConfig) -> Result<()> {
    anyhow::bail!("Ansible uninstallation not supported on this platform")
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub async fn check_dure_wss_installed(_host_config: &SshHostConfig) -> Result<bool> {
    Ok(false)
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub async fn install_dure_wss(_host_config: &SshHostConfig) -> Result<()> {
    anyhow::bail!("Dure-WSS installation not supported on this platform")
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
pub async fn uninstall_dure_wss(_host_config: &SshHostConfig) -> Result<()> {
    anyhow::bail!("Dure-WSS uninstallation not supported on this platform")
}
