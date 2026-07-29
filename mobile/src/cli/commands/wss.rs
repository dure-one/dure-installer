//! WebSocket Secure (WSS) command implementation

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use crate::calc::db;
use crate::calc::wss::{WssServerConfig, get_server_status, start_server};
use crate::storage::models::acme::get_certificate;
use crate::storage::models::session::{list_active_sessions, list_sessions_for_domain};
use crate::storage::models::wss::{
    cleanup_old_sessions, get_server_config, init_wss_tables, list_server_configs,
    store_server_config,
};
use anyhow::Result;
use diesel::prelude::*;

/// Execute WSS status command
///
/// Displays the current WebSocket server status.
pub fn execute_wss_status(domain: Option<String>) -> Result<()> {
    let mut conn = db::establish_connection();

    init_wss_tables(&mut conn)?;

    if let Some(domain) = domain {
        // Show status for specific domain
        dure_info!("WebSocket Server Status for: {}", domain);
        dure_info!("");

        let status = get_server_status(&domain)?;

        // Use the DB path stored in the PID file so we query the server's actual DB.
        if let Some(ref db) = status.db_path {
            crate::calc::db::set_db_path(db.clone());
            conn = crate::calc::db::establish_connection();
        }

        if status.is_running {
            println!("Status: RUNNING");
            if let Some(pid) = status.pid {
                println!("PID: {}", pid);
            }
            if let Some(started_at) = status.started_at {
                let started = chrono::DateTime::from_timestamp(started_at as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());
                println!("Started: {}", started);
            }

            let active = list_active_sessions(&mut conn, 300).unwrap_or_default();
            let active_for_domain: Vec<_> = active.iter().filter(|s| s.domain == domain).collect();
            let total_sessions = list_sessions_for_domain(&mut conn, &domain).unwrap_or_default();
            let http_count = total_sessions
                .iter()
                .filter(|s| s.session_type == "http")
                .count();
            let wss_count = total_sessions
                .iter()
                .filter(|s| s.session_type == "wss")
                .count();

            println!("Active Sessions: {}", active_for_domain.len());
            println!("Total Connections: {}", total_sessions.len());
            println!("  HTTP: {}  WSS: {}", http_count, wss_count);
        } else {
            println!("Status: STOPPED");
        }

        // Show server configuration
        if let Some(config) = get_server_config(&mut conn, &domain)? {
            println!();
            println!("Configuration:");
            println!("  Bind Address: {}", config.bind_address());
            println!("  Server ID: {}", config.server_id);
            println!("  Ping Interval: {}s", config.ping_interval);
            println!("  Idle Timeout: {}s", config.idle_timeout);
            println!("  Max Connections: {}", config.max_connections);
        }

        // Show recent sessions
        let sessions = list_sessions_for_domain(&mut conn, &domain).unwrap_or_default();
        if !sessions.is_empty() {
            println!();
            println!("Recent Sessions ({}):", sessions.len().min(10));
            for session in sessions.iter().take(10) {
                let last_seen = chrono::DateTime::from_timestamp(session.last_seen as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                println!(
                    "  [{}] {} from {} - {} requests (last seen: {})",
                    session.session_type,
                    session.session_id,
                    session.remote_addr,
                    session.request_count,
                    last_seen
                );
            }
        }
    } else {
        // Show status for all servers
        let configs = list_server_configs(&mut conn)?;

        if configs.is_empty() {
            println!("No WebSocket servers configured");
            println!();
            println!("To start a WebSocket server:");
            println!("  dure wss start <domain>");
            return Ok(());
        }

        println!("WebSocket Servers:");
        println!();

        for config in configs {
            let status = get_server_status(&config.domain)?;

            print!("  {} - ", config.domain);
            if status.is_running {
                println!("RUNNING (active sessions: {})", status.active_sessions);
            } else {
                println!("STOPPED");
            }
        }
    }

    Ok(())
}

/// Execute WSS start command
///
/// Starts a WebSocket server for the specified domain using ACME certificates.
pub fn execute_wss_start(
    domain: String,
    bind_addr: Option<String>,
    bind_port: Option<u16>,
) -> Result<()> {
    let mut conn = db::establish_connection();

    init_wss_tables(&mut conn)?;

    // Check if server is already running
    let status = get_server_status(&domain)?;
    if status.is_running {
        dure_info!("WebSocket server for {} is already running", domain);
        return Ok(());
    }

    // Check for ACME certificate
    dure_info!("Checking for TLS certificate...");

    let cert = get_certificate(&mut conn, &domain)?;
    let cert = cert.ok_or_else(|| {
        anyhow::anyhow!(
            "No TLS certificate found for domain {}. Run 'dure acme issue {}' first.",
            domain,
            domain
        )
    })?;

    // Check if certificate is valid
    if !cert.is_still_valid() {
        anyhow::bail!(
            "TLS certificate for {} has expired. Run 'dure acme renew {}' to renew.",
            domain,
            domain
        );
    }

    dure_info!(" Found valid TLS certificate");
    dure_info!("  Certificate: {}", cert.cert_path);
    dure_info!("  Private Key: {}", cert.key_path);
    dure_info!("");

    // Create or update server configuration
    let mut config = get_server_config(&mut conn, &domain)?
        .unwrap_or_else(|| WssServerConfig::new(domain.clone()));

    if let Some(addr) = bind_addr {
        config.bind_addr = addr;
    }

    if let Some(port) = bind_port {
        config.bind_port = port;
    }

    store_server_config(&mut conn, &config)?;

    dure_info!("Starting WebSocket server...");
    dure_info!("");

    // Start the server
    start_server(&config)?;

    dure_info!("");
    dure_info!(" WebSocket server started successfully");
    dure_info!("  Domain: {}", config.domain);
    dure_info!("  Address: wss://{}", config.bind_address());

    Ok(())
}

/// Execute WSS server command — runs the HTTPS/WSS server in-process.
///
/// This is called when the user runs `dure wss server <domain>` directly,
/// or when `execute_wss_start` spawns a background subprocess.
pub fn execute_wss_server(
    domain: String,
    addr: Option<String>,
    no_download: bool,
    stats_interval: Option<u64>,
) -> Result<()> {
    let db_path = crate::calc::db::get_db_path();
    let addr = addr.unwrap_or_else(|| "0.0.0.0:8443".to_string());

    crate::wss::server::run_with_args(crate::wss::server::RunArgs {
        domain,
        addr,
        db_path,
        stats_interval: stats_interval.unwrap_or(60),
        download_static: !no_download,
    })
    .map_err(|e| anyhow::anyhow!("{}", e))
}

/// Execute WSS client command
///
/// Test client for HTTPS GET/POST and WebSocket connections.
pub fn execute_wss_client(
    url: String,
    mode: Option<String>,
    path: Option<String>,
    body: Option<String>,
    insecure: bool,
) -> Result<()> {
    let client_mode = mode.as_deref().unwrap_or("ws");
    let request_path = path.unwrap_or_else(|| "/".to_string());
    let request_body = body.unwrap_or_else(|| r#"{"test":"data"}"#.to_string());

    crate::wss::client::run_with_args(url, client_mode, request_path, request_body, insecure)
        .map_err(|e| anyhow::anyhow!("{}", e))
}

/// Execute WSS stop command
///
/// Stops the WebSocket server for the specified domain.
pub fn execute_wss_stop(domain: String) -> Result<()> {
    let mut conn = db::establish_connection();

    init_wss_tables(&mut conn)?;

    // Check if server is running
    let status = get_server_status(&domain)?;
    if !status.is_running {
        dure_info!("WebSocket server for {} is not running", domain);
        return Ok(());
    }

    dure_info!("Stopping WebSocket server for {}...", domain);

    // Stop the server
    crate::calc::wss::stop_server(&domain)?;

    // Cleanup old sessions
    dure_info!("Cleaning up sessions...");
    let deleted = cleanup_old_sessions(&mut conn, 0)?; // Delete all sessions
    dure_info!("Cleaned up {} sessions", deleted);

    dure_info!("");
    dure_info!(" WebSocket server stopped successfully");

    Ok(())
}

fn get_db_path() -> Result<std::path::PathBuf> {
    let db_path = crate::calc::db::get_db_path();
    Ok(std::path::PathBuf::from(db_path))
}

fn get_acme_db_path() -> Result<std::path::PathBuf> {
    let db_path = crate::calc::db::get_db_path();
    Ok(std::path::PathBuf::from(db_path))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_db_path() {
        let path = get_db_path().unwrap();
        assert!(path.to_string_lossy().contains("dure"));
        assert!(path.to_string_lossy().ends_with(".db"));
    }

    #[test]
    fn test_get_acme_db_path() {
        let path = get_acme_db_path().unwrap();
        assert!(path.to_string_lossy().contains("dure"));
        assert!(path.to_string_lossy().ends_with(".db"));
    }
}
