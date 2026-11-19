//! Platform actor events

use crate::api::gcp::bigquery::BillingRecord;
use crate::{dure_debug, dure_error, dure_info, dure_warn};

#[derive(Debug, Clone)]
pub struct VmInfo {
    pub name: String,
    pub project_id: String,
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

/// Firewall rule information with IP inclusion check (simplified)
#[derive(Debug, Clone)]
pub struct FirewallRuleInfo {
    pub name: String,
    pub source_ranges: Vec<String>,
    pub ip_included: bool,
    pub allows_ssh: bool,
}

/// Complete firewall rule data with all GCP fields
#[derive(Debug, Clone)]
pub struct FullFirewallRule {
    pub name: String,
    pub network: String,
    pub direction: String,
    pub priority: u32,
    pub source_ranges: Vec<String>,
    pub dest_ranges: Vec<String>,
    pub allowed: Vec<String>, // Formatted as "tcp:22", "udp:53", "icmp"
    pub denied: Vec<String>,
    pub source_tags: Vec<String>,
    pub source_service_accounts: Vec<String>,
    pub target_tags: Vec<String>,
    pub target_service_accounts: Vec<String>,
    pub ip_included: bool,
    pub allows_ssh: bool,
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
    PlatformOAuthUpdated {
        platform_name: String,
        connected_email: String,
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
        focus_tab: Option<crate::viewmodel::platform::DrawerTab>,
    },
    VMDeleted {
        platform_name: String,
        vm_name: String,
        focus_tab: Option<crate::viewmodel::platform::DrawerTab>,
    },
    VmStaticIpChecked {
        platform_name: String,
        vm_name: String,
        has_static_ip: bool,
        ip_address: Option<String>,
        ip_name: Option<String>,
    },
    VMRestarted {
        platform_name: String,
        vm_name: String,
        focus_tab: Option<crate::viewmodel::platform::DrawerTab>,
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
        focus_tab: Option<crate::viewmodel::platform::DrawerTab>,
    },
    FirewallRulesFetched {
        platform_name: String,
        rules: Vec<FullFirewallRule>,
        current_ip: String,
    },
    FirewallRuleUpdated {
        platform_name: String,
        rule_name: String,
        operation: String, // "add_ip", "remove_ip", "flush"
        focus_tab: Option<crate::viewmodel::platform::DrawerTab>,
    },
    IpWhitelistChecked {
        platform_name: String,
        whitelisted: bool,
        current_ip: String,
    },

    // Billing Events
    BillingFetched {
        platform_name: String,
        records: Vec<BillingRecord>,
    },
    BillingTableDiscovered {
        platform_name: String,
        dataset: String,
        table: String,
    },

    /// Refresh completed with comprehensive status
    RefreshCompleted {
        platform_name: String,
        vm_status: VmStatus,
        firewall_status: FirewallStatus,
        ssh_status: SshStatus,
        project_count: Option<usize>, // Total number of GCP projects accessible
        focus_tab: Option<crate::viewmodel::platform::DrawerTab>,
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
        operation: String, // "firewall", "restart", etc.
        error: String,
    },

    // Operation Logging Events
    /// Operation log created
    OperationLogCreated {
        platform_name: String,
        log_id: i64,
        operation_type: String,
    },

    /// Operation log updated
    OperationLogUpdated {
        log_id: i64,
        status: String,
    },
}
