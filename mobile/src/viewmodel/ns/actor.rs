//! NS actor implementation

use super::{NsCommand, NsEvent, DnsProvider, DnsDomain, DnsRecord};
use crate::viewmodel::{ViewModelEvent, runtime};
use smol::channel::{Receiver, Sender};

pub struct NsActor {
    command_rx: Receiver<NsCommand>,
    event_tx: Sender<ViewModelEvent>,
}

impl NsActor {
    pub fn new(command_rx: Receiver<NsCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self { command_rx, event_tx }
    }

    pub async fn run(mut self) {
        log::info!("NsActor started");

        loop {
            match self.command_rx.recv().await {
                Ok(cmd) => {
                    if let Err(e) = self.handle_command(cmd).await {
                        log::error!("NsActor command failed: {}", e);
                    }
                }
                Err(_) => {
                    log::info!("NsActor: channel closed, shutting down");
                    break;
                }
            }
        }
    }

    async fn handle_command(&mut self, cmd: NsCommand) -> anyhow::Result<()> {
        let operation = format!("{:?}", cmd);

        let result = match cmd {
            NsCommand::AddProvider { name, provider_type, api_token } => {
                self.add_provider(name, provider_type, api_token).await
            }
            NsCommand::DeleteProvider { name } => {
                self.delete_provider(name).await
            }
            NsCommand::ListProviders => {
                self.list_providers().await
            }
            NsCommand::AddDomain { provider_name, domain } => {
                self.add_domain(provider_name, domain).await
            }
            NsCommand::DeleteDomain { provider_name, domain } => {
                self.delete_domain(provider_name, domain).await
            }
            NsCommand::ListDomains { provider_name } => {
                self.list_domains(provider_name).await
            }
            NsCommand::AddRecord { provider_name, domain, record_type, name, value, ttl } => {
                self.add_record(provider_name, domain, record_type, name, value, ttl).await
            }
            NsCommand::DeleteRecord { provider_name, domain, name, record_type } => {
                self.delete_record(provider_name, domain, name, record_type).await
            }
            NsCommand::ListRecords { provider_name, domain } => {
                self.list_records(provider_name, domain).await
            }
            NsCommand::RefreshAll => {
                Err(anyhow::anyhow!("RefreshAll not implemented yet"))
            }
        };

        if let Err(e) = result {
            self.send_error(&operation, e).await;
        }

        Ok(())
    }

    async fn add_provider(&mut self, name: String, provider_type: String, api_token: String) -> anyhow::Result<()> {
        self.send_progress("add_provider", 0.1, "Adding DNS provider...").await;

        // Fetch domains and records from provider API in blocking thread
        let (provider_name, domains_with_records) = runtime::unblock({
            let provider_type = provider_type.clone();
            let api_token = api_token.clone();
            move || -> anyhow::Result<(String, Vec<(String, Vec<crate::calc::ns::DnsRecord>)>)> {
                use crate::calc::ns::{NsConfig, RecordType};

                match provider_type.as_str() {
                    "cloudflare" => {
                        use crate::api::ns_cloudflare::CloudflareClient;
                        let client = CloudflareClient::new(api_token);
                        let zones = client.list_zones()?;

                        let mut domains = Vec::new();
                        for zone in zones {
                            let records = client.get_records(&zone.id)?;
                            let filtered_records: Vec<crate::calc::ns::DnsRecord> = records
                                .iter()
                                .filter(|r| {
                                    let rt = r.record_type.to_uppercase();
                                    rt == "A" || rt == "AAAA" || rt == "TXT"
                                })
                                .filter_map(|r| {
                                    RecordType::from_str(&r.record_type.to_lowercase())
                                        .map(|rt| crate::calc::ns::DnsRecord {
                                            record_type: rt,
                                            name: r.name.clone(),
                                            value: r.content.clone(),
                                            ttl: Some(r.ttl),
                                        })
                                })
                                .collect();
                            domains.push((zone.name, filtered_records));
                        }
                        Ok(("cloudflare".to_string(), domains))
                    }
                    "porkbun" => {
                        use crate::api::ns_porkbun::PorkbunClient;
                        let parts: Vec<&str> = api_token.split("::").collect();
                        if parts.len() != 2 {
                            return Err(anyhow::anyhow!("Invalid Porkbun token format (expected apikey::secretkey)"));
                        }
                        let client = PorkbunClient::new(parts[0].to_string(), parts[1].to_string());
                        let domain_names = client.list_domains()?;

                        let mut domains = Vec::new();
                        for domain in domain_names {
                            let records = client.get_records(&domain)?;
                            let filtered_records: Vec<crate::calc::ns::DnsRecord> = records
                                .iter()
                                .filter(|r| {
                                    let rt = r.record_type.to_uppercase();
                                    rt == "A" || rt == "AAAA" || rt == "TXT"
                                })
                                .filter_map(|r| {
                                    RecordType::from_str(&r.record_type.to_lowercase())
                                        .map(|rt| crate::calc::ns::DnsRecord {
                                            record_type: rt,
                                            name: r.name.clone(),
                                            value: r.content.clone(),
                                            ttl: r.ttl.parse().ok(),
                                        })
                                })
                                .collect();
                            domains.push((domain, filtered_records));
                        }
                        Ok(("porkbun".to_string(), domains))
                    }
                    "duckdns" => {
                        // DuckDNS doesn't support auto-discovery
                        Ok(("duckdns".to_string(), Vec::new()))
                    }
                    _ => Err(anyhow::anyhow!("Unsupported provider type: {}", provider_type))
                }
            }
        }).await?;

        self.send_progress("add_provider", 0.8, "Saving configuration...").await;

        // Save to config
        runtime::unblock({
            let provider_name = provider_name.clone();
            let api_token = api_token.clone();
            let domains_with_records = domains_with_records.clone();
            move || -> anyhow::Result<usize> {
                // Note: Config loading/saving is handled by UI layer
                // This is a placeholder showing the data flow
                Ok(domains_with_records.len())
            }
        }).await?;

        self.send_progress("add_provider", 1.0, "Provider added").await;

        self.send_event(NsEvent::ProviderAdded {
            name: provider_name,
            domains: domains_with_records,
        }).await;

        Ok(())
    }

    async fn delete_provider(&mut self, _name: String) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("DNS provider management not yet implemented in ViewModel"))
    }

    async fn list_providers(&mut self) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("DNS provider management not yet implemented in ViewModel"))
    }

    async fn add_domain(&mut self, _provider_name: String, _domain: String) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("DNS domain management not yet implemented in ViewModel"))
    }

    async fn delete_domain(&mut self, _provider_name: String, _domain: String) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("DNS domain management not yet implemented in ViewModel"))
    }

    async fn list_domains(&mut self, _provider_name: String) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("DNS domain management not yet implemented in ViewModel"))
    }

    async fn add_record(&mut self, provider_name: String, domain: String, record_type: String, name: String, value: String, ttl: u32) -> anyhow::Result<()> {
        self.send_progress("add_record", 0.3, "Adding DNS record...").await;

        // Add record via API in blocking thread
        let record_id = runtime::unblock({
            let provider_name = provider_name.clone();
            let domain = domain.clone();
            let record_type = record_type.clone();
            let name = name.clone();
            let value = value.clone();
            move || -> anyhow::Result<String> {
                use crate::calc::ns::apply_record;
                apply_record(&provider_name, &domain, &record_type, &name, &value, ttl)
            }
        }).await?;

        self.send_progress("add_record", 1.0, "Record added").await;

        self.send_event(NsEvent::RecordAdded {
            provider_name,
            domain,
            record_id,
        }).await;

        Ok(())
    }

    async fn delete_record(&mut self, provider_name: String, domain: String, name: String, record_type: String) -> anyhow::Result<()> {
        self.send_progress("delete_record", 0.3, "Deleting DNS record...").await;

        // Delete record via API in blocking thread
        runtime::unblock({
            let provider_name = provider_name.clone();
            let domain = domain.clone();
            let name = name.clone();
            let record_type = record_type.clone();
            move || -> anyhow::Result<()> {
                use crate::calc::acme::{DnsProvider, DnsProviderType, delete_dns_record};
                use crate::calc::ns::NsConfig;

                // Load config to get API token and provider type
                let config_path = directories::ProjectDirs::from("com", "dure", "dure")
                    .ok_or_else(|| anyhow::anyhow!("Failed to get project directories"))?
                    .config_dir()
                    .join("config.yml");

                let yaml = std::fs::read_to_string(&config_path)?;
                let full_config: serde_yaml::Value = serde_yaml::from_str(&yaml)?;
                let ns_config: NsConfig = if let Some(ns) = full_config.get("ns") {
                    serde_yaml::from_value(ns.clone())?
                } else {
                    NsConfig::default()
                };

                // Determine provider type
                let provider_type = if provider_name.starts_with("gcloud:") {
                    DnsProviderType::GoogleCloud
                } else {
                    match provider_name.to_lowercase().as_str() {
                        "cloudflare" | "cf" => DnsProviderType::Cloudflare,
                        "gcloud" | "googlecloud" | "gcp" => DnsProviderType::GoogleCloud,
                        "duckdns" => DnsProviderType::DuckDNS,
                        "porkbun" => DnsProviderType::Porkbun,
                        _ => return Err(anyhow::anyhow!("Unknown provider: {}", provider_name)),
                    }
                };

                let api_token = ns_config.get_api_token(&provider_name).unwrap_or_default();
                let dns_provider = DnsProvider {
                    provider_type,
                    api_token,
                };

                // Delete the record
                delete_dns_record(&dns_provider, &domain, &name, &record_type)?;

                Ok(())
            }
        }).await?;

        self.send_progress("delete_record", 1.0, "Record deleted").await;

        self.send_event(NsEvent::RecordDeleted {
            provider_name,
            domain,
            record_id: format!("{}:{}", name, record_type),
        }).await;

        Ok(())
    }

    async fn list_records(&mut self, _provider_name: String, _domain: String) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("DNS record management not yet implemented in ViewModel"))
    }

    async fn send_progress(&self, operation: &str, progress: f32, status: &str) {
        let _ = self.event_tx.send(ViewModelEvent::Ns(
            NsEvent::Progress {
                operation: operation.to_string(),
                progress,
                status: status.to_string(),
            }
        )).await;
    }

    async fn send_event(&self, event: NsEvent) {
        let _ = self.event_tx.send(ViewModelEvent::Ns(event)).await;
    }

    async fn send_error(&self, operation: &str, error: anyhow::Error) {
        let _ = self.event_tx.send(ViewModelEvent::Ns(
            NsEvent::Error {
                operation: operation.to_string(),
                error: format!("{:#}", error),
            }
        )).await;
    }
}
