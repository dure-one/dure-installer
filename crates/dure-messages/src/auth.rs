//! Authentication message types

use asyncapi_rust::schemars::JsonSchema;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Authentication login request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct AuthLoginRequest {
    /// Device ID (from machine-id)
    pub device_id: String,
    /// Device public key for encryption
    pub public_key: String,
    /// Optional session ID for reconnection
    pub session_id: Option<String>,
    /// Client version
    pub client_version: Option<String>,
}

/// Authentication response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct AuthResponse {
    /// Whether authentication was successful
    pub success: bool,
    /// Session ID for this connection
    pub session_id: Option<String>,
    /// Server public key for encryption
    pub server_public_key: Option<String>,
    /// Error message if authentication failed
    pub error: Option<String>,
    /// Authenticated user/device information
    pub device_info: Option<DeviceInfo>,
    /// Session expiry time
    pub expires_at: Option<DateTime<Utc>>,
}

/// Device information
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct DeviceInfo {
    /// Device ID
    pub device_id: String,
    /// Device name/hostname
    pub device_name: Option<String>,
    /// Device platform (linux, windows, macos, android)
    pub platform: Option<String>,
    /// Last seen timestamp
    pub last_seen: Option<DateTime<Utc>>,
}

/// Logout request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct AuthLogoutRequest {
    /// Session ID to logout
    pub session_id: String,
}

/// Logout response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct AuthLogoutResponse {
    /// Whether logout was successful
    pub success: bool,
    /// Optional message
    pub message: Option<String>,
}

// ============================================================================
// WebAuthn Messages
// ============================================================================

/// Request to begin WebAuthn registration (signup)
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct WebAuthnSignupBeginRequest {
    /// Username for the account
    pub username: String,
    /// Display name shown to user
    pub display_name: String,
    /// Authentication scenario: "mfa", "passwordless", or "usernameless"
    pub scenario: String,
}

/// Response from beginning WebAuthn registration
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct WebAuthnSignupBeginResponse {
    pub success: bool,
    /// Session ID to use for finish request
    pub session_id: Option<String>,
    /// JSON-serialized WebAuthn creation challenge
    pub challenge_json: Option<String>,
    pub error: Option<String>,
}

/// Request to finish WebAuthn registration
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct WebAuthnSignupFinishRequest {
    /// Session ID from begin response
    pub session_id: String,
    /// JSON-serialized credential from navigator.credentials.create()
    pub credential_json: String,
}

/// Response from finishing WebAuthn registration
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct WebAuthnSignupFinishResponse {
    pub success: bool,
    /// User ID if successful
    pub user_id: Option<String>,
    pub error: Option<String>,
}

/// Request to begin WebAuthn authentication (signin)
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct WebAuthnSigninBeginRequest {
    /// Username (empty for usernameless scenario)
    pub username: String,
    /// Authentication scenario: "mfa", "passwordless", or "usernameless"
    pub scenario: String,
}

/// Response from beginning WebAuthn authentication
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct WebAuthnSigninBeginResponse {
    pub success: bool,
    /// Session ID to use for finish request
    pub session_id: Option<String>,
    /// JSON-serialized WebAuthn assertion challenge
    pub challenge_json: Option<String>,
    pub error: Option<String>,
}

/// Request to finish WebAuthn authentication
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct WebAuthnSigninFinishRequest {
    /// Session ID from begin response
    pub session_id: String,
    /// JSON-serialized credential from navigator.credentials.get()
    pub credential_json: String,
}

/// Response from finishing WebAuthn authentication
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct WebAuthnSigninFinishResponse {
    pub success: bool,
    /// User ID if successful
    pub user_id: Option<String>,
    /// Session token for authenticated session
    pub session_token: Option<String>,
    pub error: Option<String>,
}
