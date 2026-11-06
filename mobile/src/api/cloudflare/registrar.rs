//! Cloudflare Registrar API client
//!
//! Provides domain registration and nameserver management.
//! API Documentation: https://developers.cloudflare.com/registrar/registrar-api/

use crate::{dure_debug, dure_info, dure_warn};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const API_BASE: &str = "https://api.cloudflare.com/client/v4";

/// Cloudflare Registrar API client
pub struct CloudflareRegistrarClient {
    api_token: String,
}

/// Domain information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Domain {
    pub id: String,
    pub name: String,
    pub status: String,
    pub nameservers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

impl Domain {
    /// Check if domain uses Cloudflare nameservers
    pub fn has_cloudflare_nameservers(&self) -> bool {
        self.nameservers
            .iter()
            .any(|ns| ns.contains("cloudflare.com"))
    }

    /// Get nameserver status summary
    pub fn nameserver_status(&self) -> String {
        if self.has_cloudflare_nameservers() {
            format!("✓ Using Cloudflare NS: {}", self.nameservers.join(", "))
        } else {
            format!("⚠ External NS: {}", self.nameservers.join(", "))
        }
    }
}

impl CloudflareRegistrarClient {
    /// Create a new Registrar client
    pub fn new(api_token: String) -> Self {
        Self { api_token }
    }

    /// Get API token (for testing)
    pub fn api_token(&self) -> &str {
        &self.api_token
    }

    /// List all domains in account
    pub fn list_domains(&self, account_id: &str) -> Result<Vec<Domain>> {
        let url = format!("{}/accounts/{}/registrar/domains", API_BASE, account_id);

        dure_debug!("Listing domains for account: {}", account_id);

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .call()
            .context("Failed to call Cloudflare Registrar API")?;

        #[derive(Deserialize)]
        struct DomainsResponse {
            result: Vec<Domain>,
            success: bool,
        }

        let domains_response: DomainsResponse = response
            .into_json()
            .context("Failed to parse domains response")?;

        if !domains_response.success {
            anyhow::bail!("Cloudflare API returned success: false");
        }

        dure_info!("Found {} domains", domains_response.result.len());

        Ok(domains_response.result)
    }

    /// Get domain information
    pub fn get_domain(&self, account_id: &str, domain: &str) -> Result<Domain> {
        let url = format!(
            "{}/accounts/{}/registrar/domains/{}",
            API_BASE, account_id, domain
        );

        dure_debug!("Getting domain info: {}", domain);

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .call()
            .context("Failed to get domain info")?;

        #[derive(Deserialize)]
        struct DomainResponse {
            result: Domain,
            success: bool,
        }

        let domain_response: DomainResponse = response
            .into_json()
            .context("Failed to parse domain response")?;

        if !domain_response.success {
            anyhow::bail!("Failed to get domain information");
        }

        Ok(domain_response.result)
    }

    /// Check domain availability for registration
    pub fn check_availability(&self, account_id: &str, domain: &str) -> Result<bool> {
        let url = format!(
            "{}/accounts/{}/registrar/domains/{}/check",
            API_BASE, account_id, domain
        );

        dure_debug!("Checking availability for: {}", domain);

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .call()
            .context("Failed to check domain availability")?;

        #[derive(Deserialize)]
        struct AvailabilityResponse {
            result: AvailabilityResult,
            success: bool,
        }

        #[derive(Deserialize)]
        struct AvailabilityResult {
            available: bool,
        }

        let availability: AvailabilityResponse = response
            .into_json()
            .context("Failed to parse availability response")?;

        if !availability.success {
            anyhow::bail!("Failed to check domain availability");
        }

        let available = availability.result.available;
        dure_info!(
            "Domain {} is {}",
            domain,
            if available { "available" } else { "taken" }
        );

        Ok(available)
    }

    /// Register a new domain
    pub fn register_domain(&self, account_id: &str, domain: &str, years: u32) -> Result<Domain> {
        let url = format!("{}/accounts/{}/registrar/domains", API_BASE, account_id);

        dure_info!("Registering domain: {} for {} years", domain, years);

        #[derive(Serialize)]
        struct RegisterRequest {
            name: String,
            years: u32,
        }

        let request = RegisterRequest {
            name: domain.to_string(),
            years,
        };

        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .set("Content-Type", "application/json")
            .send_json(&request)
            .context("Failed to register domain")?;

        #[derive(Deserialize)]
        struct RegisterResponse {
            result: Domain,
            success: bool,
        }

        let register_response: RegisterResponse = response
            .into_json()
            .context("Failed to parse registration response")?;

        if !register_response.success {
            anyhow::bail!("Domain registration failed");
        }

        dure_info!("✓ Domain registered successfully: {}", domain);

        Ok(register_response.result)
    }

    /// Update domain nameservers
    pub fn update_nameservers(
        &self,
        account_id: &str,
        domain: &str,
        nameservers: &[String],
    ) -> Result<()> {
        let url = format!(
            "{}/accounts/{}/registrar/domains/{}",
            API_BASE, account_id, domain
        );

        dure_info!("Updating nameservers for: {}", domain);

        #[derive(Serialize)]
        struct UpdateRequest {
            nameservers: Vec<String>,
        }

        let request = UpdateRequest {
            nameservers: nameservers.to_vec(),
        };

        let response = ureq::patch(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .set("Content-Type", "application/json")
            .send_json(&request)
            .context("Failed to update nameservers")?;

        #[derive(Deserialize)]
        struct UpdateResponse {
            success: bool,
        }

        let update_response: UpdateResponse = response
            .into_json()
            .context("Failed to parse update response")?;

        if !update_response.success {
            anyhow::bail!("Nameserver update failed");
        }

        dure_info!("✓ Nameservers updated successfully");

        Ok(())
    }

    /// Get Cloudflare's recommended nameservers for a domain
    pub fn get_cloudflare_nameservers(&self, account_id: &str, domain: &str) -> Result<Vec<String>> {
        // First, get the domain info which includes assigned nameservers
        let domain_info = self.get_domain(account_id, domain)?;

        if domain_info.nameservers.is_empty() {
            // If no nameservers set, return default Cloudflare NS
            Ok(vec![
                "ns1.cloudflare.com".to_string(),
                "ns2.cloudflare.com".to_string(),
            ])
        } else {
            Ok(domain_info.nameservers)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registrar_client_creation() {
        let client = CloudflareRegistrarClient::new("token123".into());
        assert_eq!(client.api_token(), "token123");
    }

    #[test]
    fn test_domain_has_cloudflare_nameservers() {
        let domain = Domain {
            id: "test123".to_string(),
            name: "example.com".to_string(),
            status: "active".to_string(),
            nameservers: vec![
                "ns1.cloudflare.com".to_string(),
                "ns2.cloudflare.com".to_string(),
            ],
            created_at: None,
            expires_at: None,
        };

        assert!(domain.has_cloudflare_nameservers());
    }

    #[test]
    fn test_domain_without_cloudflare_nameservers() {
        let domain = Domain {
            id: "test123".to_string(),
            name: "example.com".to_string(),
            status: "active".to_string(),
            nameservers: vec![
                "ns1.example.com".to_string(),
                "ns2.example.com".to_string(),
            ],
            created_at: None,
            expires_at: None,
        };

        assert!(!domain.has_cloudflare_nameservers());
    }

    #[test]
    fn test_nameserver_status() {
        let domain = Domain {
            id: "test123".to_string(),
            name: "example.com".to_string(),
            status: "active".to_string(),
            nameservers: vec!["ns1.cloudflare.com".to_string()],
            created_at: None,
            expires_at: None,
        };

        let status = domain.nameserver_status();
        assert!(status.contains("✓ Using Cloudflare NS"));
    }
}
