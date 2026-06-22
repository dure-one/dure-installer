//! Session management functionality for HTTP/WSS connections
//!
//! Provides unified session tracking for both HTTP and WebSocket connections.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Session information for HTTP/WSS connections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub domain: String,
    pub session_type: SessionType,
    pub connected_at: u64,
    pub last_seen: u64,
    pub request_count: u64,
    pub remote_addr: String,
}

/// Session type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionType {
    Http,
    Wss,
}

impl SessionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionType::Http => "http",
            SessionType::Wss => "wss",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "http" => Some(SessionType::Http),
            "wss" => Some(SessionType::Wss),
            _ => None,
        }
    }
}

impl Session {
    pub fn new(
        session_id: String,
        domain: String,
        session_type: SessionType,
        remote_addr: String,
    ) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();

        Self {
            session_id,
            domain,
            session_type,
            connected_at: now,
            last_seen: now,
            request_count: 0,
            remote_addr,
        }
    }

    /// Check if session is active within the timeout period
    pub fn is_active(&self, timeout_secs: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();

        now - self.last_seen < timeout_secs
    }

    /// Update last_seen timestamp
    pub fn touch(&mut self) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();

        self.last_seen = now;
        self.request_count += 1;
    }

    /// Get session age in seconds
    pub fn age_secs(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();

        now - self.connected_at
    }

    /// Get idle time in seconds
    pub fn idle_secs(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(std::time::Duration::from_secs(0))
            .as_secs();

        now - self.last_seen
    }
}

/// Generate a unique session ID
pub fn generate_session_id() -> String {
    use rand::Rng;
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let random: u32 = rand::thread_rng().r#gen();
    format!("session-{}-{}", timestamp, random)
}

/// Extract session ID from cookie string
pub fn extract_session_from_cookie(cookie: &str) -> Option<String> {
    cookie
        .split(';')
        .find(|c| c.trim().starts_with("session_id="))
        .and_then(|c| c.trim().strip_prefix("session_id="))
        .map(|s| s.to_string())
}

/// Parse session ID from header value
pub fn parse_session_id(value: &str) -> Option<String> {
    if value.starts_with("session-") {
        Some(value.to_string())
    } else {
        None
    }
}

/// Create session cookie string
pub fn create_session_cookie(session_id: &str) -> String {
    format!(
        "session_id={}; Path=/; HttpOnly; Secure; SameSite=Strict",
        session_id
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new(
            "test-session".to_string(),
            "example.com".to_string(),
            SessionType::Http,
            "127.0.0.1:1234".to_string(),
        );

        assert_eq!(session.session_id, "test-session");
        assert_eq!(session.domain, "example.com");
        assert_eq!(session.session_type, SessionType::Http);
        assert_eq!(session.request_count, 0);
        assert!(session.is_active(3600));
    }

    #[test]
    fn test_session_touch() {
        let mut session = Session::new(
            "test-session".to_string(),
            "example.com".to_string(),
            SessionType::Http,
            "127.0.0.1:1234".to_string(),
        );

        let old_last_seen = session.last_seen;
        std::thread::sleep(std::time::Duration::from_secs(1));
        session.touch();

        assert!(session.last_seen > old_last_seen);
        assert_eq!(session.request_count, 1);
    }

    #[test]
    fn test_session_type() {
        assert_eq!(SessionType::Http.as_str(), "http");
        assert_eq!(SessionType::Wss.as_str(), "wss");

        assert_eq!(SessionType::from_str("http"), Some(SessionType::Http));
        assert_eq!(SessionType::from_str("wss"), Some(SessionType::Wss));
        assert_eq!(SessionType::from_str("invalid"), None);
    }

    #[test]
    fn test_generate_session_id() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();

        assert!(id1.starts_with("session-"));
        assert!(id2.starts_with("session-"));
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_extract_session_from_cookie() {
        let cookie = "session_id=test-123; Path=/";
        let session_id = extract_session_from_cookie(cookie);
        assert_eq!(session_id, Some("test-123".to_string()));

        let cookie = "other=value; session_id=test-456";
        let session_id = extract_session_from_cookie(cookie);
        assert_eq!(session_id, Some("test-456".to_string()));

        let cookie = "no_session=here";
        let session_id = extract_session_from_cookie(cookie);
        assert_eq!(session_id, None);
    }

    #[test]
    fn test_parse_session_id() {
        assert_eq!(
            parse_session_id("session-123-456"),
            Some("session-123-456".to_string())
        );
        assert_eq!(parse_session_id("invalid"), None);
    }

    #[test]
    fn test_create_session_cookie() {
        let cookie = create_session_cookie("test-123");
        assert!(cookie.starts_with("session_id=test-123"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("Secure"));
        assert!(cookie.contains("SameSite=Strict"));
    }
}
