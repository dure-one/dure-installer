//! WebSocket Secure (WSS) server management functionality
//!
//! Provides WebSocket server functionality using TLS certificates from ACME,
//! with SQLite-based storage for session and connection tracking.

use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// WebSocket server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WssServerConfig {
    pub domain: String,
    pub bind_addr: String,
    pub bind_port: u16,
    pub server_id: String,
    pub ping_interval: u64,
    pub idle_timeout: u64,
    pub max_connections: usize,
}

impl WssServerConfig {
    pub fn new(domain: String) -> Self {
        Self {
            domain,
            bind_addr: "0.0.0.0".to_string(),
            bind_port: 443,
            server_id: format!("dure-{}", std::process::id()),
            ping_interval: 30,
            idle_timeout: 300,
            max_connections: 10000,
        }
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.bind_addr, self.bind_port)
    }
}

/// WebSocket session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WssSession {
    pub session_id: String,
    pub domain: String,
    pub connected_at: u64,
    pub last_seen: u64,
    pub message_count: u64,
    pub reconnect_count: u32,
}

impl WssSession {
    pub fn new(session_id: String, domain: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        Self {
            session_id,
            domain,
            connected_at: now,
            last_seen: now,
            message_count: 0,
            reconnect_count: 0,
        }
    }

    pub fn is_active(&self, timeout_secs: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        now - self.last_seen < timeout_secs
    }
}

/// WebSocket server status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WssServerStatus {
    pub domain: String,
    pub is_running: bool,
    pub pid: Option<u32>,
    pub started_at: Option<u64>,
    pub active_sessions: usize,
    pub total_connections: u64,
    pub db_path: Option<String>,
}

impl WssServerStatus {
    pub fn stopped(domain: String) -> Self {
        Self {
            domain,
            is_running: false,
            pid: None,
            started_at: None,
            active_sessions: 0,
            total_connections: 0,
            db_path: None,
        }
    }

    pub fn running(
        domain: String,
        pid: u32,
        started_at: u64,
        active_sessions: usize,
        total_connections: u64,
        db_path: Option<String>,
    ) -> Self {
        Self {
            domain,
            is_running: true,
            pid: Some(pid),
            started_at: Some(started_at),
            active_sessions,
            total_connections,
            db_path,
        }
    }
}

/// Path to the PID file for a given domain's WSS server process.
pub fn pid_file_path(domain: &str) -> Result<PathBuf> {
    let proj_dirs =
        ProjectDirs::from("pe", "nikescar", "dure").context("Failed to get project directories")?;
    let config_dir = proj_dirs.config_dir().to_path_buf();
    std::fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join(format!("wss-{}.pid", domain)))
}

/// Path to the server log file for a given domain.
pub fn log_file_path(domain: &str) -> Result<PathBuf> {
    let proj_dirs =
        ProjectDirs::from("pe", "nikescar", "dure").context("Failed to get project directories")?;
    let config_dir = proj_dirs.config_dir().to_path_buf();
    std::fs::create_dir_all(&config_dir)?;
    Ok(config_dir.join(format!("wss-{}.log", domain)))
}

/// Read `(pid, started_at, db_path)` from the PID file if it exists.
fn read_pid_file(domain: &str) -> Option<(u32, u64, Option<String>)> {
    let path = pid_file_path(domain).ok()?;
    let contents = std::fs::read_to_string(path).ok()?;
    let mut lines = contents.lines();
    let pid: u32 = lines.next()?.trim().parse().ok()?;
    let started_at: u64 = lines
        .next()
        .and_then(|l| l.trim().parse().ok())
        .unwrap_or(0);
    let db_path = lines
        .next()
        .map(|l| l.trim().to_string())
        .filter(|s| !s.is_empty());
    Some((pid, started_at, db_path))
}

/// Check whether a process with the given PID is alive.
fn process_alive(pid: u32) -> bool {
    // Use `kill -0` — works on all Unix systems without requiring libc
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get WebSocket server status
pub fn get_server_status(domain: &str) -> Result<WssServerStatus> {
    match read_pid_file(domain) {
        Some((pid, started_at, db_path)) if process_alive(pid) => Ok(WssServerStatus::running(
            domain.to_string(),
            pid,
            started_at,
            0,
            0,
            db_path,
        )),
        Some(_) => {
            // Stale PID file — process no longer running; clean up
            let _ = pid_file_path(domain).map(std::fs::remove_file);
            Ok(WssServerStatus::stopped(domain.to_string()))
        }
        None => Ok(WssServerStatus::stopped(domain.to_string())),
    }
}

/// Start WebSocket server
///
/// Spawns the current executable as a background daemon with `wss server <domain>`,
/// writes a PID file, and returns immediately.
pub fn start_server(config: &WssServerConfig) -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    anyhow::bail!("WebSocket server is only supported on Linux");

    #[cfg(target_os = "linux")]
    {
        use std::os::unix::process::CommandExt;
        use std::process::{Command, Stdio};

        let current_exe =
            std::env::current_exe().context("Failed to get current executable path")?;

        let addr = config.bind_address();

        // Append server output to a per-domain log file
        let log_path = log_file_path(&config.domain)?;
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .with_context(|| format!("Failed to open log file {:?}", log_path))?;

        // Spawn: <current_exe> wss server <domain> --addr <addr>
        // process_group(0) puts the child in its own process group so SIGINT
        // from the terminal does not propagate to it.
        let child = Command::new(&current_exe)
            .args(["wss", "server", &config.domain, "--addr", &addr])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(log_file)
            .process_group(0)
            .spawn()
            .context("Failed to spawn server process")?;

        let pid = child.id();
        let started_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let pid_path = pid_file_path(&config.domain)?;
        std::fs::write(&pid_path, format!("{}\n{}", pid, started_at))
            .context("Failed to write PID file")?;

        eprintln!("Server PID: {}", pid);
        eprintln!("Log file:   {:?}", log_path);

        Ok(())
    }
}

/// Stop WebSocket server
///
/// Sends SIGTERM to the server process identified by the PID file and removes the file.
pub fn stop_server(domain: &str) -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    anyhow::bail!("WebSocket server is only supported on Linux");

    #[cfg(target_os = "linux")]
    {
        let (pid, _, _) = read_pid_file(domain)
            .ok_or_else(|| anyhow::anyhow!("No PID file found for domain {}", domain))?;

        if !process_alive(pid) {
            // Already dead — just clean up the stale PID file
            let _ = pid_file_path(domain).map(std::fs::remove_file);
            eprintln!(
                "Process {} was already stopped (stale PID file removed)",
                pid
            );
            return Ok(());
        }

        // Send SIGTERM
        let status = std::process::Command::new("kill")
            .args(["-15", &pid.to_string()])
            .status()
            .context("Failed to send SIGTERM")?;

        if !status.success() {
            anyhow::bail!("kill -15 {} failed", pid);
        }

        // Wait up to 5 s for the process to exit
        for _ in 0..10 {
            std::thread::sleep(Duration::from_millis(500));
            if !process_alive(pid) {
                break;
            }
        }

        if process_alive(pid) {
            eprintln!(
                "Warning: process {} did not stop within 5 s, sending SIGKILL",
                pid
            );
            let _ = std::process::Command::new("kill")
                .args(["-9", &pid.to_string()])
                .status();
        }

        // Remove PID file
        let _ = pid_file_path(domain).map(std::fs::remove_file);

        Ok(())
    }
}

/// Generate a unique session ID
pub fn generate_session_id() -> String {
    use rand::Rng;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let random: u32 = rand::thread_rng().r#gen();
    format!("session-{}-{}", timestamp, random)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wss_server_config() {
        let config = WssServerConfig::new("example.com".to_string());
        assert_eq!(config.domain, "example.com");
        assert_eq!(config.bind_port, 443);
        assert_eq!(config.bind_address(), "0.0.0.0:443");
    }

    #[test]
    fn test_wss_session_creation() {
        let session = WssSession::new("test-session".to_string(), "example.com".to_string());
        assert_eq!(session.session_id, "test-session");
        assert_eq!(session.domain, "example.com");
        assert_eq!(session.message_count, 0);
        assert_eq!(session.reconnect_count, 0);
    }

    #[test]
    fn test_session_is_active() {
        let session = WssSession::new("test".to_string(), "example.com".to_string());
        assert!(session.is_active(3600)); // Active within 1 hour
    }

    #[test]
    fn test_generate_session_id() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();
        assert!(id1.starts_with("session-"));
        assert_ne!(id1, id2); // Should be unique
    }

    #[test]
    fn test_server_status_stopped() {
        let status = WssServerStatus::stopped("example.com".to_string());
        assert!(!status.is_running);
        assert!(status.pid.is_none());
        assert!(status.started_at.is_none());
    }

    #[test]
    fn test_server_status_running() {
        let status = WssServerStatus::running(
            "example.com".to_string(),
            1234,
            1000,
            5,
            100,
            Some("/path/to/db".to_string()),
        );
        assert!(status.is_running);
        assert_eq!(status.pid, Some(1234));
        assert_eq!(status.started_at, Some(1000));
        assert_eq!(status.active_sessions, 5);
        assert_eq!(status.total_connections, 100);
    }
}
