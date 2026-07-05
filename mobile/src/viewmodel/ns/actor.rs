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
            NsCommand::DeleteRecord { provider_name, domain, record_id } => {
                self.delete_record(provider_name, domain, record_id).await
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
        self.send_progress("add_provider", 0.5, "Adding DNS provider...").await;

        runtime::unblock({
            let name = name.clone();
            move || crate::calc::db::save_dns_provider(&name, &provider_type, &api_token)
        }).await?;

        self.send_event(NsEvent::ProviderAdded { name }).await;
        Ok(())
    }

    async fn delete_provider(&mut self, name: String) -> anyhow::Result<()> {
        self.send_progress("delete_provider", 0.5, "Deleting DNS provider...").await;

        runtime::unblock({
            let name = name.clone();
            move || crate::calc::db::delete_dns_provider(&name)
        }).await?;

        self.send_event(NsEvent::ProviderDeleted { name }).await;
        Ok(())
    }

    async fn list_providers(&mut self) -> anyhow::Result<()> {
        self.send_progress("list_providers", 0.5, "Loading DNS providers...").await;

        let providers = runtime::unblock(|| {
            crate::calc::db::load_dns_providers()
        }).await?;

        let provider_infos: Vec<DnsProvider> = providers.into_iter().map(|p| DnsProvider {
            name: p.name,
            provider_type: p.provider_type,
        }).collect();

        self.send_event(NsEvent::ProvidersListed { providers: provider_infos }).await;
        Ok(())
    }

    async fn add_domain(&mut self, provider_name: String, domain: String) -> anyhow::Result<()> {
        self.send_progress("add_domain", 0.5, "Adding domain to DNS provider...").await;

        runtime::unblock({
            let provider_name = provider_name.clone();
            let domain = domain.clone();
            move || crate::calc::ns::add_domain(&provider_name, &domain)
        }).await?;

        self.send_event(NsEvent::DomainAdded { provider_name, domain }).await;
        Ok(())
    }

    async fn delete_domain(&mut self, provider_name: String, domain: String) -> anyhow::Result<()> {
        self.send_progress("delete_domain", 0.5, "Deleting domain from DNS provider...").await;

        runtime::unblock({
            let provider_name = provider_name.clone();
            let domain = domain.clone();
            move || crate::calc::ns::delete_domain(&provider_name, &domain)
        }).await?;

        self.send_event(NsEvent::DomainDeleted { provider_name, domain }).await;
        Ok(())
    }

    async fn list_domains(&mut self, provider_name: String) -> anyhow::Result<()> {
        self.send_progress("list_domains", 0.5, "Loading domains...").await;

        let domains = runtime::unblock({
            let provider_name = provider_name.clone();
            move || crate::calc::ns::list_domains(&provider_name)
        }).await?;

        let domain_infos: Vec<DnsDomain> = domains.into_iter().map(|d| DnsDomain {
            domain: d.domain,
            provider_name: provider_name.clone(),
        }).collect();

        self.send_event(NsEvent::DomainsListed { provider_name, domains: domain_infos }).await;
        Ok(())
    }

    async fn add_record(&mut self, provider_name: String, domain: String, record_type: String, name: String, value: String, ttl: u32) -> anyhow::Result<()> {
        self.send_progress("add_record", 0.5, "Adding DNS record...").await;

        let record_id = runtime::unblock({
            let provider_name = provider_name.clone();
            let domain = domain.clone();
            move || crate::calc::ns::add_record(&provider_name, &domain, &record_type, &name, &value, ttl)
        }).await?;

        self.send_event(NsEvent::RecordAdded {
            provider_name,
            domain,
            record_id,
        }).await;
        Ok(())
    }

    async fn delete_record(&mut self, provider_name: String, domain: String, record_id: String) -> anyhow::Result<()> {
        self.send_progress("delete_record", 0.5, "Deleting DNS record...").await;

        runtime::unblock({
            let provider_name = provider_name.clone();
            let domain = domain.clone();
            let record_id = record_id.clone();
            move || crate::calc::ns::delete_record(&provider_name, &domain, &record_id)
        }).await?;

        self.send_event(NsEvent::RecordDeleted {
            provider_name,
            domain,
            record_id,
        }).await;
        Ok(())
    }

    async fn list_records(&mut self, provider_name: String, domain: String) -> anyhow::Result<()> {
        self.send_progress("list_records", 0.5, "Loading DNS records...").await;

        let records = runtime::unblock({
            let provider_name = provider_name.clone();
            let domain = domain.clone();
            move || crate::calc::ns::list_records(&provider_name, &domain)
        }).await?;

        let record_infos: Vec<DnsRecord> = records.into_iter().map(|r| DnsRecord {
            id: r.id,
            record_type: r.record_type,
            name: r.name,
            value: r.value,
            ttl: r.ttl,
        }).collect();

        self.send_event(NsEvent::RecordsListed {
            provider_name,
            domain,
            records: record_infos,
        }).await;
        Ok(())
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
