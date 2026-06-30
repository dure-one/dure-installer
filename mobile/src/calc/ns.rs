//! NS (nameserver) management controller
//!
//! Provides domain and DNS record management functionality for multiple providers:
//! - Cloudflare
//! - Google Cloud DNS
//! - DuckDNS
//! - Porkbun
//!
//! Supports A, AAAA, TXT, and NS record types.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use crate::calc::acme::{
    DnsProvider, DnsProviderType, set_a_record, set_aaaa_record, set_txt_record,
};

/// DNS record type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordType {
    A,
    AAAA,
    TXT,
    NS,
}

impl RecordType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RecordType::A => "a",
            RecordType::AAAA => "aaaa",
            RecordType::TXT => "txt",
            RecordType::NS => "ns",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "a" => Some(RecordType::A),
            "aaaa" => Some(RecordType::AAAA),
            "txt" => Some(RecordType::TXT),
            "ns" => Some(RecordType::NS),
            _ => None,
        }
    }
}

/// DNS record entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub record_type: RecordType,
    #[serde(default)]
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,
}

/// Domain configuration with DNS records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainEntry {
    pub domain: String,
    #[serde(default)]
    pub records: Vec<DnsRecord>,
}

/// Provider configuration with API token and domains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub api_token: String,
    #[serde(default)]
    pub domains: Vec<DomainEntry>,
}

/// GCP account configuration with OAuth credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcpAccount {
    pub access_token: String,
    pub refresh_token: String,
    pub token_expiry: u64,
    pub connected_email: String,
    pub project_id: String,
    #[serde(default)]
    pub domains: Vec<DomainEntry>,
}

/// NS configuration stored in config.yml
#[derive(Debug, Clone, Default)]
pub struct NsConfig {
    pub providers: HashMap<String, ProviderConfig>,
    pub gcp_accounts: Vec<GcpAccount>,
}

impl serde::Serialize for NsConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        let mut root = serializer.serialize_map(Some(1))?;

        // Build providers map
        let mut providers_map: HashMap<String, serde_yaml::Value> = HashMap::new();

        // Add regular providers
        for (name, config) in &self.providers {
            let config_value = serde_yaml::to_value(config).map_err(serde::ser::Error::custom)?;
            providers_map.insert(name.clone(), config_value);
        }

        // Add GCP accounts as array
        if !self.gcp_accounts.is_empty() {
            let gcp_array: Vec<serde_yaml::Value> = self
                .gcp_accounts
                .iter()
                .map(|account| serde_yaml::to_value(account).unwrap_or(serde_yaml::Value::Null))
                .collect();
            providers_map.insert("gcloud".to_string(), serde_yaml::Value::Sequence(gcp_array));
        }

        root.serialize_entry("providers", &providers_map)?;
        root.end()
    }
}

impl<'de> serde::Deserialize<'de> for NsConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            #[serde(default)]
            providers: HashMap<String, serde_yaml::Value>,
        }

        let helper = Helper::deserialize(deserializer)?;
        let mut providers = HashMap::new();
        let mut gcp_accounts = Vec::new();

        for (key, val) in helper.providers {
            if key == "gcloud" {
                // Handle gcloud as array of accounts
                if let serde_yaml::Value::Sequence(accounts) = val {
                    for account_val in accounts {
                        if let Ok(account) = serde_yaml::from_value::<GcpAccount>(account_val) {
                            gcp_accounts.push(account);
                        }
                    }
                }
            } else {
                // Handle other providers normally
                if let Ok(config) = serde_yaml::from_value::<ProviderConfig>(val) {
                    providers.insert(key, config);
                }
            }
        }

        Ok(NsConfig {
            providers,
            gcp_accounts,
        })
    }
}

impl NsConfig {
    /// Load from YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self> {
        let value: serde_yaml::Value =
            serde_yaml::from_str(yaml).context("Failed to parse NS config YAML")?;

        let mut providers = HashMap::new();
        let mut gcp_accounts = Vec::new();

        if let serde_yaml::Value::Mapping(ref map) = value {
            if let Some(serde_yaml::Value::Mapping(providers_map)) =
                map.get(&serde_yaml::Value::String("providers".to_string()))
            {
                for (key, val) in providers_map {
                    if let serde_yaml::Value::String(provider_name) = key {
                        if provider_name == "gcloud" {
                            // Handle gcloud as array of accounts
                            if let serde_yaml::Value::Sequence(accounts) = val {
                                for account_val in accounts {
                                    if let Ok(account) =
                                        serde_yaml::from_value::<GcpAccount>(account_val.clone())
                                    {
                                        gcp_accounts.push(account);
                                    }
                                }
                            }
                        } else {
                            // Handle other providers normally
                            if let Ok(config) =
                                serde_yaml::from_value::<ProviderConfig>(val.clone())
                            {
                                providers.insert(provider_name.clone(), config);
                            }
                        }
                    }
                }
            }
        }

        Ok(NsConfig {
            providers,
            gcp_accounts,
        })
    }

    /// Save to YAML string
    pub fn to_yaml(&self) -> Result<String> {
        let mut providers_map = serde_yaml::Mapping::new();

        // Add regular providers
        for (name, config) in &self.providers {
            let config_value =
                serde_yaml::to_value(config).context("Failed to serialize provider config")?;
            providers_map.insert(serde_yaml::Value::String(name.clone()), config_value);
        }

        // Add GCP accounts as array
        if !self.gcp_accounts.is_empty() {
            let gcp_array: Vec<serde_yaml::Value> = self
                .gcp_accounts
                .iter()
                .map(|account| serde_yaml::to_value(account).unwrap_or(serde_yaml::Value::Null))
                .collect();
            providers_map.insert(
                serde_yaml::Value::String("gcloud".to_string()),
                serde_yaml::Value::Sequence(gcp_array),
            );
        }

        let mut root = serde_yaml::Mapping::new();
        root.insert(
            serde_yaml::Value::String("providers".to_string()),
            serde_yaml::Value::Mapping(providers_map),
        );

        serde_yaml::to_string(&serde_yaml::Value::Mapping(root))
            .context("Failed to serialize NS config")
    }

    /// Parse GCP provider identifier (e.g., "gcloud:email@example.com" -> "email@example.com")
    fn parse_gcp_identifier(provider: &str) -> Option<&str> {
        if provider.starts_with("gcloud:") {
            Some(&provider[7..])
        } else {
            None
        }
    }

    /// Get domain entry by provider and name
    pub fn get_domain(&self, provider: &str, domain: &str) -> Option<&DomainEntry> {
        // Check if it's a GCP provider
        if let Some(email) = Self::parse_gcp_identifier(provider) {
            return self
                .gcp_accounts
                .iter()
                .find(|acc| acc.connected_email == email)?
                .domains
                .iter()
                .find(|d| d.domain == domain);
        }

        // Regular provider
        self.providers
            .get(provider)?
            .domains
            .iter()
            .find(|d| d.domain == domain)
    }

    /// Get mutable domain entry by provider and name
    pub fn get_domain_mut(&mut self, provider: &str, domain: &str) -> Option<&mut DomainEntry> {
        // Check if it's a GCP provider
        if let Some(email) = Self::parse_gcp_identifier(provider) {
            return self
                .gcp_accounts
                .iter_mut()
                .find(|acc| acc.connected_email == email)?
                .domains
                .iter_mut()
                .find(|d| d.domain == domain);
        }

        // Regular provider
        self.providers
            .get_mut(provider)?
            .domains
            .iter_mut()
            .find(|d| d.domain == domain)
    }

    /// Add a new domain to a provider
    pub fn add_domain(
        &mut self,
        provider: String,
        domain: String,
        api_token: String,
    ) -> Result<()> {
        // Check if it's a GCP provider
        if let Some(email) = Self::parse_gcp_identifier(&provider) {
            let account = self
                .gcp_accounts
                .iter_mut()
                .find(|acc| acc.connected_email == email)
                .ok_or_else(|| anyhow::anyhow!("GCP account {} not found", email))?;

            // Check for duplicate within the same account
            if account.domains.iter().any(|d| d.domain == domain) {
                anyhow::bail!("Domain {} already exists for GCP account {}", domain, email);
            }

            account.domains.push(DomainEntry {
                domain,
                records: Vec::new(),
            });

            return Ok(());
        }

        // Regular provider
        let provider_config =
            self.providers
                .entry(provider.clone())
                .or_insert_with(|| ProviderConfig {
                    api_token: api_token.clone(),
                    domains: Vec::new(),
                });

        // Update api_token if provided
        if !api_token.is_empty() {
            provider_config.api_token = api_token;
        }

        // Check for duplicate within the same provider
        if provider_config.domains.iter().any(|d| d.domain == domain) {
            anyhow::bail!("Domain {} already exists for provider {}", domain, provider);
        }

        provider_config.domains.push(DomainEntry {
            domain,
            records: Vec::new(),
        });

        Ok(())
    }

    /// Remove a domain from a provider
    pub fn remove_domain(&mut self, provider: &str, domain: &str) -> Result<()> {
        // Check if it's a GCP provider
        if let Some(email) = Self::parse_gcp_identifier(provider) {
            let account = self
                .gcp_accounts
                .iter_mut()
                .find(|acc| acc.connected_email == email)
                .ok_or_else(|| anyhow::anyhow!("GCP account {} not found", email))?;

            let index = account
                .domains
                .iter()
                .position(|d| d.domain == domain)
                .ok_or_else(|| {
                    anyhow::anyhow!("Domain {} not found in GCP account {}", domain, email)
                })?;

            account.domains.remove(index);
            return Ok(());
        }

        // Regular provider
        let provider_config = self
            .providers
            .get_mut(provider)
            .ok_or_else(|| anyhow::anyhow!("Provider {} not found", provider))?;

        let index = provider_config
            .domains
            .iter()
            .position(|d| d.domain == domain)
            .ok_or_else(|| {
                anyhow::anyhow!("Domain {} not found in provider {}", domain, provider)
            })?;

        provider_config.domains.remove(index);

        // Remove provider if no domains left
        if provider_config.domains.is_empty() {
            self.providers.remove(provider);
        }

        Ok(())
    }

    /// Add a record to a domain
    pub fn add_record(
        &mut self,
        provider: &str,
        domain: &str,
        record_type: RecordType,
        name: String,
        value: String,
    ) -> Result<()> {
        let domain_entry = self.get_domain_mut(provider, domain).ok_or_else(|| {
            anyhow::anyhow!("Domain {} not found in provider {}", domain, provider)
        })?;

        // Check if record already exists
        if domain_entry
            .records
            .iter()
            .any(|r| r.record_type == record_type && r.name == name && r.value == value)
        {
            anyhow::bail!(
                "Record already exists: {} {} {} {}",
                domain,
                name,
                record_type.as_str(),
                value
            );
        }

        domain_entry.records.push(DnsRecord {
            record_type,
            name,
            value,
            ttl: None,
        });

        Ok(())
    }

    /// Remove a record from a domain
    pub fn remove_record(
        &mut self,
        provider: &str,
        domain: &str,
        record_type: RecordType,
        value: &str,
    ) -> Result<()> {
        let domain_entry = self.get_domain_mut(provider, domain).ok_or_else(|| {
            anyhow::anyhow!("Domain {} not found in provider {}", domain, provider)
        })?;

        let index = domain_entry
            .records
            .iter()
            .position(|r| r.record_type == record_type && r.value == value)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Record not found: {} {} {}",
                    domain,
                    record_type.as_str(),
                    value
                )
            })?;

        domain_entry.records.remove(index);
        Ok(())
    }

    /// List all domains across all providers
    pub fn list_domains(&self) -> Vec<(String, String)> {
        let mut domains = Vec::new();

        // Add regular providers
        for (provider, config) in &self.providers {
            for domain_entry in &config.domains {
                domains.push((provider.clone(), domain_entry.domain.clone()));
            }
        }

        // Add GCP accounts
        for account in &self.gcp_accounts {
            let provider_id = format!("gcloud:{}", account.connected_email);
            for domain_entry in &account.domains {
                domains.push((provider_id.clone(), domain_entry.domain.clone()));
            }
        }

        domains
    }

    /// Get all records for a domain in a provider
    pub fn get_records(&self, provider: &str, domain: &str) -> Option<Vec<DnsRecord>> {
        self.get_domain(provider, domain).map(|d| d.records.clone())
    }

    /// Count total domains across all providers
    pub fn total_domains(&self) -> usize {
        let regular_count: usize = self.providers.values().map(|p| p.domains.len()).sum();
        let gcp_count: usize = self.gcp_accounts.iter().map(|acc| acc.domains.len()).sum();
        regular_count + gcp_count
    }

    /// Get list of all provider names
    pub fn provider_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.providers.keys().cloned().collect();

        // Add GCP accounts
        for account in &self.gcp_accounts {
            names.push(format!("gcloud:{}", account.connected_email));
        }

        names
    }

    /// Get all (provider, domain_entry) pairs
    pub fn iter_all_domains(&self) -> Vec<(String, &DomainEntry)> {
        let mut result = Vec::new();

        // Add regular providers
        for (provider, config) in &self.providers {
            for domain_entry in &config.domains {
                result.push((provider.clone(), domain_entry));
            }
        }

        // Add GCP accounts
        for account in &self.gcp_accounts {
            let provider_id = format!("gcloud:{}", account.connected_email);
            for domain_entry in &account.domains {
                result.push((provider_id.clone(), domain_entry));
            }
        }

        result
    }

    /// Get API token for a provider (for GCP, returns "access_token::project_id")
    pub fn get_api_token(&self, provider: &str) -> Option<String> {
        // Check if it's a GCP provider
        if let Some(email) = Self::parse_gcp_identifier(provider) {
            let account = self
                .gcp_accounts
                .iter()
                .find(|acc| acc.connected_email == email)?;
            return Some(format!("{}::{}", account.access_token, account.project_id));
        }

        // Regular provider
        self.providers.get(provider).map(|p| p.api_token.clone())
    }

    /// Get API token with automatic refresh for GCP accounts (mutable version)
    pub fn get_api_token_refreshed(&mut self, provider: &str) -> Option<String> {
        // Check if it's a GCP provider
        if let Some(email) = Self::parse_gcp_identifier(provider) {
            let account = self
                .gcp_accounts
                .iter_mut()
                .find(|acc| acc.connected_email == email)?;

            // Check if token is expired or will expire soon (within 5 minutes)
            let now = chrono::Utc::now().timestamp() as u64;
            let expires_soon = account.token_expiry < now + 300; // 5 minutes buffer

            if expires_soon {
                // Refresh the token using embedded OAuth credentials
                // Note: Using OAuthHandler constants which are compiled into binary
                let handler = crate::api::gcp_oauth::OAuthHandler::default();
                match crate::api::gcp_oauth::refresh_access_token(
                    &handler.client_id(),
                    &handler.client_secret(),
                    &account.refresh_token,
                ) {
                    Ok(oauth_result) => {
                        account.access_token = oauth_result.access_token;
                        account.token_expiry = oauth_result.expires_at;
                        account.refresh_token = oauth_result.refresh_token;
                    }
                    Err(e) => {
                        eprintln!("Failed to refresh GCP token for {}: {}", email, e);
                        // Continue with old token, might still work
                    }
                }
            }

            return Some(format!("{}::{}", account.access_token, account.project_id));
        }

        // Regular provider
        self.providers.get(provider).map(|p| p.api_token.clone())
    }

    /// Find which provider(s) have a given domain
    pub fn find_providers_for_domain(&self, domain: &str) -> Vec<String> {
        let mut providers = Vec::new();

        // Check regular providers
        for (provider_name, provider_config) in &self.providers {
            if provider_config.domains.iter().any(|d| d.domain == domain) {
                providers.push(provider_name.clone());
            }
        }

        // Check GCP accounts
        for account in &self.gcp_accounts {
            if account.domains.iter().any(|d| d.domain == domain) {
                providers.push(format!("gcloud:{}", account.connected_email));
            }
        }

        providers
    }

    /// Get domain entry by domain name only (searches all providers, returns first match)
    pub fn get_domain_any_provider(&self, domain: &str) -> Option<(String, &DomainEntry)> {
        // Check regular providers
        for (provider_name, provider_config) in &self.providers {
            if let Some(domain_entry) = provider_config.domains.iter().find(|d| d.domain == domain)
            {
                return Some((provider_name.clone(), domain_entry));
            }
        }

        // Check GCP accounts
        for account in &self.gcp_accounts {
            if let Some(domain_entry) = account.domains.iter().find(|d| d.domain == domain) {
                return Some((format!("gcloud:{}", account.connected_email), domain_entry));
            }
        }

        None
    }

    /// Add a new GCP account
    pub fn add_gcp_account(&mut self, account: GcpAccount) -> Result<()> {
        // Check for duplicate email
        if self
            .gcp_accounts
            .iter()
            .any(|acc| acc.connected_email == account.connected_email)
        {
            anyhow::bail!("GCP account {} already exists", account.connected_email);
        }

        self.gcp_accounts.push(account);
        Ok(())
    }

    /// Get GCP account by email
    pub fn get_gcp_account(&self, email: &str) -> Option<&GcpAccount> {
        self.gcp_accounts
            .iter()
            .find(|acc| acc.connected_email == email)
    }

    /// Get mutable GCP account by email
    pub fn get_gcp_account_mut(&mut self, email: &str) -> Option<&mut GcpAccount> {
        self.gcp_accounts
            .iter_mut()
            .find(|acc| acc.connected_email == email)
    }
}

/// Apply DNS record to actual DNS provider
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub fn apply_record(
    provider_name: &str,
    api_token: &str,
    domain: &str,
    record: &DnsRecord,
) -> Result<()> {
    let provider_type = if provider_name.starts_with("gcloud:") {
        // GCP provider with email format: "gcloud:email"
        DnsProviderType::GoogleCloud
    } else {
        match provider_name.to_lowercase().as_str() {
            "cloudflare" | "cf" => DnsProviderType::Cloudflare,
            "gcloud" | "googlecloud" | "gcp" => DnsProviderType::GoogleCloud,
            "duckdns" => DnsProviderType::DuckDNS,
            "porkbun" => DnsProviderType::Porkbun,
            _ => anyhow::bail!("Invalid DNS provider: {}", provider_name),
        }
    };

    let provider = DnsProvider {
        provider_type,
        api_token: api_token.to_string(),
    };

    match record.record_type {
        RecordType::A => set_a_record(&provider, domain, &record.name, &record.value)?,
        RecordType::TXT => set_txt_record(&provider, domain, &record.name, &record.value)?,
        RecordType::AAAA => set_aaaa_record(&provider, domain, &record.name, &record.value)?,
        RecordType::NS => {
            anyhow::bail!("NS record type not yet supported for DNS provider operations")
        }
    }

    Ok(())
}

/// Apply all records for a domain to the DNS provider
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub fn apply_all_records(
    provider_name: &str,
    api_token: &str,
    domain_entry: &DomainEntry,
) -> Result<Vec<(RecordType, String, Result<()>)>> {
    let mut results = Vec::new();

    for record in &domain_entry.records {
        let result = apply_record(provider_name, api_token, &domain_entry.domain, record);
        results.push((record.record_type.clone(), record.value.clone(), result));
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_type_conversion() {
        assert_eq!(RecordType::from_str("a"), Some(RecordType::A));
        assert_eq!(RecordType::from_str("A"), Some(RecordType::A));
        assert_eq!(RecordType::from_str("aaaa"), Some(RecordType::AAAA));
        assert_eq!(RecordType::from_str("txt"), Some(RecordType::TXT));
        assert_eq!(RecordType::from_str("invalid"), None);
    }

    #[test]
    fn test_ns_config_add_domain() {
        let mut config = NsConfig::default();
        config
            .add_domain(
                "cloudflare".to_string(),
                "example.com".to_string(),
                "token123".to_string(),
            )
            .unwrap();

        assert_eq!(config.providers.get("cloudflare").unwrap().domains.len(), 1);
        assert_eq!(
            config.providers.get("cloudflare").unwrap().domains[0].domain,
            "example.com"
        );
    }

    #[test]
    fn test_ns_config_add_record() {
        let mut config = NsConfig::default();
        config
            .add_domain(
                "cloudflare".to_string(),
                "example.com".to_string(),
                "token123".to_string(),
            )
            .unwrap();
        config
            .add_record(
                "cloudflare",
                "example.com",
                RecordType::A,
                "@".to_string(),
                "1.2.3.4".to_string(),
            )
            .unwrap();

        let domain = config.get_domain("cloudflare", "example.com").unwrap();
        assert_eq!(domain.records.len(), 1);
        assert_eq!(domain.records[0].record_type, RecordType::A);
        assert_eq!(domain.records[0].name, "@");
        assert_eq!(domain.records[0].value, "1.2.3.4");
    }

    #[test]
    fn test_ns_config_yaml_serialization() {
        let mut config = NsConfig::default();
        config
            .add_domain(
                "cloudflare".to_string(),
                "example.com".to_string(),
                "token123".to_string(),
            )
            .unwrap();
        config
            .add_record(
                "cloudflare",
                "example.com",
                RecordType::A,
                "@".to_string(),
                "1.2.3.4".to_string(),
            )
            .unwrap();

        let yaml = config.to_yaml().unwrap();
        assert!(yaml.contains("example.com"));
        assert!(yaml.contains("cloudflare"));

        let loaded = NsConfig::from_yaml(&yaml).unwrap();
        assert_eq!(loaded.providers.get("cloudflare").unwrap().domains.len(), 1);
        assert_eq!(
            loaded.providers.get("cloudflare").unwrap().domains[0].domain,
            "example.com"
        );
    }
}
