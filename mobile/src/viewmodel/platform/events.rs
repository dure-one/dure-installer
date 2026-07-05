//! Platform actor events

use crate::calc::gcp_rest::BillingRecord;

#[derive(Debug, Clone)]
pub struct VmInfo {
    pub name: String,
    pub zone: String,
    pub external_ip: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone)]
pub enum PlatformEvent {
    // OAuth Events
    OAuthStarted { platform_name: String, auth_url: String },
    OAuthCompleted { platform_name: String, email: String },

    // Platform Management Events
    PlatformAdded { platform_name: String, platform_type: String },
    PlatformDeleted { platform_name: String, vm_count: usize },

    // Project Events
    ProjectsListed {
        platform_name: String,
        projects: Vec<(String, String)>  // (id, name)
    },
    ProjectSelected { platform_name: String, project_id: String },

    // VM Events
    VMsListed {
        platform_name: String,
        vms: Vec<VmInfo>
    },
    VMCreated { platform_name: String, vm_name: String, external_ip: String },
    VMDeleted { platform_name: String, vm_name: String },
    VMRestarted { platform_name: String, vm_name: String },
    VMRegenerated { platform_name: String, vm_name: String, message: String },

    // Firewall Events
    FirewallUpdated { platform_name: String, whitelisted_ip: String },

    // Billing Events
    BillingFetched {
        platform_name: String,
        records: Vec<BillingRecord>
    },

    // Progress & Errors
    Progress {
        operation: String,
        progress: f32,
        status: String
    },
    Error { operation: String, error: String },
}
