//! Platform tab - Platform configuration and management with GCP integration

use eframe::egui;
use egui_material3::MaterialButton;
use poll_promise::Promise;
use std::collections::HashMap;

use crate::config::{AppConfig, CloudPlatformConfig, VmInstance, SshHostConfig};
use crate::calc::ssh::{test_connection, SshConnectionResult};
use crate::calc::gcp_rest::{get_current_ip, GcpRestClient};

/// Platform tab state
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PlatformTab {
    #[cfg_attr(feature = "serde", serde(skip))]
    rows: Vec<PlatformRow>,
    #[cfg_attr(feature = "serde", serde(skip))]
    loaded: bool,

    // Background SSH tests: key = "{platform_name}:{vm_name}"
    #[cfg_attr(feature = "serde", serde(skip))]
    ssh_test_tasks: HashMap<String, Promise<Result<SshConnectionResult, String>>>,

    // Background data fetching
    #[cfg_attr(feature = "serde", serde(skip))]
    current_ip_task: Option<Promise<Result<String, String>>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    current_ip: Option<String>,

    // Firewall status checks: key = "{platform_name}:{project_id}"
    #[cfg_attr(feature = "serde", serde(skip))]
    firewall_check_tasks: HashMap<String, Promise<Result<bool, String>>>,

    // Project count fetch: key = platform_name
    #[cfg_attr(feature = "serde", serde(skip))]
    project_count_tasks: HashMap<String, Promise<Result<usize, String>>>,

    // Confirmation dialog state
    #[cfg_attr(feature = "serde", serde(skip))]
    show_update_firewall_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    firewall_project_id: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    firewall_confirmation_text: String,

    // Delete VM dialog state
    #[cfg_attr(feature = "serde", serde(skip))]
    show_delete_vm_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_vm_platform_name: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_vm_name: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_vm_confirmation_text: String,

    // Restart VM dialog state
    #[cfg_attr(feature = "serde", serde(skip))]
    show_restart_vm_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    restart_vm_platform_name: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    restart_vm_name: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    restart_vm_confirmation_text: String,

    // Regenerate VM dialog state
    #[cfg_attr(feature = "serde", serde(skip))]
    show_regenerate_vm_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    regenerate_vm_platform_name: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    regenerate_vm_confirmation_text: String,
}

impl Default for PlatformTab {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            loaded: false,
            ssh_test_tasks: HashMap::new(),
            current_ip_task: None,
            current_ip: None,
            firewall_check_tasks: HashMap::new(),
            project_count_tasks: HashMap::new(),
            show_update_firewall_dialog: false,
            firewall_project_id: String::new(),
            firewall_confirmation_text: String::new(),
            show_delete_vm_dialog: false,
            delete_vm_platform_name: String::new(),
            delete_vm_name: String::new(),
            delete_vm_confirmation_text: String::new(),
            show_restart_vm_dialog: false,
            restart_vm_platform_name: String::new(),
            restart_vm_name: String::new(),
            restart_vm_confirmation_text: String::new(),
            show_regenerate_vm_dialog: false,
            regenerate_vm_platform_name: String::new(),
            regenerate_vm_confirmation_text: String::new(),
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
                // Force reload and clear all background tasks
                self.loaded = false;
                self.current_ip_task = None;
                self.current_ip = None;
                self.firewall_check_tasks.clear();
                self.project_count_tasks.clear();
                self.ssh_test_tasks.clear();
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

        // Spawn current IP fetch task if not already running
        if self.current_ip_task.is_none() && self.current_ip.is_none() {
            self.current_ip_task = Some(Promise::spawn_thread("fetch_ip", || {
                get_current_ip().map_err(|e| e.to_string())
            }));
        }

        // Check if IP fetch completed
        if let Some(task) = &self.current_ip_task {
            if let Some(result) = task.ready() {
                if let Ok(ip) = result {
                    self.current_ip = Some(ip.clone());

                    // Update all Project rows with the new IP
                    for row in &mut self.rows {
                        if let PlatformRow::Project { current_ip, .. } = row {
                            *current_ip = Some(ip.clone());
                        }
                    }
                }
                self.current_ip_task = None;
            }
        }

        // Spawn firewall check tasks for projects (only if we have IP)
        if let Some(ip) = &self.current_ip {
            for row in &self.rows {
                if let PlatformRow::Project { platform_name, project_id, .. } = row {
                    let key = format!("{}:{}", platform_name, project_id);

                    if !self.firewall_check_tasks.contains_key(&key) {
                        // Get OAuth token from config
                        if let Ok(config) = load_config() {
                            if let Some(platform) = config.platforms.iter()
                                .find(|p| &p.name == platform_name)
                            {
                                if let Some(access_token) = &platform.gcp_oauth_access_token {
                                    let client = GcpRestClient::new(access_token.clone());
                                    let project_id = project_id.clone();
                                    let ip = ip.clone();

                                    let task = Promise::spawn_thread("firewall_check", move || {
                                        client.check_ip_whitelisted(&project_id, &ip)
                                            .map_err(|e| e.to_string())
                                    });

                                    self.firewall_check_tasks.insert(key, task);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check completed firewall check tasks
        let mut completed_firewall_tasks = Vec::new();

        for (key, task) in &self.firewall_check_tasks {
            if let Some(result) = task.ready() {
                // Update the Project row firewall status
                for row in &mut self.rows {
                    if let PlatformRow::Project { platform_name, project_id, firewall_whitelisted, .. } = row {
                        let row_key = format!("{}:{}", platform_name, project_id);
                        if row_key == *key {
                            if let Ok(whitelisted) = result {
                                *firewall_whitelisted = *whitelisted;
                            }
                        }
                    }
                }
                completed_firewall_tasks.push(key.clone());
            }
        }

        // Remove completed tasks
        for key in completed_firewall_tasks {
            self.firewall_check_tasks.remove(&key);
        }

        // Spawn project count fetch tasks for Account rows
        for row in &self.rows {
            if let PlatformRow::Account { platform_name, .. } = row {
                if !self.project_count_tasks.contains_key(platform_name) {
                    // Get OAuth token from config
                    if let Ok(config) = load_config() {
                        if let Some(platform) = config.platforms.iter()
                            .find(|p| &p.name == platform_name)
                        {
                            if let Some(access_token) = &platform.gcp_oauth_access_token {
                                let client = GcpRestClient::new(access_token.clone());
                                let platform_name = platform_name.clone();

                                let task = Promise::spawn_thread("project_count", move || {
                                    client.list_projects(None)
                                        .map(|list| list.projects.len())
                                        .map_err(|e| e.to_string())
                                });

                                self.project_count_tasks.insert(platform_name, task);
                            }
                        }
                    }
                }
            }
        }

        // Check completed project count tasks
        let mut completed_project_tasks = Vec::new();

        for (platform_name, task) in &self.project_count_tasks {
            if let Some(result) = task.ready() {
                // Update the Account row project count
                for row in &mut self.rows {
                    if let PlatformRow::Account { platform_name: row_platform, project_count, .. } = row {
                        if row_platform == platform_name {
                            if let Ok(count) = result {
                                *project_count = *count;
                            }
                        }
                    }
                }
                completed_project_tasks.push(platform_name.clone());
            }
        }

        // Remove completed tasks
        for key in completed_project_tasks {
            self.project_count_tasks.remove(&key);
        }

        // Spawn SSH tests for VMs that don't have active tasks
        for row in &self.rows {
            if let PlatformRow::Vm { platform_name, vm_name, .. } = row {
                let key = format!("{}:{}", platform_name, vm_name);

                if !self.ssh_test_tasks.contains_key(&key) {
                    // Find the VM in config to spawn test
                    if let Ok(config) = load_config() {
                        for platform in &config.platforms {
                            if &platform.name == platform_name {
                                if let Some(vm) = platform.vms.iter()
                                    .find(|v| &v.name == vm_name)
                                {
                                    let task = spawn_ssh_test(vm);
                                    self.ssh_test_tasks.insert(key.clone(), task);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check completed SSH tasks and update row status
        let mut completed_tasks = Vec::new();

        for (key, task) in &self.ssh_test_tasks {
            if let Some(result) = task.ready() {
                // Find the VM row and update its status
                for row in &mut self.rows {
                    if let PlatformRow::Vm { platform_name, vm_name, ssh_status, .. } = row {
                        let row_key = format!("{}:{}", platform_name, vm_name);
                        if row_key == *key {
                            *ssh_status = match result {
                                Ok(conn_result) if conn_result.success => SshStatus::Available,
                                Ok(_) => SshStatus::Failed("Auth failed".to_string()),
                                Err(e) => SshStatus::Failed(e.clone()),
                            };
                        }
                    }
                }
                completed_tasks.push(key.clone());
            }
        }

        // Remove completed tasks
        for key in completed_tasks {
            self.ssh_test_tasks.remove(&key);
        }

        // Render Update Firewall confirmation dialog
        if let Some(()) = render_update_firewall_dialog(
            ui.ctx(),
            &mut self.show_update_firewall_dialog,
            &self.firewall_project_id,
            &mut self.firewall_confirmation_text,
        ) {
            // User confirmed - execute firewall update
            if let Ok(ip) = get_current_ip() {
                // Get OAuth token from config
                if let Ok(config) = load_config() {
                    // Find the platform with this project
                    let platform = config.platforms.iter()
                        .find(|p| p.gcp_selected_project_id.as_ref() == Some(&self.firewall_project_id));

                    if let Some(platform) = platform {
                        if let Some(access_token) = &platform.gcp_oauth_access_token {
                            // Call GCP API to update firewall
                            let client = GcpRestClient::new(access_token.clone());
                            match client.add_ip_to_firewall(&self.firewall_project_id, &ip) {
                                Ok(_) => {
                                    println!("✓ Firewall updated for {} with IP {}", self.firewall_project_id, ip);
                                    // Force reload to show updated status
                                    self.loaded = false;
                                }
                                Err(e) => {
                                    eprintln!("✗ Failed to update firewall: {}", e);
                                }
                            }
                        } else {
                            eprintln!("✗ No OAuth token found for platform");
                        }
                    } else {
                        eprintln!("✗ Platform not found for project {}", self.firewall_project_id);
                    }
                }
            }
        }

        // Render Delete VM confirmation dialog
        if let Some(()) = render_delete_vm_dialog(
            ui.ctx(),
            &mut self.show_delete_vm_dialog,
            &self.delete_vm_name,
            &mut self.delete_vm_confirmation_text,
        ) {
            // User confirmed - execute VM deletion
            if let Ok(config) = load_config() {
                // Find the platform and VM
                let platform = config.platforms.iter()
                    .find(|p| p.name == self.delete_vm_platform_name);

                if let Some(platform) = platform {
                    if let Some(access_token) = &platform.gcp_oauth_access_token {
                        let vm = platform.vms.iter()
                            .find(|v| v.name == self.delete_vm_name);

                        if let Some(vm) = vm {
                            // Call delete_vm function
                            let client = GcpRestClient::new(access_token.clone());
                            match crate::calc::hosting_gcp::delete_vm(&client, vm) {
                                Ok(msg) => {
                                    println!("✓ {}", msg);
                                    // Force reload to update table
                                    self.loaded = false;
                                }
                                Err(e) => {
                                    eprintln!("✗ Failed to delete VM: {}", e);
                                }
                            }
                        } else {
                            eprintln!("✗ VM not found: {}", self.delete_vm_name);
                        }
                    } else {
                        eprintln!("✗ No OAuth token found for platform");
                    }
                } else {
                    eprintln!("✗ Platform not found: {}", self.delete_vm_platform_name);
                }
            }
        }

        // Render Restart VM confirmation dialog
        if let Some(()) = render_restart_vm_dialog(
            ui.ctx(),
            &mut self.show_restart_vm_dialog,
            &self.restart_vm_name,
            &mut self.restart_vm_confirmation_text,
        ) {
            // User confirmed - execute VM restart
            if let Ok(config) = load_config() {
                // Find the platform and VM
                let platform = config.platforms.iter()
                    .find(|p| p.name == self.restart_vm_platform_name);

                if let Some(platform) = platform {
                    if let Some(access_token) = &platform.gcp_oauth_access_token {
                        let vm = platform.vms.iter()
                            .find(|v| v.name == self.restart_vm_name);

                        if let Some(vm) = vm {
                            // Call restart_vm function
                            let client = GcpRestClient::new(access_token.clone());
                            match crate::calc::hosting_gcp::restart_vm(&client, vm) {
                                Ok(msg) => {
                                    println!("✓ {}", msg);
                                    // Force reload to update SSH status
                                    self.loaded = false;
                                }
                                Err(e) => {
                                    eprintln!("✗ Failed to restart VM: {}", e);
                                }
                            }
                        } else {
                            eprintln!("✗ VM not found: {}", self.restart_vm_name);
                        }
                    } else {
                        eprintln!("✗ No OAuth token found for platform");
                    }
                } else {
                    eprintln!("✗ Platform not found: {}", self.restart_vm_platform_name);
                }
            }
        }

        // Render Regenerate VM confirmation dialog
        if let Some(()) = render_regenerate_vm_dialog(
            ui.ctx(),
            &mut self.show_regenerate_vm_dialog,
            &self.regenerate_vm_platform_name,
            &mut self.regenerate_vm_confirmation_text,
        ) {
            // User confirmed - execute VM regeneration
            if let Ok(mut config) = load_config() {
                // Find the mutable platform
                if let Some(platform) = config.platforms.iter_mut()
                    .find(|p| p.name == self.regenerate_vm_platform_name)
                {
                    if let Some(access_token) = &platform.gcp_oauth_access_token {
                        if platform.gcp_selected_project_id.is_some() {
                            // Determine zone from first VM or use default
                            let zone = platform.vms.first()
                                .map(|vm| vm.zone.clone())
                                .unwrap_or_else(|| "us-central1-a".to_string());

                            // Call regenerate_vm function
                            let client = GcpRestClient::new(access_token.clone());
                            match crate::calc::hosting_gcp::regenerate_vm(&client, platform, &zone) {
                                Ok(msg) => {
                                    println!("✓ {}", msg);
                                    // Save updated config
                                    if let Ok(config_path) = get_config_path() {
                                        let _ = config.save(&config_path);
                                    }
                                    // Force reload to update table
                                    self.loaded = false;
                                }
                                Err(e) => {
                                    eprintln!("✗ Failed to regenerate VM: {}", e);
                                }
                            }
                        } else {
                            eprintln!("✗ No project selected for platform");
                        }
                    } else {
                        eprintln!("✗ No OAuth token found for platform");
                    }
                } else {
                    eprintln!("✗ Platform not found: {}", self.regenerate_vm_platform_name);
                }
            }
        }

        // Render table
        egui::ScrollArea::vertical()
            .max_height(600.0)
            .show(ui, |ui_inner| {
                // Need to split borrow to pass both rows and self
                let rows_clone = self.rows.clone();
                render_table(ui_inner, &rows_clone, self);
            });
    }
}

/// Get config file path
#[cfg(not(target_arch = "wasm32"))]
fn get_config_path() -> Result<std::path::PathBuf, String> {
    use directories::ProjectDirs;

    let proj_dirs = ProjectDirs::from("pe", "nikescar", "dure")
        .ok_or_else(|| "Failed to get project directories".to_string())?;
    Ok(proj_dirs.config_dir().join("config.yml"))
}

/// Load application config
#[cfg(not(target_arch = "wasm32"))]
fn load_config() -> Result<AppConfig, String> {
    let config_path = get_config_path()?;
    Ok(AppConfig::load_or_default(&config_path))
}

#[cfg(target_arch = "wasm32")]
fn load_config() -> Result<AppConfig, String> {
    // WASM not supported for this feature
    Err("Platform tab not available on WASM".to_string())
}

/// Spawn SSH test for a VM
fn spawn_ssh_test(vm: &VmInstance) -> Promise<Result<SshConnectionResult, String>> {
    let vm = vm.clone();

    Promise::spawn_thread("ssh_test", move || {
        // Build SSH config from VM
        let external_ip = vm.external_ip
            .ok_or_else(|| "No external IP".to_string())?;

        let ssh_config = SshHostConfig {
            host: format!("generated_user@{}", external_ip),
            port: 22,
            password: None,
            private_key_path: None,
            keyring_domain: vm.ssh_key_name.clone(),
            initialized: false,
            last_status: None,
        };

        // Test connection
        test_connection(&ssh_config)
            .map_err(|e| format!("Timeout: {}", e))
    })
}

/// Render regenerate VM confirmation dialog
fn render_regenerate_vm_dialog(
    ctx: &egui::Context,
    show: &mut bool,
    platform_name: &str,
    confirmation_text: &mut String,
) -> Option<()> {
    if !*show {
        return None;
    }

    let mut confirmed = false;

    egui::Window::new("Regenerate VM")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("⚠️  This will DELETE ALL VMs in this project and create a fresh one.");
            ui.label("All data on existing VMs will be permanently lost.");
            ui.add_space(8.0);

            ui.label(format!("Platform: {}", platform_name));

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Type 'regenerate' to confirm:");
                ui.text_edit_singleline(confirmation_text);
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    *show = false;
                }

                ui.add_enabled_ui(confirmation_text == "regenerate", |ui| {
                    if ui.button("Confirm Regenerate").clicked() {
                        confirmed = true;
                        *show = false;
                    }
                });
            });
        });

    if confirmed {
        Some(())
    } else {
        None
    }
}

/// Render restart VM confirmation dialog
fn render_restart_vm_dialog(
    ctx: &egui::Context,
    show: &mut bool,
    vm_name: &str,
    confirmation_text: &mut String,
) -> Option<()> {
    if !*show {
        return None;
    }

    let mut confirmed = false;

    egui::Window::new("Restart VM")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("This will reset (hard reboot) the VM instance.");
            ui.label("Any unsaved data will be lost.");
            ui.add_space(8.0);

            ui.label(format!("VM: {}", vm_name));

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Type 'restart' to confirm:");
                ui.text_edit_singleline(confirmation_text);
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    *show = false;
                }

                ui.add_enabled_ui(confirmation_text == "restart", |ui| {
                    if ui.button("Confirm Restart").clicked() {
                        confirmed = true;
                        *show = false;
                    }
                });
            });
        });

    if confirmed {
        Some(())
    } else {
        None
    }
}

/// Render delete VM confirmation dialog
fn render_delete_vm_dialog(
    ctx: &egui::Context,
    show: &mut bool,
    vm_name: &str,
    confirmation_text: &mut String,
) -> Option<()> {
    if !*show {
        return None;
    }

    let mut confirmed = false;

    egui::Window::new("Delete VM")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("⚠️  This will permanently delete the VM instance.");
            ui.label("All data on the VM will be lost.");
            ui.add_space(8.0);

            ui.label(format!("VM: {}", vm_name));

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Type 'delete' to confirm:");
                ui.text_edit_singleline(confirmation_text);
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    *show = false;
                }

                ui.add_enabled_ui(confirmation_text == "delete", |ui| {
                    if ui.button("Confirm Delete").clicked() {
                        confirmed = true;
                        *show = false;
                    }
                });
            });
        });

    if confirmed {
        Some(())
    } else {
        None
    }
}

/// Render update firewall confirmation dialog
fn render_update_firewall_dialog(
    ctx: &egui::Context,
    show: &mut bool,
    project_id: &str,
    confirmation_text: &mut String,
) -> Option<()> {
    if !*show {
        return None;
    }

    let mut confirmed = false;

    egui::Window::new("Update GCP Firewall")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("This will add your current IP to the GCP firewall");
            ui.label("whitelist for SSH access (port 22).");
            ui.add_space(8.0);

            ui.label(format!("Project: {}", project_id));

            if let Ok(ip) = get_current_ip() {
                ui.label(format!("Current IP: {}", ip));
            }

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Type 'update' to confirm:");
                ui.text_edit_singleline(confirmation_text);
            });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    *show = false;
                }

                ui.add_enabled_ui(confirmation_text == "update", |ui| {
                    if ui.button("Confirm").clicked() {
                        confirmed = true;
                        *show = false;
                    }
                });
            });
        });

    if confirmed {
        Some(())
    } else {
        None
    }
}

/// Render the platform table
fn render_table(ui: &mut egui::Ui, rows: &[PlatformRow], platform_tab: &mut PlatformTab) {
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
                render_row(ui, row, platform_tab);
            }
        });
}

/// Render a single table row
fn render_row(ui: &mut egui::Ui, row: &PlatformRow, platform_tab: &mut PlatformTab) {
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
                platform_tab.show_update_firewall_dialog = true;
                platform_tab.firewall_project_id = project_id.clone();
                platform_tab.firewall_confirmation_text.clear();
            }
            ui.end_row();
        }

        PlatformRow::Vm { platform_name, vm_name, ssh_status, .. } => {
            ui.label(format!("  └─── {}", vm_name));

            let ssh_text = match ssh_status {
                SshStatus::Testing => "🔄 SSH Connection Testing...".to_string(),
                SshStatus::Available => "✓ SSH Connection OK(:22)".to_string(),
                SshStatus::Failed(err) => format!("✗ SSH Connection Failed(:22) - {}", err),
            };
            ui.label(ssh_text);

            ui.horizontal(|ui| {
                if ui.add(MaterialButton::outlined("Delete VM")).clicked() {
                    platform_tab.show_delete_vm_dialog = true;
                    platform_tab.delete_vm_platform_name = platform_name.clone();
                    platform_tab.delete_vm_name = vm_name.clone();
                    platform_tab.delete_vm_confirmation_text.clear();
                }
                if ui.add(MaterialButton::outlined("Regenerate VM")).clicked() {
                    platform_tab.show_regenerate_vm_dialog = true;
                    platform_tab.regenerate_vm_platform_name = platform_name.clone();
                    platform_tab.regenerate_vm_confirmation_text.clear();
                }
                if ui.add(MaterialButton::outlined("Restart VM")).clicked() {
                    platform_tab.show_restart_vm_dialog = true;
                    platform_tab.restart_vm_platform_name = platform_name.clone();
                    platform_tab.restart_vm_name = vm_name.clone();
                    platform_tab.restart_vm_confirmation_text.clear();
                }
                if ui.add(MaterialButton::outlined("Refresh")).clicked() {
                    // Force reload of platform data and re-trigger all background tasks
                    platform_tab.loaded = false;
                    platform_tab.current_ip_task = None;
                    platform_tab.current_ip = None;
                    platform_tab.firewall_check_tasks.clear();
                    platform_tab.project_count_tasks.clear();
                    platform_tab.ssh_test_tasks.clear();
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
