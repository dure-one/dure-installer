//! Unit tests for Cloudflare OAuth (inline tests)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oauth_handler_creation() {
        let handler = CloudflareOAuthHandler::new();
        assert_eq!(handler.client_id(), "54d11594-84e4-41aa-b438-e81b8fa78ee7");
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
        use super::super::oauth::{generate_pkce_pair, get_oauth_client_id};

        let handler = CloudflareOAuthHandler::new();
        let (_, code_challenge) = generate_pkce_pair();
        let redirect_uri = "http://localhost:8976/oauth/callback";
        let url = handler.build_auth_url(redirect_uri, "state123", &code_challenge).unwrap();

        assert!(url.contains(&format!("client_id={}", get_oauth_client_id())));
        assert!(url.contains("redirect_uri=http%3A%2F%2Flocalhost%3A8976"));
        assert!(url.contains("state=state123"));
        assert!(url.contains("response_type=code"));
        assert!(url.contains("code_challenge="));
        assert!(url.contains("code_challenge_method=S256"));
        assert!(url.starts_with("https://dash.cloudflare.com/oauth2/auth"));
    }
}
