//! Webhook management functionality
//!
//! Provides webhook pattern matching and request logging capabilities.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Webhook configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub logging_enabled: bool,
}

impl WebhookConfig {
    pub fn new() -> Self {
        Self {
            logging_enabled: false,
        }
    }

    pub fn with_logging(mut self, enabled: bool) -> Self {
        self.logging_enabled = enabled;
        self
    }
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Webhook allow pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookAllowPattern {
    pub id: i64,
    pub pattern: String,
    pub created_at: u64,
}

impl WebhookAllowPattern {
    pub fn new(id: i64, pattern: String, created_at: u64) -> Self {
        Self {
            id,
            pattern,
            created_at,
        }
    }
}

/// Webhook request log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookRequest {
    pub id: i64,
    pub pattern: String,
    pub path: String,
    pub method: String,
    pub headers: String, // JSON string
    pub body: String,
    pub remote_addr: String,
    pub received_at: u64,
}

impl WebhookRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i64,
        pattern: String,
        path: String,
        method: String,
        headers: String,
        body: String,
        remote_addr: String,
        received_at: u64,
    ) -> Self {
        Self {
            id,
            pattern,
            path,
            method,
            headers,
            body,
            remote_addr,
            received_at,
        }
    }
}

/// Webhook POST request data
#[derive(Debug, Clone)]
pub struct WebhookPostRequest {
    pub path: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub remote_addr: String,
}

impl WebhookPostRequest {
    pub fn new(
        path: String,
        method: String,
        headers: Vec<(String, String)>,
        body: Vec<u8>,
        remote_addr: String,
    ) -> Self {
        Self {
            path,
            method,
            headers,
            body,
            remote_addr,
        }
    }

    /// Convert headers to JSON string
    pub fn headers_as_json(&self) -> String {
        let headers: Vec<_> = self
            .headers
            .iter()
            .map(|(k, v)| format!("\"{}\":\"{}\"", k, v))
            .collect();
        format!("{{{}}}", headers.join(","))
    }

    /// Convert body to string (UTF-8)
    pub fn body_as_string(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }
}

/// Check if a path matches any allow pattern and return the matched pattern
pub fn check_webhook_pattern_match(patterns: &[WebhookAllowPattern], path: &str) -> Option<String> {
    for pattern in patterns {
        if path_matches_pattern(path, &pattern.pattern) {
            return Some(pattern.pattern.clone());
        }
    }
    None
}

/// Simple pattern matching (supports wildcards)
fn path_matches_pattern(path: &str, pattern: &str) -> bool {
    // Simple wildcard matching: * matches any sequence
    if pattern == "*" {
        return true;
    }

    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];
            return path.starts_with(prefix) && path.ends_with(suffix);
        }
    }

    // Exact match
    path == pattern
}

/// Validate webhook pattern
pub fn validate_webhook_pattern(pattern: &str) -> Result<()> {
    if pattern.is_empty() {
        anyhow::bail!("Pattern cannot be empty");
    }

    if pattern.len() > 1024 {
        anyhow::bail!("Pattern too long (max 1024 characters)");
    }

    // Check for invalid characters
    if pattern.contains('\0') {
        anyhow::bail!("Pattern contains null character");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_webhook_config() {
        let config = WebhookConfig::new();
        assert!(!config.logging_enabled);

        let config = WebhookConfig::new().with_logging(true);
        assert!(config.logging_enabled);
    }

    #[test]
    fn test_path_matches_pattern() {
        assert!(path_matches_pattern("/webhook/test", "*"));
        assert!(path_matches_pattern("/webhook/test", "/webhook/*"));
        assert!(path_matches_pattern("/webhook/test", "/webhook/test"));
        assert!(!path_matches_pattern("/webhook/test", "/other/*"));
    }

    #[test]
    fn test_check_webhook_pattern_match() {
        let patterns = vec![
            WebhookAllowPattern::new(1, "/webhook/*".to_string(), 1000),
            WebhookAllowPattern::new(2, "/api/v1/*".to_string(), 2000),
        ];

        let matched = check_webhook_pattern_match(&patterns, "/webhook/test");
        assert_eq!(matched, Some("/webhook/*".to_string()));

        let matched = check_webhook_pattern_match(&patterns, "/api/v1/data");
        assert_eq!(matched, Some("/api/v1/*".to_string()));

        let matched = check_webhook_pattern_match(&patterns, "/other/path");
        assert_eq!(matched, None);
    }

    #[test]
    fn test_validate_webhook_pattern() {
        assert!(validate_webhook_pattern("/webhook/*").is_ok());
        assert!(validate_webhook_pattern("*").is_ok());
        assert!(validate_webhook_pattern("/api/test").is_ok());

        assert!(validate_webhook_pattern("").is_err());
        assert!(validate_webhook_pattern(&"x".repeat(2000)).is_err());
        assert!(validate_webhook_pattern("/test\0abc").is_err());
    }

    #[test]
    fn test_webhook_post_request() {
        let headers = vec![
            ("Content-Type".to_string(), "application/json".to_string()),
            ("User-Agent".to_string(), "test-client".to_string()),
        ];

        let body = b"{\"test\":\"data\"}".to_vec();

        let req = WebhookPostRequest::new(
            "/webhook/test".to_string(),
            "POST".to_string(),
            headers,
            body,
            "127.0.0.1:1234".to_string(),
        );

        let json_headers = req.headers_as_json();
        assert!(json_headers.contains("Content-Type"));
        assert!(json_headers.contains("application/json"));

        let body_str = req.body_as_string();
        assert_eq!(body_str, "{\"test\":\"data\"}");
    }
}
