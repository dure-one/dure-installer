//! DuckDNS API implementation
//!
//! DuckDNS is a free dynamic DNS service that provides a simple HTTP API
//! for updating DNS records. Domains cannot be registered via API - they must
//! be created through the DuckDNS web interface at https://www.duckdns.org
//!
//! API Documentation: https://www.duckdns.org/spec.jsp

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// DuckDNS API client
pub struct DuckDnsClient {
    token: String,
}

/// DuckDNS API response
#[derive(Debug, Deserialize, Serialize)]
struct DuckDnsResponse {
    #[serde(rename = "OK")]
    ok: bool,
    #[serde(default)]
    text: Option<String>,
}

impl DuckDnsClient {
    /// Create a new DuckDNS client with the given API token
    pub fn new(token: String) -> Self {
        Self { token }
    }

    /// Update DNS records for a domain
    ///
    /// # Arguments
    /// * `domain` - The subdomain (without .duckdns.org suffix)
    /// * `ipv4` - Optional IPv4 address for A record
    /// * `ipv6` - Optional IPv6 address for AAAA record
    /// * `txt` - Optional TXT record value
    pub fn update(
        &self,
        domain: &str,
        ipv4: Option<&str>,
        ipv6: Option<&str>,
        txt: Option<&str>,
    ) -> Result<()> {
        // Extract subdomain from full domain if needed
        let subdomain = domain.strip_suffix(".duckdns.org").unwrap_or(domain);

        let mut url = format!(
            "https://www.duckdns.org/update?domains={}&token={}",
            subdomain, self.token
        );

        if let Some(ip) = ipv4 {
            url.push_str(&format!("&ip={}", ip));
        }

        if let Some(ip) = ipv6 {
            url.push_str(&format!("&ipv6={}", ip));
        }

        if let Some(txt_val) = txt {
            url.push_str(&format!("&txt={}", urlencoding::encode(txt_val)));
        }

        // Make HTTP request
        let response = ureq::get(&url)
            .call()
            .context("Failed to call DuckDNS API")?;

        let body = response
            .into_string()
            .context("Failed to read DuckDNS response")?;

        // DuckDNS returns "OK" or "KO" as plain text
        if body.trim() == "OK" {
            Ok(())
        } else {
            anyhow::bail!("DuckDNS update failed: {}", body)
        }
    }

    /// Update A record (IPv4)
    pub fn update_a_record(&self, domain: &str, ipv4: &str) -> Result<()> {
        self.update(domain, Some(ipv4), None, None)
    }

    /// Update AAAA record (IPv6)
    pub fn update_aaaa_record(&self, domain: &str, ipv6: &str) -> Result<()> {
        self.update(domain, None, Some(ipv6), None)
    }

    /// Update TXT record
    pub fn update_txt_record(&self, domain: &str, txt: &str) -> Result<()> {
        self.update(domain, None, None, Some(txt))
    }

    /// Clear TXT record
    pub fn clear_txt_record(&self, domain: &str) -> Result<()> {
        self.update(domain, None, None, Some(""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = DuckDnsClient::new("test-token".to_string());
        assert_eq!(client.token, "test-token");
    }

    #[test]
    fn test_domain_strip_suffix() {
        let domain = "example.duckdns.org";
        let subdomain = domain.strip_suffix(".duckdns.org").unwrap();
        assert_eq!(subdomain, "example");
    }
}
