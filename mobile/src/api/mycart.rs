//! Mycart admin API client
//!
//! Provides API calls for mycart admin operations:
//! - Admin credential verification via JWT token
//! - Stats retrieval (orders, products, messages)
//! - Seller info retrieval
//! - Admin password and email management
//! - Log file retrieval via SSH
//!
//! Authentication: Uses JWT token stored in cookie after login

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[cfg(not(target_arch = "wasm32"))]
use crate::calc::keyring::DatabaseHandle;
#[cfg(not(target_arch = "wasm32"))]
use crate::calc::ssh;
#[cfg(not(target_arch = "wasm32"))]
use crate::config::SshHostConfig;

/// Mycart statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MycartStats {
    pub site: String,
    pub orders_count: u64,
    pub products_count: u64,
    pub messages_count: u64,
}

/// Seller information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SellerInfo {
    pub seller_id: String,
    pub seller_name: String,
    pub email: String,
    pub phone: Option<String>,
}

/// Login and get JWT token from mycart API
///
/// # Arguments
/// * `site` - Site address (e.g., "dure.co")
/// * `email` - Admin email address
/// * `password` - Admin password
///
/// # Returns
/// * `Ok(Some(token))` - Login successful, returns JWT token
/// * `Ok(None)` - Invalid credentials
/// * `Err(_)` - Network or API error
pub async fn login_and_get_token(
    site: &str,
    email: &str,
    password: &str,
) -> Result<Option<String>> {
    let url = format!("https://{}/api/sign/in", site);

    log::info!("Mycart login: POST {} (email: {})", url, email);

    let request_body = serde_json::json!({
        "email": email,
        "password": password,
    });

    let response = smol::unblock(move || {
        ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(&request_body)
    })
    .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            log::info!("Mycart login response: status {}", status);

            if status == 200 {
                // Extract JWT token from response body
                #[derive(Deserialize)]
                struct SignInResponse {
                    success: bool,
                    result: Option<String>,
                    message: Option<String>,
                }

                match resp.into_json::<SignInResponse>() {
                    Ok(data) => {
                        if data.success {
                            if let Some(token) = data.result {
                                log::info!("JWT token extracted from response (length: {})", token.len());
                                return Ok(Some(token));
                            } else {
                                log::error!("Login successful but no token in response");
                                return Err(anyhow::anyhow!("No token in response"));
                            }
                        } else {
                            log::warn!("Login failed: {}", data.message.unwrap_or_default());
                            return Ok(None);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to parse login response: {}", e);
                        return Err(anyhow::anyhow!("Failed to parse response: {}", e));
                    }
                }
            } else if status == 400 || status == 401 || status == 403 {
                log::warn!("Login failed: invalid credentials (status {})", status);
                Ok(None)
            } else {
                log::error!("Login failed with unexpected status: {}", status);
                Err(anyhow::anyhow!(
                    "Unexpected status code: {}",
                    status
                ))
            }
        }
        Err(ureq::Error::Status(400, _)) | Err(ureq::Error::Status(401, _)) | Err(ureq::Error::Status(403, _)) => {
            log::warn!("Login failed: invalid credentials");
            Ok(None)
        }
        Err(e) => {
            log::error!("Login network error: {}", e);
            Err(anyhow::anyhow!("Failed to login: {}", e))
        }
    }
}

/// Verify admin credentials and get JWT token
///
/// This function combines login and version check to verify credentials.
///
/// # Arguments
/// * `site` - Site address (e.g., "dure.co")
/// * `email` - Admin email address
/// * `password` - Admin password
///
/// # Returns
/// * `Ok(Some(token))` - Credentials valid, returns JWT token
/// * `Ok(None)` - Credentials invalid
/// * `Err(_)` - Network or API error
pub async fn verify_admin_access(
    site: &str,
    email: &str,
    password: &str,
) -> Result<Option<String>> {
    log::info!("Verifying admin access for {} (email: {})", site, email);

    // Step 1: Login and get JWT token
    let token = login_and_get_token(site, email, password).await?;

    if let Some(ref jwt) = token {
        // Step 2: Verify token works by calling /api/_/version
        let url = format!("https://{}/api/_/version", site);
        log::info!("Verifying token with GET {}", url);
        let jwt_clone = jwt.clone();

        let response = smol::unblock(move || {
            ureq::get(&url)
                .set("Authorization", &format!("Bearer {}", jwt_clone))
                .call()
        })
        .await;

        match response {
            Ok(resp) if resp.status() == 200 => {
                log::info!("Token verification successful");
                Ok(Some(jwt.clone()))
            }
            Ok(resp) if resp.status() == 401 || resp.status() == 403 => {
                log::warn!("Token verification failed: unauthorized");
                Ok(None)
            }
            Ok(resp) => {
                log::error!("Token verification failed with status: {}", resp.status());
                Err(anyhow::anyhow!(
                    "Unexpected status code from /api/_/version: {}",
                    resp.status()
                ))
            }
            Err(ureq::Error::Status(401, _)) | Err(ureq::Error::Status(403, _)) => {
                log::warn!("Token verification failed: unauthorized");
                Ok(None)
            }
            Err(e) => {
                log::error!("Token verification network error: {}", e);
                Err(anyhow::anyhow!("Failed to verify token: {}", e))
            }
        }
    } else {
        log::warn!("Login returned no token - credentials invalid");
        Ok(None)
    }
}

/// Get mycart statistics (orders, products, messages counts)
///
/// # Arguments
/// * `site` - Site address
/// * `jwt_token` - JWT authentication token
///
/// # Note
/// ponytail: Backend endpoint not implemented yet, returns stub data
pub async fn get_mycart_stats(site: &str, _jwt_token: &str) -> Result<MycartStats> {
    // ponytail: Backend /api/_/stats endpoint not implemented yet
    // Return stub data until backend is ready
    Ok(MycartStats {
        site: site.to_string(),
        orders_count: 0,
        products_count: 0,
        messages_count: 0,
    })
}

/// Get seller information
///
/// # Arguments
/// * `site` - Site address
/// * `jwt_token` - JWT authentication token
pub async fn get_seller_info(site: &str, jwt_token: &str) -> Result<SellerInfo> {
    let url = format!("https://{}/api/sellerinfo", site);
    let jwt = jwt_token.to_string();

    let response = smol::unblock(move || {
        ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", jwt))
            .call()
    })
    .await;

    match response {
        Ok(resp) => {
            if resp.status() == 200 {
                let seller: SellerInfo = resp
                    .into_json()
                    .context("Failed to parse seller info response")?;
                Ok(seller)
            } else if resp.status() == 401 || resp.status() == 403 {
                Err(anyhow::anyhow!("Unauthorized - token may be expired"))
            } else {
                Err(anyhow::anyhow!(
                    "Unexpected status code: {}",
                    resp.status()
                ))
            }
        }
        Err(ureq::Error::Status(401, _)) | Err(ureq::Error::Status(403, _)) => {
            Err(anyhow::anyhow!("Unauthorized - token may be expired"))
        }
        Err(e) => Err(anyhow::anyhow!("Failed to get seller info: {}", e)),
    }
}

/// List log files in /srv/data/logs via SSH
#[cfg(not(target_arch = "wasm32"))]
pub async fn list_log_files(
    host_config: &SshHostConfig,
    profile_keyring: Option<&Arc<DatabaseHandle>>,
) -> Result<Vec<String>> {
    let log_dir = "/srv/data/logs";
    let cmd = format!("ls -1 {}", log_dir);

    let output = ssh::execute_command(host_config, &cmd, profile_keyring)
        .await
        .context("Failed to list log files")?;

    let files: Vec<String> = output
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect();

    Ok(files)
}

/// Read log file content via SSH
#[cfg(not(target_arch = "wasm32"))]
pub async fn read_log_file(
    host_config: &SshHostConfig,
    log_name: &str,
    tail_lines: usize,
    profile_keyring: Option<&Arc<DatabaseHandle>>,
) -> Result<String> {
    let log_path = format!("/srv/data/logs/{}", log_name);
    let cmd = format!("tail -n {} {} 2>&1 || echo 'File not found'", tail_lines, log_path);

    ssh::execute_command(host_config, &cmd, profile_keyring)
        .await
        .context("Failed to read log file")
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock test - actual API calls would require a running mycart instance
    #[test]
    fn test_mycart_stats_struct() {
        let stats = MycartStats {
            site: "dure.co".to_string(),
            orders_count: 42,
            products_count: 100,
            messages_count: 5,
        };

        assert_eq!(stats.site, "dure.co");
        assert_eq!(stats.orders_count, 42);
        assert_eq!(stats.products_count, 100);
        assert_eq!(stats.messages_count, 5);
    }

    #[test]
    fn test_seller_info_struct() {
        let seller = SellerInfo {
            seller_id: "seller123".to_string(),
            seller_name: "Test Shop".to_string(),
            email: "test@shop.com".to_string(),
            phone: Some("+1234567890".to_string()),
        };

        assert_eq!(seller.seller_id, "seller123");
        assert_eq!(seller.seller_name, "Test Shop");
        assert!(seller.phone.is_some());
    }

    // Integration tests for actual API calls would go here
    // They would be marked with #[ignore] and run only when mycart is available

    #[ignore]
    #[test]
    fn test_verify_admin_access_integration() {
        smol::block_on(async {
            // Requires running mycart instance
            let result = verify_admin_access("localhost:8080", "admin", "wrong_password").await;
            // Should return Ok(None) for invalid credentials
            assert!(result.is_ok());
            assert!(result.unwrap().is_none());
        });
    }

    #[ignore]
    #[test]
    fn test_login_and_get_token_integration() {
        smol::block_on(async {
            // Requires running mycart instance with valid credentials
            let result = login_and_get_token("localhost:8080", "admin", "correct_password").await;
            // Should return Ok(Some(token)) for valid credentials
            assert!(result.is_ok());
            assert!(result.unwrap().is_some());
        });
    }
}
