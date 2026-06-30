//! Rust client for go-webauthn CLI process
//!
//! Communicates with the Go WebAuthn/crypto server via JSON-RPC over stdin/stdout.
//! This approach works on all platforms including OpenBSD where c-archive is not supported.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

/// JSON-RPC 2.0 Request
#[derive(Debug, Serialize)]
struct Request<T> {
    id: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<T>,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Deserialize)]
struct Response<T> {
    id: String,
    result: Option<T>,
    error: Option<ErrorObj>,
}

/// JSON-RPC 2.0 Error Object
#[derive(Debug, Deserialize)]
struct ErrorObj {
    code: i32,
    message: String,
}

/// Ed25519 key generation result (internal, with base64 strings)
#[derive(Debug, Deserialize)]
struct Ed25519KeyPairRaw {
    public_key: String,   // base64-encoded
    private_key: String,  // base64-encoded
}

/// Ed25519 key generation result
#[derive(Debug)]
pub struct Ed25519KeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

/// Ed25519 sign parameters
#[derive(Debug, Serialize)]
struct Ed25519SignParams {
    private_key: Vec<u8>,
    message: Vec<u8>,
}

/// Ed25519 sign result
#[derive(Debug, Deserialize)]
struct Ed25519SignResult {
    signature: String,  // base64-encoded
}

/// Ed25519 verify parameters
#[derive(Debug, Serialize)]
struct Ed25519VerifyParams {
    public_key: Vec<u8>,
    message: Vec<u8>,
    signature: Vec<u8>,
}

/// Ed25519 verify result
#[derive(Debug, Deserialize)]
struct Ed25519VerifyResult {
    valid: bool,
}

/// ChaCha20-Poly1305 encrypt parameters
#[derive(Debug, Serialize)]
struct ChaCha20EncryptParams {
    key: Vec<u8>,
    nonce: Vec<u8>,
    plaintext: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    additional_data: Option<Vec<u8>>,
}

/// ChaCha20-Poly1305 encrypt result
#[derive(Debug, Deserialize)]
struct ChaCha20EncryptResult {
    ciphertext: String,  // base64-encoded
}

/// ChaCha20-Poly1305 decrypt parameters
#[derive(Debug, Serialize)]
struct ChaCha20DecryptParams {
    key: Vec<u8>,
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    additional_data: Option<Vec<u8>>,
}

/// ChaCha20-Poly1305 decrypt result
#[derive(Debug, Deserialize)]
struct ChaCha20DecryptResult {
    plaintext: String,  // base64-encoded
}

// ============================================================================
// WebAuthn Types
// ============================================================================

/// WebAuthn signup begin parameters
#[derive(Debug, Serialize)]
pub struct WebAuthnSignupBeginParams {
    pub rp_display_name: String,
    pub rp_id: String,
    pub rp_origins: String, // Comma-separated
    pub username: String,
    pub display_name: String,
    pub scenario: String, // "mfa", "passwordless", or "usernameless"
}

/// WebAuthn signup begin result
#[derive(Debug, Deserialize)]
pub struct WebAuthnSignupBeginResult {
    pub session_id: String,
    pub challenge_json: String,
}

/// WebAuthn signup finish parameters
#[derive(Debug, Serialize)]
pub struct WebAuthnSignupFinishParams {
    pub session_id: String,
    pub credential_json: String,
}

/// WebAuthn signup finish result
#[derive(Debug, Deserialize)]
pub struct WebAuthnSignupFinishResult {
    pub user_id: String,
    pub credential_id: String,
}

/// WebAuthn signin begin parameters
#[derive(Debug, Serialize)]
pub struct WebAuthnSigninBeginParams {
    pub username: String,
    pub scenario: String,
}

/// WebAuthn signin begin result
#[derive(Debug, Deserialize)]
pub struct WebAuthnSigninBeginResult {
    pub session_id: String,
    pub challenge_json: String,
}

/// WebAuthn signin finish parameters
#[derive(Debug, Serialize)]
pub struct WebAuthnSigninFinishParams {
    pub session_id: String,
    pub credential_json: String,
}

/// WebAuthn signin finish result
#[derive(Debug, Deserialize)]
pub struct WebAuthnSigninFinishResult {
    pub user_id: String,
    pub username: String,
}

// ============================================================================
// Passkey Login (Discoverable Credentials) Types
// ============================================================================

/// WebAuthn passkey login begin parameters
#[derive(Debug, Serialize)]
pub struct WebAuthnPasskeyLoginBeginParams {
    pub rp_display_name: String,
    pub rp_id: String,
    pub rp_origins: String,
}

/// WebAuthn passkey login begin result
#[derive(Debug, Deserialize)]
pub struct WebAuthnPasskeyLoginBeginResult {
    pub session_id: String,
    pub challenge_json: String,
}

/// WebAuthn passkey login finish parameters
#[derive(Debug, Serialize)]
pub struct WebAuthnPasskeyLoginFinishParams {
    pub session_id: String,
    pub credential_json: String,
}

/// WebAuthn passkey login finish result
#[derive(Debug, Deserialize)]
pub struct WebAuthnPasskeyLoginFinishResult {
    pub user_id: String,
    pub username: String,
}

// ============================================================================
// MFA Login Types
// ============================================================================

/// WebAuthn MFA login begin parameters
#[derive(Debug, Serialize)]
pub struct WebAuthnMfaLoginBeginParams {
    pub rp_display_name: String,
    pub rp_id: String,
    pub rp_origins: String,
    pub username: String,
}

/// WebAuthn MFA login begin result
#[derive(Debug, Deserialize)]
pub struct WebAuthnMfaLoginBeginResult {
    pub session_id: String,
    pub challenge_json: String,
}

/// WebAuthn MFA login finish parameters
#[derive(Debug, Serialize)]
pub struct WebAuthnMfaLoginFinishParams {
    pub session_id: String,
    pub credential_json: String,
}

/// WebAuthn MFA login finish result
#[derive(Debug, Deserialize)]
pub struct WebAuthnMfaLoginFinishResult {
    pub user_id: String,
    pub username: String,
}

// ============================================================================
// Credential Management Types
// ============================================================================

/// WebAuthn list credentials parameters
#[derive(Debug, Serialize)]
pub struct WebAuthnListCredentialsParams {
    pub username: String,
}

/// Credential info
#[derive(Debug, Deserialize)]
pub struct CredentialInfo {
    pub id: String,
    pub public_key: String,
    pub aaguid: String,
    pub sign_count: u32,
}

/// WebAuthn list credentials result
#[derive(Debug, Deserialize)]
pub struct WebAuthnListCredentialsResult {
    pub credentials: Vec<CredentialInfo>,
}

/// WebAuthn delete credential parameters
#[derive(Debug, Serialize)]
pub struct WebAuthnDeleteCredentialParams {
    pub username: String,
    pub credential_id: String,
}

/// WebAuthn delete credential result
#[derive(Debug, Deserialize)]
pub struct WebAuthnDeleteCredentialResult {
    pub success: bool,
}

/// Client for go-webauthn CLI process
pub struct GoWebAuthnClient {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
    request_id: AtomicU64,
}

impl GoWebAuthnClient {
    /// Create a new client by spawning the go-webauthn CLI process
    ///
    /// # Arguments
    /// * `cli_path` - Path to the go-webauthn-cli executable. If None, searches in PATH and common locations.
    pub fn new(cli_path: Option<&str>) -> Result<Self> {
        let exe_path = if let Some(path) = cli_path {
            path.to_string()
        } else {
            // Try to find the executable
            Self::find_cli_executable()?
        };

        let mut child = Command::new(&exe_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit()) // Let stderr messages go to parent's stderr
            .spawn()
            .with_context(|| format!("Failed to spawn go-webauthn-cli at: {}", exe_path))?;

        let stdin = BufWriter::new(
            child
                .stdin
                .take()
                .context("Failed to capture child stdin")?,
        );

        let stdout = BufReader::new(
            child
                .stdout
                .take()
                .context("Failed to capture child stdout")?,
        );

        Ok(Self {
            child,
            stdin,
            stdout,
            request_id: AtomicU64::new(1),
        })
    }

    /// Find the go-webauthn-cli executable
    fn find_cli_executable() -> Result<String> {
        use std::path::Path;

        // Try common locations (relative to different possible working directories)
        let candidates = [
            // Relative to workspace root (when running from dure/)
            "./crates/go-webauthn/bin/go-webauthn-cli",
            "crates/go-webauthn/bin/go-webauthn-cli",
            // Relative to mobile directory (when running from dure/mobile/)
            "../crates/go-webauthn/bin/go-webauthn-cli",
            // Relative to current directory
            "./bin/go-webauthn-cli",
            "./go-webauthn-cli",
            // One more level up (when running from nested directories)
            "../../crates/go-webauthn/bin/go-webauthn-cli",
        ];

        // First try relative paths by checking file existence
        for candidate in &candidates {
            let path = Path::new(candidate);
            if path.exists() {
                // Convert to absolute path for reliability
                if let Ok(abs_path) = path.canonicalize() {
                    return Ok(abs_path.to_string_lossy().to_string());
                }
            }
        }

        // Then try PATH
        if let Ok(path) = which::which("go-webauthn-cli") {
            return Ok(path.to_string_lossy().to_string());
        }

        anyhow::bail!(
            "go-webauthn-cli executable not found. Please build it first or specify the path.\n\
             Searched locations:\n\
             - ./crates/go-webauthn/bin/go-webauthn-cli\n\
             - crates/go-webauthn/bin/go-webauthn-cli\n\
             - ../crates/go-webauthn/bin/go-webauthn-cli\n\
             - ./bin/go-webauthn-cli\n\
             - ./go-webauthn-cli\n\
             - ../../crates/go-webauthn/bin/go-webauthn-cli\n\
             - go-webauthn-cli (in PATH)\n\n\
             To build: cd crates/go-webauthn && ./build-cli.sh"
        )
    }

    /// Send a request and receive a response
    fn call<P: Serialize, R: for<'de> Deserialize<'de>>(
        &mut self,
        method: &str,
        params: Option<P>,
    ) -> Result<R> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst).to_string();

        let request = Request {
            id: id.clone(),
            method: method.to_string(),
            params,
        };

        // Send request
        serde_json::to_writer(&mut self.stdin, &request)
            .context("Failed to serialize request")?;
        self.stdin
            .write_all(b"\n")
            .context("Failed to write newline")?;
        self.stdin.flush().context("Failed to flush stdin")?;

        // Read response
        let mut line = String::new();
        self.stdout
            .read_line(&mut line)
            .context("Failed to read response")?;

        let response: Response<R> =
            serde_json::from_str(&line).context("Failed to parse response")?;

        if response.id != id {
            anyhow::bail!("Response ID mismatch: expected {}, got {}", id, response.id);
        }

        if let Some(error) = response.error {
            anyhow::bail!("RPC error {}: {}", error.code, error.message);
        }

        response
            .result
            .context("Response missing result and error")
    }

    /// Generate an Ed25519 key pair
    pub fn ed25519_generate_key(&mut self) -> Result<Ed25519KeyPair> {
        let raw = self.call::<(), Ed25519KeyPairRaw>("ed25519.generateKey", None)?;

        use base64::Engine;
        let public_key = base64::engine::general_purpose::STANDARD
            .decode(&raw.public_key)
            .context("Failed to decode public key")?;
        let private_key = base64::engine::general_purpose::STANDARD
            .decode(&raw.private_key)
            .context("Failed to decode private key")?;

        Ok(Ed25519KeyPair {
            public_key,
            private_key,
        })
    }

    /// Sign a message with Ed25519
    pub fn ed25519_sign(&mut self, private_key: &[u8], message: &[u8]) -> Result<Vec<u8>> {
        use base64::Engine;

        let params = Ed25519SignParams {
            private_key: private_key.to_vec(),
            message: message.to_vec(),
        };

        let result = self.call::<_, Ed25519SignResult>("ed25519.sign", Some(params))?;
        base64::engine::general_purpose::STANDARD
            .decode(&result.signature)
            .context("Failed to decode signature")
    }

    /// Verify an Ed25519 signature
    pub fn ed25519_verify(
        &mut self,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<bool> {
        let params = Ed25519VerifyParams {
            public_key: public_key.to_vec(),
            message: message.to_vec(),
            signature: signature.to_vec(),
        };

        let result = self.call::<_, Ed25519VerifyResult>("ed25519.verify", Some(params))?;
        Ok(result.valid)
    }

    /// Encrypt data with ChaCha20-Poly1305
    pub fn chacha20_encrypt(
        &mut self,
        key: &[u8],
        nonce: &[u8],
        plaintext: &[u8],
        additional_data: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        use base64::Engine;

        let params = ChaCha20EncryptParams {
            key: key.to_vec(),
            nonce: nonce.to_vec(),
            plaintext: plaintext.to_vec(),
            additional_data: additional_data.map(|d| d.to_vec()),
        };

        let result = self.call::<_, ChaCha20EncryptResult>("chacha20.encrypt", Some(params))?;
        base64::engine::general_purpose::STANDARD
            .decode(&result.ciphertext)
            .context("Failed to decode ciphertext")
    }

    /// Decrypt data with ChaCha20-Poly1305
    pub fn chacha20_decrypt(
        &mut self,
        key: &[u8],
        nonce: &[u8],
        ciphertext: &[u8],
        additional_data: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        use base64::Engine;

        let params = ChaCha20DecryptParams {
            key: key.to_vec(),
            nonce: nonce.to_vec(),
            ciphertext: ciphertext.to_vec(),
            additional_data: additional_data.map(|d| d.to_vec()),
        };

        let result = self.call::<_, ChaCha20DecryptResult>("chacha20.decrypt", Some(params))?;
        base64::engine::general_purpose::STANDARD
            .decode(&result.plaintext)
            .context("Failed to decode plaintext")
    }

    // ========================================================================
    // WebAuthn Methods
    // ========================================================================

    /// Begin WebAuthn registration (signup)
    pub fn webauthn_signup_begin(
        &mut self,
        params: WebAuthnSignupBeginParams,
    ) -> Result<WebAuthnSignupBeginResult> {
        self.call("webauthn.signup.begin", Some(params))
    }

    /// Finish WebAuthn registration (signup)
    pub fn webauthn_signup_finish(
        &mut self,
        params: WebAuthnSignupFinishParams,
    ) -> Result<WebAuthnSignupFinishResult> {
        self.call("webauthn.signup.finish", Some(params))
    }

    /// Begin WebAuthn authentication (signin)
    pub fn webauthn_signin_begin(
        &mut self,
        params: WebAuthnSigninBeginParams,
    ) -> Result<WebAuthnSigninBeginResult> {
        self.call("webauthn.signin.begin", Some(params))
    }

    /// Finish WebAuthn authentication (signin)
    pub fn webauthn_signin_finish(
        &mut self,
        params: WebAuthnSigninFinishParams,
    ) -> Result<WebAuthnSigninFinishResult> {
        self.call("webauthn.signin.finish", Some(params))
    }

    // ========================================================================
    // Passkey Login (Discoverable Credentials) Methods
    // ========================================================================

    /// Begin WebAuthn passkey login (discoverable credentials, usernameless)
    pub fn webauthn_passkey_login_begin(
        &mut self,
        params: WebAuthnPasskeyLoginBeginParams,
    ) -> Result<WebAuthnPasskeyLoginBeginResult> {
        self.call("webauthn.passkey.begin", Some(params))
    }

    /// Finish WebAuthn passkey login
    pub fn webauthn_passkey_login_finish(
        &mut self,
        params: WebAuthnPasskeyLoginFinishParams,
    ) -> Result<WebAuthnPasskeyLoginFinishResult> {
        self.call("webauthn.passkey.finish", Some(params))
    }

    // ========================================================================
    // MFA Login Methods
    // ========================================================================

    /// Begin WebAuthn MFA login (second factor authentication)
    pub fn webauthn_mfa_login_begin(
        &mut self,
        params: WebAuthnMfaLoginBeginParams,
    ) -> Result<WebAuthnMfaLoginBeginResult> {
        self.call("webauthn.mfa.begin", Some(params))
    }

    /// Finish WebAuthn MFA login
    pub fn webauthn_mfa_login_finish(
        &mut self,
        params: WebAuthnMfaLoginFinishParams,
    ) -> Result<WebAuthnMfaLoginFinishResult> {
        self.call("webauthn.mfa.finish", Some(params))
    }

    // ========================================================================
    // Credential Management Methods
    // ========================================================================

    /// List credentials for a user
    pub fn webauthn_list_credentials(
        &mut self,
        params: WebAuthnListCredentialsParams,
    ) -> Result<WebAuthnListCredentialsResult> {
        self.call("webauthn.credentials.list", Some(params))
    }

    /// Delete a credential for a user
    pub fn webauthn_delete_credential(
        &mut self,
        params: WebAuthnDeleteCredentialParams,
    ) -> Result<WebAuthnDeleteCredentialResult> {
        self.call("webauthn.credentials.delete", Some(params))
    }
}

impl Drop for GoWebAuthnClient {
    fn drop(&mut self) {
        // Kill the child process when client is dropped
        let _ = self.child.kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to create client for tests
    fn create_test_client() -> GoWebAuthnClient {
        GoWebAuthnClient::new(None).expect("Failed to create client. Ensure go-webauthn-cli is built: ./crates/go-webauthn/build-cli.sh")
    }

    #[test]
    #[ignore] // Only run if go-webauthn-cli is built
    fn test_ed25519_generate_key() {
        let mut client = create_test_client();
        let keypair = client
            .ed25519_generate_key()
            .expect("Failed to generate key");

        assert_eq!(keypair.public_key.len(), 32);
        assert_eq!(keypair.private_key.len(), 64);

        // Keys should not be all zeros
        assert!(keypair.public_key.iter().any(|&b| b != 0));
        assert!(keypair.private_key.iter().any(|&b| b != 0));
    }

    #[test]
    #[ignore]
    fn test_ed25519_sign_verify() {
        let mut client = create_test_client();

        let keypair = client
            .ed25519_generate_key()
            .expect("Failed to generate key");
        let message = b"Hello, World!";

        let signature = client
            .ed25519_sign(&keypair.private_key, message)
            .expect("Failed to sign");

        assert_eq!(signature.len(), 64, "Ed25519 signature should be 64 bytes");

        let valid = client
            .ed25519_verify(&keypair.public_key, message, &signature)
            .expect("Failed to verify");

        assert!(valid, "Valid signature should verify");

        // Test with wrong message
        let wrong_message = b"Different message";
        let valid = client
            .ed25519_verify(&keypair.public_key, wrong_message, &signature)
            .expect("Failed to verify");

        assert!(!valid, "Signature should not verify with wrong message");
    }

    #[test]
    #[ignore]
    fn test_chacha20_encrypt_decrypt() {
        let mut client = create_test_client();

        let key = vec![42u8; 32];
        let nonce = vec![13u8; 24];
        let plaintext = b"Secret message for encryption test";

        let ciphertext = client
            .chacha20_encrypt(&key, &nonce, plaintext, None)
            .expect("Failed to encrypt");

        assert_ne!(
            ciphertext.as_slice(),
            plaintext,
            "Ciphertext should differ from plaintext"
        );
        assert!(
            ciphertext.len() > plaintext.len(),
            "Ciphertext should be longer (includes auth tag)"
        );

        let decrypted = client
            .chacha20_decrypt(&key, &nonce, &ciphertext, None)
            .expect("Failed to decrypt");

        assert_eq!(
            decrypted.as_slice(),
            plaintext,
            "Decrypted should match original plaintext"
        );
    }

    #[test]
    #[ignore]
    fn test_chacha20_with_aad() {
        let mut client = create_test_client();

        let key = vec![1u8; 32];
        let nonce = vec![2u8; 24];
        let plaintext = b"Message";
        let aad = b"Additional authenticated data";

        let ciphertext = client
            .chacha20_encrypt(&key, &nonce, plaintext, Some(aad))
            .expect("Failed to encrypt");

        // Decrypt with correct AAD
        let decrypted = client
            .chacha20_decrypt(&key, &nonce, &ciphertext, Some(aad))
            .expect("Failed to decrypt");

        assert_eq!(decrypted.as_slice(), plaintext);

        // Decrypt with wrong AAD should fail
        let wrong_aad = b"Wrong data";
        let result = client.chacha20_decrypt(&key, &nonce, &ciphertext, Some(wrong_aad));

        assert!(result.is_err(), "Should fail with wrong AAD");
    }

    #[test]
    #[ignore]
    fn test_webauthn_signup_begin() {
        let mut client = create_test_client();

        let params = WebAuthnSignupBeginParams {
            rp_display_name: "Test Corp".to_string(),
            rp_id: "test.example.com".to_string(),
            rp_origins: "https://test.example.com".to_string(),
            username: "testuser@example.com".to_string(),
            display_name: "Test User".to_string(),
            scenario: "passwordless".to_string(),
        };

        let result = client.webauthn_signup_begin(params);
        assert!(result.is_ok(), "Signup begin should succeed: {:?}", result);

        let signup = result.unwrap();
        assert!(!signup.session_id.is_empty(), "Should have session ID");
        assert!(!signup.challenge_json.is_empty(), "Should have challenge JSON");

        // Verify challenge is valid JSON
        let _parsed: serde_json::Value = serde_json::from_str(&signup.challenge_json)
            .expect("Challenge should be valid JSON");
    }

    #[test]
    #[ignore]
    fn test_webauthn_signin_begin_no_user() {
        let mut client = create_test_client();

        let params = WebAuthnSigninBeginParams {
            username: "nonexistent@example.com".to_string(),
            scenario: "passwordless".to_string(),
        };

        let result = client.webauthn_signin_begin(params);

        // Should fail - WebAuthn not initialized (fresh process)
        assert!(result.is_err());
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("WebAuthn not initialized") || error.contains("user not found"),
            "Should indicate WebAuthn not initialized or user not found, got: {}",
            error
        );
    }

    #[test]
    #[ignore]
    fn test_webauthn_passkey_login_begin() {
        let mut client = create_test_client();

        let params = WebAuthnPasskeyLoginBeginParams {
            rp_display_name: "Test Corp".to_string(),
            rp_id: "test.example.com".to_string(),
            rp_origins: "https://test.example.com".to_string(),
        };

        let result = client.webauthn_passkey_login_begin(params);
        assert!(result.is_ok(), "Passkey login begin should succeed: {:?}", result);

        let login = result.unwrap();
        assert!(!login.session_id.is_empty());
        assert!(!login.challenge_json.is_empty());
    }

    #[test]
    #[ignore]
    fn test_webauthn_mfa_login_begin_no_user() {
        let mut client = create_test_client();

        let params = WebAuthnMfaLoginBeginParams {
            rp_display_name: "Test Corp".to_string(),
            rp_id: "test.example.com".to_string(),
            rp_origins: "https://test.example.com".to_string(),
            username: "nonexistent@example.com".to_string(),
        };

        let result = client.webauthn_mfa_login_begin(params);

        // Should fail - user doesn't exist
        assert!(result.is_err());
    }

    #[test]
    #[ignore]
    fn test_webauthn_list_credentials_empty() {
        let mut client = create_test_client();

        // First initialize WebAuthn with a signup
        let signup_params = WebAuthnSignupBeginParams {
            rp_display_name: "Test".to_string(),
            rp_id: "test.com".to_string(),
            rp_origins: "https://test.com".to_string(),
            username: "newuser@example.com".to_string(),
            display_name: "New User".to_string(),
            scenario: "passwordless".to_string(),
        };

        // This initializes WebAuthn state
        let _ = client.webauthn_signup_begin(signup_params)
            .expect("Signup should succeed");

        // Now list credentials
        let params = WebAuthnListCredentialsParams {
            username: "newuser@example.com".to_string(),
        };

        let result = client.webauthn_list_credentials(params);
        assert!(result.is_ok(), "List credentials should succeed: {:?}", result);

        let list = result.unwrap();
        assert_eq!(
            list.credentials.len(),
            0,
            "New user should have no credentials"
        );
    }

    #[test]
    #[ignore]
    fn test_multiple_operations() {
        let mut client = create_test_client();

        // Generate key
        let keypair = client.ed25519_generate_key().expect("Generate key");

        // Sign message
        let message = b"test";
        let signature = client
            .ed25519_sign(&keypair.private_key, message)
            .expect("Sign");

        // Verify signature
        let valid = client
            .ed25519_verify(&keypair.public_key, message, &signature)
            .expect("Verify");
        assert!(valid);

        // Encrypt data
        let key = vec![1u8; 32];
        let nonce = vec![2u8; 24];
        let ciphertext = client
            .chacha20_encrypt(&key, &nonce, message, None)
            .expect("Encrypt");

        // Decrypt data
        let decrypted = client
            .chacha20_decrypt(&key, &nonce, &ciphertext, None)
            .expect("Decrypt");
        assert_eq!(decrypted.as_slice(), message);

        // WebAuthn signup
        let signup_params = WebAuthnSignupBeginParams {
            rp_display_name: "Test".to_string(),
            rp_id: "test.com".to_string(),
            rp_origins: "https://test.com".to_string(),
            username: "multi@example.com".to_string(),
            display_name: "Multi Test".to_string(),
            scenario: "passwordless".to_string(),
        };

        let signup = client.webauthn_signup_begin(signup_params).expect("Signup");
        assert!(!signup.session_id.is_empty());
    }

    #[test]
    #[ignore]
    fn test_request_id_increment() {
        let mut client = create_test_client();

        // Make multiple calls - request IDs should increment
        // This is internal behavior but ensures the client is working correctly
        let _k1 = client.ed25519_generate_key().expect("Generate 1");
        let _k2 = client.ed25519_generate_key().expect("Generate 2");
        let _k3 = client.ed25519_generate_key().expect("Generate 3");

        // All should succeed without ID conflicts
    }

    #[test]
    fn test_client_creation_failure() {
        // Test with invalid path
        let result = GoWebAuthnClient::new(Some("/nonexistent/path/to/cli"));

        assert!(result.is_err(), "Should fail with invalid CLI path");
    }
}
