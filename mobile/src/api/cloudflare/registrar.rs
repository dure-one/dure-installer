//! Cloudflare Registrar API client
//!
//! Provides domain registration and nameserver management.
//! API Documentation: https://developers.cloudflare.com/registrar/registrar-api/

use crate::{dure_debug, dure_error, dure_info, dure_warn};
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

/// Domain search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainSearchResult {
    pub name: String,
    pub pricing: DomainPricing,
    pub registrable: bool,
    pub tier: String,
}

/// Domain pricing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainPricing {
    pub currency: String,
    pub registration_cost: String,
    pub renewal_cost: String,
}

/// Domain registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    pub domain_name: String,
    pub auto_renew: bool,
    pub created_at: String,
    pub expires_at: String,
    pub locked: bool,
    pub privacy_mode: String,
    pub status: String,
}

/// Registration status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationStatus {
    pub completed: bool,
    pub state: String,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<RegistrationContext>,
}

/// Registration context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationContext {
    pub domain_name: String,
    pub registration: Registration,
}

/// Create registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRegistrationRequest {
    pub domain_name: String,
    pub years: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub privacy_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_renew: Option<bool>,
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

    /// Search for available domains
    pub fn domain_search(&self, account_id: &str, query: &str) -> Result<Vec<DomainSearchResult>> {
        // URL encode the query parameter
        let encoded_query = urlencoding::encode(query);
        let url = format!(
            "{}/accounts/{}/registrar/domain-search?q={}",
            API_BASE, account_id, encoded_query
        );

        dure_debug!("Searching domains for query: {}", query);
        dure_info!("Domain search URL: {}", url);

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .call();

        let response = match response {
            Ok(resp) => resp,
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_else(|_| format!("HTTP {}", code));
                dure_error!("Domain search API error {}: {}", code, body);
                anyhow::bail!("Domain search failed with status {}: {}", code, body);
            }
            Err(ureq::Error::Transport(err)) => {
                dure_error!("Domain search network error: {}", err);
                anyhow::bail!("Network error: {}", err);
            }
        };

        #[derive(Deserialize)]
        struct SearchResponse {
            result: SearchResult,
            success: bool,
            #[serde(default)]
            errors: Vec<ApiError>,
        }

        #[derive(Deserialize)]
        struct SearchResult {
            domains: Vec<DomainSearchResult>,
        }

        #[derive(Deserialize)]
        struct ApiError {
            code: u32,
            message: String,
        }

        let body = response.into_string()
            .context("Failed to read response body")?;

        dure_debug!("Domain search response: {}", &body[..body.len().min(500)]);

        let search_response: SearchResponse = serde_json::from_str(&body)
            .context(format!("Failed to parse search response: {}", &body[..body.len().min(200)]))?;

        if !search_response.success {
            let error_msg = search_response.errors.iter()
                .map(|e| format!("{}:{}", e.code, e.message))
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::bail!("Domain search failed: {}", error_msg);
        }

        let count = search_response.result.domains.len();
        dure_info!("Found {} domain results for query: {}", count, query);

        Ok(search_response.result.domains)
    }

    /// List all domain registrations
    pub fn list_registrations(&self, account_id: &str) -> Result<Vec<Registration>> {
        let url = format!(
            "{}/accounts/{}/registrar/registrations",
            API_BASE, account_id
        );

        dure_debug!("Listing registrations for account: {}", account_id);

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .call()
            .context("Failed to list registrations")?;

        #[derive(Deserialize)]
        struct RegistrationsResponse {
            result: Vec<Registration>,
            success: bool,
        }

        let registrations_response: RegistrationsResponse = response
            .into_json()
            .context("Failed to parse registrations response")?;

        if !registrations_response.success {
            anyhow::bail!("Failed to list registrations");
        }

        let count = registrations_response.result.len();
        dure_info!("Found {} registrations", count);

        Ok(registrations_response.result)
    }

    /// Create a new domain registration
    pub fn create_registration(
        &self,
        account_id: &str,
        request: CreateRegistrationRequest,
    ) -> Result<RegistrationStatus> {
        let url = format!(
            "{}/accounts/{}/registrar/registrations",
            API_BASE, account_id
        );

        dure_info!("Creating registration for domain: {}", request.domain_name);

        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .set("Content-Type", "application/json")
            .send_json(&request)
            .context("Failed to create registration")?;

        #[derive(Deserialize)]
        struct CreateRegistrationResponse {
            result: RegistrationStatus,
            success: bool,
        }

        let create_response: CreateRegistrationResponse = response
            .into_json()
            .context("Failed to parse registration response")?;

        if !create_response.success {
            anyhow::bail!("Registration creation failed");
        }

        dure_info!("✓ Registration created: {}", request.domain_name);

        Ok(create_response.result)
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

    #[test]
    fn test_domain_search_result_structure() {
        let result = DomainSearchResult {
            name: "example.com".to_string(),
            pricing: DomainPricing {
                currency: "USD".to_string(),
                registration_cost: "10.00".to_string(),
                renewal_cost: "10.00".to_string(),
            },
            registrable: true,
            tier: "standard".to_string(),
        };

        assert_eq!(result.name, "example.com");
        assert!(result.registrable);
        assert_eq!(result.pricing.currency, "USD");
    }

    #[test]
    fn test_registration_structure() {
        let registration = Registration {
            domain_name: "example.com".to_string(),
            auto_renew: true,
            created_at: "2025-01-15T10:00:00Z".to_string(),
            expires_at: "2026-01-15T10:00:00Z".to_string(),
            locked: true,
            privacy_mode: "redaction".to_string(),
            status: "active".to_string(),
        };

        assert_eq!(registration.domain_name, "example.com");
        assert!(registration.auto_renew);
        assert_eq!(registration.status, "active");
    }

    #[test]
    fn test_create_registration_request() {
        let request = CreateRegistrationRequest {
            domain_name: "example.com".to_string(),
            years: 1,
            privacy_mode: Some("redaction".to_string()),
            auto_renew: Some(true),
        };

        assert_eq!(request.domain_name, "example.com");
        assert_eq!(request.years, 1);
        assert_eq!(request.privacy_mode, Some("redaction".to_string()));
    }
}
