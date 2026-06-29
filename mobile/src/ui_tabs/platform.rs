//! Platform tab - Platform configuration and management with GCP integration

use eframe::egui;
use egui_material3::MaterialButton;

use crate::config::{AppConfig, CloudPlatformConfig, VmInstance};

/// Platform tab state (placeholder for Phase 1)
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PlatformTab {
    // Will be fully implemented in later tasks
}

impl Default for PlatformTab {
    fn default() -> Self {
        Self {}
    }
}

impl PlatformTab {
    pub fn ui(&mut self, _ui: &mut egui::Ui) {
        // Will be implemented in Task 9
    }
}

/// Platform table row types
#[derive(Debug, Clone)]
enum PlatformRow {
    Account {
        platform_name: String,
        email: String,
        project_count: usize,
        vm_count: usize,
    },
    Project {
        platform_name: String,
        project_id: String,
        vm_count: usize,
        current_ip: Option<String>,
        firewall_whitelisted: bool,
    },
    Vm {
        platform_name: String,
        project_id: String,
        vm_name: String,
        zone: String,
        instance_id: String,
        external_ip: Option<String>,
        ssh_status: SshStatus,
    },
}

/// SSH connection status
#[derive(Debug, Clone)]
enum SshStatus {
    Testing,
    Available,
    Failed(String),
}

/// Build table rows from platform configurations
fn build_platform_rows(platforms: &[CloudPlatformConfig]) -> Vec<PlatformRow> {
    let mut rows = Vec::new();

    for platform in platforms {
        // Only process GCP platforms for now
        if platform.platform_type != "gcp" {
            continue;
        }

        let email = platform.gcp_connected_email.clone()
            .unwrap_or_else(|| "Not connected".to_string());

        // Account row
        rows.push(PlatformRow::Account {
            platform_name: platform.name.clone(),
            email,
            project_count: 0, // Will be fetched from API
            vm_count: platform.vms.len(),
        });

        // Project row (if project selected)
        if let Some(project_id) = &platform.gcp_selected_project_id {
            rows.push(PlatformRow::Project {
                platform_name: platform.name.clone(),
                project_id: project_id.clone(),
                vm_count: platform.vms.len(),
                current_ip: None, // Will be fetched from icanhazip.com
                firewall_whitelisted: false, // Will be checked via API
            });

            // VM row (show first VM only)
            if let Some(vm) = platform.vms.first() {
                rows.push(PlatformRow::Vm {
                    platform_name: platform.name.clone(),
                    project_id: project_id.clone(),
                    vm_name: vm.name.clone(),
                    zone: vm.zone.clone(),
                    instance_id: vm.instance_id.clone(),
                    external_ip: vm.external_ip.clone(),
                    ssh_status: SshStatus::Testing, // Will be tested in background
                });
            }
        }
    }

    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_platform_rows_empty() {
        let platforms = vec![];
        let rows = build_platform_rows(&platforms);
        assert_eq!(rows.len(), 0);
    }

    #[test]
    fn test_build_platform_rows_single_platform() {
        let platform = CloudPlatformConfig {
            name: "test-gcp".to_string(),
            platform_type: "gcp".to_string(),
            gcp_connected_email: Some("test@gmail.com".to_string()),
            gcp_selected_project_id: Some("dure".to_string()),
            vms: vec![VmInstance {
                name: "test-vm".to_string(),
                instance_id: "123".to_string(),
                zone: "us-central1-a".to_string(),
                gcp_region: "us-central1".to_string(),
                gcp_project_id: "dure".to_string(),
                machine_type: "e2-micro".to_string(),
                status: "RUNNING".to_string(),
                external_ip: Some("1.2.3.4".to_string()),
                internal_ip: None,
                gcp_billing_account: None,
                created_at: 0,
                ssh_key_name: Some("gcp.test.vm".to_string()),
            }],
            ..Default::default()
        };

        let rows = build_platform_rows(&vec![platform]);

        // Should have 3 rows: Account, Project, VM
        assert_eq!(rows.len(), 3);

        match &rows[0] {
            PlatformRow::Account { platform_name, email, vm_count, .. } => {
                assert_eq!(platform_name, "test-gcp");
                assert_eq!(email, "test@gmail.com");
                assert_eq!(vm_count, &1);
            }
            _ => panic!("First row should be Account"),
        }

        match &rows[1] {
            PlatformRow::Project { project_id, vm_count, .. } => {
                assert_eq!(project_id, "dure");
                assert_eq!(vm_count, &1);
            }
            _ => panic!("Second row should be Project"),
        }

        match &rows[2] {
            PlatformRow::Vm { vm_name, zone, .. } => {
                assert_eq!(vm_name, "test-vm");
                assert_eq!(zone, "us-central1-a");
            }
            _ => panic!("Third row should be Vm"),
        }
    }
}
