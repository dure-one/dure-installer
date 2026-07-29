//! Message type definitions for WebSocket communication
//!
//! These types should eventually be moved to a separate `dure-messages` crate
//! that can be shared between client and server.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    AuthLogin(AuthLoginRequest),
    AuthLogout(AuthLogoutRequest),
    WebAuthnSignupBegin(WebAuthnSignupBeginRequest),
    WebAuthnSignupFinish(WebAuthnSignupFinishRequest),
    WebAuthnSigninBegin(WebAuthnSigninBeginRequest),
    WebAuthnSigninFinish(WebAuthnSigninFinishRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    AuthResponse(AuthResponse),
    AuthLogoutResponse(AuthLogoutResponse),
    WebAuthnSignupBeginResponse(WebAuthnSignupBeginResponse),
    WebAuthnSignupFinishResponse(WebAuthnSignupFinishResponse),
    WebAuthnSigninBeginResponse(WebAuthnSigninBeginResponse),
    WebAuthnSigninFinishResponse(WebAuthnSigninFinishResponse),
    Error(ErrorResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthLoginRequest {
    pub device_id: String,
    pub public_key: String,
    pub session_id: Option<String>,
    pub device_info: Option<DeviceInfo>,
    pub client_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthLogoutRequest {
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub session_id: Option<String>,
    pub server_public_key: Option<String>,
    pub error: Option<String>,
    pub device_info: Option<DeviceInfo>,
    pub expires_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthLogoutResponse {
    pub success: bool,
    pub message: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub device_name: Option<String>,
    pub platform: String,
    pub os_version: Option<String>,
    pub app_version: Option<String>,
    pub last_seen: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnSignupBeginRequest {
    pub username: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnSignupBeginResponse {
    pub challenge: String,
    pub user_id: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnSignupFinishRequest {
    pub credential: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnSignupFinishResponse {
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnSigninBeginRequest {
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnSigninBeginResponse {
    pub challenge: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnSigninFinishRequest {
    pub credential: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebAuthnSigninFinishResponse {
    pub success: bool,
    pub session_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub request_id: Option<String>,
    pub details: Option<String>,
}
