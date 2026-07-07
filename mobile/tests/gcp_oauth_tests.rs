//! Tests for api/gcp/oauth module

use dure::api::gcp::oauth::{OAuthHandler, OAuthResult};

#[test]
fn test_oauth_result_structure() {
    let result = OAuthResult {
        refresh_token: "refresh".to_string(),
        access_token: "access".to_string(),
        expires_at: 1234567890,
    };

    assert_eq!(result.refresh_token, "refresh");
    assert_eq!(result.access_token, "access");
    assert_eq!(result.expires_at, 1234567890);
}

#[test]
fn test_oauth_handler_creation() {
    let handler = OAuthHandler::new(
        "client_id".to_string(),
        "client_secret".to_string(),
    );

    assert_eq!(handler.client_id(), "client_id");
    assert_eq!(handler.client_secret(), "client_secret");
}
