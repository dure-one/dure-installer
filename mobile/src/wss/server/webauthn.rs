//! WebAuthn authentication support for WebSocket connections
//!
//! Provides passkey-based authentication using WebAuthn protocol.
//! Registration and authentication endpoints allow clients to register
//! security keys and authenticate using them.

use async_lock::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use webauthn_rs::prelude::*;

/// Custom errors for WebAuthn operations
#[derive(Debug)]
pub enum AuthError {
    WebauthnError(WebauthnError),
    InvalidRegistrationState,
    InvalidAuthenticationState,
    UserNotFound,
    UserHasNoCredentials,
    LockError,
}

impl From<WebauthnError> for AuthError {
    fn from(e: WebauthnError) -> Self {
        AuthError::WebauthnError(e)
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::WebauthnError(e) => write!(f, "WebAuthn error: {:?}", e),
            AuthError::InvalidRegistrationState => write!(f, "Invalid registration state"),
            AuthError::InvalidAuthenticationState => write!(f, "Invalid authentication state"),
            AuthError::UserNotFound => write!(f, "User not found"),
            AuthError::UserHasNoCredentials => write!(f, "User has no credentials"),
            AuthError::LockError => write!(f, "Lock acquisition failed"),
        }
    }
}

impl std::error::Error for AuthError {}

/// WebAuthn application state
#[derive(Clone)]
pub struct WebAuthnState {
    /// WebAuthn instance (immutable, can be shared)
    pub webauthn: Arc<Webauthn>,
    /// User data storage (requires mutation, needs mutex)
    pub users: Arc<Mutex<UserData>>,
    /// Session storage for registration/authentication state
    pub sessions: Arc<Mutex<SessionStore>>,
}

/// User data storage
pub struct UserData {
    /// Map username to user UUID
    pub name_to_id: HashMap<String, Uuid>,
    /// Map user UUID to their passkeys
    pub keys: HashMap<Uuid, Vec<Passkey>>,
}

/// Session storage for registration and authentication challenges
pub struct SessionStore {
    /// Registration states: session_id -> (username, user_id, registration_state)
    pub reg_states: HashMap<String, (String, Uuid, PasskeyRegistration)>,
    /// Authentication states: session_id -> (user_id, authentication_state)
    pub auth_states: HashMap<String, (Uuid, PasskeyAuthentication)>,
}

impl WebAuthnState {
    /// Create a new WebAuthn state for the given domain and origin
    ///
    /// # Arguments
    /// * `rp_id` - Relying party ID (effective domain name, e.g., "localhost" or "example.com")
    /// * `rp_origin` - Relying party origin URL (must include port, e.g., "https://example.com:8443")
    /// * `rp_name` - Optional relying party display name
    pub fn new(rp_id: &str, rp_origin: &str, rp_name: Option<&str>) -> Result<Self, WebauthnError> {
        let origin = Url::parse(rp_origin).map_err(|_| WebauthnError::Configuration)?;

        let mut builder = WebauthnBuilder::new(rp_id, &origin)?;

        if let Some(name) = rp_name {
            builder = builder.rp_name(name);
        }

        let webauthn = Arc::new(builder.build()?);

        let users = Arc::new(Mutex::new(UserData {
            name_to_id: HashMap::new(),
            keys: HashMap::new(),
        }));

        let sessions = Arc::new(Mutex::new(SessionStore {
            reg_states: HashMap::new(),
            auth_states: HashMap::new(),
        }));

        Ok(WebAuthnState {
            webauthn,
            users,
            sessions,
        })
    }

    /// Start passkey registration for a user
    ///
    /// Returns the creation challenge to send to the client
    pub async fn start_registration(
        &self,
        session_id: String,
        username: String,
    ) -> Result<CreationChallengeResponse, AuthError> {
        let user_unique_id = {
            let users_guard = self.users.lock().await;
            users_guard
                .name_to_id
                .get(&username)
                .copied()
                .unwrap_or_else(Uuid::new_v4)
        };

        // Get existing credentials to exclude
        let exclude_credentials = {
            let users_guard = self.users.lock().await;
            users_guard
                .keys
                .get(&user_unique_id)
                .map(|keys| keys.iter().map(|sk| sk.cred_id().clone()).collect())
        };

        let (ccr, reg_state) = self.webauthn.start_passkey_registration(
            user_unique_id,
            &username,
            &username,
            exclude_credentials,
        )?;

        // Store registration state in session
        let mut sessions_guard = self.sessions.lock().await;
        sessions_guard
            .reg_states
            .insert(session_id, (username, user_unique_id, reg_state));

        Ok(ccr)
    }

    /// Finish passkey registration
    ///
    /// Verifies the registration credential and stores the passkey
    pub async fn finish_registration(
        &self,
        session_id: String,
        reg: RegisterPublicKeyCredential,
    ) -> Result<(), AuthError> {
        // Retrieve registration state from session
        let (username, user_unique_id, reg_state) = {
            let mut sessions_guard = self.sessions.lock().await;
            sessions_guard
                .reg_states
                .remove(&session_id)
                .ok_or(AuthError::InvalidRegistrationState)?
        };

        // Verify and create passkey
        let sk = self
            .webauthn
            .finish_passkey_registration(&reg, &reg_state)?;

        // Store passkey
        let mut users_guard = self.users.lock().await;
        users_guard
            .keys
            .entry(user_unique_id)
            .and_modify(|keys| keys.push(sk.clone()))
            .or_insert_with(|| vec![sk]);

        users_guard.name_to_id.insert(username, user_unique_id);

        Ok(())
    }

    /// Start passkey authentication for a user
    ///
    /// Returns the request challenge to send to the client
    pub async fn start_authentication(
        &self,
        session_id: String,
        username: String,
    ) -> Result<RequestChallengeResponse, AuthError> {
        let users_guard = self.users.lock().await;

        // Look up user ID from username
        let user_unique_id = users_guard
            .name_to_id
            .get(&username)
            .copied()
            .ok_or(AuthError::UserNotFound)?;

        // Get user's credentials
        let allow_credentials = users_guard
            .keys
            .get(&user_unique_id)
            .ok_or(AuthError::UserHasNoCredentials)?;

        let (rcr, auth_state) = self
            .webauthn
            .start_passkey_authentication(allow_credentials)?;

        // Release lock before acquiring sessions lock
        drop(users_guard);

        // Store authentication state in session
        let mut sessions_guard = self.sessions.lock().await;
        sessions_guard
            .auth_states
            .insert(session_id, (user_unique_id, auth_state));

        Ok(rcr)
    }

    /// Finish passkey authentication
    ///
    /// Verifies the authentication credential
    pub async fn finish_authentication(
        &self,
        session_id: String,
        auth: PublicKeyCredential,
    ) -> Result<Uuid, AuthError> {
        // Retrieve authentication state from session
        let (user_unique_id, auth_state) = {
            let mut sessions_guard = self.sessions.lock().await;
            sessions_guard
                .auth_states
                .remove(&session_id)
                .ok_or(AuthError::InvalidAuthenticationState)?
        };

        // Verify authentication
        let auth_result = self
            .webauthn
            .finish_passkey_authentication(&auth, &auth_state)?;

        // Update credential counter
        let mut users_guard = self.users.lock().await;
        users_guard
            .keys
            .get_mut(&user_unique_id)
            .map(|keys| {
                keys.iter_mut().for_each(|sk| {
                    sk.update_credential(&auth_result);
                })
            })
            .ok_or(AuthError::UserHasNoCredentials)?;

        Ok(user_unique_id)
    }
}
