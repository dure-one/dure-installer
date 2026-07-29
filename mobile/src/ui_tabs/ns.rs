//! NS tab - Domain and DNS record management

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use eframe::egui;
use egui_material3::MaterialButton;
use egui_material3::spreadsheet::{MaterialSpreadsheet, text_column};
use poll_promise::Promise;

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use crate::calc::audit;
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use crate::calc::ns::{NsConfig, RecordType, apply_record};
use directories::ProjectDirs;
use std::path::PathBuf;

/// NS tab state
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct NsTab {
    #[cfg_attr(feature = "serde", serde(skip))]
    selected_row: Option<usize>,

    /// Display data for domain table
    #[cfg_attr(feature = "serde", serde(skip))]
    domain_rows: Vec<DomainRowData>,

    /// Cached domain/record rows (domain, provider, record count) - OLD, will be removed
    #[cfg_attr(feature = "serde", serde(skip))]
    old_domain_rows: Vec<[String; 3]>,

    /// Cached records for selected domain (name, type, value)
    #[cfg_attr(feature = "serde", serde(skip))]
    record_rows: Vec<[String; 3]>,

    #[cfg_attr(feature = "serde", serde(skip))]
    loaded: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    load_error: Option<String>,

    // Spreadsheets
    #[cfg_attr(feature = "serde", serde(skip))]
    domain_spreadsheet: Option<MaterialSpreadsheet>,

    #[cfg_attr(feature = "serde", serde(skip))]
    record_spreadsheet: Option<MaterialSpreadsheet>,

    row_selection_enabled: bool,

    // Selected domain for record view (provider, domain)
    #[cfg_attr(feature = "serde", serde(skip))]
    selected_domain: Option<(String, String)>,

    // Add nameserver provider dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_add_provider_dialog: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_provider_type: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_token: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_secret_key: String,

    // GCP OAuth fields
    #[cfg_attr(feature = "serde", serde(skip))]
    add_gcp_oauth_result: Option<crate::api::gcp::oauth::OAuthResult>,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_gcp_oauth_promise:
        Option<poll_promise::Promise<Result<crate::api::gcp::oauth::OAuthResult, String>>>,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_gcp_connected_email: Option<String>,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_gcp_project_id: String,

    // GCP project list
    #[cfg_attr(feature = "serde", serde(skip))]
    add_gcp_projects: Vec<crate::api::gcp::resourcemanager::Project>,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_gcp_projects_loaded: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_gcp_projects_error: Option<String>,

    // Add domain dialog (for manually adding domains to existing providers)
    #[cfg_attr(feature = "serde", serde(skip))]
    show_add_domain_dialog: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_domain: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_domain_provider: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_domain_existing_gcp_accounts: Vec<String>,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_domain_selected_gcp_account: String,

    // Add record dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_add_record_dialog: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_record_name: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_record_type: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_record_value: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_record_apply: bool,

    // Error dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_error_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    error_message: String,

    // Nameservers dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_nameservers_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    ns_dialog_domain: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    ns_dialog_provider_ns: Vec<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    ns_dialog_actual_ns: Vec<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    ns_dialog_loading: bool,

    // Background task for provider addition
    #[cfg_attr(feature = "serde", serde(skip))]
    add_provider_promise: Option<Promise<Result<Vec<String>, String>>>,

    // Progress/status
    #[cfg_attr(feature = "serde", serde(skip))]
    progress_log: Vec<String>,
}

impl Default for NsTab {
    fn default() -> Self {
        let domain_spreadsheet = {
            let columns = vec![
                text_column("Domain", 250.0),
                text_column("Provider", 120.0),
                text_column("Records", 80.0),
            ];

            MaterialSpreadsheet::new("ns_domain_spreadsheet", columns)
                .ok()
                .map(|mut s| {
                    s.set_striped(true);
                    s.set_row_selection_enabled(true);
                    s.set_allow_selection(true);
                    s
                })
        };

        let record_spreadsheet = {
            let columns = vec![
                text_column("Name", 150.0),
                text_column("Type", 80.0),
                text_column("Value", 300.0),
            ];

            MaterialSpreadsheet::new("ns_record_spreadsheet", columns)
                .ok()
                .map(|mut s| {
                    s.set_striped(true);
                    s.set_row_selection_enabled(true);
                    s.set_allow_selection(true);
                    s
                })
        };

        Self {
            selected_row: None,
            domain_rows: Vec::new(),
            old_domain_rows: Vec::new(),
            record_rows: Vec::new(),
            loaded: false,
            load_error: None,
            domain_spreadsheet,
            record_spreadsheet,
            row_selection_enabled: true,
            selected_domain: None,
            show_add_provider_dialog: false,
            add_provider_type: "cloudflare".to_string(),
            add_token: String::new(),
            add_secret_key: String::new(),
            add_gcp_oauth_result: None,
            add_gcp_oauth_promise: None,
            add_gcp_connected_email: None,
            add_gcp_project_id: String::new(),
            add_gcp_projects: Vec::new(),
            add_gcp_projects_loaded: false,
            add_gcp_projects_error: None,
            show_add_domain_dialog: false,
            add_domain: String::new(),
            add_domain_provider: "cloudflare".to_string(),
            add_domain_existing_gcp_accounts: Vec::new(),
            add_domain_selected_gcp_account: String::new(),
            show_add_record_dialog: false,
            add_record_name: String::new(),
            add_record_type: "a".to_string(),
            add_record_value: String::new(),
            add_record_apply: true,
            show_error_dialog: false,
            error_message: String::new(),
            show_nameservers_dialog: false,
            ns_dialog_domain: String::new(),
            ns_dialog_provider_ns: Vec::new(),
            ns_dialog_actual_ns: Vec::new(),
            ns_dialog_loading: false,
            add_provider_promise: None,
            progress_log: Vec::new(),
        }
    }
}

/// Display data for domain table row + drawer
#[derive(Clone, Debug)]
struct DomainRowData {
    domain: String,
    provider: String,              // Internal identifier (e.g., "cloudflare", "gcloud:email")
    provider_display: String,      // Display name (e.g., "Cloudflare", "Google Cloud (email)")
    records: Vec<crate::calc::ns::DnsRecord>,
}

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
fn build_domain_rows(config: &crate::calc::ns::NsConfig) -> Vec<DomainRowData> {
    let mut rows = Vec::new();

    // Add domains from regular providers (cloudflare, porkbun, duckdns)
    for (provider, provider_config) in &config.providers {
        for domain_entry in &provider_config.domains {
            let provider_display = match provider.as_str() {
                "cloudflare" => "Cloudflare".to_string(),
                "porkbun" => "Porkbun".to_string(),
                "duckdns" => "DuckDNS".to_string(),
                _ => provider.clone(),
            };

            rows.push(DomainRowData {
                domain: domain_entry.domain.clone(),
                provider: provider.clone(),
                provider_display,
                records: domain_entry.records.clone(),
            });
        }
    }

    // Add domains from GCP accounts
    for gcp_account in &config.gcp_accounts {
        let provider = format!("gcloud:{}", gcp_account.connected_email);
        let provider_display = format!("Google Cloud ({})", gcp_account.connected_email);

        for domain_entry in &gcp_account.domains {
            rows.push(DomainRowData {
                domain: domain_entry.domain.clone(),
                provider: provider.clone(),
                provider_display: provider_display.clone(),
                records: domain_entry.records.clone(),
            });
        }
    }

    rows.sort_by(|a, b| a.domain.cmp(&b.domain));
    rows
}

#[cfg(any(target_os = "android", target_arch = "wasm32"))]
fn build_domain_rows(_config: &crate::calc::ns::NsConfig) -> Vec<DomainRowData> {
    Vec::new()
}

/// Get config file path
#[cfg(not(target_arch = "wasm32"))]
fn get_config_path() -> Result<PathBuf, String> {
    let proj_dirs = ProjectDirs::from("com", "dure", "dure")
        .ok_or_else(|| "Failed to get project directories".to_string())?;
    Ok(proj_dirs.config_dir().join("config.yml"))
}

/// Parse Porkbun credentials from combined token format
/// Format: "apikey::secretkey"
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
fn parse_porkbun_credentials(token: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = token.split("::").collect();
    if parts.len() == 2 {
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

/// Load NS config from YAML
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
fn load_ns_config() -> Result<NsConfig, String> {
    let config_path = get_config_path()?;

    if !config_path.exists() {
        return Ok(NsConfig::default());
    }

    let yaml = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    #[derive(serde::Deserialize)]
    struct FullConfig {
        #[serde(default)]
        ns: NsConfig,
    }

    match serde_yaml::from_str::<FullConfig>(&yaml) {
        Ok(full_config) => Ok(full_config.ns),
        Err(_) => Ok(NsConfig::default()),
    }
}

/// Save NS config to YAML
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
fn save_ns_config(ns_config: &NsConfig) -> Result<(), String> {
    let config_path = get_config_path()?;

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config dir: {}", e))?;
    }

    let mut full_config = if config_path.exists() {
        let yaml = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;
        serde_yaml::from_str::<serde_yaml::Value>(&yaml)
            .unwrap_or_else(|_| serde_yaml::Value::Mapping(Default::default()))
    } else {
        serde_yaml::Value::Mapping(Default::default())
    };

    let ns_value = serde_yaml::to_value(ns_config)
        .map_err(|e| format!("Failed to serialize NS config: {}", e))?;

    if let serde_yaml::Value::Mapping(ref mut map) = full_config {
        map.insert(serde_yaml::Value::String("ns".to_string()), ns_value);
    }

    let yaml = serde_yaml::to_string(&full_config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    std::fs::write(&config_path, yaml).map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

/// Add GCP account to config
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
fn add_gcp_account_to_config(
    oauth: &crate::api::gcp::oauth::OAuthResult,
    email: &str,
    project_id: &str,
) -> Result<(), String> {
    use crate::calc::ns::GcpAccount;

    let mut config = load_ns_config().map_err(|e| format!("Failed to load config: {}", e))?;

    let account = GcpAccount {
        access_token: oauth.access_token.clone(),
        refresh_token: oauth.refresh_token.clone(),
        token_expiry: oauth.expires_at,
        connected_email: email.to_string(),
        project_id: project_id.to_string(),
        domains: Vec::new(),
    };

    config
        .add_gcp_account(account)
        .map_err(|e| format!("Failed to add GCP account: {}", e))?;

    save_ns_config(&config).map_err(|e| format!("Failed to save config: {}", e))?;

    Ok(())
}

/// Execute add provider in blocking mode (for background thread)
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
fn execute_add_provider_blocking(provider: String, token: String) -> Result<Vec<String>, String> {
    use crate::api::{ns_cloudflare, ns_porkbun};
    use crate::calc::ns::{NsConfig, RecordType};

    let mut log = Vec::new();

    log.push(format!("Fetching domains from {}...", provider));

    match provider.as_str() {
        "cloudflare" => {
            log.push("Calling Cloudflare API...".to_string());
            let client = ns_cloudflare::CloudflareClient::new(token.clone());

            let zones = client
                .list_zones()
                .map_err(|e| format!("Error fetching domains: {}", e))?;

            log.push(format!("API call successful, found {} zones", zones.len()));

            let mut config =
                load_ns_config().map_err(|e| format!("Error loading config: {}", e))?;

            log.push(format!(
                "Loaded config with {} existing domains",
                config.total_domains()
            ));
            let mut added_count = 0;

            for zone in zones {
                log.push(format!("  Processing zone: {}", zone.name));

                match client.get_records(&zone.id) {
                    Ok(records) => {
                        let filtered_records: Vec<_> = records
                            .iter()
                            .filter(|r| {
                                let rt = r.record_type.to_uppercase();
                                rt == "A" || rt == "AAAA" || rt == "TXT"
                            })
                            .collect();

                        log.push(format!(
                            "    Found {} DNS records (A/AAAA/TXT)",
                            filtered_records.len()
                        ));

                        match config.add_domain(provider.clone(), zone.name.clone(), token.clone())
                        {
                            Ok(_) => {
                                added_count += 1;

                                if let Some(domain_entry) =
                                    config.get_domain_mut(&provider, &zone.name)
                                {
                                    for record in filtered_records {
                                        let rec_type = RecordType::from_str(
                                            &record.record_type.to_lowercase(),
                                        );
                                        if let Some(rt) = rec_type {
                                            domain_entry.records.push(crate::calc::ns::DnsRecord {
                                                record_type: rt,
                                                name: record.name.clone(),
                                                value: record.content.clone(),
                                                ttl: Some(record.ttl),
                                            });
                                        }
                                    }
                                    log.push(format!(
                                        "    ✓ Added {} with {} records",
                                        zone.name,
                                        domain_entry.records.len()
                                    ));
                                }
                            }
                            Err(e) => {
                                log.push(format!("    ⚠ Skipped {}: {}", zone.name, e));
                            }
                        }
                    }
                    Err(e) => {
                        log.push(format!(
                            "    ⚠ Failed to fetch records for {}: {}",
                            zone.name, e
                        ));
                        let _ =
                            config.add_domain(provider.clone(), zone.name.clone(), token.clone());
                    }
                }
            }

            log.push(format!(
                "Total in config now: {} entries",
                config.total_domains()
            ));

            if added_count > 0 {
                log.push("Saving config...".to_string());
                save_ns_config(&config).map_err(|e| format!("Error saving config: {}", e))?;

                log.push("✓ Config saved successfully".to_string());
                let _ = audit::push_gui("system", "desktop", "ns add provider", &provider);
                log.push(format!(
                    "✓ Added {} provider with {} entries",
                    provider, added_count
                ));
            } else {
                log.push("⚠ No new entries added (all already exist)".to_string());
            }

            Ok(log)
        }
        "porkbun" => {
            log.push("Calling Porkbun API...".to_string());

            let (api_key, secret_key) = parse_porkbun_credentials(&token)
                .ok_or_else(|| "Error: Invalid Porkbun credentials format".to_string())?;

            let client = ns_porkbun::PorkbunClient::new(api_key, secret_key);

            let domains = client
                .list_domains()
                .map_err(|e| format!("Error fetching domains: {}", e))?;

            log.push(format!(
                "API call successful, found {} domains",
                domains.len()
            ));

            let mut config =
                load_ns_config().map_err(|e| format!("Error loading config: {}", e))?;

            log.push(format!(
                "Loaded config with {} existing domains",
                config.total_domains()
            ));
            let mut added_count = 0;

            for domain in domains {
                log.push(format!("  Processing domain: {}", domain));

                match client.get_records(&domain) {
                    Ok(records) => {
                        let filtered_records: Vec<_> = records
                            .iter()
                            .filter(|r| {
                                let rt = r.record_type.to_uppercase();
                                rt == "A" || rt == "AAAA" || rt == "TXT"
                            })
                            .collect();

                        log.push(format!(
                            "    Found {} DNS records (A/AAAA/TXT)",
                            filtered_records.len()
                        ));

                        match config.add_domain(provider.clone(), domain.clone(), token.clone()) {
                            Ok(_) => {
                                added_count += 1;

                                if let Some(domain_entry) =
                                    config.get_domain_mut(&provider, &domain)
                                {
                                    for record in filtered_records {
                                        let rec_type = RecordType::from_str(
                                            &record.record_type.to_lowercase(),
                                        );
                                        if let Some(rt) = rec_type {
                                            domain_entry.records.push(crate::calc::ns::DnsRecord {
                                                record_type: rt,
                                                name: record.name.clone(),
                                                value: record.content.clone(),
                                                ttl: record.ttl.parse().ok(),
                                            });
                                        }
                                    }
                                    log.push(format!(
                                        "    ✓ Added {} with {} records",
                                        domain,
                                        domain_entry.records.len()
                                    ));
                                }
                            }
                            Err(e) => {
                                log.push(format!("    ⚠ Skipped {}: {}", domain, e));
                            }
                        }
                    }
                    Err(e) => {
                        log.push(format!(
                            "    ⚠ Failed to fetch records for {}: {}",
                            domain, e
                        ));
                        let _ = config.add_domain(provider.clone(), domain.clone(), token.clone());
                    }
                }
            }

            log.push(format!(
                "Total in config now: {} entries",
                config.total_domains()
            ));

            if added_count > 0 {
                log.push("Saving config...".to_string());
                save_ns_config(&config).map_err(|e| format!("Error saving config: {}", e))?;

                log.push("✓ Config saved successfully".to_string());
                let _ = audit::push_gui("system", "desktop", "ns add provider", &provider);
                log.push(format!(
                    "✓ Added {} provider with {} entries",
                    provider, added_count
                ));
            } else {
                log.push("⚠ No new entries added (all already exist)".to_string());
            }

            Ok(log)
        }
        "duckdns" => {
            log.push("DuckDNS domains cannot be auto-discovered. Saving provider for manual domain addition.".to_string());

            let mut config =
                load_ns_config().map_err(|e| format!("Error loading config: {}", e))?;

            let placeholder = format!("{} (provider)", provider);
            config
                .add_domain(provider.clone(), placeholder.clone(), token.clone())
                .map_err(|e| format!("Failed to add provider: {}", e))?;

            save_ns_config(&config).map_err(|e| format!("Error saving config: {}", e))?;

            let _ = audit::push_gui("system", "desktop", "ns add provider", &provider);
            log.push(format!(
                "✓ Added {} provider (add domains manually)",
                provider
            ));

            Ok(log)
        }
        p if p.starts_with("gcloud") => {
            log.push("Calling Google Cloud DNS API...".to_string());

            // Parse token format: "access_token::project_id"
            let parts: Vec<&str> = token.split("::").collect();
            if parts.len() != 2 {
                return Err(
                    "Error: Invalid GCP token format (expected access_token::project_id)"
                        .to_string(),
                );
            }

            let access_token = parts[0].to_string();
            let project_id = parts[1].to_string();

            use crate::api::ns_gcp;
            let client = ns_gcp::GcpDnsClient::new(access_token);

            // List managed zones
            let zones = client
                .list_managed_zones(&project_id)
                .map_err(|e| format!("Error fetching managed zones: {}", e))?;

            log.push(format!(
                "API call successful, found {} managed zones",
                zones.len()
            ));

            let mut config =
                load_ns_config().map_err(|e| format!("Error loading config: {}", e))?;

            log.push(format!(
                "Loaded config with {} existing domains",
                config.total_domains()
            ));
            let mut added_count = 0;
            let has_zones = !zones.is_empty();

            // If no zones found, add a placeholder to store OAuth info
            if !has_zones {
                log.push(
                    "No managed zones found. Adding placeholder entry to store OAuth connection."
                        .to_string(),
                );
                let placeholder = format!("gcloud ({})", project_id);
                match config.add_domain(provider.clone(), placeholder.clone(), token.clone()) {
                    Ok(_) => {
                        added_count += 1;
                        log.push(format!("  ✓ Added placeholder: {}", placeholder));
                    }
                    Err(e) => {
                        log.push(format!("  ⚠ Failed to add placeholder: {}", e));
                    }
                }
            }

            for zone in zones {
                // Remove trailing dot from DNS name for display
                let domain_name = zone.dns_name.trim_end_matches('.');
                log.push(format!("  Processing zone: {}", domain_name));

                // Fetch resource record sets for this zone
                match client.list_rrsets(&project_id, &zone.name) {
                    Ok(rrsets) => {
                        // Filter for A, AAAA, TXT records only
                        let filtered_records: Vec<_> = rrsets
                            .iter()
                            .filter(|r| {
                                let rt = r.record_type.to_uppercase();
                                rt == "A" || rt == "AAAA" || rt == "TXT"
                            })
                            .collect();

                        log.push(format!(
                            "    Found {} DNS records (A/AAAA/TXT)",
                            filtered_records.len()
                        ));

                        // Add or update domain
                        match config.add_domain(
                            provider.clone(),
                            domain_name.to_string(),
                            token.clone(),
                        ) {
                            Ok(_) => {
                                added_count += 1;

                                // Add DNS records
                                if let Some(domain_entry) =
                                    config.get_domain_mut(&provider, domain_name)
                                {
                                    for rrset in filtered_records {
                                        let rec_type =
                                            RecordType::from_str(&rrset.record_type.to_lowercase());
                                        if let Some(rt) = rec_type {
                                            // GCP returns multiple values in rrdatas array
                                            for value in &rrset.rrdatas {
                                                domain_entry.records.push(
                                                    crate::calc::ns::DnsRecord {
                                                        record_type: rt.clone(),
                                                        name: rrset
                                                            .name
                                                            .trim_end_matches('.')
                                                            .to_string(),
                                                        value: value.clone(),
                                                        ttl: Some(rrset.ttl),
                                                    },
                                                );
                                            }
                                        }
                                    }
                                    log.push(format!(
                                        "    ✓ Added {} with {} records",
                                        domain_name,
                                        domain_entry.records.len()
                                    ));
                                }
                            }
                            Err(e) => {
                                log.push(format!("    ⚠ Skipped {}: {}", domain_name, e));
                            }
                        }
                    }
                    Err(e) => {
                        log.push(format!(
                            "    ⚠ Failed to fetch records for {}: {}",
                            domain_name, e
                        ));
                        // Still add the domain even if records fetch fails
                        let _ = config.add_domain(
                            provider.clone(),
                            domain_name.to_string(),
                            token.clone(),
                        );
                    }
                }
            }

            log.push(format!(
                "Total in config now: {} entries",
                config.total_domains()
            ));

            if added_count > 0 {
                log.push("Saving config...".to_string());
                save_ns_config(&config).map_err(|e| format!("Error saving config: {}", e))?;

                log.push("✓ Config saved successfully".to_string());
                let _ = audit::push_gui("system", "desktop", "ns add provider", &provider);

                if !has_zones {
                    log.push(format!(
                        "✓ Added {} provider (no zones found, create zones in GCP Console)",
                        provider
                    ));
                } else {
                    log.push(format!(
                        "✓ Added {} provider with {} managed zone{}",
                        provider,
                        added_count,
                        if added_count == 1 { "" } else { "s" }
                    ));
                }
            } else {
                log.push("⚠ No new entries added (all already exist)".to_string());
            }

            Ok(log)
        }
        _ => Err(format!("Error: Unknown provider '{}'", provider)),
    }
}

impl NsTab {
    /// Render the NS tab UI
    pub fn ui(&mut self, ui: &mut egui::Ui, mut vm: Option<&mut crate::viewmodel::ViewModel>) {
        // ViewModel event processing (MVVM pattern)
        if let Some(ref mut vm) = vm {
            let events = vm.poll_events(ui.ctx());
            for event in events {
                use crate::viewmodel::ViewModelEvent;
                use crate::viewmodel::ns::NsEvent;

                match event {
                    ViewModelEvent::Ns(NsEvent::RecordAdded {
                        provider_name,
                        domain,
                        record_id,
                    }) => {
                        self.add_progress(format!("✓ Record added (ID: {})", record_id));

                        // Update config
                        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
                        if let Ok(mut config) = load_ns_config() {
                            // Note: Record details already in config from UI,
                            // this event confirms API succeeded
                            if let Err(e) = save_ns_config(&config) {
                                self.add_progress(format!("Error saving config: {}", e));
                            } else {
                                self.load_records();
                                self.load_data();
                            }
                        }
                    }
                    ViewModelEvent::Ns(NsEvent::ProviderAdded { name, domains }) => {
                        self.add_progress(format!(
                            "✓ Added {} provider with {} domain(s)",
                            name,
                            domains.len()
                        ));

                        // Save domains to config
                        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
                        {
                            if let Ok(mut config) = load_ns_config() {
                                let api_token = if name == "porkbun" {
                                    format!("{}::{}", self.add_token, self.add_secret_key)
                                } else {
                                    self.add_token.clone()
                                };

                                let mut added_count = 0;
                                for (domain, records) in domains {
                                    if let Ok(_) = config.add_domain(
                                        name.clone(),
                                        domain.clone(),
                                        api_token.clone(),
                                    ) {
                                        added_count += 1;
                                        // Add records to domain
                                        if let Some(domain_entry) =
                                            config.get_domain_mut(&name, &domain)
                                        {
                                            domain_entry.records.extend(records);
                                        }
                                        self.add_progress(format!("  ✓ Added domain: {}", domain));
                                    }
                                }

                                if let Err(e) = save_ns_config(&config) {
                                    self.add_progress(format!("Error saving config: {}", e));
                                } else {
                                    self.add_progress(format!(
                                        "✓ Configuration saved ({} domains)",
                                        added_count
                                    ));
                                    self.load_data();
                                }
                            }
                        }
                    }
                    ViewModelEvent::Ns(NsEvent::Progress {
                        operation,
                        progress,
                        status,
                    }) => {
                        if operation == "add_provider" {
                            self.add_progress(format!("[{:>3.0}%] {}", progress * 100.0, status));
                        }
                    }
                    ViewModelEvent::Ns(NsEvent::RecordDeleted {
                        provider_name,
                        domain,
                        record_id,
                    }) => {
                        self.add_progress(format!("✓ Deleted record: {}", record_id));

                        // Remove record from config
                        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
                        {
                            // Parse record_id as "name:type"
                            let parts: Vec<&str> = record_id.split(':').collect();
                            if parts.len() == 2 {
                                let name = parts[0];
                                let record_type = parts[1];

                                if let Ok(mut config) = load_ns_config() {
                                    if let Some(domain_entry) =
                                        config.get_domain_mut(&provider_name, &domain)
                                    {
                                        domain_entry.records.retain(|r| {
                                            !(r.name == name
                                                && r.record_type.as_str().to_lowercase()
                                                    == record_type.to_lowercase())
                                        });

                                        if let Err(e) = save_ns_config(&config) {
                                            self.add_progress(format!(
                                                "Error saving config: {}",
                                                e
                                            ));
                                        } else {
                                            self.load_records();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    ViewModelEvent::Ns(NsEvent::DomainDeleted {
                        provider_name,
                        domain,
                    }) => {
                        self.add_progress(format!("✓ Deleted domain: {}", domain));

                        // Remove domain from config
                        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
                        {
                            if let Ok(mut config) = load_ns_config() {
                                let _ = config.remove_domain(&provider_name, &domain);

                                // Handle GCP placeholder logic
                                if provider_name.starts_with("gcloud:") {
                                    let email = &provider_name[7..];
                                    if let Some(account) = config.get_gcp_account(email) {
                                        if account.domains.is_empty() {
                                            let stub_domain =
                                                format!("gcloud ({})", account.project_id);
                                            let token = account.access_token.clone();
                                            let _ = config.add_domain(
                                                provider_name.clone(),
                                                stub_domain,
                                                token,
                                            );
                                        }
                                    }
                                }

                                if let Err(e) = save_ns_config(&config) {
                                    self.add_progress(format!("Error saving config: {}", e));
                                } else {
                                    if let Some((sel_prov, sel_dom)) = &self.selected_domain {
                                        if sel_prov == &provider_name && sel_dom == &domain {
                                            self.selected_domain = None;
                                            self.record_rows.clear();
                                        }
                                    }
                                    self.load_data();
                                }
                            }
                        }
                    }
                    ViewModelEvent::Ns(NsEvent::Error { operation, error }) => {
                        if operation == "add_record" {
                            self.error_message = format!("Failed to add DNS record:\n\n{}", error);
                            self.show_error_dialog = true;
                        } else if operation == "add_provider" {
                            self.add_progress(format!("❌ Failed to add provider: {}", error));
                        } else if operation == "delete_record" {
                            self.error_message =
                                format!("Failed to delete DNS record:\n\n{}", error);
                            self.show_error_dialog = true;
                        } else if operation == "delete_domain" {
                            self.add_progress(format!("❌ Failed to delete domain: {}", error));
                        }
                    }
                    _ => {}
                }
            }
        }

        // Poll background task for adding provider (TODO: Replace with ViewModel)
        let promise_result = if let Some(promise) = &self.add_provider_promise {
            promise.ready().cloned()
        } else {
            None
        };

        if let Some(result) = promise_result {
            match result {
                Ok(log_messages) => {
                    for msg in log_messages {
                        self.add_progress(msg);
                    }
                    self.add_progress("Reloading UI data...".to_string());
                    self.load_data();
                    self.add_progress("✓ UI refreshed".to_string());
                }
                Err(err_msg) => {
                    self.add_progress(format!("❌ {}", err_msg));
                }
            }
            self.add_provider_promise = None;
        }

        ui.heading("Nameserver Management");
        ui.add_space(4.0);
        ui.label("Manage domains and DNS records (A, AAAA, TXT) for Cloudflare, Google Cloud DNS, DuckDNS, and Porkbun.");
        ui.add_space(8.0);

        // Load data on first render
        if !self.loaded {
            self.load_data();
        }

        // Show error if any
        if let Some(ref error) = self.load_error {
            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
            ui.add_space(8.0);
        }

        // Nameserver provider and domain management section
        ui.horizontal(|ui| {
            ui.label("Domains:");
            ui.add_space(8.0);

            if ui
                .add(MaterialButton::filled("Add Nameserver Provider"))
                .clicked()
            {
                self.show_add_provider_dialog = true;
                self.add_token.clear();
                self.add_secret_key.clear();
            }

            if ui.add(MaterialButton::filled("Add Domain")).clicked() {
                self.show_add_domain_dialog = true;
                self.add_domain.clear();
            }

            let domain_selected = self
                .domain_spreadsheet
                .as_ref()
                .and_then(|s| s.get_selected_row())
                .is_some();

            let delete_button = MaterialButton::outlined("Delete Domain");
            let delete_button = if domain_selected {
                delete_button
            } else {
                delete_button.enabled(false)
            };

            if ui.add(delete_button).clicked() {
                if let Some(idx) = self
                    .domain_spreadsheet
                    .as_ref()
                    .and_then(|s| s.get_selected_row())
                {
                    if idx < self.old_domain_rows.len() {
                        let domain = self.old_domain_rows[idx][0].clone();
                        self.execute_delete_domain(&domain, vm.as_deref_mut());
                    }
                }
            }

            if ui.add(MaterialButton::text("Refresh")).clicked() {
                self.refresh_from_api();
            }
        });

        ui.add_space(8.0);

        // Domain spreadsheet
        if let Some(ref mut spreadsheet) = self.domain_spreadsheet {
            ui.push_id("domain_spreadsheet_container", |ui| {
                spreadsheet.show(ui);
            });

            // Check for selection change
            if let Some(selected_idx) = spreadsheet.get_selected_row() {
                if selected_idx < self.old_domain_rows.len() {
                    let domain = self.old_domain_rows[selected_idx][0].clone();
                    let provider_display = self.old_domain_rows[selected_idx][1].clone();
                    // Extract provider identifier
                    let provider = if provider_display.starts_with("Google Cloud (")
                        && provider_display.ends_with(")")
                    {
                        // Extract email from "Google Cloud (email)" and format as "gcloud:email"
                        let email = &provider_display[14..provider_display.len() - 1];
                        format!("gcloud:{}", email)
                    } else {
                        // Regular provider
                        provider_display
                    };
                    let selection = (provider, domain);
                    if self.selected_domain.as_ref() != Some(&selection) {
                        self.selected_domain = Some(selection);
                        self.load_records();
                    }
                }
            }
        }

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(8.0);

        // Records section
        if let Some((provider, domain)) = self.selected_domain.clone() {
            ui.horizontal(|ui| {
                ui.label(format!("Records for {} ({}):", domain, provider));
                ui.add_space(8.0);

                if ui.add(MaterialButton::filled("Add Record")).clicked() {
                    self.show_add_record_dialog = true;
                    self.add_record_name.clear();
                    self.add_record_value.clear();
                }

                // Disable "Show Nameservers" button for DuckDNS
                let is_duckdns =
                    provider.to_lowercase() == "duckdns" || domain.ends_with(".duckdns.org");
                let nameservers_button = MaterialButton::outlined("Show Nameservers");
                let nameservers_button = if is_duckdns {
                    nameservers_button.enabled(false)
                } else {
                    nameservers_button
                };

                if ui.add(nameservers_button).clicked() {
                    self.show_nameservers(&provider, &domain);
                }

                let record_selected = self
                    .record_spreadsheet
                    .as_ref()
                    .and_then(|s| s.get_selected_row())
                    .is_some();

                let delete_record_button = MaterialButton::outlined("Delete Record");
                let delete_record_button = if record_selected {
                    delete_record_button
                } else {
                    delete_record_button.enabled(false)
                };

                if ui.add(delete_record_button).clicked() {
                    if let Some(idx) = self
                        .record_spreadsheet
                        .as_ref()
                        .and_then(|s| s.get_selected_row())
                    {
                        if idx < self.record_rows.len() {
                            let name = self.record_rows[idx][0].clone();
                            let record_type = self.record_rows[idx][1].clone();
                            let value = self.record_rows[idx][2].clone();
                            self.execute_delete_record(
                                &name,
                                &record_type,
                                &value,
                                vm.as_deref_mut(),
                            );
                        }
                    }
                }
            });

            ui.add_space(8.0);

            if let Some(ref mut spreadsheet) = self.record_spreadsheet {
                ui.push_id("record_spreadsheet_container", |ui| {
                    spreadsheet.show(ui);
                });
            }
        } else {
            ui.label("Select a domain to view and manage its records");
        }

        // Progress log
        if !self.progress_log.is_empty() {
            ui.add_space(16.0);
            ui.separator();
            ui.add_space(8.0);
            ui.label("Progress:");
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    for line in &self.progress_log {
                        ui.label(line);
                    }
                });
        }

        // Dialogs
        self.show_add_provider_dialog(ui.ctx(), vm.as_deref_mut());
        self.show_add_domain_dialog(ui.ctx());
        self.show_add_record_dialog(ui.ctx(), vm);
        self.show_error_dialog(ui.ctx());
        self.show_nameservers_dialog(ui.ctx());
    }

    /// Load domain data from config
    fn load_data(&mut self) {
        self.loaded = true;
        self.load_error = None;

        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            match load_ns_config() {
                Ok(config) => {
                    self.add_progress(format!(
                        "Loading {} domains from config",
                        config.total_domains()
                    ));

                    self.old_domain_rows = config
                        .iter_all_domains()
                        .iter()
                        .map(|(provider, d)| {
                            // For gcloud:email format, display as "Google Cloud (email)"
                            let provider_display = if provider.starts_with("gcloud:") {
                                let email = &provider[7..];
                                format!("Google Cloud ({})", email)
                            } else {
                                provider.to_string()
                            };

                            self.add_progress(format!("  - {} ({})", d.domain, provider_display));
                            [
                                d.domain.clone(),
                                provider_display,
                                d.records.len().to_string(),
                            ]
                        })
                        .collect();

                    // Recreate spreadsheet with fresh data to avoid duplicates
                    let columns = vec![
                        text_column("Domain", 250.0),
                        text_column("Provider", 200.0), // Wider to show email for gcloud
                        text_column("Records", 80.0),
                    ];

                    self.domain_spreadsheet =
                        MaterialSpreadsheet::new("ns_domain_spreadsheet", columns)
                            .ok()
                            .map(|mut s| {
                                s.set_striped(true);
                                s.set_row_selection_enabled(true);
                                s.set_allow_selection(true);
                                s.init_with_data(
                                    self.old_domain_rows
                                        .iter()
                                        .map(|row| row.iter().cloned().collect())
                                        .collect(),
                                );
                                s
                            });
                }
                Err(e) => {
                    self.load_error = Some(e);
                }
            }
        }

        #[cfg(any(target_os = "android", target_arch = "wasm32"))]
        {
            self.load_error = Some("NS management not supported on this platform".to_string());
        }
    }

    /// Load records for selected domain
    fn load_records(&mut self) {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            if let Some((ref provider, ref domain)) = self.selected_domain {
                match load_ns_config() {
                    Ok(config) => {
                        if let Some(domain_entry) = config.get_domain(provider, domain) {
                            self.record_rows = domain_entry
                                .records
                                .iter()
                                .map(|r| {
                                    [
                                        r.name.clone(),
                                        r.record_type.as_str().to_uppercase(),
                                        r.value.clone(),
                                    ]
                                })
                                .collect();

                            // Recreate spreadsheet with fresh data to avoid duplicates
                            let columns = vec![
                                text_column("Name", 150.0),
                                text_column("Type", 80.0),
                                text_column("Value", 300.0),
                            ];

                            self.record_spreadsheet =
                                MaterialSpreadsheet::new("ns_record_spreadsheet", columns)
                                    .ok()
                                    .map(|mut s| {
                                        s.set_striped(true);
                                        s.set_row_selection_enabled(true);
                                        s.set_allow_selection(true);
                                        s.init_with_data(
                                            self.record_rows
                                                .iter()
                                                .map(|row| row.iter().cloned().collect())
                                                .collect(),
                                        );
                                        s
                                    });
                        }
                    }
                    Err(e) => {
                        self.add_progress(format!("Error loading records: {}", e));
                    }
                }
            }
        }
    }

    /// Show add nameserver provider dialog
    fn show_add_provider_dialog(
        &mut self,
        ctx: &egui::Context,
        mut vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        if !self.show_add_provider_dialog {
            return;
        }

        let mut open = true;
        egui::Window::new("Add Nameserver Provider")
            .id(egui::Id::new("ns_add_provider_dialog"))
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Provider type:");
                egui::ComboBox::from_id_salt("provider_type_combo")
                    .selected_text(match self.add_provider_type.as_str() {
                        "cloudflare" => "Cloudflare",
                        "gcloud" => "Google Cloud DNS",
                        "duckdns" => "DuckDNS",
                        "porkbun" => "Porkbun",
                        _ => &self.add_provider_type,
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.add_provider_type,
                            "cloudflare".to_string(),
                            "Cloudflare",
                        );
                        ui.selectable_value(
                            &mut self.add_provider_type,
                            "gcloud".to_string(),
                            "Google Cloud DNS",
                        );
                        ui.selectable_value(
                            &mut self.add_provider_type,
                            "duckdns".to_string(),
                            "DuckDNS",
                        );
                        ui.selectable_value(
                            &mut self.add_provider_type,
                            "porkbun".to_string(),
                            "Porkbun",
                        );
                    });

                ui.add_space(8.0);

                // Show different fields based on provider
                if self.add_provider_type == "porkbun" {
                    ui.label("API Key:");
                    ui.add(egui::TextEdit::singleline(&mut self.add_token).password(true));

                    ui.add_space(8.0);
                    ui.label("Secret Key:");
                    ui.add(egui::TextEdit::singleline(&mut self.add_secret_key).password(true));
                } else if self.add_provider_type == "gcloud" {
                    // GCP OAuth connection UI
                    ui.separator();
                    ui.add_space(8.0);

                    // Check for OAuth promise result
                    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
                    {
                        if let Some(promise) = &self.add_gcp_oauth_promise {
                            if let Some(result) = promise.ready() {
                                match result {
                                    Ok(oauth_result) => {
                                        self.add_gcp_oauth_result = Some(oauth_result.clone());
                                        self.fetch_gcp_connected_email();
                                        self.add_gcp_oauth_promise = None;
                                    }
                                    Err(e) => {
                                        self.add_progress(format!("OAuth failed: {}", e));
                                        self.add_gcp_oauth_promise = None;
                                    }
                                }
                            }
                        }
                    }

                    if let Some(email) = &self.add_gcp_connected_email {
                        ui.colored_label(
                            egui::Color32::from_rgb(72, 187, 120),
                            format!("✓ Connected as: {}", email),
                        );
                        ui.add_space(8.0);

                        // Load projects if not loaded yet
                        if !self.add_gcp_projects_loaded {
                            self.load_gcp_projects();
                        }

                        // Show error if project loading failed
                        if let Some(error) = &self.add_gcp_projects_error {
                            ui.colored_label(
                                egui::Color32::from_rgb(245, 101, 101),
                                format!("⚠ {}", error),
                            );
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                if ui.button("🔄 Retry").clicked() {
                                    self.add_gcp_projects_loaded = false;
                                    self.add_gcp_projects_error = None;
                                }
                            });
                            ui.add_space(8.0);
                        }

                        // Filter for active projects only
                        let active_projects: Vec<_> = self
                            .add_gcp_projects
                            .iter()
                            .filter(|p| p.is_active())
                            .collect();

                        if !active_projects.is_empty() {
                            ui.label("Select GCP Project:");
                            ui.add_space(4.0);

                            // Determine combo box display text
                            let combo_display = if !self.add_gcp_project_id.is_empty() {
                                // Find matching project for display
                                active_projects
                                    .iter()
                                    .find(|p| p.project_id == self.add_gcp_project_id)
                                    .map(|p| {
                                        let display = p.display_name();
                                        if display != p.project_id {
                                            format!("{} ({})", display, p.project_id)
                                        } else {
                                            p.project_id.clone()
                                        }
                                    })
                                    .unwrap_or_else(|| self.add_gcp_project_id.clone())
                            } else {
                                // Auto-select first project
                                if let Some(first) = active_projects.first() {
                                    self.add_gcp_project_id = first.project_id.clone();
                                    let display = first.display_name();
                                    if display != first.project_id {
                                        format!("{} ({})", display, first.project_id)
                                    } else {
                                        first.project_id.clone()
                                    }
                                } else {
                                    "Select a project".to_string()
                                }
                            };

                            egui::ComboBox::from_id_salt("ns_gcp_project_combo")
                                .selected_text(&combo_display)
                                .width(350.0)
                                .show_ui(ui, |ui| {
                                    for project in &active_projects {
                                        let display = project.display_name();
                                        let label = if display != project.project_id {
                                            format!("{} ({})", display, project.project_id)
                                        } else {
                                            project.project_id.clone()
                                        };

                                        if ui
                                            .selectable_label(
                                                self.add_gcp_project_id == project.project_id,
                                                label,
                                            )
                                            .clicked()
                                        {
                                            self.add_gcp_project_id = project.project_id.clone();
                                        }
                                    }
                                });

                            ui.add_space(4.0);
                            ui.colored_label(
                                egui::Color32::GRAY,
                                format!(
                                    "ℹ Found {} active project{}",
                                    active_projects.len(),
                                    if active_projects.len() == 1 { "" } else { "s" }
                                ),
                            );
                        } else if self.add_gcp_projects_loaded {
                            ui.colored_label(
                                egui::Color32::from_rgb(255, 152, 0),
                                "⚠ No active projects found",
                            );
                            ui.add_space(4.0);
                            ui.label("Please create a project in GCP Console first.");
                        } else {
                            ui.spinner();
                            ui.label("Loading projects...");
                        }
                    } else if self.add_gcp_oauth_promise.is_some() {
                        ui.spinner();
                        ui.label("Waiting for authorization...");
                        ui.label("Please complete the OAuth flow in your browser.");
                    } else {
                        if ui
                            .add(MaterialButton::outlined("Connect to Google Cloud"))
                            .clicked()
                        {
                            self.start_gcp_oauth();
                        }
                        ui.add_space(4.0);
                        ui.colored_label(egui::Color32::GRAY, "⚠ Connection required for GCP DNS");
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                } else {
                    ui.label("API Key:");
                    ui.add(egui::TextEdit::singleline(&mut self.add_token).password(true));
                }

                ui.add_space(8.0);
                if self.add_provider_type != "gcloud" {
                    ui.label("This will fetch all domains from the provider (except Porkbun).");
                } else {
                    ui.label("This will fetch all managed zones from GCP Cloud DNS.");
                }

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    let can_add = if self.add_provider_type == "gcloud" {
                        self.add_gcp_connected_email.is_some()
                            && !self.add_gcp_project_id.is_empty()
                    } else {
                        true
                    };

                    let add_button = if can_add {
                        MaterialButton::filled("Add")
                    } else {
                        MaterialButton::filled("Add").enabled(false)
                    };

                    if ui.add(add_button).clicked() {
                        self.start_add_provider_background(vm.as_deref_mut());
                        self.show_add_provider_dialog = false;
                    }

                    if ui.add(MaterialButton::text("Cancel")).clicked() {
                        self.show_add_provider_dialog = false;
                        self.add_gcp_oauth_result = None;
                        self.add_gcp_oauth_promise = None;
                        self.add_gcp_connected_email = None;
                        self.add_gcp_project_id.clear();
                        self.add_gcp_projects.clear();
                        self.add_gcp_projects_loaded = false;
                        self.add_gcp_projects_error = None;
                    }

                    if !can_add && self.add_provider_type == "gcloud" {
                        if self.add_gcp_connected_email.is_none() {
                            ui.label("⚠ Connect to Google Cloud first");
                        } else if self.add_gcp_project_id.is_empty() {
                            ui.label("⚠ Project ID required");
                        }
                    }
                });
            });

        if !open {
            self.show_add_provider_dialog = false;
            self.add_gcp_oauth_result = None;
            self.add_gcp_oauth_promise = None;
            self.add_gcp_connected_email = None;
            self.add_gcp_project_id.clear();
            self.add_gcp_projects.clear();
            self.add_gcp_projects_loaded = false;
            self.add_gcp_projects_error = None;
        }
    }

    /// Show add domain dialog (for manually adding domains to providers)
    fn show_add_domain_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_add_domain_dialog {
            return;
        }

        // Load existing GCP accounts once when dialog opens
        if self.add_domain_existing_gcp_accounts.is_empty() {
            if let Ok(config) = load_ns_config() {
                self.add_domain_existing_gcp_accounts = config
                    .gcp_accounts
                    .iter()
                    .map(|acc| acc.connected_email.clone())
                    .collect();

                // Auto-select first account if available
                if !self.add_domain_existing_gcp_accounts.is_empty() {
                    self.add_domain_selected_gcp_account =
                        self.add_domain_existing_gcp_accounts[0].clone();
                }
            }
        }

        let mut open = true;
        egui::Window::new("Add Domain")
            .id(egui::Id::new("ns_add_domain_manual_dialog"))
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Domain name (primary domain only):");
                ui.text_edit_singleline(&mut self.add_domain);

                ui.add_space(8.0);
                ui.label("Provider:");
                egui::ComboBox::from_id_salt("domain_provider_combo")
                    .selected_text(match self.add_domain_provider.as_str() {
                        "cloudflare" => "Cloudflare",
                        "gcloud" => "Google Cloud DNS",
                        "duckdns" => "DuckDNS",
                        "porkbun" => "Porkbun",
                        _ => &self.add_domain_provider,
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.add_domain_provider,
                            "cloudflare".to_string(),
                            "Cloudflare",
                        );
                        ui.selectable_value(
                            &mut self.add_domain_provider,
                            "gcloud".to_string(),
                            "Google Cloud DNS",
                        );
                        ui.selectable_value(
                            &mut self.add_domain_provider,
                            "duckdns".to_string(),
                            "DuckDNS",
                        );
                        ui.selectable_value(
                            &mut self.add_domain_provider,
                            "porkbun".to_string(),
                            "Porkbun",
                        );
                    });

                // Show GCP account selector if gcloud is selected
                if self.add_domain_provider == "gcloud" {
                    ui.add_space(8.0);
                    if self.add_domain_existing_gcp_accounts.is_empty() {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 152, 0),
                            "⚠ No Google Cloud DNS accounts found",
                        );
                        ui.label("Please add a Google Cloud DNS provider first.");
                    } else {
                        ui.label("Google Account:");
                        egui::ComboBox::from_id_salt("gcp_account_combo")
                            .selected_text(&self.add_domain_selected_gcp_account)
                            .show_ui(ui, |ui| {
                                for account in &self.add_domain_existing_gcp_accounts {
                                    ui.selectable_value(
                                        &mut self.add_domain_selected_gcp_account,
                                        account.clone(),
                                        account,
                                    );
                                }
                            });
                    }
                }

                // Show warning for Porkbun
                if self.add_domain_provider == "porkbun" {
                    ui.add_space(8.0);
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 152, 0),
                        "⚠ For porkbun domain registration, visit porkbun.com.",
                    );
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 152, 0),
                        "   Not allowed with API.",
                    );
                }

                // Show format hint for DuckDNS
                if self.add_domain_provider == "duckdns" {
                    ui.add_space(8.0);
                    ui.colored_label(
                        egui::Color32::from_rgb(100, 149, 237),
                        "ℹ Format: xxx.duckdns.org",
                    );
                }

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    let can_add = !self.add_domain.is_empty()
                        && self.add_domain_provider != "porkbun"
                        && (self.add_domain_provider != "gcloud"
                            || !self.add_domain_existing_gcp_accounts.is_empty());
                    let add_button = if can_add {
                        MaterialButton::filled("Add")
                    } else {
                        MaterialButton::filled("Add").enabled(false)
                    };

                    if ui.add(add_button).clicked() {
                        self.execute_add_domain_manual();
                        self.show_add_domain_dialog = false;
                        // Reset GCP accounts list for next open
                        self.add_domain_existing_gcp_accounts.clear();
                    }

                    if ui.add(MaterialButton::text("Cancel")).clicked() {
                        self.show_add_domain_dialog = false;
                        // Reset GCP accounts list for next open
                        self.add_domain_existing_gcp_accounts.clear();
                    }
                });
            });

        if !open {
            self.show_add_domain_dialog = false;
            // Reset GCP accounts list for next open
            self.add_domain_existing_gcp_accounts.clear();
        }
    }

    /// Show add record dialog
    fn show_add_record_dialog(
        &mut self,
        ctx: &egui::Context,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        if !self.show_add_record_dialog {
            return;
        }

        let mut open = true;
        egui::Window::new("Add DNS Record")
            .id(egui::Id::new("ns_add_record_dialog"))
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Record name (e.g., www, @, mail):");
                ui.text_edit_singleline(&mut self.add_record_name);

                ui.add_space(8.0);
                ui.label("Record type:");
                egui::ComboBox::from_id_salt("record_type_combo")
                    .selected_text(self.add_record_type.to_uppercase())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.add_record_type, "a".to_string(), "A (IPv4)");
                        ui.selectable_value(
                            &mut self.add_record_type,
                            "aaaa".to_string(),
                            "AAAA (IPv6)",
                        );
                        ui.selectable_value(
                            &mut self.add_record_type,
                            "txt".to_string(),
                            "TXT (Text)",
                        );
                    });

                ui.add_space(8.0);
                ui.label("Value:");
                ui.text_edit_singleline(&mut self.add_record_value);

                ui.add_space(8.0);
                ui.checkbox(
                    &mut self.add_record_apply,
                    "Apply to DNS provider immediately",
                );

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.add(MaterialButton::filled("Add")).clicked() {
                        self.execute_add_record(vm);
                        self.show_add_record_dialog = false;
                    }

                    if ui.add(MaterialButton::text("Cancel")).clicked() {
                        self.show_add_record_dialog = false;
                    }
                });
            });

        if !open {
            self.show_add_record_dialog = false;
        }
    }

    /// Show error dialog
    fn show_error_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_error_dialog {
            return;
        }

        let mut open = true;
        egui::Window::new("Error")
            .id(egui::Id::new("ns_error_dialog"))
            .open(&mut open)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(&self.error_message);

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.add(MaterialButton::filled("OK")).clicked() {
                        self.show_error_dialog = false;
                    }
                });
            });

        if !open {
            self.show_error_dialog = false;
        }
    }

    /// Start add nameserver provider in background (non-blocking)
    fn start_add_provider_background(&mut self, mut vm: Option<&mut crate::viewmodel::ViewModel>) {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            let provider = self.add_provider_type.clone();

            // Use ViewModel for Cloudflare and Porkbun (simpler providers)
            if (provider == "cloudflare" || provider == "porkbun") && vm.is_some() {
                let api_token = if provider == "porkbun" {
                    if self.add_token.is_empty() || self.add_secret_key.is_empty() {
                        self.add_progress(
                            "Error: Both API Key and Secret Key are required for Porkbun"
                                .to_string(),
                        );
                        return;
                    }
                    format!("{}::{}", self.add_token, self.add_secret_key)
                } else {
                    if self.add_token.is_empty() {
                        self.add_progress("Error: API Key is required".to_string());
                        return;
                    }
                    self.add_token.clone()
                };

                self.add_progress(format!("Fetching domains from {}...", provider));

                if let Some(ref mut vm) = vm {
                    if let Err(e) =
                        vm.add_dns_provider(provider.clone(), provider.clone(), api_token)
                    {
                        self.add_progress(format!("Failed to start provider addition: {}", e));
                    }
                }
                return;
            }

            // Fallback to poll_promise for DuckDNS and GCP
            let (provider_id, token) = if provider == "gcloud" {
                if let (Some(oauth), Some(email)) =
                    (&self.add_gcp_oauth_result, &self.add_gcp_connected_email)
                {
                    // Clone values to avoid borrow issues
                    let email = email.clone();
                    let oauth = oauth.clone();
                    let project_id = self.add_gcp_project_id.clone();

                    if project_id.is_empty() {
                        self.add_progress("Error: Project ID is required for GCP".to_string());
                        return;
                    }

                    // Create GCP account
                    use crate::calc::ns::GcpAccount;
                    let account = GcpAccount {
                        access_token: oauth.access_token.clone(),
                        refresh_token: oauth.refresh_token.clone(),
                        token_expiry: oauth.expires_at,
                        connected_email: email.clone(),
                        project_id: project_id.clone(),
                        domains: Vec::new(),
                    };

                    match load_ns_config() {
                        Ok(mut config) => {
                            // Add account if doesn't exist
                            if config.get_gcp_account(&email).is_none() {
                                if let Err(e) = config.add_gcp_account(account) {
                                    self.add_progress(format!(
                                        "Error: Failed to add GCP account: {}",
                                        e
                                    ));
                                    return;
                                }

                                if let Err(e) = save_ns_config(&config) {
                                    self.add_progress(format!(
                                        "Error: Failed to save config: {}",
                                        e
                                    ));
                                    return;
                                }

                                self.add_progress(format!("✓ Added GCP account: {}", email));
                            }
                        }
                        Err(e) => {
                            self.add_progress(format!("Error: Failed to load config: {}", e));
                            return;
                        }
                    }

                    let provider_id = format!("gcloud:{}", email);
                    let token = format!("{}::{}", oauth.access_token, project_id);
                    (provider_id, token)
                } else {
                    self.add_progress("Error: OAuth connection required for GCP".to_string());
                    return;
                }
            } else if provider == "porkbun" {
                if self.add_token.is_empty() || self.add_secret_key.is_empty() {
                    self.add_progress(
                        "Error: Both API Key and Secret Key are required for Porkbun".to_string(),
                    );
                    return;
                }
                let token = format!("{}::{}", self.add_token, self.add_secret_key);
                (provider, token)
            } else {
                if self.add_token.is_empty() {
                    self.add_progress("Error: API Key is required".to_string());
                    return;
                }
                (provider, self.add_token.clone())
            };

            self.add_progress(format!(
                "Starting background task to fetch domains from {}...",
                provider_id
            ));

            // Spawn background task
            let promise = Promise::spawn_thread("add-provider", move || {
                execute_add_provider_blocking(provider_id, token)
            });

            self.add_provider_promise = Some(promise);
        }
    }

    /// Execute add nameserver provider (fetches domains from provider API)
    fn execute_add_provider(&mut self) {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            use crate::api::{ns_cloudflare, ns_porkbun};

            let provider = self.add_provider_type.clone();

            // For Porkbun, combine API key and secret key
            let token = if provider == "porkbun" {
                if self.add_token.is_empty() || self.add_secret_key.is_empty() {
                    self.add_progress(
                        "Error: Both API Key and Secret Key are required for Porkbun".to_string(),
                    );
                    return;
                }
                // Store as "apikey::secretkey"
                format!("{}::{}", self.add_token, self.add_secret_key)
            } else {
                if self.add_token.is_empty() {
                    self.add_progress("Error: API Key is required".to_string());
                    return;
                }
                self.add_token.clone()
            };

            self.add_progress(format!("Fetching domains from {}...", provider));

            // Fetch domains from provider API (except Porkbun per requirements)
            match provider.as_str() {
                "cloudflare" => {
                    self.add_progress("Calling Cloudflare API...".to_string());
                    let client = ns_cloudflare::CloudflareClient::new(token.clone());
                    match client.list_zones() {
                        Ok(zones) => {
                            self.add_progress(format!(
                                "API call successful, found {} zones",
                                zones.len()
                            ));

                            // Fetch DNS records for each zone and add them to config
                            match load_ns_config() {
                                Ok(mut config) => {
                                    self.add_progress(format!(
                                        "Loaded config with {} existing domains",
                                        config.total_domains()
                                    ));
                                    let mut added_count = 0;

                                    for zone in zones {
                                        self.add_progress(format!(
                                            "  Processing zone: {}",
                                            zone.name
                                        ));

                                        // Fetch DNS records for this zone
                                        match client.get_records(&zone.id) {
                                            Ok(records) => {
                                                // Filter for A, AAAA, TXT records only
                                                let filtered_records: Vec<_> = records
                                                    .iter()
                                                    .filter(|r| {
                                                        let rt = r.record_type.to_uppercase();
                                                        rt == "A" || rt == "AAAA" || rt == "TXT"
                                                    })
                                                    .collect();

                                                self.add_progress(format!(
                                                    "    Found {} DNS records (A/AAAA/TXT)",
                                                    filtered_records.len()
                                                ));

                                                // Add or update domain
                                                match config.add_domain(
                                                    provider.clone(),
                                                    zone.name.clone(),
                                                    token.clone(),
                                                ) {
                                                    Ok(_) => {
                                                        added_count += 1;

                                                        // Add DNS records
                                                        if let Some(domain_entry) = config
                                                            .get_domain_mut(&provider, &zone.name)
                                                        {
                                                            for record in filtered_records {
                                                                let rec_type = RecordType::from_str(
                                                                    &record
                                                                        .record_type
                                                                        .to_lowercase(),
                                                                );
                                                                if let Some(rt) = rec_type {
                                                                    domain_entry.records.push(crate::calc::ns::DnsRecord {
                                                                        record_type: rt,
                                                                        name: record.name.clone(),
                                                                        value: record.content.clone(),
                                                                        ttl: Some(record.ttl),
                                                                    });
                                                                }
                                                            }
                                                            self.add_progress(format!(
                                                                "    ✓ Added {} with {} records",
                                                                zone.name,
                                                                domain_entry.records.len()
                                                            ));
                                                        }
                                                    }
                                                    Err(e) => {
                                                        self.add_progress(format!(
                                                            "    ⚠ Skipped {}: {}",
                                                            zone.name, e
                                                        ));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                self.add_progress(format!(
                                                    "    ⚠ Failed to fetch records for {}: {}",
                                                    zone.name, e
                                                ));
                                                // Still add the domain even if records fetch fails
                                                let _ = config.add_domain(
                                                    provider.clone(),
                                                    zone.name.clone(),
                                                    token.clone(),
                                                );
                                            }
                                        }
                                    }

                                    self.add_progress(format!(
                                        "Total in config now: {} entries",
                                        config.total_domains()
                                    ));

                                    if added_count > 0 {
                                        self.add_progress("Saving config...".to_string());
                                        match save_ns_config(&config) {
                                            Ok(_) => {
                                                self.add_progress(
                                                    "✓ Config saved successfully".to_string(),
                                                );

                                                // Record audit event
                                                let _ = audit::push_gui(
                                                    "system",
                                                    "desktop",
                                                    "ns add provider",
                                                    &provider,
                                                );

                                                self.add_progress(format!(
                                                    "✓ Added {} provider with {} entries",
                                                    provider, added_count
                                                ));

                                                // Reload the data
                                                self.add_progress(
                                                    "Reloading UI data...".to_string(),
                                                );
                                                self.load_data();
                                                self.add_progress("✓ UI refreshed".to_string());
                                            }
                                            Err(e) => {
                                                self.add_progress(format!(
                                                    "❌ Error saving config: {}",
                                                    e
                                                ));
                                            }
                                        }
                                    } else {
                                        self.add_progress(
                                            "⚠ No new entries added (all already exist)"
                                                .to_string(),
                                        );
                                    }
                                }
                                Err(e) => {
                                    self.add_progress(format!("❌ Error loading config: {}", e));
                                }
                            }

                            return; // Exit early since we handled everything above
                        }
                        Err(e) => {
                            self.add_progress(format!("❌ Error fetching domains: {}", e));
                            self.add_progress(format!("  Error details: {:?}", e));
                            return;
                        }
                    }
                }
                "duckdns" => {
                    self.add_progress(
                        "DuckDNS domains cannot be auto-discovered. Saving provider for manual domain addition."
                            .to_string(),
                    );

                    // Add provider entry with placeholder so token is saved
                    match load_ns_config() {
                        Ok(mut config) => {
                            let placeholder = format!("{} (provider)", provider);
                            match config.add_domain(
                                provider.clone(),
                                placeholder.clone(),
                                token.clone(),
                            ) {
                                Ok(_) => match save_ns_config(&config) {
                                    Ok(_) => {
                                        let _ = audit::push_gui(
                                            "system",
                                            "desktop",
                                            "ns add provider",
                                            &provider,
                                        );
                                        self.add_progress(format!(
                                            "✓ Added {} provider (add domains manually)",
                                            provider
                                        ));
                                        self.load_data();
                                    }
                                    Err(e) => {
                                        self.add_progress(format!("❌ Error saving config: {}", e));
                                    }
                                },
                                Err(e) => {
                                    self.add_progress(format!("⚠ Failed to add provider: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            self.add_progress(format!("❌ Error loading config: {}", e));
                        }
                    }
                    return;
                }
                "gcloud" => {
                    self.add_progress(
                        "GCP requires project ID. Saving provider for manual domain addition."
                            .to_string(),
                    );

                    // Add provider entry with placeholder so token is saved
                    match load_ns_config() {
                        Ok(mut config) => {
                            let placeholder = format!("{} (provider)", provider);
                            match config.add_domain(
                                provider.clone(),
                                placeholder.clone(),
                                token.clone(),
                            ) {
                                Ok(_) => match save_ns_config(&config) {
                                    Ok(_) => {
                                        let _ = audit::push_gui(
                                            "system",
                                            "desktop",
                                            "ns add provider",
                                            &provider,
                                        );
                                        self.add_progress(format!(
                                            "✓ Added {} provider (add domains manually)",
                                            provider
                                        ));
                                        self.load_data();
                                    }
                                    Err(e) => {
                                        self.add_progress(format!("❌ Error saving config: {}", e));
                                    }
                                },
                                Err(e) => {
                                    self.add_progress(format!("⚠ Failed to add provider: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            self.add_progress(format!("❌ Error loading config: {}", e));
                        }
                    }
                    return;
                }
                "porkbun" => {
                    self.add_progress("Calling Porkbun API...".to_string());

                    // Parse API key and secret key from combined token
                    let (api_key, secret_key) = match parse_porkbun_credentials(&token) {
                        Some((k, s)) => (k, s),
                        None => {
                            self.add_progress(
                                "❌ Error: Invalid Porkbun credentials format".to_string(),
                            );
                            return;
                        }
                    };

                    let client = ns_porkbun::PorkbunClient::new(api_key, secret_key);

                    // Fetch all domains
                    match client.list_domains() {
                        Ok(domains) => {
                            self.add_progress(format!(
                                "API call successful, found {} domains",
                                domains.len()
                            ));

                            // Fetch DNS records for each domain and add them to config
                            match load_ns_config() {
                                Ok(mut config) => {
                                    self.add_progress(format!(
                                        "Loaded config with {} existing domains",
                                        config.total_domains()
                                    ));
                                    let mut added_count = 0;

                                    for domain in domains {
                                        self.add_progress(format!(
                                            "  Processing domain: {}",
                                            domain
                                        ));

                                        // Fetch DNS records for this domain
                                        match client.get_records(&domain) {
                                            Ok(records) => {
                                                // Filter for A, AAAA, TXT records only
                                                let filtered_records: Vec<_> = records
                                                    .iter()
                                                    .filter(|r| {
                                                        let rt = r.record_type.to_uppercase();
                                                        rt == "A" || rt == "AAAA" || rt == "TXT"
                                                    })
                                                    .collect();

                                                self.add_progress(format!(
                                                    "    Found {} DNS records (A/AAAA/TXT)",
                                                    filtered_records.len()
                                                ));

                                                // Add or update domain
                                                match config.add_domain(
                                                    provider.clone(),
                                                    domain.clone(),
                                                    token.clone(),
                                                ) {
                                                    Ok(_) => {
                                                        added_count += 1;

                                                        // Add DNS records
                                                        if let Some(domain_entry) = config
                                                            .get_domain_mut(&provider, &domain)
                                                        {
                                                            for record in filtered_records {
                                                                let rec_type = RecordType::from_str(
                                                                    &record
                                                                        .record_type
                                                                        .to_lowercase(),
                                                                );
                                                                if let Some(rt) = rec_type {
                                                                    domain_entry.records.push(crate::calc::ns::DnsRecord {
                                                                        record_type: rt,
                                                                        name: record.name.clone(),
                                                                        value: record.content.clone(),
                                                                        ttl: record.ttl.parse().ok(),
                                                                    });
                                                                }
                                                            }
                                                            self.add_progress(format!(
                                                                "    ✓ Added {} with {} records",
                                                                domain,
                                                                domain_entry.records.len()
                                                            ));
                                                        }
                                                    }
                                                    Err(e) => {
                                                        self.add_progress(format!(
                                                            "    ⚠ Skipped {}: {}",
                                                            domain, e
                                                        ));
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                self.add_progress(format!(
                                                    "    ⚠ Failed to fetch records for {}: {}",
                                                    domain, e
                                                ));
                                                // Still add the domain even if records fetch fails
                                                let _ = config.add_domain(
                                                    provider.clone(),
                                                    domain.clone(),
                                                    token.clone(),
                                                );
                                            }
                                        }
                                    }

                                    self.add_progress(format!(
                                        "Total in config now: {} entries",
                                        config.total_domains()
                                    ));

                                    if added_count > 0 {
                                        self.add_progress("Saving config...".to_string());
                                        match save_ns_config(&config) {
                                            Ok(_) => {
                                                self.add_progress(
                                                    "✓ Config saved successfully".to_string(),
                                                );

                                                // Record audit event
                                                let _ = audit::push_gui(
                                                    "system",
                                                    "desktop",
                                                    "ns add provider",
                                                    &provider,
                                                );

                                                self.add_progress(format!(
                                                    "✓ Added {} provider with {} entries",
                                                    provider, added_count
                                                ));

                                                // Reload the data
                                                self.add_progress(
                                                    "Reloading UI data...".to_string(),
                                                );
                                                self.load_data();
                                                self.add_progress("✓ UI refreshed".to_string());
                                            }
                                            Err(e) => {
                                                self.add_progress(format!(
                                                    "❌ Error saving config: {}",
                                                    e
                                                ));
                                            }
                                        }
                                    } else {
                                        self.add_progress(
                                            "⚠ No new entries added (all already exist)"
                                                .to_string(),
                                        );
                                    }
                                }
                                Err(e) => {
                                    self.add_progress(format!("❌ Error loading config: {}", e));
                                }
                            }

                            return; // Exit early since we handled everything above
                        }
                        Err(e) => {
                            self.add_progress(format!("❌ Error fetching domains: {}", e));
                            self.add_progress(format!("  Error details: {:?}", e));
                            return;
                        }
                    }
                }
                _ => {
                    self.add_progress(format!("Error: Unknown provider '{}'", provider));
                }
            }
        }
    }

    /// Execute add domain manually
    fn execute_add_domain_manual(&mut self) {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            use crate::calc::dns;

            let domain = self.add_domain.trim().to_string();
            let provider = self.add_domain_provider.clone();

            if domain.is_empty() {
                self.error_message = "Domain name is required".to_string();
                self.show_error_dialog = true;
                return;
            }

            // Validate DuckDNS format
            if provider == "duckdns" && !domain.ends_with(".duckdns.org") {
                self.error_message =
                    "DuckDNS domains must be in the format: xxx.duckdns.org".to_string();
                self.show_error_dialog = true;
                return;
            }

            // Find existing provider token from config
            match load_ns_config() {
                Ok(mut config) => {
                    // For GCP: use existing account or add new one
                    let provider = if provider == "gcloud" {
                        // First check if using existing account
                        if !self.add_domain_selected_gcp_account.is_empty() {
                            // Using existing account
                            let email = self.add_domain_selected_gcp_account.clone();

                            // Verify account still exists in config
                            if config.get_gcp_account(&email).is_none() {
                                self.error_message =
                                    format!("Selected account {} not found in config", email);
                                self.show_error_dialog = true;
                                return;
                            }

                            // Use full provider ID
                            format!("gcloud:{}", email)
                        } else if let (Some(oauth), Some(email)) =
                            (&self.add_gcp_oauth_result, &self.add_gcp_connected_email)
                        {
                            // Adding new account via OAuth (from add provider flow)
                            // Clone values needed after borrow
                            let email = email.clone();
                            let oauth = oauth.clone();
                            let project_id = self.add_gcp_project_id.clone();

                            if project_id.is_empty() {
                                self.error_message =
                                    "Please select a GCP project first".to_string();
                                self.show_error_dialog = true;
                                return;
                            }

                            // Add GCP account if it doesn't exist
                            if config.get_gcp_account(&email).is_none() {
                                use crate::calc::ns::GcpAccount;
                                let account = GcpAccount {
                                    access_token: oauth.access_token.clone(),
                                    refresh_token: oauth.refresh_token.clone(),
                                    token_expiry: oauth.expires_at,
                                    connected_email: email.clone(),
                                    project_id: project_id.clone(),
                                    domains: Vec::new(),
                                };

                                if let Err(e) = config.add_gcp_account(account) {
                                    self.error_message =
                                        format!("Failed to add GCP account: {}", e);
                                    self.show_error_dialog = true;
                                    return;
                                }

                                self.add_progress(format!("✓ Added GCP account: {}", email));
                            }

                            // Use full provider ID
                            format!("gcloud:{}", email)
                        } else {
                            self.error_message = "Please select an existing Google Cloud account or add a new provider first".to_string();
                            self.show_error_dialog = true;
                            return;
                        }
                    } else {
                        provider.clone()
                    };

                    // Check if we already have a token for this provider (with refresh for GCP)
                    let existing_token = config.get_api_token_refreshed(&provider);

                    let token = if let Some(tok) = existing_token {
                        tok
                    } else {
                        // No existing provider, need to prompt for token
                        self.add_progress(format!(
                            "Error: No {} provider configured. Add provider first.",
                            provider
                        ));
                        return;
                    };

                    // Save config to persist refreshed token
                    if provider.starts_with("gcloud:") {
                        if let Err(e) = save_ns_config(&config) {
                            dure_debug!("Warning: Failed to save refreshed token: {}", e);
                        }
                    }

                    // Remove placeholder entries if they exist
                    // GCP placeholder format: "gcloud (project-id)"
                    if provider.starts_with("gcloud:") {
                        // Extract email from provider (format: "gcloud:email")
                        let email = &provider[7..]; // Skip "gcloud:" prefix
                        if let Some(account) = config.get_gcp_account(email) {
                            let gcp_placeholder = format!("gcloud ({})", account.project_id);
                            if config.get_domain(&provider, &gcp_placeholder).is_some() {
                                match config.remove_domain(&provider, &gcp_placeholder) {
                                    Ok(_) => {
                                        self.add_progress(format!(
                                            "Removed placeholder: {}",
                                            gcp_placeholder
                                        ));
                                    }
                                    Err(e) => {
                                        self.add_progress(format!(
                                            "Warning: Failed to remove placeholder: {}",
                                            e
                                        ));
                                    }
                                }
                            }
                        }
                    } else {
                        // Regular provider placeholder format: "provider (provider)"
                        let placeholder = format!("{} (provider)", provider);
                        if config.get_domain(&provider, &placeholder).is_some() {
                            match config.remove_domain(&provider, &placeholder) {
                                Ok(_) => {
                                    self.add_progress(format!(
                                        "Removed placeholder for {}",
                                        provider
                                    ));
                                }
                                Err(e) => {
                                    self.add_progress(format!(
                                        "Warning: Failed to remove placeholder: {}",
                                        e
                                    ));
                                }
                            }
                        }
                    }

                    // Add the domain
                    match config.add_domain(provider.clone(), domain.clone(), token.clone()) {
                        Ok(_) => {
                            // For DuckDNS, fetch DNS records using DoH
                            if provider == "duckdns" {
                                self.add_progress(format!(
                                    "Fetching DNS records for {} via DoH...",
                                    domain
                                ));

                                let mut found_any_records = false;

                                // Fetch A, AAAA, TXT records
                                for record_type in [
                                    dns::RecordType::A,
                                    dns::RecordType::AAAA,
                                    dns::RecordType::TXT,
                                ] {
                                    match dns::resolve_dns(&domain, record_type) {
                                        Ok(records) => {
                                            if !records.is_empty() {
                                                found_any_records = true;
                                                self.add_progress(format!(
                                                    "  Found {} {} records",
                                                    records.len(),
                                                    record_type.as_str()
                                                ));

                                                // Add records to config
                                                if let Some(domain_entry) =
                                                    config.get_domain_mut(&provider, &domain)
                                                {
                                                    for dns_record in records {
                                                        // Convert dns::RecordType to calc::ns::RecordType
                                                        if let Some(ns_record_type) =
                                                            RecordType::from_str(
                                                                &dns_record
                                                                    .record_type
                                                                    .as_str()
                                                                    .to_lowercase(),
                                                            )
                                                        {
                                                            domain_entry.records.push(
                                                                crate::calc::ns::DnsRecord {
                                                                    record_type: ns_record_type,
                                                                    name: dns_record.domain.clone(),
                                                                    value: dns_record.value.clone(),
                                                                    ttl: Some(dns_record.ttl),
                                                                },
                                                            );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            self.add_progress(format!(
                                                "  ⚠ Failed to fetch {} records: {}",
                                                record_type.as_str(),
                                                e
                                            ));
                                        }
                                    }
                                }

                                // If no records found, add empty placeholders
                                if !found_any_records {
                                    self.add_progress(
                                        "  No DNS records found, adding empty placeholders"
                                            .to_string(),
                                    );
                                    if let Some(domain_entry) =
                                        config.get_domain_mut(&provider, &domain)
                                    {
                                        // Add empty A record
                                        domain_entry.records.push(crate::calc::ns::DnsRecord {
                                            record_type: RecordType::A,
                                            name: String::new(),
                                            value: String::new(),
                                            ttl: None,
                                        });
                                    }
                                }
                            }

                            // For Cloudflare, create zone if it doesn't exist
                            if provider == "cloudflare" {
                                use crate::api::ns_cloudflare::CloudflareClient;

                                self.add_progress(format!(
                                    "Creating Cloudflare zone for {}...",
                                    domain
                                ));
                                let client = CloudflareClient::new(token.clone());

                                match client.create_zone(&domain) {
                                    Ok(zone) => {
                                        self.add_progress(format!("✓ Zone created: {}", zone.name));
                                        if !zone.name_servers.is_empty() {
                                            self.add_progress(format!(
                                                "  Nameservers: {}",
                                                zone.name_servers.join(", ")
                                            ));
                                        }
                                    }
                                    Err(e) => {
                                        let error_str = e.to_string();
                                        // If zone already exists, that's okay
                                        if error_str.contains("already exists")
                                            || error_str.contains("1061")
                                        {
                                            self.add_progress("  Zone already exists".to_string());
                                        } else if error_str.contains("403")
                                            || error_str.contains("zone.create")
                                        {
                                            self.error_message = format!(
                                                "Cloudflare API token doesn't have zone creation permission.\n\n\
                                                Please create a new API token at:\n\
                                                https://dash.cloudflare.com/profile/api-tokens\n\n\
                                                Required permissions:\n\
                                                • Zone - Zone - Edit\n\
                                                • Zone - DNS - Edit\n\n\
                                                Error: {}",
                                                e
                                            );
                                            self.show_error_dialog = true;
                                            return;
                                        } else {
                                            self.error_message = format!(
                                                "Failed to create Cloudflare zone:\n\n{}",
                                                e
                                            );
                                            self.show_error_dialog = true;
                                            return;
                                        }
                                    }
                                }
                            }

                            // For GCP, create managed zone with DNSSEC
                            if provider.starts_with("gcloud") {
                                use crate::api::ns_gcp::GcpDnsClient;

                                // Parse GCP token (format: access_token::project_id)
                                let parts: Vec<&str> = token.split("::").collect();
                                if parts.len() == 2 {
                                    let access_token = parts[0].to_string();
                                    let project_id = parts[1].to_string();

                                    self.add_progress(format!(
                                        "Creating GCP managed zone for {}...",
                                        domain
                                    ));
                                    let client = GcpDnsClient::new(access_token);

                                    match client.create_managed_zone(&project_id, &domain) {
                                        Ok(zone) => {
                                            let zone_name = domain.replace('.', "-");
                                            self.add_progress(format!(
                                                "✓ Zone created: {} (DNSSEC enabled)",
                                                zone_name
                                            ));
                                            if !zone.name_servers.is_empty() {
                                                self.add_progress(format!(
                                                    "  Nameservers: {}",
                                                    zone.name_servers.join(", ")
                                                ));
                                            }
                                        }
                                        Err(e) => {
                                            // If zone already exists, that's okay
                                            if e.to_string().contains("already exists")
                                                || e.to_string().contains("409")
                                            {
                                                self.add_progress(
                                                    "  Zone already exists".to_string(),
                                                );
                                            } else {
                                                self.error_message = format!(
                                                    "Failed to create GCP managed zone:\n\n{}",
                                                    e
                                                );
                                                self.show_error_dialog = true;
                                                return;
                                            }
                                        }
                                    }
                                } else {
                                    self.error_message = "Invalid GCP token format. Expected: access_token::project_id".to_string();
                                    self.show_error_dialog = true;
                                    return;
                                }
                            }

                            match save_ns_config(&config) {
                                Ok(_) => {
                                    // Record audit event
                                    let _ = audit::push_gui("system", "desktop", "ns add", &domain);

                                    let record_count = config
                                        .get_domain(&provider, &domain)
                                        .map(|d| d.records.len())
                                        .unwrap_or(0);

                                    self.add_progress(format!(
                                        "✓ Added domain: {} ({}) with {} records",
                                        domain, provider, record_count
                                    ));
                                    self.load_data();
                                }
                                Err(e) => {
                                    self.add_progress(format!("Error saving config: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            self.add_progress(format!("Error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.add_progress(format!("Error loading config: {}", e));
                }
            }
        }
    }

    /// Refresh domains and records from API
    fn refresh_from_api(&mut self) {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            use crate::api::{ns_cloudflare, ns_porkbun};
            use crate::calc::dns;

            self.add_progress("Refreshing from API...".to_string());

            match load_ns_config() {
                Ok(mut config) => {
                    let mut updated_count = 0;

                    // Refresh each provider
                    for (provider_name, provider_config) in config.providers.clone().iter() {
                        if provider_config.domains.is_empty() {
                            continue;
                        }

                        let token = provider_config.api_token.clone();
                        let provider = provider_name.clone();

                        self.add_progress(format!("Refreshing {} domains...", provider));

                        match provider.as_str() {
                            "cloudflare" => {
                                let client = ns_cloudflare::CloudflareClient::new(token);

                                // Fetch all zones
                                match client.list_zones() {
                                    Ok(zones) => {
                                        for zone in zones {
                                            // Find corresponding domain in config
                                            if let Some(domain_entry) =
                                                config.get_domain_mut(&provider, &zone.name)
                                            {
                                                // Fetch DNS records
                                                match client.get_records(&zone.id) {
                                                    Ok(records) => {
                                                        // Clear existing records and add fresh ones
                                                        domain_entry.records.clear();

                                                        for record in records {
                                                            let rt =
                                                                record.record_type.to_uppercase();
                                                            if rt == "A"
                                                                || rt == "AAAA"
                                                                || rt == "TXT"
                                                            {
                                                                if let Some(rec_type) =
                                                                    RecordType::from_str(
                                                                        &rt.to_lowercase(),
                                                                    )
                                                                {
                                                                    // For Cloudflare, keep full FQDN as returned by API
                                                                    domain_entry.records.push(
                                                                        crate::calc::ns::DnsRecord {
                                                                            record_type: rec_type,
                                                                            name: record.name.clone(),
                                                                            value: record.content.clone(),
                                                                            ttl: Some(record.ttl),
                                                                        },
                                                                    );
                                                                }
                                                            }
                                                        }

                                                        updated_count += 1;
                                                        self.add_progress(format!(
                                                            "  ✓ {} - {} records",
                                                            zone.name,
                                                            domain_entry.records.len()
                                                        ));
                                                    }
                                                    Err(e) => {
                                                        self.add_progress(format!(
                                                            "  ⚠ Failed to fetch records for {}: {}",
                                                            zone.name, e
                                                        ));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        self.add_progress(format!(
                                            "  ⚠ Failed to fetch zones: {}",
                                            e
                                        ));
                                    }
                                }
                            }
                            "duckdns" => {
                                // For DuckDNS, refresh records using DoH for each domain
                                for domain_entry in provider_config.domains.iter() {
                                    let domain = domain_entry.domain.clone();

                                    // Skip placeholder entries
                                    if domain.ends_with("(provider)") {
                                        continue;
                                    }

                                    self.add_progress(format!(
                                        "  Fetching records for {} via DoH...",
                                        domain
                                    ));

                                    if let Some(domain_entry) =
                                        config.get_domain_mut("duckdns", &domain)
                                    {
                                        // Clear existing records
                                        domain_entry.records.clear();

                                        let mut found_any_records = false;

                                        // Fetch A, AAAA, TXT records
                                        for record_type in [
                                            dns::RecordType::A,
                                            dns::RecordType::AAAA,
                                            dns::RecordType::TXT,
                                        ] {
                                            match dns::resolve_dns(&domain, record_type) {
                                                Ok(records) => {
                                                    if !records.is_empty() {
                                                        found_any_records = true;
                                                        for dns_record in records {
                                                            if let Some(ns_record_type) =
                                                                RecordType::from_str(
                                                                    &dns_record
                                                                        .record_type
                                                                        .as_str()
                                                                        .to_lowercase(),
                                                                )
                                                            {
                                                                domain_entry.records.push(
                                                                    crate::calc::ns::DnsRecord {
                                                                        record_type: ns_record_type,
                                                                        name: dns_record
                                                                            .domain
                                                                            .clone(),
                                                                        value: dns_record
                                                                            .value
                                                                            .clone(),
                                                                        ttl: Some(dns_record.ttl),
                                                                    },
                                                                );
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(e) => {
                                                    self.add_progress(format!(
                                                        "    ⚠ Failed to fetch {} records: {}",
                                                        record_type.as_str(),
                                                        e
                                                    ));
                                                }
                                            }
                                        }

                                        // If no records found, add empty placeholder
                                        if !found_any_records {
                                            self.add_progress(format!(
                                                "    No records found, adding empty placeholder"
                                            ));
                                            domain_entry.records.push(crate::calc::ns::DnsRecord {
                                                record_type: RecordType::A,
                                                name: String::new(),
                                                value: String::new(),
                                                ttl: None,
                                            });
                                        }

                                        updated_count += 1;
                                        self.add_progress(format!(
                                            "  ✓ {} - {} records",
                                            domain,
                                            domain_entry.records.len()
                                        ));
                                    }
                                }
                            }
                            "porkbun" => {
                                // Parse Porkbun credentials
                                let (api_key, secret_key) = match parse_porkbun_credentials(&token)
                                {
                                    Some((k, s)) => (k, s),
                                    None => {
                                        self.add_progress(format!(
                                            "  ⚠ Invalid Porkbun credentials format"
                                        ));
                                        continue;
                                    }
                                };

                                let client = ns_porkbun::PorkbunClient::new(api_key, secret_key);

                                // For Porkbun, refresh records for each domain
                                for domain_entry in provider_config.domains.iter() {
                                    let domain = domain_entry.domain.clone();

                                    // Skip placeholder entries
                                    if domain.ends_with("(provider)") {
                                        continue;
                                    }

                                    self.add_progress(format!(
                                        "  Fetching records for {}...",
                                        domain
                                    ));

                                    if let Some(domain_entry) =
                                        config.get_domain_mut("porkbun", &domain)
                                    {
                                        // Fetch DNS records
                                        match client.get_records(&domain) {
                                            Ok(records) => {
                                                // Clear existing records and add fresh ones
                                                domain_entry.records.clear();

                                                for record in records {
                                                    let rt = record.record_type.to_uppercase();
                                                    if rt == "A" || rt == "AAAA" || rt == "TXT" {
                                                        if let Some(rec_type) =
                                                            RecordType::from_str(&rt.to_lowercase())
                                                        {
                                                            domain_entry.records.push(
                                                                crate::calc::ns::DnsRecord {
                                                                    record_type: rec_type,
                                                                    name: record.name.clone(),
                                                                    value: record.content.clone(),
                                                                    ttl: record.ttl.parse().ok(),
                                                                },
                                                            );
                                                        }
                                                    }
                                                }

                                                updated_count += 1;
                                                self.add_progress(format!(
                                                    "  ✓ {} - {} records",
                                                    domain,
                                                    domain_entry.records.len()
                                                ));
                                            }
                                            Err(e) => {
                                                self.add_progress(format!(
                                                    "  ⚠ Failed to fetch records for {}: {}",
                                                    domain, e
                                                ));
                                            }
                                        }
                                    }
                                }
                            }
                            "gcloud" => {
                                // Parse GCP credentials (access_token::project_id)
                                let parts: Vec<&str> = token.split("::").collect();
                                if parts.len() != 2 {
                                    self.add_progress(format!("  ⚠ Invalid GCP token format"));
                                    continue;
                                }

                                let access_token = parts[0].to_string();
                                let project_id = parts[1].to_string();

                                use crate::api::ns_gcp;
                                let client = ns_gcp::GcpDnsClient::new(access_token);

                                // Refresh records for each domain
                                for domain_entry in provider_config.domains.iter() {
                                    let domain = domain_entry.domain.clone();

                                    // Skip placeholder entries
                                    if domain.ends_with("(provider)") {
                                        continue;
                                    }

                                    self.add_progress(format!("  Fetching zone for {}...", domain));

                                    // Find the managed zone for this domain
                                    match client.find_zone_by_domain(&project_id, &domain) {
                                        Ok(Some(zone)) => {
                                            self.add_progress(format!(
                                                "    Found zone: {}",
                                                zone.name
                                            ));

                                            // Fetch resource record sets
                                            match client.list_rrsets(&project_id, &zone.name) {
                                                Ok(rrsets) => {
                                                    if let Some(domain_entry) =
                                                        config.get_domain_mut("gcloud", &domain)
                                                    {
                                                        // Clear existing records
                                                        domain_entry.records.clear();

                                                        // Add fresh records
                                                        for rrset in rrsets {
                                                            let rt =
                                                                rrset.record_type.to_uppercase();
                                                            if rt == "A"
                                                                || rt == "AAAA"
                                                                || rt == "TXT"
                                                            {
                                                                if let Some(rec_type) =
                                                                    RecordType::from_str(
                                                                        &rt.to_lowercase(),
                                                                    )
                                                                {
                                                                    // GCP returns multiple values in rrdatas array
                                                                    for value in &rrset.rrdatas {
                                                                        domain_entry.records.push(
                                                                            crate::calc::ns::DnsRecord {
                                                                                record_type: rec_type.clone(),
                                                                                name: rrset.name.trim_end_matches('.').to_string(),
                                                                                value: value.clone(),
                                                                                ttl: Some(rrset.ttl),
                                                                            },
                                                                        );
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        updated_count += 1;
                                                        self.add_progress(format!(
                                                            "  ✓ {} - {} records",
                                                            domain,
                                                            domain_entry.records.len()
                                                        ));
                                                    }
                                                }
                                                Err(e) => {
                                                    self.add_progress(format!(
                                                        "  ⚠ Failed to fetch records: {}",
                                                        e
                                                    ));
                                                }
                                            }
                                        }
                                        Ok(None) => {
                                            self.add_progress(format!(
                                                "  ⚠ No managed zone found for {}",
                                                domain
                                            ));
                                        }
                                        Err(e) => {
                                            self.add_progress(format!(
                                                "  ⚠ Failed to find zone for {}: {}",
                                                domain, e
                                            ));
                                        }
                                    }
                                }
                            }
                            _ => {
                                self.add_progress(format!(
                                    "  ⚠ Refresh not implemented for {}",
                                    provider
                                ));
                            }
                        }
                    }

                    if updated_count > 0 {
                        // Save updated config
                        match save_ns_config(&config) {
                            Ok(_) => {
                                self.add_progress(format!("✓ Refreshed {} domains", updated_count));
                                self.load_data();
                                // Also reload records if a domain is selected
                                if self.selected_domain.is_some() {
                                    self.load_records();
                                }
                            }
                            Err(e) => {
                                self.add_progress(format!("❌ Error saving config: {}", e));
                            }
                        }
                    } else {
                        self.add_progress("No domains to refresh".to_string());
                        self.load_data(); // Still reload UI from config
                    }
                }
                Err(e) => {
                    self.add_progress(format!("❌ Error loading config: {}", e));
                }
            }
        }
    }

    /// Execute delete domain
    fn execute_delete_domain(
        &mut self,
        domain: &str,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            // Find which provider has this domain
            let provider = if let Some((ref p, _)) = self.selected_domain {
                p.clone()
            } else {
                // If no selection, try to find it in loaded data
                match load_ns_config() {
                    Ok(config) => {
                        if let Some((prov, _)) = config.get_domain_any_provider(domain) {
                            prov.to_string()
                        } else {
                            self.add_progress(format!("Error: Domain '{}' not found", domain));
                            return;
                        }
                    }
                    Err(e) => {
                        self.add_progress(format!("Error loading config: {}", e));
                        return;
                    }
                }
            };

            if let Some(vm) = vm {
                self.add_progress(format!("Deleting domain: {}...", domain));
                match vm.delete_dns_domain(provider, domain.to_string()) {
                    Ok(_) => {
                        let _ = audit::push_gui("system", "desktop", "ns del", domain);
                    }
                    Err(e) => {
                        self.add_progress(format!("Failed to start domain deletion: {}", e));
                    }
                }
            }
        }
    }

    /// Execute add record
    fn execute_add_record(&mut self, mut vm: Option<&mut crate::viewmodel::ViewModel>) {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            let name = self.add_record_name.trim().to_string();
            let value = self.add_record_value.trim().to_string();
            if value.is_empty() {
                self.add_progress("Error: Value is required".to_string());
                return;
            }

            if let Some((provider, domain)) = self.selected_domain.clone() {
                let record_type = RecordType::from_str(&self.add_record_type);
                if record_type.is_none() {
                    self.add_progress("Error: Invalid record type".to_string());
                    return;
                }
                let record_type_enum = record_type.unwrap();

                // Normalize name based on provider
                let normalized_name = if provider == "cloudflare" || provider == "cf" {
                    if name.is_empty() || name == "@" {
                        domain.clone()
                    } else {
                        format!("{}.{}", name, domain)
                    }
                } else {
                    name.clone()
                };

                // If applying to DNS provider, use ViewModel
                if self.add_record_apply {
                    if let Some(ref mut vm) = vm {
                        // Send command to ViewModel
                        let record_type_str = self.add_record_type.to_uppercase();
                        match vm.add_dns_record(
                            provider.clone(),
                            domain.clone(),
                            record_type_str.clone(),
                            normalized_name.clone(),
                            value.clone(),
                            300, // Default TTL
                        ) {
                            Ok(_) => {
                                // Add to config optimistically
                                match load_ns_config() {
                                    Ok(mut config) => {
                                        if let Ok(_) = config.add_record(
                                            &provider,
                                            &domain,
                                            record_type_enum.clone(),
                                            normalized_name.clone(),
                                            value.clone(),
                                        ) {
                                            // Record audit event
                                            let record_desc = format!(
                                                "{} {} {} {}",
                                                domain, name, record_type_str, value
                                            );
                                            let _ = audit::push_gui(
                                                "system",
                                                "desktop",
                                                "ns insert",
                                                &record_desc,
                                            );
                                        }
                                        // Note: Config will be saved when RecordAdded event arrives
                                    }
                                    Err(e) => {
                                        self.error_message =
                                            format!("Failed to load config:\n\n{}", e);
                                        self.show_error_dialog = true;
                                    }
                                }
                            }
                            Err(e) => {
                                self.error_message =
                                    format!("Failed to start record addition:\n\n{}", e);
                                self.show_error_dialog = true;
                            }
                        }
                    } else {
                        self.error_message = "ViewModel not available".to_string();
                        self.show_error_dialog = true;
                    }
                } else {
                    // Not applying - just save to config (no API call)
                    match load_ns_config() {
                        Ok(mut config) => {
                            match config.add_record(
                                &provider,
                                &domain,
                                record_type_enum.clone(),
                                normalized_name.clone(),
                                value.clone(),
                            ) {
                                Ok(_) => match save_ns_config(&config) {
                                    Ok(_) => {
                                        let record_desc = format!(
                                            "{} {} {} {}",
                                            domain, name, self.add_record_type, value
                                        );
                                        let _ = audit::push_gui(
                                            "system",
                                            "desktop",
                                            "ns insert",
                                            &record_desc,
                                        );

                                        self.add_progress(format!(
                                            "✓ Added record: {} {} {} -> {}",
                                            domain,
                                            normalized_name,
                                            self.add_record_type.to_uppercase(),
                                            value
                                        ));

                                        self.load_records();
                                        self.load_data();
                                    }
                                    Err(e) => {
                                        self.error_message =
                                            format!("Failed to save config:\n\n{}", e);
                                        self.show_error_dialog = true;
                                    }
                                },
                                Err(e) => {
                                    self.error_message = format!("Failed to add record:\n\n{}", e);
                                    self.show_error_dialog = true;
                                }
                            }
                        }
                        Err(e) => {
                            self.error_message = format!("Failed to load config:\n\n{}", e);
                            self.show_error_dialog = true;
                        }
                    }
                }
            }
        }
    }

    /// Execute delete record
    fn execute_delete_record(
        &mut self,
        name: &str,
        record_type: &str,
        _value: &str,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            let (provider, domain) = if let Some((ref p, ref d)) = self.selected_domain {
                (p.clone(), d.clone())
            } else {
                return;
            };

            // ViewModel-based implementation
            if let Some(vm) = vm {
                self.add_progress(format!("Deleting record: {} {}...", name, record_type));

                match vm.delete_dns_record(
                    provider.clone(),
                    domain.clone(),
                    name.to_string(),
                    record_type.to_string(),
                ) {
                    Ok(_) => {
                        // Record audit event
                        let record_desc = format!("{} {} {}", domain, name, record_type);
                        let _ = audit::push_gui("system", "desktop", "ns remove", &record_desc);
                        // Config will be updated when RecordDeleted event arrives
                    }
                    Err(e) => {
                        self.error_message = format!("Failed to start record deletion:\n\n{}", e);
                        self.show_error_dialog = true;
                    }
                }
            }
        }
    }

    /// Show nameservers for a domain
    fn show_nameservers(&mut self, provider: &str, domain: &str) {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            use crate::calc::dns;

            // Initialize dialog state
            self.ns_dialog_domain = domain.to_string();
            self.ns_dialog_provider_ns = Vec::new();
            self.ns_dialog_actual_ns = Vec::new();
            self.ns_dialog_loading = true;
            self.show_nameservers_dialog = true;

            // Resolve actual DNS NS records
            match dns::resolve_dns(domain, dns::RecordType::NS) {
                Ok(records) => {
                    self.ns_dialog_actual_ns = records.iter().map(|r| r.value.clone()).collect();
                }
                Err(e) => {
                    self.add_progress(format!("⚠ Failed to resolve NS records: {}", e));
                }
            }

            // Fetch provider NS records
            match load_ns_config() {
                Ok(config) => {
                    // Extract base provider name (strip email for gcloud)
                    let base_provider = if provider.starts_with("gcloud:") {
                        "gcloud"
                    } else {
                        provider
                    };

                    let token_opt = config.get_api_token(provider);
                    if token_opt.is_none() {
                        self.add_progress("⚠ No API token found for provider".to_string());
                        self.ns_dialog_loading = false;
                        return;
                    }
                    let token = token_opt.unwrap();

                    match base_provider {
                        "cloudflare" => {
                            use crate::api::ns_cloudflare::CloudflareClient;
                            let client = CloudflareClient::new(token.clone());

                            match client.list_zones() {
                                Ok(zones) => {
                                    if let Some(zone) = zones.iter().find(|z| z.name == domain) {
                                        self.ns_dialog_provider_ns = zone.name_servers.clone();
                                    }
                                }
                                Err(e) => {
                                    self.add_progress(format!(
                                        "⚠ Failed to fetch Cloudflare zones: {}",
                                        e
                                    ));
                                }
                            }
                        }
                        "gcloud" => {
                            use crate::api::ns_gcp::GcpDnsClient;

                            let parts: Vec<&str> = token.split("::").collect();
                            if parts.len() == 2 {
                                let access_token = parts[0].to_string();
                                let project_id = parts[1].to_string();
                                let client = GcpDnsClient::new(access_token);

                                match client.list_managed_zones(&project_id) {
                                    Ok(zones) => {
                                        let dns_name = if domain.ends_with('.') {
                                            domain.to_string()
                                        } else {
                                            format!("{}.", domain)
                                        };

                                        if let Some(zone) =
                                            zones.iter().find(|z| z.dns_name == dns_name)
                                        {
                                            self.ns_dialog_provider_ns = zone.name_servers.clone();
                                        }
                                    }
                                    Err(e) => {
                                        self.add_progress(format!(
                                            "⚠ Failed to fetch GCP zones: {}",
                                            e
                                        ));
                                    }
                                }
                            }
                        }
                        "porkbun" => {
                            use crate::api::ns_porkbun::PorkbunClient;

                            let parts: Vec<&str> = token.split("::").collect();
                            if parts.len() == 2 {
                                let client =
                                    PorkbunClient::new(parts[0].to_string(), parts[1].to_string());

                                match client.get_records(domain) {
                                    Ok(records) => {
                                        self.ns_dialog_provider_ns = records
                                            .iter()
                                            .filter(|r| r.record_type.to_uppercase() == "NS")
                                            .map(|r| r.content.clone())
                                            .collect();
                                    }
                                    Err(e) => {
                                        self.add_progress(format!(
                                            "⚠ Failed to fetch Porkbun records: {}",
                                            e
                                        ));
                                    }
                                }
                            }
                        }
                        "duckdns" => {
                            self.ns_dialog_provider_ns =
                                vec!["ns1.duckdns.org".to_string(), "ns2.duckdns.org".to_string()];
                        }
                        _ => {}
                    }

                    self.ns_dialog_loading = false;
                }
                Err(e) => {
                    self.add_progress(format!("⚠ Failed to load config: {}", e));
                    self.ns_dialog_loading = false;
                }
            }
        }
    }

    /// Show nameservers comparison dialog
    fn show_nameservers_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_nameservers_dialog {
            return;
        }

        let mut open = true;
        egui::Window::new(format!("Nameservers - {}", self.ns_dialog_domain))
            .id(egui::Id::new("ns_nameservers_dialog"))
            .open(&mut open)
            .resizable(true)
            .default_width(600.0)
            .show(ctx, |ui| {
                if self.ns_dialog_loading {
                    ui.spinner();
                    ui.label("Loading nameservers...");
                    return;
                }

                ui.horizontal(|ui| {
                    // Provider NS column
                    ui.vertical(|ui| {
                        ui.heading("Provider Nameservers");
                        ui.add_space(8.0);

                        if self.ns_dialog_provider_ns.is_empty() {
                            ui.label("(No nameservers found)");
                        } else {
                            for ns in &self.ns_dialog_provider_ns {
                                ui.label(format!("• {}", ns));
                            }
                        }
                    });

                    ui.separator();

                    // Actual DNS resolution column
                    ui.vertical(|ui| {
                        ui.heading("Actual DNS Resolution");
                        ui.add_space(8.0);

                        if self.ns_dialog_actual_ns.is_empty() {
                            ui.label("(No NS records resolved)");
                        } else {
                            // Compare and highlight matches
                            for ns in &self.ns_dialog_actual_ns {
                                let matches = self.ns_dialog_provider_ns.contains(ns);
                                let color = if matches {
                                    egui::Color32::from_rgb(76, 175, 80) // Green if matches
                                } else {
                                    egui::Color32::from_rgb(255, 152, 0) // Orange if different
                                };
                                ui.colored_label(color, format!("• {}", ns));
                            }
                        }
                    });
                });

                ui.add_space(16.0);

                // Status summary
                if !self.ns_dialog_provider_ns.is_empty() && !self.ns_dialog_actual_ns.is_empty() {
                    let all_match = self
                        .ns_dialog_actual_ns
                        .iter()
                        .all(|ns| self.ns_dialog_provider_ns.contains(ns));

                    if all_match {
                        ui.colored_label(
                            egui::Color32::from_rgb(76, 175, 80),
                            "✓ DNS is correctly configured",
                        );
                    } else {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 152, 0),
                            "⚠ DNS nameservers don't match - DNS may not be propagated yet",
                        );
                    }
                }

                ui.add_space(8.0);
                if ui.add(MaterialButton::text("Close")).clicked() {
                    self.show_nameservers_dialog = false;
                }
            });

        if !open {
            self.show_nameservers_dialog = false;
        }
    }

    fn add_progress(&mut self, message: String) {
        self.progress_log.push(message);
        // Keep only last 20 messages
        if self.progress_log.len() > 20 {
            self.progress_log.remove(0);
        }
    }

    /// Start GCP OAuth flow
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn start_gcp_oauth(&mut self) {
        use crate::api::gcp::oauth::OAuthHandler;
        use poll_promise::Promise;

        self.add_progress("Starting GCP OAuth flow...".to_string());

        // Use embedded OAuth credentials (compiled into binary)
        let handler = OAuthHandler::default();

        self.add_gcp_oauth_promise = Some(Promise::spawn_thread("gcp_oauth_ns", move || {
            handler.run_oauth_flow().map_err(|e| e.to_string())
        }));
    }

    /// Fetch connected email from OAuth result
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn fetch_gcp_connected_email(&mut self) {
        if let Some(oauth) = self.add_gcp_oauth_result.clone() {
            use crate::api::gcp::GcpRestClient;

            let client = GcpRestClient::new(oauth.access_token.clone());

            match client.get_user_info() {
                Ok(user_info) => {
                    let display = if let Some(email) = user_info.email {
                        email
                    } else if let Some(name) = user_info.name {
                        name
                    } else {
                        "Connected Account".to_string()
                    };
                    self.add_gcp_connected_email = Some(display.clone());
                    self.add_progress("✓ Connected to Google Cloud".to_string());

                    // Note: GCP account will be saved when project is selected and domain is added
                    self.add_progress("  Select a project to continue".to_string());
                }
                Err(e) => {
                    self.add_progress(format!("⚠ Failed to fetch user info: {}", e));
                    self.add_gcp_connected_email = Some("Connected Account".to_string());
                }
            }
        }
    }

    /// Load GCP projects list
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn load_gcp_projects(&mut self) {
        if self.add_gcp_projects_loaded {
            return;
        }

        if let Some(oauth) = &self.add_gcp_oauth_result {
            use crate::api::gcp::GcpRestClient;

            let client = GcpRestClient::new(oauth.access_token.clone());

            match client.list_projects(None) {
                Ok(project_list) => {
                    self.add_gcp_projects = project_list.projects;
                    self.add_gcp_projects_loaded = true;
                    self.add_gcp_projects_error = None;

                    let active_count = self
                        .add_gcp_projects
                        .iter()
                        .filter(|p| p.is_active())
                        .count();

                    self.add_progress(format!(
                        "✓ Loaded {} projects ({} active)",
                        self.add_gcp_projects.len(),
                        active_count
                    ));

                    // Auto-select first active project
                    if let Some(project) = self.add_gcp_projects.iter().find(|p| p.is_active()) {
                        self.add_gcp_project_id = project.project_id.clone();
                    }
                }
                Err(e) => {
                    self.add_gcp_projects_error = Some(format!("Failed to load projects: {}", e));
                    self.add_gcp_projects_loaded = true;
                    self.add_progress(format!("⚠ Failed to load projects: {}", e));
                }
            }
        }
    }
}

#[cfg(test)]
mod ns_table_tests {
    use super::*;

    #[test]
    fn test_domain_row_data_from_empty_config() {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            use crate::calc::ns::NsConfig;
            let config = NsConfig::default();
            let rows = build_domain_rows(&config);
            assert_eq!(rows.len(), 0);
        }
    }

    #[test]
    fn test_domain_with_no_records() {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            use crate::calc::ns::NsConfig;
            let mut config = NsConfig::default();
            config.add_domain(
                "cloudflare".to_string(),
                "example.com".to_string(),
                "test_token".to_string()
            ).unwrap();

            let rows = build_domain_rows(&config);
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].domain, "example.com");
            assert_eq!(rows[0].provider, "cloudflare");
            assert_eq!(rows[0].provider_display, "Cloudflare");
            assert_eq!(rows[0].records.len(), 0);
        }
    }

    #[test]
    fn test_domain_with_multiple_records() {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            use crate::calc::ns::{NsConfig, DnsRecord, RecordType};
            let mut config = NsConfig::default();
            config.add_domain(
                "cloudflare".to_string(),
                "example.com".to_string(),
                "test_token".to_string()
            ).unwrap();

            let domain_entry = config.get_domain_mut("cloudflare", "example.com").unwrap();
            domain_entry.records.push(DnsRecord {
                record_type: RecordType::A,
                name: "www".to_string(),
                value: "1.2.3.4".to_string(),
                ttl: Some(300),
            });
            domain_entry.records.push(DnsRecord {
                record_type: RecordType::TXT,
                name: "_dmarc".to_string(),
                value: "v=DMARC1; p=none;".to_string(),
                ttl: Some(3600),
            });

            let rows = build_domain_rows(&config);
            assert_eq!(rows[0].records.len(), 2);
            assert_eq!(rows[0].records[0].name, "www");
            assert_eq!(rows[0].records[1].name, "_dmarc");
        }
    }

    #[test]
    fn test_gcp_provider_display_format() {
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            use crate::calc::ns::{NsConfig, GcpAccount};
            let mut config = NsConfig::default();

            // Add GCP account first
            let gcp_account = GcpAccount {
                access_token: "test_token".to_string(),
                refresh_token: "refresh_token".to_string(),
                token_expiry: 3600,
                connected_email: "user@gmail.com".to_string(),
                project_id: "test-project".to_string(),
                domains: Vec::new(),
            };
            config.add_gcp_account(gcp_account).unwrap();

            // Now add domain to GCP account
            config.add_domain(
                "gcloud:user@gmail.com".to_string(),
                "example.com".to_string(),
                "test_token".to_string()
            ).unwrap();

            let rows = build_domain_rows(&config);
            assert_eq!(rows[0].provider, "gcloud:user@gmail.com");
            assert_eq!(rows[0].provider_display, "Google Cloud (user@gmail.com)");
        }
    }
}
