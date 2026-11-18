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
pub use oauth::{
    get_user_info, CloudflareOAuthClientManager, CloudflareOAuthHandler, CreateOAuthClientRequest,
    OAuthClient, OAuthResult, UpdateOAuthClientRequest, UserInfo,
};
pub use registrar::{
    CloudflareRegistrarClient, CreateRegistrationRequest, Domain, DomainPricing,
    DomainSearchResult, Registration, RegistrationContext, RegistrationStatus,
};
