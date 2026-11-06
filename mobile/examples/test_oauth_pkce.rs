//! Simple example to test OAuth PKCE generation

use dure::api::cloudflare::oauth::{CloudflareOAuthHandler, generate_pkce_pair};

fn main() {
    println!("Testing Cloudflare OAuth PKCE implementation\n");

    // Test 1: PKCE pair generation
    println!("Test 1: Generating PKCE pair...");
    let (verifier, challenge) = generate_pkce_pair();
    println!("  ✓ code_verifier length: {}", verifier.len());
    println!("  ✓ code_challenge length: {}", challenge.len());
    println!("  ✓ verifier: {}...", &verifier[..20]);
    println!("  ✓ challenge: {}...", &challenge[..20]);
    assert!(verifier.len() >= 43 && verifier.len() <= 128);
    assert_eq!(challenge.len(), 43);

    // Test 2: OAuth handler creation
    println!("\nTest 2: Creating OAuth handler...");
    let handler = CloudflareOAuthHandler::new(
        "test_client_id".to_string(),
        "test_secret".to_string(),
    );
    println!("  ✓ client_id: {}", handler.client_id());
    assert_eq!(handler.client_id(), "test_client_id");

    // Test 3: Build auth URL
    println!("\nTest 3: Building OAuth URL...");
    let url = handler
        .build_auth_url("http://localhost:8080/callback", "test_state")
        .unwrap();
    println!("  ✓ URL: {}...", &url[..80]);
    assert!(url.contains("client_id=test_client_id"));
    assert!(url.contains("code_challenge="));
    assert!(url.contains("code_challenge_method=S256"));
    assert!(url.starts_with("https://dash.cloudflare.com/oauth2/auth"));

    println!("\n✅ All OAuth PKCE tests passed!");
}
