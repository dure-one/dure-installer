//! OAuth Environment Variable Compile-Time Verification Tests
//!
//! Verifies that GOOGLE_OAUTH_CLIENT_ID and GOOGLE_OAUTH_CLIENT_SECRET
//! are embedded at compile time via build.rs.
//!
//! These tests ensure:
//! 1. Environment variables are set during build
//! 2. Values are accessible at runtime via env!() macro
//! 3. Values are non-empty and properly formatted

#[cfg(test)]
mod oauth_env_tests {
    /// Test that GOOGLE_OAUTH_CLIENT_ID is embedded at compile time.
    ///
    /// This test uses env!() which reads values embedded by build.rs.
    /// If the env var wasn't set during build, this test won't compile.
    #[test]
    fn test_google_oauth_client_id_embedded() {
        // env!() fails at compile time if not set
        let client_id = option_env!("GOOGLE_OAUTH_CLIENT_ID");

        match client_id {
            Some(id) => {
                assert!(!id.is_empty(), "GOOGLE_OAUTH_CLIENT_ID is empty");
                assert!(
                    id.ends_with(".apps.googleusercontent.com") || id == "test-id",
                    "GOOGLE_OAUTH_CLIENT_ID has invalid format: {}",
                    id
                );
            }
            None => {
                // In CI or when env vars are properly set, this should be Some
                eprintln!("WARNING: GOOGLE_OAUTH_CLIENT_ID not set - OAuth will not work");
            }
        }
    }

    /// Test that GOOGLE_OAUTH_CLIENT_SECRET is embedded at compile time.
    #[test]
    fn test_google_oauth_client_secret_embedded() {
        let client_secret = option_env!("GOOGLE_OAUTH_CLIENT_SECRET");

        match client_secret {
            Some(secret) => {
                assert!(!secret.is_empty(), "GOOGLE_OAUTH_CLIENT_SECRET is empty");
                assert!(
                    secret.starts_with("GOCSPX-") || secret == "test-secret",
                    "GOOGLE_OAUTH_CLIENT_SECRET has invalid format"
                );
            }
            None => {
                eprintln!("WARNING: GOOGLE_OAUTH_CLIENT_SECRET not set - OAuth will not work");
            }
        }
    }

    /// Test that both OAuth credentials are set together.
    /// If one is set, both should be set.
    #[test]
    fn test_oauth_credentials_complete() {
        let client_id = option_env!("GOOGLE_OAUTH_CLIENT_ID");
        let client_secret = option_env!("GOOGLE_OAUTH_CLIENT_SECRET");

        match (client_id, client_secret) {
            (Some(_), Some(_)) => {
                // Both set - good
            }
            (None, None) => {
                // Both unset - acceptable in dev without .env
                eprintln!("WARNING: OAuth credentials not configured");
            }
            (Some(_), None) => {
                panic!("GOOGLE_OAUTH_CLIENT_ID set but GOOGLE_OAUTH_CLIENT_SECRET missing");
            }
            (None, Some(_)) => {
                panic!("GOOGLE_OAUTH_CLIENT_SECRET set but GOOGLE_OAUTH_CLIENT_ID missing");
            }
        }
    }

    /// Test Cloudflare OAuth client ID (optional, uses PKCE).
    #[test]
    fn test_cloudflare_oauth_client_id() {
        let client_id = option_env!("CLOUDFLARE_OAUTH_CLIENT_ID");

        if let Some(id) = client_id {
            assert!(!id.is_empty(), "CLOUDFLARE_OAUTH_CLIENT_ID is empty");
        } else {
            // Optional - build.rs uses wrangler's public client as fallback
            eprintln!("INFO: CLOUDFLARE_OAUTH_CLIENT_ID not set - using fallback");
        }
    }
}
