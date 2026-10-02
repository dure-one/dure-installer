//! SSH tab - SSH host configuration and management with drawer

use crate::{dure_debug, dure_error, dure_info};
use eframe::egui;
use egui_material3::MaterialButton;

use crate::config::{AppConfig, SshHostConfig};
use crate::viewmodel::ssh::{DrawerState, DrawerTab};
use crate::ui_components::drawer::{LogsRenderer, OperationsRenderer};

// Button spacing matching Platform tab
const BUTTON_VERTICAL_SPACING: f32 = 4.0;
const BUTTON_HORIZONTAL_SPACING: f32 = 4.0;

/// Operation state for visual feedback with timestamps
#[derive(Debug, Clone, PartialEq)]
pub enum OperationState {
    Idle,
    InProgress {
        operation: String,
        started_at: i64, // Unix timestamp
    },
    Completed {
        operation: String,
        completed_at: i64,
    },
    Failed {
        operation: String,
        error: String,
        failed_at: i64,
    },
}

impl OperationState {
    pub fn start(operation: impl Into<String>) -> Self {
        Self::InProgress {
            operation: operation.into(),
            started_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn complete(self) -> Self {
        if let Self::InProgress { operation, .. } = self {
            Self::Completed {
                operation,
                completed_at: chrono::Utc::now().timestamp(),
            }
        } else {
            self
        }
    }

    pub fn fail(self, error: impl Into<String>) -> Self {
        if let Self::InProgress { operation, .. } = self {
            Self::Failed {
                operation,
                error: error.into(),
                failed_at: chrono::Utc::now().timestamp(),
            }
        } else {
            self
        }
    }

    pub fn reset(&mut self) {
        *self = Self::Idle;
    }
}

impl Default for OperationState {
    fn default() -> Self {
        Self::Idle
    }
}

/// SSH connection check result
#[derive(Clone, Debug)]
pub struct SshCheckResult {
    pub connected: bool,
    pub error: Option<String>,
    pub checked_at: i64,
}

/// Base system check result
#[derive(Clone, Debug)]
pub struct BaseCheckResult {
    pub installed: bool,
    pub missing_packages: Vec<String>,
    pub checked_at: i64,
}

/// Docker installation check result
#[derive(Clone, Debug)]
pub struct DockerCheckResult {
    pub installed: bool,
    pub version: Option<String>,
    pub checked_at: i64,
}

/// Dure service check result
#[derive(Clone, Debug)]
pub struct DureCheckResult {
    pub installed: bool,
    pub running: bool,
    pub services: Vec<String>,
    pub checked_at: i64,
}

/// SSH row data for data table
#[derive(Clone, Debug)]
pub struct SshRow {
    // Identity
    pub host: String, // IP address (row key)
    pub port: u16,
    pub platform_id: Option<String>, // Connected platform ID (if applicable)

    // Connection state
    pub ssh_connected: bool,

    // Service flags
    pub base_installed: bool,
    pub docker_installed: bool,
    pub dure_installed: bool,

    // SSH key for copy action
    pub ssh_private_key: Option<String>,

    // Drawer state
    pub drawer_open: bool,
    pub drawer_state: DrawerState,

    // Operation state tracking
    pub operation_state: OperationState,

    // Last check results
    pub last_ssh_check: Option<SshCheckResult>,
    pub last_base_check: Option<BaseCheckResult>,
    pub last_docker_check: Option<DockerCheckResult>,
    pub last_dure_check: Option<DureCheckResult>,
}

/// Actions that can be triggered from SSH table rows
#[derive(Debug, Clone)]
enum SshAction {
    Refresh(String),       // host
    SshCheck(String),      // host
    Edit(String),          // host
    Delete(String),        // host
    CheckBase(String),     // host
    InstallBase(String),   // host
    CheckDocker(String),   // host
    InstallDocker(String), // host
    RemoveDocker(String),  // host
    CheckDure(String),     // host
    InstallDure(String),   // host (triggers .env dialog)
    RemoveDure(String),    // host
}

/// Render operations column with 3 rows of conditional buttons
fn render_operations_column(
    ui: &mut egui::Ui,
    row: &SshRow,
    action_trigger: &mut Option<SshAction>,
) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = BUTTON_VERTICAL_SPACING;
        ui.spacing_mut().item_spacing.x = BUTTON_HORIZONTAL_SPACING;

        // Row 1: Basic operations
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = BUTTON_HORIZONTAL_SPACING;

            if ui.add(MaterialButton::outlined("Refresh").small()).clicked() {
                *action_trigger = Some(SshAction::Refresh(row.host.clone()));
            }

            if ui.add(MaterialButton::outlined("SSH Check").small()).clicked() {
                *action_trigger = Some(SshAction::SshCheck(row.host.clone()));
            }

            if ui.add(MaterialButton::outlined("Edit").small()).clicked() {
                *action_trigger = Some(SshAction::Edit(row.host.clone()));
            }

            if ui.add(MaterialButton::outlined("Delete").small()).clicked() {
                *action_trigger = Some(SshAction::Delete(row.host.clone()));
            }
        });

        // Row 2: Base and Docker
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = BUTTON_HORIZONTAL_SPACING;

            // Check Base (always)
            if ui.add(MaterialButton::outlined("Check Base").small()).clicked() {
                *action_trigger = Some(SshAction::CheckBase(row.host.clone()));
            }

            // Install Base (only when NOT installed)
            if !row.base_installed {
                if ui.add(MaterialButton::outlined("Install Base").small()).clicked() {
                    *action_trigger = Some(SshAction::InstallBase(row.host.clone()));
                }
            }

            // Check Docker (always)
            if ui.add(MaterialButton::outlined("Check Docker").small()).clicked() {
                *action_trigger = Some(SshAction::CheckDocker(row.host.clone()));
            }

            // Install/Remove Docker (conditional)
            if !row.docker_installed {
                if ui.add(MaterialButton::outlined("Install Docker").small()).clicked() {
                    *action_trigger = Some(SshAction::InstallDocker(row.host.clone()));
                }
            } else {
                if ui.add(MaterialButton::outlined("Remove Docker").small()).clicked() {
                    *action_trigger = Some(SshAction::RemoveDocker(row.host.clone()));
                }
            }
        });

        // Row 3: Dure operations
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = BUTTON_HORIZONTAL_SPACING;

            // Check Dure (always)
            if ui.add(MaterialButton::outlined("Check Dure").small()).clicked() {
                *action_trigger = Some(SshAction::CheckDure(row.host.clone()));
            }

            // Install/Remove Dure (conditional)
            if !row.dure_installed {
                if ui.add(MaterialButton::outlined("Install Dure").small()).clicked() {
                    *action_trigger = Some(SshAction::InstallDure(row.host.clone()));
                }
            } else {
                if ui.add(MaterialButton::outlined("Remove Dure").small()).clicked() {
                    *action_trigger = Some(SshAction::RemoveDure(row.host.clone()));
                }
            }
        });
    });
}

/// Render host column with platform badge
fn render_host_column(ui: &mut egui::Ui, row: &SshRow) {
    use egui_material3::{badge, BadgeColor, BadgeSize};

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 2.0;

        // Line 1: Platform badge (if connected)
        if let Some(platform_id) = &row.platform_id {
            ui.add(
                badge(platform_id)
                    .color(BadgeColor::Primary)
                    .size(BadgeSize::Small)
            );
        }

        // Line 2: Host IP
        ui.label(&row.host);
    });
}

/// SSH tab state
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SshTab {
    /// SSH rows for data table
    #[cfg_attr(feature = "serde", serde(skip))]
    rows: Vec<SshRow>,

    #[cfg_attr(feature = "serde", serde(skip))]
    loaded: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    load_error: Option<String>,

    /// Config file last modified time (to detect changes)
    #[cfg_attr(feature = "serde", serde(skip))]
    config_last_modified: Option<std::time::SystemTime>,

    // Add host dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_add_dialog: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_host: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_password: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_private_key_path: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_port: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_use_password: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    add_use_private_key: bool,

    // Edit host dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_edit_dialog: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    edit_original_host: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    edit_host: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    edit_password: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    edit_private_key_path: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    edit_port: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    edit_use_password: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    edit_use_private_key: bool,
}

impl Default for SshTab {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            loaded: false,
            load_error: None,
            config_last_modified: None,
            show_add_dialog: false,
            add_host: String::new(),
            add_password: String::new(),
            add_private_key_path: String::new(),
            add_port: "22".to_string(),
            add_use_password: false,
            add_use_private_key: false,
            show_edit_dialog: false,
            edit_original_host: String::new(),
            edit_host: String::new(),
            edit_password: String::new(),
            edit_private_key_path: String::new(),
            edit_port: "22".to_string(),
            edit_use_password: false,
            edit_use_private_key: false,
        }
    }
}

/// Get config file path
#[cfg(not(target_arch = "wasm32"))]
fn get_config_path(
    profile: &Option<crate::calc::profile::ProfileContext>,
) -> Result<std::path::PathBuf, String> {
    match profile {
        Some(ctx) => Ok(ctx.config_file.clone()),
        None => Err("No active profile - please select or create a profile first".to_string()),
    }
}

/// Load application config
#[cfg(not(target_arch = "wasm32"))]
fn load_config(
    profile: &Option<crate::calc::profile::ProfileContext>,
) -> Result<(AppConfig, std::path::PathBuf), String> {
    let config_path = get_config_path(profile)?;
    let app_config = AppConfig::load_or_default(&config_path);
    Ok((app_config, config_path))
}

impl SshTab {
    /// Load SSH hosts from config and build row data
    fn load_rows(&mut self, profile: &Option<crate::calc::profile::ProfileContext>) {
        self.rows.clear();
        self.load_error = None;

        #[cfg(not(target_arch = "wasm32"))]
        {
            match load_config(profile) {
                Ok((app_config, config_path)) => {
                    // Cache config file metadata
                    if let Ok(metadata) = std::fs::metadata(&config_path) {
                        if let Ok(modified) = metadata.modified() {
                            self.config_last_modified = Some(modified);
                        }
                    }

                    for host_config in &app_config.ssh_hosts {
                        let mut drawer_state = DrawerState::new();
                        drawer_state.set_ssh_host(host_config.host.clone());

                        self.rows.push(SshRow {
                            host: host_config.host.clone(),
                            port: host_config.port,
                            platform_id: host_config.platform_name.clone(), // Platform connection (if any)
                            ssh_connected: false, // Updated via event handlers
                            base_installed: false, // Updated via event handlers
                            docker_installed: !host_config.docker_containers.is_empty(),
                            dure_installed: host_config.dure_wss_config.is_some(),
                            ssh_private_key: host_config.private_key_path.clone(),
                            drawer_open: false,
                            drawer_state,
                            operation_state: OperationState::Idle,
                            last_ssh_check: None,
                            last_base_check: None,
                            last_docker_check: None,
                            last_dure_check: None,
                        });
                    }
                    self.loaded = true;
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {}", e));
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.load_error = Some("SSH management not available on WASM".to_string());
            self.loaded = true;
        }
    }

    /// Reset loaded flag to force config reload on next render
    pub fn reset_loaded(&mut self) {
        self.loaded = false;
    }

    /// Check if config file has changed since last load
    #[cfg(not(target_arch = "wasm32"))]
    pub fn should_reload_config(&self, profile: &Option<crate::calc::profile::ProfileContext>) -> bool {
        // If never loaded, should reload
        if self.config_last_modified.is_none() {
            return true;
        }

        // Get current config file metadata
        if let Ok(config_path) = get_config_path(profile) {
            if let Ok(metadata) = std::fs::metadata(&config_path) {
                if let Ok(current_modified) = metadata.modified() {
                    // Compare with cached metadata
                    return Some(current_modified) != self.config_last_modified;
                }
            }
        }

        // If we can't get metadata, don't reload (ponytail: fail safe)
        false
    }

    #[cfg(target_arch = "wasm32")]
    pub fn should_reload_config(&self, _profile: &Option<crate::calc::profile::ProfileContext>) -> bool {
        // WASM doesn't have config files
        false
    }

    /// Handle ViewModel events to update UI state
    fn handle_event(
        &mut self,
        event: crate::viewmodel::ViewModelEvent,
        profile: &Option<crate::calc::profile::ProfileContext>,
    ) {
        use crate::viewmodel::ssh::SshEvent;
        use crate::viewmodel::ViewModelEvent;

        match event {
            ViewModelEvent::Ssh(SshEvent::HostAdded { name }) => {
                dure_info!("SSH host {} added", name);
                self.loaded = false; // Trigger reload
            }

            ViewModelEvent::Ssh(SshEvent::HostDeleted { name }) => {
                dure_info!("SSH host {} deleted", name);

                // Remove from config
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok((mut app_config, config_path)) = load_config(profile) {
                    app_config.ssh_hosts.retain(|h| h.host != name);
                    let _ = app_config.save(&config_path);
                }

                self.loaded = false; // Trigger reload
            }

            ViewModelEvent::Ssh(SshEvent::DockerStatusRetrieved { name, installed, .. }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.docker_installed = installed;
                }
            }

            ViewModelEvent::Ssh(SshEvent::RefreshCompleted { host, ssh_connected, base_installed, docker_installed, dure_installed }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.ssh_connected = ssh_connected;
                    row.base_installed = base_installed;
                    row.docker_installed = docker_installed;
                    row.dure_installed = dure_installed;
                    row.operation_state = OperationState::Completed {
                        operation: "Refresh".to_string(),
                        completed_at: chrono::Utc::now().timestamp(),
                    };
                }
                dure_info!("Refresh completed for host {}", host);
            }

            ViewModelEvent::Ssh(SshEvent::SshCheckCompleted { host, connected, error }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.ssh_connected = connected;
                    row.last_ssh_check = Some(SshCheckResult {
                        connected,
                        error: error.clone(),
                        checked_at: chrono::Utc::now().timestamp(),
                    });
                    row.operation_state = if error.is_none() {
                        OperationState::Completed {
                            operation: "SSH Check".to_string(),
                            completed_at: chrono::Utc::now().timestamp(),
                        }
                    } else {
                        OperationState::Failed {
                            operation: "SSH Check".to_string(),
                            error: error.unwrap(),
                            failed_at: chrono::Utc::now().timestamp(),
                        }
                    };
                }
            }

            ViewModelEvent::Ssh(SshEvent::BaseCheckCompleted { host, installed, missing_packages }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.base_installed = installed;
                    row.last_base_check = Some(BaseCheckResult {
                        installed,
                        missing_packages,
                        checked_at: chrono::Utc::now().timestamp(),
                    });
                    row.operation_state = OperationState::Completed {
                        operation: "Base Check".to_string(),
                        completed_at: chrono::Utc::now().timestamp(),
                    };
                }
            }

            ViewModelEvent::Ssh(SshEvent::BaseInstallCompleted { host, success, error }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.base_installed = success;
                    row.operation_state = if success {
                        OperationState::Completed {
                            operation: "Install Base".to_string(),
                            completed_at: chrono::Utc::now().timestamp(),
                        }
                    } else {
                        OperationState::Failed {
                            operation: "Install Base".to_string(),
                            error: error.unwrap_or_else(|| "Unknown error".to_string()),
                            failed_at: chrono::Utc::now().timestamp(),
                        }
                    };
                }
                if success {
                    dure_info!("Base packages installed successfully on {}", host);
                } else {
                    dure_error!("Base package installation failed on {}", host);
                }
            }

            ViewModelEvent::Ssh(SshEvent::DockerCheckCompleted { host, installed, version }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.docker_installed = installed;
                    row.last_docker_check = Some(DockerCheckResult {
                        installed,
                        version,
                        checked_at: chrono::Utc::now().timestamp(),
                    });
                    row.operation_state = OperationState::Completed {
                        operation: "Docker Check".to_string(),
                        completed_at: chrono::Utc::now().timestamp(),
                    };
                }
            }

            ViewModelEvent::Ssh(SshEvent::DockerInstallCompleted { host, success, error }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.docker_installed = success;
                    row.operation_state = if success {
                        OperationState::Completed {
                            operation: "Install Docker".to_string(),
                            completed_at: chrono::Utc::now().timestamp(),
                        }
                    } else {
                        OperationState::Failed {
                            operation: "Install Docker".to_string(),
                            error: error.unwrap_or_else(|| "Unknown error".to_string()),
                            failed_at: chrono::Utc::now().timestamp(),
                        }
                    };
                }
                if success {
                    dure_info!("Docker installed successfully on {}", host);
                } else {
                    dure_error!("Docker installation failed on {}", host);
                }
            }

            ViewModelEvent::Ssh(SshEvent::DockerRemoveCompleted { host, success, error }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.docker_installed = !success;
                    row.operation_state = if success {
                        OperationState::Completed {
                            operation: "Remove Docker".to_string(),
                            completed_at: chrono::Utc::now().timestamp(),
                        }
                    } else {
                        OperationState::Failed {
                            operation: "Remove Docker".to_string(),
                            error: error.unwrap_or_else(|| "Unknown error".to_string()),
                            failed_at: chrono::Utc::now().timestamp(),
                        }
                    };
                }
                if success {
                    dure_info!("Docker removed successfully from {}", host);
                } else {
                    dure_error!("Docker removal failed on {}", host);
                }
            }

            ViewModelEvent::Ssh(SshEvent::DureCheckCompleted { host, installed, running, services }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.dure_installed = installed;
                    row.last_dure_check = Some(DureCheckResult {
                        installed,
                        running,
                        services,
                        checked_at: chrono::Utc::now().timestamp(),
                    });
                    row.operation_state = OperationState::Completed {
                        operation: "Dure Check".to_string(),
                        completed_at: chrono::Utc::now().timestamp(),
                    };
                }
            }

            ViewModelEvent::Ssh(SshEvent::DureInstallCompleted { host, success, error }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.dure_installed = success;
                    row.operation_state = if success {
                        OperationState::Completed {
                            operation: "Install Dure".to_string(),
                            completed_at: chrono::Utc::now().timestamp(),
                        }
                    } else {
                        OperationState::Failed {
                            operation: "Install Dure".to_string(),
                            error: error.unwrap_or_else(|| "Unknown error".to_string()),
                            failed_at: chrono::Utc::now().timestamp(),
                        }
                    };
                }
                if success {
                    dure_info!("Dure installed successfully on {}", host);
                } else {
                    dure_error!("Dure installation failed on {}", host);
                }
            }

            ViewModelEvent::Ssh(SshEvent::DureRemoveCompleted { host, success, error }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.dure_installed = !success;
                    row.operation_state = if success {
                        OperationState::Completed {
                            operation: "Remove Dure".to_string(),
                            completed_at: chrono::Utc::now().timestamp(),
                        }
                    } else {
                        OperationState::Failed {
                            operation: "Remove Dure".to_string(),
                            error: error.unwrap_or_else(|| "Unknown error".to_string()),
                            failed_at: chrono::Utc::now().timestamp(),
                        }
                    };
                }
                if success {
                    dure_info!("Dure removed successfully from {}", host);
                } else {
                    dure_error!("Dure removal failed on {}", host);
                }
            }

            ViewModelEvent::Logs(crate::viewmodel::logs::LogEvent::LogsRetrieved { filter_id, lines }) => {
                dure_info!("===== LogsRetrieved EVENT: filter_id='{}', {} lines =====", filter_id, lines.len());

                // Debug: Show all current row hosts
                let all_hosts: Vec<_> = self.rows.iter().map(|r| r.host.as_str()).collect();
                dure_info!("Current SSH rows: {:?}", all_hosts);

                // Update drawer logs for matching SSH host
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == filter_id) {
                    dure_info!("✓ Found matching row for '{}'", filter_id);
                    row.drawer_state.set_logs(lines.clone());
                    dure_info!("✓ Set logs, drawer now has {} lines", row.drawer_state.logs.len());

                    if !lines.is_empty() {
                        dure_info!("First log line: '{}'", lines[0]);
                    }
                } else {
                    dure_error!("✗ No matching row found for filter_id '{}'", filter_id);
                }
            }

            _ => {}
        }
    }

    /// Render the SSH tab
    pub fn ui(
        &mut self,
        profile: &Option<crate::calc::profile::ProfileContext>,
        ui: &mut egui::Ui,
        mut vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        // Poll for ViewModel events
        if let Some(ref mut viewmodel) = vm {
            let events = viewmodel.poll_events(ui.ctx());
            for event in events {
                self.handle_event(event, profile);
            }
        }

        // Load rows on first render
        if !self.loaded {
            self.load_rows(profile);
        }

        // Error dialog
        let mut close_error = false;
        if let Some(ref error) = self.load_error {
            let error_msg = error.clone();
            egui::Window::new("Error")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ui.ctx(), |ui| {
                    ui.label(&error_msg);
                    if ui.add(MaterialButton::filled("OK")).clicked() {
                        close_error = true;
                    }
                });
        }
        if close_error {
            self.load_error = None;
            return;
        }
        if self.load_error.is_some() {
            return;
        }

        // SSH host table (header is inside render_table)
        self.render_table(ui, profile, vm.as_deref_mut());

        // Add host dialog
        self.render_add_dialog(ui, profile);

        // Edit host dialog
        self.render_edit_dialog(ui, profile);
    }

    /// Render SSH hosts table with drawer
    fn render_table(
        &mut self,
        ui: &mut egui::Ui,
        profile: &Option<crate::calc::profile::ProfileContext>,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        use egui_material3::{data_table, MaterialButton};

        // Calculate responsive column widths (match platform tab layout)
        let available_width = ui.available_width() - 40.0;
        let base_width = 740.0; // Match platform tab: 230 + 510
        let width_ratio = (available_width / base_width).min(1.5).max(0.8);

        // Build table with drawer support
        let table_id = egui::Id::new("ssh_hosts_table");

        let mut table = data_table()
            .id(table_id)
            .allow_selection(false)
            .allow_drawer(true)
            .min_row_height(100.0)
            .drawer_row_height(500.0)
            .column("Host", 230.0 * width_ratio, false)
            .column("Operations", 510.0 * width_ratio, false);

        for row in self.rows.iter() {
            let row_for_cells = row.clone();
            let row_for_drawer = row.clone();
            let row_for_actions = row.clone();

            let row_for_ops = row.clone();
            let mut action_trigger: Option<SshAction> = None;

            table = table.row(move |r| {
                r.cell_widget(move |ui| {
                        render_host_column(ui, &row_for_cells);
                    })
                    .cell_widget(move |ui| {
                        let mut local_action = None;
                        render_operations_column(ui, &row_for_ops, &mut local_action);
                        if let Some(action) = local_action {
                            ui.data_mut(|d| {
                                d.insert_temp(egui::Id::new("ssh_pending_action"), action);
                            });
                        }
                    })
                    .drawer(move |ui| {
                        use crate::ui_tabs::ssh_drawer;
                        let mut tab_switch = None;
                        ssh_drawer::render_drawer(ui, &row_for_drawer, &row_for_drawer.drawer_state, &mut tab_switch);

                        if let Some(new_tab) = tab_switch {
                            ui.data_mut(|d| {
                                d.insert_temp(
                                    egui::Id::new("ssh_drawer_tab_switch").with(&row_for_drawer.host),
                                    (row_for_drawer.host.clone(), new_tab),
                                );
                            });
                        }
                    })
            });
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading("SSH Hosts");
            ui.add_space(4.0);
            ui.label("Manage SSH host connections and services.");
            ui.add_space(8.0);

            if ui
                .add(MaterialButton::filled("Add SSH Host"))
                .clicked()
            {
                self.show_add_dialog = true;
            }
            ui.add_space(8.0);

            table.show(ui);
        });

        // Handle pending actions
        if let Some(action) = ui.data(|d| d.get_temp::<SshAction>(egui::Id::new("ssh_pending_action"))) {
            ui.data_mut(|d| d.remove::<SshAction>(egui::Id::new("ssh_pending_action")));

            match action {
                SshAction::Refresh(host) => {
                    if let Some(ref vm) = vm {
                        let _ = vm.refresh_ssh_host(host.clone());
                        dure_info!("Refresh triggered for {}", host);
                    }
                }
                SshAction::SshCheck(host) => {
                    if let Some(ref vm) = vm {
                        let _ = vm.ssh_check(host.clone());
                        dure_info!("SSH check triggered for {}", host);
                    }
                }
                SshAction::Edit(host) => {
                    self.start_edit_host(&host, profile);
                }
                SshAction::Delete(host) => {
                    self.delete_host(&host, profile);
                }
                SshAction::CheckBase(host) => {
                    if let Some(ref vm) = vm {
                        let _ = vm.check_base(host.clone());
                        dure_info!("Base check triggered for {}", host);
                    }
                }
                SshAction::InstallBase(host) => {
                    if let Some(ref vm) = vm {
                        let _ = vm.install_base(host.clone());
                        dure_info!("Base installation triggered for {}", host);
                    }
                }
                SshAction::CheckDocker(host) => {
                    if let Some(ref vm) = vm {
                        let _ = vm.check_docker(host.clone());
                        dure_info!("Docker check triggered for {}", host);
                    }
                }
                SshAction::InstallDocker(host) => {
                    if let Some(ref vm) = vm {
                        let _ = vm.install_docker_daemon(host.clone());
                        dure_info!("Docker installation triggered for {}", host);
                    }
                }
                SshAction::RemoveDocker(host) => {
                    if let Some(ref vm) = vm {
                        let _ = vm.remove_docker(host.clone());
                        dure_info!("Docker removal triggered for {}", host);
                    }
                }
                SshAction::CheckDure(host) => {
                    if let Some(ref vm) = vm {
                        let _ = vm.check_dure(host.clone());
                        dure_info!("Dure check triggered for {}", host);
                    }
                }
                SshAction::InstallDure(host) => {
                    dure_info!("Install Dure on {} (env dialog not implemented)", host);
                    // TODO: Show .env configuration dialog before calling vm.install_dure()
                }
                SshAction::RemoveDure(host) => {
                    if let Some(ref vm) = vm {
                        let _ = vm.remove_dure(host.clone());
                        dure_info!("Dure removal triggered for {}", host);
                    }
                }
            }
        }

        // Handle drawer tab switch
        if let Some((host, new_tab)) = ui.data(|d| {
            self.rows.iter().find_map(|r| {
                d.get_temp::<(String, DrawerTab)>(egui::Id::new("ssh_drawer_tab_switch").with(&r.host))
            })
        }) {
            if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                row.drawer_state.switch_tab(new_tab);

                // Load data when switching to Logs or Operations tabs
                match new_tab {
                    DrawerTab::Logs => {
                        // Request logs from log system
                        dure_info!("===== LOGS TAB SWITCH: Requesting logs for host {} =====", host);
                        if let Some(ref vm) = vm {
                            match vm.get_ssh_logs(host.clone()) {
                                Ok(_) => dure_info!("✓ Successfully sent GetLogs request for {}", host),
                                Err(e) => dure_error!("✗ Failed to send GetLogs request: {}", e),
                            }
                        } else {
                            dure_error!("✗ No ViewModel available");
                        }
                    }
                    DrawerTab::Operations => {
                        // Load operations from database
                        self.load_operations_for_host(&host);
                        if let Some(updated_row) = self.rows.iter_mut().find(|r| r.host == host) {
                            updated_row.drawer_state.set_loading(false);
                        }
                    }
                    _ => {}
                }

                ui.data_mut(|d| {
                    d.remove::<(String, DrawerTab)>(egui::Id::new("ssh_drawer_tab_switch").with(&host));
                });
            }
        }
    }

    /// Render add host dialog
    fn render_add_dialog(
        &mut self,
        ui: &mut egui::Ui,
        profile: &Option<crate::calc::profile::ProfileContext>,
    ) {
        if !self.show_add_dialog {
            return;
        }

        let mut open = self.show_add_dialog;

        egui::Window::new("Add SSH Host")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label("Configure a new SSH host connection:");
                ui.add_space(8.0);

                ui.label("Host (IP or domain):");
                ui.text_edit_singleline(&mut self.add_host);
                ui.add_space(8.0);

                ui.label("Port:");
                ui.text_edit_singleline(&mut self.add_port);
                ui.add_space(8.0);

                ui.checkbox(&mut self.add_use_password, "Use password authentication");
                if self.add_use_password {
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.add_password).password(true));
                }
                ui.add_space(8.0);

                ui.checkbox(&mut self.add_use_private_key, "Use private key authentication");
                if self.add_use_private_key {
                    ui.label("Private key path:");
                    ui.text_edit_singleline(&mut self.add_private_key_path);
                }
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.add(MaterialButton::filled("Add")).clicked() {
                        self.add_host_action(profile);
                    }
                    if ui.add(MaterialButton::outlined("Cancel")).clicked() {
                        self.show_add_dialog = false;
                        self.reset_add_dialog();
                    }
                });
            });

        if !open {
            self.show_add_dialog = false;
        }
    }

    /// Add SSH host action
    fn add_host_action(&mut self, profile: &Option<crate::calc::profile::ProfileContext>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Validate port number
            let port = match self.add_port.parse::<u16>() {
                Ok(p) => p,
                Err(e) => {
                    self.load_error = Some(format!("Invalid port '{}': {}", self.add_port, e));
                    return;
                }
            };

            match load_config(profile) {
                Ok((mut app_config, config_path)) => {
                    // Check for duplicates
                    if app_config
                        .ssh_hosts
                        .iter()
                        .any(|h| h.host == self.add_host)
                    {
                        self.load_error = Some(format!("Host {} already exists", self.add_host));
                        return;
                    }

                    // Create new host config
                    let host_config = SshHostConfig {
                        host: self.add_host.clone(),
                        password: if self.add_use_password {
                            Some(self.add_password.clone())
                        } else {
                            None
                        },
                        private_key_path: if self.add_use_private_key {
                            Some(self.add_private_key_path.clone())
                        } else {
                            None
                        },
                        keyring_domain: None,
                        port,
                        initialized: false,
                        last_status: None,
                        platform_name: None,
                        docker_containers: Vec::new(),
                        ansible_roles: Vec::new(),
                        dure_wss_config: None,
                    };

                    app_config.ssh_hosts.push(host_config);

                    // Save config
                    if let Err(e) = app_config.save(&config_path) {
                        self.load_error = Some(format!("Failed to save config: {}", e));
                        return;
                    }

                    dure_info!("SSH host '{}' added successfully", self.add_host);

                    // Reload rows
                    self.loaded = false;
                    self.show_add_dialog = false;
                    self.reset_add_dialog();
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {}", e));
                }
            }
        }
    }

    /// Reset add dialog fields
    fn reset_add_dialog(&mut self) {
        self.add_host.clear();
        self.add_password.clear();
        self.add_private_key_path.clear();
        self.add_port = "22".to_string();
        self.add_use_password = false;
        self.add_use_private_key = false;
    }

    /// Start editing an SSH host
    fn start_edit_host(&mut self, host: &str, profile: &Option<crate::calc::profile::ProfileContext>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match load_config(profile) {
                Ok((app_config, _)) => {
                    if let Some(host_config) = app_config.ssh_hosts.iter().find(|h| h.host == host) {
                        self.edit_original_host = host.to_string();
                        self.edit_host = host_config.host.clone();
                        self.edit_port = host_config.port.to_string();
                        self.edit_use_password = host_config.password.is_some();
                        self.edit_password = host_config.password.clone().unwrap_or_default();
                        self.edit_use_private_key = host_config.private_key_path.is_some();
                        self.edit_private_key_path = host_config.private_key_path.clone().unwrap_or_default();
                        self.show_edit_dialog = true;
                        dure_info!("Started editing host {}", host);
                    }
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {}", e));
                }
            }
        }
    }

    /// Render edit SSH host dialog
    fn render_edit_dialog(
        &mut self,
        ui: &mut egui::Ui,
        profile: &Option<crate::calc::profile::ProfileContext>,
    ) {
        if !self.show_edit_dialog {
            return;
        }

        let mut open = self.show_edit_dialog;

        egui::Window::new("Edit SSH Host")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label("Edit SSH host connection:");
                ui.add_space(8.0);

                ui.label("Host (IP or domain):");
                ui.text_edit_singleline(&mut self.edit_host);
                ui.add_space(8.0);

                ui.label("Port:");
                ui.text_edit_singleline(&mut self.edit_port);
                ui.add_space(8.0);

                ui.checkbox(&mut self.edit_use_password, "Use password authentication");
                if self.edit_use_password {
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.edit_password).password(true));
                }
                ui.add_space(8.0);

                ui.checkbox(&mut self.edit_use_private_key, "Use private key authentication");
                if self.edit_use_private_key {
                    ui.label("Private key path:");
                    ui.text_edit_singleline(&mut self.edit_private_key_path);
                }
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.add(MaterialButton::filled("Save")).clicked() {
                        self.save_edit_host(profile);
                    }
                    if ui.add(MaterialButton::outlined("Cancel")).clicked() {
                        self.show_edit_dialog = false;
                        self.reset_edit_dialog();
                    }
                });
            });

        if !open {
            self.show_edit_dialog = false;
        }
    }

    /// Save edited SSH host
    fn save_edit_host(&mut self, profile: &Option<crate::calc::profile::ProfileContext>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let port = match self.edit_port.parse::<u16>() {
                Ok(p) => p,
                Err(e) => {
                    self.load_error = Some(format!("Invalid port '{}': {}", self.edit_port, e));
                    return;
                }
            };

            match load_config(profile) {
                Ok((mut app_config, config_path)) => {
                    if let Some(host_config) = app_config.ssh_hosts.iter_mut().find(|h| h.host == self.edit_original_host) {
                        host_config.host = self.edit_host.clone();
                        host_config.port = port;
                        host_config.password = if self.edit_use_password {
                            Some(self.edit_password.clone())
                        } else {
                            None
                        };
                        host_config.private_key_path = if self.edit_use_private_key {
                            Some(self.edit_private_key_path.clone())
                        } else {
                            None
                        };

                        if let Err(e) = app_config.save(&config_path) {
                            self.load_error = Some(format!("Failed to save config: {}", e));
                            return;
                        }

                        self.show_edit_dialog = false;
                        self.reset_edit_dialog();
                        self.loaded = false;
                        dure_info!("Saved changes to SSH host {}", self.edit_host);
                    }
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {}", e));
                }
            }
        }
    }

    /// Reset edit dialog fields
    fn reset_edit_dialog(&mut self) {
        self.edit_original_host.clear();
        self.edit_host.clear();
        self.edit_password.clear();
        self.edit_private_key_path.clear();
        self.edit_port = "22".to_string();
        self.edit_use_password = false;
        self.edit_use_private_key = false;
    }

    /// Load operation logs for SSH host from database
    fn load_operations_for_host(&mut self, host: &str) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::calc::db;
            use crate::storage::models::opslog::list_by_project;

            // Use SSH host as project_id for filtering
            if let Ok(mut conn) = std::panic::catch_unwind(|| db::establish_connection()) {
                if let Ok(ops) = list_by_project(&mut conn, host, 50) {
                    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                        row.drawer_state.set_operations(ops);
                    }
                } else {
                    dure_error!("Failed to load operations for host {}", host);
                }
            }
        }
    }

    /// Delete SSH host
    fn delete_host(&mut self, host: &str, profile: &Option<crate::calc::profile::ProfileContext>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match load_config(profile) {
                Ok((mut app_config, config_path)) => {
                    app_config.ssh_hosts.retain(|h| h.host != host);

                    if let Err(e) = app_config.save(&config_path) {
                        self.load_error = Some(format!("Failed to save config: {}", e));
                        return;
                    }

                    dure_info!("SSH host '{}' deleted successfully", host);

                    self.loaded = false;
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {}", e));
                }
            }
        }
    }
}

#[cfg(test)]
mod operation_state_tests {
    use super::*;

    #[test]
    fn test_operation_state_lifecycle() {
        let state = OperationState::start("test_op");

        match &state {
            OperationState::InProgress { operation, .. } => {
                assert_eq!(operation, "test_op");
            }
            _ => panic!("Expected InProgress state"),
        }

        let state = state.complete();

        match &state {
            OperationState::Completed { operation, .. } => {
                assert_eq!(operation, "test_op");
            }
            _ => panic!("Expected Completed state"),
        }
    }

    #[test]
    fn test_operation_state_failure() {
        let state = OperationState::start("failing_op");
        let state = state.fail("connection timeout");

        match &state {
            OperationState::Failed { operation, error, .. } => {
                assert_eq!(operation, "failing_op");
                assert_eq!(error, "connection timeout");
            }
            _ => panic!("Expected Failed state"),
        }
    }

    #[test]
    fn test_operation_state_reset() {
        let mut state = OperationState::start("test_op");
        state.reset();

        assert_eq!(state, OperationState::Idle);
    }

    #[test]
    fn test_ssh_check_result_creation() {
        let result = SshCheckResult {
            connected: true,
            error: None,
            checked_at: 1234567890,
        };

        assert!(result.connected);
        assert!(result.error.is_none());
        assert_eq!(result.checked_at, 1234567890);
    }

    #[test]
    fn test_ssh_check_result_with_error() {
        let result = SshCheckResult {
            connected: false,
            error: Some("Connection refused".to_string()),
            checked_at: 1234567890,
        };

        assert!(!result.connected);
        assert_eq!(result.error, Some("Connection refused".to_string()));
    }

    #[test]
    fn test_base_check_result_creation() {
        let result = BaseCheckResult {
            installed: true,
            missing_packages: vec![],
            checked_at: 1234567890,
        };

        assert!(result.installed);
        assert!(result.missing_packages.is_empty());
        assert_eq!(result.checked_at, 1234567890);
    }

    #[test]
    fn test_base_check_result_with_missing_packages() {
        let result = BaseCheckResult {
            installed: false,
            missing_packages: vec!["curl".to_string(), "jq".to_string()],
            checked_at: 1234567890,
        };

        assert!(!result.installed);
        assert_eq!(result.missing_packages.len(), 2);
        assert!(result.missing_packages.contains(&"curl".to_string()));
        assert!(result.missing_packages.contains(&"jq".to_string()));
    }

    #[test]
    fn test_docker_check_result_creation() {
        let result = DockerCheckResult {
            installed: true,
            version: Some("24.0.0".to_string()),
            checked_at: 1234567890,
        };

        assert!(result.installed);
        assert_eq!(result.version, Some("24.0.0".to_string()));
        assert_eq!(result.checked_at, 1234567890);
    }

    #[test]
    fn test_docker_check_result_no_version() {
        let result = DockerCheckResult {
            installed: false,
            version: None,
            checked_at: 1234567890,
        };

        assert!(!result.installed);
        assert!(result.version.is_none());
    }

    #[test]
    fn test_dure_check_result_creation() {
        let result = DureCheckResult {
            installed: true,
            running: true,
            services: vec!["api".to_string(), "worker".to_string()],
            checked_at: 1234567890,
        };

        assert!(result.installed);
        assert!(result.running);
        assert_eq!(result.services.len(), 2);
        assert!(result.services.contains(&"api".to_string()));
    }

    #[test]
    fn test_dure_check_result_not_running() {
        let result = DureCheckResult {
            installed: true,
            running: false,
            services: vec![],
            checked_at: 1234567890,
        };

        assert!(result.installed);
        assert!(!result.running);
        assert!(result.services.is_empty());
    }

    #[test]
    fn test_ssh_row_new_fields_initialized() {
        let drawer_state = DrawerState::new();
        let row = SshRow {
            host: "192.168.1.1".to_string(),
            port: 22,
            platform_id: None,
            ssh_connected: false,
            base_installed: false,
            docker_installed: false,
            dure_installed: false,
            ssh_private_key: None,
            drawer_open: false,
            drawer_state,
            operation_state: OperationState::Idle,
            last_ssh_check: None,
            last_base_check: None,
            last_docker_check: None,
            last_dure_check: None,
        };

        assert_eq!(row.host, "192.168.1.1");
        assert_eq!(row.port, 22);
        assert!(row.platform_id.is_none());
        assert_eq!(row.operation_state, OperationState::Idle);
        assert!(row.last_ssh_check.is_none());
        assert!(row.last_base_check.is_none());
        assert!(row.last_docker_check.is_none());
        assert!(row.last_dure_check.is_none());
    }

    #[test]
    fn test_ssh_row_with_check_results() {
        let drawer_state = DrawerState::new();
        let ssh_check = SshCheckResult {
            connected: true,
            error: None,
            checked_at: 1234567890,
        };
        let docker_check = DockerCheckResult {
            installed: true,
            version: Some("24.0.0".to_string()),
            checked_at: 1234567890,
        };

        let row = SshRow {
            host: "192.168.1.1".to_string(),
            port: 22,
            platform_id: None,
            ssh_connected: true,
            base_installed: false,
            docker_installed: true,
            dure_installed: false,
            ssh_private_key: None,
            drawer_open: false,
            drawer_state,
            operation_state: OperationState::Completed {
                operation: "ssh_check".to_string(),
                completed_at: 1234567890,
            },
            last_ssh_check: Some(ssh_check),
            last_base_check: None,
            last_docker_check: Some(docker_check),
            last_dure_check: None,
        };

        assert!(row.last_ssh_check.is_some());
        assert!(row.last_docker_check.is_some());
        assert!(row.last_base_check.is_none());
        assert!(row.last_dure_check.is_none());

        if let Some(ssh_check) = &row.last_ssh_check {
            assert!(ssh_check.connected);
        }
    }

    #[test]
    fn test_operation_state_transitions() {
        // Idle -> InProgress -> Completed
        let mut state = OperationState::Idle;
        state = OperationState::start("check");
        assert!(matches!(state, OperationState::InProgress { .. }));

        state = state.complete();
        assert!(matches!(state, OperationState::Completed { .. }));

        // Can reset from Completed
        state.reset();
        assert_eq!(state, OperationState::Idle);
    }

    #[test]
    fn test_operation_state_fail_from_progress() {
        let state = OperationState::start("check");
        let state = state.fail("timeout");
        assert!(matches!(state, OperationState::Failed { .. }));
    }

    #[test]
    fn test_operation_state_complete_from_idle_is_noop() {
        let state = OperationState::Idle;
        let state = state.complete();
        // Should remain Idle since we can't transition from Idle
        assert_eq!(state, OperationState::Idle);
    }

    #[test]
    fn test_check_result_clone() {
        let result = SshCheckResult {
            connected: true,
            error: None,
            checked_at: 1234567890,
        };
        let cloned = result.clone();
        assert_eq!(result.connected, cloned.connected);
        assert_eq!(result.checked_at, cloned.checked_at);
    }
}
