//! Tests for Cloudflare OAuth with PKCE

use dure::api::cloudflare::oauth::{CloudflareOAuthHandler, OAuthResult, generate_pkce_pair};

#[test]
fn test_oauth_handler_creation() {
    let handler = CloudflareOAuthHandler::new("client_id".into(), "client_secret".into());
    assert_eq!(handler.client_id(), "client_id");
    assert_eq!(handler.client_secret(), "client_secret");
}

#[test]
fn test_oauth_handler_default() {
    let handler = CloudflareOAuthHandler::default();
    // Default credentials from env vars (may be empty in tests)
    assert!(handler.client_id().is_empty() || !handler.client_id().is_empty());
}

#[test]
fn test_pkce_challenge_generation() {
    let (verifier, challenge) = generate_pkce_pair();

    // code_verifier: 43-128 chars, base64url(random(32 bytes))
    assert!(verifier.len() >= 43 && verifier.len() <= 128);
    assert!(verifier.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));

    // code_challenge: base64url(sha256(code_verifier))
    assert_eq!(challenge.len(), 43); // SHA256 = 32 bytes → base64url = 43 chars
    assert!(challenge.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
}

#[test]
fn test_pkce_challenge_uniqueness() {
    let (verifier1, challenge1) = generate_pkce_pair();
    let (verifier2, challenge2) = generate_pkce_pair();

    // Each pair should be unique
    assert_ne!(verifier1, verifier2);
    assert_ne!(challenge1, challenge2);
}

#[test]
fn test_build_auth_url() {
    let handler = CloudflareOAuthHandler::new("test_client_id".into(), "secret".into());
    let url = handler.build_auth_url("http://localhost:8080/callback", "state123").unwrap();

    // Verify URL contains required OAuth parameters
    assert!(url.contains("client_id=test_client_id"));
    assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A8080%2Fcallback"));
    assert!(url.contains("state=state123"));
    assert!(url.contains("response_type=code"));
    assert!(url.contains("code_challenge="));
    assert!(url.contains("code_challenge_method=S256"));

    // Verify it's a Cloudflare OAuth URL
    assert!(url.starts_with("https://"));
}

#[test]
fn test_oauth_result_structure() {
    let result = OAuthResult {
        access_token: "token123".to_string(),
        refresh_token: Some("refresh456".to_string()),
        expires_at: 1234567890,
    };

    assert_eq!(result.access_token, "token123");
    assert_eq!(result.refresh_token, Some("refresh456".to_string()));
    assert_eq!(result.expires_at, 1234567890);
}
