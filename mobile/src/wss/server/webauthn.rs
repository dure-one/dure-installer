//! WebAuthn authentication support for WebSocket connections
//!
//! Provides passkey-based authentication using WebAuthn protocol via go-webauthn bridge.
//! This implementation uses pure Rust + Go (no OpenSSL dependencies).

use go_webauthn::*;

/// Custom errors for WebAuthn operations
#[derive(Debug)]
pub enum AuthError {
    WebAuthnFailed(String),
    InvalidJson(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::WebAuthnFailed(msg) => write!(f, "WebAuthn error: {}", msg),
            AuthError::InvalidJson(msg) => write!(f, "JSON parsing error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

/// WebAuthn application state (simplified wrapper for go-webauthn)
///
/// Note: Session management is handled by the Go side, so this struct
/// only needs to store configuration. All state is managed internally
/// by the go-webauthn implementation.
#[derive(Clone)]
pub struct WebAuthnState {
    /// Default scenario for operations ("passwordless", "mfa", or "usernameless")
    pub scenario: String,
}

impl WebAuthnState {
    /// Create a new WebAuthn state
    ///
    /// # Arguments
    /// * `_rp_id` - Relying party ID (currently unused, go-webauthn configures this internally)
    /// * `_rp_origin` - Relying party origin URL (currently unused, go-webauthn configures this internally)
    /// * `_rp_name` - Optional relying party display name (currently unused)
    ///
    /// Note: The go-webauthn bridge is configured at initialization time with RP details.
    /// This constructor exists for API compatibility but doesn't need the parameters.
    pub fn new(_rp_id: &str, _rp_origin: &str, _rp_name: Option<&str>) -> Result<Self, AuthError> {
        Ok(WebAuthnState {
            scenario: "passwordless".to_string(),
        })
    }

    /// Start passkey registration for a user
    ///
    /// Returns JSON-serialized creation challenge to send to the client
    pub async fn start_registration(
        &self,
        username: String,
    ) -> Result<(String, String), AuthError> {
        let req = SignupBeginRequest {
            username: username.clone(),
            display_name: username,
            scenario: self.scenario.clone(),
        };

        let resp = webauthn_signup_begin(&req).await;

        if !resp.success {
            return Err(AuthError::WebAuthnFailed(resp.error));
        }

        Ok((resp.session_id, resp.challenge_json))
    }

    /// Finish passkey registration
    ///
    /// Verifies the registration credential JSON from client
    pub async fn finish_registration(
        &self,
        session_id: String,
        credential_json: String,
    ) -> Result<String, AuthError> {
        let req = SignupFinishRequest {
            session_id,
            credential_json,
        };

        let resp = webauthn_signup_finish(&req).await;

        if !resp.success {
            return Err(AuthError::WebAuthnFailed(resp.error));
        }

        Ok(resp.user_id)
    }

    /// Start passkey authentication for a user
    ///
    /// Returns JSON-serialized request challenge to send to the client
    pub async fn start_authentication(
        &self,
        username: String,
    ) -> Result<(String, String), AuthError> {
        let req = SigninBeginRequest {
            username,
            scenario: self.scenario.clone(),
        };

        let resp = webauthn_signin_begin(&req).await;

        if !resp.success {
            return Err(AuthError::WebAuthnFailed(resp.error));
        }

        Ok((resp.session_id, resp.challenge_json))
    }

    /// Finish passkey authentication
    ///
    /// Verifies the authentication credential JSON from client
    pub async fn finish_authentication(
        &self,
        session_id: String,
        credential_json: String,
    ) -> Result<String, AuthError> {
        let req = SigninFinishRequest {
            session_id,
            credential_json,
        };

        let resp = webauthn_signin_finish(&req).await;

        if !resp.success {
            return Err(AuthError::WebAuthnFailed(resp.error));
        }

        Ok(resp.user_id)
    }

    /// Start passkey login (discoverable credentials)
    ///
    /// Returns JSON-serialized challenge for usernameless authentication
    pub async fn start_passkey_login(
        &self,
        mediation: String,
    ) -> Result<(String, String), AuthError> {
        let req = PasskeyLoginBeginRequest { mediation };

        let resp = webauthn_passkey_login_begin(&req).await;

        if !resp.success {
            return Err(AuthError::WebAuthnFailed(resp.error));
        }

        Ok((resp.session_id, resp.challenge_json))
    }

    /// Finish passkey login
    ///
    /// Verifies the passkey login credential JSON from client
    pub async fn finish_passkey_login(
        &self,
        session_id: String,
        credential_json: String,
    ) -> Result<(String, String), AuthError> {
        let req = PasskeyLoginFinishRequest {
            session_id,
            credential_json,
        };

        let resp = webauthn_passkey_login_finish(&req).await;

        if !resp.success {
            return Err(AuthError::WebAuthnFailed(resp.error));
        }

        Ok((resp.user_id, resp.username))
    }

    /// Start multi-factor authentication
    ///
    /// Returns JSON-serialized challenge for MFA
    pub async fn start_mfa_login(
        &self,
        username: String,
        mediation: String,
    ) -> Result<(String, String), AuthError> {
        let req = MfaLoginBeginRequest {
            username,
            mediation,
        };

        let resp = webauthn_mfa_login_begin(&req).await;

        if !resp.success {
            return Err(AuthError::WebAuthnFailed(resp.error));
        }

        Ok((resp.session_id, resp.challenge_json))
    }

    /// Finish multi-factor authentication
    ///
    /// Verifies the MFA credential JSON from client
    pub async fn finish_mfa_login(
        &self,
        session_id: String,
        credential_json: String,
    ) -> Result<String, AuthError> {
        let req = MfaLoginFinishRequest {
            session_id,
            credential_json,
        };

        let resp = webauthn_mfa_login_finish(&req).await;

        if !resp.success {
            return Err(AuthError::WebAuthnFailed(resp.error));
        }

        Ok(resp.user_id)
    }
}
