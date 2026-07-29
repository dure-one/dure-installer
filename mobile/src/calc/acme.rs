//! ACME (Automatic Certificate Management Environment) functionality
//!
//! Provides SSL certificate management using acme.sh with SQLite-based storage
//! for certificate results and renewal tracking.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::{Context, Result};
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::api::ns_cloudflare::CloudflareClient;
use crate::api::ns_duckdns::DuckDnsClient;
use crate::api::ns_porkbun::PorkbunClient;

/// Certificate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    pub domain: String,
    pub cert_path: String,
    pub key_path: String,
    pub ca_path: String,
    pub fullchain_path: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub is_valid: bool,
}

impl Certificate {
    pub fn new(
        domain: String,
        cert_path: String,
        key_path: String,
        ca_path: String,
        fullchain_path: String,
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
            ca_path,
            fullchain_path,
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

/// Represents a certificate entry from `acme.sh --list`
#[derive(Debug, Clone)]
pub struct AcmeListEntry {
    pub main_domain: String,
    pub key_length: String,
    pub san_domains: Vec<String>,
    /// Unix timestamp parsed from acme.sh Created field
    pub created_at: Option<u64>,
    /// Unix timestamp parsed from acme.sh Renew field
    pub renew_at: Option<u64>,
    /// Original Created string for display
    pub created_str: String,
    /// Original Renew string for display
    pub renew_str: String,
}

/// Check if acme.sh is installed on the system.
///
/// Mirrors acme.sh's own install detection logic:
/// - First checks `LE_WORKING_DIR` environment variable
/// - Falls back to checking `~/.acme.sh/account.conf`
///
/// Returns the working directory path if acme.sh is installed, or `None` otherwise.
pub fn check_acme_installed() -> Option<String> {
    // Check LE_WORKING_DIR env var first (user may have custom install location)
    if let Ok(le_dir) = std::env::var("LE_WORKING_DIR") {
        if std::path::Path::new(&format!("{}/account.conf", le_dir)).exists() {
            return Some(le_dir);
        }
    }

    // Check default install location: ~/.acme.sh
    if let Ok(home) = std::env::var("HOME") {
        let default_home = format!("{}/.acme.sh", home);
        if std::path::Path::new(&format!("{}/account.conf", default_home)).exists() {
            return Some(default_home);
        }
    }

    None
}

/// Get the path to the acme.sh binary, respecting LE_WORKING_DIR.
pub fn get_acme_sh_path() -> Result<String> {
    let working_dir = check_acme_installed().unwrap_or_else(|| {
        std::env::var("HOME")
            .map(|h| format!("{}/.acme.sh", h))
            .unwrap_or_default()
    });
    Ok(format!("{}/acme.sh", working_dir))
}

/// List certificates currently managed by the acme.sh system.
///
/// Runs `acme.sh --list` and parses its output into structured entries.
///
/// # Examples
///
/// ```rust,no_run
/// use dure::calc::acme::list_system_certificates;
///
/// # fn example() -> anyhow::Result<()> {
/// let certs = list_system_certificates()?;
/// for cert in certs {
///     println!("{}: created {}", cert.main_domain, cert.created_str);
/// }
/// # Ok(())
/// # }
/// ```
pub fn list_system_certificates() -> Result<Vec<AcmeListEntry>> {
    #[cfg(not(target_os = "linux"))]
    {
        anyhow::bail!("acme.sh is only supported on Linux");
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        let acme_sh = get_acme_sh_path()?;

        let output = Command::new(&acme_sh)
            .arg("--list")
            .output()
            .context("Failed to execute acme.sh --list")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("acme.sh --list failed: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_acme_list_output(&stdout)
    }
}

/// Parse the output of `acme.sh --list` into structured entries.
///
/// acme.sh uses fixed-width padded columns separated by 2+ spaces.
fn parse_acme_list_output(output: &str) -> Result<Vec<AcmeListEntry>> {
    let mut entries = Vec::new();

    for (i, line) in output.lines().enumerate() {
        if i == 0 {
            continue; // skip header row
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let cols = split_padded_columns(line);
        if cols.is_empty() {
            continue;
        }

        let main_domain = cols.first().cloned().unwrap_or_default();
        let key_length = cols.get(1).cloned().unwrap_or_default();
        let san_str = cols.get(2).cloned().unwrap_or_default();
        let created_str = cols.get(3).cloned().unwrap_or_default();
        let renew_str = cols.get(4).cloned().unwrap_or_default();

        let san_domains = san_str.split_whitespace().map(|s| s.to_string()).collect();

        entries.push(AcmeListEntry {
            main_domain,
            key_length,
            san_domains,
            created_at: parse_acme_date(&created_str),
            renew_at: parse_acme_date(&renew_str),
            created_str,
            renew_str,
        });
    }

    Ok(entries)
}

/// Split a padded-column line by runs of 2+ spaces.
///
/// acme.sh `--list` output uses printf-style fixed-width columns separated by
/// at least two spaces, so single spaces may appear within a column value (e.g.
/// SAN domain lists).
fn split_padded_columns(line: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut space_count = 0usize;

    for ch in line.chars() {
        if ch == ' ' {
            space_count += 1;
            if space_count < 2 {
                current.push(ch);
            }
        } else {
            if space_count >= 2 {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    result.push(trimmed);
                }
                current = String::new();
            }
            space_count = 0;
            current.push(ch);
        }
    }

    let trimmed = current.trim().to_string();
    if !trimmed.is_empty() {
        result.push(trimmed);
    }

    result
}

/// Parse an acme.sh date string (`YYYY-MM-DD,HH:MM:SS`) into a Unix timestamp.
fn parse_acme_date(date_str: &str) -> Option<u64> {
    // acme.sh format: "2024-01-15,06:31:19"
    let s = date_str.replace(',', " ");
    let (date_part, time_part) = s.split_once(' ')?;

    let mut date_nums = date_part.split('-');
    let year: i32 = date_nums.next()?.parse().ok()?;
    let month: u32 = date_nums.next()?.parse().ok()?;
    let day: u32 = date_nums.next()?.parse().ok()?;

    let mut time_nums = time_part.split(':');
    let hour: u32 = time_nums.next()?.parse().ok()?;
    let min: u32 = time_nums.next()?.parse().ok()?;
    let sec: u32 = time_nums.next()?.parse().ok()?;

    let dt = Utc
        .with_ymd_and_hms(year, month, day, hour, min, sec)
        .single()?;
    Some(dt.timestamp() as u64)
}

/// Install acme.sh to the system
///
/// This function will download and install acme.sh if not already installed.
/// Requires root/sudo privileges on Linux.
///
/// # Examples
///
/// ```rust,no_run
/// use dure::calc::acme::install_acme;
///
/// # async fn example() -> anyhow::Result<()> {
/// install_acme()?;
/// # Ok(())
/// # }
/// ```
pub fn install_acme() -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    {
        anyhow::bail!("ACME installation is only supported on Linux");
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        // Check if acme.sh is already installed
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let acme_sh_path = format!("{}/.acme.sh/acme.sh", home);

        if std::path::Path::new(&acme_sh_path).exists() {
            dure_info!("acme.sh is already installed at {}", acme_sh_path);
            return Ok(());
        }

        // Install acme.sh using the official installation script
        dure_info!("Installing acme.sh...");

        let output = Command::new("sh")
            .arg("-c")
            .arg("curl https://get.acme.sh | sh -s email=my@example.com")
            .output()
            .context("Failed to execute acme.sh installation")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to install acme.sh: {}", stderr);
        }

        dure_info!("acme.sh installed successfully");
        Ok(())
    }
}

/// Issue a new certificate for the specified domain
///
/// Uses standalone mode which requires port 80/443 to be available.
/// Supports multiple domains with -d flags.
///
/// # Examples
///
/// ```rust,no_run
/// use dure::calc::acme::issue_certificate;
///
/// # async fn example() -> anyhow::Result<()> {
/// issue_certificate(&["example.com", "www.example.com"])?;
/// # Ok(())
/// # }
/// ```
pub fn issue_certificate(domains: &[&str]) -> Result<Certificate> {
    #[cfg(not(target_os = "linux"))]
    {
        anyhow::bail!("ACME certificate issuance is only supported on Linux");
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        if domains.is_empty() {
            anyhow::bail!("At least one domain must be specified");
        }

        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let acme_sh = format!("{}/.acme.sh/acme.sh", home);

        // Build domain arguments
        let mut domain_args = Vec::new();
        for domain in domains {
            domain_args.push("-d");
            domain_args.push(*domain);
        }

        dure_info!("Issuing certificate for: {}", domains.join(", "));

        let output = Command::new(&acme_sh)
            .arg("--issue")
            .arg("--standalone")
            .args(&domain_args)
            .output()
            .context("Failed to execute acme.sh --issue")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to issue certificate: {}", stderr);
        }

        // Parse certificate paths from acme.sh directory structure
        let primary_domain = domains[0];
        let cert_dir = format!("{}/.acme.sh/{}", home, primary_domain);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        // Certificates are valid for 90 days
        let expires_at = now + 90 * 24 * 60 * 60;

        Ok(Certificate::new(
            primary_domain.to_string(),
            format!("{}/{}.cer", cert_dir, primary_domain),
            format!("{}/{}.key", cert_dir, primary_domain),
            format!("{}/ca.cer", cert_dir),
            format!("{}/fullchain.cer", cert_dir),
            now,
            expires_at,
        ))
    }
}

/// Renew an existing certificate for the specified domain
///
/// Typically should be run every 60 days. acme.sh will check if renewal is needed.
///
/// # Examples
///
/// ```rust,no_run
/// use dure::calc::acme::renew_certificate;
///
/// # async fn example() -> anyhow::Result<()> {
/// renew_certificate("example.com", false)?;
/// # Ok(())
/// # }
/// ```
pub fn renew_certificate(domain: &str, force: bool) -> Result<Certificate> {
    #[cfg(not(target_os = "linux"))]
    {
        anyhow::bail!("ACME certificate renewal is only supported on Linux");
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let acme_sh = format!("{}/.acme.sh/acme.sh", home);

        dure_info!("Renewing certificate for: {}", domain);

        let mut cmd = Command::new(&acme_sh);
        cmd.arg("--renew").arg("-d").arg(domain);

        if force {
            cmd.arg("--force");
        }

        let output = cmd.output().context("Failed to execute acme.sh --renew")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to renew certificate: {}", stderr);
        }

        let cert_dir = format!("{}/.acme.sh/{}", home, domain);

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        let expires_at = now + 90 * 24 * 60 * 60;

        Ok(Certificate::new(
            domain.to_string(),
            format!("{}/{}.cer", cert_dir, domain),
            format!("{}/{}.key", cert_dir, domain),
            format!("{}/ca.cer", cert_dir),
            format!("{}/fullchain.cer", cert_dir),
            now,
            expires_at,
        ))
    }
}

/// DNS provider configuration for setting records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsProvider {
    pub provider_type: DnsProviderType,
    pub api_token: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DnsProviderType {
    Cloudflare,
    GoogleCloud,
    DuckDNS,
    Porkbun,
}

impl DnsProviderType {
    pub fn as_str(&self) -> &'static str {
        match self {
            DnsProviderType::Cloudflare => "cloudflare",
            DnsProviderType::GoogleCloud => "gcloud",
            DnsProviderType::DuckDNS => "duckdns",
            DnsProviderType::Porkbun => "porkbun",
        }
    }
}

/// Set A record for a domain
///
/// # Examples
///
/// ```rust,no_run
/// use dure::calc::acme::{DnsProvider, DnsProviderType, set_a_record};
///
/// # async fn example() -> anyhow::Result<()> {
/// let provider = DnsProvider {
///     provider_type: DnsProviderType::Cloudflare,
///     api_token: "your_api_token".to_string(),
/// };
/// set_a_record(&provider, "example.com", "1.2.3.4")?;
/// # Ok(())
/// # }
/// ```
pub fn set_a_record(provider: &DnsProvider, domain: &str, subdomain: &str, ip: &str) -> Result<()> {
    match provider.provider_type {
        DnsProviderType::Cloudflare => {
            set_cloudflare_a_record(&provider.api_token, domain, subdomain, ip)
        }
        DnsProviderType::GoogleCloud => {
            set_gcp_a_record(&provider.api_token, domain, subdomain, ip)
        }
        DnsProviderType::DuckDNS => set_duckdns_a_record(&provider.api_token, domain, ip),
        DnsProviderType::Porkbun => {
            set_porkbun_a_record(&provider.api_token, domain, subdomain, ip)
        }
    }
}

/// Set AAAA record (IPv6) for a domain
pub fn set_aaaa_record(
    provider: &DnsProvider,
    domain: &str,
    subdomain: &str,
    ipv6: &str,
) -> Result<()> {
    match provider.provider_type {
        DnsProviderType::Cloudflare => {
            set_cloudflare_aaaa_record(&provider.api_token, domain, subdomain, ipv6)
        }
        DnsProviderType::GoogleCloud => {
            set_gcp_aaaa_record(&provider.api_token, domain, subdomain, ipv6)
        }
        DnsProviderType::DuckDNS => set_duckdns_aaaa_record(&provider.api_token, domain, ipv6),
        DnsProviderType::Porkbun => {
            set_porkbun_aaaa_record(&provider.api_token, domain, subdomain, ipv6)
        }
    }
}

/// Delete DNS record from provider
pub fn delete_dns_record(
    provider: &DnsProvider,
    domain: &str,
    subdomain: &str,
    record_type: &str,
) -> Result<()> {
    match provider.provider_type {
        DnsProviderType::Cloudflare => {
            delete_cloudflare_record(&provider.api_token, domain, subdomain, record_type)
        }
        DnsProviderType::GoogleCloud => {
            delete_gcp_record(&provider.api_token, domain, subdomain, record_type)
        }
        DnsProviderType::DuckDNS => {
            anyhow::bail!("DuckDNS record deletion not supported (records expire automatically)")
        }
        DnsProviderType::Porkbun => {
            delete_porkbun_record(&provider.api_token, domain, subdomain, record_type)
        }
    }
}

fn delete_cloudflare_record(
    api_token: &str,
    domain: &str,
    record_name: &str,
    record_type: &str,
) -> Result<()> {
    dure_debug!("Deleting Cloudflare record");
    dure_debug!("  Domain: {}", domain);
    dure_debug!("  Record name (from config): {}", record_name);
    dure_debug!("  Type: {}", record_type);
    dure_debug!(
        "  API Token: {}...",
        &api_token.chars().take(8).collect::<String>()
    );

    let client = CloudflareClient::new(api_token.to_string());

    // Find zone ID
    dure_debug!("  Calling Cloudflare API to find zone...");
    let zone = match client.find_zone_by_domain(domain) {
        Ok(Some(z)) => {
            dure_debug!("  ✓ Found zone: {} ({})", z.name, z.id);
            z
        }
        Ok(None) => {
            dure_debug!("  ❌ Zone not found for domain: {}", domain);
            anyhow::bail!("Zone not found for domain: {}", domain)
        }
        Err(e) => {
            dure_debug!("  ❌ Error finding zone: {}", e);
            return Err(e);
        }
    };

    // record_name is already full FQDN from config (e.g., "test.dure.app")
    // Use it directly for searching
    let full_name = record_name.to_string();

    dure_debug!("  Searching with full name: '{}'", full_name);

    dure_debug!(
        "  Searching for record with name='{}' type='{}'",
        full_name, record_type
    );
    dure_debug!("  Calling Cloudflare API to find record...");

    // Find the record
    match client.find_record(&zone.id, &full_name, record_type) {
        Ok(Some(record)) => {
            dure_debug!("  ✓ Found record:");
            dure_debug!("    ID: {}", record.id);
            dure_debug!("    Name: {}", record.name);
            dure_debug!("    Type: {}", record.record_type);
            dure_debug!("    Content: {}", record.content);
            dure_debug!("  Calling Cloudflare API to delete record...");

            match client.delete_record(&zone.id, &record.id) {
                Ok(_) => {
                    dure_debug!("  ✓ Successfully deleted record");
                    Ok(())
                }
                Err(e) => {
                    dure_debug!("  ❌ Error deleting record: {}", e);
                    Err(e)
                }
            }
        }
        Ok(None) => {
            dure_debug!("  ❌ Record not found");
            dure_debug!("  Getting all records for zone to debug...");

            match client.get_records(&zone.id) {
                Ok(records) => {
                    dure_debug!("  All records in zone:");
                    for rec in &records {
                        dure_debug!(
                            "    - {} ({}) -> {}",
                            rec.name, rec.record_type, rec.content
                        );
                    }
                    dure_debug!("  Total records: {}", records.len());
                }
                Err(e) => {
                    dure_debug!("  ❌ Error listing records: {}", e);
                }
            }

            anyhow::bail!("Record not found: {} ({})", full_name, record_type)
        }
        Err(e) => {
            dure_debug!("  ❌ Error finding record: {}", e);
            Err(e)
        }
    }
}

/// Set TXT record for a domain
///
/// Used for storing public keys or verification records.
///
/// # Examples
///
/// ```rust,no_run
/// use dure::calc::acme::{DnsProvider, DnsProviderType, set_txt_record};
///
/// # async fn example() -> anyhow::Result<()> {
/// let provider = DnsProvider {
///     provider_type: DnsProviderType::Cloudflare,
///     api_token: "your_api_token".to_string(),
/// };
/// // NOTE: Public key generation not yet implemented
/// // set_txt_record(&provider, "example.com", "durepubkey=...")?;
/// # Ok(())
/// # }
/// ```
pub fn set_txt_record(
    provider: &DnsProvider,
    domain: &str,
    subdomain: &str,
    txt_value: &str,
) -> Result<()> {
    match provider.provider_type {
        DnsProviderType::Cloudflare => {
            set_cloudflare_txt_record(&provider.api_token, domain, subdomain, txt_value)
        }
        DnsProviderType::GoogleCloud => {
            set_gcp_txt_record(&provider.api_token, domain, subdomain, txt_value)
        }
        DnsProviderType::DuckDNS => set_duckdns_txt_record(&provider.api_token, domain, txt_value),
        DnsProviderType::Porkbun => {
            set_porkbun_txt_record(&provider.api_token, domain, subdomain, txt_value)
        }
    }
}

// Cloudflare DNS API implementations
fn set_cloudflare_a_record(
    api_token: &str,
    domain: &str,
    record_name: &str,
    ip: &str,
) -> Result<()> {
    dure_debug!("Setting Cloudflare A record");
    dure_debug!("  Domain: {}", domain);
    dure_debug!("  Record name: {}", record_name);
    dure_debug!("  IP: {}", ip);
    dure_debug!(
        "  API Token: {}...",
        &api_token.chars().take(8).collect::<String>()
    );

    let client = CloudflareClient::new(api_token.to_string());

    // Find zone ID
    let zone = client
        .find_zone_by_domain(domain)?
        .ok_or_else(|| anyhow::anyhow!("Zone not found for domain: {}", domain))?;

    dure_debug!("  Found zone: {} ({})", zone.name, zone.id);

    // record_name is already full FQDN from config (e.g., "test.dure.app")
    // Use it directly
    let record_name = record_name.to_string();

    dure_debug!("  Record name: {}", record_name);

    // Check if record exists
    match client.find_record(&zone.id, &record_name, "A")? {
        Some(existing) => {
            dure_debug!("  Updating existing record (ID: {})", existing.id);
            client.update_record(&zone.id, &existing.id, &record_name, "A", ip, None, None)?;
            dure_debug!("  ✓ Updated A record");
        }
        None => {
            dure_debug!("  Creating new record");
            client.create_record(&zone.id, &record_name, "A", ip, None, None)?;
            dure_debug!("  ✓ Created A record");
        }
    }

    Ok(())
}

fn set_cloudflare_txt_record(
    api_token: &str,
    domain: &str,
    record_name: &str,
    txt_value: &str,
) -> Result<()> {
    dure_debug!("Setting Cloudflare TXT record");
    dure_debug!("  Domain: {}", domain);
    dure_debug!("  Record name: {}", record_name);
    dure_debug!("  TXT Value: {}", txt_value);
    dure_debug!(
        "  API Token: {}...",
        &api_token.chars().take(8).collect::<String>()
    );

    let client = CloudflareClient::new(api_token.to_string());

    // Find zone ID
    let zone = client
        .find_zone_by_domain(domain)?
        .ok_or_else(|| anyhow::anyhow!("Zone not found for domain: {}", domain))?;

    dure_debug!("  Found zone: {} ({})", zone.name, zone.id);

    // record_name is already full FQDN from config (e.g., "test.dure.app")
    // Use it directly
    let record_name = record_name.to_string();

    dure_debug!("  Record name: {}", record_name);

    // Check if record exists
    match client.find_record(&zone.id, &record_name, "TXT")? {
        Some(existing) => {
            dure_debug!("  Updating existing record (ID: {})", existing.id);
            client.update_record(
                &zone.id,
                &existing.id,
                &record_name,
                "TXT",
                txt_value,
                None,
                None,
            )?;
            dure_debug!("  ✓ Updated TXT record");
        }
        None => {
            dure_debug!("  Creating new record");
            client.create_record(&zone.id, &record_name, "TXT", txt_value, None, None)?;
            dure_debug!("  ✓ Created TXT record");
        }
    }

    Ok(())
}

fn set_cloudflare_aaaa_record(
    api_token: &str,
    domain: &str,
    record_name: &str,
    ipv6: &str,
) -> Result<()> {
    dure_debug!("Setting Cloudflare AAAA record");
    dure_debug!("  Domain: {}", domain);
    dure_debug!("  Record name: {}", record_name);
    dure_debug!("  IPv6: {}", ipv6);
    dure_debug!(
        "  API Token: {}...",
        &api_token.chars().take(8).collect::<String>()
    );

    let client = CloudflareClient::new(api_token.to_string());

    // Find zone ID
    let zone = client
        .find_zone_by_domain(domain)?
        .ok_or_else(|| anyhow::anyhow!("Zone not found for domain: {}", domain))?;

    dure_debug!("  Found zone: {} ({})", zone.name, zone.id);

    // record_name is already full FQDN from config (e.g., "test.dure.app")
    // Use it directly
    let record_name = record_name.to_string();

    dure_debug!("  Record name: {}", record_name);

    // Check if record exists
    match client.find_record(&zone.id, &record_name, "AAAA")? {
        Some(existing) => {
            dure_debug!("  Updating existing record (ID: {})", existing.id);
            client.update_record(
                &zone.id,
                &existing.id,
                &record_name,
                "AAAA",
                ipv6,
                None,
                None,
            )?;
            dure_debug!("  ✓ Updated AAAA record");
        }
        None => {
            dure_debug!("  Creating new record");
            client.create_record(&zone.id, &record_name, "AAAA", ipv6, None, None)?;
            dure_debug!("  ✓ Created AAAA record");
        }
    }

    Ok(())
}

// DuckDNS API implementations
fn set_duckdns_a_record(token: &str, domain: &str, ip: &str) -> Result<()> {
    // Reference: https://github.com/acmesh-official/acme.sh/blob/master/dnsapi/dns_duckdns.sh
    // DuckDNS API: https://www.duckdns.org/update?domains={domain}&token={token}&ip={ip}
    dure_debug!("Setting DuckDNS A record: {} -> {}", domain, ip);
    dure_debug!("Token: {}...", &token.chars().take(8).collect::<String>());

    let client = DuckDnsClient::new(token.to_string());
    client
        .update_a_record(domain, ip)
        .context("Failed to update DuckDNS A record")
}

fn set_duckdns_txt_record(token: &str, domain: &str, txt_value: &str) -> Result<()> {
    // Reference: https://github.com/acmesh-official/acme.sh/blob/master/dnsapi/dns_duckdns.sh
    dure_debug!("Setting DuckDNS TXT record: {} -> {}", domain, txt_value);
    dure_debug!("Token: {}...", &token.chars().take(8).collect::<String>());

    let client = DuckDnsClient::new(token.to_string());
    client
        .update_txt_record(domain, txt_value)
        .context("Failed to update DuckDNS TXT record")
}

fn set_duckdns_aaaa_record(token: &str, domain: &str, ipv6: &str) -> Result<()> {
    // Reference: https://www.duckdns.org/spec.jsp
    // DuckDNS API: https://www.duckdns.org/update?domains={domain}&token={token}&ipv6={ipv6}
    dure_debug!("Setting DuckDNS AAAA record: {} -> {}", domain, ipv6);
    dure_debug!("Token: {}...", &token.chars().take(8).collect::<String>());

    let client = DuckDnsClient::new(token.to_string());
    client
        .update_aaaa_record(domain, ipv6)
        .context("Failed to update DuckDNS AAAA record")
}

// Porkbun API implementations
fn set_porkbun_a_record(api_token: &str, domain: &str, subdomain: &str, ip: &str) -> Result<()> {
    // Reference: https://github.com/acmesh-official/acme.sh/blob/master/dnsapi/dns_porkbun.sh
    // Porkbun API: https://porkbun.com/api/json/v3/dns/create/{domain}
    dure_debug!("Setting Porkbun A record");
    dure_debug!("  Domain: {}", domain);
    dure_debug!("  Subdomain (input): {}", subdomain);
    dure_debug!("  IP: {}", ip);

    // Parse credentials from "apikey::secretkey" format
    let parts: Vec<&str> = api_token.split("::").collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid Porkbun credentials format. Expected 'apikey::secretkey'");
    }
    let (api_key, secret_key) = (parts[0], parts[1]);

    dure_debug!(
        "  API Key: {}...",
        &api_key.chars().take(8).collect::<String>()
    );

    // Convert @ to empty string for root domain (Porkbun expects empty, not @)
    let porkbun_subdomain = if subdomain == "@" || subdomain == domain {
        ""
    } else {
        subdomain
    };

    dure_debug!("  Subdomain (for API): '{}'", porkbun_subdomain);

    let client = PorkbunClient::new(api_key.to_string(), secret_key.to_string());

    // For Porkbun: domain in URL, subdomain in body
    dure_debug!("  Creating record via Porkbun API...");
    dure_debug!("    URL: /dns/create/{}", domain);
    dure_debug!(
        "    Body: subdomain='{}', type='A', content='{}', ttl=600",
        porkbun_subdomain, ip
    );

    client
        .create_record(domain, porkbun_subdomain, "A", ip, Some(600))
        .context("Failed to create Porkbun A record")
}

fn set_porkbun_txt_record(
    api_token: &str,
    domain: &str,
    subdomain: &str,
    txt_value: &str,
) -> Result<()> {
    // Reference: https://github.com/acmesh-official/acme.sh/blob/master/dnsapi/dns_porkbun.sh
    dure_debug!("Setting Porkbun TXT record");
    dure_debug!("  Domain: {}", domain);
    dure_debug!("  Subdomain (input): {}", subdomain);
    dure_debug!("  TXT Value: {}", txt_value);

    // Parse credentials from "apikey::secretkey" format
    let parts: Vec<&str> = api_token.split("::").collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid Porkbun credentials format. Expected 'apikey::secretkey'");
    }
    let (api_key, secret_key) = (parts[0], parts[1]);

    dure_debug!(
        "  API Key: {}...",
        &api_key.chars().take(8).collect::<String>()
    );

    // Convert @ to empty string for root domain (Porkbun expects empty, not @)
    let porkbun_subdomain = if subdomain == "@" || subdomain == domain {
        ""
    } else {
        subdomain
    };

    dure_debug!("  Subdomain (for API): '{}'", porkbun_subdomain);

    let client = PorkbunClient::new(api_key.to_string(), secret_key.to_string());

    // For Porkbun: domain in URL, subdomain in body
    dure_debug!("  Creating record via Porkbun API...");
    dure_debug!("    URL: /dns/create/{}", domain);
    dure_debug!(
        "    Body: subdomain='{}', type='TXT', content='{}', ttl=600",
        porkbun_subdomain, txt_value
    );

    client
        .create_record(domain, porkbun_subdomain, "TXT", txt_value, Some(600))
        .context("Failed to create Porkbun TXT record")
}

fn set_porkbun_aaaa_record(
    api_token: &str,
    domain: &str,
    subdomain: &str,
    ipv6: &str,
) -> Result<()> {
    // Reference: https://github.com/acmesh-official/acme.sh/blob/master/dnsapi/dns_porkbun.sh
    dure_debug!("Setting Porkbun AAAA record");
    dure_debug!("  Domain: {}", domain);
    dure_debug!("  Subdomain (input): {}", subdomain);
    dure_debug!("  IPv6: {}", ipv6);

    // Parse credentials from "apikey::secretkey" format
    let parts: Vec<&str> = api_token.split("::").collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid Porkbun credentials format. Expected 'apikey::secretkey'");
    }
    let (api_key, secret_key) = (parts[0], parts[1]);

    dure_debug!(
        "  API Key: {}...",
        &api_key.chars().take(8).collect::<String>()
    );

    // Convert @ to empty string for root domain (Porkbun expects empty, not @)
    let porkbun_subdomain = if subdomain == "@" || subdomain == domain {
        ""
    } else {
        subdomain
    };

    dure_debug!("  Subdomain (for API): '{}'", porkbun_subdomain);

    let client = PorkbunClient::new(api_key.to_string(), secret_key.to_string());

    // For Porkbun: domain in URL, subdomain in body
    dure_debug!("  Creating record via Porkbun API...");
    dure_debug!("    URL: /dns/create/{}", domain);
    dure_debug!(
        "    Body: subdomain='{}', type='AAAA', content='{}', ttl=600",
        porkbun_subdomain, ipv6
    );

    client
        .create_record(domain, porkbun_subdomain, "AAAA", ipv6, Some(600))
        .context("Failed to create Porkbun AAAA record")
}

fn delete_porkbun_record(
    api_token: &str,
    domain: &str,
    record_name: &str,
    record_type: &str,
) -> Result<()> {
    dure_debug!("Deleting Porkbun record");
    dure_debug!("  Domain: {}", domain);
    dure_debug!("  Record name: {}", record_name);
    dure_debug!("  Type: {}", record_type);

    // Extract subdomain portion from full record name
    // Porkbun API expects: subdomain portion only, omit or empty for root
    let subdomain = if record_name == domain {
        // Root domain record
        ""
    } else if let Some(prefix) = record_name.strip_suffix(&format!(".{}", domain)) {
        // Subdomain record like "test.getonthe.top" -> "test"
        prefix
    } else if record_name.ends_with(domain) && record_name.len() > domain.len() {
        // Handle case where name is like "testgetonthe.top" (no dot)
        &record_name[..record_name.len() - domain.len() - 1]
    } else {
        // Fallback: use the name as-is
        record_name
    };

    dure_debug!("  Extracted subdomain: '{}'", subdomain);

    // Parse credentials from "apikey::secretkey" format
    let parts: Vec<&str> = api_token.split("::").collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid Porkbun credentials format. Expected 'apikey::secretkey'");
    }
    let (api_key, secret_key) = (parts[0], parts[1]);

    dure_debug!(
        "  API Key: {}...",
        &api_key.chars().take(8).collect::<String>()
    );

    let client = PorkbunClient::new(api_key.to_string(), secret_key.to_string());

    dure_debug!("  Calling Porkbun delete API...");
    dure_debug!(
        "    URL: /dns/deleteByNameType/{}/{}/{}",
        domain, record_type, subdomain
    );

    client
        .delete_record(domain, subdomain, record_type)
        .context("Failed to delete Porkbun record")
}

// Google Cloud DNS API implementations
fn set_gcp_a_record(api_token: &str, domain: &str, record_name: &str, ip: &str) -> Result<()> {
    dure_debug!("Setting GCP A record");
    dure_debug!("  Domain: {}", domain);
    dure_debug!("  Record name: {}", record_name);
    dure_debug!("  IP: {}", ip);

    // Parse token format: "access_token::project_id"
    let parts: Vec<&str> = api_token.split("::").collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid GCP token format. Expected 'access_token::project_id'");
    }
    let (access_token, project_id) = (parts[0], parts[1]);

    use crate::api::ns_gcp::GcpDnsClient;
    let client = GcpDnsClient::new(access_token.to_string());

    // Find managed zone for this domain
    let zone = client
        .find_zone_by_domain(project_id, domain)?
        .ok_or_else(|| anyhow::anyhow!("Managed zone not found for domain: {}", domain))?;

    dure_debug!("  Found zone: {} ({})", zone.name, zone.id);

    // Construct FQDN based on record name
    let fqdn = if record_name.ends_with('.') {
        record_name.to_string()
    } else if record_name.is_empty() || record_name == "@" {
        format!("{}.", domain) // Root domain
    } else {
        format!("{}.{}.", record_name, domain) // Subdomain
    };

    dure_debug!("  FQDN: {}", fqdn);

    // Try to fetch existing record
    let existing_rrsets = client.list_rrsets(project_id, &zone.name)?;
    let existing = existing_rrsets
        .iter()
        .find(|r| r.name == fqdn && r.record_type == "A");

    if let Some(_existing) = existing {
        dure_debug!("  Updating existing A record");
        client.update_rrset(
            project_id,
            &zone.name,
            &fqdn,
            "A",
            300,
            vec![ip.to_string()],
        )?;
        dure_debug!("  ✓ Updated A record");
    } else {
        dure_debug!("  Creating new A record");
        client.create_rrset(
            project_id,
            &zone.name,
            &fqdn,
            "A",
            300,
            vec![ip.to_string()],
        )?;
        dure_debug!("  ✓ Created A record");
    }

    Ok(())
}

fn set_gcp_aaaa_record(api_token: &str, domain: &str, record_name: &str, ipv6: &str) -> Result<()> {
    dure_debug!("Setting GCP AAAA record");
    dure_debug!("  Domain: {}", domain);
    dure_debug!("  Record name: {}", record_name);
    dure_debug!("  IPv6: {}", ipv6);

    // Parse token format: "access_token::project_id"
    let parts: Vec<&str> = api_token.split("::").collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid GCP token format. Expected 'access_token::project_id'");
    }
    let (access_token, project_id) = (parts[0], parts[1]);

    use crate::api::ns_gcp::GcpDnsClient;
    let client = GcpDnsClient::new(access_token.to_string());

    // Find managed zone for this domain
    let zone = client
        .find_zone_by_domain(project_id, domain)?
        .ok_or_else(|| anyhow::anyhow!("Managed zone not found for domain: {}", domain))?;

    dure_debug!("  Found zone: {} ({})", zone.name, zone.id);

    // Construct FQDN based on record name
    let fqdn = if record_name.ends_with('.') {
        record_name.to_string()
    } else if record_name.is_empty() || record_name == "@" {
        format!("{}.", domain) // Root domain
    } else {
        format!("{}.{}.", record_name, domain) // Subdomain
    };

    dure_debug!("  FQDN: {}", fqdn);

    // Try to fetch existing record
    let existing_rrsets = client.list_rrsets(project_id, &zone.name)?;
    let existing = existing_rrsets
        .iter()
        .find(|r| r.name == fqdn && r.record_type == "AAAA");

    if let Some(_existing) = existing {
        dure_debug!("  Updating existing AAAA record");
        client.update_rrset(
            project_id,
            &zone.name,
            &fqdn,
            "AAAA",
            300,
            vec![ipv6.to_string()],
        )?;
        dure_debug!("  ✓ Updated AAAA record");
    } else {
        dure_debug!("  Creating new AAAA record");
        client.create_rrset(
            project_id,
            &zone.name,
            &fqdn,
            "AAAA",
            300,
            vec![ipv6.to_string()],
        )?;
        dure_debug!("  ✓ Created AAAA record");
    }

    Ok(())
}

fn set_gcp_txt_record(
    api_token: &str,
    domain: &str,
    record_name: &str,
    txt_value: &str,
) -> Result<()> {
    dure_debug!("Setting GCP TXT record");
    dure_debug!("  Domain: {}", domain);
    dure_debug!("  Record name: {}", record_name);
    dure_debug!("  TXT Value: {}", txt_value);

    // Parse token format: "access_token::project_id"
    let parts: Vec<&str> = api_token.split("::").collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid GCP token format. Expected 'access_token::project_id'");
    }
    let (access_token, project_id) = (parts[0], parts[1]);

    use crate::api::ns_gcp::GcpDnsClient;
    let client = GcpDnsClient::new(access_token.to_string());

    // Find managed zone for this domain
    let zone = client
        .find_zone_by_domain(project_id, domain)?
        .ok_or_else(|| anyhow::anyhow!("Managed zone not found for domain: {}", domain))?;

    dure_debug!("  Found zone: {} ({})", zone.name, zone.id);

    // Construct FQDN based on record name
    let fqdn = if record_name.ends_with('.') {
        record_name.to_string()
    } else if record_name.is_empty() || record_name == "@" {
        format!("{}.", domain) // Root domain
    } else {
        format!("{}.{}.", record_name, domain) // Subdomain
    };

    dure_debug!("  FQDN: {}", fqdn);

    // GCP TXT records need to be quoted
    let quoted_txt = if txt_value.starts_with('"') && txt_value.ends_with('"') {
        txt_value.to_string()
    } else {
        format!("\"{}\"", txt_value)
    };

    // Try to fetch existing record
    let existing_rrsets = client.list_rrsets(project_id, &zone.name)?;
    let existing = existing_rrsets
        .iter()
        .find(|r| r.name == fqdn && r.record_type == "TXT");

    if let Some(_existing) = existing {
        dure_debug!("  Updating existing TXT record");
        client.update_rrset(project_id, &zone.name, &fqdn, "TXT", 300, vec![quoted_txt])?;
        dure_debug!("  ✓ Updated TXT record");
    } else {
        dure_debug!("  Creating new TXT record");
        client.create_rrset(project_id, &zone.name, &fqdn, "TXT", 300, vec![quoted_txt])?;
        dure_debug!("  ✓ Created TXT record");
    }

    Ok(())
}

fn delete_gcp_record(
    api_token: &str,
    domain: &str,
    record_name: &str,
    record_type: &str,
) -> Result<()> {
    dure_debug!("Deleting GCP record");
    dure_debug!("  Domain: {}", domain);
    dure_debug!("  Record name: {}", record_name);
    dure_debug!("  Type: {}", record_type);

    // Parse token format: "access_token::project_id"
    let parts: Vec<&str> = api_token.split("::").collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid GCP token format. Expected 'access_token::project_id'");
    }
    let (access_token, project_id) = (parts[0], parts[1]);

    use crate::api::ns_gcp::GcpDnsClient;
    let client = GcpDnsClient::new(access_token.to_string());

    // Find managed zone for this domain
    let zone = client
        .find_zone_by_domain(project_id, domain)?
        .ok_or_else(|| anyhow::anyhow!("Managed zone not found for domain: {}", domain))?;

    dure_debug!("  Found zone: {} ({})", zone.name, zone.id);

    // Construct FQDN based on record name
    let fqdn = if record_name.ends_with('.') {
        record_name.to_string()
    } else if record_name.is_empty() || record_name == "@" {
        format!("{}.", domain) // Root domain
    } else {
        format!("{}.{}.", record_name, domain) // Subdomain
    };

    dure_debug!("  FQDN: {}", fqdn);
    dure_debug!("  Calling GCP delete API...");

    client
        .delete_rrset(project_id, &zone.name, &fqdn, record_type)
        .context("Failed to delete GCP record")?;

    dure_debug!("  ✓ Deleted {} record", record_type);
    Ok(())
}

// TODO: Public key generation placeholder
// This will be implemented when cryptographic key management is ready
//
// /// Generate ChaCha20 public key for system
// ///
// /// # Examples
// ///
// /// ```rust,no_run
// /// use dure::calc::acme::generate_pubkey;
// ///
// /// # fn example() -> anyhow::Result<()> {
// /// let pubkey = generate_pubkey()?;
// /// println!("Public key: {}", pubkey);
// /// # Ok(())
// /// # }
// /// ```
// pub fn generate_pubkey() -> Result<String> {
//     // TODO: Implement X25519 + ChaCha20 key generation
//     // Reference: Use chacha20poly1305 and x25519-dalek crates
//     anyhow::bail!("Public key generation not yet implemented")
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_validity() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let cert = Certificate::new(
            "example.com".to_string(),
            "/path/cert".to_string(),
            "/path/key".to_string(),
            "/path/ca".to_string(),
            "/path/fullchain".to_string(),
            now,
            now + 90 * 24 * 60 * 60, // 90 days
        );

        assert!(cert.is_still_valid());
        assert!(!cert.needs_renewal()); // Should not need renewal yet
    }

    #[test]
    fn test_certificate_needs_renewal() {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Certificate expiring in 20 days
        let cert = Certificate::new(
            "example.com".to_string(),
            "/path/cert".to_string(),
            "/path/key".to_string(),
            "/path/ca".to_string(),
            "/path/fullchain".to_string(),
            now,
            now + 20 * 24 * 60 * 60,
        );

        assert!(cert.is_still_valid());
        assert!(cert.needs_renewal()); // Should need renewal (< 30 days)
    }

    #[test]
    fn test_dns_provider_type() {
        assert_eq!(DnsProviderType::Cloudflare.as_str(), "cloudflare");
        assert_eq!(DnsProviderType::GoogleCloud.as_str(), "gcloud");
        assert_eq!(DnsProviderType::DuckDNS.as_str(), "duckdns");
        assert_eq!(DnsProviderType::Porkbun.as_str(), "porkbun");
    }
}
