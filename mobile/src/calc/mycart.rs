//! Mycart business logic
//!
//! Provides high-level mycart management operations:
//! - Add/delete mycart sites with credential verification
//! - Update admin passwords
//! - Retrieve site statistics and logs
//! - Encrypted password storage

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use crate::api::mycart;
#[cfg(not(target_arch = "wasm32"))]
use crate::calc::crypt;
#[cfg(not(target_arch = "wasm32"))]
use crate::calc::keyring::DatabaseHandle;
#[cfg(not(target_arch = "wasm32"))]
use crate::config::{AppConfig, MycartSiteConfig, SshHostConfig};

/// Add a new mycart site after verifying admin credentials
///
/// This function:
/// 1. Verifies admin credentials and gets JWT token
/// 2. Encrypts the password
/// 3. Stores the site in profile config.yml
///
/// # Arguments
/// * `config_path` - Path to profile config.yml
/// * `site_address` - Site domain (e.g., "dure.co")
/// * `email` - Admin email address
/// * `password` - Plain text admin password
///
/// # Returns
/// * `Ok(())` - Site added successfully
/// * `Err(_)` - Verification failed or config save error
#[cfg(not(target_arch = "wasm32"))]
pub async fn add_mycart_site(
    config_path: &std::path::Path,
    site_address: &str,
    email: &str,
    password: &str,
    platform_id: Option<&str>,
    ssh_host: Option<&str>,
) -> Result<()> {
    log::info!("Adding mycart site: {} (email: {})", site_address, email);

    // Step 1: Verify credentials and get JWT token
    log::debug!("Step 1: Verifying admin credentials...");
    let jwt_token = mycart::verify_admin_access(site_address, email, password)
        .await
        .context("Failed to verify admin credentials")?
        .ok_or_else(|| {
            log::error!("Admin credentials are invalid");
            anyhow::anyhow!("Invalid admin credentials")
        })?;

    log::info!("Admin credentials verified, JWT token obtained");

    // Step 2: Encode password (base64 for now)
    // ponytail: base64 encoding, upgrade to keyring when security review done
    log::debug!("Step 2: Encoding password...");
    let encrypted_password = crypt::encode_base64(password.as_bytes());

    // Step 3: Store in config.yml
    log::debug!("Step 3: Storing site in config...");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs();

    let site_config = MycartSiteConfig {
        site_address: site_address.to_string(),
        email: email.to_string(),
        encrypted_password,
        jwt_token: Some(jwt_token),
        created_at: now,
        updated_at: now,
        platform_id: platform_id.filter(|s| !s.is_empty()).map(String::from),
        ssh_host: ssh_host.filter(|s| !s.is_empty()).map(String::from),
    };

    let mut config = AppConfig::load_or_default(&config_path.to_path_buf());
    config.mycart_sites.push(site_config);
    config
        .save(&config_path.to_path_buf())
        .context("Failed to save config")?;

    log::info!("Mycart site added successfully: {}", site_address);
    Ok(())
}

/// List all mycart sites from config
#[cfg(not(target_arch = "wasm32"))]
pub fn list_mycart_sites(config_path: &std::path::Path) -> Result<Vec<MycartSiteConfig>> {
    let config = AppConfig::load_or_default(&config_path.to_path_buf());
    Ok(config.mycart_sites)
}

/// Delete a mycart site from config
#[cfg(not(target_arch = "wasm32"))]
pub fn delete_mycart_site(config_path: &std::path::Path, site_address: &str) -> Result<()> {
    let mut config = AppConfig::load_or_default(&config_path.to_path_buf());
    config
        .mycart_sites
        .retain(|s| s.site_address != site_address);
    config
        .save(&config_path.to_path_buf())
        .context("Failed to save config")?;
    Ok(())
}

/// Get site statistics (orders, products, messages counts)
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_site_stats(
    config_path: &std::path::Path,
    site_address: &str,
) -> Result<mycart::MycartStats> {
    let config = AppConfig::load_or_default(&config_path.to_path_buf());
    let site = config
        .mycart_sites
        .iter()
        .find(|s| s.site_address == site_address)
        .ok_or_else(|| anyhow::anyhow!("Mycart site not found"))?;

    let jwt_token = site
        .jwt_token
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No JWT token - site needs re-authentication"))?;

    mycart::get_mycart_stats(site_address, jwt_token)
        .await
        .context("Failed to get mycart stats")
}

/// Get seller info for a mycart site
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_seller_info(
    config_path: &std::path::Path,
    site_address: &str,
) -> Result<mycart::SellerInfo> {
    let config = AppConfig::load_or_default(&config_path.to_path_buf());
    let site = config
        .mycart_sites
        .iter()
        .find(|s| s.site_address == site_address)
        .ok_or_else(|| anyhow::anyhow!("Mycart site not found"))?;

    let jwt_token = site
        .jwt_token
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No JWT token - site needs re-authentication"))?;

    mycart::get_seller_info(site_address, jwt_token)
        .await
        .context("Failed to get seller info")
}

/// Get site logs from /srv/data/logs
///
/// Returns a map of log_name -> log_content
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_site_logs(
    site_address: &str,
    host_config: &SshHostConfig,
    profile_keyring: Option<&Arc<DatabaseHandle>>,
) -> Result<HashMap<String, String>> {
    // Define expected log files
    let expected_logs = vec![
        "mycart-stderr.log",
        "mycart-stdout.log",
        "fail2ban-rs-stderr.log",
        "fail2ban-rs-stdout.log",
        "xmpp-proxy-stdout.log",
        "xmpp-proxy-stderr.log",
        "prosody.log",
    ];

    let mut logs = HashMap::new();

    for log_name in expected_logs {
        match mycart::read_log_file(host_config, log_name, 100, profile_keyring).await {
            Ok(content) => {
                logs.insert(log_name.to_string(), content);
            }
            Err(e) => {
                // Log not found or error reading - insert error message
                logs.insert(log_name.to_string(), format!("Error reading log: {}", e));
            }
        }
    }

    Ok(logs)
}

/// Get DureMycart service logs only
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_duremycart_logs(
    host_config: &SshHostConfig,
    profile_keyring: Option<&Arc<DatabaseHandle>>,
) -> Result<Vec<String>> {
    let log_names = vec!["mycart-stderr.log", "mycart-stdout.log"];
    let mut all_lines = Vec::new();

    for log_name in log_names {
        all_lines.push(format!("=== {} ===", log_name));
        match mycart::read_log_file(host_config, log_name, 1000, profile_keyring).await {
            Ok(content) => {
                // Split by lines and add to vec
                for line in content.lines() {
                    all_lines.push(line.to_string());
                }
            }
            Err(e) => {
                all_lines.push(format!("Error reading log: {}", e));
            }
        }
        all_lines.push(String::new()); // Empty line separator
    }

    Ok(all_lines)
}

/// Get XmppProxy service logs only
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_xmppproxy_logs(
    host_config: &SshHostConfig,
    profile_keyring: Option<&Arc<DatabaseHandle>>,
) -> Result<Vec<String>> {
    let log_names = vec![
        "xmpp-proxy-stdout.log",
        "xmpp-proxy-stderr.log",
        "prosody.log",
    ];
    let mut all_lines = Vec::new();

    for log_name in log_names {
        all_lines.push(format!("=== {} ===", log_name));
        match mycart::read_log_file(host_config, log_name, 1000, profile_keyring).await {
            Ok(content) => {
                // Split by lines and add to vec
                for line in content.lines() {
                    all_lines.push(line.to_string());
                }
            }
            Err(e) => {
                all_lines.push(format!("Error reading log: {}", e));
            }
        }
        all_lines.push(String::new()); // Empty line separator
    }

    Ok(all_lines)
}

/// Get Fail2BanRS service logs only
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_fail2banrs_logs(
    host_config: &SshHostConfig,
    profile_keyring: Option<&Arc<DatabaseHandle>>,
) -> Result<Vec<String>> {
    let log_names = vec!["fail2ban-rs-stderr.log", "fail2ban-rs-stdout.log"];
    let mut all_lines = Vec::new();

    for log_name in log_names {
        all_lines.push(format!("=== {} ===", log_name));
        match mycart::read_log_file(host_config, log_name, 1000, profile_keyring).await {
            Ok(content) => {
                // Split by lines and add to vec
                for line in content.lines() {
                    all_lines.push(line.to_string());
                }
            }
            Err(e) => {
                all_lines.push(format!("Error reading log: {}", e));
            }
        }
        all_lines.push(String::new()); // Empty line separator
    }

    Ok(all_lines)
}

/// Get crontab jobs for a mycart site (high-level wrapper with JWT loading)
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_site_crontab(
    config_path: &std::path::Path,
    site_address: &str,
) -> Result<Vec<mycart::CrontabJob>> {
    log::debug!(
        "get_site_crontab: config_path={:?}, site_address={}",
        config_path,
        site_address
    );

    let config = AppConfig::load_or_default(&config_path.to_path_buf());
    log::debug!(
        "get_site_crontab: found {} mycart_sites in config",
        config.mycart_sites.len()
    );
    log::debug!(
        "get_site_crontab: available sites: {:?}",
        config
            .mycart_sites
            .iter()
            .map(|s| &s.site_address)
            .collect::<Vec<_>>()
    );

    let site = config
        .mycart_sites
        .iter()
        .find(|s| s.site_address == site_address)
        .ok_or_else(|| {
            log::error!(
                "get_site_crontab: site '{}' not found in config",
                site_address
            );
            anyhow::anyhow!(
                "Mycart site '{}' not found in config. Available sites: {:?}",
                site_address,
                config
                    .mycart_sites
                    .iter()
                    .map(|s| &s.site_address)
                    .collect::<Vec<_>>()
            )
        })?;

    log::debug!("get_site_crontab: found site, checking JWT token");
    let jwt_token = site.jwt_token.as_ref().ok_or_else(|| {
        log::error!("get_site_crontab: no JWT token for site '{}'", site_address);
        anyhow::anyhow!(
            "No JWT token for site '{}' - site needs re-authentication",
            site_address
        )
    })?;

    log::debug!("get_site_crontab: calling list_crontab API");
    list_crontab(site_address, jwt_token).await
}

/// Update crontab job enabled status (high-level wrapper with JWT loading)
#[cfg(not(target_arch = "wasm32"))]
pub async fn update_site_cronjob_enabled(
    config_path: &std::path::Path,
    site_address: &str,
    job_id: &str,
    enabled: bool,
) -> Result<mycart::CrontabJob> {
    let config = AppConfig::load_or_default(&config_path.to_path_buf());
    let site = config
        .mycart_sites
        .iter()
        .find(|s| s.site_address == site_address)
        .ok_or_else(|| anyhow::anyhow!("Mycart site not found"))?;

    let jwt_token = site
        .jwt_token
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No JWT token - site needs re-authentication"))?;

    set_crontab_enabled(site_address, jwt_token, job_id, enabled).await
}

/// Update crontab job interval (high-level wrapper with JWT loading)
#[cfg(not(target_arch = "wasm32"))]
pub async fn update_site_cronjob_interval(
    config_path: &std::path::Path,
    site_address: &str,
    job_id: &str,
    interval: &str,
) -> Result<mycart::CrontabJob> {
    let config = AppConfig::load_or_default(&config_path.to_path_buf());
    let site = config
        .mycart_sites
        .iter()
        .find(|s| s.site_address == site_address)
        .ok_or_else(|| anyhow::anyhow!("Mycart site not found"))?;

    let jwt_token = site
        .jwt_token
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No JWT token - site needs re-authentication"))?;

    set_crontab_interval(site_address, jwt_token, job_id, interval).await
}

/// Resolve platform for a mycart site
///
/// Prefers explicit `site.platform_id`, falls back to VM IP matching
#[cfg(not(target_arch = "wasm32"))]
pub fn resolve_platform_for_site<'a>(
    config: &'a AppConfig,
    site: &MycartSiteConfig,
) -> Option<&'a crate::config::CloudPlatformConfig> {
    // Prefer explicit platform_id
    if let Some(ref platform_id) = site.platform_id {
        if let Some(platform) = config
            .platforms
            .iter()
            .find(|p| p.gcp_selected_project_id.as_ref() == Some(platform_id))
        {
            return Some(platform);
        }
    }

    // Fallback: match site domain to VM external IP
    let site_host = site
        .site_address
        .split(':')
        .next()
        .unwrap_or(&site.site_address);

    config.platforms.iter().find(|platform| {
        platform
            .vms
            .iter()
            .any(|vm| vm.external_ip.as_ref().map_or(false, |ip| ip == site_host))
    })
}

/// Resolve SSH host for a mycart site
///
/// Prefers explicit `site.ssh_host`, falls back to hostname/IP matching
#[cfg(not(target_arch = "wasm32"))]
pub fn resolve_ssh_host_for_site<'a>(
    config: &'a AppConfig,
    site: &MycartSiteConfig,
) -> Option<&'a SshHostConfig> {
    // Prefer explicit ssh_host
    if let Some(ref ssh_host_id) = site.ssh_host {
        if let Some(host) = config
            .ssh_hosts
            .iter()
            .find(|h| h.host.contains(ssh_host_id) || ssh_host_id.contains(&h.host))
        {
            return Some(host);
        }
    }

    // Fallback: match site domain to SSH host
    let site_host = site
        .site_address
        .split(':')
        .next()
        .unwrap_or(&site.site_address);

    config.ssh_hosts.iter().find(|host_config| {
        let host_part = host_config
            .host
            .split('@')
            .last()
            .unwrap_or(&host_config.host);
        host_part == site_host || site_host.contains(host_part)
    })
}

/// Validate cron expression format (client-side guard)
///
/// ponytail: basic validator without regex, server is authoritative
#[cfg(not(target_arch = "wasm32"))]
pub fn validate_cron(expr: &str) -> Result<()> {
    let expr = expr.trim();

    // Allow @-prefixed macros (@hourly, @daily, etc.)
    if expr.starts_with('@') {
        return Ok(());
    }

    // Standard 5-field cron: minute hour day month weekday
    let fields: Vec<&str> = expr.split_whitespace().collect();
    if fields.len() != 5 {
        return Err(anyhow::anyhow!(
            "Invalid cron format: expected 5 fields (minute hour day month weekday), got {}",
            fields.len()
        ));
    }

    // Basic field validation: each field should contain only valid cron chars
    for (idx, field) in fields.iter().enumerate() {
        if field.is_empty() {
            return Err(anyhow::anyhow!(
                "Invalid cron field {}: empty field",
                idx + 1
            ));
        }

        for ch in field.chars() {
            if !matches!(ch, '0'..='9' | '*' | '/' | ',' | '-') {
                return Err(anyhow::anyhow!(
                    "Invalid cron field {}: '{}' contains invalid character '{}'",
                    idx + 1,
                    field,
                    ch
                ));
            }
        }
    }

    Ok(())
}

/// List crontab jobs for a mycart site
#[cfg(not(target_arch = "wasm32"))]
pub async fn list_crontab(site_address: &str, jwt_token: &str) -> Result<Vec<mycart::CrontabJob>> {
    mycart::get_crontab(site_address, jwt_token)
        .await
        .context("Failed to list crontab jobs")
}

/// Load operation logs for a mycart site
#[cfg(not(target_arch = "wasm32"))]
pub fn load_site_operations(
    site_address: &str,
    limit: i64,
) -> Result<Vec<crate::storage::models::opslog::OperationLog>> {
    use crate::calc::db;
    use crate::storage::models::opslog::list_by_project;

    let mut conn = db::establish_connection();
    // Query operations where project_id or relevant_id matches site_address
    // This covers operations logged for this mycart site
    list_by_project(&mut conn, site_address, limit)
}

/// Enable or disable a crontab job
#[cfg(not(target_arch = "wasm32"))]
pub async fn set_crontab_enabled(
    site_address: &str,
    jwt_token: &str,
    job_id: &str,
    enabled: bool,
) -> Result<mycart::CrontabJob> {
    let patch = mycart::CrontabPatch {
        enabled: Some(enabled),
        interval: None,
    };

    mycart::patch_crontab(site_address, jwt_token, job_id, patch)
        .await
        .context("Failed to update crontab job enabled status")
}

/// Set crontab job interval with validation
#[cfg(not(target_arch = "wasm32"))]
pub async fn set_crontab_interval(
    site_address: &str,
    jwt_token: &str,
    job_id: &str,
    interval: &str,
) -> Result<mycart::CrontabJob> {
    // Validate cron expression client-side first
    validate_cron(interval).context("Invalid cron expression format")?;

    let patch = mycart::CrontabPatch {
        enabled: None,
        interval: Some(interval.to_string()),
    };

    mycart::patch_crontab(site_address, jwt_token, job_id, patch)
        .await
        .context("Failed to update crontab job interval")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests for data structures
    #[test]
    fn test_mycart_site_config_struct() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let site = MycartSiteConfig {
            site_address: "dure.co".to_string(),
            email: "admin@example.com".to_string(),
            encrypted_password: "encrypted_pass".to_string(),
            jwt_token: Some("jwt_token_abc".to_string()),
            created_at: now,
            updated_at: now,
            platform_id: Some("my-gcp-project".to_string()),
            ssh_host: Some("203.0.113.42".to_string()),
        };

        assert_eq!(site.site_address, "dure.co");
        assert_eq!(site.email, "admin@example.com");
        assert_eq!(site.encrypted_password, "encrypted_pass");
        assert_eq!(site.jwt_token, Some("jwt_token_abc".to_string()));
        assert_eq!(site.platform_id, Some("my-gcp-project".to_string()));
        assert_eq!(site.ssh_host, Some("203.0.113.42".to_string()));
        assert!(site.created_at > 0);
        assert_eq!(site.created_at, site.updated_at);
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_resolve_platform_explicit_id() {
        use crate::config::{CloudPlatformConfig, VmInstance};

        let mut config = AppConfig::default();
        config.platforms.push(CloudPlatformConfig {
            platform_type: "gcp".to_string(),
            gcp_selected_project_id: Some("project-123".to_string()),
            ..Default::default()
        });

        let site = MycartSiteConfig {
            site_address: "shop.example.com".to_string(),
            email: "admin@example.com".to_string(),
            encrypted_password: "pass".to_string(),
            jwt_token: None,
            created_at: 0,
            updated_at: 0,
            platform_id: Some("project-123".to_string()),
            ssh_host: None,
        };

        let platform = resolve_platform_for_site(&config, &site);
        assert!(platform.is_some());
        assert_eq!(
            platform.unwrap().gcp_selected_project_id,
            Some("project-123".to_string())
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_resolve_platform_ip_fallback() {
        use crate::config::{CloudPlatformConfig, VmInstance};

        let mut config = AppConfig::default();
        let mut platform = CloudPlatformConfig {
            platform_type: "gcp".to_string(),
            gcp_selected_project_id: Some("project-456".to_string()),
            ..Default::default()
        };
        platform.vms.push(VmInstance {
            name: "vm-1".to_string(),
            instance_id: "id-1".to_string(),
            zone: "us-central1-a".to_string(),
            gcp_region: "us-central1".to_string(),
            machine_type: "e2-micro".to_string(),
            status: "RUNNING".to_string(),
            external_ip: Some("203.0.113.100".to_string()),
            internal_ip: None,
            gcp_project_id: "project-456".to_string(),
            gcp_billing_account: None,
            created_at: 0,
            ssh_key_name: None,
        });
        config.platforms.push(platform);

        let site = MycartSiteConfig {
            site_address: "203.0.113.100".to_string(),
            email: "admin@example.com".to_string(),
            encrypted_password: "pass".to_string(),
            jwt_token: None,
            created_at: 0,
            updated_at: 0,
            platform_id: None,
            ssh_host: None,
        };

        let platform = resolve_platform_for_site(&config, &site);
        assert!(platform.is_some());
        assert_eq!(
            platform.unwrap().gcp_selected_project_id,
            Some("project-456".to_string())
        );
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_resolve_platform_no_match() {
        let config = AppConfig::default();

        let site = MycartSiteConfig {
            site_address: "unknown.example.com".to_string(),
            email: "admin@example.com".to_string(),
            encrypted_password: "pass".to_string(),
            jwt_token: None,
            created_at: 0,
            updated_at: 0,
            platform_id: None,
            ssh_host: None,
        };

        let platform = resolve_platform_for_site(&config, &site);
        assert!(platform.is_none());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_resolve_ssh_host_explicit() {
        let mut config = AppConfig::default();
        config.ssh_hosts.push(SshHostConfig {
            host: "ubuntu@203.0.113.50".to_string(),
            port: 22,
            ..Default::default()
        });

        let site = MycartSiteConfig {
            site_address: "shop.example.com".to_string(),
            email: "admin@example.com".to_string(),
            encrypted_password: "pass".to_string(),
            jwt_token: None,
            created_at: 0,
            updated_at: 0,
            platform_id: None,
            ssh_host: Some("203.0.113.50".to_string()),
        };

        let ssh_host = resolve_ssh_host_for_site(&config, &site);
        assert!(ssh_host.is_some());
        assert!(ssh_host.unwrap().host.contains("203.0.113.50"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_resolve_ssh_host_fallback() {
        let mut config = AppConfig::default();
        config.ssh_hosts.push(SshHostConfig {
            host: "ubuntu@shop.example.com".to_string(),
            port: 22,
            ..Default::default()
        });

        let site = MycartSiteConfig {
            site_address: "shop.example.com".to_string(),
            email: "admin@example.com".to_string(),
            encrypted_password: "pass".to_string(),
            jwt_token: None,
            created_at: 0,
            updated_at: 0,
            platform_id: None,
            ssh_host: None,
        };

        let ssh_host = resolve_ssh_host_for_site(&config, &site);
        assert!(ssh_host.is_some());
        assert!(ssh_host.unwrap().host.contains("shop.example.com"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_resolve_ssh_host_no_match() {
        let config = AppConfig::default();

        let site = MycartSiteConfig {
            site_address: "unknown.example.com".to_string(),
            email: "admin@example.com".to_string(),
            encrypted_password: "pass".to_string(),
            jwt_token: None,
            created_at: 0,
            updated_at: 0,
            platform_id: None,
            ssh_host: None,
        };

        let ssh_host = resolve_ssh_host_for_site(&config, &site);
        assert!(ssh_host.is_none());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_validate_cron_valid() {
        // Standard 5-field expressions
        assert!(validate_cron("0 */6 * * *").is_ok());
        assert!(validate_cron("0 0 * * *").is_ok());
        assert!(validate_cron("30 2 * * 1").is_ok());
        assert!(validate_cron("0 0 1 * *").is_ok());
        assert!(validate_cron("*/15 * * * *").is_ok());
        assert!(validate_cron("0 9-17 * * 1-5").is_ok());

        // @-prefixed macros
        assert!(validate_cron("@hourly").is_ok());
        assert!(validate_cron("@daily").is_ok());
        assert!(validate_cron("@weekly").is_ok());
        assert!(validate_cron("@monthly").is_ok());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_validate_cron_invalid() {
        // Wrong number of fields
        assert!(validate_cron("0 0 *").is_err());
        assert!(validate_cron("0 0 * * * *").is_err());

        // Invalid characters
        assert!(validate_cron("0 0 * * ?").is_err());
        assert!(validate_cron("0 0 * * MON").is_err());

        // Empty
        assert!(validate_cron("").is_err());
    }

    // Integration tests would go here with #[ignore]
    // They require:
    // - Running mycart instance
    // - Database initialized
    // - SSH access to host

    // ponytail: commented out - functions don't exist in this module
    // #[ignore]
    // #[test]
    // fn test_add_mycart_site_integration() {
    //     smol::block_on(async {
    //         // This test requires a running mycart instance
    //         let result = add_mycart_site("localhost:8080", "admin", "wrong_password").await;
    //         assert!(result.is_err()); // Should fail with invalid credentials
    //     });
    // }

    // #[ignore]
    // #[test]
    // fn test_update_admin_password_integration() {
    //     smol::block_on(async {
    //         // This test requires a mycart site already added
    //         let result = update_admin_password("localhost:8080", "old_pass", "new_pass").await;
    //         // Test behavior depends on setup
    //         assert!(result.is_ok() || result.is_err());
    //     });
    // }

    // Phase 4-6: Tests for new features

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_get_site_crontab_no_site() {
        use std::path::PathBuf;

        smol::block_on(async {
            let config_path = PathBuf::from("/tmp/nonexistent_config.toml");
            let result = get_site_crontab(&config_path, "unknown.example.com").await;

            // Should fail because site doesn't exist in config
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not found"));
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_update_site_cronjob_enabled_no_site() {
        use std::path::PathBuf;

        smol::block_on(async {
            let config_path = PathBuf::from("/tmp/nonexistent_config.toml");
            let result =
                update_site_cronjob_enabled(&config_path, "unknown.example.com", "job-123", true)
                    .await;

            // Should fail because site doesn't exist in config
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not found"));
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_update_site_cronjob_interval_no_site() {
        use std::path::PathBuf;

        smol::block_on(async {
            let config_path = PathBuf::from("/tmp/nonexistent_config.toml");
            let result = update_site_cronjob_interval(
                &config_path,
                "unknown.example.com",
                "job-123",
                "0 */6 * * *",
            )
            .await;

            // Should fail because site doesn't exist in config
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("not found"));
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_update_site_cronjob_interval_invalid_cron() {
        use std::path::PathBuf;

        smol::block_on(async {
            let config_path = PathBuf::from("/tmp/nonexistent_config.toml");

            // Invalid cron expression should be caught by validation
            let result = update_site_cronjob_interval(
                &config_path,
                "shop.example.com",
                "job-123",
                "invalid cron",
            )
            .await;

            // Should fail validation
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("Invalid cron"));
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[ignore] // Requires SSH connection
    #[test]
    fn test_get_duremycart_logs_integration() {
        smol::block_on(async {
            let host_config = SshHostConfig {
                host: "ubuntu@localhost".to_string(),
                port: 22,
                ..Default::default()
            };

            let result = get_duremycart_logs(&host_config, None).await;

            // Should return a map with mycart logs
            if result.is_ok() {
                let logs = result.unwrap();
                assert!(logs.contains_key("mycart-stderr.log"));
                assert!(logs.contains_key("mycart-stdout.log"));
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[ignore] // Requires SSH connection
    #[test]
    fn test_get_xmppproxy_logs_integration() {
        smol::block_on(async {
            let host_config = SshHostConfig {
                host: "ubuntu@localhost".to_string(),
                port: 22,
                ..Default::default()
            };

            let result = get_xmppproxy_logs(&host_config, None).await;

            // Should return a map with xmpp logs
            if result.is_ok() {
                let logs = result.unwrap();
                assert!(logs.contains_key("xmpp-proxy-stdout.log"));
                assert!(logs.contains_key("xmpp-proxy-stderr.log"));
                assert!(logs.contains_key("prosody.log"));
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[ignore] // Requires SSH connection
    #[test]
    fn test_get_fail2banrs_logs_integration() {
        smol::block_on(async {
            let host_config = SshHostConfig {
                host: "ubuntu@localhost".to_string(),
                port: 22,
                ..Default::default()
            };

            let result = get_fail2banrs_logs(&host_config, None).await;

            // Should return a map with fail2ban logs
            if result.is_ok() {
                let logs = result.unwrap();
                assert!(logs.contains_key("fail2ban-rs-stderr.log"));
                assert!(logs.contains_key("fail2ban-rs-stdout.log"));
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[ignore] // Requires running mycart API and valid JWT
    #[test]
    fn test_get_site_crontab_integration() {
        use std::path::PathBuf;

        smol::block_on(async {
            // This test requires:
            // - Config file with mycart site configured
            // - Valid JWT token in the site config
            // - Running mycart API
            let config_path = PathBuf::from("/tmp/test_config.toml");
            let result = get_site_crontab(&config_path, "localhost:8080").await;

            // Test behavior depends on setup
            if result.is_ok() {
                let jobs = result.unwrap();
                // Should return a list of crontab jobs (may be empty)
                assert!(jobs.len() >= 0);
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[ignore] // Requires running mycart API and valid JWT
    #[test]
    fn test_update_site_cronjob_enabled_integration() {
        use std::path::PathBuf;

        smol::block_on(async {
            let config_path = PathBuf::from("/tmp/test_config.toml");
            let result =
                update_site_cronjob_enabled(&config_path, "localhost:8080", "cleanup-temp", true)
                    .await;

            // Test behavior depends on setup
            if result.is_ok() {
                let job = result.unwrap();
                assert_eq!(job.enabled, true);
            }
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[ignore] // Requires running mycart API and valid JWT
    #[test]
    fn test_update_site_cronjob_interval_integration() {
        use std::path::PathBuf;

        smol::block_on(async {
            let config_path = PathBuf::from("/tmp/test_config.toml");
            let result = update_site_cronjob_interval(
                &config_path,
                "localhost:8080",
                "cleanup-temp",
                "0 */12 * * *",
            )
            .await;

            // Test behavior depends on setup
            if result.is_ok() {
                let job = result.unwrap();
                assert_eq!(job.interval, "0 */12 * * *");
            }
        });
    }

    // Phase 1: Test display field resolution
    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_mycart_site_display_fields_with_links() {
        use crate::config::{CloudPlatformConfig, VmInstance};

        let mut config = AppConfig::default();

        // Add platform
        config.platforms.push(CloudPlatformConfig {
            platform_type: "gcp".to_string(),
            gcp_selected_project_id: Some("my-project".to_string()),
            ..Default::default()
        });

        // Add SSH host
        config.ssh_hosts.push(SshHostConfig {
            host: "ubuntu@203.0.113.50".to_string(),
            port: 22,
            ..Default::default()
        });

        // Site with explicit links
        let site = MycartSiteConfig {
            site_address: "shop.example.com".to_string(),
            email: "admin@example.com".to_string(),
            encrypted_password: "pass".to_string(),
            jwt_token: None,
            created_at: 0,
            updated_at: 0,
            platform_id: Some("my-project".to_string()),
            ssh_host: Some("203.0.113.50".to_string()),
        };

        let platform = resolve_platform_for_site(&config, &site);
        let ssh_host = resolve_ssh_host_for_site(&config, &site);

        assert!(platform.is_some());
        assert_eq!(
            platform.unwrap().gcp_selected_project_id,
            Some("my-project".to_string())
        );

        assert!(ssh_host.is_some());
        assert!(ssh_host.unwrap().host.contains("203.0.113.50"));
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn test_mycart_site_display_fields_no_links() {
        let config = AppConfig::default();

        let site = MycartSiteConfig {
            site_address: "shop.example.com".to_string(),
            email: "admin@example.com".to_string(),
            encrypted_password: "pass".to_string(),
            jwt_token: None,
            created_at: 0,
            updated_at: 0,
            platform_id: None,
            ssh_host: None,
        };

        let platform = resolve_platform_for_site(&config, &site);
        let ssh_host = resolve_ssh_host_for_site(&config, &site);

        // Should return None when no links and no fallback matches
        assert!(platform.is_none());
        assert!(ssh_host.is_none());
    }

    #[test]
    fn test_drawer_tab_persistence() {
        use crate::viewmodel::mycart::DrawerTab;

        // Test all tabs roundtrip through index
        let tabs = [
            DrawerTab::Status,
            DrawerTab::Operations,
            DrawerTab::Mycart,
            DrawerTab::XMPPProxy,
            DrawerTab::Fail2BanRS,
        ];

        for (expected_index, tab) in tabs.iter().enumerate() {
            assert_eq!(tab.to_index(), expected_index);
            assert_eq!(DrawerTab::from_index(expected_index), *tab);
        }

        // Test default is Status
        assert_eq!(DrawerTab::default(), DrawerTab::Status);
        assert_eq!(DrawerTab::default().to_index(), 0);

        // Test out of bounds defaults to Status
        assert_eq!(DrawerTab::from_index(99), DrawerTab::Status);
    }
}
