//! WebSocket message handlers

pub mod auth;

use crate::site::messages::{ClientMessage, ErrorResponse, ServerMessage};
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
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "openbsd")))]
        ClientMessage::WebAuthnSignupBegin(req) => {
            let response = auth::handle_webauthn_signup_begin(req).await?;
            Ok(ServerMessage::WebAuthnSignupBeginResponse(response))
        }
        #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "openbsd"))]
        ClientMessage::WebAuthnSignupBegin(_) => Ok(ServerMessage::Error(ErrorResponse {
            code: "NOT_AVAILABLE".to_string(),
            message: "WebAuthn not available on this platform".to_string(),
            request_id: None,
            details: None,
        })),
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "openbsd")))]
        ClientMessage::WebAuthnSignupFinish(req) => {
            let response = auth::handle_webauthn_signup_finish(req).await?;
            Ok(ServerMessage::WebAuthnSignupFinishResponse(response))
        }
        #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "openbsd"))]
        ClientMessage::WebAuthnSignupFinish(_) => Ok(ServerMessage::Error(ErrorResponse {
            code: "NOT_AVAILABLE".to_string(),
            message: "WebAuthn not available on this platform".to_string(),
            request_id: None,
            details: None,
        })),
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "openbsd")))]
        ClientMessage::WebAuthnSigninBegin(req) => {
            let response = auth::handle_webauthn_signin_begin(req).await?;
            Ok(ServerMessage::WebAuthnSigninBeginResponse(response))
        }
        #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "openbsd"))]
        ClientMessage::WebAuthnSigninBegin(_) => Ok(ServerMessage::Error(ErrorResponse {
            code: "NOT_AVAILABLE".to_string(),
            message: "WebAuthn not available on this platform".to_string(),
            request_id: None,
            details: None,
        })),
        #[cfg(not(any(target_arch = "wasm32", target_os = "android", target_os = "openbsd")))]
        ClientMessage::WebAuthnSigninFinish(req) => {
            let response = auth::handle_webauthn_signin_finish(req).await?;
            Ok(ServerMessage::WebAuthnSigninFinishResponse(response))
        }
        #[cfg(any(target_arch = "wasm32", target_os = "android", target_os = "openbsd"))]
        ClientMessage::WebAuthnSigninFinish(_) => Ok(ServerMessage::Error(ErrorResponse {
            code: "NOT_AVAILABLE".to_string(),
            message: "WebAuthn not available on this platform".to_string(),
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
