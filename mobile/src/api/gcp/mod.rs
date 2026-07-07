//! Google Cloud Platform API common module
//!
//! Provides shared client, utilities, and types for all GCP services.

use anyhow::Result;
use serde::{Deserialize, Serialize};

// ============================================================================
// API Base URLs
// ============================================================================

pub const GCP_COMPUTE_API_BASE: &str = "https://compute.googleapis.com/compute/v1";
pub const GCP_RESOURCE_MANAGER_API_BASE: &str = "https://cloudresourcemanager.googleapis.com/v1";
pub const GCP_BILLING_API_BASE: &str = "https://cloudbilling.googleapis.com/v1";
pub const GCP_SERVICE_USAGE_API_BASE: &str = "https://serviceusage.googleapis.com/v1";

// ============================================================================
// GCP REST Client
// ============================================================================

/// GCP REST API client using ureq
pub struct GcpRestClient {
    access_token: String,
}

impl GcpRestClient {
    /// Create new client with access token
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }

    /// Make authenticated GET request with better error handling
    pub(crate) fn get(&self, url: &str) -> Result<ureq::Response> {
        match ureq::get(url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .set("Content-Type", "application/json")
            .call()
        {
            Ok(response) => Ok(response),
            Err(ureq::Error::Status(code, response)) => {
                let body = response.into_string().unwrap_or_default();

                // Check for API not enabled error
                if code == 403
                    && (body.contains("has not been used in project")
                        || body.contains("it is disabled"))
                {
                    let api_name = if body.contains("cloudresourcemanager") {
                        "Cloud Resource Manager API"
                    } else if body.contains("cloudbilling") {
                        "Cloud Billing API"
                    } else if body.contains("compute") {
                        "Compute Engine API"
                    } else {
                        "Required API"
                    };

                    return Err(anyhow::anyhow!(
                        "{} is not enabled. Please enable it in the GCP Console:\n{}",
                        api_name,
                        body
                    ));
                }

                Err(anyhow::anyhow!(
                    "HTTP {} error for {}:\n{}",
                    code,
                    url,
                    if body.len() > 500 {
                        format!("{}...", &body[..500])
                    } else {
                        body
                    }
                ))
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(anyhow::anyhow!("Network error for {}:\n{}", url, transport))
            }
        }
    }

    /// Make authenticated POST request
    /// Returns Response for both success and error statuses (caller must check status)
    pub(crate) fn post(&self, url: &str, body: &str) -> Result<ureq::Response> {
        match ureq::post(url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .set("Content-Type", "application/json")
            .send_string(body)
        {
            Ok(response) => Ok(response),
            Err(ureq::Error::Status(_code, response)) => {
                // Return the response so caller can inspect error
                Ok(response)
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(anyhow::anyhow!("Network error for {}: {}", url, transport))
            }
        }
    }

    /// Make authenticated DELETE request
    pub(crate) fn delete(&self, url: &str) -> Result<ureq::Response> {
        match ureq::delete(url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .set("Content-Type", "application/json")
            .call()
        {
            Ok(response) => Ok(response),
            Err(ureq::Error::Status(code, response)) => {
                let body = response.into_string().unwrap_or_default();
                Err(anyhow::anyhow!("HTTP {} error for {}: {}", code, url, body))
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(anyhow::anyhow!("Network error for {}: {}", url, transport))
            }
        }
    }

    /// Make authenticated PATCH request
    pub(crate) fn patch(&self, url: &str, body: &str) -> Result<ureq::Response> {
        match ureq::request("PATCH", url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .set("Content-Type", "application/json")
            .send_string(body)
        {
            Ok(response) => Ok(response),
            Err(ureq::Error::Status(code, response)) => {
                let body = response.into_string().unwrap_or_default();
                Err(anyhow::anyhow!("HTTP {} error for {}: {}", code, url, body))
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(anyhow::anyhow!("Network error for {}: {}", url, transport))
            }
        }
    }
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Get current public IP address from icanhazip.com
pub fn get_current_ip() -> Result<String> {
    let response = ureq::get("https://icanhazip.com")
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map_err(|e| anyhow::anyhow!("Failed to fetch IP: {}", e))?;

    let ip_text = response
        .into_string()
        .map_err(|e| anyhow::anyhow!("Failed to read IP response: {}", e))?;

    let ip = ip_text.trim().to_string();

    // Validate it looks like an IP
    if !ip.contains('.') || ip.is_empty() {
        return Err(anyhow::anyhow!("Invalid IP format: {}", ip));
    }

    Ok(ip)
}

/// Check if an IP is in any of the CIDR ranges
pub fn ip_in_ranges(ip: &str, ranges: &[String]) -> bool {
    // Simple check: exact match or 0.0.0.0/0
    ranges
        .iter()
        .any(|range| range == ip || range == &format!("{}/32", ip) || range == "0.0.0.0/0")
}

// ============================================================================
// Common Types
// ============================================================================

/// Machine type information for UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineType {
    pub name: String,
    pub description: String,
    pub cpus: i32,
    pub memory_mb: i64,
}

/// Get common machine types for UI selection
pub fn get_common_machine_types() -> Vec<MachineType> {
    vec![
        MachineType {
            name: "e2-micro".to_string(),
            description: "0.25-2 vCPU, 1 GB RAM (free tier eligible)".to_string(),
            cpus: 1,
            memory_mb: 1024,
        },
        MachineType {
            name: "e2-small".to_string(),
            description: "0.5-2 vCPU, 2 GB RAM".to_string(),
            cpus: 1,
            memory_mb: 2048,
        },
        MachineType {
            name: "e2-medium".to_string(),
            description: "1-2 vCPU, 4 GB RAM".to_string(),
            cpus: 1,
            memory_mb: 4096,
        },
        MachineType {
            name: "n1-standard-1".to_string(),
            description: "1 vCPU, 3.75 GB RAM".to_string(),
            cpus: 1,
            memory_mb: 3840,
        },
        MachineType {
            name: "n1-standard-2".to_string(),
            description: "2 vCPU, 7.5 GB RAM".to_string(),
            cpus: 2,
            memory_mb: 7680,
        },
        MachineType {
            name: "n2-standard-2".to_string(),
            description: "2 vCPU, 8 GB RAM".to_string(),
            cpus: 2,
            memory_mb: 8192,
        },
    ]
}

/// GCP Region information for UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub name: String,
    pub location: String,
    pub zones: Vec<String>,
}

/// Get common regions for UI selection
pub fn get_common_regions() -> Vec<Region> {
    vec![
        Region {
            name: "us-central1".to_string(),
            location: "Iowa, USA".to_string(),
            zones: vec![
                "us-central1-a".to_string(),
                "us-central1-b".to_string(),
                "us-central1-c".to_string(),
                "us-central1-f".to_string(),
            ],
        },
        Region {
            name: "us-east1".to_string(),
            location: "South Carolina, USA".to_string(),
            zones: vec![
                "us-east1-b".to_string(),
                "us-east1-c".to_string(),
                "us-east1-d".to_string(),
            ],
        },
        Region {
            name: "asia-northeast3".to_string(),
            location: "Seoul, South Korea".to_string(),
            zones: vec![
                "asia-northeast3-a".to_string(),
                "asia-northeast3-b".to_string(),
                "asia-northeast3-c".to_string(),
            ],
        },
        Region {
            name: "asia-northeast1".to_string(),
            location: "Tokyo, Japan".to_string(),
            zones: vec![
                "asia-northeast1-a".to_string(),
                "asia-northeast1-b".to_string(),
                "asia-northeast1-c".to_string(),
            ],
        },
        Region {
            name: "europe-west1".to_string(),
            location: "Belgium, Europe".to_string(),
            zones: vec![
                "europe-west1-b".to_string(),
                "europe-west1-c".to_string(),
                "europe-west1-d".to_string(),
            ],
        },
    ]
}


// ============================================================================
// Submodules
// ============================================================================

pub mod compute;
pub mod resourcemanager;
pub mod billing;
pub mod bigquery;
pub mod serviceusage;
pub mod dns;
pub mod oauth;

// Re-export commonly used types for convenience
// (Uncomment as types are migrated)
// pub use compute::{Instance, InstanceRequest, FirewallRule};
// pub use resourcemanager::Project;
// pub use billing::BillingAccount;
// pub use bigquery::BigQueryResponse;
// pub use oauth::OAuthHandler;
