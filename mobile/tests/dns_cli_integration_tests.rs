//! DNS CLI integration tests
//!
//! These tests verify the end-to-end flow of DNS commands through the CLI.

use mobile::calc::profile::ProfileContext;
use mobile::viewmodel::common::ViewModelEvent;
use mobile::viewmodel::dns::{DnsActor, DnsCommand, DnsEvent, RecordType};
use smol::channel::bounded;
use std::path::PathBuf;

/// Create a test profile for integration tests
fn create_test_profile() -> ProfileContext {
    ProfileContext {
        name: "integration-test-profile".to_string(),
        kdbx_path: PathBuf::from("/tmp/test-integration.kdbx"),
        kpkey_path: PathBuf::from("/tmp/test-integration.keyx"),
    }
}

#[test]
fn test_dns_actor_set_token_flow() {
    smol::block_on(async {
        let (cmd_tx, cmd_rx) = bounded(10);
        let (evt_tx, evt_rx) = bounded(10);

        let actor = DnsActor::new(cmd_rx, evt_tx);
        smol::spawn(actor.run()).detach();

        // Send SetAccessToken command
        cmd_tx
            .send(DnsCommand::SetAccessToken {
                access_token: "test-token-123".to_string(),
                refresh_token: Some("refresh-token-456".to_string()),
            })
            .await
            .unwrap();

        // Wait for AccessTokenSet event
        let event = evt_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Dns(DnsEvent::AccessTokenSet) => {
                // Success
            }
            other => panic!("Expected AccessTokenSet event, got: {:?}", other),
        }
    });
}

#[test]
fn test_dns_actor_list_zones_no_token() {
    smol::block_on(async {
        let (cmd_tx, cmd_rx) = bounded(10);
        let (evt_tx, evt_rx) = bounded(10);

        let actor = DnsActor::new(cmd_rx, evt_tx);
        smol::spawn(actor.run()).detach();

        // Try to list zones without setting token
        cmd_tx.send(DnsCommand::ListZones).await.unwrap();

        // Should get Error event
        let event = evt_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Dns(DnsEvent::Error { operation, error }) => {
                assert_eq!(operation, "command");
                assert!(
                    error.contains("access token") || error.contains("No access token"),
                    "Error should mention access token: {}",
                    error
                );
            }
            other => panic!("Expected Error event, got: {:?}", other),
        }
    });
}

#[test]
fn test_dns_actor_domain_commands_no_token() {
    smol::block_on(async {
        let (cmd_tx, cmd_rx) = bounded(10);
        let (evt_tx, evt_rx) = bounded(10);

        let actor = DnsActor::new(cmd_rx, evt_tx);
        smol::spawn(actor.run()).detach();

        // Test ListDomains without token
        cmd_tx
            .send(DnsCommand::ListDomains {
                account_id: "test-account".to_string(),
            })
            .await
            .unwrap();

        let event = evt_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Dns(DnsEvent::Error { operation, error }) => {
                assert_eq!(operation, "command");
                assert!(error.contains("access token"));
            }
            other => panic!("Expected Error event, got: {:?}", other),
        }
    });
}

#[test]
fn test_dns_actor_check_availability_no_token() {
    smol::block_on(async {
        let (cmd_tx, cmd_rx) = bounded(10);
        let (evt_tx, evt_rx) = bounded(10);

        let actor = DnsActor::new(cmd_rx, evt_tx);
        smol::spawn(actor.run()).detach();

        // Test CheckAvailability without token
        cmd_tx
            .send(DnsCommand::CheckAvailability {
                account_id: "test-account".to_string(),
                domain: "test-domain.com".to_string(),
            })
            .await
            .unwrap();

        let event = evt_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Dns(DnsEvent::Error { operation, error }) => {
                assert_eq!(operation, "command");
                assert!(error.contains("access token"));
            }
            other => panic!("Expected Error event, got: {:?}", other),
        }
    });
}

#[test]
fn test_dns_actor_record_operations_no_token() {
    smol::block_on(async {
        let (cmd_tx, cmd_rx) = bounded(10);
        let (evt_tx, evt_rx) = bounded(10);

        let actor = DnsActor::new(cmd_rx, evt_tx);
        smol::spawn(actor.run()).detach();

        // Test ListRecords without token
        cmd_tx
            .send(DnsCommand::ListRecords {
                zone_id: "test-zone-123".to_string(),
            })
            .await
            .unwrap();

        let event = evt_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Dns(DnsEvent::Error { operation, error }) => {
                assert_eq!(operation, "command");
                assert!(error.contains("access token"));
            }
            other => panic!("Expected Error event, got: {:?}", other),
        }
    });
}

#[test]
fn test_dns_actor_multiple_commands_sequence() {
    smol::block_on(async {
        let (cmd_tx, cmd_rx) = bounded(10);
        let (evt_tx, evt_rx) = bounded(10);

        let actor = DnsActor::new(cmd_rx, evt_tx);
        smol::spawn(actor.run()).detach();

        // 1. Set access token
        cmd_tx
            .send(DnsCommand::SetAccessToken {
                access_token: "test-token".to_string(),
                refresh_token: None,
            })
            .await
            .unwrap();

        let event1 = evt_rx.recv().await.unwrap();
        assert!(matches!(
            event1,
            ViewModelEvent::Dns(DnsEvent::AccessTokenSet)
        ));

        // 2. Try to list zones (will fail with API error since token is fake)
        cmd_tx.send(DnsCommand::ListZones).await.unwrap();

        let event2 = evt_rx.recv().await.unwrap();
        // Should get an error (either auth error or network error)
        assert!(matches!(
            event2,
            ViewModelEvent::Dns(DnsEvent::Error { .. })
        ));
    });
}

#[test]
fn test_dns_record_type_variations() {
    // Test that all RecordType variants work
    let record_types = vec![
        RecordType::A,
        RecordType::AAAA,
        RecordType::CNAME,
        RecordType::MX,
        RecordType::TXT,
        RecordType::NS,
        RecordType::SRV,
    ];

    for record_type in record_types {
        // Verify each can be used in a command
        let _cmd = DnsCommand::FindRecord {
            zone_id: "test-zone".to_string(),
            record_type,
            name: "test.example.com".to_string(),
        };
    }
}

#[test]
fn test_dns_command_cloning() {
    // Verify that commands can be cloned
    let cmd = DnsCommand::ListDomains {
        account_id: "test-account".to_string(),
    };

    let cloned = cmd.clone();
    match cloned {
        DnsCommand::ListDomains { account_id } => {
            assert_eq!(account_id, "test-account");
        }
        _ => panic!("Clone changed command type"),
    }
}

#[test]
fn test_dns_event_cloning() {
    // Verify that events can be cloned
    let event = DnsEvent::DomainAvailability {
        domain: "example.com".to_string(),
        available: true,
    };

    let cloned = event.clone();
    match cloned {
        DnsEvent::DomainAvailability { domain, available } => {
            assert_eq!(domain, "example.com");
            assert!(available);
        }
        _ => panic!("Clone changed event type"),
    }
}

// Note: Tests that require actual Cloudflare API access should be:
// 1. Marked with #[ignore] by default
// 2. Only run when specific environment variables are set
// 3. Use test accounts with proper cleanup
//
// Example:
// #[test]
// #[ignore]
// fn test_cloudflare_api_real_account() {
//     // Requires CLOUDFLARE_API_TOKEN and CLOUDFLARE_ACCOUNT_ID env vars
//     // ...
// }
