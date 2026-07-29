//! WebAuthn authentication support for WebSocket connections
//!
//! Provides passkey-based authentication using WebAuthn protocol via go-webauthn-client.
//! Uses CLI-based JSON-RPC communication (works on all platforms including OpenBSD).
//!
//! NOTE: Currently disabled - go_webauthn_client crate not available.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::Result;
// Disabled: go_webauthn_client crate not available
// use go_webauthn_client::*;
use std::sync::{Arc, Mutex};

// Stub types for go_webauthn_client (disabled feature)
#[derive(Clone)]
#[allow(dead_code)]
struct GoWebAuthnClient;

impl GoWebAuthnClient {
    fn new(_config: Option<String>) -> Result<Self> {
        Ok(GoWebAuthnClient)
    }

    fn webauthn_signup_begin(&mut self, _params: WebAuthnSignupBeginParams) -> Result<WebAuthnSignupBeginResult> {
        Err(anyhow::anyhow!("WebAuthn not available"))
    }

    fn webauthn_signup_finish(&mut self, _params: WebAuthnSignupFinishParams) -> Result<WebAuthnSignupFinishResult> {
        Err(anyhow::anyhow!("WebAuthn not available"))
    }

    fn webauthn_signin_begin(&mut self, _params: WebAuthnSigninBeginParams) -> Result<WebAuthnSigninBeginResult> {
        Err(anyhow::anyhow!("WebAuthn not available"))
    }

    fn webauthn_signin_finish(&mut self, _params: WebAuthnSigninFinishParams) -> Result<WebAuthnSigninFinishResult> {
        Err(anyhow::anyhow!("WebAuthn not available"))
    }

    fn webauthn_passkey_login_begin(&mut self, _params: WebAuthnPasskeyLoginBeginParams) -> Result<WebAuthnPasskeyLoginBeginResult> {
        Err(anyhow::anyhow!("WebAuthn not available"))
    }

    fn webauthn_passkey_login_finish(&mut self, _params: WebAuthnPasskeyLoginFinishParams) -> Result<WebAuthnPasskeyLoginFinishResult> {
        Err(anyhow::anyhow!("WebAuthn not available"))
    }

    fn webauthn_mfa_login_begin(&mut self, _params: WebAuthnMfaLoginBeginParams) -> Result<WebAuthnMfaLoginBeginResult> {
        Err(anyhow::anyhow!("WebAuthn not available"))
    }

    fn webauthn_mfa_login_finish(&mut self, _params: WebAuthnMfaLoginFinishParams) -> Result<WebAuthnMfaLoginFinishResult> {
        Err(anyhow::anyhow!("WebAuthn not available"))
    }
}

#[allow(dead_code)]
#[allow(dead_code)]
struct WebAuthnSignupBeginParams {
    rp_display_name: String,
    rp_id: String,
    rp_origins: String,
    username: String,
    display_name: String,
    scenario: String,
}

#[allow(dead_code)]
struct WebAuthnSignupBeginResult {
    session_id: String,
    challenge_json: String,
}

#[allow(dead_code)]
struct WebAuthnSignupFinishParams {
    session_id: String,
    credential_json: String,
}

#[allow(dead_code)]
struct WebAuthnSignupFinishResult {
    user_id: String,
}

#[allow(dead_code)]
struct WebAuthnSigninBeginParams {
    username: String,
    scenario: String,
}

#[allow(dead_code)]
struct WebAuthnSigninBeginResult {
    session_id: String,
    challenge_json: String,
}

#[allow(dead_code)]
struct WebAuthnSigninFinishParams {
    session_id: String,
    credential_json: String,
}

#[allow(dead_code)]
struct WebAuthnSigninFinishResult {
    user_id: String,
}

#[allow(dead_code)]
struct WebAuthnPasskeyLoginBeginParams {
    rp_display_name: String,
    rp_id: String,
    rp_origins: String,
}

#[allow(dead_code)]
struct WebAuthnPasskeyLoginBeginResult {
    session_id: String,
    challenge_json: String,
}

#[allow(dead_code)]
struct WebAuthnPasskeyLoginFinishParams {
    session_id: String,
    credential_json: String,
}

#[allow(dead_code)]
struct WebAuthnPasskeyLoginFinishResult {
    user_id: String,
    username: String,
}

#[allow(dead_code)]
struct WebAuthnMfaLoginBeginParams {
    rp_display_name: String,
    rp_id: String,
    rp_origins: String,
    username: String,
}

#[allow(dead_code)]
struct WebAuthnMfaLoginBeginResult {
    session_id: String,
    challenge_json: String,
}

#[allow(dead_code)]
struct WebAuthnMfaLoginFinishParams {
    session_id: String,
    credential_json: String,
}

#[allow(dead_code)]
struct WebAuthnMfaLoginFinishResult {
    success: bool,
    user_id: String,
}

/// Custom errors for WebAuthn operations
#[derive(Debug)]
pub enum AuthError {
    WebAuthnFailed(String),
    ClientError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::WebAuthnFailed(msg) => write!(f, "WebAuthn error: {}", msg),
            AuthError::ClientError(msg) => write!(f, "Client error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

impl From<anyhow::Error> for AuthError {
    fn from(err: anyhow::Error) -> Self {
        AuthError::ClientError(err.to_string())
    }
}

/// WebAuthn application state using go-webauthn-client
///
/// Uses CLI-based JSON-RPC communication with go-webauthn process.
/// Session management is handled by the CLI process.
#[derive(Clone)]
#[allow(dead_code)]
pub struct WebAuthnState {
    /// Relying party ID (domain name)
    pub rp_id: String,
    /// Relying party origins (comma-separated)
    pub rp_origins: String,
    /// Relying party display name
    pub rp_name: String,
    /// Default scenario for operations ("passwordless", "mfa", or "usernameless")
    pub scenario: String,
    /// Go WebAuthn CLI client (wrapped in Arc<Mutex> for shared access)
    client: Arc<Mutex<GoWebAuthnClient>>,
}

impl WebAuthnState {
    /// Create a new WebAuthn state
    ///
    /// # Arguments
    /// * `rp_id` - Relying party ID (effective domain name, e.g., "localhost" or "example.com")
    /// * `rp_origin` - Relying party origin URL (e.g., "https://example.com:8443")
    /// * `rp_name` - Optional relying party display name
    pub fn new(rp_id: &str, rp_origin: &str, rp_name: Option<&str>) -> Result<Self, AuthError> {
        let client = GoWebAuthnClient::new(None)?;

        Ok(WebAuthnState {
            rp_id: rp_id.to_string(),
            rp_origins: rp_origin.to_string(),
            rp_name: rp_name.unwrap_or("Dure").to_string(),
            scenario: "passwordless".to_string(),
            client: Arc::new(Mutex::new(client)),
        })
    }

    /// Start passkey registration for a user
    ///
    /// Returns (session_id, challenge_json) to send to the client
    pub async fn start_registration(
        &self,
        username: String,
    ) -> Result<(String, String), AuthError> {
        let params = WebAuthnSignupBeginParams {
            rp_display_name: self.rp_name.clone(),
            rp_id: self.rp_id.clone(),
            rp_origins: self.rp_origins.clone(),
            username: username.clone(),
            display_name: username,
            scenario: self.scenario.clone(),
        };

        let mut client = self.client.lock().unwrap();
        let result = client.webauthn_signup_begin(params)?;

        Ok((result.session_id, result.challenge_json))
    }

    /// Finish passkey registration
    ///
    /// Verifies the registration credential JSON from client
    /// Returns user_id
    pub async fn finish_registration(
        &self,
        session_id: String,
        credential_json: String,
    ) -> Result<String, AuthError> {
        let params = WebAuthnSignupFinishParams {
            session_id,
            credential_json,
        };

        let mut client = self.client.lock().unwrap();
        let result = client.webauthn_signup_finish(params)?;

        Ok(result.user_id)
    }

    /// Start passkey authentication for a user
    ///
    /// Returns (session_id, challenge_json) to send to the client
    pub async fn start_authentication(
        &self,
        username: String,
    ) -> Result<(String, String), AuthError> {
        let params = WebAuthnSigninBeginParams {
            username,
            scenario: self.scenario.clone(),
        };

        let mut client = self.client.lock().unwrap();
        let result = client.webauthn_signin_begin(params)?;

        Ok((result.session_id, result.challenge_json))
    }

    /// Finish passkey authentication
    ///
    /// Verifies the authentication credential JSON from client
    /// Returns user_id
    pub async fn finish_authentication(
        &self,
        session_id: String,
        credential_json: String,
    ) -> Result<String, AuthError> {
        let params = WebAuthnSigninFinishParams {
            session_id,
            credential_json,
        };

        let mut client = self.client.lock().unwrap();
        let result = client.webauthn_signin_finish(params)?;

        Ok(result.user_id)
    }

    /// Start passkey login (discoverable credentials, usernameless)
    ///
    /// Returns (session_id, challenge_json) to send to the client
    pub async fn start_passkey_login(&self) -> Result<(String, String), AuthError> {
        let params = WebAuthnPasskeyLoginBeginParams {
            rp_display_name: self.rp_name.clone(),
            rp_id: self.rp_id.clone(),
            rp_origins: self.rp_origins.clone(),
        };

        let mut client = self.client.lock().unwrap();
        let result = client.webauthn_passkey_login_begin(params)?;

        Ok((result.session_id, result.challenge_json))
    }

    /// Finish passkey login
    ///
    /// Verifies the passkey login credential JSON from client
    /// Returns (user_id, username)
    pub async fn finish_passkey_login(
        &self,
        session_id: String,
        credential_json: String,
    ) -> Result<(String, String), AuthError> {
        let params = WebAuthnPasskeyLoginFinishParams {
            session_id,
            credential_json,
        };

        let mut client = self.client.lock().unwrap();
        let result = client.webauthn_passkey_login_finish(params)?;

        Ok((result.user_id, result.username))
    }

    /// Start multi-factor authentication
    ///
    /// Returns (session_id, challenge_json) to send to the client
    pub async fn start_mfa_login(&self, username: String) -> Result<(String, String), AuthError> {
        let params = WebAuthnMfaLoginBeginParams {
            rp_display_name: self.rp_name.clone(),
            rp_id: self.rp_id.clone(),
            rp_origins: self.rp_origins.clone(),
            username,
        };

        let mut client = self.client.lock().unwrap();
        let result = client.webauthn_mfa_login_begin(params)?;

        Ok((result.session_id, result.challenge_json))
    }

    /// Finish multi-factor authentication
    ///
    /// Verifies the MFA credential JSON from client
    /// Returns user_id
    pub async fn finish_mfa_login(
        &self,
        session_id: String,
        credential_json: String,
    ) -> Result<String, AuthError> {
        let params = WebAuthnMfaLoginFinishParams {
            session_id,
            credential_json,
        };

        let mut client = self.client.lock().unwrap();
        let result = client.webauthn_mfa_login_finish(params)?;

        Ok(result.user_id)
    }
}
