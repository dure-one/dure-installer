//! Platform actor commands

#[derive(Debug, Clone)]
pub struct DeleteOptions {
    pub delete_vms: bool,
    pub delete_project: bool,
}

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
        platform_type: String,
        oauth_access_token: Option<String>,
        oauth_refresh_token: Option<String>,
        oauth_token_expiry: Option<i64>,
        connected_email: Option<String>,
        selected_project_id: Option<String>,
    },
    DeletePlatform {
        platform_name: String,
        delete_options: DeleteOptions,
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
    ScanExistingVMs {
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

    // Refresh Operation
    RefreshPlatform {
        platform_name: String,
    },

    // Refresh all platform data
    RefreshAll,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_options_struct() {
        let opts = DeleteOptions {
            delete_vms: true,
            delete_project: false,
        };
        assert!(opts.delete_vms);
        assert!(!opts.delete_project);
    }

    #[test]
    fn test_delete_platform_with_options() {
        let cmd = PlatformCommand::DeletePlatform {
            platform_name: "test-platform".to_string(),
            delete_options: DeleteOptions {
                delete_vms: true,
                delete_project: true,
            },
        };
        match cmd {
            PlatformCommand::DeletePlatform {
                platform_name,
                delete_options,
            } => {
                assert_eq!(platform_name, "test-platform");
                assert!(delete_options.delete_vms);
                assert!(delete_options.delete_project);
            }
            _ => panic!("Expected DeletePlatform command"),
        }
    }
}
