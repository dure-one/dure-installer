//! Platform actor events

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use crate::api::gcp::bigquery::BillingRecord;

#[derive(Debug, Clone)]
pub struct VmInfo {
    pub name: String,
    pub zone: String,
    pub external_ip: Option<String>,
    pub status: String,
}

/// VM existence and network status
#[derive(Debug, Clone)]
pub struct VmStatus {
    pub exists: bool,
    pub name: Option<String>,
    pub zone: Option<String>,
    pub external_ip: Option<String>,
    pub status: Option<String>, // "RUNNING", "STOPPED", etc.
}

/// Firewall whitelist status
#[derive(Debug, Clone)]
pub struct FirewallStatus {
    pub whitelisted: bool,
    pub current_ip: Option<String>,
}

/// SSH connectivity status
#[derive(Debug, Clone)]
pub struct SshStatus {
    pub connected: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PlatformEvent {
    // OAuth Events
    OAuthStarted {
        platform_name: String,
        auth_url: String,
    },
    OAuthCompleted {
        platform_name: String,
        email: String,
    },

    // Platform Management Events
    PlatformAdded {
        platform_name: String,
        platform_type: String,
    },
    PlatformDeleted {
        platform_name: String,
        vm_count: usize,
    },

    // Project Events
    ProjectsListed {
        platform_name: String,
        projects: Vec<(String, String)>, // (id, name)
    },
    ProjectSelected {
        platform_name: String,
        project_id: String,
    },

    // VM Events
    VMsListed {
        platform_name: String,
        vms: Vec<VmInfo>,
    },
    VMCreated {
        platform_name: String,
        vm_name: String,
        external_ip: String,
    },
    VMDeleted {
        platform_name: String,
        vm_name: String,
    },
    VMRestarted {
        platform_name: String,
        vm_name: String,
    },
    VMRegenerated {
        platform_name: String,
        vm_name: String,
        message: String,
    },
    VMsScanned {
        platform_name: String,
        vm_count: usize,
    },

    // Firewall Events
    FirewallUpdated {
        platform_name: String,
        whitelisted_ip: String,
    },

    // Billing Events
    BillingFetched {
        platform_name: String,
        records: Vec<BillingRecord>,
    },

    /// Refresh completed with comprehensive status
    RefreshCompleted {
        platform_name: String,
        vm_status: VmStatus,
        firewall_status: FirewallStatus,
        ssh_status: SshStatus,
        project_count: Option<usize>, // Total number of GCP projects accessible
    },

    // Progress & Errors
    Progress {
        operation: String,
        progress: f32,
        status: String,
    },
    Error {
        operation: String,
        error: String,
    },

    /// Operation failed with error
    OperationFailed {
        platform_name: String,
        operation: String,    // "firewall", "restart", etc.
        error: String,
    },
}
