//! SSH tunnel implementation using russh + smol
//!
//! Implements local port forwarding: localhost:local_port -> remote_host:remote_port via SSH
//! Uses async-compat to bridge russh (tokio-based) with smol runtime

use anyhow::{Context, Result};
use russh::client::{self, Handle};
use russh_keys::key::PublicKey;
use smol::net::TcpListener;
use std::net::ToSocketAddrs;
use std::sync::Arc;

use crate::config::SshHostConfig;
use crate::{dure_debug, dure_error, dure_info, dure_warn};

/// SSH tunnel handle for managing tunnel lifecycle
pub struct TunnelHandle {
    local_port: u16,
    remote_host: String,
    remote_port: u16,
    cancel_sender: smol::channel::Sender<()>,
}

impl TunnelHandle {
    /// Stop the tunnel
    pub async fn stop(self) -> Result<()> {
        self.cancel_sender.send(()).await.ok();
        dure_info!(
            "Tunnel stopped: localhost:{} -> {}:{}",
            self.local_port,
            self.remote_host,
            self.remote_port
        );
        Ok(())
    }

    pub fn local_port(&self) -> u16 {
        self.local_port
    }

    pub fn remote_host(&self) -> &str {
        &self.remote_host
    }

    pub fn remote_port(&self) -> u16 {
        self.remote_port
    }
}

/// SSH client handler for tunnel connections
struct TunnelClient;

#[async_trait::async_trait]
impl client::Handler for TunnelClient {
    type Error = anyhow::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &PublicKey,
    ) -> Result<bool, Self::Error> {
        // ponytail: Accept all keys for MVP, add known_hosts verification later
        Ok(true)
    }
}

/// Start SSH tunnel: localhost:local_port -> remote_host:remote_port via SSH
pub async fn start_tunnel(
    host_config: &SshHostConfig,
    local_port: u16,
    remote_host: String,
    remote_port: u16,
    profile_keyring: Option<&Arc<crate::calc::keyring::DatabaseHandle>>,
) -> Result<TunnelHandle> {
    let (username, hostname) = parse_ssh_host(&host_config.host)?;

    let ssh_address = format!("{}:{}", hostname, host_config.port);
    let socket_addrs: Vec<_> = ssh_address
        .to_socket_addrs()
        .context("Failed to resolve SSH hostname")?
        .collect();
    let ssh_socket_addr = *socket_addrs
        .first()
        .ok_or_else(|| anyhow::anyhow!("Could not resolve SSH address: {}", ssh_address))?;

    let local_bind_addr = format!("127.0.0.1:{}", local_port);

    dure_info!(
        "Starting tunnel: {} -> {}:{} via {}",
        local_bind_addr,
        remote_host,
        remote_port,
        host_config.host
    );

    // Connect to SSH server inside async-compat wrapper
    let host_config_clone = host_config.clone();
    let profile_keyring = profile_keyring.cloned();

    let session = async_compat::Compat::new(async move {
        let config = Arc::new(russh::client::Config::default());
        let mut handle = client::connect(config, ssh_socket_addr, TunnelClient)
            .await
            .context("Failed to connect to SSH server")?;

        // Authenticate
        authenticate_tunnel(&mut handle, &username, &host_config_clone, profile_keyring.as_ref())
            .await
            .context("SSH authentication failed")?;

        Ok::<_, anyhow::Error>(handle)
    })
    .await?;

    let session = Arc::new(session);

    // Bind local TCP listener
    let listener = TcpListener::bind(&local_bind_addr)
        .await
        .context("Failed to bind local port")?;

    dure_info!("Tunnel listening on {}", local_bind_addr);

    // Create cancel channel
    let (cancel_tx, cancel_rx) = smol::channel::bounded(1);

    // Spawn tunnel acceptor task
    let session_clone = Arc::clone(&session);
    let remote_host_clone = remote_host.clone();
    smol::spawn(async move {
        let result = run_tunnel_acceptor(
            listener,
            session_clone,
            remote_host_clone,
            remote_port,
            cancel_rx,
        )
        .await;

        if let Err(e) = result {
            dure_error!("Tunnel acceptor error: {}", e);
        }
    })
    .detach();

    Ok(TunnelHandle {
        local_port,
        remote_host,
        remote_port,
        cancel_sender: cancel_tx,
    })
}

/// Run the tunnel acceptor loop
async fn run_tunnel_acceptor(
    listener: TcpListener,
    session: Arc<Handle<TunnelClient>>,
    remote_host: String,
    remote_port: u16,
    cancel_rx: smol::channel::Receiver<()>,
) -> Result<()> {
    loop {
        // Poll for cancel signal (non-blocking)
        if cancel_rx.try_recv().is_ok() {
            dure_info!("Tunnel acceptor received cancel signal");
            break;
        }

        // Accept with timeout to check cancel periodically
        let timeout = std::time::Duration::from_millis(500);
        let accept_result = smol::future::or(
            async {
                listener.accept().await.map(Some)
            },
            async {
                smol::Timer::after(timeout).await;
                Ok::<Option<_>, std::io::Error>(None)
            },
        )
        .await;

        match accept_result {
            Ok(Some((local_stream, peer_addr))) => {
                dure_debug!("Tunnel: accepted connection from {}", peer_addr);

                let session = Arc::clone(&session);
                let remote_host = remote_host.clone();

                smol::spawn(async move {
                    if let Err(e) = handle_tunnel_connection(
                        session,
                        local_stream,
                        peer_addr.to_string(),
                        peer_addr.port() as u32,
                        remote_host,
                        remote_port,
                    )
                    .await
                    {
                        dure_error!("Tunnel connection error for {}: {}", peer_addr, e);
                    }
                })
                .detach();
            }
            Ok(None) => {
                // Timeout, loop to check cancel signal
                continue;
            }
            Err(e) => {
                dure_error!("Tunnel accept error: {}", e);
                return Err(e.into());
            }
        }
    }

    Ok(())
}

/// Handle a single tunnel connection
async fn handle_tunnel_connection(
    session: Arc<Handle<TunnelClient>>,
    local_stream: smol::net::TcpStream,
    originator_addr: String,
    originator_port: u32,
    target_host: String,
    target_port: u16,
) -> Result<()> {
    // Open direct-tcpip channel (requires async-compat wrapper)
    let channel = async_compat::Compat::new(async {
        session
            .channel_open_direct_tcpip(
                target_host.clone(),
                target_port as u32,
                originator_addr,
                originator_port,
            )
            .await
            .context("Failed to open SSH channel")
    })
    .await?;

    dure_debug!(
        "Tunnel: opened channel to {}:{}",
        target_host,
        target_port
    );

    // Convert russh channel to futures-io compatible stream
    use async_compat::CompatExt;
    let ssh_stream = channel.into_stream().compat();

    // Bidirectional copy
    use futures::{AsyncReadExt as FutAsyncReadExt, AsyncWriteExt as FutAsyncWriteExt};
    use smol::io::{AsyncReadExt as SmolAsyncReadExt, AsyncWriteExt as SmolAsyncWriteExt};

    let (mut local_read, mut local_write) = local_stream.split();
    let (mut ssh_read, mut ssh_write) = ssh_stream.split();

    // Copy client -> server
    let client_to_server = async {
        let mut buffer = vec![0u8; 8192];
        loop {
            let n = SmolAsyncReadExt::read(&mut local_read, &mut buffer).await?;
            if n == 0 {
                break;
            }
            FutAsyncWriteExt::write_all(&mut ssh_write, &buffer[..n]).await?;
            FutAsyncWriteExt::flush(&mut ssh_write).await?;
        }
        FutAsyncWriteExt::close(&mut ssh_write).await?;
        Ok::<_, anyhow::Error>(())
    };

    // Copy server -> client
    let server_to_client = async {
        let mut buffer = vec![0u8; 8192];
        loop {
            let n = FutAsyncReadExt::read(&mut ssh_read, &mut buffer).await?;
            if n == 0 {
                break;
            }
            SmolAsyncWriteExt::write_all(&mut local_write, &buffer[..n]).await?;
            SmolAsyncWriteExt::flush(&mut local_write).await?;
        }
        SmolAsyncWriteExt::close(&mut local_write).await?;
        Ok::<_, anyhow::Error>(())
    };

    // Run both directions concurrently
    let result = smol::future::race(client_to_server, server_to_client).await;

    if let Err(e) = result {
        dure_debug!("Tunnel connection ended with error: {}", e);
    } else {
        dure_debug!("Tunnel connection closed cleanly");
    }

    Ok(())
}

/// Parse "user@host" into (user, host)
fn parse_ssh_host(host: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = host.split('@').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid SSH host format, expected 'user@host'");
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Load private key from keyring by domain
fn load_private_key_from_keyring(
    domain: &str,
    profile_keyring: Option<&Arc<crate::calc::keyring::DatabaseHandle>>,
) -> Result<String> {
    use crate::calc::keyring;

    let keys = if let Some(handle) = profile_keyring {
        keyring::list_keys_from_handle(handle)
            .context("Failed to list keys from profile keyring")?
    } else {
        let kdbx_path = keyring::ensure_kdbx_exists()?;
        let kpkey_path = keyring::get_default_kpkey_path()?;
        keyring::list_keys(&kdbx_path, Some(&kpkey_path), None)
            .context("Failed to list keys from keyring")?
    };

    let key_entry = keys
        .iter()
        .find(|k| k.domain == domain)
        .ok_or_else(|| anyhow::anyhow!("Key not found in keyring for domain '{}'", domain))?;

    let ssh_key_bytes = key_entry
        .ssh_key
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("SSH key not found in keyring entry"))?;

    String::from_utf8(ssh_key_bytes.clone())
        .context("Invalid UTF-8 in SSH key")
}

/// Authenticate SSH session (reused from calc/ssh.rs pattern)
async fn authenticate_tunnel(
    session: &mut Handle<TunnelClient>,
    username: &str,
    host_config: &SshHostConfig,
    profile_keyring: Option<&Arc<crate::calc::keyring::DatabaseHandle>>,
) -> Result<()> {
    // Try keyring first (if available)
    if let Some(keyring_domain) = &host_config.keyring_domain {
        match load_private_key_from_keyring(keyring_domain, profile_keyring) {
            Ok(private_key_pem) => {
                let key_pair = russh_keys::decode_secret_key(&private_key_pem, None)
                    .context("Failed to decode private key from keyring")?;

                let auth_result = session
                    .authenticate_publickey(username, Arc::new(key_pair))
                    .await
                    .context("Public key authentication failed")?;

                if auth_result {
                    return Ok(());
                } else {
                    anyhow::bail!("SSH authentication rejected");
                }
            }
            Err(e) => {
                dure_warn!("Failed to retrieve key from keyring: {}", e);
            }
        }
    }

    // Try private key file
    if let Some(key_path) = &host_config.private_key_path {
        let key_path = shellexpand::tilde(key_path).to_string();
        let private_key_pem = std::fs::read_to_string(&key_path)
            .context(format!("Failed to read private key file: {}", key_path))?;

        let key_pair = russh_keys::decode_secret_key(&private_key_pem, None)
            .context("Failed to decode private key")?;

        let auth_result = session
            .authenticate_publickey(username, Arc::new(key_pair))
            .await
            .context("Public key authentication failed")?;

        if auth_result {
            return Ok(());
        } else {
            anyhow::bail!("SSH authentication rejected");
        }
    }

    // Try password
    if let Some(password) = &host_config.password {
        let auth_result = session
            .authenticate_password(username, password)
            .await
            .context("Password authentication failed")?;

        if auth_result {
            return Ok(());
        } else {
            anyhow::bail!("SSH authentication rejected");
        }
    }

    anyhow::bail!("No authentication method available")
}
