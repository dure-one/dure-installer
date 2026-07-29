//! Google Cloud DNS API implementation
//!
//! Google Cloud DNS API for managing managed zones and resource record sets.
//! API Documentation: https://cloud.google.com/dns/docs/reference/v1

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const API_BASE: &str = "https://dns.googleapis.com/dns/v1";

/// Google Cloud DNS API client
pub struct GcpDnsClient {
    access_token: String,
}

/// Project information
#[derive(Debug, Deserialize)]
pub struct Project {
    pub id: String,
    pub number: String,
}

/// Managed zone list response
#[derive(Debug, Deserialize)]
struct ManagedZonesResponse {
    #[serde(default)]
    #[serde(rename = "managedZones")]
    managed_zones: Vec<ManagedZone>,
}

/// Managed zone
#[derive(Debug, Deserialize, Serialize)]
pub struct ManagedZone {
    pub id: String,
    pub name: String,
    #[serde(rename = "dnsName")]
    pub dns_name: String,
    #[serde(default)]
    #[serde(rename = "nameServers")]
    pub name_servers: Vec<String>,
}

/// DNSSEC configuration
#[derive(Debug, Serialize)]
struct DnssecConfig {
    state: String, // "on" or "off"
}

/// Managed zone creation request
#[derive(Debug, Serialize)]
struct CreateManagedZoneRequest {
    name: String,
    #[serde(rename = "dnsName")]
    dns_name: String,
    description: String,
    #[serde(rename = "dnssecConfig")]
    dnssec_config: DnssecConfig,
}

/// Resource record sets list response
#[derive(Debug, Deserialize)]
struct ResourceRecordSetsResponse {
    #[serde(default)]
    rrsets: Vec<ResourceRecordSet>,
}

/// Resource record set
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ResourceRecordSet {
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub ttl: u32,
    #[serde(default)]
    pub rrdatas: Vec<String>,
}

/// Resource record set creation/update request
#[derive(Debug, Serialize)]
struct ResourceRecordSetRequest {
    name: String,
    #[serde(rename = "type")]
    record_type: String,
    ttl: u32,
    rrdatas: Vec<String>,
}

impl GcpDnsClient {
    /// Create a new GCP DNS client with an access token
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }

    /// Get project information
    pub fn get_project(&self, project_id: &str) -> Result<Project> {
        let url = format!("{}/projects/{}", API_BASE, project_id);

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .call()
            .context("Failed to call GCP DNS API")?;

        let project: Project = response
            .into_json()
            .context("Failed to parse project response")?;

        Ok(project)
    }

    /// List all managed zones in a project
    pub fn list_managed_zones(&self, project_id: &str) -> Result<Vec<ManagedZone>> {
        let url = format!("{}/projects/{}/managedZones", API_BASE, project_id);

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .call()
            .context("Failed to call GCP DNS API")?;

        let result: ManagedZonesResponse = response
            .into_json()
            .context("Failed to parse managed zones response")?;

        Ok(result.managed_zones)
    }

    /// Get a managed zone by name
    pub fn get_managed_zone(&self, project_id: &str, managed_zone: &str) -> Result<ManagedZone> {
        let url = format!(
            "{}/projects/{}/managedZones/{}",
            API_BASE, project_id, managed_zone
        );

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .call()
            .context("Failed to call GCP DNS API")?;

        let zone: ManagedZone = response
            .into_json()
            .context("Failed to parse managed zone response")?;

        Ok(zone)
    }

    /// Create a new managed zone with DNSSEC enabled
    pub fn create_managed_zone(&self, project_id: &str, domain: &str) -> Result<ManagedZone> {
        let url = format!("{}/projects/{}/managedZones", API_BASE, project_id);

        // Convert domain to zone name format (replace dots with hyphens)
        // e.g., google.com -> google-com
        let zone_name = domain.replace('.', "-");

        // DNS name must end with a dot
        let dns_name = if domain.ends_with('.') {
            domain.to_string()
        } else {
            format!("{}.", domain)
        };

        let request_body = CreateManagedZoneRequest {
            name: zone_name,
            dns_name,
            description: format!("Managed zone for {}", domain),
            dnssec_config: DnssecConfig {
                state: "on".to_string(),
            },
        };

        let body_json = serde_json::to_string(&request_body)
            .context("Failed to serialize zone creation request")?;

        let response = match ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .set("Content-Type", "application/json")
            .send_string(&body_json)
        {
            Ok(resp) => resp,
            Err(ureq::Error::Status(code, resp)) => {
                let error_text = resp
                    .into_string()
                    .unwrap_or_else(|_| format!("HTTP error {}", code));
                anyhow::bail!("GCP DNS API returned status {}: {}", code, error_text);
            }
            Err(ureq::Error::Transport(transport_err)) => {
                anyhow::bail!("Network error calling GCP DNS API: {}", transport_err);
            }
        };

        let zone: ManagedZone = response
            .into_json()
            .context("Failed to parse zone creation response")?;

        Ok(zone)
    }

    /// List all resource record sets in a managed zone
    pub fn list_rrsets(
        &self,
        project_id: &str,
        managed_zone: &str,
    ) -> Result<Vec<ResourceRecordSet>> {
        let url = format!(
            "{}/projects/{}/managedZones/{}/rrsets",
            API_BASE, project_id, managed_zone
        );

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .call()
            .context("Failed to call GCP DNS API")?;

        let result: ResourceRecordSetsResponse = response
            .into_json()
            .context("Failed to parse rrsets response")?;

        Ok(result.rrsets)
    }

    /// Create a new resource record set
    pub fn create_rrset(
        &self,
        project_id: &str,
        managed_zone: &str,
        name: &str,
        record_type: &str,
        ttl: u32,
        rrdatas: Vec<String>,
    ) -> Result<()> {
        let url = format!(
            "{}/projects/{}/managedZones/{}/rrsets",
            API_BASE, project_id, managed_zone
        );

        let request = ResourceRecordSetRequest {
            name: name.to_string(),
            record_type: record_type.to_uppercase(),
            ttl,
            rrdatas,
        };

        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .set("Content-Type", "application/json")
            .send_json(&request)
            .context("Failed to create resource record set")?;

        // Check if response indicates success (2xx status code)
        if response.status() >= 200 && response.status() < 300 {
            Ok(())
        } else {
            let error_text = response
                .into_string()
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to create rrset: {}", error_text)
        }
    }

    /// Update a resource record set
    pub fn update_rrset(
        &self,
        project_id: &str,
        managed_zone: &str,
        name: &str,
        record_type: &str,
        ttl: u32,
        rrdatas: Vec<String>,
    ) -> Result<()> {
        let url = format!(
            "{}/projects/{}/managedZones/{}/rrsets/{}/{}",
            API_BASE, project_id, managed_zone, name, record_type
        );

        let request = ResourceRecordSetRequest {
            name: name.to_string(),
            record_type: record_type.to_uppercase(),
            ttl,
            rrdatas,
        };

        let response = ureq::request("PATCH", &url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .set("Content-Type", "application/json")
            .send_json(&request)
            .context("Failed to update resource record set")?;

        // Check if response indicates success (2xx status code)
        if response.status() >= 200 && response.status() < 300 {
            Ok(())
        } else {
            let error_text = response
                .into_string()
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to update rrset: {}", error_text)
        }
    }

    /// Delete a resource record set
    pub fn delete_rrset(
        &self,
        project_id: &str,
        managed_zone: &str,
        name: &str,
        record_type: &str,
    ) -> Result<()> {
        let url = format!(
            "{}/projects/{}/managedZones/{}/rrsets/{}/{}",
            API_BASE, project_id, managed_zone, name, record_type
        );

        let response = ureq::delete(&url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .call()
            .context("Failed to delete resource record set")?;

        // Check if response indicates success (2xx status code)
        if response.status() >= 200 && response.status() < 300 {
            Ok(())
        } else {
            let error_text = response
                .into_string()
                .unwrap_or_else(|_| "Unknown error".to_string());
            anyhow::bail!("Failed to delete rrset: {}", error_text)
        }
    }

    /// Find a managed zone by domain name
    pub fn find_zone_by_domain(
        &self,
        project_id: &str,
        domain: &str,
    ) -> Result<Option<ManagedZone>> {
        let zones = self.list_managed_zones(project_id)?;
        // Ensure domain ends with a dot for comparison
        let domain_normalized = if domain.ends_with('.') {
            domain.to_string()
        } else {
            format!("{}.", domain)
        };
        Ok(zones.into_iter().find(|z| z.dns_name == domain_normalized))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = GcpDnsClient::new("test-token".to_string());
        assert_eq!(client.access_token, "test-token");
    }

    #[test]
    fn test_domain_normalization() {
        let domain1 = "example.com";
        let domain2 = "example.com.";
        let normalized = if domain1.ends_with('.') {
            domain1.to_string()
        } else {
            format!("{}.", domain1)
        };
        assert_eq!(normalized, domain2);
    }
}
