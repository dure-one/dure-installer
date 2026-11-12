//! Platform actor commands

#[derive(Debug, Clone)]
pub struct DeleteOptions {
    pub delete_vms: bool,
    pub delete_project: bool,
}

/// Firewall rule data for create/update operations
#[derive(Debug, Clone)]
pub struct FirewallRuleData {
    pub name: String,
    pub network: String,
    pub direction: String,
    pub priority: u32,
    pub source_ranges: Vec<String>,
    pub dest_ranges: Vec<String>,
    pub allowed: Vec<String>,
    pub denied: Vec<String>,
    pub source_tags: Vec<String>,
    pub source_service_accounts: Vec<String>,
    pub target_tags: Vec<String>,
    pub target_service_accounts: Vec<String>,
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
        profile_config_path: std::path::PathBuf,
        platform_type: String,
        oauth_access_token: Option<String>,
        oauth_refresh_token: Option<String>,
        oauth_token_expiry: Option<i64>,
        connected_email: Option<String>,
        selected_project_id: Option<String>,
    },
    UpdatePlatformOAuth {
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        oauth_access_token: String,
        oauth_refresh_token: String,
        oauth_token_expiry: i64,
        connected_email: String,
    },
    DeletePlatform {
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        delete_options: DeleteOptions,
    },

    // Project Operations
    ListProjects {
        platform_name: String,
    },
    SelectProject {
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        project_id: String,
    },

    // VM Operations
    ListVMs {
        profile_config_path: std::path::PathBuf,
        platform_name: String,
    },
    ScanExistingVMs {
        profile_config_path: std::path::PathBuf,
        platform_name: String,
    },
    CreateVM {
        platform_name: String,
        vm_name: String,
        zone: String,
        machine_type: String,
    },
    DeleteVM {
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        vm_name: String,
        zone: String,
        force: bool,
        /// If Some((region, address_name)), release this reserved static IP
        /// after VM deletion completes on GCP.
        release_ip: Option<(String, String)>,
    },
    RestartVM {
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        vm_name: String,
        zone: String,
    },
    RegenerateVM {
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        vm_name: String,
        zone: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    },

    // Firewall Operations
    UpdateFirewall {
        platform_name: String,
        allow_ip: String,
    },
    FetchFirewallRules {
        profile_config_path: std::path::PathBuf,
        platform_name: String,
    },
    UpdateFirewallRule {
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        rule_name: String,
        add_ip: String,
    },
    RemoveIpFromFirewall {
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        rule_name: String,
        remove_ips: Vec<String>,
    },
    FlushFirewallRule {
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        rule_name: String,
    },
    CreateFirewallRule {
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        rule_data: FirewallRuleData,
    },
    DeleteFirewallRule {
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        rule_name: String,
    },

    // Billing Operations
    FetchBilling {
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        project_id: String,
        dataset: String,
        table: String,
    },

    // Refresh Operation
    RefreshPlatform {
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
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
            profile_config_path: std::path::PathBuf::from("test.yml"),
            platform_name: "test-platform".to_string(),
            delete_options: DeleteOptions {
                delete_vms: true,
                delete_project: true,
            },
        };
        match cmd {
            PlatformCommand::DeletePlatform {
                profile_config_path: _,
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

    #[test]
    fn test_firewall_rule_data_struct() {
        let rule_data = FirewallRuleData {
            name: "test-firewall".to_string(),
            network: "default".to_string(),
            direction: "INGRESS".to_string(),
            priority: 1000,
            source_ranges: vec!["0.0.0.0/0".to_string()],
            dest_ranges: vec![],
            allowed: vec!["tcp:22".to_string()],
            denied: vec![],
            source_tags: vec![],
            source_service_accounts: vec![],
            target_tags: vec!["ssh-server".to_string()],
            target_service_accounts: vec![],
        };

        assert_eq!(rule_data.name, "test-firewall");
        assert_eq!(rule_data.network, "default");
        assert_eq!(rule_data.direction, "INGRESS");
        assert_eq!(rule_data.priority, 1000);
        assert_eq!(rule_data.source_ranges.len(), 1);
        assert_eq!(rule_data.allowed.len(), 1);
    }

    #[test]
    fn test_create_firewall_rule_command() {
        let rule_data = FirewallRuleData {
            name: "allow-ssh".to_string(),
            network: "default".to_string(),
            direction: "INGRESS".to_string(),
            priority: 1000,
            source_ranges: vec!["0.0.0.0/0".to_string()],
            dest_ranges: vec![],
            allowed: vec!["tcp:22".to_string()],
            denied: vec![],
            source_tags: vec![],
            source_service_accounts: vec![],
            target_tags: vec!["ssh-server".to_string()],
            target_service_accounts: vec![],
        };

        let cmd = PlatformCommand::CreateFirewallRule {
            profile_config_path: std::path::PathBuf::from("test.yml"),
            platform_name: "test-platform".to_string(),
            rule_data: rule_data.clone(),
        };

        match cmd {
            PlatformCommand::CreateFirewallRule {
                profile_config_path: _,
                platform_name,
                rule_data,
            } => {
                assert_eq!(platform_name, "test-platform");
                assert_eq!(rule_data.name, "allow-ssh");
                assert_eq!(rule_data.direction, "INGRESS");
            }
            _ => panic!("Expected CreateFirewallRule command"),
        }
    }

    #[test]
    fn test_delete_firewall_rule_command() {
        let cmd = PlatformCommand::DeleteFirewallRule {
            profile_config_path: std::path::PathBuf::from("test.yml"),
            platform_name: "test-platform".to_string(),
            rule_name: "allow-ssh".to_string(),
        };

        match cmd {
            PlatformCommand::DeleteFirewallRule {
                profile_config_path: _,
                platform_name,
                rule_name,
            } => {
                assert_eq!(platform_name, "test-platform");
                assert_eq!(rule_name, "allow-ssh");
            }
            _ => panic!("Expected DeleteFirewallRule command"),
        }
    }
}
