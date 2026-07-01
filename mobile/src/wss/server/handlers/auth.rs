//! Authentication message handlers

use crate::site::messages::{
    AuthLoginRequest, AuthLogoutRequest, AuthLogoutResponse, AuthResponse, DeviceInfo,
    ServerMessage, WebAuthnSigninBeginRequest, WebAuthnSigninBeginResponse,
    WebAuthnSigninFinishRequest, WebAuthnSigninFinishResponse, WebAuthnSignupBeginRequest,
    WebAuthnSignupBeginResponse, WebAuthnSignupFinishRequest, WebAuthnSignupFinishResponse,
};
use crate::storage::models::device::AuthenticatedDevice;
use crate::wss::server::ServerSettings;
use anyhow::Result;
use chrono::Utc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Handle authentication login request
pub async fn handle_login(
    req: AuthLoginRequest,
    session_id: &str,
    settings: &ServerSettings,
) -> Result<ServerMessage> {
    eprintln!("[Auth] Login request from device: {}", req.device_id);

    // Validate request
    if req.device_id.is_empty() || req.public_key.is_empty() {
        return Ok(ServerMessage::AuthResponse(AuthResponse {
            success: false,
            session_id: None,
            server_public_key: None,
            error: Some("Invalid device_id or public_key".to_string()),
            device_info: None,
            expires_at: None,
        }));
    }

    // Check if reconnecting with existing session
    if let Some(ref reconnect_session_id) = req.session_id {
        if let Ok(mut db) = settings.db.lock() {
            if let Ok(Some(_)) =
                crate::storage::models::session::get_session(&mut db, reconnect_session_id)
            {
                eprintln!("[Auth] Reconnecting session: {}", reconnect_session_id);
                // TODO: Validate the session belongs to this device
            }
        }
    }

    // Store authenticated device
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0))
        .as_secs();

    let device = AuthenticatedDevice {
        device_id: req.device_id.clone(),
        public_key: req.public_key.clone(),
        session_id: session_id.to_string(),
        authenticated_at: now,
        last_seen: now,
    };

    if let Ok(mut db) = settings.db.lock() {
        if let Err(e) = crate::storage::models::device::store_device_auth(&mut db, &device) {
            eprintln!("[Auth] Failed to store device auth: {}", e);
            return Ok(ServerMessage::AuthResponse(AuthResponse {
                success: false,
                session_id: None,
                server_public_key: None,
                error: Some("Failed to store authentication".to_string()),
                device_info: None,
                expires_at: None,
            }));
        }
    }

    // Build device info
    let device_info = DeviceInfo {
        device_id: req.device_id.clone(),
        device_name: None,
        platform: req.client_version.clone(),
        last_seen: Some(Utc::now()),
    };

    // TODO: Generate actual server public key for E2E encryption
    // For now, use a placeholder
    let server_public_key = Some(format!("SERVER_KEY_{}", settings.server_id));

    eprintln!(
        "[Auth] Successfully authenticated device: {}",
        req.device_id
    );

    Ok(ServerMessage::AuthResponse(AuthResponse {
        success: true,
        session_id: Some(session_id.to_string()),
        server_public_key,
        error: None,
        device_info: Some(device_info),
        expires_at: Some(Utc::now() + chrono::Duration::hours(24)),
    }))
}

/// Handle logout request
pub async fn handle_logout(
    req: AuthLogoutRequest,
    session_id: &str,
    settings: &ServerSettings,
) -> Result<ServerMessage> {
    eprintln!("[Auth] Logout request for session: {}", req.session_id);

    // Verify the session_id matches or allow logout of own session
    let target_session = if req.session_id.is_empty() {
        session_id
    } else {
        &req.session_id
    };

    // Delete session from database
    if let Ok(mut db) = settings.db.lock() {
        let _ = crate::storage::models::session::delete_session(&mut db, target_session);

        // Also remove device authentication
        if let Ok(Some(device_id)) =
            crate::storage::models::device::get_device_by_session(&mut db, target_session)
        {
            let _ = crate::storage::models::device::delete_device_auth(&mut db, &device_id);
        }
    }

    eprintln!("[Auth] Successfully logged out session: {}", target_session);

    Ok(ServerMessage::AuthLogoutResponse(AuthLogoutResponse {
        success: true,
        message: Some("Logged out successfully".to_string()),
    }))
}

// ============================================================================
// WebAuthn Handlers
// ============================================================================

/// Handle WebAuthn signup begin request
#[cfg(any())] // Disabled: go-webauthn crate excluded from workspace
pub async fn handle_webauthn_signup_begin(
    req: WebAuthnSignupBeginRequest,
) -> Result<WebAuthnSignupBeginResponse> {
    eprintln!("[WebAuthn] Signup begin for user: {}", req.username);

    let go_req = go_webauthn::SignupBeginRequest {
        username: req.username.clone(),
        display_name: req.display_name.clone(),
        scenario: req.scenario.clone(),
    };

    // Use LocalPool for !Send futures
    let (tx, rx) = futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let mut pool = futures::executor::LocalPool::new();
        // Use the safe wrapper which handles unsafe internally
        let response = pool.run_until(go_webauthn::webauthn_signup_begin(&go_req));
        let _ = tx.send(response);
    });

    let response = rx.await?;

    if response.success {
        eprintln!(
            "[WebAuthn] Signup begin successful, session: {}",
            response.session_id
        );
        Ok(WebAuthnSignupBeginResponse {
            success: true,
            session_id: Some(response.session_id),
            challenge_json: Some(response.challenge_json),
            error: None,
        })
    } else {
        eprintln!("[WebAuthn] Signup begin failed: {}", response.error);
        Ok(WebAuthnSignupBeginResponse {
            success: false,
            session_id: None,
            challenge_json: None,
            error: Some(response.error),
        })
    }
}

/// Handle WebAuthn signup finish request
#[cfg(any())] // Disabled: go-webauthn crate excluded from workspace
pub async fn handle_webauthn_signup_finish(
    req: WebAuthnSignupFinishRequest,
) -> Result<WebAuthnSignupFinishResponse> {
    eprintln!("[WebAuthn] Signup finish for session: {}", req.session_id);

    let go_req = go_webauthn::SignupFinishRequest {
        session_id: req.session_id.clone(),
        credential_json: req.credential_json.clone(),
    };

    // Use LocalPool for !Send futures
    let (tx, rx) = futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let mut pool = futures::executor::LocalPool::new();
        let response = pool.run_until(go_webauthn::webauthn_signup_finish(&go_req));
        let _ = tx.send(response);
    });

    let response = rx.await?;

    if response.success {
        eprintln!(
            "[WebAuthn] Signup finish successful, user: {}",
            response.user_id
        );
        Ok(WebAuthnSignupFinishResponse {
            success: true,
            user_id: Some(response.user_id),
            error: None,
        })
    } else {
        eprintln!("[WebAuthn] Signup finish failed: {}", response.error);
        Ok(WebAuthnSignupFinishResponse {
            success: false,
            user_id: None,
            error: Some(response.error),
        })
    }
}

/// Handle WebAuthn signin begin request
#[cfg(any())] // Disabled: go-webauthn crate excluded from workspace
pub async fn handle_webauthn_signin_begin(
    req: WebAuthnSigninBeginRequest,
) -> Result<WebAuthnSigninBeginResponse> {
    eprintln!(
        "[WebAuthn] Signin begin for user: {} (scenario: {})",
        req.username, req.scenario
    );

    let go_req = go_webauthn::SigninBeginRequest {
        username: req.username.clone(),
        scenario: req.scenario.clone(),
    };

    // Use LocalPool for !Send futures
    let (tx, rx) = futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let mut pool = futures::executor::LocalPool::new();
        let response = pool.run_until(go_webauthn::webauthn_signin_begin(&go_req));
        let _ = tx.send(response);
    });

    let response = rx.await?;

    if response.success {
        eprintln!(
            "[WebAuthn] Signin begin successful, session: {}",
            response.session_id
        );
        Ok(WebAuthnSigninBeginResponse {
            success: true,
            session_id: Some(response.session_id),
            challenge_json: Some(response.challenge_json),
            error: None,
        })
    } else {
        eprintln!("[WebAuthn] Signin begin failed: {}", response.error);
        Ok(WebAuthnSigninBeginResponse {
            success: false,
            session_id: None,
            challenge_json: None,
            error: Some(response.error),
        })
    }
}

/// Handle WebAuthn signin finish request
#[cfg(any())] // Disabled: go-webauthn crate excluded from workspace
pub async fn handle_webauthn_signin_finish(
    req: WebAuthnSigninFinishRequest,
) -> Result<WebAuthnSigninFinishResponse> {
    eprintln!("[WebAuthn] Signin finish for session: {}", req.session_id);

    let go_req = go_webauthn::SigninFinishRequest {
        session_id: req.session_id.clone(),
        credential_json: req.credential_json.clone(),
    };

    // Use LocalPool for !Send futures
    let (tx, rx) = futures::channel::oneshot::channel();
    std::thread::spawn(move || {
        let mut pool = futures::executor::LocalPool::new();
        let response = pool.run_until(go_webauthn::webauthn_signin_finish(&go_req));
        let _ = tx.send(response);
    });

    let response = rx.await?;

    if response.success {
        eprintln!(
            "[WebAuthn] Signin finish successful, user: {}",
            response.user_id
        );
        Ok(WebAuthnSigninFinishResponse {
            success: true,
            user_id: Some(response.user_id),
            session_token: Some(response.session_token),
            error: None,
        })
    } else {
        eprintln!("[WebAuthn] Signin finish failed: {}", response.error);
        Ok(WebAuthnSigninFinishResponse {
            success: false,
            user_id: None,
            session_token: None,
            error: Some(response.error),
        })
    }
}

// Stub implementations for WASM/Android (WebAuthn not supported via Go bridge)
#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub async fn handle_webauthn_signup_begin(
    _req: WebAuthnSignupBeginRequest,
) -> Result<WebAuthnSignupBeginResponse> {
    Ok(WebAuthnSignupBeginResponse {
        success: false,
        session_id: None,
        challenge_json: None,
        error: Some("WebAuthn not supported on this platform".to_string()),
    })
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub async fn handle_webauthn_signup_finish(
    _req: WebAuthnSignupFinishRequest,
) -> Result<WebAuthnSignupFinishResponse> {
    Ok(WebAuthnSignupFinishResponse {
        success: false,
        user_id: None,
        error: Some("WebAuthn not supported on this platform".to_string()),
    })
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub async fn handle_webauthn_signin_begin(
    _req: WebAuthnSigninBeginRequest,
) -> Result<WebAuthnSigninBeginResponse> {
    Ok(WebAuthnSigninBeginResponse {
        success: false,
        session_id: None,
        challenge_json: None,
        error: Some("WebAuthn not supported on this platform".to_string()),
    })
}

#[cfg(any(target_arch = "wasm32", target_os = "android"))]
pub async fn handle_webauthn_signin_finish(
    _req: WebAuthnSigninFinishRequest,
) -> Result<WebAuthnSigninFinishResponse> {
    Ok(WebAuthnSigninFinishResponse {
        success: false,
        user_id: None,
        session_token: None,
        error: Some("WebAuthn not supported on this platform".to_string()),
    })
}
