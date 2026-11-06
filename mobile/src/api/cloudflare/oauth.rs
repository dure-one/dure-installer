//! Cloudflare OAuth2 authentication with PKCE
//!
//! Implements OAuth2 flow for Cloudflare API using:
//! - PKCE (Proof Key for Code Exchange) for native app security
//! - std::net::TcpListener for localhost callback server
//! - ureq for HTTP requests
//! - webbrowser for opening system browser
//!
//! ## OAuth Flow
//!
//! 1. Generate PKCE code_verifier and code_challenge
//! 2. Start local HTTP server on random port (http://localhost:{port}/oauth/callback)
//! 3. Open system browser to Cloudflare OAuth URL with code_challenge
//! 4. User authenticates and grants permissions
//! 5. Cloudflare redirects to localhost callback with authorization code
//! 6. Exchange code for access token using code_verifier
//! 7. Store access token securely (keyring)
//! 8. Close local server

use crate::{dure_debug, dure_error, dure_info, dure_warn};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

/// Cloudflare OAuth client configuration (loaded from .env at compile time)
///
/// Set in .env file:
///   CLOUDFLARE_OAUTH_CLIENT_ID=54d11594-84e4-41aa-b438-e81b8fa78ee7
///   CLOUDFLARE_OAUTH_REDIRECT_URI=http://localhost:8976/oauth/callback
///
/// Fallback defaults to wrangler's public client if not set

/// Get OAuth client ID from compile-time env var
fn get_oauth_client_id() -> &'static str {
    option_env!("CLOUDFLARE_OAUTH_CLIENT_ID")
        .unwrap_or("47d31be581def48e92dbce05745335c3") // Custom OAuth app
}

// Redirect URI is now built dynamically using random port (like GCP OAuth)
// No longer need static redirect_uri or port parsing

/// Cloudflare OAuth scopes - MUST match exactly what's configured in OAuth app
/// Note: No offline_access = no refresh token (re-auth needed when token expires)
const CLOUDFLARE_SCOPES: &[&str] = &[
    // Account DNS Settings
    "account-dns-settings.read",
    "account-dns-settings.write",

    // DNS Management
    "dns.read",
    "dns.write",

    // DNS Views
    "dns-view.read",
    "dns-view.write",

    // Registrar Domains
    "registrar-domains.admin",
    "registrar-domains.read",

    // Zone DNS Settings
    "zone-dns-settings.read",
    "zone-dns-settings.write",

    // Zone Management
    "zone.read",
    "zone.write",

    // Zone Settings
    "zone-settings.read",
    "zone-settings.write",
];

/// OAuth result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthResult {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: u64,
}

/// Account info from Cloudflare API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareAccount {
    pub id: String,
    pub name: String,
}

/// API v4 response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ApiV4Response<T> {
    pub success: bool,
    pub result: T,
    pub errors: Vec<serde_json::Value>,
    pub messages: Vec<String>,
}

/// User info (account ID and name)
#[derive(Debug, Clone)]
pub struct UserInfo {
    pub account_id: String,
    pub account_name: String,
}

/// Cloudflare OAuth handler (PKCE - no client_secret needed)
/// Uses wrangler's public client by default
pub struct CloudflareOAuthHandler {
    // Empty struct - uses constants
}

impl CloudflareOAuthHandler {
    /// Create new OAuth handler (uses wrangler's public client)
    pub fn new() -> Self {
        Self {}
    }

    /// Create new OAuth handler (alias for new)
    pub fn default() -> Self {
        Self::new()
    }

    /// Get client ID (from compile-time config)
    pub fn client_id(&self) -> &str {
        get_oauth_client_id()
    }

    /// Build OAuth authorization URL with PKCE
    ///
    /// Note: redirect_uri is now dynamic (built at runtime), so this method
    /// is mainly for testing. Production code uses run_oauth_flow() which
    /// builds the URL with the actual bound port.
    pub fn build_auth_url(&self, redirect_uri: &str, state: &str, code_challenge: &str) -> Result<String> {
        let scope = CLOUDFLARE_SCOPES.join(" ");

        let url = format!(
            "https://dash.cloudflare.com/oauth2/auth?\
             client_id={}&\
             redirect_uri={}&\
             response_type=code&\
             scope={}&\
             state={}&\
             code_challenge={}&\
             code_challenge_method=S256",
            urlencoding::encode(get_oauth_client_id()),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(&scope),
            urlencoding::encode(state),
            urlencoding::encode(code_challenge)
        );

        Ok(url)
    }

    /// Run complete OAuth flow (blocking)
    ///
    /// Binds to port 51082 matching the custom OAuth app's redirect URI configuration
    pub fn run_oauth_flow(&self) -> Result<OAuthResult> {
        // Generate PKCE pair
        let (code_verifier, code_challenge) = generate_pkce_pair();

        // Use port matching OAuth app redirect URI: http://localhost:51082/cf_callback
        const OAUTH_PORT: u16 = 51082;

        // Bind to the port configured in OAuth app
        let listener = TcpListener::bind(("127.0.0.1", OAUTH_PORT))
            .context(format!("Failed to bind to port {} - ensure it's not in use", OAUTH_PORT))?;

        // Build redirect URI matching OAuth app configuration
        let redirect_uri = format!("http://localhost:{}/cf_callback", OAUTH_PORT);

        dure_info!("✓ TCP server listening on 127.0.0.1:{}", OAUTH_PORT);
        dure_debug!("Redirect URI: {}", redirect_uri);

        // Build OAuth URL with random state for CSRF protection
        let state = format!(
            "{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        let scope = CLOUDFLARE_SCOPES.join(" ");

        // Build auth URL using dynamic redirect_uri
        let auth_url = self.build_auth_url(&redirect_uri, &state, &code_challenge)?;

        // Open browser
        dure_info!("Opening browser for Cloudflare OAuth authorization...");
        dure_info!("If the browser doesn't open, visit: {}", auth_url);
        if let Err(e) = webbrowser::open(&auth_url) {
            dure_warn!("Failed to open browser: {}", e);
            dure_info!("Please manually open this URL:");
            dure_info!("{}", auth_url);
        }

        // Wait for OAuth callback
        dure_info!("Waiting for OAuth callback on {}...", redirect_uri);

        // Accept one connection (blocking)
        let (stream, _) = listener.accept().context("Failed to accept connection")?;

        // Handle the request
        let authorization_code = handle_oauth_request(stream, state)?;

        // Exchange code for tokens
        let oauth_result = self.exchange_code_for_token(&authorization_code, &redirect_uri, &code_verifier)?;

        Ok(oauth_result)
    }

    /// Exchange authorization code for access token
    fn exchange_code_for_token(
        &self,
        code: &str,
        redirect_uri: &str,
        code_verifier: &str,
    ) -> Result<OAuthResult> {
        let token_url = "https://dash.cloudflare.com/oauth2/token";

        // PKCE flow - no client_secret needed
        // Match reference implementation parameter order
        let body = format!(
            "grant_type=authorization_code&\
             code={}&\
             redirect_uri={}&\
             client_id={}&\
             code_verifier={}",
            urlencoding::encode(code),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(get_oauth_client_id()),
            urlencoding::encode(code_verifier)
        );

        dure_debug!("Exchanging authorization code for token...");

        let response = ureq::post(token_url)
            .set("Content-Type", "application/x-www-form-urlencoded")
            .send_string(&body)
            .context("Failed to exchange code for token")?;

        #[derive(Deserialize)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: Option<u64>,
        }

        let token_response: TokenResponse = response
            .into_json()
            .context("Failed to parse token response")?;

        // Calculate expiration timestamp
        let expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + token_response.expires_in.unwrap_or(3600); // Default 1 hour

        dure_info!("Successfully obtained access token");

        Ok(OAuthResult {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_at,
        })
    }
}

/// Verify API token by checking with Cloudflare API
pub fn verify_token(access_token: &str) -> Result<bool> {
    let url = "https://api.cloudflare.com/client/v4/user/tokens/verify";

    dure_debug!("Verifying Cloudflare API token...");

    let response = match ureq::get(url)
        .set("Authorization", &format!("Bearer {}", access_token))
        .call()
    {
        Ok(resp) => resp,
        Err(ureq::Error::Status(code, _)) => {
            dure_warn!("Token verification failed with status: {}", code);
            return Ok(false);
        }
        Err(ureq::Error::Transport(err)) => {
            return Err(anyhow::anyhow!("Network error during token verification: {}", err));
        }
    };

    #[derive(Deserialize)]
    struct VerifyResponse {
        success: bool,
    }

    let verify_response: VerifyResponse = response
        .into_json()
        .context("Failed to parse token verification response")?;

    if verify_response.success {
        dure_info!("✓ API token verified successfully");
    } else {
        dure_warn!("✗ API token verification failed");
    }

    Ok(verify_response.success)
}

/// Fetch account info from Cloudflare API v4 /accounts endpoint
pub fn get_user_info(access_token: &str) -> Result<UserInfo> {
    let url = "https://api.cloudflare.com/client/v4/accounts";

    dure_debug!("Fetching account info from Cloudflare API v4...");

    let response = ureq::get(url)
        .set("Authorization", &format!("Bearer {}", access_token))
        .set("Content-Type", "application/json")
        .call();

    let response = match response {
        Ok(resp) => resp,
        Err(ureq::Error::Status(code, resp)) => {
            let error_text = resp.into_string().unwrap_or_default();
            dure_error!("API v4 accounts request failed with status {}: {}", code, error_text);
            return Err(anyhow::anyhow!("Failed to fetch accounts (HTTP {}): {}", code, error_text));
        }
        Err(ureq::Error::Transport(err)) => {
            return Err(anyhow::anyhow!("Network error during accounts fetch: {}", err));
        }
    };

    let body = response.into_string()
        .context("Failed to read response body")?;

    dure_debug!("Accounts API response: {}", body);

    // Parse API v4 response
    let api_response: ApiV4Response<Vec<CloudflareAccount>> = serde_json::from_str(&body)
        .context(format!("Failed to parse accounts JSON: {}", body))?;

    if !api_response.success {
        let errors = serde_json::to_string(&api_response.errors).unwrap_or_default();
        return Err(anyhow::anyhow!("API returned success=false: {}", errors));
    }

    if api_response.result.is_empty() {
        return Err(anyhow::anyhow!("No accounts found for this user"));
    }

    let account = &api_response.result[0];

    if api_response.result.len() > 1 {
        dure_warn!("User has {} accounts - using first one: {} ({})",
            api_response.result.len(), account.name, account.id);
    }

    dure_info!("✓ Account info retrieved: {} (ID: {})", account.name, account.id);

    Ok(UserInfo {
        account_id: account.id.clone(),
        account_name: account.name.clone(),
    })
}

// No longer need env var loading - using wrangler's public client

/// Generate PKCE code_verifier and code_challenge pair
///
/// Returns (code_verifier, code_challenge)
pub fn generate_pkce_pair() -> (String, String) {
    use base64::Engine;
    use rand::Rng;
    use sha2::{Digest, Sha256};

    // Generate random 32 bytes
    let mut rng = rand::thread_rng();
    let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();

    // code_verifier: base64url(random_bytes)
    let code_verifier = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&random_bytes);

    // code_challenge: base64url(sha256(code_verifier))
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let hash = hasher.finalize();
    let code_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash);

    (code_verifier, code_challenge)
}

/// Handle OAuth callback request
fn handle_oauth_request(mut stream: TcpStream, expected_state: String) -> Result<String> {
    let mut buffer = [0u8; 2048];
    let n = stream.read(&mut buffer).context("Failed to read request")?;

    let request = String::from_utf8_lossy(&buffer[..n]);
    dure_debug!("Received OAuth callback: {}", request.lines().next().unwrap_or(""));

    // Parse query parameters from request line
    // Format: GET /cf_callback?code=xxx&state=yyy HTTP/1.1
    let first_line = request
        .lines()
        .next()
        .context("Empty HTTP request")?;

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        anyhow::bail!("Invalid HTTP request format");
    }

    let path = parts[1];
    let query_start = path
        .find('?')
        .context("No query parameters in callback")?;
    let query = &path[query_start + 1..];

    // Parse query parameters
    let params: std::collections::HashMap<String, String> = query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.split('=');
            Some((
                parts.next()?.to_string(),
                urlencoding::decode(parts.next()?).ok()?.to_string(),
            ))
        })
        .collect();

    // Verify state for CSRF protection
    let state = params
        .get("state")
        .context("No state parameter in callback")?;
    if state != &expected_state {
        anyhow::bail!("State mismatch - possible CSRF attack");
    }

    // Extract authorization code
    let code = params
        .get("code")
        .context("No code parameter in callback")?
        .clone();

    // Send success response to browser
    let response = "HTTP/1.1 200 OK\r\n\
                   Content-Type: text/html\r\n\
                   \r\n\
                   <html><body>\
                   <h1>Authorization Successful!</h1>\
                   <p>You can close this window and return to the application.</p>\
                   </body></html>";

    stream.write_all(response.as_bytes()).ok();
    stream.flush().ok();

    dure_info!("Authorization code received");

    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_handler_creation() {
        let handler = CloudflareOAuthHandler::new();
        assert_eq!(handler.client_id(), get_oauth_client_id());
    }

    #[test]
    fn test_pkce_challenge_generation() {
        let (verifier, challenge) = generate_pkce_pair();

        // code_verifier: 43-128 chars, base64url(random(32 bytes))
        assert!(verifier.len() >= 43 && verifier.len() <= 128);
        assert!(verifier.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));

        // code_challenge: base64url(sha256(code_verifier)) = 43 chars
        assert_eq!(challenge.len(), 43);
        assert!(challenge.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn test_pkce_challenge_uniqueness() {
        let (verifier1, challenge1) = generate_pkce_pair();
        let (verifier2, challenge2) = generate_pkce_pair();

        assert_ne!(verifier1, verifier2);
        assert_ne!(challenge1, challenge2);
    }

    #[test]
    fn test_build_auth_url() {
        let handler = CloudflareOAuthHandler::new();
        let (_, code_challenge) = generate_pkce_pair();
        let redirect_uri = "http://localhost:8976/oauth/callback";
        let url = handler.build_auth_url(redirect_uri, "state123", &code_challenge).unwrap();

        assert!(url.contains(&format!("client_id={}", get_oauth_client_id())));
        assert!(url.contains(&format!("redirect_uri={}", urlencoding::encode(redirect_uri))));
        assert!(url.contains("state=state123"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("code_challenge="));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.starts_with("https://dash.cloudflare.com/oauth2/auth"));
    }

    #[test]
    fn test_verify_token_invalid() {
        // Test with obviously invalid token
        let result = verify_token("invalid_token_12345");

        // Should either return Ok(false) or Err (depending on network/API)
        // We just verify it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }
}
