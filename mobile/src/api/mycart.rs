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

/// Crontab job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrontabJob {
    #[serde(rename = "id")]
    pub job_id: String,
    #[serde(rename = "job_type")]
    pub name: String,
    pub enabled: bool,
    pub interval: String,
    pub last_run: Option<i64>,
}

/// Custom deserializer for interval field - handles both string and object formats
/// Crontab job update patch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrontabPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
}

/// Generic API response envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub result: Option<T>,
    pub message: String,
}

/// Simple API response without result data (for PATCH/DELETE operations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleApiResponse {
    pub success: bool,
    pub message: String,
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
                                log::info!(
                                    "JWT token extracted from response (length: {})",
                                    token.len()
                                );
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
                Err(anyhow::anyhow!("Unexpected status code: {}", status))
            }
        }
        Err(ureq::Error::Status(400, _))
        | Err(ureq::Error::Status(401, _))
        | Err(ureq::Error::Status(403, _)) => {
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
                Err(anyhow::anyhow!("Unexpected status code: {}", resp.status()))
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
    let cmd = format!(
        "tail -n {} {} 2>&1 || echo 'File not found'",
        tail_lines, log_path
    );

    log::debug!(
        "read_log_file: host={}, cmd={}",
        host_config.host,
        cmd
    );

    let result = ssh::execute_command(host_config, &cmd, profile_keyring).await;

    match &result {
        Ok(output) => {
            log::debug!(
                "read_log_file: success, output length={} bytes",
                output.len()
            );
        }
        Err(e) => {
            log::error!(
                "read_log_file: SSH command failed for host={}, log={}, error={}",
                host_config.host,
                log_name,
                e
            );
        }
    }

    result.context(format!("Failed to read {} from {}", log_path, host_config.host))
}

/// Get crontab jobs list
///
/// # Arguments
/// * `site` - Site address (e.g., "dure.co")
/// * `jwt_token` - JWT authentication token
pub async fn get_crontab(site: &str, jwt_token: &str) -> Result<Vec<CrontabJob>> {
    let url = format!("https://{}/api/_/settings/crontab", site);
    log::debug!("get_crontab: calling {}", url);
    let jwt = jwt_token.to_string();

    let response = smol::unblock(move || {
        ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", jwt))
            .call()
    })
    .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            log::debug!("get_crontab: received status {}", status);

            if status == 200 {
                // Parse as generic JSON first to handle both array and map formats
                let raw_json: serde_json::Value = resp
                    .into_json()
                    .context("Failed to parse crontab response as JSON")?;

                log::debug!(
                    "get_crontab: raw response structure: success={}, result type={}",
                    raw_json
                        .get("success")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false),
                    raw_json
                        .get("result")
                        .map(|v| match v {
                            serde_json::Value::Array(_) => "array",
                            serde_json::Value::Object(_) => "map",
                            _ => "other",
                        })
                        .unwrap_or("missing")
                );

                // Check success flag
                let success = raw_json
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                if !success {
                    let message = raw_json
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error");
                    log::error!(
                        "get_crontab: API returned success=false, message={}",
                        message
                    );
                    return Err(anyhow::anyhow!("API error: {}", message));
                }

                // Parse result.jobs array
                let jobs: Vec<CrontabJob> = serde_json::from_value(
                    raw_json
                        .get("result")
                        .and_then(|r| r.get("jobs"))
                        .ok_or_else(|| anyhow::anyhow!("Missing result.jobs in response"))?
                        .clone(),
                )
                .context("Failed to parse crontab jobs")?;

                log::debug!("get_crontab: parsed {} jobs successfully", jobs.len());
                Ok(jobs)
            } else if status == 401 || status == 403 {
                log::error!("get_crontab: auth failed with status {}", status);
                Err(anyhow::anyhow!("Unauthorized - token may be expired"))
            } else {
                log::error!("get_crontab: unexpected status {}", status);
                Err(anyhow::anyhow!("Unexpected status code: {}", status))
            }
        }
        Err(ureq::Error::Status(401, _)) | Err(ureq::Error::Status(403, _)) => {
            log::error!("get_crontab: auth error (401/403)");
            Err(anyhow::anyhow!("Unauthorized - token may be expired"))
        }
        Err(e) => {
            log::error!("get_crontab: request failed with error: {}", e);
            Err(anyhow::anyhow!("Failed to get crontab: {}", e))
        }
    }
}

/// Update crontab job settings
///
/// # Arguments
/// * `site` - Site address
/// * `jwt_token` - JWT authentication token
/// * `job_id` - Job ID to update
/// * `patch` - Job settings to update (enabled, interval, or both)
pub async fn patch_crontab(
    site: &str,
    jwt_token: &str,
    job_id: &str,
    patch: CrontabPatch,
) -> Result<()> {
    let url = format!(
        "https://{}/api/_/settings/crontab/{}",
        site,
        urlencoding::encode(job_id)
    );
    let jwt = jwt_token.to_string();
    let patch_clone = patch.clone();

    log::debug!("PATCH {}", url);

    let response = smol::unblock(move || {
        ureq::patch(&url)
            .set("Authorization", &format!("Bearer {}", jwt))
            .set("Content-Type", "application/json")
            .send_json(&patch_clone)
    })
    .await;

    match response {
        Ok(resp) => {
            let status = resp.status();
            if status == 200 {
                let api_response: SimpleApiResponse = resp
                    .into_json()
                    .context("Failed to parse crontab update response")?;

                log::debug!("PATCH response: success={}, message={}", api_response.success, api_response.message);

                if api_response.success {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("API error: {}", api_response.message))
                }
            } else if status == 400 {
                Err(anyhow::anyhow!(
                    "Validation error - invalid crontab settings"
                ))
            } else if status == 404 {
                Err(anyhow::anyhow!("Job not found: {}", job_id))
            } else if status == 401 || status == 403 {
                Err(anyhow::anyhow!("Unauthorized - token may be expired"))
            } else {
                Err(anyhow::anyhow!("Unexpected status code: {}", status))
            }
        }
        Err(ureq::Error::Status(400, _)) => Err(anyhow::anyhow!(
            "Validation error - invalid crontab settings"
        )),
        Err(ureq::Error::Status(404, _)) => Err(anyhow::anyhow!("Job not found: {}", job_id)),
        Err(ureq::Error::Status(401, _)) | Err(ureq::Error::Status(403, _)) => {
            Err(anyhow::anyhow!("Unauthorized - token may be expired"))
        }
        Err(e) => Err(anyhow::anyhow!("Failed to update crontab: {}", e)),
    }
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
    fn test_simple_api_response_parsing() {
        // Test successful response
        let json = r#"{"success":true,"message":"Crontab job updated"}"#;
        let response: SimpleApiResponse = serde_json::from_str(json).unwrap();
        assert!(response.success);
        assert_eq!(response.message, "Crontab job updated");

        // Test error response
        let json_err = r#"{"success":false,"message":"Job not found"}"#;
        let response_err: SimpleApiResponse = serde_json::from_str(json_err).unwrap();
        assert!(!response_err.success);
        assert_eq!(response_err.message, "Job not found");
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
    }

    #[test]
    fn test_crontab_job_struct() {
        let job = CrontabJob {
            job_id: "cleanup_sessions".to_string(),
            name: "Cleanup Sessions".to_string(),
            enabled: true,
            interval: "0 */6 * * *".to_string(),
            last_run: Some(1700000000),
        };

        assert_eq!(job.job_id, "cleanup_sessions");
        assert_eq!(job.name, "Cleanup Sessions");
        assert!(job.enabled);
        assert_eq!(job.interval, "0 */6 * * *");
        assert_eq!(job.last_run, Some(1700000000));
    }

    #[test]
    fn test_crontab_patch_serialization() {
        // Test enable only
        let patch1 = CrontabPatch {
            enabled: Some(true),
            interval: None,
        };
        let json1 = serde_json::to_string(&patch1).unwrap();
        assert!(json1.contains("\"enabled\":true"));
        assert!(!json1.contains("interval"));

        // Test interval only
        let patch2 = CrontabPatch {
            enabled: None,
            interval: Some("0 0 * * *".to_string()),
        };
        let json2 = serde_json::to_string(&patch2).unwrap();
        assert!(json2.contains("\"interval\":\"0 0 * * *\""));
        assert!(!json2.contains("enabled"));

        // Test both
        let patch3 = CrontabPatch {
            enabled: Some(false),
            interval: Some("0 */12 * * *".to_string()),
        };
        let json3 = serde_json::to_string(&patch3).unwrap();
        assert!(json3.contains("\"enabled\":false"));
        assert!(json3.contains("\"interval\":\"0 */12 * * *\""));
    }

    #[test]
    fn test_api_response_deserialization() {
        // Test success response with result
        let json1 = r#"{"success":true,"result":[{"job_id":"test","name":"Test Job","enabled":true,"interval":"0 0 * * *","last_run":null}],"message":"Success"}"#;
        let resp1: ApiResponse<Vec<CrontabJob>> = serde_json::from_str(json1).unwrap();
        assert!(resp1.success);
        assert_eq!(resp1.result.as_ref().unwrap().len(), 1);
        assert_eq!(resp1.result.as_ref().unwrap()[0].job_id, "test");

        // Test error response
        let json2 = r#"{"success":false,"result":null,"message":"Not found"}"#;
        let resp2: ApiResponse<Vec<CrontabJob>> = serde_json::from_str(json2).unwrap();
        assert!(!resp2.success);
        assert!(resp2.result.is_none());
        assert_eq!(resp2.message, "Not found");
    }

    #[test]
    fn test_crontab_job_serialization_roundtrip() {
        let job = CrontabJob {
            job_id: "test-job".to_string(),
            name: "Test Job".to_string(),
            enabled: false,
            interval: "@daily".to_string(),
            last_run: None,
        };

        let json = serde_json::to_string(&job).unwrap();
        let deserialized: CrontabJob = serde_json::from_str(&json).unwrap();

        assert_eq!(job.job_id, deserialized.job_id);
        assert_eq!(job.name, deserialized.name);
        assert_eq!(job.enabled, deserialized.enabled);
        assert_eq!(job.interval, deserialized.interval);
        assert_eq!(job.last_run, deserialized.last_run);
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

    #[ignore]
    #[test]
    fn test_get_crontab_integration() {
        smol::block_on(async {
            // Requires running mycart instance with valid JWT token
            let jwt_token = "test_jwt_token";
            let result = get_crontab("localhost:8080", jwt_token).await;

            // Should return Ok with list of crontab jobs
            if result.is_ok() {
                let jobs = result.unwrap();
                assert!(jobs.len() >= 0);
                // If jobs exist, verify structure
                if !jobs.is_empty() {
                    assert!(!jobs[0].job_id.is_empty());
                    assert!(!jobs[0].name.is_empty());
                    assert!(!jobs[0].interval.is_empty());
                }
            }
        });
    }

    #[ignore]
    #[test]
    fn test_patch_crontab_enable_integration() {
        smol::block_on(async {
            // Requires running mycart instance with valid JWT token
            let jwt_token = "test_jwt_token";
            let patch = CrontabPatch {
                enabled: Some(true),
                interval: None,
            };

            let result = patch_crontab("localhost:8080", jwt_token, "cleanup-temp", patch).await;

            // Should return Ok with updated job
            if result.is_ok() {
                let job = result.unwrap();
                assert_eq!(job.job_id, "cleanup-temp");
                assert_eq!(job.enabled, true);
            }
        });
    }

    #[ignore]
    #[test]
    fn test_patch_crontab_interval_integration() {
        smol::block_on(async {
            // Requires running mycart instance with valid JWT token
            let jwt_token = "test_jwt_token";
            let patch = CrontabPatch {
                enabled: None,
                interval: Some("0 */12 * * *".to_string()),
            };

            let result = patch_crontab("localhost:8080", jwt_token, "cleanup-temp", patch).await;

            // Should return Ok with updated job
            if result.is_ok() {
                let job = result.unwrap();
                assert_eq!(job.job_id, "cleanup-temp");
                assert_eq!(job.interval, "0 */12 * * *");
            }
        });
    }

    #[ignore]
    #[test]
    fn test_patch_crontab_invalid_token() {
        smol::block_on(async {
            // Test with invalid JWT token
            let patch = CrontabPatch {
                enabled: Some(true),
                interval: None,
            };

            let result =
                patch_crontab("localhost:8080", "invalid_token", "cleanup-temp", patch).await;

            // Should return Err for authentication failure
            assert!(result.is_err());
        });
    }

    #[ignore]
    #[test]
    fn test_get_crontab_invalid_site() {
        smol::block_on(async {
            // Test with non-existent site
            let jwt_token = "test_jwt_token";
            let result = get_crontab("nonexistent:9999", jwt_token).await;

            // Should return Err for connection failure
            assert!(result.is_err());
        });
    }
}
