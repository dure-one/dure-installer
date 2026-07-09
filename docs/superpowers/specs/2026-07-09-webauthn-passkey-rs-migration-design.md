---
title: WebAuthn Migration from go-webauthn-client to passkey-rs
date: 2026-07-09
status: Draft
---

# WebAuthn Migration: go-webauthn-client → passkey-rs Design Specification

## Overview

Migrate Dure's WebAuthn implementation from go-webauthn-client (external Go binary via JSON-RPC) to passkey-rs (pure Rust library by 1Password). This eliminates external process dependencies, provides native smol async integration, and stores guest passkey credentials in the SQLite service database alongside other guest data.

**Scope:** WebAuthn authentication flows only (registration, authentication, passkey login, MFA). Cryptographic utilities (Ed25519 sign/verify, ChaCha20 encrypt/decrypt) use existing Rust crates (ed25519-dalek, chacha20poly1305).

**Migration Strategy:** Clean slate - no migration of existing credentials (go-webauthn-client credentials are ephemeral, not persisted). Guests re-register passkeys after upgrade.

## Requirements

### Functional Requirements

1. **WebAuthn Flows:**
   - Registration (username + passkey creation)
   - Authentication (username + passkey verification)
   - Passkey login (usernameless/discoverable credentials)
   - MFA login (multi-factor authentication)

2. **Storage:**
   - Guest passkey credentials stored in SQLite service database (`webauthn_passkeys` table)
   - WebAuthn challenge sessions stored in SQLite (`webauthn_sessions` table, 5-min TTL)
   - No KeePass storage (KeePass reserved for shop owner's private keys)

3. **User Validation:**
   - Presence check: always true (request received)
   - Verification check: validate session token exists (integrates with Dure session management)
   - Registration requires verification=true (user must have existing session, e.g., from OAuth)
   - Basic passkey auth verification=false (this IS how they get a session)

4. **Session Management:**
   - Challenge sessions expire after 5 minutes (WebAuthn standard)
   - One-time use (deleted after successful retrieval)
   - Automatic cleanup of expired sessions

### Non-Functional Requirements

1. **Runtime:** smol async (NOT tokio) - passkey-rs is runtime-agnostic via async-trait
2. **Platform Support:** All Dure platforms (Linux, macOS, Windows, OpenBSD, Android, WASM)
3. **Security:**
   - Signature counter prevents replay/cloned passkeys
   - Session replay attack prevention (one-time use)
   - Generic error messages for internal failures (no implementation leakage)
4. **Performance:** Blocking I/O wrapped with smol::unblock (SQLite Diesel queries)

## Architecture

### Component Structure

```
┌─────────────────────────────────────────────┐
│         WebAuthnManager (Public API)        │
│  - start_registration / finish_registration │
│  - start_authentication / finish_auth...    │
│  - start_passkey_login / finish_passkey...  │
│  - start_mfa_login / finish_mfa_login       │
└─────────────┬───────────────────────────────┘
              │ uses
              ├──────────────────┬──────────────────┐
              ▼                  ▼                  ▼
┌─────────────────────┐ ┌──────────────┐ ┌─────────────────────┐
│ SqliteCredential    │ │ SessionStore │ │ DureUserValidation  │
│ Store               │ │              │ │ Method              │
│                     │ │              │ │                     │
│ (CredentialStore    │ │ (Diesel +    │ │ (UserValidation     │
│  trait)             │ │  SQLite)     │ │  Method trait)      │
└──────┬──────────────┘ └──────┬───────┘ └──────┬──────────────┘
       │                       │                │
       └───────────┬───────────┘                │
                   ▼                            ▼
            ┌─────────────────┐         ┌────────────┐
            │ SQLite Service  │         │ Session    │
            │ Database        │         │ Validator  │
            │                 │         │            │
            │ - webauthn_     │         └────────────┘
            │   passkeys      │
            │ - webauthn_     │
            │   sessions      │
            └─────────────────┘
```

### File Structure

```
mobile/src/wss/server/webauthn/
├── mod.rs                  # WebAuthnManager + AuthError
├── credential_store.rs     # SqliteCredentialStore
├── session_store.rs        # SessionStore + Diesel schema
└── user_validation.rs      # DureUserValidationMethod
```

### Component Responsibilities

**1. WebAuthnManager** - Orchestrates WebAuthn ceremonies
- Creates and owns passkey-rs `Client` and `Authenticator`
- Translates between WSS server API and passkey-rs types
- No direct storage access (delegates to stores)
- Maintains same public API as current `WebAuthnState` (8 methods)

**2. SqliteCredentialStore** - Long-term passkey storage
- Implements passkey-rs `CredentialStore` trait
- Diesel model backed by `webauthn_passkeys` table
- Stores: credential_id (PK), user_id, username, rp_id, public_key, counter, etc.
- Uses smol::unblock for blocking Diesel queries

**3. SessionStore** - Ephemeral WebAuthn challenge sessions
- Diesel model backed by `webauthn_sessions` table
- Stores: session_id (PK), challenge_json, username, created_at, expires_at
- TTL: 5 minutes, one-time use, auto-cleanup

**4. DureUserValidationMethod** - User presence/verification
- Implements passkey-rs `UserValidationMethod` trait
- Presence check: always returns true
- Verification check: calls session validator function
- Integrates with Dure's existing session management

## Data Models

### Database Schema

**webauthn_passkeys table:**
```sql
CREATE TABLE webauthn_passkeys (
    credential_id BLOB PRIMARY KEY,
    user_id BLOB,                   -- User handle (optional in WebAuthn)
    username TEXT,                  -- For UI display
    rp_id TEXT NOT NULL,            -- Relying party ID
    public_key_cose BLOB NOT NULL,  -- COSE-encoded public key
    counter INTEGER,                -- Signature counter
    created_at TIMESTAMP NOT NULL,
    last_used_at TIMESTAMP,
    user_display_name TEXT
);

CREATE INDEX idx_passkeys_rp_user ON webauthn_passkeys(rp_id, username);
CREATE INDEX idx_passkeys_user_id ON webauthn_passkeys(user_id);
```

**webauthn_sessions table:**
```sql
CREATE TABLE webauthn_sessions (
    session_id TEXT PRIMARY KEY,
    challenge_json TEXT NOT NULL,
    username TEXT,
    created_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL
);

CREATE INDEX idx_sessions_expires ON webauthn_sessions(expires_at);
```

### Diesel Schema Definitions

```rust
table! {
    webauthn_passkeys (credential_id) {
        credential_id -> Binary,
        user_id -> Nullable<Binary>,
        username -> Nullable<Text>,
        rp_id -> Text,
        public_key_cose -> Binary,
        counter -> Nullable<Integer>,
        created_at -> Timestamp,
        last_used_at -> Nullable<Timestamp>,
        user_display_name -> Nullable<Text>,
    }
}

table! {
    webauthn_sessions (session_id) {
        session_id -> Text,
        challenge_json -> Text,
        username -> Nullable<Text>,
        created_at -> Timestamp,
        expires_at -> Timestamp,
    }
}
```

## Component Interfaces

### 1. SqliteCredentialStore

```rust
pub struct SqliteCredentialStore {
    db_path: PathBuf,
}

impl SqliteCredentialStore {
    pub fn new(db_path: PathBuf) -> Self { ... }
}

#[async_trait::async_trait]
impl CredentialStore for SqliteCredentialStore {
    type PasskeyItem = Passkey;

    async fn find_credentials(
        &self,
        ids: Option<&[PublicKeyCredentialDescriptor]>,
        rp_id: &str,
        user_handle: Option<&[u8]>,
    ) -> Result<Vec<Passkey>, StatusCode> {
        smol::unblock(move || {
            // Query webauthn_passkeys with filters:
            // - rp_id (always)
            // - credential_id IN (ids) if provided
            // - user_id = user_handle if provided
            // Convert DB rows to Passkey structs
        }).await
    }

    async fn save_credential(
        &mut self,
        cred: Passkey,
        user: PublicKeyCredentialUserEntity,
        rp: PublicKeyCredentialRpEntity,
        options: Options,
    ) -> Result<(), StatusCode> {
        smol::unblock(move || {
            // INSERT into webauthn_passkeys
            // Extract fields from Passkey struct
        }).await
    }

    async fn update_credential(&mut self, cred: &Passkey) -> Result<(), StatusCode> {
        smol::unblock(move || {
            // UPDATE webauthn_passkeys
            // SET counter, last_used_at
            // WHERE credential_id = cred.credential_id
        }).await
    }

    async fn get_info(&self) -> StoreInfo {
        StoreInfo {
            discoverability: DiscoverabilitySupport::Full,
        }
    }
}
```

**Error Mapping:**
- `diesel::result::Error::NotFound` → `Ctap2Error::NoCredentials`
- Other database errors → `Ctap2Error::Other`
- Conversion errors → `Ctap2Error::InvalidCredential`

### 2. SessionStore

```rust
pub struct SessionStore {
    db_path: PathBuf,
}

impl SessionStore {
    pub fn new(db_path: PathBuf) -> Self { ... }

    pub async fn create_session(
        &self,
        challenge_json: String,
        username: Option<String>,
    ) -> Result<String, AuthError> {
        smol::unblock(move || {
            // Generate UUID v4 session_id
            // Set expires_at = now + 5 minutes
            // INSERT into webauthn_sessions
            // Return session_id
        }).await
    }

    pub async fn get_session(
        &self,
        session_id: &str,
    ) -> Result<(String, Option<String>), AuthError> {
        smol::unblock(move || {
            // SELECT WHERE session_id AND expires_at > now
            // If expired, delete and return SessionExpired
            // DELETE session (one-time use)
            // Return (challenge_json, username)
        }).await
    }

    pub async fn cleanup_expired(&self) -> Result<usize, AuthError> {
        smol::unblock(move || {
            // DELETE WHERE expires_at < now
            // Return count deleted
        }).await
    }
}
```

**Error Types:**
- `SessionNotFound` - session doesn't exist in DB
- `SessionExpired` - session exists but past expiry time
- `DatabaseError` - Diesel query failure

### 3. DureUserValidationMethod

```rust
pub struct DureUserValidationMethod {
    session_validator: Arc<dyn Fn(&str) -> bool + Send + Sync>,
}

impl DureUserValidationMethod {
    pub fn new(session_validator: Arc<dyn Fn(&str) -> bool + Send + Sync>) -> Self { ... }
}

#[async_trait::async_trait]
impl UserValidationMethod for DureUserValidationMethod {
    type PasskeyItem = Passkey;

    async fn check_user<'a>(
        &self,
        hint: UiHint<'a, Passkey>,
        presence: bool,
        verification: bool,
    ) -> Result<UserCheck, Ctap2Error> {
        // Presence: always true (request received)
        let presence_result = presence;
        
        // Verification: check session validator
        let verification_result = if verification {
            // Extract user identifier from hint (username or user_id)
            // and validate with session_validator
            // Implementation will pattern match on UiHint variants
            // to extract the appropriate user identifier
            let user_id = extract_user_from_hint(&hint);
            user_id.map(|id| (self.session_validator)(id)).unwrap_or(false)
        } else {
            false
        };

        Ok(UserCheck {
            presence: presence_result,
            verification: verification_result,
        })
    }

    fn is_verification_enabled(&self) -> Option<bool> {
        Some(true)
    }

    fn is_presence_enabled(&self) -> bool {
        true
    }
}
```

**Session Validator:** Function provided at construction that checks if a valid session token exists for the user (integrates with Dure's existing session management).

### 4. WebAuthnManager

```rust
pub struct WebAuthnManager {
    rp_id: String,
    rp_origin: String,
    rp_name: String,
    client: Client,
    authenticator: Authenticator<SqliteCredentialStore>,
    session_store: SessionStore,
}

impl WebAuthnManager {
    pub fn new(
        rp_id: String,
        rp_origin: String,
        rp_name: Option<String>,
        db_path: PathBuf,
        session_validator: Arc<dyn Fn(&str) -> bool + Send + Sync>,
    ) -> Result<Self, AuthError> {
        let credential_store = SqliteCredentialStore::new(db_path.clone());
        let session_store = SessionStore::new(db_path);
        let user_validation = DureUserValidationMethod::new(session_validator);
        
        let authenticator = Authenticator::new(
            Aaguid::new_empty(),
            credential_store,
            user_validation,
        );
        
        let client = Client::new(authenticator);
        
        Ok(Self {
            rp_id,
            rp_origin,
            rp_name: rp_name.unwrap_or_else(|| "Dure".to_string()),
            client,
            authenticator,
            session_store,
        })
    }

    // Same 8 public methods as current WebAuthnState
    pub async fn start_registration(&self, username: String) -> Result<(String, String), AuthError>;
    pub async fn finish_registration(&self, session_id: String, credential_json: String) -> Result<String, AuthError>;
    pub async fn start_authentication(&self, username: String) -> Result<(String, String), AuthError>;
    pub async fn finish_authentication(&self, session_id: String, credential_json: String) -> Result<String, AuthError>;
    pub async fn start_passkey_login(&self) -> Result<(String, String), AuthError>;
    pub async fn finish_passkey_login(&self, session_id: String, credential_json: String) -> Result<(String, String), AuthError>;
    pub async fn start_mfa_login(&self, username: String) -> Result<(String, String), AuthError>;
    pub async fn finish_mfa_login(&self, session_id: String, credential_json: String) -> Result<String, AuthError>;
}
```

## Data Flows

### Registration Flow

```
WSS Client → WebAuthnManager → SessionStore → SqliteCredentialStore

1. POST /register/begin {username}
2. WebAuthnManager creates challenge (passkey-rs Client)
3. SessionStore.create_session(challenge_json) → session_id
4. Return {session_id, challenge_json} to client
5. [Browser WebAuthn API] navigator.credentials.create(challenge)
6. POST /register/finish {session_id, credential}
7. SessionStore.get_session(session_id) → challenge_json (delete session)
8. WebAuthnManager verifies credential (passkey-rs Authenticator)
9. SqliteCredentialStore.save_credential(passkey) → INSERT into DB
10. Return {user_id} to client
```

### Authentication Flow

```
1. POST /auth/begin {username}
2. SqliteCredentialStore.find_credentials(rp_id, username) → Vec<Passkey>
3. WebAuthnManager creates challenge with allowCredentials
4. SessionStore.create_session(challenge_json) → session_id
5. Return {session_id, challenge_json} to client
6. [Browser WebAuthn API] navigator.credentials.get(challenge)
7. POST /auth/finish {session_id, credential}
8. SessionStore.get_session(session_id) → challenge_json (delete session)
9. WebAuthnManager verifies signature (passkey-rs Authenticator)
10. SqliteCredentialStore.update_credential(passkey) → UPDATE counter, last_used_at
11. Return {user_id} to client
```

### Passkey Login Flow (Usernameless)

```
1. POST /passkey/begin (no username)
2. WebAuthnManager creates challenge (empty allowCredentials)
3. SessionStore.create_session(challenge_json) → session_id
4. Return {session_id, challenge_json} to client
5. [Browser shows user's passkeys, user selects one]
6. POST /passkey/finish {session_id, credential}
7. SessionStore.get_session(session_id) → challenge_json (delete session)
8. SqliteCredentialStore.find_credentials(credential_id) → Passkey
9. WebAuthnManager verifies signature
10. Extract username from passkey
11. Return {user_id, username} to client
```

## Error Handling

### Error Type Hierarchy

```rust
#[derive(Debug)]
pub enum AuthError {
    // WebAuthn protocol errors
    InvalidChallenge(String),
    InvalidCredential(String),
    CredentialNotFound,
    SessionExpired,
    SessionNotFound,
    
    // Storage errors
    DatabaseError(String),
    CredentialStorageError(String),
    
    // passkey-rs errors
    PasskeyError(String),
    
    // User validation errors
    UserNotVerified,
    UserNotPresent,
}
```

### Error Mapping

**From Diesel:**
- `diesel::result::Error::NotFound` → `AuthError::CredentialNotFound` or `AuthError::SessionNotFound`
- Other → `AuthError::DatabaseError`

**From passkey-rs:**
- `StatusCode::Ctap2(Ctap2Error::NoCredentials)` → `AuthError::CredentialNotFound`
- Other → `AuthError::PasskeyError`

**HTTP Status Codes:**
```rust
match error {
    AuthError::SessionNotFound => 404,
    AuthError::SessionExpired => 410,
    AuthError::CredentialNotFound => 404,
    AuthError::InvalidCredential(_) => 400,
    AuthError::InvalidChallenge(_) => 400,
    AuthError::UserNotVerified => 403,
    _ => 500, // Generic for internal errors
}
```

**Security:** Internal errors (database, storage) return generic 500 without details to prevent implementation leakage.

## Testing Strategy

### Unit Tests (TDD - write tests first)

**SqliteCredentialStore** (`credential_store.rs`):
- `test_save_credential_creates_new_row`
- `test_find_credentials_by_rp_id`
- `test_find_credentials_by_credential_id`
- `test_find_credentials_usernameless`
- `test_update_credential_increments_counter`
- `test_find_credentials_empty_returns_error`

**SessionStore** (`session_store.rs`):
- `test_create_session_generates_uuid`
- `test_get_session_returns_challenge`
- `test_get_session_deletes_after_retrieval`
- `test_get_session_expired_returns_error`
- `test_cleanup_expired_removes_old_sessions`

**DureUserValidationMethod** (`user_validation.rs`):
- `test_presence_always_true`
- `test_verification_checks_session_validator`
- `test_verification_fails_without_session`

**WebAuthnManager** (`mod.rs`):
- `test_start_registration_creates_session`
- `test_finish_registration_saves_credential`
- `test_start_authentication_finds_credentials`
- `test_finish_authentication_updates_counter`
- `test_passkey_login_no_username_required`

### Integration Tests

**Location:** `mobile/tests/test_webauthn_passkey_rs.rs` (replaces `test_webauthn_integration.rs`)

- `test_full_registration_flow` - end-to-end registration
- `test_multiple_passkeys_per_user` - user with 2+ devices
- `test_usernameless_login_flow` - discoverable credentials
- `test_session_replay_attack_prevented` - session one-time use
- `test_counter_prevents_cloned_passkey` - replay detection

### Testing Tools

- **Test runtime:** `smol-potat` - smol-compatible test macro
- **Database:** In-memory SQLite (`:memory:`) for unit tests, tempfile for integration tests
- **Fixtures:** Helper functions to create mock Passkey structs
- **Coverage:** 100% of public methods, key error paths

## Dependencies

### Add to mobile/Cargo.toml

```toml
[dependencies]
# WebAuthn - passkey-rs suite
passkey = "0.5"
passkey-authenticator = "0.5"
passkey-client = "0.6"
passkey-types = "0.5"

# Already present (used by passkey-rs):
# async-trait = "0.1"
# uuid = { version = "1.11", features = ["v4", "serde"] }
# serde = { version = "1.0", features = ["derive"] }
# serde_json = "1.0"

[dev-dependencies]
smol-potat = "1.1"  # smol test runtime
tempfile = "3.14"    # Temporary test databases
```

### Remove from mobile/Cargo.toml

```toml
# Remove:
[target.'cfg(not(any(target_os = "android", target_arch = "wasm32")))'.dependencies]
go-webauthn-client = { path = "../crates/go-webauthn-client" }  # DELETE
```

### Remove crates/go-webauthn-client

Delete entire `crates/go-webauthn-client` directory (no longer needed).

## Migration Steps

### Phase 1: Foundation (TDD)
1. Create Diesel migration for `webauthn_passkeys` and `webauthn_sessions` tables
2. Write unit tests for SqliteCredentialStore
3. Implement SqliteCredentialStore (make tests pass)
4. Write unit tests for SessionStore
5. Implement SessionStore (make tests pass)

### Phase 2: User Validation (TDD)
1. Write unit tests for DureUserValidationMethod
2. Implement DureUserValidationMethod (make tests pass)
3. Create session validator integration point in Dure session management

### Phase 3: WebAuthn Manager (TDD)
1. Write unit tests for WebAuthnManager (with mock stores)
2. Implement WebAuthnManager (make tests pass)
3. Write integration tests for full flows
4. Implement integration test helpers (mock browser responses)

### Phase 4: Integration
1. Update `mobile/src/wss/server/webauthn.rs` to re-export new components
2. Update WSS server handlers to use WebAuthnManager instead of WebAuthnState
3. Update `mobile/Cargo.toml` dependencies (add passkey-rs, remove go-webauthn-client)
4. Delete `crates/go-webauthn-client` directory
5. Delete `mobile/tests/test_webauthn_integration.rs`
6. Run full test suite (unit + integration)

### Phase 5: Verification
1. Test on all platforms (Linux, macOS, Windows, OpenBSD, Android, WASM)
2. Verify no go-webauthn binary dependencies remain
3. Test WebAuthn flows in deployed WSS server
4. Performance benchmarking (SQLite vs previous implementation)

## Security Considerations

1. **Signature Counter:** Prevents replay attacks and detects cloned authenticators
2. **Session One-Time Use:** Challenges consumed after verification (prevent replay)
3. **Session TTL:** 5-minute expiration aligns with WebAuthn standard
4. **Error Messages:** Generic 500 errors for internal failures (no leak)
5. **User Verification:** Requires existing session for registration (prevents unauthorized passkey creation)
6. **Discoverable Credentials:** user_handle stored allows usernameless login

## Performance Considerations

1. **Blocking I/O:** All Diesel queries wrapped in `smol::unblock`
2. **Database Indexes:** `idx_passkeys_rp_user`, `idx_passkeys_user_id`, `idx_sessions_expires`
3. **Session Cleanup:** Automatic on each `get_session` call (no separate task needed)
4. **Connection Pooling:** Consider adding if high concurrency required

## Open Questions

None - all design decisions approved.

## Success Criteria

1. ✅ All WebAuthn flows work (registration, authentication, passkey login, MFA)
2. ✅ No external process dependencies (go-webauthn binary removed)
3. ✅ Uses smol async runtime (not tokio)
4. ✅ Guest credentials stored in SQLite service DB (not KeePass)
5. ✅ Session management integrated with Dure's existing auth
6. ✅ All tests pass (unit + integration)
7. ✅ Works on all Dure platforms
8. ✅ TDD approach followed (tests written first)

## References

- [passkey-rs GitHub](https://github.com/1Password/passkey-rs)
- [WebAuthn Specification](https://w3c.github.io/webauthn/)
- [CTAP 2.0 Specification](https://fidoalliance.org/specs/fido-v2.0-ps-20190130/fido-client-to-authenticator-protocol-v2.0-ps-20190130.html)
- [Dure SSH Key Refactor Spec](./2026-07-09-ssh-key-generation-refactor-design.md) (similar migration pattern)
