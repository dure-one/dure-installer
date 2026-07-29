//! NS (nameserver) command implementation for DNS record management

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use crate::calc::audit;
use crate::calc::ns::{NsConfig, RecordType};
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use crate::calc::ns::apply_record;

/// Get the path to config.yml
fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = ProjectDirs::from("com", "dure", "dure")
        .ok_or_else(|| anyhow::anyhow!("Failed to determine config directory"))?;
    Ok(proj_dirs.config_dir().join("config.yml"))
}

/// Load NS config from YAML file
fn load_ns_config() -> Result<NsConfig> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        // Return empty config if file doesn't exist
        return Ok(NsConfig::default());
    }

    let yaml = std::fs::read_to_string(&config_path).context("Failed to read config.yml")?;

    // Try to parse as full config with ns section
    #[derive(serde::Deserialize)]
    struct FullConfig {
        #[serde(default)]
        ns: NsConfig,
    }

    match serde_yaml::from_str::<FullConfig>(&yaml) {
        Ok(full_config) => Ok(full_config.ns),
        Err(_) => {
            // If that fails, try to parse just the ns section
            Ok(NsConfig::default())
        }
    }
}

/// Save NS config to YAML file
fn save_ns_config(ns_config: &NsConfig) -> Result<()> {
    let config_path = get_config_path()?;

    // Ensure config directory exists
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create config directory")?;
    }

    // Load existing config or create new one
    let mut full_config = if config_path.exists() {
        let yaml = std::fs::read_to_string(&config_path).context("Failed to read config.yml")?;
        serde_yaml::from_str::<serde_yaml::Value>(&yaml)
            .unwrap_or_else(|_| serde_yaml::Value::Mapping(Default::default()))
    } else {
        serde_yaml::Value::Mapping(Default::default())
    };

    // Update the ns section
    let ns_value = serde_yaml::to_value(ns_config).context("Failed to serialize NS config")?;

    if let serde_yaml::Value::Mapping(ref mut map) = full_config {
        map.insert(serde_yaml::Value::String("ns".to_string()), ns_value);
    }

    // Write back to file
    let yaml = serde_yaml::to_string(&full_config).context("Failed to serialize config")?;
    std::fs::write(&config_path, yaml).context("Failed to write config.yml")?;

    Ok(())
}

/// Execute NS status command
///
/// Lists all registered domains and their records
pub fn execute_ns_status(domain: &Option<String>) -> Result<()> {
    let config = load_ns_config()?;

    if config.total_domains() == 0 {
        dure_info!("No domains registered.");
        dure_info!("");
        dure_info!("Add a domain with:");
        dure_info!("  dure ns add www.example.com --provider cloudflare --token YOUR_TOKEN");
        return Ok(());
    }

    if let Some(domain_name) = domain {
        // Show specific domain
        let (provider_name, domain_entry) = config
            .get_domain_any_provider(domain_name)
            .ok_or_else(|| anyhow::anyhow!("Domain {} not found", domain_name))?;

        let api_token = config
            .get_api_token(&provider_name)
            .ok_or_else(|| anyhow::anyhow!("API token not found for provider {}", provider_name))?;

        dure_info!("Domain: {}", domain_entry.domain);
        dure_info!("Provider: {}", provider_name);
        dure_info!("API Token: {}...", &api_token.chars().take(8).collect::<String>()
        );
        dure_info!("");
        dure_info!("Records:");

        if domain_entry.records.is_empty() {
            dure_info!("  (no records)");
        } else {
            for record in &domain_entry.records {
                dure_info!("  {} -> {}", record.record_type.as_str().to_uppercase(),
                    record.value
                );
            }
        }
    } else {
        // Show all domains
        dure_info!("Registered Domains:");
        dure_info!("");

        for (provider_name, domain_entry) in config.iter_all_domains() {
            dure_info!("• {} ({})", domain_entry.domain, provider_name);

            if domain_entry.records.is_empty() {
                dure_info!("  (no records)");
            } else {
                for record in &domain_entry.records {
                    dure_info!("  {} -> {}", record.record_type.as_str().to_uppercase(),
                        record.value
                    );
                }
            }
            dure_info!("");
        }

        dure_info!("Use 'dure ns status DOMAIN' to see details for a specific domain");
    }

    Ok(())
}

/// Execute NS add command
///
/// Adds a new domain to nameserver configuration
pub fn execute_ns_add(domain: &str, provider: &str, token: &str) -> Result<()> {
    let mut config = load_ns_config()?;

    // Validate provider
    let valid_providers = [
        "cloudflare",
        "cf",
        "gcloud",
        "googlecloud",
        "gcp",
        "duckdns",
        "porkbun",
    ];
    if !valid_providers
        .iter()
        .any(|&p| p.eq_ignore_ascii_case(provider))
    {
        anyhow::bail!(
            "Invalid provider '{}'. Valid providers: cloudflare, gcloud, duckdns, porkbun",
            provider
        );
    }

    // Normalize provider name
    let provider_lower = provider.to_lowercase();
    let normalized_provider = match provider_lower.as_str() {
        "cf" => "cloudflare".to_string(),
        "gcp" | "googlecloud" => "gcloud".to_string(),
        other => other.to_string(),
    };

    let provider_display = normalized_provider.clone();
    config.add_domain(normalized_provider, domain.to_string(), token.to_string())?;
    save_ns_config(&config)?;

    // Record audit event
    let _ = audit::push_cli("system", "cli", "ns add", domain);

    dure_info!("✓ Domain '{}' added with provider '{}'", domain, provider_display
    );
    dure_info!("");
    dure_info!("Add DNS records with:");
    dure_info!("  dure ns insert a {} 1.2.3.4", domain);
    dure_info!("  dure ns insert txt {} 'durepubkey=...'", domain);

    Ok(())
}

/// Execute NS del command
///
/// Deletes a domain from nameserver configuration
pub fn execute_ns_del(domain: &str) -> Result<()> {
    let mut config = load_ns_config()?;

    // Find which provider(s) have this domain
    let providers = config.find_providers_for_domain(domain);

    if providers.is_empty() {
        anyhow::bail!("Domain '{}' not found", domain);
    }

    if providers.len() > 1 {
        anyhow::bail!(
            "Domain '{}' exists in multiple providers: {}. Please specify provider.",
            domain,
            providers.join(", ")
        );
    }

    let provider = &providers[0];
    config.remove_domain(provider, domain)?;
    save_ns_config(&config)?;

    // Record audit event
    let _ = audit::push_cli("system", "cli", "ns del", domain);

    dure_info!(" Domain '{}' removed from provider '{}'", domain, provider);

    Ok(())
}

/// Execute NS insert command
///
/// Inserts a DNS record to a domain
pub fn execute_ns_insert(record_type: &str, domain: &str, value: &str, apply: bool) -> Result<()> {
    let mut config = load_ns_config()?;

    // Parse record type
    let rec_type = RecordType::from_str(record_type).ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid record type '{}'. Use: a, aaaa, txt, ns",
            record_type
        )
    })?;

    // Find which provider(s) have this domain
    let providers = config.find_providers_for_domain(domain);

    if providers.is_empty() {
        anyhow::bail!(
            "Domain '{}' not found. Add it first with 'dure ns add'",
            domain
        );
    }

    if providers.len() > 1 {
        anyhow::bail!(
            "Domain '{}' exists in multiple providers: {}. Please specify provider.",
            domain,
            providers.join(", ")
        );
    }

    let provider = &providers[0];

    // Add record to config (using "@" for root domain)
    config.add_record(
        provider,
        domain,
        rec_type.clone(),
        "@".to_string(),
        value.to_string(),
    )?;
    save_ns_config(&config)?;

    // Record audit event
    let record_desc = format!("{} @ {} {}", domain, record_type, value);
    let _ = audit::push_cli("system", "cli", "ns insert", &record_desc);

    dure_info!("✓ Record added: {} {} -> {}", domain,
        record_type.to_uppercase(),
        value
    );

    // Apply to DNS provider if requested
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    if apply {
        dure_info!("");
        dure_info!("Applying to DNS provider...");

        let domain_entry = config.get_domain(provider, domain).unwrap();
        let api_token = config.get_api_token(provider).unwrap();
        let record = domain_entry
            .records
            .iter()
            .find(|r| r.record_type == rec_type && r.value == value)
            .unwrap();

        match apply_record(provider, &api_token, domain, record) {
            Ok(_) => {
                dure_info!(" Record applied to DNS provider");
            }
            Err(e) => {
                dure_warn!(" Failed to apply to DNS provider: {}", e);
                dure_info!("  Record is saved in config but not applied to provider");
            }
        }
    }

    #[cfg(any(target_os = "android", target_arch = "wasm32"))]
    if apply {
        dure_warn!(" DNS provider apply not supported on this platform");
    }

    Ok(())
}

/// Execute NS remove command
///
/// Removes a DNS record from a domain
pub fn execute_ns_remove(record_type: &str, domain: &str, value: &str) -> Result<()> {
    let mut config = load_ns_config()?;

    // Parse record type
    let rec_type = RecordType::from_str(record_type).ok_or_else(|| {
        anyhow::anyhow!(
            "Invalid record type '{}'. Use: a, aaaa, txt, ns",
            record_type
        )
    })?;

    // Find which provider(s) have this domain
    let providers = config.find_providers_for_domain(domain);

    if providers.is_empty() {
        anyhow::bail!("Domain '{}' not found", domain);
    }

    if providers.len() > 1 {
        anyhow::bail!(
            "Domain '{}' exists in multiple providers: {}. Please specify provider.",
            domain,
            providers.join(", ")
        );
    }

    let provider = &providers[0];

    config.remove_record(provider, domain, rec_type, value)?;
    save_ns_config(&config)?;

    // Record audit event
    let record_desc = format!("{} {} {}", domain, record_type, value);
    let _ = audit::push_cli("system", "cli", "ns remove", &record_desc);

    dure_info!("✓ Record removed: {} {} {}", domain,
        record_type.to_uppercase(),
        value
    );
    dure_info!("");
    dure_info!("Note: This only removes from config. To remove from DNS provider,");
    dure_info!("you may need to use the provider's control panel.");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_config_path() {
        let path = get_config_path().unwrap();
        assert!(path.to_string_lossy().contains("dure"));
        assert!(path.to_string_lossy().ends_with("config.yml"));
    }
}
