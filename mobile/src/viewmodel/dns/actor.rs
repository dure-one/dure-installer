//! DNS actor implementation

use super::commands::{DnsCommand, RecordType};
use super::events::{DnsEvent, DnsRecord, SrvRecord, Zone};
use crate::viewmodel::common::ViewModelEvent;
use crate::viewmodel::runtime;
use crate::{dure_debug, dure_error, dure_info};
use smol::channel::{Receiver, Sender};

pub struct DnsActor {
    command_rx: Receiver<DnsCommand>,
    event_tx: Sender<ViewModelEvent>,
    access_token: Option<String>,
    refresh_token: Option<String>,
}

impl DnsActor {
    pub fn new(command_rx: Receiver<DnsCommand>, event_tx: Sender<ViewModelEvent>) -> Self {
        Self {
            command_rx,
            event_tx,
            access_token: None,
            refresh_token: None,
        }
    }

    pub async fn run(mut self) {
        dure_info!("[DnsActor] Starting");

        while let Ok(command) = self.command_rx.recv().await {
            dure_debug!("[DnsActor] Received command: {:?}", command);

            if let Err(e) = self.handle_command(command).await {
                dure_error!("[DnsActor] Command error: {}", e);
                self.send_event(DnsEvent::Error {
                    operation: "command".to_string(),
                    error: e.to_string(),
                })
                .await;
            }
        }

        dure_info!("[DnsActor] Stopped");
    }

    async fn handle_command(&mut self, command: DnsCommand) -> anyhow::Result<()> {
        match command {
            DnsCommand::SetAccessToken {
                access_token,
                refresh_token,
            } => self.set_access_token(access_token, refresh_token).await,

            DnsCommand::ListZones => self.list_zones().await,

            DnsCommand::FindZone { domain } => self.find_zone(domain).await,

            DnsCommand::CreateZone { domain, account_id } => {
                self.create_zone(domain, account_id).await
            }

            DnsCommand::ListRecords { zone_id } => self.list_records(zone_id).await,

            DnsCommand::FindRecord {
                zone_id,
                record_type,
                name,
            } => self.find_record(zone_id, record_type, name).await,

            DnsCommand::CreateRecord {
                zone_id,
                record_type,
                name,
                content,
                ttl,
                proxied,
            } => {
                self.create_record(zone_id, record_type, name, content, ttl, proxied)
                    .await
            }

            DnsCommand::UpdateRecord {
                zone_id,
                record_id,
                name,
                record_type,
                content,
                ttl,
                proxied,
            } => {
                self.update_record(zone_id, record_id, name, record_type, content, ttl, proxied)
                    .await
            }

            DnsCommand::DeleteRecord { zone_id, record_id } => {
                self.delete_record(zone_id, record_id).await
            }

            DnsCommand::UpsertRecord {
                zone_id,
                record_type,
                name,
                content,
                ttl,
                proxied,
            } => {
                self.upsert_record(zone_id, record_type, name, content, ttl, proxied)
                    .await
            }

            DnsCommand::CreateSrvRecord {
                zone_id,
                name,
                priority,
                weight,
                port,
                target,
            } => {
                self.create_srv_record(zone_id, name, priority, weight, port, target)
                    .await
            }

            DnsCommand::UpsertSrvRecord {
                zone_id,
                name,
                priority,
                weight,
                port,
                target,
            } => {
                self.upsert_srv_record(zone_id, name, priority, weight, port, target)
                    .await
            }

            DnsCommand::CreateDefaultXmppRecords {
                zone_id,
                domain,
                vm_ip,
            } => {
                self.create_default_xmpp_records(zone_id, domain, vm_ip)
                    .await
            }

            DnsCommand::VerifyDefaultXmppRecords {
                zone_id,
                domain,
                vm_ip,
            } => {
                self.verify_default_xmpp_records(zone_id, domain, vm_ip)
                    .await
            }
        }
    }

    async fn send_event(&self, event: DnsEvent) {
        let _ = self.event_tx.send(ViewModelEvent::Dns(event)).await;
    }

    fn require_token(&self) -> anyhow::Result<String> {
        self.access_token
            .clone()
            .ok_or_else(|| anyhow::anyhow!("No access token set"))
    }

    async fn set_access_token(
        &mut self,
        access_token: String,
        refresh_token: Option<String>,
    ) -> anyhow::Result<()> {
        self.access_token = Some(access_token);
        self.refresh_token = refresh_token;
        self.send_event(DnsEvent::AccessTokenSet).await;
        Ok(())
    }

    async fn list_zones(&mut self) -> anyhow::Result<()> {
        let token = self.require_token()?;

        let zones = runtime::unblock(move || -> anyhow::Result<Vec<Zone>> {
            use crate::api::cloudflare::dns::CloudflareClient;
            let client = CloudflareClient::new(token);
            let cf_zones = client.list_zones()?;

            Ok(cf_zones
                .into_iter()
                .map(|z| Zone {
                    id: z.id,
                    name: z.name,
                    name_servers: z.name_servers,
                    account_id: z.account.id,
                    account_name: z.account.name,
                })
                .collect())
        })
        .await?;

        self.send_event(DnsEvent::ZonesListed { zones }).await;
        Ok(())
    }

    async fn find_zone(&mut self, domain: String) -> anyhow::Result<()> {
        let token = self.require_token()?;

        let zone = runtime::unblock(move || -> anyhow::Result<Option<Zone>> {
            use crate::api::cloudflare::dns::CloudflareClient;
            let client = CloudflareClient::new(token);
            let cf_zone = client.find_zone_by_domain(&domain)?;

            Ok(cf_zone.map(|z| Zone {
                id: z.id,
                name: z.name,
                name_servers: z.name_servers,
                account_id: z.account.id,
                account_name: z.account.name,
            }))
        })
        .await?;

        self.send_event(DnsEvent::ZoneFound { zone }).await;
        Ok(())
    }

    async fn create_zone(&mut self, domain: String, _account_id: String) -> anyhow::Result<()> {
        let token = self.require_token()?;

        let zone = runtime::unblock(move || -> anyhow::Result<Zone> {
            use crate::api::cloudflare::dns::CloudflareClient;
            let client = CloudflareClient::new(token);
            let cf_zone = client.create_zone(&domain)?;

            Ok(Zone {
                id: cf_zone.id,
                name: cf_zone.name,
                name_servers: cf_zone.name_servers,
                account_id: cf_zone.account.id,
                account_name: cf_zone.account.name,
            })
        })
        .await?;

        self.send_event(DnsEvent::ZoneCreated { zone }).await;
        Ok(())
    }

    async fn list_records(&mut self, zone_id: String) -> anyhow::Result<()> {
        let token = self.require_token()?;
        let zone_id_clone = zone_id.clone();

        let records = runtime::unblock(move || -> anyhow::Result<Vec<DnsRecord>> {
            use crate::api::cloudflare::dns::CloudflareClient;
            let client = CloudflareClient::new(token);
            let cf_records = client.get_records(&zone_id_clone)?;

            Ok(cf_records
                .into_iter()
                .map(|r| DnsRecord {
                    id: r.id,
                    name: r.name,
                    record_type: r.record_type,
                    content: r.content,
                    ttl: r.ttl,
                    proxied: r.proxied,
                })
                .collect())
        })
        .await?;

        self.send_event(DnsEvent::RecordsListed {
            zone_id,
            records,
        })
        .await;
        Ok(())
    }

    async fn find_record(
        &mut self,
        zone_id: String,
        record_type: RecordType,
        name: String,
    ) -> anyhow::Result<()> {
        let token = self.require_token()?;
        let record_type_str = record_type.as_str().to_string();
        let zone_id_clone = zone_id.clone();

        let record = runtime::unblock(move || -> anyhow::Result<Option<DnsRecord>> {
            use crate::api::cloudflare::dns::CloudflareClient;
            let client = CloudflareClient::new(token);
            let cf_record = client.find_record(&zone_id_clone, &record_type_str, &name)?;

            Ok(cf_record.map(|r| DnsRecord {
                id: r.id,
                name: r.name,
                record_type: r.record_type,
                content: r.content,
                ttl: r.ttl,
                proxied: r.proxied,
            }))
        })
        .await?;

        self.send_event(DnsEvent::RecordFound {
            zone_id,
            record,
        })
        .await;
        Ok(())
    }

    async fn create_record(
        &mut self,
        zone_id: String,
        record_type: RecordType,
        name: String,
        content: String,
        ttl: Option<u32>,
        proxied: Option<bool>,
    ) -> anyhow::Result<()> {
        let token = self.require_token()?;
        let record_type_str = record_type.as_str().to_string();
        let zone_id_clone = zone_id.clone();
        let name_clone = name.clone();
        let record_type_clone = record_type_str.clone();

        runtime::unblock(move || -> anyhow::Result<()> {
            use crate::api::cloudflare::dns::CloudflareClient;
            let client = CloudflareClient::new(token);
            client.create_record(&zone_id_clone, &name_clone, &record_type_clone, &content, ttl, proxied)?;
            Ok(())
        })
        .await?;

        self.send_event(DnsEvent::RecordCreated {
            zone_id,
            name,
            record_type: record_type_str,
        })
        .await;
        Ok(())
    }

    async fn update_record(
        &mut self,
        zone_id: String,
        record_id: String,
        name: String,
        record_type: RecordType,
        content: String,
        ttl: Option<u32>,
        proxied: Option<bool>,
    ) -> anyhow::Result<()> {
        let token = self.require_token()?;
        let record_type_str = record_type.as_str().to_string();
        let zone_id_clone = zone_id.clone();
        let record_id_clone = record_id.clone();

        runtime::unblock(move || -> anyhow::Result<()> {
            use crate::api::cloudflare::dns::CloudflareClient;
            let client = CloudflareClient::new(token);
            client.update_record(&zone_id_clone, &record_id_clone, &name, &record_type_str, &content, ttl, proxied)?;
            Ok(())
        })
        .await?;

        self.send_event(DnsEvent::RecordUpdated {
            zone_id,
            record_id,
        })
        .await;
        Ok(())
    }

    async fn delete_record(&mut self, zone_id: String, record_id: String) -> anyhow::Result<()> {
        let token = self.require_token()?;
        let zone_id_clone = zone_id.clone();
        let record_id_clone = record_id.clone();

        runtime::unblock(move || -> anyhow::Result<()> {
            use crate::api::cloudflare::dns::CloudflareClient;
            let client = CloudflareClient::new(token);
            client.delete_record(&zone_id_clone, &record_id_clone)?;
            Ok(())
        })
        .await?;

        self.send_event(DnsEvent::RecordDeleted {
            zone_id,
            record_id,
        })
        .await;
        Ok(())
    }

    async fn upsert_record(
        &mut self,
        zone_id: String,
        record_type: RecordType,
        name: String,
        content: String,
        ttl: Option<u32>,
        proxied: Option<bool>,
    ) -> anyhow::Result<()> {
        let token = self.require_token()?;
        let record_type_str = record_type.as_str().to_string();
        let zone_id_clone = zone_id.clone();
        let name_clone = name.clone();
        let record_type_clone = record_type_str.clone();

        runtime::unblock(move || -> anyhow::Result<()> {
            use crate::api::cloudflare::dns::CloudflareClient;
            let client = CloudflareClient::new(token);
            client.upsert_record(&zone_id_clone, &name_clone, &record_type_clone, &content, ttl, proxied)?;
            Ok(())
        })
        .await?;

        self.send_event(DnsEvent::RecordUpserted {
            zone_id,
            name,
            record_type: record_type_str,
        })
        .await;
        Ok(())
    }

    async fn create_srv_record(
        &mut self,
        zone_id: String,
        name: String,
        priority: u16,
        weight: u16,
        port: u16,
        target: String,
    ) -> anyhow::Result<()> {
        let token = self.require_token()?;
        let zone_id_clone = zone_id.clone();
        let name_clone = name.clone();

        runtime::unblock(move || -> anyhow::Result<()> {
            use crate::api::cloudflare::dns::CloudflareClient;
            let client = CloudflareClient::new(token);
            client.create_srv_record(&zone_id_clone, &name_clone, priority, weight, port, &target)?;
            Ok(())
        })
        .await?;

        self.send_event(DnsEvent::SrvRecordCreated {
            zone_id,
            name,
        })
        .await;
        Ok(())
    }

    async fn upsert_srv_record(
        &mut self,
        zone_id: String,
        name: String,
        priority: u16,
        weight: u16,
        port: u16,
        target: String,
    ) -> anyhow::Result<()> {
        let token = self.require_token()?;
        let zone_id_clone = zone_id.clone();
        let name_clone = name.clone();

        runtime::unblock(move || -> anyhow::Result<()> {
            use crate::api::cloudflare::dns::CloudflareClient;
            let client = CloudflareClient::new(token);
            client.upsert_srv_record(&zone_id_clone, &name_clone, priority, weight, port, &target)?;
            Ok(())
        })
        .await?;

        self.send_event(DnsEvent::SrvRecordUpserted {
            zone_id,
            name,
        })
        .await;
        Ok(())
    }

    async fn create_default_xmpp_records(
        &mut self,
        zone_id: String,
        domain: String,
        vm_ip: String,
    ) -> anyhow::Result<()> {
        let token = self.require_token()?;
        let zone_id_clone = zone_id.clone();
        let domain_clone = domain.clone();

        runtime::unblock(move || -> anyhow::Result<()> {
            use crate::api::cloudflare::dns::CloudflareClient;
            let client = CloudflareClient::new(token);
            client.create_default_xmpp_records(&zone_id_clone, &domain_clone, &vm_ip)?;
            Ok(())
        })
        .await?;

        self.send_event(DnsEvent::DefaultXmppRecordsCreated { zone_id, domain })
            .await;
        Ok(())
    }

    async fn verify_default_xmpp_records(
        &mut self,
        zone_id: String,
        domain: String,
        vm_ip: String,
    ) -> anyhow::Result<()> {
        let token = self.require_token()?;
        let zone_id_clone = zone_id.clone();
        let domain_clone = domain.clone();

        let verified = runtime::unblock(move || -> anyhow::Result<bool> {
            use crate::api::cloudflare::dns::CloudflareClient;
            let client = CloudflareClient::new(token);
            client.verify_default_xmpp_records(&zone_id_clone, &domain_clone, &vm_ip)
        })
        .await?;

        self.send_event(DnsEvent::DefaultXmppRecordsVerified {
            zone_id,
            domain,
            verified,
        })
        .await;
        Ok(())
    }
}
