//! Simple example to test Cloudflare Registrar API

use dure::api::cloudflare::registrar::{CloudflareRegistrarClient, Domain};

fn main() {
    println!("Testing Cloudflare Registrar API implementation\n");

    // Test 1: Client creation
    println!("Test 1: Creating Registrar client...");
    let client = CloudflareRegistrarClient::new("test_api_token_123".to_string());
    println!("  ✓ Client created with token: {}...", &client.api_token()[..15]);
    assert_eq!(client.api_token(), "test_api_token_123");

    // Test 2: Domain structure
    println!("\nTest 2: Testing Domain structure...");
    let domain = Domain {
        id: "domain_abc123".to_string(),
        name: "example.com".to_string(),
        status: "active".to_string(),
        nameservers: vec![
            "ns1.cloudflare.com".to_string(),
            "ns2.cloudflare.com".to_string(),
        ],
        created_at: Some("2024-01-01T00:00:00Z".to_string()),
        expires_at: Some("2025-01-01T00:00:00Z".to_string()),
    };
    println!("  ✓ Domain: {}", domain.name);
    println!("  ✓ Status: {}", domain.status);
    println!("  ✓ Nameservers: {}", domain.nameservers.len());
    assert_eq!(domain.name, "example.com");
    assert_eq!(domain.nameservers.len(), 2);

    // Test 3: Cloudflare nameserver detection
    println!("\nTest 3: Checking Cloudflare nameserver detection...");
    assert!(domain.has_cloudflare_nameservers());
    println!("  ✓ Correctly identified Cloudflare nameservers");

    let status = domain.nameserver_status();
    println!("  ✓ Status: {}", status);
    assert!(status.contains("✓ Using Cloudflare NS"));

    // Test 4: External nameservers
    println!("\nTest 4: Testing external nameservers...");
    let external_domain = Domain {
        id: "domain_xyz789".to_string(),
        name: "test.com".to_string(),
        status: "active".to_string(),
        nameservers: vec![
            "ns1.example.com".to_string(),
            "ns2.example.com".to_string(),
        ],
        created_at: None,
        expires_at: None,
    };
    assert!(!external_domain.has_cloudflare_nameservers());
    println!("  ✓ Correctly identified external nameservers");

    let ext_status = external_domain.nameserver_status();
    println!("  ✓ Status: {}", ext_status);
    assert!(ext_status.contains("⚠ External NS"));

    println!("\n✅ All Registrar API tests passed!");
    println!("\nNote: Actual API calls require:");
    println!("  - Valid Cloudflare API token");
    println!("  - Valid account ID");
    println!("  - Network connectivity to api.cloudflare.com");
}
