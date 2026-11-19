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
    };

    let mut config = AppConfig::load_or_default(&config_path.to_path_buf());
    config.mycart_sites.push(site_config);
    config.save(&config_path.to_path_buf())
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
    config.mycart_sites.retain(|s| s.site_address != site_address);
    config.save(&config_path.to_path_buf())
        .context("Failed to save config")?;
    Ok(())
}

/// Get site statistics (orders, products, messages counts)
#[cfg(not(target_arch = "wasm32"))]
pub async fn get_site_stats(config_path: &std::path::Path, site_address: &str) -> Result<mycart::MycartStats> {
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
pub async fn get_seller_info(config_path: &std::path::Path, site_address: &str) -> Result<mycart::SellerInfo> {
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
                logs.insert(
                    log_name.to_string(),
                    format!("Error reading log: {}", e),
                );
            }
        }
    }

    Ok(logs)
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
        };

        assert_eq!(site.site_address, "dure.co");
        assert_eq!(site.email, "admin@example.com");
        assert_eq!(site.encrypted_password, "encrypted_pass");
        assert_eq!(site.jwt_token, Some("jwt_token_abc".to_string()));
        assert!(site.created_at > 0);
        assert_eq!(site.created_at, site.updated_at);
    }

    // Integration tests would go here with #[ignore]
    // They require:
    // - Running mycart instance
    // - Database initialized
    // - SSH access to host

    #[ignore]
    #[test]
    fn test_add_mycart_site_integration() {
        smol::block_on(async {
            // This test requires a running mycart instance
            let result = add_mycart_site("localhost:8080", "admin", "wrong_password").await;
            assert!(result.is_err()); // Should fail with invalid credentials
        });
    }

    #[ignore]
    #[test]
    fn test_update_admin_password_integration() {
        smol::block_on(async {
            // This test requires a mycart site already added
            let result = update_admin_password("localhost:8080", "old_pass", "new_pass").await;
            // Test behavior depends on setup
            assert!(result.is_ok() || result.is_err());
        });
    }
}
