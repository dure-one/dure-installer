//! DNS resolution functionality with caching
//!
//! Provides DNS-over-HTTPS (DOH) resolution for A, AAAA, and TXT records
//! with SQLite-based caching.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// DNS record type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordType {
    A,
    AAAA,
    TXT,
    NS,
}

impl RecordType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecordType::A => "A",
            RecordType::AAAA => "AAAA",
            RecordType::TXT => "TXT",
            RecordType::NS => "NS",
        }
    }

    pub fn dns_type(&self) -> u16 {
        match self {
            RecordType::A => 1,
            RecordType::AAAA => 28,
            RecordType::TXT => 16,
            RecordType::NS => 2,
        }
    }
}

impl std::fmt::Display for RecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// DNS record result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub domain: String,
    pub record_type: RecordType,
    pub value: String,
    pub ttl: u32,
    pub timestamp: u64,
}

impl DnsRecord {
    pub fn new(domain: String, record_type: RecordType, value: String, ttl: u32) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        Self {
            domain,
            record_type,
            value,
            ttl,
            timestamp,
        }
    }

    /// Check if this record is still valid based on TTL
    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();

        now < self.timestamp + u64::from(self.ttl)
    }
}

/// Resolve DNS records using DNS-over-HTTPS
#[cfg(not(target_arch = "wasm32"))]
pub fn resolve_dns(domain: &str, record_type: RecordType) -> Result<Vec<DnsRecord>> {
    // Use Cloudflare's DOH service
    let url = format!(
        "https://cloudflare-dns.com/dns-query?name={}&type={}",
        domain,
        record_type.as_str()
    );

    #[derive(Deserialize)]
    struct DohResponse {
        #[serde(rename = "Answer")]
        answer: Option<Vec<DohAnswer>>,
    }

    #[derive(Deserialize)]
    struct DohAnswer {
        data: String,
        #[serde(rename = "TTL")]
        ttl: u32,
    }

    // Make HTTP request
    let response_text = ureq::get(&url)
        .set("Accept", "application/dns-json")
        .call()
        .context("Failed to query DNS-over-HTTPS")?
        .into_string()
        .context("Failed to read response")?;

    let doh_response: DohResponse =
        serde_json::from_str(&response_text).context("Failed to parse DOH response")?;

    let records = doh_response
        .answer
        .unwrap_or_default()
        .into_iter()
        .map(|answer| DnsRecord::new(domain.to_string(), record_type, answer.data, answer.ttl))
        .collect();

    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_type_display() {
        assert_eq!(RecordType::A.to_string(), "A");
        assert_eq!(RecordType::AAAA.to_string(), "AAAA");
        assert_eq!(RecordType::TXT.to_string(), "TXT");
    }

    #[test]
    fn test_dns_record_validity() {
        let record = DnsRecord::new(
            "example.com".to_string(),
            RecordType::A,
            "1.2.3.4".to_string(),
            300,
        );
        assert!(record.is_valid());
    }

    #[test]
    fn test_expired_dns_record() {
        let mut record = DnsRecord::new(
            "example.com".to_string(),
            RecordType::A,
            "1.2.3.4".to_string(),
            1,
        );
        record.timestamp = 0; // Set to epoch
        assert!(!record.is_valid());
    }
}
