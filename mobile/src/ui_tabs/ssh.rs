//! SSH tab - SSH host configuration and management with drawer

use crate::{dure_debug, dure_error, dure_info, dure_warn};
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{dialog, MaterialButton};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::config::{AppConfig, SshHostConfig};
use crate::ui_components::drawer::{LogsRenderer, OperationsRenderer};
use crate::viewmodel::ssh::{DrawerState, DrawerTab};

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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SshCheckResult {
    pub connected: bool,
    pub error: Option<String>,
    pub checked_at: i64,
}

/// Host system check result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HostCheckResult {
    pub installed: bool,
    pub missing_packages: Vec<String>,
    pub checked_at: i64,
}

/// Docker installation check result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DockerCheckResult {
    pub installed: bool,
    pub version: Option<String>,
    pub checked_at: i64,
}

/// Dure service check result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DureCheckResult {
    pub installed: bool,
    pub running: bool,
    pub services: Vec<String>,
    #[serde(default)]
    pub containers: Vec<crate::calc::dure_mycart::DureContainer>,
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
    pub host_installed: bool,
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
    pub last_host_check: Option<HostCheckResult>,
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
    CheckHost(String),     // host
    InstallHost(String),   // host
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
    use egui_material3::linear_progress;

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = BUTTON_VERTICAL_SPACING;
        ui.spacing_mut().item_spacing.x = BUTTON_HORIZONTAL_SPACING;

        let operation_in_progress =
            matches!(row.operation_state, OperationState::InProgress { .. });
        let overlay_pos = ui.cursor().min;
        let column_width = ui.available_width();

        // Row 1: Basic operations
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = BUTTON_HORIZONTAL_SPACING;

            if ui
                .add(MaterialButton::outlined(tr!("ssh-button-refresh")).small())
                .on_hover_text(tr!("ssh-button-refresh-tooltip"))
                .clicked()
            {
                *action_trigger = Some(SshAction::Refresh(row.host.clone()));
            }

            if ui
                .add(MaterialButton::outlined(tr!("ssh-button-ssh-check")).small())
                .on_hover_text(tr!("ssh-button-ssh-check-tooltip"))
                .clicked()
            {
                *action_trigger = Some(SshAction::SshCheck(row.host.clone()));
            }

            if ui
                .add(MaterialButton::outlined(tr!("ssh-button-edit")).small())
                .on_hover_text(tr!("ssh-button-edit-tooltip"))
                .clicked()
            {
                *action_trigger = Some(SshAction::Edit(row.host.clone()));
            }

            if ui
                .add(MaterialButton::outlined(tr!("ssh-button-delete")).small())
                .on_hover_text(tr!("ssh-button-delete-tooltip"))
                .clicked()
            {
                *action_trigger = Some(SshAction::Delete(row.host.clone()));
            }
        });

        // Row 2: Host and Docker
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = BUTTON_HORIZONTAL_SPACING;

            // Check Host (always)
            if ui
                .add(MaterialButton::outlined(tr!("ssh-button-check-host")).small())
                .on_hover_text(tr!("ssh-button-check-host-tooltip"))
                .clicked()
            {
                *action_trigger = Some(SshAction::CheckHost(row.host.clone()));
            }

            // Install Host (only when NOT installed)
            if !row.host_installed {
                if ui
                    .add(MaterialButton::outlined(tr!("ssh-button-install-host")).small())
                    .on_hover_text(tr!("ssh-button-install-host-tooltip"))
                    .clicked()
                {
                    *action_trigger = Some(SshAction::InstallHost(row.host.clone()));
                }
            }

            // Check Docker (always)
            if ui
                .add(MaterialButton::outlined(tr!("ssh-button-check-docker")).small())
                .on_hover_text(tr!("ssh-button-check-docker-tooltip"))
                .clicked()
            {
                *action_trigger = Some(SshAction::CheckDocker(row.host.clone()));
            }

            // Install/Remove Docker (conditional)
            if !row.docker_installed {
                if ui
                    .add(MaterialButton::outlined(tr!("ssh-button-install-docker")).small())
                    .on_hover_text(tr!("ssh-button-install-docker-tooltip"))
                    .clicked()
                {
                    *action_trigger = Some(SshAction::InstallDocker(row.host.clone()));
                }
            } else {
                if ui
                    .add(MaterialButton::outlined(tr!("ssh-button-remove-docker")).small())
                    .on_hover_text(tr!("ssh-button-remove-docker-tooltip"))
                    .clicked()
                {
                    *action_trigger = Some(SshAction::RemoveDocker(row.host.clone()));
                }
            }
        });

        // Row 3: Dure operations
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = BUTTON_HORIZONTAL_SPACING;

            // Check Dure (always)
            if ui
                .add(MaterialButton::outlined(tr!("ssh-button-check-dure")).small())
                .on_hover_text(tr!("ssh-button-check-dure-tooltip"))
                .clicked()
            {
                *action_trigger = Some(SshAction::CheckDure(row.host.clone()));
            }

            // Install/Remove Dure (conditional)
            if !row.dure_installed {
                if ui
                    .add(MaterialButton::outlined(tr!("ssh-button-install-dure")).small())
                    .on_hover_text(tr!("ssh-button-install-dure-tooltip"))
                    .clicked()
                {
                    *action_trigger = Some(SshAction::InstallDure(row.host.clone()));
                }
            } else {
                if ui
                    .add(MaterialButton::outlined(tr!("ssh-button-remove-dure")).small())
                    .on_hover_text(tr!("ssh-button-remove-dure-tooltip"))
                    .clicked()
                {
                    *action_trigger = Some(SshAction::RemoveDure(row.host.clone()));
                }
            }
        });

        // Overlay progress on foreground layer
        if operation_in_progress {
            let current_area_id = ui.layer_id().id;
            let foreground_layer = egui::LayerId::new(egui::Order::Foreground, current_area_id);

            ui.with_layer_id(foreground_layer, |ui| {
                use egui_material3::theme::get_global_color;
                let progress_rect =
                    egui::Rect::from_min_size(overlay_pos, egui::vec2(column_width, 30.0));
                let overlay_color = get_global_color("surfaceContainer");
                ui.painter()
                    .rect_filled(progress_rect, egui::Rounding::same(4), overlay_color);

                ui.allocate_ui_at_rect(progress_rect, |ui| {
                    ui.add_space(2.0);
                    if let OperationState::InProgress { operation, .. } = &row.operation_state {
                        ui.label(operation);
                    }
                    ui.add(
                        linear_progress()
                            .indeterminate(true)
                            .width(column_width - 20.0)
                            .height(4.0)
                            .four_color_enabled(true),
                    );
                });
            });
        }
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
                    .size(BadgeSize::Small),
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

    // Install Dure dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_install_dure_dialog: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    install_dure_host: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    install_dure_domain: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    install_dure_image_tag: String,

    // Remove Dure dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_remove_dure_dialog: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    remove_dure_host: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    remove_dure_delete_data: bool,

    /// Automatic refresh deadline for log updates after operations
    #[cfg_attr(feature = "serde", serde(skip))]
    refresh_until: Option<Instant>,

    /// Install Dure wizard dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    install_dure_dialog: Option<crate::ui_dlg::ssh::install_dure::InstallDure>,
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
            show_install_dure_dialog: false,
            install_dure_host: String::new(),
            install_dure_domain: String::new(),
            install_dure_image_tag: "latest".to_string(),
            show_remove_dure_dialog: false,
            remove_dure_host: String::new(),
            remove_dure_delete_data: false,
            refresh_until: None,
            install_dure_dialog: None,
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
        None => Err(tr!("ssh-error-no-profile")),
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
                            platform_id: host_config.platform_id.clone(),
                            ssh_connected: host_config
                                .last_ssh_check
                                .as_ref()
                                .map(|c| c.connected)
                                .unwrap_or(false),
                            host_installed: host_config
                                .last_host_check
                                .as_ref()
                                .map(|c| c.installed)
                                .unwrap_or(false),
                            docker_installed: host_config
                                .last_docker_check
                                .as_ref()
                                .map(|c| c.installed)
                                .unwrap_or_else(|| !host_config.docker_containers.is_empty()),
                            dure_installed: host_config
                                .last_dure_check
                                .as_ref()
                                .map(|c| c.installed)
                                .unwrap_or_else(|| host_config.dure_wss_config.is_some()),
                            ssh_private_key: host_config.private_key_path.clone(),
                            drawer_open: false,
                            drawer_state,
                            operation_state: OperationState::Idle,
                            last_ssh_check: host_config.last_ssh_check.clone(),
                            last_host_check: host_config.last_host_check.clone(),
                            last_docker_check: host_config.last_docker_check.clone(),
                            last_dure_check: host_config.last_dure_check.clone(),
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
            self.load_error = Some(tr!("ssh-error-wasm"));
            self.loaded = true;
        }
    }

    /// Reset loaded flag to force config reload on next render
    pub fn reset_loaded(&mut self) {
        self.loaded = false;
    }

    /// Check if config file has changed since last load
    #[cfg(not(target_arch = "wasm32"))]
    pub fn should_reload_config(
        &self,
        profile: &Option<crate::calc::profile::ProfileContext>,
    ) -> bool {
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
    pub fn should_reload_config(
        &self,
        _profile: &Option<crate::calc::profile::ProfileContext>,
    ) -> bool {
        // WASM doesn't have config files
        false
    }

    /// Returns true if automatic refresh window is active (for log updates)
    pub fn needs_refresh(&mut self) -> bool {
        if let Some(deadline) = self.refresh_until {
            if Instant::now() < deadline {
                true // Still within 5-second refresh window
            } else {
                self.refresh_until = None; // Clear expired deadline
                false
            }
        } else {
            false
        }
    }

    /// Save SSH check cache results to config
    #[cfg(not(target_arch = "wasm32"))]
    fn save_ssh_cache(
        &self,
        profile: &Option<crate::calc::profile::ProfileContext>,
        host: &str,
        last_ssh_check: Option<SshCheckResult>,
        last_host_check: Option<HostCheckResult>,
        last_docker_check: Option<DockerCheckResult>,
        last_dure_check: Option<DureCheckResult>,
    ) {
        dure_info!("Saving SSH cache for host: {}", host);
        match load_config(profile) {
            Ok((mut app_config, config_path)) => {
                dure_info!("Config loaded from: {:?}", config_path);
                if let Some(host_config) = app_config.ssh_hosts.iter_mut().find(|h| h.host == host)
                {
                    dure_info!("Found host config for: {}", host);
                    host_config.last_ssh_check = last_ssh_check;
                    host_config.last_host_check = last_host_check;
                    host_config.last_docker_check = last_docker_check;
                    host_config.last_dure_check = last_dure_check;

                    match app_config.save(&config_path) {
                        Ok(_) => dure_info!("SSH cache saved successfully to: {:?}", config_path),
                        Err(e) => dure_error!("Failed to save SSH cache to config: {}", e),
                    }
                } else {
                    dure_error!(
                        "Host '{}' not found in config. Available hosts: {:?}",
                        host,
                        app_config
                            .ssh_hosts
                            .iter()
                            .map(|h| &h.host)
                            .collect::<Vec<_>>()
                    );
                }
            }
            Err(e) => {
                dure_error!("Failed to load config for saving SSH cache: {}", e);
            }
        }
    }

    /// Find domain for SSH host by looking up platform config
    ///
    /// Logic:
    /// 1. Extract IP from host string (e.g., "root@1.2.3.4" → "1.2.3.4")
    /// 2. Find platform containing VM with matching external_ip
    /// 3. Check platform.bound_domain first
    /// 4. Fall back to dns_domains entry with matching platform_id
    #[cfg(not(target_arch = "wasm32"))]
    fn find_domain_for_host(host: &str, config_path: &std::path::PathBuf) -> Option<String> {
        use crate::config::AppConfig;

        // Extract IP from host (format: "user@ip" or "ip")
        let ip = host.split('@').last().unwrap_or(host);

        // Load config
        let config = AppConfig::load(config_path).ok()?;

        // Find platform with VM matching this IP
        for platform in &config.platforms {
            for vm in &platform.vms {
                if vm.external_ip.as_deref() == Some(ip) {
                    // Found matching platform, check bound_domain first
                    if let Some(ref domain) = platform.bound_domain {
                        dure_info!("Found bound_domain {} for host {}", domain, host);
                        return Some(domain.clone());
                    }

                    // Get platform identifier (GCP uses gcp_selected_project_id)
                    let platform_id = if platform.platform_type == "gcp" {
                        platform.gcp_selected_project_id.as_ref()
                    } else {
                        None
                    };

                    // Look for domain in dns_domains with matching platform_id
                    if let Some(pid) = platform_id {
                        for dns_domain in &config.dns_domains {
                            if dns_domain.platform_id.as_ref() == Some(pid) {
                                dure_info!(
                                    "Found dns_domain {} for platform_id {} (host {})",
                                    dns_domain.domain,
                                    pid,
                                    host
                                );
                                return Some(dns_domain.domain.clone());
                            }
                        }
                    }

                    dure_error!("Platform found for host {} but no domain configured", host);
                    return None;
                }
            }
        }

        dure_error!("No platform found with VM matching host {}", host);
        None
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
            ViewModelEvent::Ssh(SshEvent::HostAdded { name, focus_tab }) => {
                dure_info!("SSH host {} added", name);
                // Note: focus_tab will be used when drawer opens for new host
                let _ = focus_tab; // Silence unused warning for now
                self.loaded = false; // Trigger reload
            }

            ViewModelEvent::Ssh(SshEvent::HostDeleted { name, focus_tab }) => {
                dure_info!("SSH host {} deleted", name);
                let _ = focus_tab; // Not used - no drawer to focus after deletion

                // Remove from config
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok((mut app_config, config_path)) = load_config(profile) {
                    app_config.ssh_hosts.retain(|h| h.host != name);
                    let _ = app_config.save(&config_path);
                }

                self.loaded = false; // Trigger reload
            }

            ViewModelEvent::Ssh(SshEvent::DockerStatusRetrieved {
                name, installed, ..
            }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.docker_installed = installed;
                }
            }

            ViewModelEvent::Ssh(SshEvent::RefreshCompleted {
                host,
                ssh_connected,
                host_installed,
                docker_installed,
                dure_installed,
                ssh_error,
                host_missing_packages,
                docker_version,
                dure_running,
                dure_services,
                focus_tab,
            }) => {
                #[cfg(not(target_arch = "wasm32"))]
                let cache_data = {
                    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                        let now = chrono::Utc::now().timestamp();

                        // Auto-focus drawer tab if specified
                        if let Some(tab) = focus_tab {
                            row.drawer_state.active_tab = tab;
                        }

                        row.ssh_connected = ssh_connected;
                        row.host_installed = host_installed;
                        row.docker_installed = docker_installed;
                        row.dure_installed = dure_installed;
                        row.operation_state = OperationState::Completed {
                            operation: tr!("ssh-operation-refresh"),
                            completed_at: now,
                        };

                        // Update cache from refresh results
                        row.last_ssh_check = Some(SshCheckResult {
                            connected: ssh_connected,
                            error: ssh_error.clone(),
                            checked_at: now,
                        });

                        row.last_host_check = Some(HostCheckResult {
                            installed: host_installed,
                            missing_packages: host_missing_packages.clone().unwrap_or_default(),
                            checked_at: now,
                        });

                        row.last_docker_check = Some(DockerCheckResult {
                            installed: docker_installed,
                            version: docker_version.clone(),
                            checked_at: now,
                        });

                        row.last_dure_check = Some(DureCheckResult {
                            installed: dure_installed,
                            running: dure_running.unwrap_or(false),
                            services: dure_services.clone().unwrap_or_default(),
                            containers: Vec::new(),
                            checked_at: now,
                        });

                        Some((
                            row.last_ssh_check.clone(),
                            row.last_host_check.clone(),
                            row.last_docker_check.clone(),
                            row.last_dure_check.clone(),
                        ))
                    } else {
                        None
                    }
                };

                #[cfg(not(target_arch = "wasm32"))]
                if let Some((ssh, host_check, docker, dure)) = cache_data {
                    self.save_ssh_cache(profile, &host, ssh, host_check, docker, dure);
                }

                #[cfg(target_arch = "wasm32")]
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.ssh_connected = ssh_connected;
                    row.host_installed = host_installed;
                    row.docker_installed = docker_installed;
                    row.dure_installed = dure_installed;
                    row.operation_state = OperationState::Completed {
                        operation: tr!("ssh-operation-refresh"),
                        completed_at: chrono::Utc::now().timestamp(),
                    };
                }

                dure_info!("Refresh completed for host {}", host);
            }

            ViewModelEvent::Ssh(SshEvent::SshCheckCompleted {
                host,
                connected,
                error,
                focus_tab,
            }) => {
                #[cfg(not(target_arch = "wasm32"))]
                let cache_data = {
                    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                        // Auto-focus drawer tab if specified
                        if let Some(tab) = focus_tab {
                            row.drawer_state.active_tab = tab;
                        }

                        row.ssh_connected = connected;
                        row.last_ssh_check = Some(SshCheckResult {
                            connected,
                            error: error.clone(),
                            checked_at: chrono::Utc::now().timestamp(),
                        });
                        row.operation_state = if error.is_none() {
                            OperationState::Completed {
                                operation: tr!("ssh-operation-ssh-check"),
                                completed_at: chrono::Utc::now().timestamp(),
                            }
                        } else {
                            OperationState::Failed {
                                operation: tr!("ssh-operation-ssh-check"),
                                error: error.unwrap(),
                                failed_at: chrono::Utc::now().timestamp(),
                            }
                        };
                        Some((
                            row.last_ssh_check.clone(),
                            row.last_host_check.clone(),
                            row.last_docker_check.clone(),
                            row.last_dure_check.clone(),
                        ))
                    } else {
                        None
                    }
                };

                #[cfg(not(target_arch = "wasm32"))]
                if let Some((ssh, host_check, docker, dure)) = cache_data {
                    self.save_ssh_cache(profile, &host, ssh, host_check, docker, dure);
                }

                #[cfg(target_arch = "wasm32")]
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.ssh_connected = connected;
                    row.last_ssh_check = Some(SshCheckResult {
                        connected,
                        error: error.clone(),
                        checked_at: chrono::Utc::now().timestamp(),
                    });
                    row.operation_state = if error.is_none() {
                        OperationState::Completed {
                            operation: tr!("ssh-operation-ssh-check"),
                            completed_at: chrono::Utc::now().timestamp(),
                        }
                    } else {
                        OperationState::Failed {
                            operation: tr!("ssh-operation-ssh-check"),
                            error: error.unwrap(),
                            failed_at: chrono::Utc::now().timestamp(),
                        }
                    };
                }
            }

            ViewModelEvent::Ssh(SshEvent::HostCheckCompleted {
                host,
                installed,
                missing_packages,
                focus_tab,
            }) => {
                #[cfg(not(target_arch = "wasm32"))]
                let cache_data = {
                    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                        // Auto-focus drawer tab if specified
                        if let Some(tab) = focus_tab {
                            row.drawer_state.active_tab = tab;
                        }

                        // Trigger 5-second refresh for log updates
                        self.refresh_until = Some(Instant::now() + Duration::from_secs(5));

                        row.host_installed = installed;
                        row.last_host_check = Some(HostCheckResult {
                            installed,
                            missing_packages,
                            checked_at: chrono::Utc::now().timestamp(),
                        });
                        row.operation_state = OperationState::Completed {
                            operation: tr!("ssh-operation-host-check"),
                            completed_at: chrono::Utc::now().timestamp(),
                        };
                        Some((
                            row.last_ssh_check.clone(),
                            row.last_host_check.clone(),
                            row.last_docker_check.clone(),
                            row.last_dure_check.clone(),
                        ))
                    } else {
                        None
                    }
                };

                #[cfg(not(target_arch = "wasm32"))]
                if let Some((ssh, host_check, docker, dure)) = cache_data {
                    self.save_ssh_cache(profile, &host, ssh, host_check, docker, dure);
                }

                #[cfg(target_arch = "wasm32")]
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.host_installed = installed;
                    row.last_host_check = Some(HostCheckResult {
                        installed,
                        missing_packages,
                        checked_at: chrono::Utc::now().timestamp(),
                    });
                    row.operation_state = OperationState::Completed {
                        operation: tr!("ssh-operation-base-check"),
                        completed_at: chrono::Utc::now().timestamp(),
                    };
                }
            }

            ViewModelEvent::Ssh(SshEvent::HostInstallCompleted {
                host,
                success,
                error,
            }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.host_installed = success;
                    row.operation_state = if success {
                        OperationState::Completed {
                            operation: tr!("ssh-operation-install-host"),
                            completed_at: chrono::Utc::now().timestamp(),
                        }
                    } else {
                        OperationState::Failed {
                            operation: tr!("ssh-operation-install-host"),
                            error: error.unwrap_or_else(|| tr!("ssh-error-unknown")),
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

            ViewModelEvent::Ssh(SshEvent::DockerCheckCompleted {
                host,
                installed,
                version,
                focus_tab,
            }) => {
                #[cfg(not(target_arch = "wasm32"))]
                let cache_data = {
                    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                        // Auto-focus drawer tab if specified
                        if let Some(tab) = focus_tab {
                            row.drawer_state.active_tab = tab;
                        }

                        // Trigger 5-second refresh for log updates
                        self.refresh_until = Some(Instant::now() + Duration::from_secs(5));

                        row.docker_installed = installed;
                        row.last_docker_check = Some(DockerCheckResult {
                            installed,
                            version,
                            checked_at: chrono::Utc::now().timestamp(),
                        });
                        row.operation_state = OperationState::Completed {
                            operation: tr!("ssh-operation-docker-check"),
                            completed_at: chrono::Utc::now().timestamp(),
                        };
                        Some((
                            row.last_ssh_check.clone(),
                            row.last_host_check.clone(),
                            row.last_docker_check.clone(),
                            row.last_dure_check.clone(),
                        ))
                    } else {
                        None
                    }
                };

                #[cfg(not(target_arch = "wasm32"))]
                if let Some((ssh, host_check, docker, dure)) = cache_data {
                    self.save_ssh_cache(profile, &host, ssh, host_check, docker, dure);
                }

                #[cfg(target_arch = "wasm32")]
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.docker_installed = installed;
                    row.last_docker_check = Some(DockerCheckResult {
                        installed,
                        version,
                        checked_at: chrono::Utc::now().timestamp(),
                    });
                    row.operation_state = OperationState::Completed {
                        operation: tr!("ssh-operation-docker-check"),
                        completed_at: chrono::Utc::now().timestamp(),
                    };
                }
            }

            ViewModelEvent::Ssh(SshEvent::DockerInstallCompleted {
                host,
                success,
                error,
                focus_tab,
            }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    // Auto-focus drawer tab if specified
                    if let Some(tab) = focus_tab {
                        row.drawer_state.active_tab = tab;
                    }

                    // Trigger 5-second refresh for log updates
                    self.refresh_until = Some(Instant::now() + Duration::from_secs(5));

                    row.docker_installed = success;
                    row.operation_state = if success {
                        OperationState::Completed {
                            operation: tr!("ssh-operation-install-docker"),
                            completed_at: chrono::Utc::now().timestamp(),
                        }
                    } else {
                        OperationState::Failed {
                            operation: tr!("ssh-operation-install-docker"),
                            error: error.unwrap_or_else(|| tr!("ssh-error-unknown")),
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

            ViewModelEvent::Ssh(SshEvent::DockerRemoveCompleted {
                host,
                success,
                error,
                focus_tab,
            }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    // Auto-focus drawer tab if specified
                    if let Some(tab) = focus_tab {
                        row.drawer_state.active_tab = tab;
                    }

                    row.docker_installed = !success;
                    row.operation_state = if success {
                        OperationState::Completed {
                            operation: tr!("ssh-operation-remove-docker"),
                            completed_at: chrono::Utc::now().timestamp(),
                        }
                    } else {
                        OperationState::Failed {
                            operation: tr!("ssh-operation-remove-docker"),
                            error: error.unwrap_or_else(|| tr!("ssh-error-unknown")),
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

            ViewModelEvent::Ssh(SshEvent::DureCheckCompleted {
                host,
                installed,
                running,
                services,
                containers,
                focus_tab,
            }) => {
                #[cfg(not(target_arch = "wasm32"))]
                let cache_data = {
                    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                        // Auto-focus drawer tab if specified
                        if let Some(tab) = focus_tab {
                            row.drawer_state.active_tab = tab;
                        }

                        // Trigger 5-second refresh for log updates
                        self.refresh_until = Some(Instant::now() + Duration::from_secs(5));

                        row.dure_installed = installed;
                        row.last_dure_check = Some(DureCheckResult {
                            installed,
                            running,
                            services,
                            containers: containers.clone(),
                            checked_at: chrono::Utc::now().timestamp(),
                        });
                        row.operation_state = OperationState::Completed {
                            operation: tr!("ssh-operation-dure-check"),
                            completed_at: chrono::Utc::now().timestamp(),
                        };
                        row.drawer_state.dure_containers = containers.clone();
                        Some((
                            row.last_ssh_check.clone(),
                            row.last_host_check.clone(),
                            row.last_docker_check.clone(),
                            row.last_dure_check.clone(),
                        ))
                    } else {
                        None
                    }
                };

                #[cfg(not(target_arch = "wasm32"))]
                if let Some((ssh, host_check, docker, dure)) = cache_data {
                    self.save_ssh_cache(profile, &host, ssh, host_check, docker, dure);
                }

                #[cfg(target_arch = "wasm32")]
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.dure_installed = installed;
                    row.last_dure_check = Some(DureCheckResult {
                        installed,
                        running,
                        services: services.clone(),
                        containers: containers.clone(),
                        checked_at: chrono::Utc::now().timestamp(),
                    });
                    row.operation_state = OperationState::Completed {
                        operation: tr!("ssh-operation-dure-check"),
                        completed_at: chrono::Utc::now().timestamp(),
                    };
                    row.drawer_state.dure_containers = containers;
                }
            }

            ViewModelEvent::Ssh(SshEvent::DureInstallCompleted {
                host,
                success,
                error,
                progress: _,
                focus_tab,
            }) => {
                // Clone error early for dialog update
                let error_msg = error.unwrap_or_else(|| tr!("ssh-error-unknown"));

                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    // Auto-focus drawer tab if specified
                    if let Some(tab) = focus_tab {
                        row.drawer_state.active_tab = tab;
                    }

                    // Trigger 5-second refresh for log updates
                    self.refresh_until = Some(Instant::now() + Duration::from_secs(5));

                    row.dure_installed = success;
                    row.operation_state = if success {
                        OperationState::Completed {
                            operation: tr!("ssh-operation-install-dure"),
                            completed_at: chrono::Utc::now().timestamp(),
                        }
                    } else {
                        OperationState::Failed {
                            operation: tr!("ssh-operation-install-dure"),
                            error: error_msg.clone(),
                            failed_at: chrono::Utc::now().timestamp(),
                        }
                    };
                }

                // Update install wizard dialog if open for this host
                if let Some(dialog) = &mut self.install_dure_dialog {
                    if dialog.state.host == host {
                        use crate::ui_dlg::ssh::install_dure_stt::InstallStep;
                        if success {
                            dialog.state.step = InstallStep::Complete;
                            dure_info!("[UI::SSH] Updated dialog to Complete state");
                        } else {
                            dialog.state.step = InstallStep::Error(error_msg.clone());
                            dure_info!("[UI::SSH] Updated dialog to Error state");
                        }
                    }
                }

                if success {
                    dure_info!("Dure installed successfully on {}", host);
                } else {
                    dure_error!("Dure installation failed on {}", host);
                }
            }

            ViewModelEvent::Ssh(SshEvent::DureRemoveCompleted {
                host,
                success,
                error,
                focus_tab,
            }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    // Auto-focus drawer tab if specified
                    if let Some(tab) = focus_tab {
                        row.drawer_state.active_tab = tab;
                    }

                    // Trigger 5-second refresh for log updates
                    self.refresh_until = Some(Instant::now() + Duration::from_secs(5));

                    row.dure_installed = !success;
                    row.operation_state = if success {
                        OperationState::Completed {
                            operation: tr!("ssh-operation-remove-dure"),
                            completed_at: chrono::Utc::now().timestamp(),
                        }
                    } else {
                        OperationState::Failed {
                            operation: tr!("ssh-operation-remove-dure"),
                            error: error.unwrap_or_else(|| tr!("ssh-error-unknown")),
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

            ViewModelEvent::Logs(crate::viewmodel::logs::LogEvent::LogsRetrieved {
                filter_id,
                lines,
            }) => {
                // Update drawer logs for matching SSH host
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == filter_id) {
                    row.drawer_state.set_logs(lines.clone());
                } else {
                    dure_error!("✗ No matching row found for filter_id '{}'", filter_id);
                }
            }

            ViewModelEvent::Ssh(SshEvent::HostStatusRetrieved { host, raw_output }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.drawer_state.host_status_raw = Some(raw_output);
                    dure_info!("Host status updated for {}", host);
                }
            }

            ViewModelEvent::Ssh(SshEvent::DockerStatusRawRetrieved { host, raw_output }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.drawer_state.docker_ps_raw = Some(raw_output);
                    dure_info!("Docker status updated for {}", host);
                }
            }

            ViewModelEvent::Ssh(SshEvent::DureContainerLogsRetrieved {
                host,
                container_logs,
            }) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                    row.drawer_state.dure_container_logs = container_logs;
                    dure_info!(
                        "Dure container logs updated for {} ({} containers)",
                        host,
                        row.drawer_state.dure_container_logs.len()
                    );
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
        current_profile_kdbx: &Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
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
        if let Some(ref error) = self.load_error {
            let error_msg = error.clone();
            let mut show_error = true;
            let window_width = ui.ctx().viewport_rect().width();
            let window_height = ui.ctx().viewport_rect().height();

            dialog("ssh_error_dialog", "Error", &mut show_error)
                .min_width(350.0)
                .max_width(window_width - 40.0)
                .max_height(window_height - 40.0)
                .scrollable(true)
                .content(|ui, _open| {
                    ui.label(&error_msg);
                })
                .primary_action("OK", || {})
                .show(ui.ctx());

            if !show_error {
                self.load_error = None;
                return;
            }
        }
        if self.load_error.is_some() {
            return;
        }

        // SSH host table (header is inside render_table)
        self.render_table(ui, profile, vm.as_deref_mut(), current_profile_kdbx);

        // Add host dialog
        self.render_add_dialog(ui, profile);

        // Edit host dialog
        self.render_edit_dialog(ui, profile);

        // Install Dure dialog
        self.render_install_dure_dialog(ui, profile);

        // Install Dure wizard dialog
        if let Some(dialog) = &mut self.install_dure_dialog {
            dialog.show(ui.ctx(), vm.as_deref_mut());
            if dialog.should_close() {
                self.install_dure_dialog = None;
            }
        }

        // Remove Dure dialog
        self.render_remove_dure_dialog(ui, profile);
    }

    /// Render SSH hosts table with drawer
    fn render_table(
        &mut self,
        ui: &mut egui::Ui,
        profile: &Option<crate::calc::profile::ProfileContext>,
        vm: Option<&mut crate::viewmodel::ViewModel>,
        current_profile_kdbx: &Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
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
            .column(&tr!("ssh-drawer-tab-host"), 230.0 * width_ratio, false)
            .column(
                &tr!("ssh-drawer-tab-operations"),
                510.0 * width_ratio,
                false,
            );

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
                    ssh_drawer::render_drawer(
                        ui,
                        &row_for_drawer,
                        &row_for_drawer.drawer_state,
                        &mut tab_switch,
                    );

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
            ui.heading(tr!("ssh-main-heading"));
            ui.add_space(4.0);
            ui.label(tr!("ssh-main-desc"));
            ui.add_space(8.0);

            if ui
                .add(MaterialButton::filled(tr!("ssh-button-add")))
                .clicked()
            {
                self.show_add_dialog = true;
            }
            ui.add_space(8.0);

            table.show(ui);
        });

        // Handle pending actions
        if let Some(action) =
            ui.data(|d| d.get_temp::<SshAction>(egui::Id::new("ssh_pending_action")))
        {
            ui.data_mut(|d| d.remove::<SshAction>(egui::Id::new("ssh_pending_action")));

            match action {
                SshAction::Refresh(host) => {
                    // Set operation state to InProgress before starting
                    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                        row.operation_state = OperationState::start(tr!("ssh-operation-refresh"));
                    }
                    if let Some(ref vm) = vm {
                        let _ = vm.refresh_ssh_host(host.clone(), (*current_profile_kdbx).clone());
                        dure_info!(project_id = host, "Refresh triggered for {}", host);
                    }
                }
                SshAction::SshCheck(host) => {
                    // Set operation state to InProgress before starting
                    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                        row.operation_state = OperationState::start(tr!("ssh-operation-ssh-check"));
                    }
                    if let Some(ref vm) = vm {
                        let _ = vm.ssh_check(host.clone(), (*current_profile_kdbx).clone());
                        dure_info!(project_id = host, "SSH check triggered for {}", host);
                    }
                }
                SshAction::Edit(host) => {
                    self.start_edit_host(&host, profile);
                }
                SshAction::Delete(host) => {
                    self.delete_host(&host, profile);
                }
                SshAction::CheckHost(host) => {
                    // Set operation state to InProgress before starting
                    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                        row.operation_state =
                            OperationState::start(tr!("ssh-operation-host-check"));
                    }
                    if let Some(ref vm) = vm {
                        let _ = vm.check_host(host.clone(), (*current_profile_kdbx).clone());
                        dure_info!(project_id = host, "Host check triggered for {}", host);
                    }
                }
                SshAction::InstallHost(host) => {
                    // Set operation state to InProgress before starting
                    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                        row.operation_state =
                            OperationState::start(tr!("ssh-operation-install-host"));
                    }
                    if let Some(ref vm) = vm {
                        let _ = vm.install_host(host.clone(), (*current_profile_kdbx).clone());
                        dure_info!(
                            project_id = host,
                            "Host installation triggered for {}",
                            host
                        );
                    }
                }
                SshAction::CheckDocker(host) => {
                    // Set operation state to InProgress before starting
                    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                        row.operation_state =
                            OperationState::start(tr!("ssh-operation-docker-check"));
                    }
                    if let Some(ref vm) = vm {
                        let _ = vm.check_docker(host.clone(), (*current_profile_kdbx).clone());
                        dure_info!(project_id = host, "Docker check triggered for {}", host);
                    }
                }
                SshAction::InstallDocker(host) => {
                    // Set operation state to InProgress before starting
                    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                        row.operation_state =
                            OperationState::start(tr!("ssh-operation-install-docker"));
                    }
                    if let Some(ref vm) = vm {
                        let _ =
                            vm.install_docker_daemon(host.clone(), (*current_profile_kdbx).clone());
                        dure_info!(
                            project_id = host,
                            "Docker installation triggered for {}",
                            host
                        );
                    }
                }
                SshAction::RemoveDocker(host) => {
                    // Set operation state to InProgress before starting
                    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                        row.operation_state =
                            OperationState::start(tr!("ssh-operation-remove-docker"));
                    }
                    if let Some(ref vm) = vm {
                        let _ = vm.remove_docker(host.clone(), (*current_profile_kdbx).clone());
                        dure_info!(project_id = host, "Docker removal triggered for {}", host);
                    }
                }
                SshAction::CheckDure(host) => {
                    // Set operation state to InProgress before starting
                    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                        row.operation_state =
                            OperationState::start(tr!("ssh-operation-dure-check"));
                    }
                    if let Some(ref vm) = vm {
                        let _ = vm.check_dure(host.clone(), (*current_profile_kdbx).clone());
                        dure_info!(project_id = host, "Dure check triggered for {}", host);
                    }
                }
                SshAction::InstallDure(host) => {
                    // Look up platform domain for this host
                    dure_debug!("[SSH] Looking up platform domain for host: {}", host);

                    let row = self.rows.iter().find(|r| r.host == host);
                    if let Some(row) = row {
                        dure_debug!("[SSH] Found row, platform_id: {:?}", row.platform_id);
                    } else {
                        dure_warn!("[SSH] No row found for host: {}", host);
                    }

                    let platform_domain = self.rows.iter()
                        .find(|r| r.host == host)
                        .and_then(|row| {
                            let platform_id = row.platform_id.clone();
                            if platform_id.is_none() {
                                dure_warn!("[SSH] Host {} has no platform_id", host);
                            }
                            platform_id
                        })
                        .and_then(|platform_id| {
                            dure_debug!("[SSH] Looking for domain with platform_id: {}", platform_id);
                            // Load config and find domain by project ID
                            profile.as_ref().and_then(|p| {
                                let config = AppConfig::load_or_default(&p.config_file);
                                dure_debug!("[SSH] Config has {} platforms, {} dns_domains",
                                    config.platforms.len(), config.dns_domains.len());

                                // Try 1: Check platforms[].bound_domain
                                let found_platform = config.platforms.iter()
                                    .find(|plat| {
                                        // Check GCP project ID
                                        if let Some(ref gcp_id) = plat.gcp_selected_project_id {
                                            gcp_id == &platform_id
                                        } else {
                                            false
                                        }
                                    });

                                if let Some(plat) = found_platform {
                                    if let Some(ref domain) = plat.bound_domain {
                                        dure_info!("[SSH] Found domain from platform.bound_domain: {}", domain);
                                        return Some(domain.clone());
                                    }
                                }

                                // Try 2: Check dns_domains[] where platform_id matches
                                let found_dns_domain = config.dns_domains.iter()
                                    .find(|dns| {
                                        dns.platform_id.as_ref() == Some(&platform_id)
                                    });

                                if let Some(dns) = found_dns_domain {
                                    dure_info!("[SSH] Found domain from dns_domains: {}", dns.domain);
                                    return Some(dns.domain.clone());
                                }

                                dure_warn!("[SSH] No domain found for platform_id: {} (checked platforms.bound_domain and dns_domains)", platform_id);
                                None
                            })
                        });

                    // Open new wizard dialog
                    let mut dlg = crate::ui_dlg::ssh::install_dure::InstallDure::new(host.clone())
                        .with_domain(platform_domain.clone())
                        .with_keyring((*current_profile_kdbx).clone());

                    // Get GCP project ID and access token from platform config
                    #[cfg(not(target_arch = "wasm32"))]
                    if let Some(ref p) = profile {
                        let config = AppConfig::load_or_default(&p.config_file);
                        // Find host config to get platform_id
                        if let Some(host_config) = config.ssh_hosts.iter().find(|h| h.host == host) {
                            if let Some(ref platform_id) = host_config.platform_id {
                                // Find platform config
                                if let Some(platform) = config.platforms.iter().find(|plat| {
                                    plat.gcp_selected_project_id.as_ref() == Some(platform_id)
                                }) {
                                    dlg.state.gcp_project_id = platform.gcp_selected_project_id.clone();
                                    dlg.state.gcp_access_token = platform.gcp_oauth_access_token.clone();
                                }
                            }
                        }
                    }

                    self.install_dure_dialog = Some(dlg);

                    if let Some(domain) = platform_domain {
                        dure_info!("[SSH] Opening install Dure wizard for {} (domain: {})", host, domain);
                    } else {
                        dure_info!("[SSH] Opening install Dure wizard for {} (no platform domain)", host);
                    }
                }
                SshAction::RemoveDure(host) => {
                    // Open remove dialog
                    self.remove_dure_host = host.clone();
                    self.show_remove_dure_dialog = true;
                    dure_info!("Opening remove Dure dialog for {}", host);
                }
            }
        }

        // Handle pending install Dure action
        if let Some((host, env_config)) = ui.data(|d| {
            d.get_temp::<(String, std::collections::HashMap<String, String>)>(egui::Id::new(
                "ssh_install_dure_pending",
            ))
        }) {
            ui.data_mut(|d| {
                d.remove::<(String, std::collections::HashMap<String, String>)>(egui::Id::new(
                    "ssh_install_dure_pending",
                ))
            });

            if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                row.operation_state = OperationState::start("Install Dure");
            }
            if let Some(ref vm) = vm {
                // Legacy code path - use defaults for version and image_tag
                // TODO: Remove this once old dialog is fully replaced by wizard
                // Convert HashMap to Vec<EnvField> (no comments in legacy path)
                let env_fields: Vec<crate::calc::env_parser::EnvField> = env_config
                    .into_iter()
                    .map(|(k, v)| crate::calc::env_parser::EnvField::new(k, v))
                    .collect();

                let _ = vm.install_dure(
                    host,
                    "main".to_string(),  // Default version
                    env_fields,
                    "latest".to_string(),  // Default image tag
                    (*current_profile_kdbx).clone(),
                );
            }
        }

        // Handle pending remove Dure action
        if let Some((host, delete_data)) =
            ui.data(|d| d.get_temp::<(String, bool)>(egui::Id::new("ssh_remove_dure_pending")))
        {
            ui.data_mut(|d| d.remove::<(String, bool)>(egui::Id::new("ssh_remove_dure_pending")));

            if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                row.operation_state = OperationState::start("Remove Dure");
            }
            if let Some(ref vm) = vm {
                let _ = vm.remove_dure(host, delete_data, (*current_profile_kdbx).clone());
            }
        }

        // Handle drawer tab switch
        if let Some((host, new_tab)) = ui.data(|d| {
            self.rows.iter().find_map(|r| {
                d.get_temp::<(String, DrawerTab)>(
                    egui::Id::new("ssh_drawer_tab_switch").with(&r.host),
                )
            })
        }) {
            if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                row.drawer_state.switch_tab(new_tab);

                // Load data when switching to Logs or Operations tabs
                match new_tab {
                    DrawerTab::Logs => {
                        // Request logs from log system
                        if let Some(ref vm) = vm {
                            if let Err(e) = vm.get_ssh_logs(host.clone()) {
                                dure_error!("✗ Failed to send GetLogs request: {}", e);
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
                    d.remove::<(String, DrawerTab)>(
                        egui::Id::new("ssh_drawer_tab_switch").with(&host),
                    );
                });
            }
        }

        // Handle dure subtab selection
        if let Some((host, new_subtab)) = ui.data(|d| {
            self.rows.iter().find_map(|r| {
                d.get_temp::<(String, usize)>(egui::Id::new("dure_subtab_selection").with(&r.host))
            })
        }) {
            if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                row.drawer_state.dure_active_subtab = new_subtab;
                row.drawer_state.dure_subtab_should_focus = true;
            }
            ui.data_mut(|d| {
                d.remove::<(String, usize)>(egui::Id::new("dure_subtab_selection").with(&host));
            });
        }

        // Clear dure subtab focus flag after applied
        if let Some(host) = ui.data(|d| {
            self.rows.iter().find_map(|r| {
                d.get_temp::<String>(egui::Id::new("dure_subtab_focus_applied").with(&r.host))
            })
        }) {
            if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                row.drawer_state.dure_subtab_should_focus = false;
            }
            ui.data_mut(|d| {
                d.remove::<String>(egui::Id::new("dure_subtab_focus_applied").with(&host));
            });
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

        use std::cell::Cell;

        // Clone data for dialog
        let mut add_host = self.add_host.clone();
        let mut add_port = self.add_port.clone();
        let mut add_use_password = self.add_use_password;
        let mut add_password = self.add_password.clone();
        let mut add_use_private_key = self.add_use_private_key;
        let mut add_private_key_path = self.add_private_key_path.clone();

        let add_requested = Cell::new(false);
        let reset_requested = Cell::new(false);

        let window_width = ui.ctx().viewport_rect().width();
        let window_height = ui.ctx().viewport_rect().height();

        dialog(
            "add_ssh_host_dialog",
            &tr!("ssh-dialog-add-title"),
            &mut self.show_add_dialog,
        )
        .min_width(350.0)
        .max_width(window_width - 40.0)
        .max_height(window_height - 40.0)
        .scrollable(true)
        .content(|ui, open| {
            ui.label(tr!("ssh-dialog-add-heading"));
            ui.add_space(8.0);

            ui.label(tr!("ssh-dialog-add-host-label"));
            ui.text_edit_singleline(&mut add_host);
            ui.add_space(8.0);

            ui.label(tr!("ssh-dialog-add-port-label"));
            ui.text_edit_singleline(&mut add_port);
            ui.add_space(8.0);

            ui.checkbox(&mut add_use_password, &tr!("ssh-dialog-add-use-password"));
            if add_use_password {
                ui.label(tr!("ssh-dialog-add-password-label"));
                ui.add(egui::TextEdit::singleline(&mut add_password).password(true));
            }
            ui.add_space(8.0);

            ui.checkbox(&mut add_use_private_key, &tr!("ssh-dialog-add-use-key"));
            if add_use_private_key {
                ui.label(tr!("ssh-dialog-add-key-path-label"));
                ui.text_edit_singleline(&mut add_private_key_path);
            }
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(MaterialButton::filled(tr!("button-add"))).clicked() {
                    add_requested.set(true);
                    *open = false;
                }
                if ui
                    .add(MaterialButton::outlined(tr!("button-cancel")))
                    .clicked()
                {
                    reset_requested.set(true);
                    *open = false;
                }
            });
        })
        .show(ui.ctx());

        // Update state from dialog
        self.add_host = add_host;
        self.add_port = add_port;
        self.add_use_password = add_use_password;
        self.add_password = add_password;
        self.add_use_private_key = add_use_private_key;
        self.add_private_key_path = add_private_key_path;

        if add_requested.get() {
            self.add_host_action(profile);
            self.reset_add_dialog();
        }

        if reset_requested.get() {
            self.reset_add_dialog();
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
                    self.load_error = Some(
                        tr!("ssh-error-invalid-port", {port: self.add_port.clone(), error: e.to_string()}),
                    );
                    return;
                }
            };

            match load_config(profile) {
                Ok((mut app_config, config_path)) => {
                    // Check for duplicates
                    if app_config.ssh_hosts.iter().any(|h| h.host == self.add_host) {
                        self.load_error =
                            Some(tr!("ssh-error-host-exists", {host: self.add_host.clone()}));
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
                        platform_id: None,
                        docker_containers: Vec::new(),
                        ansible_roles: Vec::new(),
                        dure_wss_config: None,
                        last_ssh_check: None,
                        last_host_check: None,
                        last_docker_check: None,
                        last_dure_check: None,
                    };

                    app_config.ssh_hosts.push(host_config);

                    // Save config
                    if let Err(e) = app_config.save(&config_path) {
                        self.load_error =
                            Some(tr!("ssh-error-save-config", {error: e.to_string()}));
                        return;
                    }

                    dure_info!("SSH host '{}' added successfully", self.add_host);

                    // Reload rows
                    self.loaded = false;
                    self.show_add_dialog = false;
                    self.reset_add_dialog();
                }
                Err(e) => {
                    self.load_error = Some(tr!("ssh-error-load-config", {error: e.to_string()}));
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
    fn start_edit_host(
        &mut self,
        host: &str,
        profile: &Option<crate::calc::profile::ProfileContext>,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match load_config(profile) {
                Ok((app_config, _)) => {
                    if let Some(host_config) = app_config.ssh_hosts.iter().find(|h| h.host == host)
                    {
                        self.edit_original_host = host.to_string();
                        self.edit_host = host_config.host.clone();
                        self.edit_port = host_config.port.to_string();
                        self.edit_use_password = host_config.password.is_some();
                        self.edit_password = host_config.password.clone().unwrap_or_default();
                        self.edit_use_private_key = host_config.private_key_path.is_some();
                        self.edit_private_key_path =
                            host_config.private_key_path.clone().unwrap_or_default();
                        self.show_edit_dialog = true;
                        dure_info!("Started editing host {}", host);
                    }
                }
                Err(e) => {
                    self.load_error = Some(tr!("ssh-error-load-config", {error: e.to_string()}));
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

        use std::cell::Cell;

        // Clone data for dialog
        let mut edit_host = self.edit_host.clone();
        let mut edit_port = self.edit_port.clone();
        let mut edit_use_password = self.edit_use_password;
        let mut edit_password = self.edit_password.clone();
        let mut edit_use_private_key = self.edit_use_private_key;
        let mut edit_private_key_path = self.edit_private_key_path.clone();

        let save_requested = Cell::new(false);
        let reset_requested = Cell::new(false);

        let window_width = ui.ctx().viewport_rect().width();
        let window_height = ui.ctx().viewport_rect().height();

        dialog(
            "edit_ssh_host_dialog",
            &tr!("ssh-dialog-edit-title"),
            &mut self.show_edit_dialog,
        )
        .min_width(350.0)
        .max_width(window_width - 40.0)
        .max_height(window_height - 40.0)
        .scrollable(true)
        .content(|ui, open| {
            ui.label(tr!("ssh-dialog-edit-heading"));
            ui.add_space(8.0);

            ui.label(tr!("ssh-dialog-add-host-label"));
            ui.text_edit_singleline(&mut edit_host);
            ui.add_space(8.0);

            ui.label(tr!("ssh-dialog-add-port-label"));
            ui.text_edit_singleline(&mut edit_port);
            ui.add_space(8.0);

            ui.checkbox(&mut edit_use_password, &tr!("ssh-dialog-add-use-password"));
            if edit_use_password {
                ui.label(tr!("ssh-dialog-add-password-label"));
                ui.add(egui::TextEdit::singleline(&mut edit_password).password(true));
            }
            ui.add_space(8.0);

            ui.checkbox(&mut edit_use_private_key, &tr!("ssh-dialog-add-use-key"));
            if edit_use_private_key {
                ui.label(tr!("ssh-dialog-add-key-path-label"));
                ui.text_edit_singleline(&mut edit_private_key_path);
            }
            ui.add_space(12.0);
            ui.separator();
            ui.add_space(8.0);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(MaterialButton::filled(tr!("button-save"))).clicked() {
                    save_requested.set(true);
                    *open = false;
                }
                if ui
                    .add(MaterialButton::outlined(tr!("button-cancel")))
                    .clicked()
                {
                    reset_requested.set(true);
                    *open = false;
                }
            });
        })
        .show(ui.ctx());

        // Update state from dialog
        self.edit_host = edit_host;
        self.edit_port = edit_port;
        self.edit_use_password = edit_use_password;
        self.edit_password = edit_password;
        self.edit_use_private_key = edit_use_private_key;
        self.edit_private_key_path = edit_private_key_path;

        if save_requested.get() {
            self.save_edit_host(profile);
            self.reset_edit_dialog();
        }

        if reset_requested.get() {
            self.reset_edit_dialog();
        }
    }

    /// Save edited SSH host
    fn save_edit_host(&mut self, profile: &Option<crate::calc::profile::ProfileContext>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let port = match self.edit_port.parse::<u16>() {
                Ok(p) => p,
                Err(e) => {
                    self.load_error = Some(
                        tr!("ssh-error-invalid-port", {port: self.edit_port.clone(), error: e.to_string()}),
                    );
                    return;
                }
            };

            match load_config(profile) {
                Ok((mut app_config, config_path)) => {
                    if let Some(host_config) = app_config
                        .ssh_hosts
                        .iter_mut()
                        .find(|h| h.host == self.edit_original_host)
                    {
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
                            self.load_error =
                                Some(tr!("ssh-error-save-config", {error: e.to_string()}));
                            return;
                        }

                        self.show_edit_dialog = false;
                        self.reset_edit_dialog();
                        self.loaded = false;
                        dure_info!("Saved changes to SSH host {}", self.edit_host);
                    }
                }
                Err(e) => {
                    self.load_error = Some(tr!("ssh-error-load-config", {error: e.to_string()}));
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
                        self.load_error =
                            Some(tr!("ssh-error-save-config", {error: e.to_string()}));
                        return;
                    }

                    dure_info!("SSH host '{}' deleted successfully", host);

                    self.loaded = false;
                }
                Err(e) => {
                    self.load_error = Some(tr!("ssh-error-load-config", {error: e.to_string()}));
                }
            }
        }
    }

    /// Render Remove Dure dialog
    fn render_remove_dure_dialog(
        &mut self,
        ui: &mut egui::Ui,
        profile: &Option<crate::calc::profile::ProfileContext>,
    ) {
        if !self.show_remove_dure_dialog {
            return;
        }

        use std::cell::Cell;

        let mut delete_data = self.remove_dure_delete_data;
        let remove_requested = Cell::new(false);
        let cancel_requested = Cell::new(false);

        let window_width = ui.ctx().viewport_rect().width();
        let window_height = ui.ctx().viewport_rect().height();

        dialog(
            "remove_dure_dialog",
            "Remove Dure",
            &mut self.show_remove_dure_dialog,
        )
        .min_width(350.0)
        .max_width(window_width - 40.0)
        .max_height(window_height - 40.0)
        .scrollable(true)
        .content(|ui, open| {
            ui.label(format!("Remove Dure from host: {}", self.remove_dure_host));
            ui.add_space(12.0);

            ui.checkbox(
                &mut delete_data,
                "Delete data directories (/srv/dure-mycart and /srv/data)",
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Warning: Deleting data cannot be undone!")
                    .color(egui::Color32::from_rgb(211, 47, 47)),
            );
            ui.add_space(12.0);

            ui.separator();
            ui.add_space(8.0);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(MaterialButton::filled("Remove")).clicked() {
                    remove_requested.set(true);
                    *open = false;
                }
                if ui
                    .add(MaterialButton::outlined(tr!("button-cancel")))
                    .clicked()
                {
                    cancel_requested.set(true);
                    *open = false;
                }
            });
        })
        .show(ui.ctx());

        self.remove_dure_delete_data = delete_data;

        if remove_requested.get() {
            // Note: Actual removal will be handled via UI action trigger
            // since we need vm reference which is not available here
            // Store state for the action handler to pick up
            ui.data_mut(|d| {
                d.insert_temp(
                    egui::Id::new("ssh_remove_dure_pending"),
                    (self.remove_dure_host.clone(), self.remove_dure_delete_data),
                );
            });

            self.reset_remove_dure_dialog();
        }

        if cancel_requested.get() {
            self.reset_remove_dure_dialog();
        }
    }

    /// Reset remove Dure dialog state
    fn reset_remove_dure_dialog(&mut self) {
        self.remove_dure_host = String::new();
        self.remove_dure_delete_data = false;
    }

    /// Render Install Dure dialog
    fn render_install_dure_dialog(
        &mut self,
        ui: &mut egui::Ui,
        profile: &Option<crate::calc::profile::ProfileContext>,
    ) {
        if !self.show_install_dure_dialog {
            return;
        }

        use std::cell::Cell;

        let mut domain = self.install_dure_domain.clone();
        let mut image_tag = self.install_dure_image_tag.clone();
        let install_requested = Cell::new(false);
        let cancel_requested = Cell::new(false);

        let window_width = ui.ctx().viewport_rect().width();
        let window_height = ui.ctx().viewport_rect().height();

        dialog(
            "install_dure_dialog",
            "Install Dure",
            &mut self.show_install_dure_dialog,
        )
        .min_width(350.0)
        .max_width(window_width - 40.0)
        .max_height(window_height - 40.0)
        .scrollable(true)
        .content(|ui, open| {
            ui.label(format!("Install Dure on host: {}", self.install_dure_host));
            ui.add_space(12.0);

            ui.label("Domain (required):");
            ui.text_edit_singleline(&mut domain);
            if domain.is_empty() {
                ui.colored_label(egui::Color32::from_rgb(211, 47, 47), "⚠ Domain is required");
            }
            ui.add_space(8.0);

            ui.label("Image Tag:");
            ui.horizontal(|ui| {
                ui.radio_value(&mut image_tag, "latest".to_string(), "latest (production)");
                ui.radio_value(&mut image_tag, "test".to_string(), "test (development)");
            });
            ui.add_space(12.0);

            ui.separator();
            ui.add_space(8.0);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.add(MaterialButton::filled("Install")).clicked() {
                    install_requested.set(true);
                    *open = false;
                }
                if ui
                    .add(MaterialButton::outlined(tr!("button-cancel")))
                    .clicked()
                {
                    cancel_requested.set(true);
                    *open = false;
                }
            });
        })
        .show(ui.ctx());

        self.install_dure_domain = domain;
        self.install_dure_image_tag = image_tag;

        if install_requested.get() {
            if self.install_dure_domain.is_empty() {
                self.load_error = Some("Domain is required".to_string());
                return;
            }

            // Build env_config
            use std::collections::HashMap;
            let mut env_config = HashMap::new();
            env_config.insert("DOMAIN".to_string(), self.install_dure_domain.clone());
            env_config.insert(
                "MYCART_IMAGE_TAG".to_string(),
                self.install_dure_image_tag.clone(),
            );

            // Extract username and hostname from SSH host (format: user@host)
            let (username, hostname) = if let Some(at_pos) = self.install_dure_host.rfind('@') {
                (
                    self.install_dure_host[..at_pos].to_string(),
                    self.install_dure_host[at_pos + 1..].to_string(),
                )
            } else {
                ("root".to_string(), self.install_dure_host.clone())
            };

            env_config.insert("username".to_string(), username);
            env_config.insert("host".to_string(), hostname);

            // Note: Actual installation will be handled via UI action trigger
            // Store state for the action handler to pick up
            ui.data_mut(|d| {
                d.insert_temp(
                    egui::Id::new("ssh_install_dure_pending"),
                    (self.install_dure_host.clone(), env_config),
                );
            });

            self.reset_install_dure_dialog();
        }

        if cancel_requested.get() {
            self.reset_install_dure_dialog();
        }
    }

    /// Reset install Dure dialog state
    fn reset_install_dure_dialog(&mut self) {
        self.install_dure_host = String::new();
        self.install_dure_domain = String::new();
        self.install_dure_image_tag = "latest".to_string();
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
            OperationState::Failed {
                operation, error, ..
            } => {
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
    fn test_host_check_result_creation() {
        let result = HostCheckResult {
            installed: true,
            missing_packages: vec![],
            checked_at: 1234567890,
        };

        assert!(result.installed);
        assert!(result.missing_packages.is_empty());
        assert_eq!(result.checked_at, 1234567890);
    }

    #[test]
    fn test_host_check_result_with_missing_packages() {
        let result = HostCheckResult {
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
            containers: Vec::new(),
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
            containers: Vec::new(),
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
            host_installed: false,
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
        assert!(row.last_host_check.is_none());
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
            host_installed: false,
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
        assert!(row.last_host_check.is_none());
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

    #[test]
    fn test_operation_starts_with_in_progress_state() {
        // When operation button clicked, state should be InProgress
        let state = OperationState::start("Refresh");

        match state {
            OperationState::InProgress { operation, .. } => {
                assert_eq!(operation, "Refresh");
            }
            _ => panic!("Expected InProgress state when operation starts"),
        }
    }

    #[test]
    fn test_multiple_operations_tracked_by_name() {
        // Different operations should be distinguishable by name
        let refresh = OperationState::start("Refresh");
        let ssh_check = OperationState::start("SSH Check");

        match (&refresh, &ssh_check) {
            (
                OperationState::InProgress { operation: op1, .. },
                OperationState::InProgress { operation: op2, .. },
            ) => {
                assert_eq!(op1, "Refresh");
                assert_eq!(op2, "SSH Check");
                assert_ne!(op1, op2);
            }
            _ => panic!("Both should be InProgress"),
        }
    }

    #[test]
    fn test_operation_state_has_timestamp() {
        // InProgress state should include start timestamp
        let state = OperationState::start("Docker Install");

        match state {
            OperationState::InProgress { started_at, .. } => {
                // Timestamp should be recent (within last second)
                let now = chrono::Utc::now().timestamp();
                assert!(started_at <= now && started_at > now - 2);
            }
            _ => panic!("Expected InProgress state"),
        }
    }
}
