//! Porkbun API implementation
//!
//! Porkbun is a domain registrar and DNS provider with a REST API.
//! API Documentation: https://porkbun.com/api/json/v3/documentation

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const API_BASE: &str = "https://api.porkbun.com/api/json/v3";

/// Porkbun API client
pub struct PorkbunClient {
    api_key: String,
    secret_api_key: String,
}

/// API credentials structure used in all requests
#[derive(Debug, Serialize)]
struct ApiCredentials {
    apikey: String,
    secretapikey: String,
}

/// Domain list response
#[derive(Debug, Deserialize)]
struct DomainListResponse {
    status: String,
    #[serde(default)]
    domains: Vec<DomainInfo>,
}

/// Domain information
#[derive(Debug, Deserialize)]
struct DomainInfo {
    domain: String,
}

/// DNS records response
#[derive(Debug, Deserialize)]
struct DnsRecordsResponse {
    status: String,
    #[serde(default)]
    records: Vec<DnsRecord>,
}

/// DNS record
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DnsRecord {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub content: String,
    pub ttl: String,
    pub prio: Option<String>,
    pub notes: Option<String>,
}

/// Generic API response
#[derive(Debug, Deserialize)]
struct ApiResponse {
    status: String,
    #[serde(default)]
    message: Option<String>,
}

/// DNS record creation/update request
#[derive(Debug, Serialize)]
struct DnsRecordRequest {
    apikey: String,
    secretapikey: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    name: String,
    #[serde(rename = "type")]
    record_type: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ttl: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prio: Option<u32>,
}

impl PorkbunClient {
    /// Create a new Porkbun client
    pub fn new(api_key: String, secret_api_key: String) -> Self {
        Self {
            api_key,
            secret_api_key,
        }
    }

    /// List all registered domains
    pub fn list_domains(&self) -> Result<Vec<String>> {
        let url = format!("{}/domain/listAll", API_BASE);

        let creds = ApiCredentials {
            apikey: self.api_key.clone(),
            secretapikey: self.secret_api_key.clone(),
        };

        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(&creds)
            .context("Failed to call Porkbun API")?;

        let result: DomainListResponse = response
            .into_json()
            .context("Failed to parse domain list response")?;

        if result.status != "SUCCESS" {
            anyhow::bail!("Porkbun API error: {}", result.status);
        }

        Ok(result.domains.iter().map(|d| d.domain.clone()).collect())
    }

    /// Get all DNS records for a domain
    pub fn get_records(&self, domain: &str) -> Result<Vec<DnsRecord>> {
        let url = format!("{}/dns/retrieve/{}", API_BASE, domain);

        let creds = ApiCredentials {
            apikey: self.api_key.clone(),
            secretapikey: self.secret_api_key.clone(),
        };

        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(&creds)
            .context("Failed to call Porkbun API")?;

        // Get response as text first for debugging
        let body = response
            .into_string()
            .context("Failed to read response body")?;

        eprintln!(
            "DEBUG: Porkbun DNS records response: {}",
            if body.len() > 500 {
                &body[..500]
            } else {
                &body
            }
        );

        // Try to parse the JSON
        let result: DnsRecordsResponse = serde_json::from_str(&body).context(format!(
            "Failed to parse DNS records response. Body: {}",
            if body.len() > 200 {
                &body[..200]
            } else {
                &body
            }
        ))?;

        if result.status != "SUCCESS" {
            anyhow::bail!("Porkbun API error: {}", result.status);
        }

        Ok(result.records)
    }

    /// Create a new DNS record
    pub fn create_record(
        &self,
        domain: &str,
        subdomain: &str,
        record_type: &str,
        content: &str,
        ttl: Option<u32>,
    ) -> Result<()> {
        let url = format!("{}/dns/create/{}", API_BASE, domain);

        eprintln!("DEBUG Porkbun create_record:");
        eprintln!("  URL: {}", url);
        eprintln!("  Domain: {}", domain);
        eprintln!("  Subdomain: {}", subdomain);
        eprintln!("  Type: {}", record_type);
        eprintln!("  Content: {}", content);
        eprintln!("  TTL: {:?}", ttl);

        let request = DnsRecordRequest {
            apikey: self.api_key.clone(),
            secretapikey: self.secret_api_key.clone(),
            name: subdomain.to_string(),
            record_type: record_type.to_uppercase(),
            content: content.to_string(),
            ttl,
            prio: None,
        };

        eprintln!(
            "  Request JSON: {}",
            serde_json::to_string(&request).unwrap_or_default()
        );

        let response = match ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(&request)
        {
            Ok(resp) => resp,
            Err(ureq::Error::Status(code, resp)) => {
                let error_body = resp
                    .into_string()
                    .unwrap_or_else(|_| "Could not read error body".to_string());
                eprintln!("  ❌ HTTP {} error", code);
                eprintln!("  Error response: {}", error_body);
                return Err(anyhow::anyhow!(
                    "Failed to create DNS record: HTTP {} - {}",
                    code,
                    error_body
                ));
            }
            Err(e) => {
                eprintln!("  ❌ HTTP request failed: {:?}", e);
                return Err(anyhow::anyhow!("Failed to create DNS record: {:?}", e));
            }
        };

        let body = response
            .into_string()
            .context("Failed to read response body")?;

        eprintln!("  Response: {}", body);

        let result: ApiResponse =
            serde_json::from_str(&body).context("Failed to parse create response")?;

        if result.status != "SUCCESS" {
            let msg = result.message.unwrap_or_else(|| result.status.clone());
            eprintln!("  ❌ API returned error: {}", msg);
            anyhow::bail!("Failed to create record: {}", msg);
        }

        eprintln!("  ✓ API returned SUCCESS");
        Ok(())
    }

    /// Update DNS record by name and type
    pub fn update_record(
        &self,
        domain: &str,
        subdomain: &str,
        record_type: &str,
        content: &str,
        ttl: Option<u32>,
    ) -> Result<()> {
        let url = format!(
            "{}/dns/editByNameType/{}/{}/{}",
            API_BASE,
            domain,
            record_type.to_uppercase(),
            subdomain
        );

        eprintln!("DEBUG Porkbun update_record:");
        eprintln!("  URL: {}", url);
        eprintln!("  Domain: {}", domain);
        eprintln!("  Subdomain: {}", subdomain);
        eprintln!("  Type: {}", record_type);
        eprintln!("  Content: {}", content);
        eprintln!("  TTL: {:?}", ttl);

        let request = DnsRecordRequest {
            apikey: self.api_key.clone(),
            secretapikey: self.secret_api_key.clone(),
            name: subdomain.to_string(),
            record_type: record_type.to_uppercase(),
            content: content.to_string(),
            ttl,
            prio: None,
        };

        eprintln!(
            "  Request JSON: {}",
            serde_json::to_string(&request).unwrap_or_default()
        );

        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(&request)
            .context("Failed to update DNS record")?;

        let body = response
            .into_string()
            .context("Failed to read response body")?;

        eprintln!("  Response: {}", body);

        let result: ApiResponse =
            serde_json::from_str(&body).context("Failed to parse update response")?;

        if result.status != "SUCCESS" {
            let msg = result.message.unwrap_or_else(|| result.status.clone());
            eprintln!("  ❌ API returned error: {}", msg);
            anyhow::bail!("Failed to update record: {}", msg);
        }

        eprintln!("  ✓ API returned SUCCESS");
        Ok(())
    }

    /// Delete DNS record by name and type
    pub fn delete_record(&self, domain: &str, subdomain: &str, record_type: &str) -> Result<()> {
        let url = format!(
            "{}/dns/deleteByNameType/{}/{}/{}",
            API_BASE,
            domain,
            record_type.to_uppercase(),
            subdomain
        );

        let creds = ApiCredentials {
            apikey: self.api_key.clone(),
            secretapikey: self.secret_api_key.clone(),
        };

        let response = ureq::post(&url)
            .set("Content-Type", "application/json")
            .send_json(&creds)
            .context("Failed to delete DNS record")?;

        let result: ApiResponse = response
            .into_json()
            .context("Failed to parse delete response")?;

        if result.status != "SUCCESS" {
            let msg = result.message.unwrap_or_else(|| result.status.clone());
            anyhow::bail!("Failed to delete record: {}", msg);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = PorkbunClient::new("pk1_test".to_string(), "sk1_test".to_string());
        assert_eq!(client.api_key, "pk1_test");
        assert_eq!(client.secret_api_key, "sk1_test");
    }
}
