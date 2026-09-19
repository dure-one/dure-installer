//! GCP OAuth2 authentication handler
//!
//! Implements OAuth2 flow for Google Cloud Platform using:
//! - std::net::TcpListener for localhost callback server
//! - ureq for HTTP requests
//! - webbrowser for opening system browser
//!
//! ## OAuth Flow
//!
//! 1. Start local HTTP server on random port (http://localhost:{port}/oauth/callback)
//! 2. Open system browser to Google OAuth URL with required scopes
//! 3. User authenticates and grants permissions
//! 4. Google redirects to localhost callback with authorization code
//! 5. Extract code from query parameters
//! 6. Exchange code for refresh token using ureq
//! 7. Store refresh token securely (keyring)
//! 8. Close local server

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

/// Required GCP OAuth scopes (from Outline-apps)
const GCP_SCOPES: &[&str] = &[
    "https://www.googleapis.com/auth/userinfo.email", // ✅ Safe - just email
    "https://www.googleapis.com/auth/compute", // https://developers.google.com/identity/protocols/oauth2/scopes?text=cloudplatformprojects#compute
    "https://www.googleapis.com/auth/cloudplatformprojects", // https://docs.cloud.google.com/resource-manager/reference/rest/v3/projects/list
    "https://www.googleapis.com/auth/cloud-billing", // https://developers.google.com/identity/protocols/oauth2/scopes?text=cloudplatformprojects#cloudbilling
    "https://www.googleapis.com/auth/service.management", // https://developers.google.com/identity/protocols/oauth2/scopes?text=cloudplatformprojects#servicemanagement
    "https://www.googleapis.com/auth/cloud-platform.read-only", // 🟡 BROAD - read everything
    "https://www.googleapis.com/auth/ndev.clouddns.readwrite", // ✅ NEEDED - DNS management
];

// Note: For native apps, the "client secret" is not actually a secret.
// See https://developers.google.com/identity/protocols/oauth2/native-app
// These credentials are embedded at compile-time from environment variables.
// Development: Load from .env file via build.rs
// CI/CD: Load from GitHub Secrets via build.rs
fn get_oauth_client_id() -> &'static str {
    option_env!("GOOGLE_OAUTH_CLIENT_ID").unwrap_or("")
}

fn get_oauth_client_secret() -> &'static str {
    option_env!("GOOGLE_OAUTH_CLIENT_SECRET").unwrap_or("")
}

/// OAuth result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthResult {
    pub refresh_token: String,
    pub access_token: String,
    pub expires_at: u64,
}

/// OAuth handler
pub struct OAuthHandler {
    client_id: String,
    client_secret: String,
}

impl OAuthHandler {
    /// Create new OAuth handler with embedded credentials
    ///
    /// Uses credentials compiled into the binary at build time.
    /// Credentials come from:
    /// - Development: .env file (via build.rs)
    /// - CI/CD: GitHub Secrets (via build.rs)
    pub fn default() -> Self {
        Self {
            client_id: get_oauth_client_id().to_string(),
            client_secret: get_oauth_client_secret().to_string(),
        }
    }

    /// Create new OAuth handler with custom credentials
    ///
    /// For advanced users who want to use their own OAuth client.
    /// Most users should use `OAuthHandler::default()` instead.
    pub fn new(client_id: String, client_secret: String) -> Self {
        Self {
            client_id,
            client_secret,
        }
    }

    /// Get client ID
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Get client secret
    pub fn client_secret(&self) -> &str {
        &self.client_secret
    }

    /// Run complete OAuth flow (blocking)
    ///
    /// Returns refresh token that can be stored securely.
    pub fn run_oauth_flow(&self) -> Result<OAuthResult> {
        // Bind to random port on localhost
        let listener = TcpListener::bind("127.0.0.1:0").context("Failed to bind to localhost")?;
        let addr = listener.local_addr()?;
        let port = addr.port();

        let redirect_uri = format!("http://localhost:{}/oauth/callback", port);

        // Build OAuth URL
        let state = uuid::Uuid::new_v4().to_string();
        let auth_url = self.build_auth_url(&redirect_uri, &state)?;

        // Open browser
        dure_info!("Opening browser for OAuth authorization...");
        dure_info!("If the browser doesn't open, visit: {}", auth_url);
        if let Err(e) = webbrowser::open(&auth_url) {
            dure_warn!("Failed to open browser: {}", e);
            dure_info!("Please manually open this URL:");
            dure_info!("{}", auth_url);
        }

        // Wait for OAuth callback
        dure_info!("Waiting for OAuth callback on http://localhost:{}...", port);

        // Accept one connection (blocking)
        let (stream, _) = listener.accept().context("Failed to accept connection")?;

        // Handle the request
        let authorization_code = handle_oauth_request(stream, state)?;

        // Exchange code for tokens
        let oauth_result = self.exchange_code_for_token(&authorization_code, &redirect_uri)?;

        Ok(oauth_result)
    }

    /// Build OAuth authorization URL
    fn build_auth_url(&self, redirect_uri: &str, state: &str) -> Result<String> {
        let scope = GCP_SCOPES.join(" ");

        let url = format!(
            "https://accounts.google.com/o/oauth2/v2/auth?\
             client_id={}&\
             redirect_uri={}&\
             response_type=code&\
             scope={}&\
             state={}&\
             access_type=offline&\
             prompt=consent",
            urlencoding::encode(&self.client_id),
            urlencoding::encode(redirect_uri),
            urlencoding::encode(&scope),
            urlencoding::encode(state),
        );

        Ok(url)
    }

    /// Exchange authorization code for access and refresh tokens
    fn exchange_code_for_token(&self, code: &str, redirect_uri: &str) -> Result<OAuthResult> {
        let params = [
            ("code", code),
            ("client_id", &self.client_id),
            ("client_secret", &self.client_secret),
            ("redirect_uri", redirect_uri),
            ("grant_type", "authorization_code"),
        ];

        let body = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let response = ureq::post("https://oauth2.googleapis.com/token")
            .set("Content-Type", "application/x-www-form-urlencoded")
            .send_string(&body)
            .context("Failed to exchange code for token")?;

        if response.status() != 200 {
            let error_text = response.into_string().unwrap_or_default();
            return Err(anyhow::anyhow!("Token exchange failed: {}", error_text));
        }

        let token_response: TokenResponse = response
            .into_json()
            .context("Failed to parse token response")?;

        let refresh_token = token_response
            .refresh_token
            .ok_or_else(|| anyhow::anyhow!("No refresh token in response"))?;

        let expires_at = chrono::Utc::now().timestamp() as u64 + token_response.expires_in;

        Ok(OAuthResult {
            refresh_token,
            access_token: token_response.access_token,
            expires_at,
        })
    }
}

/// Token exchange response
#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    refresh_token: Option<String>,
    scope: String,
    token_type: String,
}

/// Handle OAuth callback request
fn handle_oauth_request(mut stream: TcpStream, expected_state: String) -> Result<String> {
    let mut buffer = [0u8; 4096];
    let n = stream
        .read(&mut buffer)
        .context("Failed to read from stream")?;

    let request = String::from_utf8_lossy(&buffer[..n]);

    // Parse the HTTP request
    let first_line = request
        .lines()
        .next()
        .ok_or_else(|| anyhow::anyhow!("Empty request"))?;

    // Extract path from "GET /oauth/callback?code=... HTTP/1.1"
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return Err(anyhow::anyhow!("Invalid HTTP request"));
    }

    let path = parts[1];

    // Parse query parameters
    let (_, query) = path
        .split_once('?')
        .ok_or_else(|| anyhow::anyhow!("No query parameters"))?;

    let mut code = None;
    let mut state = None;
    let mut error = None;

    for param in query.split('&') {
        if let Some((key, value)) = param.split_once('=') {
            match key {
                "code" => code = Some(urlencoding::decode(value)?.into_owned()),
                "state" => state = Some(urlencoding::decode(value)?.into_owned()),
                "error" => error = Some(urlencoding::decode(value)?.into_owned()),
                _ => {}
            }
        }
    }

    // Validate state to prevent CSRF
    if state.as_deref() != Some(expected_state.as_str()) {
        send_error_response(&mut stream, "Invalid state parameter")?;
        return Err(anyhow::anyhow!("Invalid state parameter"));
    }

    // Check for errors
    if let Some(err) = error {
        send_error_response(&mut stream, &format!("OAuth error: {}", err))?;
        return Err(anyhow::anyhow!("OAuth error: {}", err));
    }

    // Get authorization code
    let authorization_code =
        code.ok_or_else(|| anyhow::anyhow!("No authorization code in callback"))?;

    // Send success response
    send_success_response(&mut stream)?;

    Ok(authorization_code)
}

/// Send success HTML response
fn send_success_response(stream: &mut TcpStream) -> Result<()> {
    let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>Authorization Complete</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
        }
        .container {
            background: white;
            padding: 2rem;
            border-radius: 8px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
            text-align: center;
            max-width: 400px;
        }
        h1 {
            color: #2d3748;
            margin-top: 0;
        }
        p {
            color: #4a5568;
            line-height: 1.6;
        }
        .checkmark {
            font-size: 4rem;
            color: #48bb78;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="checkmark">✓</div>
        <h1>Authorization Complete</h1>
        <p>You have successfully authorized Dure to access your Google Cloud Platform account.</p>
        <p>You can close this window and return to the application.</p>
    </div>
</body>
</html>"#;

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
        html.len(),
        html
    );

    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}

/// Send error HTML response
fn send_error_response(stream: &mut TcpStream, error_msg: &str) -> Result<()> {
    let html = format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>Authorization Failed</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            display: flex;
            justify-content: center;
            align-items: center;
            height: 100vh;
            margin: 0;
            background: linear-gradient(135deg, #f56565 0%, #c53030 100%);
        }}
        .container {{
            background: white;
            padding: 2rem;
            border-radius: 8px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
            text-align: center;
            max-width: 400px;
        }}
        h1 {{
            color: #2d3748;
            margin-top: 0;
        }}
        p {{
            color: #4a5568;
            line-height: 1.6;
        }}
        .error {{
            font-size: 4rem;
            color: #f56565;
        }}
    </style>
</head>
<body>
    <div class="container">
        <div class="error">✗</div>
        <h1>Authorization Failed</h1>
        <p>{}</p>
        <p>Please close this window and try again.</p>
    </div>
</body>
</html>"#,
        error_msg
    );

    let response = format!(
        "HTTP/1.1 400 Bad Request\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
        html.len(),
        html
    );

    stream.write_all(response.as_bytes())?;
    stream.flush()?;

    Ok(())
}

/// Helper to refresh access token from refresh token
pub fn refresh_access_token(
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<OAuthResult> {
    // Validate inputs
    if client_id.is_empty() {
        dure_error!("OAuth client_id is empty - check GOOGLE_OAUTH_CLIENT_ID environment variable");
        return Err(anyhow::anyhow!("OAuth client_id is not configured"));
    }
    if client_secret.is_empty() {
        dure_error!("OAuth client_secret is empty - check GOOGLE_OAUTH_CLIENT_SECRET environment variable");
        return Err(anyhow::anyhow!("OAuth client_secret is not configured"));
    }
    if refresh_token.is_empty() {
        return Err(anyhow::anyhow!("Refresh token is empty"));
    }

    dure_debug!("Refreshing access token (client_id: {}...)", &client_id.chars().take(10).collect::<String>());

    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    let body = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let response = ureq::post("https://oauth2.googleapis.com/token")
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

    if response.status() != 200 {
        let status = response.status();
        let error_text = response.into_string().unwrap_or_default();
        dure_error!("Unexpected status {}: {}", status, error_text);
        return Err(anyhow::anyhow!("Token refresh failed: {}", error_text));
    }

    let token_response: TokenResponse = response
        .into_json()
        .context("Failed to parse token response")?;

    let expires_at = chrono::Utc::now().timestamp() as u64 + token_response.expires_in;

    Ok(OAuthResult {
        refresh_token: refresh_token.to_string(), // Keep original refresh token
        access_token: token_response.access_token,
        expires_at,
    })
}

/// OAuth2 user info response
#[derive(Debug, Deserialize)]
pub struct UserInfo {
    pub email: Option<String>,
    pub verified_email: Option<bool>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
}

/// Get user info from OAuth2 userinfo endpoint
pub fn get_user_info(access_token: &str) -> Result<UserInfo> {
    let url = "https://www.googleapis.com/oauth2/v2/userinfo";
    let response = ureq::get(url)
        .set("Authorization", &format!("Bearer {}", access_token))
        .call()
        .context("Failed to get user info")?;
    let info: UserInfo = response.into_json()?;
    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scopes_defined() {
        assert!(!GCP_SCOPES.is_empty());
        assert!(GCP_SCOPES.contains(&"https://www.googleapis.com/auth/compute"));
    }

    #[test]
    fn test_build_auth_url() {
        let handler = OAuthHandler::new("test_client_id".to_string(), "test_secret".to_string());

        let url = handler
            .build_auth_url("http://localhost:8080/callback", "test_state")
            .expect("Failed to build URL");

        assert!(url.contains("client_id=test_client_id"));
        assert!(url.contains("state=test_state"));
        assert!(url.contains("access_type=offline"));
        assert!(url.contains("prompt=consent"));
    }
}
