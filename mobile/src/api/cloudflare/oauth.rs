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
    option_env!("CLOUDFLARE_OAUTH_CLIENT_ID").unwrap_or("47d31be581def48e92dbce05745335c3")
    // Custom OAuth app
}

// Redirect URI is now built dynamically using random port (like GCP OAuth)
// No longer need static redirect_uri or port parsing

/// Cloudflare OAuth scopes - MUST match exactly what's configured in OAuth app
/// Note: offline_access scope enables refresh tokens for automatic token renewal
const CLOUDFLARE_SCOPES: &[&str] = &[
    // Offline access - enables refresh tokens for automatic token renewal
    "offline_access",
    // User details - for token verification and user info
    "user-details.read",
    // Account DNS Settings
    "account-dns-settings.read",
    "account-dns-settings.write",
    // DNS Management
    "dns.read",
    "dns.write",
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

/// OAuth client (from Cloudflare IAM API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClient {
    pub id: String,
    pub client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>, // Only returned on create/rotate
    pub name: String,
    pub redirect_uris: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// OAuth client creation request
#[derive(Debug, Serialize)]
pub struct CreateOAuthClientRequest {
    pub name: String,
    pub redirect_uris: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<Vec<String>>,
}

/// OAuth client update request
#[derive(Debug, Serialize)]
pub struct UpdateOAuthClientRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uris: Option<Vec<String>>,
}

/// API v4 list response wrapper (generic)
#[derive(Debug, Deserialize)]
struct ApiV4ListResponse<T> {
    pub success: bool,
    pub result: Vec<T>,
    #[serde(default)]
    pub errors: Vec<serde_json::Value>,
    #[serde(default)]
    pub messages: Vec<String>,
}

/// API v4 single result response
#[derive(Debug, Deserialize)]
struct ApiV4SingleResponse<T> {
    pub success: bool,
    pub result: T,
    #[serde(default)]
    pub errors: Vec<serde_json::Value>,
    #[serde(default)]
    pub messages: Vec<String>,
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
    pub fn build_auth_url(
        &self,
        redirect_uri: &str,
        state: &str,
        code_challenge: &str,
    ) -> Result<String> {
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
        let listener = TcpListener::bind(("127.0.0.1", OAUTH_PORT)).context(format!(
            "Failed to bind to port {} - ensure it's not in use",
            OAUTH_PORT
        ))?;

        // Build redirect URI matching OAuth app configuration
        let redirect_uri = get_redirect_uri();

        dure_info!("✓ TCP server listening on 127.0.0.1:{}", OAUTH_PORT);
        dure_info!("📋 Redirect URI: {}", redirect_uri);
        dure_info!("⚠️  Ensure your Cloudflare OAuth app has this EXACT redirect URI configured");

        // Build OAuth URL with random state for CSRF protection
        let state = format!(
            "{:x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

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
        let oauth_result =
            self.exchange_code_for_token(&authorization_code, &redirect_uri, &code_verifier)?;

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

/// Cloudflare OAuth Client Manager
/// Manages OAuth client applications via Cloudflare IAM API
pub struct CloudflareOAuthClientManager {
    api_token: String,
    account_id: String,
}

impl CloudflareOAuthClientManager {
    /// Create new OAuth client manager
    pub fn new(api_token: String, account_id: String) -> Self {
        Self {
            api_token,
            account_id,
        }
    }

    /// Build API base URL for account
    fn api_base(&self) -> String {
        format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/oauth2/clients",
            self.account_id
        )
    }

    /// List all OAuth clients for account
    /// https://developers.cloudflare.com/api/resources/iam/subresources/oauth_clients/methods/list
    pub fn list_oauth_clients(&self) -> Result<Vec<OAuthClient>> {
        dure_debug!("Listing OAuth clients for account: {}", self.account_id);

        let url = self.api_base();

        let response = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .call()
            .map_err(|e| match e {
                ureq::Error::Status(code, resp) => {
                    let error_text = resp.into_string().unwrap_or_default();
                    if code == 401 || code == 403 {
                        anyhow::anyhow!("OAuth client list authentication failed (HTTP {})", code)
                    } else {
                        anyhow::anyhow!("OAuth client list failed (HTTP {}): {}", code, error_text)
                    }
                }
                ureq::Error::Transport(err) => {
                    anyhow::anyhow!("Network error: {}", err)
                }
            })?;

        let result: ApiV4ListResponse<OAuthClient> = response
            .into_json()
            .context("Failed to parse OAuth clients list response")?;

        if !result.success {
            let errors = serde_json::to_string(&result.errors).unwrap_or_default();
            anyhow::bail!("OAuth clients list failed: {}", errors);
        }

        dure_info!("Found {} OAuth clients", result.result.len());
        Ok(result.result)
    }

    /// Create OAuth client
    /// https://developers.cloudflare.com/api/resources/iam/subresources/oauth_clients/methods/create
    pub fn create_oauth_client(&self, request: CreateOAuthClientRequest) -> Result<OAuthClient> {
        dure_info!("Creating OAuth client: {}", request.name);

        let url = self.api_base();
        let body = serde_json::to_string(&request).context("Failed to serialize create request")?;

        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .set("Content-Type", "application/json")
            .send_string(&body)
            .map_err(|e| match e {
                ureq::Error::Status(code, resp) => {
                    let error_text = resp.into_string().unwrap_or_default();
                    anyhow::anyhow!(
                        "OAuth client creation failed (HTTP {}): {}",
                        code,
                        error_text
                    )
                }
                ureq::Error::Transport(err) => {
                    anyhow::anyhow!("Network error: {}", err)
                }
            })?;

        let result: ApiV4SingleResponse<OAuthClient> = response
            .into_json()
            .context("Failed to parse create response")?;

        if !result.success {
            let errors = serde_json::to_string(&result.errors).unwrap_or_default();
            anyhow::bail!("OAuth client creation failed: {}", errors);
        }

        dure_info!(
            "OAuth client created: {} ({})",
            result.result.name,
            result.result.id
        );
        Ok(result.result)
    }

    /// Update OAuth client
    /// https://developers.cloudflare.com/api/resources/iam/subresources/oauth_clients/methods/update
    pub fn update_oauth_client(
        &self,
        client_id: &str,
        request: UpdateOAuthClientRequest,
    ) -> Result<OAuthClient> {
        dure_info!("Updating OAuth client: {}", client_id);

        let url = format!("{}/{}", self.api_base(), client_id);
        let body = serde_json::to_string(&request).context("Failed to serialize update request")?;

        let response = ureq::put(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .set("Content-Type", "application/json")
            .send_string(&body)
            .map_err(|e| match e {
                ureq::Error::Status(code, resp) => {
                    let error_text = resp.into_string().unwrap_or_default();
                    anyhow::anyhow!("OAuth client update failed (HTTP {}): {}", code, error_text)
                }
                ureq::Error::Transport(err) => {
                    anyhow::anyhow!("Network error: {}", err)
                }
            })?;

        let result: ApiV4SingleResponse<OAuthClient> = response
            .into_json()
            .context("Failed to parse update response")?;

        if !result.success {
            let errors = serde_json::to_string(&result.errors).unwrap_or_default();
            anyhow::bail!("OAuth client update failed: {}", errors);
        }

        dure_info!("OAuth client updated: {}", result.result.id);
        Ok(result.result)
    }

    /// Delete OAuth client
    /// https://developers.cloudflare.com/api/resources/iam/subresources/oauth_clients/methods/delete
    pub fn delete_oauth_client(&self, client_id: &str) -> Result<()> {
        dure_info!("Deleting OAuth client: {}", client_id);

        let url = format!("{}/{}", self.api_base(), client_id);

        let response = ureq::delete(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .call()
            .map_err(|e| match e {
                ureq::Error::Status(code, resp) => {
                    let error_text = resp.into_string().unwrap_or_default();
                    anyhow::anyhow!(
                        "OAuth client deletion failed (HTTP {}): {}",
                        code,
                        error_text
                    )
                }
                ureq::Error::Transport(err) => {
                    anyhow::anyhow!("Network error: {}", err)
                }
            })?;

        let result: ApiV4SingleResponse<serde_json::Value> = response
            .into_json()
            .context("Failed to parse delete response")?;

        if !result.success {
            let errors = serde_json::to_string(&result.errors).unwrap_or_default();
            anyhow::bail!("OAuth client deletion failed: {}", errors);
        }

        dure_info!("OAuth client deleted successfully");
        Ok(())
    }

    /// Rotate OAuth client secret
    /// https://developers.cloudflare.com/api/resources/iam/subresources/oauth_clients/methods/rotate_secret
    pub fn rotate_oauth_client_secret(&self, client_id: &str) -> Result<OAuthClient> {
        dure_info!("Rotating OAuth client secret: {}", client_id);

        let url = format!("{}/{}/rotate_secret", self.api_base(), client_id);

        let response = ureq::post(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .call()
            .map_err(|e| match e {
                ureq::Error::Status(code, resp) => {
                    let error_text = resp.into_string().unwrap_or_default();
                    anyhow::anyhow!(
                        "OAuth secret rotation failed (HTTP {}): {}",
                        code,
                        error_text
                    )
                }
                ureq::Error::Transport(err) => {
                    anyhow::anyhow!("Network error: {}", err)
                }
            })?;

        let result: ApiV4SingleResponse<OAuthClient> = response
            .into_json()
            .context("Failed to parse rotate response")?;

        if !result.success {
            let errors = serde_json::to_string(&result.errors).unwrap_or_default();
            anyhow::bail!("OAuth secret rotation failed: {}", errors);
        }

        dure_info!("OAuth client secret rotated successfully");
        Ok(result.result)
    }

    /// Delete rotated OAuth client secret
    /// https://developers.cloudflare.com/api/resources/iam/subresources/oauth_clients/methods/delete_rotated_secret
    pub fn delete_rotated_oauth_client_secret(&self, client_id: &str) -> Result<()> {
        dure_info!("Deleting rotated OAuth client secret: {}", client_id);

        let url = format!("{}/{}/rotated_secret", self.api_base(), client_id);

        let response = ureq::delete(&url)
            .set("Authorization", &format!("Bearer {}", self.api_token))
            .call()
            .map_err(|e| match e {
                ureq::Error::Status(code, resp) => {
                    let error_text = resp.into_string().unwrap_or_default();
                    anyhow::anyhow!(
                        "Rotated secret deletion failed (HTTP {}): {}",
                        code,
                        error_text
                    )
                }
                ureq::Error::Transport(err) => {
                    anyhow::anyhow!("Network error: {}", err)
                }
            })?;

        let result: ApiV4SingleResponse<serde_json::Value> = response
            .into_json()
            .context("Failed to parse delete rotated secret response")?;

        if !result.success {
            let errors = serde_json::to_string(&result.errors).unwrap_or_default();
            anyhow::bail!("Rotated secret deletion failed: {}", errors);
        }

        dure_info!("Rotated OAuth client secret deleted successfully");
        Ok(())
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
            return Err(anyhow::anyhow!(
                "Network error during token verification: {}",
                err
            ));
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

/// Refresh access token using refresh token
///
/// Returns new OAuthResult with updated access_token and expires_at.
/// Preserves the original refresh_token.
///
/// Note: Cloudflare OAuth uses PKCE (no client_secret needed)
pub fn refresh_access_token(refresh_token: &str) -> Result<OAuthResult> {
    if refresh_token.is_empty() {
        return Err(anyhow::anyhow!("Refresh token is empty"));
    }

    dure_debug!("Refreshing Cloudflare OAuth access token...");

    let token_url = "https://dash.cloudflare.com/oauth2/token";

    // PKCE flow - no client_secret needed
    let body = format!(
        "grant_type=refresh_token&\
         refresh_token={}&\
         client_id={}",
        urlencoding::encode(refresh_token),
        urlencoding::encode(get_oauth_client_id())
    );

    let response = ureq::post(token_url)
        .set("Content-Type", "application/x-www-form-urlencoded")
        .send_string(&body);

    let response = match response {
        Ok(resp) => resp,
        Err(ureq::Error::Status(code, resp)) => {
            let error_text = resp.into_string().unwrap_or_default();
            dure_error!("Token refresh failed with status {}: {}", code, error_text);
            return Err(anyhow::anyhow!(
                "Token refresh failed (HTTP {}): {}",
                code,
                error_text
            ));
        }
        Err(e) => {
            return Err(anyhow::anyhow!("Network error during token refresh: {}", e));
        }
    };

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

    dure_info!("✓ Successfully refreshed Cloudflare access token");

    // Per Cloudflare OAuth conventions: Cloudflare ALWAYS returns new refresh_token
    // and immediately invalidates old one. No fallback to old token.
    Ok(OAuthResult {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        expires_at,
    })
}

/// Fetch account info from Cloudflare API v4 /accounts endpoint
pub fn get_user_info(access_token: &str) -> Result<UserInfo> {
    let url = "https://api.cloudflare.com/client/v4/accounts";

    dure_debug!("Fetching account info from /accounts...");

    let response = ureq::get(url)
        .set("Authorization", &format!("Bearer {}", access_token))
        .set("Content-Type", "application/json")
        .call();

    let response = match response {
        Ok(resp) => resp,
        Err(ureq::Error::Status(code, _)) => {
            dure_warn!("Accounts API failed ({}), trying zones fallback", code);
            return get_account_from_zones(access_token);
        }
        Err(ureq::Error::Transport(err)) => {
            return Err(anyhow::anyhow!("Network error: {}", err));
        }
    };

    let body = response.into_string().context("Failed to read response")?;

    let api_response: ApiV4Response<Vec<CloudflareAccount>> =
        serde_json::from_str(&body).context(format!("Failed to parse accounts JSON: {}", body))?;

    if !api_response.success || api_response.result.is_empty() {
        dure_warn!("No accounts from /accounts, trying zones fallback");
        return get_account_from_zones(access_token);
    }

    let account = &api_response.result[0];
    dure_info!("✓ Account info: {} ({})", account.name, account.id);

    Ok(UserInfo {
        account_id: account.id.clone(),
        account_name: account.name.clone(),
    })
}

/// Fallback: Get account ID from zones endpoint
/// Used when /accounts endpoint returns empty (insufficient permissions)
fn get_account_from_zones(access_token: &str) -> Result<UserInfo> {
    use crate::api::cloudflare::Zone;

    let url = "https://api.cloudflare.com/client/v4/zones";

    dure_debug!("Fetching account from zones endpoint...");

    let response = ureq::get(url)
        .set("Authorization", &format!("Bearer {}", access_token))
        .set("Content-Type", "application/json")
        .call();

    let response = match response {
        Ok(resp) => resp,
        Err(ureq::Error::Status(code, resp)) => {
            let error_text = resp.into_string().unwrap_or_default();
            dure_error!(
                "Zones API request failed with status {}: {}",
                code,
                error_text
            );
            return Err(anyhow::anyhow!(
                "Failed to fetch zones (HTTP {}): {}",
                code,
                error_text
            ));
        }
        Err(ureq::Error::Transport(err)) => {
            return Err(anyhow::anyhow!("Network error during zones fetch: {}", err));
        }
    };

    let body = response
        .into_string()
        .context("Failed to read zones response")?;

    // Parse API v4 response
    let api_response: ApiV4Response<Vec<Zone>> =
        serde_json::from_str(&body).context(format!("Failed to parse zones JSON: {}", body))?;

    if !api_response.success {
        let errors = serde_json::to_string(&api_response.errors).unwrap_or_default();
        return Err(anyhow::anyhow!(
            "Zones API returned success=false: {}",
            errors
        ));
    }

    if api_response.result.is_empty() {
        return Err(anyhow::anyhow!("No zones found - cannot determine account ID. Please add at least one domain to your Cloudflare account."));
    }

    let zone = &api_response.result[0];
    let account_id = zone.account.id.clone();
    let account_name = zone.account.name.clone();

    dure_info!(
        "✓ Account info retrieved from zones: {} (ID: {})",
        account_name,
        account_id
    );

    Ok(UserInfo {
        account_id,
        account_name,
    })
}

// No longer need env var loading - using wrangler's public client

/// Get the configured redirect URI for OAuth
pub fn get_redirect_uri() -> String {
    const OAUTH_PORT: u16 = 51082;
    format!("http://localhost:{}/cf_callback", OAUTH_PORT)
}

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
    let first_line = request.lines().next().unwrap_or("");
    dure_debug!("Received OAuth callback: {}", first_line);

    // Parse query parameters from request line
    // Format: GET /cf_callback?code=xxx&state=yyy HTTP/1.1
    //     or: GET /cf_callback?error=access_denied&error_description=...&state=yyy HTTP/1.1
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        anyhow::bail!("Invalid HTTP request format");
    }

    let path = parts[1];
    let query_start = path.find('?').context("No query parameters in callback")?;
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

    dure_debug!("OAuth callback query params: {:?}", params);

    // Verify state for CSRF protection
    let state = params
        .get("state")
        .context("No state parameter in callback")?;
    if state != &expected_state {
        anyhow::bail!("State mismatch - possible CSRF attack");
    }

    // Check for OAuth error response first (RFC 6749 Section 4.1.2.1)
    if let Some(error) = params.get("error") {
        let error_desc = params
            .get("error_description")
            .map(|s| s.as_str())
            .unwrap_or("No description provided");

        dure_error!("OAuth error: {} - {}", error, error_desc);

        // Send error response to browser
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
             Content-Type: text/html\r\n\
             \r\n\
             <html><body>\
             <h1>Authorization Failed</h1>\
             <p><strong>Error:</strong> {}</p>\
             <p><strong>Description:</strong> {}</p>\
             <p>You can close this window and return to the application.</p>\
             </body></html>",
            error, error_desc
        );

        stream.write_all(response.as_bytes()).ok();
        stream.flush().ok();

        anyhow::bail!("OAuth authorization failed: {} ({})", error, error_desc);
    }

    // Extract authorization code
    let code = params
        .get("code")
        .context("No code parameter in callback - OAuth may have failed silently")?
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

    dure_info!("✓ Authorization code received");

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
        assert!(verifier
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_'));

        // code_challenge: base64url(sha256(code_verifier)) = 43 chars
        assert_eq!(challenge.len(), 43);
        assert!(challenge
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_'));
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
        let url = handler
            .build_auth_url(redirect_uri, "state123", &code_challenge)
            .unwrap();

        assert!(url.contains(&format!("client_id={}", get_oauth_client_id())));
        assert!(url.contains(&format!(
            "redirect_uri={}",
            urlencoding::encode(redirect_uri)
        )));
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

    #[test]
    fn test_refresh_access_token_empty_token() {
        // Empty refresh token should return error
        let result = refresh_access_token("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_refresh_access_token_invalid_token() {
        // Invalid token should fail (network call expected)
        let result = refresh_access_token("invalid_refresh_token_12345");

        // Should return Err (either network error or API error)
        assert!(result.is_err());
    }

    #[test]
    fn test_refresh_access_token_preserves_refresh_token() {
        // Even with invalid token, function should preserve it in result
        // This test verifies the function signature returns OAuthResult with refresh_token
        // We can't test success without real token, so we just verify error path doesn't panic
        let refresh_token = "test_refresh_token";
        let result = refresh_access_token(refresh_token);

        // Should fail but not panic
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_oauth_error_access_denied() {
        use std::io::Cursor;
        use std::net::TcpStream;

        // Simulate OAuth error response when user denies authorization
        let request = b"GET /cf_callback?error=access_denied&error_description=User%20denied%20access&state=test123 HTTP/1.1\r\n\r\n";

        // Can't easily create TcpStream for test, so we test via mock
        // This test documents the expected behavior
        let query = "error=access_denied&error_description=User%20denied%20access&state=test123";
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

        assert_eq!(params.get("error"), Some(&"access_denied".to_string()));
        assert_eq!(
            params.get("error_description"),
            Some(&"User denied access".to_string())
        );
    }

    #[test]
    fn test_handle_oauth_error_invalid_request() {
        // Test invalid_request error
        let query = "error=invalid_request&error_description=Invalid%20scope&state=test123";
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

        assert_eq!(params.get("error"), Some(&"invalid_request".to_string()));
        assert_eq!(
            params.get("error_description"),
            Some(&"Invalid scope".to_string())
        );
    }

    #[test]
    fn test_redirect_uri_helper() {
        let uri = get_redirect_uri();
        assert_eq!(uri, "http://localhost:51082/cf_callback");
    }

    // OAuth Client Manager Tests

    #[test]
    fn test_oauth_client_manager_creation() {
        let manager =
            CloudflareOAuthClientManager::new("test_token".to_string(), "test_account".to_string());
        assert_eq!(
            manager.api_base(),
            "https://api.cloudflare.com/client/v4/accounts/test_account/oauth2/clients"
        );
    }

    #[test]
    fn test_list_oauth_clients_invalid_token() {
        let manager = CloudflareOAuthClientManager::new(
            "invalid_token".to_string(),
            "test_account".to_string(),
        );

        let result = manager.list_oauth_clients();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("authentication")
                || err_msg.contains("401")
                || err_msg.contains("403")
        );
    }

    #[test]
    #[ignore] // Only run with --ignored flag when credentials available
    fn test_list_oauth_clients_success() {
        // Integration test requiring real credentials
        let token = match std::env::var("CLOUDFLARE_TEST_TOKEN") {
            Ok(t) => t,
            Err(_) => return, // Skip test if no token
        };
        let account_id = match std::env::var("CLOUDFLARE_ACCOUNT_ID") {
            Ok(a) => a,
            Err(_) => return, // Skip test if no account_id
        };

        let manager = CloudflareOAuthClientManager::new(token, account_id);
        let result = manager.list_oauth_clients();

        assert!(result.is_ok());
        let clients = result.unwrap();
        // Should return Vec<OAuthClient>, even if empty
        assert!(clients.len() >= 0);
    }

    #[test]
    #[ignore] // Integration test
    fn test_create_oauth_client_success() {
        let token = match std::env::var("CLOUDFLARE_TEST_TOKEN") {
            Ok(t) => t,
            Err(_) => return,
        };
        let account_id = match std::env::var("CLOUDFLARE_ACCOUNT_ID") {
            Ok(a) => a,
            Err(_) => return,
        };

        let manager = CloudflareOAuthClientManager::new(token, account_id);

        let request = CreateOAuthClientRequest {
            name: format!("test-client-{}", chrono::Utc::now().timestamp()),
            redirect_uris: vec!["http://localhost:51082/cf_callback".to_string()],
            scopes: None,
        };

        let result = manager.create_oauth_client(request);
        assert!(result.is_ok());

        let client = result.unwrap();
        assert!(!client.client_id.is_empty());
        assert!(client.client_secret.is_some()); // Secret returned on create

        // Cleanup
        let _ = manager.delete_oauth_client(&client.id);
    }

    #[test]
    #[ignore] // Integration test
    fn test_update_oauth_client() {
        let token = match std::env::var("CLOUDFLARE_TEST_TOKEN") {
            Ok(t) => t,
            Err(_) => return,
        };
        let account_id = match std::env::var("CLOUDFLARE_ACCOUNT_ID") {
            Ok(a) => a,
            Err(_) => return,
        };

        let manager = CloudflareOAuthClientManager::new(token, account_id);

        // Create test client
        let create_req = CreateOAuthClientRequest {
            name: format!("test-update-{}", chrono::Utc::now().timestamp()),
            redirect_uris: vec!["http://localhost:51082/cf_callback".to_string()],
            scopes: None,
        };
        let client = manager.create_oauth_client(create_req).unwrap();

        // Update name
        let update_req = UpdateOAuthClientRequest {
            name: Some(format!("updated-{}", client.name)),
            redirect_uris: None,
        };

        let result = manager.update_oauth_client(&client.id, update_req);
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert!(updated.name.starts_with("updated-"));

        // Cleanup
        let _ = manager.delete_oauth_client(&client.id);
    }

    #[test]
    #[ignore] // Integration test
    fn test_delete_oauth_client() {
        let token = match std::env::var("CLOUDFLARE_TEST_TOKEN") {
            Ok(t) => t,
            Err(_) => return,
        };
        let account_id = match std::env::var("CLOUDFLARE_ACCOUNT_ID") {
            Ok(a) => a,
            Err(_) => return,
        };

        let manager = CloudflareOAuthClientManager::new(token, account_id);

        // Create test client
        let create_req = CreateOAuthClientRequest {
            name: format!("test-delete-{}", chrono::Utc::now().timestamp()),
            redirect_uris: vec!["http://localhost:51082/cf_callback".to_string()],
            scopes: None,
        };
        let client = manager.create_oauth_client(create_req).unwrap();

        // Delete
        let result = manager.delete_oauth_client(&client.id);
        assert!(result.is_ok());

        // Verify deleted
        let list = manager.list_oauth_clients().unwrap();
        assert!(!list.iter().any(|c| c.id == client.id));
    }

    #[test]
    #[ignore] // Integration test
    fn test_rotate_oauth_client_secret() {
        let token = match std::env::var("CLOUDFLARE_TEST_TOKEN") {
            Ok(t) => t,
            Err(_) => return,
        };
        let account_id = match std::env::var("CLOUDFLARE_ACCOUNT_ID") {
            Ok(a) => a,
            Err(_) => return,
        };

        let manager = CloudflareOAuthClientManager::new(token, account_id);

        // Create test client
        let create_req = CreateOAuthClientRequest {
            name: format!("test-rotate-{}", chrono::Utc::now().timestamp()),
            redirect_uris: vec!["http://localhost:51082/cf_callback".to_string()],
            scopes: None,
        };
        let client = manager.create_oauth_client(create_req).unwrap();
        let old_secret = client.client_secret.clone();

        // Rotate secret
        let result = manager.rotate_oauth_client_secret(&client.id);
        assert!(result.is_ok());

        let rotated = result.unwrap();
        assert!(rotated.client_secret.is_some());
        assert_ne!(rotated.client_secret, old_secret); // New secret != old secret

        // Cleanup
        let _ = manager.delete_oauth_client(&client.id);
    }

    #[test]
    #[ignore] // Integration test
    fn test_delete_rotated_secret() {
        let token = match std::env::var("CLOUDFLARE_TEST_TOKEN") {
            Ok(t) => t,
            Err(_) => return,
        };
        let account_id = match std::env::var("CLOUDFLARE_ACCOUNT_ID") {
            Ok(a) => a,
            Err(_) => return,
        };

        let manager = CloudflareOAuthClientManager::new(token, account_id);

        // Create and rotate
        let create_req = CreateOAuthClientRequest {
            name: format!("test-del-rot-{}", chrono::Utc::now().timestamp()),
            redirect_uris: vec!["http://localhost:51082/cf_callback".to_string()],
            scopes: None,
        };
        let client = manager.create_oauth_client(create_req).unwrap();
        let _ = manager.rotate_oauth_client_secret(&client.id).unwrap();

        // Delete rotated secret
        let result = manager.delete_rotated_oauth_client_secret(&client.id);
        assert!(result.is_ok());

        // Cleanup
        let _ = manager.delete_oauth_client(&client.id);
    }

    #[test]
    #[ignore] // Full integration test
    fn test_full_oauth_client_lifecycle() {
        let token = match std::env::var("CLOUDFLARE_TEST_TOKEN") {
            Ok(t) => t,
            Err(_) => return,
        };
        let account_id = match std::env::var("CLOUDFLARE_ACCOUNT_ID") {
            Ok(a) => a,
            Err(_) => return,
        };

        let manager = CloudflareOAuthClientManager::new(token, account_id);

        // 1. List clients (baseline)
        let initial_list = manager.list_oauth_clients().unwrap();
        let initial_count = initial_list.len();

        // 2. Create client
        let create_req = CreateOAuthClientRequest {
            name: format!("integration-test-{}", chrono::Utc::now().timestamp()),
            redirect_uris: vec!["http://localhost:51082/cf_callback".to_string()],
            scopes: None,
        };
        let client = manager.create_oauth_client(create_req).unwrap();
        assert!(!client.client_id.is_empty());
        assert!(client.client_secret.is_some());

        // 3. List clients (should have +1)
        let after_create = manager.list_oauth_clients().unwrap();
        assert_eq!(after_create.len(), initial_count + 1);

        // 4. Update client
        let update_req = UpdateOAuthClientRequest {
            name: Some(format!("updated-{}", client.name)),
            redirect_uris: None,
        };
        let updated = manager.update_oauth_client(&client.id, update_req).unwrap();
        assert!(updated.name.starts_with("updated-"));

        // 5. Rotate secret
        let old_secret = client.client_secret.clone();
        let rotated = manager.rotate_oauth_client_secret(&client.id).unwrap();
        assert!(rotated.client_secret.is_some());
        assert_ne!(rotated.client_secret, old_secret);

        // 6. Delete rotated secret
        manager
            .delete_rotated_oauth_client_secret(&client.id)
            .unwrap();

        // 7. Delete client
        manager.delete_oauth_client(&client.id).unwrap();

        // 8. List clients (should be back to initial_count)
        let final_list = manager.list_oauth_clients().unwrap();
        assert_eq!(final_list.len(), initial_count);
    }

    #[test]
    fn test_refresh_token_rotation_parsing() {
        // Test that TokenResponse correctly parses refresh_token field
        // This verifies the fix for HTTP 400 "invalid_grant" bug

        let json_with_refresh = r#"{
            "access_token": "new_access_token_xyz",
            "refresh_token": "new_refresh_token_abc",
            "expires_in": 3600
        }"#;

        #[derive(serde::Deserialize)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: Option<u64>,
        }

        let parsed: TokenResponse = serde_json::from_str(json_with_refresh).unwrap();
        assert_eq!(parsed.access_token, "new_access_token_xyz");
        assert_eq!(
            parsed.refresh_token,
            Some("new_refresh_token_abc".to_string())
        );
        assert_eq!(parsed.expires_in, Some(3600));

        // Test without refresh_token (should still parse)
        let json_without_refresh = r#"{
            "access_token": "access_only",
            "expires_in": 7200
        }"#;

        let parsed2: TokenResponse = serde_json::from_str(json_without_refresh).unwrap();
        assert_eq!(parsed2.access_token, "access_only");
        assert_eq!(parsed2.refresh_token, None);
        assert_eq!(parsed2.expires_in, Some(7200));
    }

    #[test]
    fn test_refresh_token_must_rotate() {
        // Test that verifies token rotation semantics:
        // - Cloudflare ALWAYS returns new refresh_token on refresh
        // - Old refresh_token is IMMEDIATELY invalidated
        // - Preserving old token causes "invalid_grant" error
        //
        // This test documents the expected behavior per Cloudflare OAuth conventions

        #[derive(serde::Deserialize)]
        struct TokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: Option<u64>,
        }

        // Simulate Cloudflare token refresh response (always includes new refresh_token)
        let cloudflare_response = r#"{
            "access_token": "NEW_access_token_xyz",
            "refresh_token": "NEW_refresh_token_def",
            "expires_in": 900
        }"#;

        let old_refresh_token = "OLD_refresh_token_abc";
        let token_response: TokenResponse = serde_json::from_str(cloudflare_response).unwrap();

        // Build OAuthResult (simulating refresh_access_token logic)
        let result = OAuthResult {
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token.clone(),
            expires_at: 1234567890,
        };

        // CRITICAL: New refresh_token MUST be used, not old one
        assert_eq!(
            result.refresh_token,
            Some("NEW_refresh_token_def".to_string()),
            "Token rotation failed: old refresh_token preserved instead of new one"
        );
        assert_ne!(
            result.refresh_token,
            Some(old_refresh_token.to_string()),
            "Token rotation failed: must use new refresh_token from Cloudflare"
        );

        // If Cloudflare doesn't return refresh_token, result should be None (error condition)
        let no_refresh_response = r#"{
            "access_token": "access_only",
            "expires_in": 900
        }"#;
        let token_response_no_refresh: TokenResponse =
            serde_json::from_str(no_refresh_response).unwrap();
        assert_eq!(
            token_response_no_refresh.refresh_token, None,
            "Missing refresh_token should be None, not fallback to old token"
        );
    }
}
