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

    async fn add_provider(&mut self, _name: String, _provider_type: String, _api_token: String) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("DNS provider management not yet implemented in ViewModel"))
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

    async fn add_record(&mut self, _provider_name: String, _domain: String, _record_type: String, _name: String, _value: String, _ttl: u32) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("DNS record management not yet implemented in ViewModel"))
    }

    async fn delete_record(&mut self, _provider_name: String, _domain: String, _record_id: String) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("DNS record management not yet implemented in ViewModel"))
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
