//! Tests for Cloudflare Registrar API

use dure::api::cloudflare::registrar::{CloudflareRegistrarClient, Domain};

#[test]
fn test_registrar_client_creation() {
    let client = CloudflareRegistrarClient::new("test_token_123".into());
    assert_eq!(client.api_token(), "test_token_123");
}

#[test]
fn test_domain_structure() {
    let domain = Domain {
        id: "domain123".to_string(),
        name: "example.com".to_string(),
        status: "active".to_string(),
        nameservers: vec![
            "ns1.cloudflare.com".to_string(),
            "ns2.cloudflare.com".to_string(),
        ],
        created_at: Some("2024-01-01T00:00:00Z".to_string()),
        expires_at: Some("2025-01-01T00:00:00Z".to_string()),
    };

    assert_eq!(domain.name, "example.com");
    assert_eq!(domain.status, "active");
    assert_eq!(domain.nameservers.len(), 2);
}

#[test]
fn test_domain_has_cloudflare_nameservers() {
    let domain = Domain {
        id: "domain123".to_string(),
        name: "example.com".to_string(),
        status: "active".to_string(),
        nameservers: vec![
            "ns1.cloudflare.com".to_string(),
            "ns2.cloudflare.com".to_string(),
        ],
        created_at: None,
        expires_at: None,
    };

    assert!(domain.has_cloudflare_nameservers());
}

#[test]
fn test_domain_without_cloudflare_nameservers() {
    let domain = Domain {
        id: "domain123".to_string(),
        name: "example.com".to_string(),
        status: "active".to_string(),
        nameservers: vec![
            "ns1.example.com".to_string(),
            "ns2.example.com".to_string(),
        ],
        created_at: None,
        expires_at: None,
    };

    assert!(!domain.has_cloudflare_nameservers());
}

#[test]
fn test_nameserver_list_formatting() {
    let nameservers = vec![
        "ns1.cloudflare.com".to_string(),
        "ns2.cloudflare.com".to_string(),
    ];

    let formatted = nameservers.join(", ");
    assert_eq!(formatted, "ns1.cloudflare.com, ns2.cloudflare.com");
}
