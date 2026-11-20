//! DNS command implementation - ViewModel actor interface

use crate::calc::profile::ProfileContext;
use crate::viewmodel::common::ViewModelEvent;
use crate::viewmodel::dns::{DnsCommand, DnsEvent, RecordType};
use crate::{dure_error, dure_info};
use anyhow::{Context, Result};
use smol::channel::bounded;

/// Helper to spawn DNS actor and send command
async fn send_dns_command(command: DnsCommand) -> Result<DnsEvent> {
    let (cmd_tx, cmd_rx) = bounded(10);
    let (evt_tx, evt_rx) = bounded(10);

    // Spawn DNS actor
    let actor = crate::viewmodel::dns::DnsActor::new(cmd_rx, evt_tx);
    smol::spawn(actor.run()).detach();

    // Send command
    cmd_tx
        .send(command)
        .await
        .context("Failed to send command to DNS actor")?;

    // Wait for response
    match evt_rx.recv().await {
        Ok(ViewModelEvent::Dns(event)) => Ok(event),
        Ok(_) => anyhow::bail!("Unexpected event type"),
        Err(e) => anyhow::bail!("Channel error: {}", e),
    }
}

// === Domain Registration Handlers ===

pub fn execute_list_domains(_profile: &ProfileContext, account_id: String) -> Result<()> {
    smol::block_on(async {
        let event = send_dns_command(DnsCommand::ListDomains { account_id }).await?;

        match event {
            DnsEvent::DomainsListed { domains } => {
                if domains.is_empty() {
                    println!("No registered domains found.");
                } else {
                    println!("Registered Domains ({}):", domains.len());
                    println!();
                    for domain in domains {
                        println!("  {} ({})", domain.name, domain.status);
                        println!("    ID: {}", domain.id);
                        println!("    Nameservers:");
                        for ns in &domain.nameservers {
                            println!("      - {}", ns);
                        }
                        if let Some(created) = &domain.created_at {
                            println!("    Created: {}", created);
                        }
                        if let Some(expires) = &domain.expires_at {
                            println!("    Expires: {}", expires);
                        }
                        println!();
                    }
                }
                Ok(())
            }
            DnsEvent::Error { operation, error } => {
                anyhow::bail!("DNS error ({}): {}", operation, error)
            }
            _ => anyhow::bail!("Unexpected event: {:?}", event),
        }
    })
}

pub fn execute_get_domain(
    _profile: &ProfileContext,
    account_id: String,
    domain: String,
) -> Result<()> {
    smol::block_on(async {
        let event = send_dns_command(DnsCommand::GetDomain { account_id, domain }).await?;

        match event {
            DnsEvent::DomainInfo { domain } => {
                println!("Domain Information:");
                println!();
                println!("  Name: {}", domain.name);
                println!("  ID: {}", domain.id);
                println!("  Status: {}", domain.status);
                println!("  Nameservers:");
                for ns in &domain.nameservers {
                    println!("    - {}", ns);
                }
                if let Some(created) = &domain.created_at {
                    println!("  Created: {}", created);
                }
                if let Some(expires) = &domain.expires_at {
                    println!("  Expires: {}", expires);
                }
                Ok(())
            }
            DnsEvent::Error { operation, error } => {
                anyhow::bail!("DNS error ({}): {}", operation, error)
            }
            _ => anyhow::bail!("Unexpected event: {:?}", event),
        }
    })
}

pub fn execute_check_availability(
    _profile: &ProfileContext,
    account_id: String,
    domain: String,
) -> Result<()> {
    smol::block_on(async {
        let event = send_dns_command(DnsCommand::CheckAvailability {
            account_id,
            domain: domain.clone(),
        })
        .await?;

        match event {
            DnsEvent::DomainAvailability { domain, available } => {
                if available {
                    println!("✓ Domain '{}' is available for registration", domain);
                } else {
                    println!("✗ Domain '{}' is not available", domain);
                }
                Ok(())
            }
            DnsEvent::Error { operation, error } => {
                anyhow::bail!("DNS error ({}): {}", operation, error)
            }
            _ => anyhow::bail!("Unexpected event: {:?}", event),
        }
    })
}

pub fn execute_register_domain(
    _profile: &ProfileContext,
    account_id: String,
    domain: String,
    years: u32,
) -> Result<()> {
    println!("Registering domain '{}' for {} year(s)...", domain, years);

    smol::block_on(async {
        let event = send_dns_command(DnsCommand::RegisterDomain {
            account_id,
            domain,
            years,
        })
        .await?;

        match event {
            DnsEvent::DomainRegistered { domain } => {
                println!();
                println!("✓ Domain registered successfully!");
                println!();
                println!("  Name: {}", domain.name);
                println!("  ID: {}", domain.id);
                println!("  Status: {}", domain.status);
                if let Some(expires) = &domain.expires_at {
                    println!("  Expires: {}", expires);
                }
                Ok(())
            }
            DnsEvent::Error { operation, error } => {
                anyhow::bail!("DNS error ({}): {}", operation, error)
            }
            _ => anyhow::bail!("Unexpected event: {:?}", event),
        }
    })
}

pub fn execute_update_nameservers(
    _profile: &ProfileContext,
    account_id: String,
    domain: String,
    nameservers: Vec<String>,
) -> Result<()> {
    println!("Updating nameservers for '{}'...", domain);

    smol::block_on(async {
        let event = send_dns_command(DnsCommand::UpdateNameservers {
            account_id,
            domain: domain.clone(),
            nameservers: nameservers.clone(),
        })
        .await?;

        match event {
            DnsEvent::NameserversUpdated { domain } => {
                println!();
                println!("✓ Nameservers updated successfully for '{}'", domain);
                println!();
                println!("  New nameservers:");
                for ns in nameservers {
                    println!("    - {}", ns);
                }
                Ok(())
            }
            DnsEvent::Error { operation, error } => {
                anyhow::bail!("DNS error ({}): {}", operation, error)
            }
            _ => anyhow::bail!("Unexpected event: {:?}", event),
        }
    })
}

// === Zone Handlers ===

pub fn execute_list_zones(_profile: &ProfileContext) -> Result<()> {
    smol::block_on(async {
        let event = send_dns_command(DnsCommand::ListZones).await?;

        match event {
            DnsEvent::ZonesListed { zones } => {
                if zones.is_empty() {
                    println!("No DNS zones found.");
                } else {
                    println!("DNS Zones ({}):", zones.len());
                    println!();
                    for zone in zones {
                        println!("  {} (ID: {})", zone.name, zone.id);
                        println!("    Account: {}", zone.account_name);
                        println!("    Name Servers:");
                        for ns in &zone.name_servers {
                            println!("      - {}", ns);
                        }
                        println!();
                    }
                }
                Ok(())
            }
            DnsEvent::Error { operation, error } => {
                anyhow::bail!("DNS error ({}): {}", operation, error)
            }
            _ => anyhow::bail!("Unexpected event: {:?}", event),
        }
    })
}

pub fn execute_find_zone(_profile: &ProfileContext, domain: String) -> Result<()> {
    smol::block_on(async {
        let event = send_dns_command(DnsCommand::FindZone {
            domain: domain.clone(),
        })
        .await?;

        match event {
            DnsEvent::ZoneFound { zone } => {
                if let Some(zone) = zone {
                    println!("Zone Found:");
                    println!();
                    println!("  Name: {}", zone.name);
                    println!("  ID: {}", zone.id);
                    println!("  Account: {}", zone.account_name);
                    println!("  Name Servers:");
                    for ns in &zone.name_servers {
                        println!("    - {}", ns);
                    }
                } else {
                    println!("Zone not found for domain: {}", domain);
                }
                Ok(())
            }
            DnsEvent::Error { operation, error } => {
                anyhow::bail!("DNS error ({}): {}", operation, error)
            }
            _ => anyhow::bail!("Unexpected event: {:?}", event),
        }
    })
}

pub fn execute_create_zone(
    _profile: &ProfileContext,
    domain: String,
    account_id: String,
) -> Result<()> {
    println!("Creating DNS zone for '{}'...", domain);

    smol::block_on(async {
        let event = send_dns_command(DnsCommand::CreateZone { domain, account_id }).await?;

        match event {
            DnsEvent::ZoneCreated { zone } => {
                println!();
                println!("✓ Zone created successfully!");
                println!();
                println!("  Name: {}", zone.name);
                println!("  ID: {}", zone.id);
                println!("  Name Servers:");
                for ns in &zone.name_servers {
                    println!("    - {}", ns);
                }
                Ok(())
            }
            DnsEvent::Error { operation, error } => {
                anyhow::bail!("DNS error ({}): {}", operation, error)
            }
            _ => anyhow::bail!("Unexpected event: {:?}", event),
        }
    })
}

// === Record Handlers ===

pub fn execute_list_records(_profile: &ProfileContext, zone_id: String) -> Result<()> {
    smol::block_on(async {
        let event = send_dns_command(DnsCommand::ListRecords {
            zone_id: zone_id.clone(),
        })
        .await?;

        match event {
            DnsEvent::RecordsListed { zone_id, records } => {
                if records.is_empty() {
                    println!("No DNS records found in zone: {}", zone_id);
                } else {
                    println!("DNS Records in zone {} ({}):", zone_id, records.len());
                    println!();
                    for record in records {
                        println!(
                            "  {} {} -> {}",
                            record.name, record.record_type, record.content
                        );
                        println!("    ID: {}", record.id);
                        println!("    TTL: {}s", record.ttl);
                        println!("    Proxied: {}", record.proxied);
                        println!();
                    }
                }
                Ok(())
            }
            DnsEvent::Error { operation, error } => {
                anyhow::bail!("DNS error ({}): {}", operation, error)
            }
            _ => anyhow::bail!("Unexpected event: {:?}", event),
        }
    })
}

fn parse_record_type(s: &str) -> Result<RecordType> {
    match s.to_uppercase().as_str() {
        "A" => Ok(RecordType::A),
        "AAAA" => Ok(RecordType::AAAA),
        "CNAME" => Ok(RecordType::CNAME),
        "MX" => Ok(RecordType::MX),
        "TXT" => Ok(RecordType::TXT),
        "NS" => Ok(RecordType::NS),
        "SRV" => Ok(RecordType::SRV),
        _ => anyhow::bail!("Invalid record type: {}", s),
    }
}

pub fn execute_find_record(
    _profile: &ProfileContext,
    zone_id: String,
    record_type_str: String,
    name: String,
) -> Result<()> {
    let record_type = parse_record_type(&record_type_str)?;

    smol::block_on(async {
        let event = send_dns_command(DnsCommand::FindRecord {
            zone_id,
            record_type,
            name: name.clone(),
        })
        .await?;

        match event {
            DnsEvent::RecordFound { zone_id: _, record } => {
                if let Some(record) = record {
                    println!("DNS Record Found:");
                    println!();
                    println!("  Name: {}", record.name);
                    println!("  Type: {}", record.record_type);
                    println!("  Content: {}", record.content);
                    println!("  ID: {}", record.id);
                    println!("  TTL: {}s", record.ttl);
                    println!("  Proxied: {}", record.proxied);
                } else {
                    println!("DNS record not found: {} {}", record_type_str, name);
                }
                Ok(())
            }
            DnsEvent::Error { operation, error } => {
                anyhow::bail!("DNS error ({}): {}", operation, error)
            }
            _ => anyhow::bail!("Unexpected event: {:?}", event),
        }
    })
}

pub fn execute_create_record(
    _profile: &ProfileContext,
    zone_id: String,
    record_type_str: String,
    name: String,
    content: String,
    ttl: Option<u32>,
    proxied: Option<bool>,
) -> Result<()> {
    let record_type = parse_record_type(&record_type_str)?;

    println!("Creating DNS record {} {}...", record_type_str, name);

    smol::block_on(async {
        let event = send_dns_command(DnsCommand::CreateRecord {
            zone_id,
            record_type,
            name,
            content,
            ttl,
            proxied,
        })
        .await?;

        match event {
            DnsEvent::RecordCreated {
                zone_id: _,
                name,
                record_type,
            } => {
                println!();
                println!("✓ DNS record created successfully!");
                println!();
                println!("  Name: {}", name);
                println!("  Type: {}", record_type);
                Ok(())
            }
            DnsEvent::Error { operation, error } => {
                anyhow::bail!("DNS error ({}): {}", operation, error)
            }
            _ => anyhow::bail!("Unexpected event: {:?}", event),
        }
    })
}

pub fn execute_update_record(
    _profile: &ProfileContext,
    zone_id: String,
    record_id: String,
    name: String,
    record_type_str: String,
    content: String,
    ttl: Option<u32>,
    proxied: Option<bool>,
) -> Result<()> {
    let record_type = parse_record_type(&record_type_str)?;

    println!("Updating DNS record {}...", record_id);

    smol::block_on(async {
        let event = send_dns_command(DnsCommand::UpdateRecord {
            zone_id,
            record_id: record_id.clone(),
            name,
            record_type,
            content,
            ttl,
            proxied,
        })
        .await?;

        match event {
            DnsEvent::RecordUpdated {
                zone_id: _,
                record_id,
            } => {
                println!();
                println!("✓ DNS record updated successfully!");
                println!();
                println!("  Record ID: {}", record_id);
                Ok(())
            }
            DnsEvent::Error { operation, error } => {
                anyhow::bail!("DNS error ({}): {}", operation, error)
            }
            _ => anyhow::bail!("Unexpected event: {:?}", event),
        }
    })
}

pub fn execute_delete_record(
    _profile: &ProfileContext,
    zone_id: String,
    record_id: String,
) -> Result<()> {
    println!("Deleting DNS record {}...", record_id);

    smol::block_on(async {
        let event = send_dns_command(DnsCommand::DeleteRecord {
            zone_id,
            record_id: record_id.clone(),
        })
        .await?;

        match event {
            DnsEvent::RecordDeleted {
                zone_id: _,
                record_id,
            } => {
                println!();
                println!("✓ DNS record deleted successfully!");
                println!();
                println!("  Record ID: {}", record_id);
                Ok(())
            }
            DnsEvent::Error { operation, error } => {
                anyhow::bail!("DNS error ({}): {}", operation, error)
            }
            _ => anyhow::bail!("Unexpected event: {:?}", event),
        }
    })
}

pub fn execute_upsert_record(
    _profile: &ProfileContext,
    zone_id: String,
    record_type_str: String,
    name: String,
    content: String,
    ttl: Option<u32>,
    proxied: Option<bool>,
) -> Result<()> {
    let record_type = parse_record_type(&record_type_str)?;

    println!(
        "Creating/updating DNS record {} {}...",
        record_type_str, name
    );

    smol::block_on(async {
        let event = send_dns_command(DnsCommand::UpsertRecord {
            zone_id,
            record_type,
            name,
            content,
            ttl,
            proxied,
        })
        .await?;

        match event {
            DnsEvent::RecordUpserted {
                zone_id: _,
                name,
                record_type,
            } => {
                println!();
                println!("✓ DNS record created/updated successfully!");
                println!();
                println!("  Name: {}", name);
                println!("  Type: {}", record_type);
                Ok(())
            }
            DnsEvent::Error { operation, error } => {
                anyhow::bail!("DNS error ({}): {}", operation, error)
            }
            _ => anyhow::bail!("Unexpected event: {:?}", event),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_profile() -> ProfileContext {
        // Create a minimal test profile
        ProfileContext {
            name: "test-profile".to_string(),
            kdbx_path: std::path::PathBuf::from("/tmp/test.kdbx"),
            kpkey_path: std::path::PathBuf::from("/tmp/test.keyx"),
        }
    }

    #[test]
    fn test_parse_record_type_valid() {
        assert!(matches!(parse_record_type("A"), Ok(RecordType::A)));
        assert!(matches!(parse_record_type("a"), Ok(RecordType::A)));
        assert!(matches!(parse_record_type("AAAA"), Ok(RecordType::AAAA)));
        assert!(matches!(parse_record_type("aaaa"), Ok(RecordType::AAAA)));
        assert!(matches!(parse_record_type("CNAME"), Ok(RecordType::CNAME)));
        assert!(matches!(parse_record_type("cname"), Ok(RecordType::CNAME)));
        assert!(matches!(parse_record_type("MX"), Ok(RecordType::MX)));
        assert!(matches!(parse_record_type("TXT"), Ok(RecordType::TXT)));
        assert!(matches!(parse_record_type("NS"), Ok(RecordType::NS)));
        assert!(matches!(parse_record_type("SRV"), Ok(RecordType::SRV)));
    }

    #[test]
    fn test_parse_record_type_invalid() {
        assert!(parse_record_type("INVALID").is_err());
        assert!(parse_record_type("").is_err());
        assert!(parse_record_type("123").is_err());
    }

    #[test]
    fn test_parse_record_type_case_insensitive() {
        assert!(matches!(parse_record_type("A"), Ok(RecordType::A)));
        assert!(matches!(parse_record_type("a"), Ok(RecordType::A)));
        assert!(matches!(parse_record_type("aAaA"), Ok(RecordType::AAAA)));
    }

    #[test]
    fn test_send_dns_command_helper() {
        // Test that send_dns_command helper function works
        smol::block_on(async {
            // Test with SetAccessToken (simple command that doesn't require external API)
            let result = send_dns_command(DnsCommand::SetAccessToken {
                access_token: "test-token".to_string(),
                refresh_token: None,
            })
            .await;

            assert!(result.is_ok());
            match result.unwrap() {
                DnsEvent::AccessTokenSet => {
                    // Success
                }
                other => panic!("Unexpected event: {:?}", other),
            }
        });
    }

    #[test]
    fn test_send_dns_command_error_no_token() {
        // Test that commands requiring token fail appropriately
        smol::block_on(async {
            let result = send_dns_command(DnsCommand::ListZones).await;

            assert!(result.is_ok());
            match result.unwrap() {
                DnsEvent::Error { operation, error } => {
                    assert_eq!(operation, "command");
                    assert!(error.contains("access token"));
                }
                other => panic!("Expected Error event, got: {:?}", other),
            }
        });
    }

    // Note: Full end-to-end tests with actual Cloudflare API calls
    // should be in integration tests with proper mocking or test accounts
}
