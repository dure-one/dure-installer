#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::viewmodel::common::ViewModelEvent;
    use futures::select;
    use futures::FutureExt;

    // Helper to create test actor
    fn create_test_actor() -> (
        smol::channel::Sender<DnsCommand>,
        smol::channel::Receiver<ViewModelEvent>,
    ) {
        let (cmd_tx, cmd_rx) = smol::channel::unbounded();
        let (event_tx, event_rx) = smol::channel::unbounded();

        let actor = DnsActor::new(cmd_rx, event_tx);
        smol::spawn(actor.run()).detach();

        (cmd_tx, event_rx)
    }

    // Helper to wait for specific event with timeout
    async fn wait_for_event(
        event_rx: &smol::channel::Receiver<ViewModelEvent>,
        timeout_secs: u64,
    ) -> Result<DnsEvent, String> {
        let timeout = smol::Timer::after(std::time::Duration::from_secs(timeout_secs));
        smol::pin!(timeout);

        select! {
            event = event_rx.recv().fuse() => {
                match event {
                    Ok(ViewModelEvent::Dns(dns_event)) => Ok(dns_event),
                    Ok(other) => Err(format!("Unexpected event type: {:?}", other)),
                    Err(e) => Err(format!("Channel error: {}", e)),
                }
            }
            _ = timeout.as_mut().fuse() => Err("Timeout waiting for event".to_string()),
        }
    }

    #[test]
    fn test_set_access_token() {
        smol::block_on(async {
            let (cmd_tx, event_rx) = create_test_actor();

            cmd_tx
                .send(DnsCommand::SetAccessToken {
                    access_token: "test-token".to_string(),
                    refresh_token: Some("refresh-token".to_string()),
                })
                .await
                .unwrap();

            let event = wait_for_event(&event_rx, 5).await.unwrap();
            assert!(matches!(event, DnsEvent::AccessTokenSet));
        });
    }

    #[test]
    fn test_list_zones_without_token() {
        smol::block_on(async {
            let (cmd_tx, event_rx) = create_test_actor();

            cmd_tx.send(DnsCommand::ListZones).await.unwrap();

            let event = wait_for_event(&event_rx, 5).await.unwrap();
            match event {
                DnsEvent::Error { operation, error } => {
                    assert_eq!(operation, "command");
                    assert!(error.contains("access token"));
                }
                other => panic!("Expected Error event, got: {:?}", other),
            }
        });
    }

    #[test]
    fn test_list_domains_without_token() {
        smol::block_on(async {
            let (cmd_tx, event_rx) = create_test_actor();

            cmd_tx
                .send(DnsCommand::ListDomains {
                    account_id: "test-account".to_string(),
                })
                .await
                .unwrap();

            let event = wait_for_event(&event_rx, 5).await.unwrap();
            match event {
                DnsEvent::Error { operation, error } => {
                    assert_eq!(operation, "command");
                    assert!(error.contains("access token"));
                }
                other => panic!("Expected Error event, got: {:?}", other),
            }
        });
    }

    #[test]
    fn test_check_availability_without_token() {
        smol::block_on(async {
            let (cmd_tx, event_rx) = create_test_actor();

            cmd_tx
                .send(DnsCommand::CheckAvailability {
                    account_id: "test-account".to_string(),
                    domain: "example.com".to_string(),
                })
                .await
                .unwrap();

            let event = wait_for_event(&event_rx, 5).await.unwrap();
            match event {
                DnsEvent::Error { operation, error } => {
                    assert_eq!(operation, "command");
                    assert!(error.contains("access token"));
                }
                other => panic!("Expected Error event, got: {:?}", other),
            }
        });
    }

    #[test]
    fn test_domain_commands_compile() {
        // Verify all domain registration commands compile and can be constructed
        let _cmd1 = DnsCommand::ListDomains {
            account_id: "test".to_string(),
        };
        let _cmd2 = DnsCommand::GetDomain {
            account_id: "test".to_string(),
            domain: "example.com".to_string(),
        };
        let _cmd3 = DnsCommand::CheckAvailability {
            account_id: "test".to_string(),
            domain: "example.com".to_string(),
        };
        let _cmd4 = DnsCommand::RegisterDomain {
            account_id: "test".to_string(),
            domain: "example.com".to_string(),
            years: 1,
        };
        let _cmd5 = DnsCommand::UpdateNameservers {
            account_id: "test".to_string(),
            domain: "example.com".to_string(),
            nameservers: vec!["ns1.example.com".to_string()],
        };
    }

    #[test]
    fn test_zone_commands_compile() {
        // Verify all zone commands compile
        let _cmd1 = DnsCommand::ListZones;
        let _cmd2 = DnsCommand::FindZone {
            domain: "example.com".to_string(),
        };
        let _cmd3 = DnsCommand::CreateZone {
            domain: "example.com".to_string(),
            account_id: "test".to_string(),
        };
    }

    #[test]
    fn test_record_commands_compile() {
        // Verify all record commands compile
        let _cmd1 = DnsCommand::ListRecords {
            zone_id: "zone-123".to_string(),
        };
        let _cmd2 = DnsCommand::FindRecord {
            zone_id: "zone-123".to_string(),
            record_type: RecordType::A,
            name: "example.com".to_string(),
        };
        let _cmd3 = DnsCommand::CreateRecord {
            zone_id: "zone-123".to_string(),
            record_type: RecordType::A,
            name: "example.com".to_string(),
            content: "1.2.3.4".to_string(),
            ttl: Some(300),
            proxied: Some(false),
        };
        let _cmd4 = DnsCommand::UpdateRecord {
            zone_id: "zone-123".to_string(),
            record_id: "rec-456".to_string(),
            name: "example.com".to_string(),
            record_type: RecordType::A,
            content: "1.2.3.4".to_string(),
            ttl: Some(300),
            proxied: Some(false),
        };
        let _cmd5 = DnsCommand::DeleteRecord {
            zone_id: "zone-123".to_string(),
            record_id: "rec-456".to_string(),
        };
        let _cmd6 = DnsCommand::UpsertRecord {
            zone_id: "zone-123".to_string(),
            record_type: RecordType::A,
            name: "example.com".to_string(),
            content: "1.2.3.4".to_string(),
            ttl: Some(300),
            proxied: Some(false),
        };
    }

    #[test]
    fn test_domain_events_compile() {
        // Verify all domain registration events compile
        let _ev1 = DnsEvent::DomainsListed {
            domains: vec![DomainInfo {
                id: "dom-123".to_string(),
                name: "example.com".to_string(),
                status: "active".to_string(),
                nameservers: vec!["ns1.cloudflare.com".to_string()],
                created_at: Some("2024-01-01".to_string()),
                expires_at: Some("2025-01-01".to_string()),
            }],
        };
        let _ev2 = DnsEvent::DomainInfo {
            domain: DomainInfo {
                id: "dom-123".to_string(),
                name: "example.com".to_string(),
                status: "active".to_string(),
                nameservers: vec![],
                created_at: None,
                expires_at: None,
            },
        };
        let _ev3 = DnsEvent::DomainAvailability {
            domain: "example.com".to_string(),
            available: true,
        };
        let _ev4 = DnsEvent::DomainRegistered {
            domain: DomainInfo {
                id: "dom-123".to_string(),
                name: "example.com".to_string(),
                status: "active".to_string(),
                nameservers: vec![],
                created_at: None,
                expires_at: None,
            },
        };
        let _ev5 = DnsEvent::NameserversUpdated {
            domain: "example.com".to_string(),
        };
    }

    #[test]
    fn test_record_type_as_str() {
        assert_eq!(RecordType::A.as_str(), "A");
        assert_eq!(RecordType::AAAA.as_str(), "AAAA");
        assert_eq!(RecordType::CNAME.as_str(), "CNAME");
        assert_eq!(RecordType::MX.as_str(), "MX");
        assert_eq!(RecordType::TXT.as_str(), "TXT");
        assert_eq!(RecordType::NS.as_str(), "NS");
        assert_eq!(RecordType::SRV.as_str(), "SRV");
    }

    #[test]
    fn test_domain_info_clone() {
        let domain = DomainInfo {
            id: "dom-123".to_string(),
            name: "example.com".to_string(),
            status: "active".to_string(),
            nameservers: vec!["ns1.cloudflare.com".to_string()],
            created_at: Some("2024-01-01".to_string()),
            expires_at: Some("2025-01-01".to_string()),
        };

        let cloned = domain.clone();
        assert_eq!(cloned.id, "dom-123");
        assert_eq!(cloned.name, "example.com");
        assert_eq!(cloned.status, "active");
        assert_eq!(cloned.nameservers.len(), 1);
    }
}
