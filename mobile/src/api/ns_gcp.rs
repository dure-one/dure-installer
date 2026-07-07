//! Google Cloud DNS API implementation (re-export)
//!
//! This module re-exports the GCP DNS API from the new location.
//! For implementation details, see `crate::api::gcp::dns`.

// Re-export all public items from the new location
pub use crate::api::gcp::dns::{GcpDnsClient, ManagedZone, Project, ResourceRecordSet};
