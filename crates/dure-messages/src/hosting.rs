//! Hosting management message types

use asyncapi_rust::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Hosting initialization request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct HostingInitRequest {
    /// Domain name
    pub domain: String,
    /// DNS provider (CLOUDFLARE_DNS, PORKBUN, DUCKDNS, GCP_CLOUDDNS)
    pub dns_provider: String,
    /// DNS provider token/API key
    pub dns_provider_token: String,
    /// Web provider (GCE, VPS, CLOUDFLARE_PAGES, FIREBASE_HOSTING)
    pub web_provider: String,
    /// Web provider token/API key
    pub web_provider_token: Option<String>,
    /// Database provider (GCE, GCP_CLOUDSQL, SUPABASE)
    pub db_provider: String,
}

/// Hosting initialization response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct HostingInitResponse {
    /// Whether initialization was successful
    pub success: bool,
    /// Hosting ID
    pub hosting_id: Option<String>,
    /// Domain name
    pub domain: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Setup steps completed
    pub steps_completed: Vec<String>,
}

/// Show hosting details request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct HostingShowRequest {
    /// Optional hosting domain/ID to show (if empty, shows current selected)
    pub hosting_id: Option<String>,
}

/// Show hosting details response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct HostingShowResponse {
    /// Hosting details
    pub hosting: Option<HostingDetails>,
    /// Error message if not found
    pub error: Option<String>,
}

/// Hosting details
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct HostingDetails {
    /// Hosting ID
    pub id: String,
    /// Domain name
    pub domain: String,
    /// DNS provider
    pub dns_provider: String,
    /// Web provider
    pub web_provider: String,
    /// Database provider
    pub db_provider: String,
    /// Whether hosting is active
    pub is_active: bool,
    /// Whether hosting is selected for operations
    pub is_selected: bool,
    /// Server IP address
    pub server_ip: Option<String>,
    /// SSL certificate status
    pub ssl_status: Option<String>,
}

/// Select hosting request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct HostingSelectRequest {
    /// Hosting ID/domain to select
    pub hosting_id: String,
}

/// Select hosting response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct HostingSelectResponse {
    /// Whether selection was successful
    pub success: bool,
    /// Selected hosting ID
    pub hosting_id: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
}

/// List hostings request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct HostingListRequest {
    /// Optional filter by status
    pub filter_active: Option<bool>,
}

/// List hostings response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct HostingListResponse {
    /// List of hostings
    pub hostings: Vec<HostingDetails>,
    /// Total count
    pub total: usize,
}

/// Close hosting request
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct HostingCloseRequest {
    /// Hosting ID/domain to close
    pub hosting_id: String,
    /// Confirmation flag
    pub confirm: bool,
}

/// Close hosting response
#[derive(Serialize, Deserialize, JsonSchema, Clone, Debug)]
pub struct HostingCloseResponse {
    /// Whether closure was successful
    pub success: bool,
    /// Message
    pub message: Option<String>,
    /// Error if failed
    pub error: Option<String>,
}
