//! Google Cloud Platform API modules organized by service domain

pub mod compute;
pub mod resourcemanager;
pub mod billing;
pub mod bigquery;
pub mod serviceusage;
pub mod dns;
pub mod oauth;

// Re-export commonly used types for convenience
// (Uncomment as types are migrated)
// pub use compute::{Instance, InstanceRequest, FirewallRule};
// pub use resourcemanager::Project;
// pub use billing::BillingAccount;
// pub use bigquery::BigQueryResponse;
// pub use oauth::OAuthHandler;
