//! LEGO (Let's Encrypt Go) functionality
//!
//! Provides SSL certificate management using lego with SQLite-based storage
//! for certificate results and renewal tracking.
//!
//! Desktop-only module (requires ureq, dirs, and process execution)

#![cfg(not(any(target_os = "android", target_arch = "wasm32")))]

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Certificate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub domain: String,
    pub cert_path: String,
    pub key_path: String,
    pub issuer_path: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub is_valid: bool,
}

impl Certificate {
    pub fn new(
        domain: String,
        cert_path: String,
        key_path: String,
        issuer_path: String,
        issued_at: u64,
        expires_at: u64,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        Self {
            domain,
            cert_path,
            key_path,
            issuer_path,
            issued_at,
            expires_at,
            is_valid: now < expires_at,
        }
    }

    /// Check if certificate needs renewal (within 30 days of expiry)
    pub fn needs_renewal(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        // Renew if less than 30 days until expiry
        const RENEWAL_THRESHOLD: u64 = 30 * 24 * 60 * 60; // 30 days
        now + RENEWAL_THRESHOLD >= self.expires_at
    }

    /// Check if certificate is still valid
    pub fn is_still_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        now < self.expires_at
    }
}

/// DNS provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DnsProvider {
    Cloudflare,
    DuckDns,
    GoogleCloud,
    Porkbun,
}

impl DnsProvider {
    pub fn to_lego_name(&self) -> &str {
        match self {
            DnsProvider::Cloudflare => "cloudflare",
            DnsProvider::DuckDns => "duckdns",
            DnsProvider::GoogleCloud => "gcloud",
            DnsProvider::Porkbun => "porkbun",
        }
    }
}

/// Get lego binary path (downloads if needed)
pub fn get_lego_path(config_dir: &Path) -> Result<PathBuf> {
    let lego_path = config_dir.join("bin").join("lego");

    if !lego_path.exists() {
        download_lego(config_dir)?;
    }

    Ok(lego_path)
}

/// Download lego binary from GitHub releases
pub fn download_lego(config_dir: &Path) -> Result<()> {
    let bin_dir = config_dir.join("bin");
    std::fs::create_dir_all(&bin_dir)?;

    // Get latest release info
    let client = ureq::builder().build();
    let response = client
        .get("https://api.github.com/repos/go-acme/lego/releases/latest")
        .call()
        .context("Failed to fetch lego release info")?;

    let body = response.into_string()?;
    let release: serde_json::Value = serde_json::from_str(&body)?;
    let assets = release["assets"]
        .as_array()
        .context("No assets in release")?;

    // Determine OS and architecture
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let lego_os = match os {
        "linux" => "linux",
        "macos" => "darwin",
        "windows" => "windows",
        _ => return Err(anyhow::anyhow!("Unsupported OS: {}", os)),
    };

    let lego_arch = match arch {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        _ => return Err(anyhow::anyhow!("Unsupported architecture: {}", arch)),
    };

    let _pattern = format!("lego_.*_{lego_os}_{lego_arch}.tar.gz");

    let asset = assets
        .iter()
        .find(|a| {
            a["name"]
                .as_str()
                .map(|n| n.contains(lego_os) && n.contains(lego_arch))
                .unwrap_or(false)
        })
        .context("No matching lego binary found")?;

    let download_url = asset["browser_download_url"]
        .as_str()
        .context("No download URL")?;

    // Download the tarball
    let tmp_dir = config_dir.join("tmp");
    std::fs::create_dir_all(&tmp_dir)?;
    let tar_path = tmp_dir.join("lego.tar.gz");

    let response = client.get(download_url).call()?;
    let mut reader = response.into_reader();
    let mut file = std::fs::File::create(&tar_path)?;
    std::io::copy(&mut reader, &mut file)?;

    // Extract lego binary
    let tar_gz = std::fs::File::open(&tar_path)?;
    let tar = flate2::read::GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);

    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?;
        if path.file_name().and_then(|n| n.to_str()) == Some("lego") {
            let dest_path = bin_dir.join("lego");
            entry.unpack(&dest_path)?;

            // Make executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = std::fs::metadata(&dest_path)?.permissions();
                perms.set_mode(0o755);
                std::fs::set_permissions(&dest_path, perms)?;
            }

            dure_info!("Downloaded lego to: {}", dest_path.display());
            return Ok(());
        }
    }

    Err(anyhow::anyhow!("lego binary not found in tarball"))
}

/// Check if lego is installed
pub fn check_lego_installed(config_dir: &Path) -> bool {
    let lego_path = config_dir.join("bin").join("lego");
    lego_path.exists()
}

/// List certificates managed by lego
pub fn list_certificates(_config_dir: &Path) -> Result<Vec<String>> {
    let cert_dir = get_lego_dir().join("certificates");

    if !cert_dir.exists() {
        return Ok(vec![]);
    }

    let mut domains = Vec::new();
    for entry in std::fs::read_dir(cert_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Look for .crt files (not .issuer.crt)
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with(".crt") && !name.ends_with(".issuer.crt") {
                let domain = name.trim_end_matches(".crt");
                domains.push(domain.to_string());
            }
        }
    }

    Ok(domains)
}

/// Get lego working directory (~/.lego)
pub fn get_lego_dir() -> PathBuf {
    if let Ok(lego_dir) = std::env::var("LEGO_PATH") {
        PathBuf::from(lego_dir)
    } else {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".lego")
    }
}

/// Issue a new certificate using lego
pub fn issue_certificate(
    config_dir: &Path,
    email: &str,
    domain: &str,
    dns_provider: DnsProvider,
    env_vars: &[(&str, &str)],
) -> Result<Certificate> {
    let lego_path = get_lego_path(config_dir)?;

    let mut cmd = Command::new(&lego_path);
    cmd.arg("--email").arg(email);
    cmd.arg("--dns").arg(dns_provider.to_lego_name());
    cmd.arg("--domains").arg(domain);

    // Add wildcard domain if not already present
    if !domain.starts_with("*.") {
        cmd.arg("--domains").arg(format!("*.{}", domain));
    }

    // Accept TOS automatically
    cmd.arg("--accept-tos");

    cmd.arg("run");

    // Set environment variables for DNS provider
    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    let output = cmd.output().context("Failed to run lego")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("lego failed: {}", stderr));
    }

    // Parse certificate files
    get_certificate_info(domain)
}

/// Renew an existing certificate
pub fn renew_certificate(
    config_dir: &Path,
    domain: &str,
    dns_provider: DnsProvider,
    env_vars: &[(&str, &str)],
) -> Result<Certificate> {
    let lego_path = get_lego_path(config_dir)?;

    let mut cmd = Command::new(&lego_path);
    cmd.arg("--dns").arg(dns_provider.to_lego_name());
    cmd.arg("--domains").arg(domain);
    cmd.arg("renew");

    // Set environment variables for DNS provider
    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    let output = cmd.output().context("Failed to run lego")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("lego renewal failed: {}", stderr));
    }

    // Parse updated certificate files
    get_certificate_info(domain)
}

/// Get certificate information from lego directory
pub fn get_certificate_info(domain: &str) -> Result<Certificate> {
    let lego_dir = get_lego_dir();
    let cert_dir = lego_dir.join("certificates");

    let cert_path = cert_dir.join(format!("{}.crt", domain));
    let key_path = cert_dir.join(format!("{}.key", domain));
    let issuer_path = cert_dir.join(format!("{}.issuer.crt", domain));

    if !cert_path.exists() {
        return Err(anyhow::anyhow!(
            "Certificate not found for domain: {}",
            domain
        ));
    }

    // Parse certificate to get expiry date
    let cert_pem = std::fs::read_to_string(&cert_path)?;
    let (issued_at, expires_at) = parse_certificate_dates(&cert_pem)?;

    Ok(Certificate::new(
        domain.to_string(),
        cert_path.to_string_lossy().to_string(),
        key_path.to_string_lossy().to_string(),
        issuer_path.to_string_lossy().to_string(),
        issued_at,
        expires_at,
    ))
}

/// Parse certificate dates from PEM format
fn parse_certificate_dates(pem: &str) -> Result<(u64, u64)> {
    // Use openssl command to parse certificate
    use std::process::{Command, Stdio};

    let mut cmd = Command::new("openssl");
    cmd.arg("x509")
        .arg("-noout")
        .arg("-dates")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());

    let mut child = cmd.spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        use std::io::Write;
        stdin.write_all(pem.as_bytes())?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to parse certificate"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let mut issued_at = 0;
    let mut expires_at = 0;

    for line in stdout.lines() {
        if line.starts_with("notBefore=") {
            let date_str = line.trim_start_matches("notBefore=");
            issued_at = parse_openssl_date(date_str)?;
        } else if line.starts_with("notAfter=") {
            let date_str = line.trim_start_matches("notAfter=");
            expires_at = parse_openssl_date(date_str)?;
        }
    }

    Ok((issued_at, expires_at))
}

/// Parse OpenSSL date format to Unix timestamp
fn parse_openssl_date(date_str: &str) -> Result<u64> {
    use chrono::{DateTime, Utc};

    let dt = DateTime::parse_from_rfc2822(date_str)
        .or_else(|_| {
            // Try alternate format
            chrono::NaiveDateTime::parse_from_str(date_str, "%b %d %H:%M:%S %Y GMT")
                .map(|ndt| DateTime::<Utc>::from_naive_utc_and_offset(ndt, Utc).fixed_offset())
        })
        .context("Failed to parse certificate date")?;

    Ok(dt.timestamp() as u64)
}
