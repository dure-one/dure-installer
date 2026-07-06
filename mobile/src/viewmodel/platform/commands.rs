//! Platform actor commands

#[derive(Debug, Clone)]
pub enum PlatformCommand {
    // OAuth & Platform Management
    StartOAuth {
        platform_name: String,
    },
    CompleteOAuth {
        platform_name: String,
        auth_code: String,
    },
    AddPlatform {
        name: String,
        platform_type: String,
        oauth_access_token: Option<String>,
        oauth_refresh_token: Option<String>,
        oauth_token_expiry: Option<i64>,
        connected_email: Option<String>,
        selected_project_id: Option<String>,
    },
    DeletePlatform {
        platform_name: String,
    },

    // Project Operations
    ListProjects {
        platform_name: String,
    },
    SelectProject {
        platform_name: String,
        project_id: String,
    },

    // VM Operations
    ListVMs {
        platform_name: String,
    },
    CreateVM {
        platform_name: String,
        vm_name: String,
        zone: String,
        machine_type: String,
    },
    DeleteVM {
        platform_name: String,
        vm_name: String,
        zone: String,
    },
    RestartVM {
        platform_name: String,
        vm_name: String,
        zone: String,
    },
    RegenerateVM {
        platform_name: String,
        vm_name: String,
        zone: String,
    },

    // Firewall Operations
    UpdateFirewall {
        platform_name: String,
        allow_ip: String,
    },

    // Billing Operations
    FetchBilling {
        platform_name: String,
        project_id: String,
        dataset: String,
        table: String,
    },

    // Refresh all platform data
    RefreshAll,
}
