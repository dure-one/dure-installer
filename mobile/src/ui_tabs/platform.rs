//! Platform tab - Platform configuration and management with GCP integration

use eframe::egui;
use egui_material3::MaterialButton;

use crate::config::{AppConfig, CloudPlatformConfig, VmInstance};

/// Platform tab state
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PlatformTab {
    #[cfg_attr(feature = "serde", serde(skip))]
    rows: Vec<PlatformRow>,
    #[cfg_attr(feature = "serde", serde(skip))]
    loaded: bool,
}

impl Default for PlatformTab {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            loaded: false,
        }
    }
}

impl PlatformTab {
    /// Render the platform tab UI
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Cloud Platforms");
        ui.add_space(4.0);
        ui.label("Manage GCP platforms and VMs with inline actions.");
        ui.add_space(8.0);

        // Action buttons
        ui.horizontal(|ui| {
            if ui.add(MaterialButton::filled("Add Platform")).clicked() {
                // TODO: Show add platform dialog
            }

            if ui.add(MaterialButton::outlined("Refresh Status")).clicked() {
                // TODO: Trigger refresh
                self.loaded = false; // Force reload
            }
        });

        ui.add_space(8.0);

        // Load platforms if not loaded
        if !self.loaded {
            if let Ok(config) = load_config() {
                self.rows = build_platform_rows(&config.platforms);
                self.loaded = true;
            }
        }

        // Render table
        egui::ScrollArea::vertical()
            .max_height(600.0)
            .show(ui, |ui| {
                render_table(ui, &self.rows);
            });
    }
}

/// Load application config
#[cfg(not(target_arch = "wasm32"))]
fn load_config() -> Result<AppConfig, String> {
    use directories::ProjectDirs;

    let proj_dirs = ProjectDirs::from("pe", "nikescar", "dure")
        .ok_or_else(|| "Failed to get project directories".to_string())?;
    let config_path = proj_dirs.config_dir().join("config.yml");

    Ok(AppConfig::load_or_default(&config_path))
}

#[cfg(target_arch = "wasm32")]
fn load_config() -> Result<AppConfig, String> {
    // WASM not supported for this feature
    Err("Platform tab not available on WASM".to_string())
}

/// Render the platform table
fn render_table(ui: &mut egui::Ui, rows: &[PlatformRow]) {
    use egui::{Grid, RichText};

    // Table header
    Grid::new("platform_table_header")
        .num_columns(3)
        .striped(false)
        .show(ui, |ui| {
            ui.label(RichText::new("Platform Name").strong());
            ui.label(RichText::new("Status").strong());
            ui.label(RichText::new("Actions").strong());
            ui.end_row();
        });

    ui.separator();

    // Table rows
    Grid::new("platform_table_body")
        .num_columns(3)
        .striped(true)
        .show(ui, |ui| {
            for row in rows {
                render_row(ui, row);
            }
        });
}

/// Render a single table row
fn render_row(ui: &mut egui::Ui, row: &PlatformRow) {
    match row {
        PlatformRow::Account { platform_name, email, project_count, vm_count } => {
            ui.label(format!("GCP: {}", email));
            ui.label(format!("{} Projects", project_count));
            ui.label(""); // No actions for account row
            ui.end_row();
        }

        PlatformRow::Project { project_id, vm_count, current_ip, firewall_whitelisted, .. } => {
            ui.label(format!("  ├─ {}", project_id));

            let firewall_text = if *firewall_whitelisted {
                format!("{} VM\n✓ GCP Firewall Whitelisted({})",
                    vm_count,
                    current_ip.as_deref().unwrap_or("unknown"))
            } else {
                format!("{} VM\n✗ GCP Firewall Not Whitelisted", vm_count)
            };
            ui.label(firewall_text);

            if ui.add(MaterialButton::outlined("Update Firewall")).clicked() {
                // TODO: Show update firewall confirmation
            }
            ui.end_row();
        }

        PlatformRow::Vm { vm_name, ssh_status, .. } => {
            ui.label(format!("  └─── {}", vm_name));

            let ssh_text = match ssh_status {
                SshStatus::Testing => "🔄 SSH Connection Testing...".to_string(),
                SshStatus::Available => "✓ SSH Connection OK(:22)".to_string(),
                SshStatus::Failed(err) => format!("✗ SSH Connection Failed(:22) - {}", err),
            };
            ui.label(ssh_text);

            ui.horizontal(|ui| {
                if ui.add(MaterialButton::outlined("Delete VM")).clicked() {
                    // TODO: Show delete confirmation
                }
                if ui.add(MaterialButton::outlined("Regenerate VM")).clicked() {
                    // TODO: Show regenerate confirmation
                }
                if ui.add(MaterialButton::outlined("Restart VM")).clicked() {
                    // TODO: Show restart confirmation
                }
                if ui.add(MaterialButton::outlined("Refresh")).clicked() {
                    // TODO: Trigger refresh
                }
            });
            ui.end_row();
        }
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
