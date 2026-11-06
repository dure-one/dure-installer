//! Cloudflare API client modules
//!
//! Provides integrations with Cloudflare services:
//! - OAuth: OAuth 2.0 with PKCE for API token acquisition
//! - DNS: Zone and DNS record management
//! - Registrar: Domain registration and nameserver management

pub mod dns;
pub mod oauth;
pub mod registrar;

// Re-export commonly used types
pub use dns::{CloudflareClient, DnsRecord, Zone};
pub use oauth::{CloudflareOAuthHandler, OAuthResult, UserInfo, get_user_info};
pub use registrar::CloudflareRegistrarClient;
