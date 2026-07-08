//! Integration tests for message serialization and validation

use dure_messages::*;

#[test]
fn test_client_message_serialization() {
    // Test that ClientMessage can serialize/deserialize
    let msg = ClientMessage::AuthLogin(AuthLoginRequest {
        device_id: "test-device".to_string(),
        public_key: "test-pubkey".to_string(),
        session_id: None,
        client_version: None,
    });

    let json = serde_json::to_string(&msg).expect("should serialize");
    let deserialized: ClientMessage = serde_json::from_str(&json).expect("should deserialize");

    // Round-trip should succeed
    assert!(matches!(deserialized, ClientMessage::AuthLogin(_)));
}

#[test]
fn test_server_message_serialization() {
    // Test that ServerMessage can serialize/deserialize
    let msg = ServerMessage::AuthResponse(AuthResponse {
        success: true,
        session_id: Some("session-123".to_string()),
        server_public_key: None,
        error: None,
        device_info: None,
        expires_at: None,
    });

    let json = serde_json::to_string(&msg).expect("should serialize");
    let deserialized: ServerMessage = serde_json::from_str(&json).expect("should deserialize");

    assert!(matches!(deserialized, ServerMessage::AuthResponse(_)));
}

#[test]
fn test_message_types_are_send_sync() {
    // Ensure types can be used in async contexts
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ClientMessage>();
    assert_send_sync::<ServerMessage>();
    assert_send_sync::<ErrorResponse>();
}

#[test]
fn test_json_schema_generation() {
    use schemars::schema_for;

    // Verify schemars integration works
    let schema = schema_for!(ClientMessage);
    assert!(!format!("{:?}", schema).is_empty());

    let schema = schema_for!(ServerMessage);
    assert!(!format!("{:?}", schema).is_empty());
}
