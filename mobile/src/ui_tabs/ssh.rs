//! SSH tab - SSH host configuration and management with drawer

use crate::{dure_debug, dure_error, dure_info};
use eframe::egui;
use egui_material3::MaterialButton;

use crate::config::{AppConfig, SshHostConfig};
use crate::viewmodel::ssh::{DrawerState, DrawerTab};

// Button spacing matching Platform tab
const BUTTON_VERTICAL_SPACING: f32 = 4.0;
const BUTTON_HORIZONTAL_SPACING: f32 = 4.0;

/// SSH row data for data table
#[derive(Clone, Debug)]
pub struct SshRow {
    // Identity
    pub host: String, // IP address (row key)
    pub port: u16,

    // Platform relationship
    pub platform_id: Option<String>, // GCP project_id if connected to platform

    // Connection state
    pub ssh_connected: bool,

    // Service status flags (cached, requires Refresh)
    pub base_installed: bool,     // Base packages installed
    pub docker_installed: bool,
    pub dure_installed: bool,

    // SSH key for copy action
    pub ssh_private_key: Option<String>,

    // Drawer state
    pub drawer_open: bool,
    pub drawer_state: DrawerState,
}

/// Actions that can be triggered from SSH table rows
#[derive(Debug, Clone)]
enum SshAction {
    Refresh(String),       // host - full status refresh
    SshCheck(String),      // host - SSH connection check only
    Edit(String),          // host - open edit dialog
    Delete(String),        // host
    CheckBase(String),     // host - check base packages
    InstallBase(String),   // host - install base packages
    CheckDocker(String),   // host - check docker installed
    InstallDocker(String), // host - install docker
    RemoveDocker(String),  // host - remove docker
    CheckDure(String),     // host - check dure installed
    InstallDure(String),   // host - install dure (with .env dialog)
    RemoveDure(String),    // host - remove dure
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

            if ui.button("Refresh").clicked() {
                *action_trigger = Some(SshAction::Refresh(row.host.clone()));
            }

            if ui.button("SSH Check").clicked() {
                *action_trigger = Some(SshAction::SshCheck(row.host.clone()));
            }

            if ui.button("Edit").clicked() {
                *action_trigger = Some(SshAction::Edit(row.host.clone()));
            }

            if ui.button("Delete").clicked() {
                *action_trigger = Some(SshAction::Delete(row.host.clone()));
            }
        });

        // Row 2: Base and Docker
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = BUTTON_HORIZONTAL_SPACING;

            // Check Base (always)
            if ui.button("Check Base").clicked() {
                *action_trigger = Some(SshAction::CheckBase(row.host.clone()));
            }

            // Install Base (only when NOT installed)
            if !row.base_installed {
                if ui.button("Install Base").clicked() {
                    *action_trigger = Some(SshAction::InstallBase(row.host.clone()));
                }
            }

            // Check Docker (always)
            if ui.button("Check Docker").clicked() {
                *action_trigger = Some(SshAction::CheckDocker(row.host.clone()));
            }

            // Install/Remove Docker (conditional)
            if !row.docker_installed {
                if ui.button("Install Docker").clicked() {
                    *action_trigger = Some(SshAction::InstallDocker(row.host.clone()));
                }
            } else {
                if ui.button("Remove Docker").clicked() {
                    *action_trigger = Some(SshAction::RemoveDocker(row.host.clone()));
                }
            }
        });

        // Row 3: Dure operations
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = BUTTON_HORIZONTAL_SPACING;

            // Check Dure (always)
            if ui.button("Check Dure").clicked() {
                *action_trigger = Some(SshAction::CheckDure(row.host.clone()));
            }

            // Install/Remove Dure (conditional)
            if !row.dure_installed {
                if ui.button("Install Dure").clicked() {
                    *action_trigger = Some(SshAction::InstallDure(row.host.clone()));
                }
            } else {
                if ui.button("Remove Dure").clicked() {
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
            ui.add(badge(platform_id).color(BadgeColor::Primary).size(BadgeSize::Small));
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

    // Add/Edit host dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_add_dialog: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    edit_mode: bool, // true = edit existing, false = add new

    #[cfg_attr(feature = "serde", serde(skip))]
    edit_original_host: String, // Original host when editing

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

    // Install Dure dialog (for .env configuration)
    #[cfg_attr(feature = "serde", serde(skip))]
    show_install_dure_dialog: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    install_dure_host: String,

    #[cfg_attr(feature = "serde", serde(skip))]
    install_dure_env_content: String, // .env file content
}

impl Default for SshTab {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            loaded: false,
            load_error: None,
            config_last_modified: None,
            show_add_dialog: false,
            edit_mode: false,
            edit_original_host: String::new(),
            add_host: String::new(),
            add_password: String::new(),
            add_private_key_path: String::new(),
            add_port: "22".to_string(),
            add_use_password: false,
            add_use_private_key: false,
            show_install_dure_dialog: false,
            install_dure_host: String::new(),
            install_dure_env_content: String::new(),
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

                        // Look up platform_id from platform_name (matches gcp_selected_project_id)
                        let platform_id = host_config.platform_name.as_ref().and_then(|name| {
                            app_config.platforms.iter()
                                .find(|cfg| cfg.gcp_selected_project_id.as_deref() == Some(name.as_str()))
                                .and_then(|cfg| cfg.gcp_selected_project_id.clone())
                        });

                        self.rows.push(SshRow {
                            host: host_config.host.clone(),
                            port: host_config.port,
                            platform_id,
                            ssh_connected: false, // TODO: Determine from actual connection state
                            base_installed: false, // TODO: Check from status cache
                            docker_installed: !host_config.docker_containers.is_empty(),
                            dure_installed: host_config.dure_wss_config.is_some(),
                            ssh_private_key: None, // TODO: Load from keyring
                            drawer_open: false,
                            drawer_state,
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

            // TODO: Handle other events for drawer state updates
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
        self.render_table(ui, profile);

        // Add/Edit host dialog
        self.render_add_dialog(ui, profile);

        // Install Dure dialog (.env configuration)
        self.render_install_dure_dialog(ui, profile);
    }

    /// Render SSH hosts table with drawer
    fn render_table(
        &mut self,
        ui: &mut egui::Ui,
        profile: &Option<crate::calc::profile::ProfileContext>,
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
            .min_row_height(100.0)  // Increased for 3 rows of buttons
            .drawer_row_height(500.0)
            .column("Host", 230.0 * width_ratio, false)
            .column("Operations", 510.0 * width_ratio, false);

        for row in self.rows.iter() {
            let row_for_cells = row.clone();
            let row_for_drawer = row.clone();
            let row_for_actions = row.clone();

            table = table.row(move |r| {
                r.cell_widget(move |ui| {
                        use egui_material3::{badge, BadgeColor, BadgeSize};

                        ui.horizontal(|ui| {
                            ui.label(&row_for_cells.host);

                            // Show platform badge if connected
                            if let Some(ref platform_id) = row_for_cells.platform_id {
                                ui.add_space(4.0);
                                ui.add(badge(platform_id).color(BadgeColor::Primary).size(BadgeSize::Small));
                            }
                        });
                    })
                    .cell_widget(move |ui| {
                        ui.vertical(|ui| {
                            // Row 1: Basic operations
                            ui.horizontal(|ui| {
                                if ui.add(MaterialButton::outlined("Refresh").small()).clicked() {
                                    ui.data_mut(|d| {
                                        d.insert_temp(egui::Id::new("ssh_action_refresh"), row_for_actions.host.clone())
                                    });
                                }
                                if ui.add(MaterialButton::outlined("SSH Check").small()).clicked() {
                                    ui.data_mut(|d| {
                                        d.insert_temp(egui::Id::new("ssh_action_ssh_check"), row_for_actions.host.clone())
                                    });
                                }
                                if ui.add(MaterialButton::outlined("Edit").small()).clicked() {
                                    ui.data_mut(|d| {
                                        d.insert_temp(egui::Id::new("ssh_action_edit"), row_for_actions.host.clone())
                                    });
                                }
                                if ui.add(MaterialButton::outlined("Delete").small()).clicked() {
                                    ui.data_mut(|d| {
                                        d.insert_temp(egui::Id::new("ssh_action_delete"), row_for_actions.host.clone())
                                    });
                                }
                            });

                            ui.add_space(2.0);

                            // Row 2: Base and Docker operations
                            ui.horizontal(|ui| {
                                if !row_for_actions.base_installed {
                                    if ui.add(MaterialButton::outlined("Check Base").small()).clicked() {
                                        ui.data_mut(|d| {
                                            d.insert_temp(egui::Id::new("ssh_action_check_base"), row_for_actions.host.clone())
                                        });
                                    }
                                    if ui.add(MaterialButton::outlined("Install Base").small()).clicked() {
                                        ui.data_mut(|d| {
                                            d.insert_temp(egui::Id::new("ssh_action_install_base"), row_for_actions.host.clone())
                                        });
                                    }
                                } else {
                                    ui.add_enabled(false, MaterialButton::outlined("Base OK").small());
                                }

                                if !row_for_actions.docker_installed {
                                    if ui.add(MaterialButton::outlined("Check Docker").small()).clicked() {
                                        ui.data_mut(|d| {
                                            d.insert_temp(egui::Id::new("ssh_action_check_docker"), row_for_actions.host.clone())
                                        });
                                    }
                                    if ui.add(MaterialButton::outlined("Install Docker").small()).clicked() {
                                        ui.data_mut(|d| {
                                            d.insert_temp(egui::Id::new("ssh_action_install_docker"), row_for_actions.host.clone())
                                        });
                                    }
                                } else {
                                    if ui.add(MaterialButton::outlined("Remove Docker").small()).clicked() {
                                        ui.data_mut(|d| {
                                            d.insert_temp(egui::Id::new("ssh_action_remove_docker"), row_for_actions.host.clone())
                                        });
                                    }
                                }
                            });

                            ui.add_space(2.0);

                            // Row 3: Dure operations
                            ui.horizontal(|ui| {
                                if !row_for_actions.dure_installed {
                                    if ui.add(MaterialButton::outlined("Check Dure").small()).clicked() {
                                        ui.data_mut(|d| {
                                            d.insert_temp(egui::Id::new("ssh_action_check_dure"), row_for_actions.host.clone())
                                        });
                                    }
                                    if ui.add(MaterialButton::outlined("Install Dure").small()).clicked() {
                                        ui.data_mut(|d| {
                                            d.insert_temp(egui::Id::new("ssh_action_install_dure"), row_for_actions.host.clone())
                                        });
                                    }
                                } else {
                                    if ui.add(MaterialButton::outlined("Remove Dure").small()).clicked() {
                                        ui.data_mut(|d| {
                                            d.insert_temp(egui::Id::new("ssh_action_remove_dure"), row_for_actions.host.clone())
                                        });
                                    }
                                }
                            });
                        });
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
        if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new("ssh_action_refresh"))) {
            ui.data_mut(|d| d.remove::<String>(egui::Id::new("ssh_action_refresh")));
            self.handle_refresh(&host);
        }

        if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new("ssh_action_ssh_check"))) {
            ui.data_mut(|d| d.remove::<String>(egui::Id::new("ssh_action_ssh_check")));
            self.handle_ssh_check(&host);
        }

        if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new("ssh_action_edit"))) {
            ui.data_mut(|d| d.remove::<String>(egui::Id::new("ssh_action_edit")));
            self.show_edit_dialog(&host, profile);
        }

        if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new("ssh_action_delete"))) {
            ui.data_mut(|d| d.remove::<String>(egui::Id::new("ssh_action_delete")));
            self.delete_host(&host, profile);
        }

        if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new("ssh_action_check_base"))) {
            ui.data_mut(|d| d.remove::<String>(egui::Id::new("ssh_action_check_base")));
            self.handle_check_base(&host);
        }

        if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new("ssh_action_install_base"))) {
            ui.data_mut(|d| d.remove::<String>(egui::Id::new("ssh_action_install_base")));
            self.handle_install_base(&host);
        }

        if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new("ssh_action_check_docker"))) {
            ui.data_mut(|d| d.remove::<String>(egui::Id::new("ssh_action_check_docker")));
            self.handle_check_docker(&host);
        }

        if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new("ssh_action_install_docker"))) {
            ui.data_mut(|d| d.remove::<String>(egui::Id::new("ssh_action_install_docker")));
            self.handle_install_docker(&host);
        }

        if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new("ssh_action_remove_docker"))) {
            ui.data_mut(|d| d.remove::<String>(egui::Id::new("ssh_action_remove_docker")));
            self.handle_remove_docker(&host);
        }

        if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new("ssh_action_check_dure"))) {
            ui.data_mut(|d| d.remove::<String>(egui::Id::new("ssh_action_check_dure")));
            self.handle_check_dure(&host);
        }

        if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new("ssh_action_install_dure"))) {
            ui.data_mut(|d| d.remove::<String>(egui::Id::new("ssh_action_install_dure")));
            self.show_install_dure_dialog = true;
            self.install_dure_host = host;
        }

        if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new("ssh_action_remove_dure"))) {
            ui.data_mut(|d| d.remove::<String>(egui::Id::new("ssh_action_remove_dure")));
            self.handle_remove_dure(&host);
        }

        // Handle drawer tab switch
        if let Some((host, new_tab)) = ui.data(|d| {
            self.rows.iter().find_map(|r| {
                d.get_temp::<(String, DrawerTab)>(egui::Id::new("ssh_drawer_tab_switch").with(&r.host))
            })
        }) {
            if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
                row.drawer_state.switch_tab(new_tab);
                ui.data_mut(|d| {
                    d.remove::<(String, DrawerTab)>(egui::Id::new("ssh_drawer_tab_switch").with(&host));
                });
            }
        }
    }

    /// Render add/edit host dialog
    fn render_add_dialog(
        &mut self,
        ui: &mut egui::Ui,
        profile: &Option<crate::calc::profile::ProfileContext>,
    ) {
        if !self.show_add_dialog {
            return;
        }

        let mut open = self.show_add_dialog;
        let title = if self.edit_mode { "Edit SSH Host" } else { "Add SSH Host" };
        let button_text = if self.edit_mode { "Save" } else { "Add" };

        egui::Window::new(title)
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                let description = if self.edit_mode {
                    "Edit SSH host connection settings:"
                } else {
                    "Configure a new SSH host connection:"
                };
                ui.label(description);
                ui.add_space(8.0);

                ui.label("Host (IP or domain):");
                ui.add_enabled(!self.edit_mode, egui::TextEdit::singleline(&mut self.add_host));
                if self.edit_mode {
                    ui.label("(Host cannot be changed when editing)");
                }
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
                    if ui.add(MaterialButton::filled(button_text)).clicked() {
                        if self.edit_mode {
                            self.save_edit_host_action(profile);
                        } else {
                            self.add_host_action(profile);
                        }
                    }
                    if ui.add(MaterialButton::outlined("Cancel")).clicked() {
                        self.show_add_dialog = false;
                        self.reset_add_dialog();
                    }
                });
            });

        if !open {
            self.show_add_dialog = false;
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

                    // TODO: Log audit with audit::push_gui()

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

    /// Reset add/edit dialog fields
    fn reset_add_dialog(&mut self) {
        self.edit_mode = false;
        self.edit_original_host.clear();
        self.add_host.clear();
        self.add_password.clear();
        self.add_private_key_path.clear();
        self.add_port = "22".to_string();
        self.add_use_password = false;
        self.add_use_private_key = false;
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

                    // TODO: Log audit with audit::push_gui()

                    self.loaded = false;
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {}", e));
                }
            }
        }
    }

    /// Show edit dialog for existing host
    fn show_edit_dialog(&mut self, host: &str, profile: &Option<crate::calc::profile::ProfileContext>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match load_config(profile) {
                Ok((app_config, _)) => {
                    if let Some(host_config) = app_config.ssh_hosts.iter().find(|h| h.host == host) {
                        self.edit_mode = true;
                        self.edit_original_host = host.to_string();
                        self.add_host = host_config.host.clone();
                        self.add_port = host_config.port.to_string();
                        self.add_password = host_config.password.clone().unwrap_or_default();
                        self.add_private_key_path = host_config.private_key_path.clone().unwrap_or_default();
                        self.add_use_password = host_config.password.is_some();
                        self.add_use_private_key = host_config.private_key_path.is_some();
                        self.show_add_dialog = true;
                    }
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {}", e));
                }
            }
        }
    }

    /// Save edited host configuration
    fn save_edit_host_action(&mut self, profile: &Option<crate::calc::profile::ProfileContext>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let port = match self.add_port.parse::<u16>() {
                Ok(p) => p,
                Err(e) => {
                    self.load_error = Some(format!("Invalid port '{}': {}", self.add_port, e));
                    return;
                }
            };

            match load_config(profile) {
                Ok((mut app_config, config_path)) => {
                    if let Some(host_config) = app_config.ssh_hosts.iter_mut()
                        .find(|h| h.host == self.edit_original_host)
                    {
                        host_config.port = port;
                        host_config.password = if self.add_use_password {
                            Some(self.add_password.clone())
                        } else {
                            None
                        };
                        host_config.private_key_path = if self.add_use_private_key {
                            Some(self.add_private_key_path.clone())
                        } else {
                            None
                        };

                        if let Err(e) = app_config.save(&config_path) {
                            self.load_error = Some(format!("Failed to save config: {}", e));
                            return;
                        }

                        dure_info!("Updated SSH host: {}", self.edit_original_host);
                        self.loaded = false;
                        self.show_add_dialog = false;
                        self.reset_add_dialog();
                    }
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {}", e));
                }
            }
        }
    }

    /// Render Install Dure dialog (.env configuration)
    fn render_install_dure_dialog(
        &mut self,
        ui: &mut egui::Ui,
        _profile: &Option<crate::calc::profile::ProfileContext>,
    ) {
        if !self.show_install_dure_dialog {
            return;
        }

        let mut open = self.show_install_dure_dialog;

        egui::Window::new("Install Dure - Configure .env")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(600.0)
            .show(ui.ctx(), |ui| {
                ui.label(format!("Configure environment variables for Dure on {}", self.install_dure_host));
                ui.add_space(8.0);

                ui.label(".env file content:");
                ui.add(
                    egui::TextEdit::multiline(&mut self.install_dure_env_content)
                        .desired_rows(15)
                        .code_editor()
                        .desired_width(f32::INFINITY)
                );
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.add(MaterialButton::filled("Install")).clicked() {
                        self.handle_install_dure_with_env(&self.install_dure_host.clone());
                        self.show_install_dure_dialog = false;
                        self.install_dure_env_content.clear();
                    }
                    if ui.add(MaterialButton::outlined("Cancel")).clicked() {
                        self.show_install_dure_dialog = false;
                        self.install_dure_env_content.clear();
                    }
                });
            });

        if !open {
            self.show_install_dure_dialog = false;
            self.install_dure_env_content.clear();
        }
    }

    // Action handlers

    /// Handle Refresh action - full status check
    fn handle_refresh(&mut self, host: &str) {
        dure_info!("Refreshing all status for SSH host: {}", host);
        // TODO: Trigger SSH actor to:
        // 1. SSH connection check
        // 2. Base packages check
        // 3. Docker installation check
        // 4. Dure operation check
        // TODO: Log audit with audit::push_gui()
    }

    /// Handle SSH Check action - connection test only
    fn handle_ssh_check(&mut self, host: &str) {
        dure_info!("Checking SSH connection to: {}", host);
        // TODO: Trigger SSH connection test (existing functionality)
        // TODO: Log audit with audit::push_gui()
    }

    /// Handle Check Base - check base packages
    fn handle_check_base(&mut self, host: &str) {
        dure_info!("Checking base packages on: {}", host);
        // TODO: Call calc::ssh::check_base_packages(host)
        // Check: extrepo, git, iptables, nftables, bpfcc-tools, moreutils
        // Update row.base_installed
        // TODO: Log audit with audit::push_gui()
    }

    /// Handle Install Base - install base packages
    fn handle_install_base(&mut self, host: &str) {
        dure_info!("Installing base packages on: {}", host);
        // TODO: Call calc::ssh::install_base_packages(host)
        // Install: extrepo, git, iptables, nftables, linux-headers, bpfcc-tools, moreutils
        // Setup: tcpconnect-bpfcc logging to /var/log/dure-network.log
        // Add to rc.local: tcpconnect-bpfcc | ts '%Y-%m-%d %H:%M:%S' >> /var/log/dure-network.log
        // TODO: Log audit with audit::push_gui()
    }

    /// Handle Check Docker - check docker installed
    fn handle_check_docker(&mut self, host: &str) {
        dure_info!("Checking Docker on: {}", host);
        // TODO: Call calc::ssh::check_docker(host)
        // Check: docker-ce package installed
        // Update row.docker_installed
        // TODO: Log audit with audit::push_gui()
    }

    /// Handle Install Docker - install docker
    fn handle_install_docker(&mut self, host: &str) {
        dure_info!("Installing Docker on: {}", host);
        // TODO: Call calc::ssh::install_docker(host)
        // Steps:
        // 1. extroot enable docker-ce
        // 2. apt update && apt install docker-ce
        // 3. Add docker user
        // 4. Configure docker user permissions
        // TODO: Log audit with audit::push_gui()
    }

    /// Handle Remove Docker - uninstall docker
    fn handle_remove_docker(&mut self, host: &str) {
        dure_info!("Removing Docker from: {}", host);
        // TODO: Call calc::ssh::remove_docker(host)
        // Run: apt purge docker-ce
        // TODO: Log audit with audit::push_gui()
    }

    /// Handle Check Dure - check dure installed
    fn handle_check_dure(&mut self, host: &str) {
        dure_info!("Checking Dure on: {}", host);
        // TODO: Call calc::ssh::check_dure(host)
        // Check: /srv/dure-mycart exists
        // Check: docker compose status in xmpp-proxy-stack
        // Update row.dure_installed
        // TODO: Log audit with audit::push_gui()
    }

    /// Handle Install Dure with .env configuration
    fn handle_install_dure_with_env(&mut self, host: &str) {
        dure_info!("Installing Dure on: {} with .env", host);
        // TODO: Call calc::ssh::install_dure(host, env_content)
        // Steps:
        // 1. git clone dure-mycart to /srv/dure-mycart
        // 2. Write .env to /srv/dure-mycart/xmpp-proxy-stack/.env
        // 3. docker compose up -d
        // TODO: Log audit with audit::push_gui()
    }

    /// Handle Remove Dure - uninstall dure
    fn handle_remove_dure(&mut self, host: &str) {
        dure_info!("Removing Dure from: {}", host);
        // TODO: Call calc::ssh::remove_dure(host)
        // Run: docker compose down in /srv/dure-mycart/xmpp-proxy-stack
        // TODO: Log audit with audit::push_gui()
    }
}
