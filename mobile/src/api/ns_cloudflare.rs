//! Cloudflare API implementation
//!
//! Cloudflare DNS API for managing zones and DNS records.
//! API Documentation: https://developers.cloudflare.com/api/

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const API_BASE: &str = "https://api.cloudflare.com/client/v4";

/// Cloudflare API client
pub struct CloudflareClient {
    api_token: String,
}

/// Zone list response
#[derive(Debug, Deserialize)]
struct ZoneListResponse {
    result: Vec<Zone>,
    success: bool,
    #[serde(default)]
    errors: Vec<ApiError>,
}

/// Zone information
#[derive(Debug, Deserialize)]
pub struct Zone {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub name_servers: Vec<String>,
}

/// Zone creation request
#[derive(Debug, Serialize)]
struct CreateZoneRequest {
    name: String,
    #[serde(rename = "type")]
    zone_type: String,
}

/// Zone creation response
#[derive(Debug, Deserialize)]
struct CreateZoneResponse {
    result: Zone,
    success: bool,
    #[serde(default)]
    errors: Vec<ApiError>,
}

/// DNS records list response
#[derive(Debug, Deserialize)]
struct DnsRecordsResponse {
    result: Vec<DnsRecord>,
    success: bool,
    #[serde(default)]
    errors: Vec<ApiError>,
}

/// DNS record
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DnsRecord {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub content: String,
    #[serde(default)]
    pub ttl: u32,
    #[serde(default)]
    pub proxied: bool,
}

/// API error
#[derive(Debug, Deserialize)]
struct ApiError {
    code: u32,
    message: String,
}

/// Generic API response
#[derive(Debug, Deserialize)]
struct ApiResponse {
    success: bool,
    #[serde(default)]
    errors: Vec<ApiError>,
}

/// DNS record creation/update request
#[derive(Debug, Serialize)]
struct DnsRecordRequest {
    name: String,
    #[serde(rename = "type")]
    record_type: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ttl: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    proxied: Option<bool>,
}

impl CloudflareClient {
    /// Create a new Cloudflare client
    pub fn new(api_token: String) -> Self {
        Self { api_token }
    }

    /// List all zones
    pub fn list_zones(&self) -> Result<Vec<Zone>> {
        let url = format!("{}/zones", API_BASE);

        let response = match ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .call()
        {
            Ok(resp) => resp,
            Err(ureq::Error::Status(code, resp)) => {
                // Handle HTTP error status codes
                let error_text = resp
                    .into_string()
                    .unwrap_or_else(|_| format!("HTTP error {}", code));
                anyhow::bail!("Cloudflare API returned status {}: {}", code, error_text);
            }
            Err(ureq::Error::Transport(transport_err)) => {
                anyhow::bail!("Network error calling Cloudflare API: {}", transport_err);
            }
        };

        // Get the response body as text first for debugging
        let body = response
            .into_string()
            .context("Failed to read response body")?;

        // Try to parse the JSON
        let result: ZoneListResponse = serde_json::from_str(&body).context(format!(
            "Failed to parse zones response. Body: {}",
            if body.len() > 500 {
                &body[..500]
            } else {
                &body
            }
        ))?;

        if !result.success {
            let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
            anyhow::bail!("Cloudflare API error: {}", errors.join(", "));
        }

        eprintln!("DEBUG: Cloudflare returned {} zones", result.result.len());
        for zone in &result.result {
            eprintln!("DEBUG:   - {} ({})", zone.name, zone.id);
        }

        Ok(result.result)
    }

    /// Create a new zone
    pub fn create_zone(&self, domain: &str) -> Result<Zone> {
        let url = format!("{}/zones", API_BASE);

        let request_body = CreateZoneRequest {
            name: domain.to_string(),
            zone_type: "full".to_string(),
        };

        let body_json = serde_json::to_string(&request_body)
            .context("Failed to serialize zone creation request")?;

        let response = match ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .set("Content-Type", "application/json")
            .send_string(&body_json)
        {
            Ok(resp) => resp,
            Err(ureq::Error::Status(code, resp)) => {
                let error_text = resp
                    .into_string()
                    .unwrap_or_else(|_| format!("HTTP error {}", code));
                anyhow::bail!("Cloudflare API returned status {}: {}", code, error_text);
            }
            Err(ureq::Error::Transport(transport_err)) => {
                anyhow::bail!("Network error calling Cloudflare API: {}", transport_err);
            }
        };

        let body = response
            .into_string()
            .context("Failed to read response body")?;

        let result: CreateZoneResponse = serde_json::from_str(&body).context(format!(
            "Failed to parse zone creation response. Body: {}",
            if body.len() > 500 {
                &body[..500]
            } else {
                &body
            }
        ))?;

        if !result.success {
            let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
            anyhow::bail!("Cloudflare API error: {}", errors.join(", "));
        }

        Ok(result.result)
    }

    /// Get all DNS records for a zone
    pub fn get_records(&self, zone_id: &str) -> Result<Vec<DnsRecord>> {
        let url = format!("{}/zones/{}/dns_records", API_BASE, zone_id);

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .call()
            .context("Failed to call Cloudflare API")?;

        let result: DnsRecordsResponse = response
            .into_json()
            .context("Failed to parse DNS records response")?;

        if !result.success {
            let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
            anyhow::bail!("Cloudflare API error: {}", errors.join(", "));
        }

        Ok(result.result)
    }

    /// Find zone by domain name
    pub fn find_zone_by_domain(&self, domain: &str) -> Result<Option<Zone>> {
        let zones = self.list_zones()?;
        Ok(zones.into_iter().find(|z| z.name == domain))
    }

    /// Create a new DNS record
    pub fn create_record(
        &self,
        zone_id: &str,
        name: &str,
        record_type: &str,
        content: &str,
        ttl: Option<u32>,
        proxied: Option<bool>,
    ) -> Result<()> {
        let url = format!("{}/zones/{}/dns_records", API_BASE, zone_id);

        let request = DnsRecordRequest {
            name: name.to_string(),
            record_type: record_type.to_uppercase(),
            content: content.to_string(),
            ttl,
            proxied,
        };

        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .set("Content-Type", "application/json")
            .send_json(&request)
            .context("Failed to create DNS record")?;

        let result: ApiResponse = response
            .into_json()
            .context("Failed to parse create response")?;

        if !result.success {
            let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
            anyhow::bail!("Failed to create record: {}", errors.join(", "));
        }

        Ok(())
    }

    /// Update an existing DNS record
    pub fn update_record(
        &self,
        zone_id: &str,
        record_id: &str,
        name: &str,
        record_type: &str,
        content: &str,
        ttl: Option<u32>,
        proxied: Option<bool>,
    ) -> Result<()> {
        let url = format!("{}/zones/{}/dns_records/{}", API_BASE, zone_id, record_id);

        let request = DnsRecordRequest {
            name: name.to_string(),
            record_type: record_type.to_uppercase(),
            content: content.to_string(),
            ttl,
            proxied,
        };

        let response = ureq::patch(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .set("Content-Type", "application/json")
            .send_json(&request)
            .context("Failed to update DNS record")?;

        let result: ApiResponse = response
            .into_json()
            .context("Failed to parse update response")?;

        if !result.success {
            let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
            anyhow::bail!("Failed to update record: {}", errors.join(", "));
        }

        Ok(())
    }

    /// Delete a DNS record
    pub fn delete_record(&self, zone_id: &str, record_id: &str) -> Result<()> {
        let url = format!("{}/zones/{}/dns_records/{}", API_BASE, zone_id, record_id);

        let response = ureq::delete(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .call()
            .context("Failed to delete DNS record")?;

        let result: ApiResponse = response
            .into_json()
            .context("Failed to parse delete response")?;

        if !result.success {
            let errors: Vec<String> = result.errors.iter().map(|e| e.message.clone()).collect();
            anyhow::bail!("Failed to delete record: {}", errors.join(", "));
        }

        Ok(())
    }

    /// Find a DNS record by name and type
    pub fn find_record(
        &self,
        zone_id: &str,
        name: &str,
        record_type: &str,
    ) -> Result<Option<DnsRecord>> {
        let records = self.get_records(zone_id)?;
        Ok(records
            .into_iter()
            .find(|r| r.name == name && r.record_type.to_uppercase() == record_type.to_uppercase()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = CloudflareClient::new("test-token".to_string());
        assert_eq!(client.api_token, "test-token");
    }
}
