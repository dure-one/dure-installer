//! WebSocket message handlers

pub mod auth;

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use crate::wss::server::messages::{ClientMessage, ErrorResponse, ServerMessage};
use crate::wss::server::ServerSettings;
use anyhow::Result;

/// Handle incoming client message and return server response
pub async fn handle_client_message(
    msg: ClientMessage,
    session_id: &str,
    settings: &ServerSettings,
) -> Result<ServerMessage> {
    match msg {
        ClientMessage::AuthLogin(req) => auth::handle_login(req, session_id, settings).await,
        ClientMessage::AuthLogout(req) => auth::handle_logout(req, session_id, settings).await,
        // WebAuthn handlers disabled: go-webauthn crate excluded from workspace
        ClientMessage::WebAuthnSignupBegin(_) => Ok(ServerMessage::Error(ErrorResponse {
            code: "NOT_AVAILABLE".to_string(),
            message: "WebAuthn server not available (go-webauthn crate excluded)".to_string(),
            request_id: None,
            details: None,
        })),
        ClientMessage::WebAuthnSignupFinish(_) => Ok(ServerMessage::Error(ErrorResponse {
            code: "NOT_AVAILABLE".to_string(),
            message: "WebAuthn server not available (go-webauthn crate excluded)".to_string(),
            request_id: None,
            details: None,
        })),
        ClientMessage::WebAuthnSigninBegin(_) => Ok(ServerMessage::Error(ErrorResponse {
            code: "NOT_AVAILABLE".to_string(),
            message: "WebAuthn server not available (go-webauthn crate excluded)".to_string(),
            request_id: None,
            details: None,
        })),
        ClientMessage::WebAuthnSigninFinish(_) => Ok(ServerMessage::Error(ErrorResponse {
            code: "NOT_AVAILABLE".to_string(),
            message: "WebAuthn server not available (go-webauthn crate excluded)".to_string(),
            request_id: None,
            details: None,
        })),
        // TODO: Add other handlers as they are implemented
        _ => Ok(ServerMessage::Error(ErrorResponse {
            code: "NOT_IMPLEMENTED".to_string(),
            message: "Handler not implemented for this message type".to_string(),
            request_id: None,
            details: None,
        })),
    }
}
