//! NFTables firewall management functionality
//!
//! Provides nftables rule management for SSH port whitelisting with SQLite-based
//! storage for IP whitelist tracking.
//!
//! By default:
//! - Ports 80 (HTTP) and 443 (HTTPS) are open to the world
//! - Port 22 (SSH) is only accessible from whitelisted IPs

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Whitelisted IP address entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistedIp {
    pub ip: String,
    pub description: String,
    pub added_at: u64,
}

impl WhitelistedIp {
    pub fn new(ip: String, description: String) -> Self {
        let added_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        Self {
            ip,
            description,
            added_at,
        }
    }
}

/// NFTables ruleset configuration
#[derive(Debug, Clone)]
pub struct NftRuleset {
    pub table_name: String,
    pub chain_input: String,
    pub ssh_port: u16,
    pub http_port: u16,
    pub https_port: u16,
}

impl Default for NftRuleset {
    fn default() -> Self {
        Self {
            table_name: "dure_filter".to_string(),
            chain_input: "input".to_string(),
            ssh_port: 22,
            http_port: 80,
            https_port: 443,
        }
    }
}

impl NftRuleset {
    /// Generate nftables ruleset configuration
    ///
    /// Creates a complete nftables configuration that:
    /// - Opens ports 80 and 443 to the world
    /// - Restricts SSH (port 22) to whitelisted IPs only
    /// - Allows established connections
    /// - Drops all other traffic
    pub fn generate_rules(&self, whitelisted_ips: &[String]) -> String {
        let mut rules = format!(
            r#"#!/usr/sbin/nft -f
# Dure NFTables Configuration
# Generated: {}

table inet {} {{
    chain {} {{
        type filter hook input priority 0; policy drop;

        # Allow loopback
        iif lo accept

        # Allow established/related connections
        ct state established,related accept

        # Allow ICMP
        ip protocol icmp accept
        ip6 nexthdr icmpv6 accept

        # Allow HTTP and HTTPS to everyone
        tcp dport {} accept
        tcp dport {} accept
"#,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S"),
            self.table_name,
            self.chain_input,
            self.http_port,
            self.https_port,
        );

        // Add SSH whitelist rules
        if whitelisted_ips.is_empty() {
            rules.push_str(&format!(
                "        # No whitelisted IPs - SSH blocked for all\n        # tcp dport {} drop\n",
                self.ssh_port
            ));
        } else {
            rules.push_str(&format!(
                "        # SSH whitelist (port {})\n",
                self.ssh_port
            ));
            for ip in whitelisted_ips {
                rules.push_str(&format!(
                    "        ip saddr {} tcp dport {} accept\n",
                    ip, self.ssh_port
                ));
            }
        }

        rules.push_str("    }\n}\n");
        rules
    }
}

/// Show current nftables ruleset
///
/// Executes `nft list ruleset` to display the current firewall configuration.
///
/// # Examples
///
/// ```rust,no_run
/// use dure::calc::nft::show_ruleset;
///
/// # async fn example() -> anyhow::Result<()> {
/// let ruleset = show_ruleset()?;
/// println!("{}", ruleset);
/// # Ok(())
/// # }
/// ```
pub fn show_ruleset() -> Result<String> {
    #[cfg(not(target_os = "linux"))]
    {
        anyhow::bail!("NFTables operations are only supported on Linux");
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        let output = Command::new("nft")
            .arg("list")
            .arg("ruleset")
            .output()
            .context("Failed to execute 'nft list ruleset'. Ensure nftables is installed and you have proper permissions.")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to list nftables ruleset: {}", stderr);
        }

        let stdout =
            String::from_utf8(output.stdout).context("Failed to parse nft output as UTF-8")?;

        Ok(stdout)
    }
}

/// Add an IP to the SSH whitelist
///
/// Updates the nftables configuration to allow SSH access from the specified IP.
/// Requires root/sudo privileges.
///
/// # Arguments
///
/// * `ip` - IP address to whitelist (e.g., "192.168.1.100")
/// * `all_whitelisted_ips` - Current list of all whitelisted IPs
///
/// # Examples
///
/// ```rust,no_run
/// use dure::calc::nft::whitelist_ip;
///
/// # async fn example() -> anyhow::Result<()> {
/// let ips = vec!["192.168.1.100".to_string()];
/// whitelist_ip("192.168.1.100", &ips)?;
/// # Ok(())
/// # }
/// ```
pub fn whitelist_ip(ip: &str, all_whitelisted_ips: &[String]) -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    {
        anyhow::bail!("NFTables operations are only supported on Linux");
    }

    #[cfg(target_os = "linux")]
    {
        validate_ip(ip)?;

        let ruleset = NftRuleset::default();
        let rules = ruleset.generate_rules(all_whitelisted_ips);

        apply_ruleset(&rules)?;

        Ok(())
    }
}

/// Remove an IP from the SSH whitelist
///
/// Updates the nftables configuration to remove SSH access for the specified IP.
/// Requires root/sudo privileges.
///
/// # Arguments
///
/// * `ip` - IP address to remove from whitelist
/// * `remaining_whitelisted_ips` - List of IPs that should remain whitelisted
///
/// # Examples
///
/// ```rust,no_run
/// use dure::calc::nft::remove_whitelisted_ip;
///
/// # async fn example() -> anyhow::Result<()> {
/// let remaining = vec!["192.168.1.101".to_string()];
/// remove_whitelisted_ip("192.168.1.100", &remaining)?;
/// # Ok(())
/// # }
/// ```
pub fn remove_whitelisted_ip(ip: &str, remaining_whitelisted_ips: &[String]) -> Result<()> {
    #[cfg(not(target_os = "linux"))]
    {
        anyhow::bail!("NFTables operations are only supported on Linux");
    }

    #[cfg(target_os = "linux")]
    {
        validate_ip(ip)?;

        let ruleset = NftRuleset::default();
        let rules = ruleset.generate_rules(remaining_whitelisted_ips);

        apply_ruleset(&rules)?;

        Ok(())
    }
}

/// Apply nftables ruleset from configuration string
///
/// Writes the ruleset to a temporary file and applies it using `nft -f`.
/// Requires root/sudo privileges.
#[cfg(target_os = "linux")]
fn apply_ruleset(rules: &str) -> Result<()> {
    use std::io::Write;
    use std::process::Command;

    // Write rules to temporary file
    let temp_dir = std::env::temp_dir();
    let rules_file = temp_dir.join("dure_nft_rules.nft");

    {
        let mut file =
            std::fs::File::create(&rules_file).context("Failed to create temporary rules file")?;
        file.write_all(rules.as_bytes())
            .context("Failed to write rules to temporary file")?;
    }

    // Apply the ruleset
    let output = Command::new("nft")
        .arg("-f")
        .arg(&rules_file)
        .output()
        .context("Failed to execute 'nft -f'. Ensure you have root/sudo privileges.")?;

    // Clean up temporary file
    let _ = std::fs::remove_file(&rules_file);

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Failed to apply nftables ruleset: {}", stderr);
    }

    Ok(())
}

/// Validate IP address format
///
/// Checks if the provided string is a valid IPv4 or IPv6 address.
fn validate_ip(ip: &str) -> Result<()> {
    use std::net::IpAddr;

    ip.parse::<IpAddr>()
        .map(|_| ())
        .context("Invalid IP address format")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ip_v4() {
        assert!(validate_ip("192.168.1.1").is_ok());
        assert!(validate_ip("8.8.8.8").is_ok());
        assert!(validate_ip("255.255.255.255").is_ok());
    }

    #[test]
    fn test_validate_ip_v6() {
        assert!(validate_ip("::1").is_ok());
        assert!(validate_ip("2001:db8::1").is_ok());
    }

    #[test]
    fn test_validate_ip_invalid() {
        assert!(validate_ip("256.1.1.1").is_err());
        assert!(validate_ip("not.an.ip").is_err());
        assert!(validate_ip("").is_err());
    }

    #[test]
    fn test_nft_ruleset_generation() {
        let ruleset = NftRuleset::default();
        let ips = vec!["192.168.1.100".to_string(), "10.0.0.5".to_string()];
        let rules = ruleset.generate_rules(&ips);

        assert!(rules.contains("table inet dure_filter"));
        assert!(rules.contains("tcp dport 80 accept"));
        assert!(rules.contains("tcp dport 443 accept"));
        assert!(rules.contains("ip saddr 192.168.1.100 tcp dport 22 accept"));
        assert!(rules.contains("ip saddr 10.0.0.5 tcp dport 22 accept"));
    }

    #[test]
    fn test_nft_ruleset_no_whitelist() {
        let ruleset = NftRuleset::default();
        let rules = ruleset.generate_rules(&[]);

        assert!(rules.contains("table inet dure_filter"));
        assert!(rules.contains("tcp dport 80 accept"));
        assert!(rules.contains("tcp dport 443 accept"));
        assert!(rules.contains("# No whitelisted IPs"));
    }

    #[test]
    fn test_whitelisted_ip_creation() {
        let ip = WhitelistedIp::new("192.168.1.1".to_string(), "Office IP".to_string());
        assert_eq!(ip.ip, "192.168.1.1");
        assert_eq!(ip.description, "Office IP");
        assert!(ip.added_at > 0);
    }
}
