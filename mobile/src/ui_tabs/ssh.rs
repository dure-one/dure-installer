//! SSH tab - SSH host configuration and management with drawer

use crate::{dure_debug, dure_error, dure_info};
use eframe::egui;
use egui_material3::MaterialButton;

use crate::config::{AppConfig, SshHostConfig};
use crate::viewmodel::ssh::{DrawerState, DrawerTab};

/// SSH row data for data table
#[derive(Clone, Debug)]
pub struct SshRow {
    // Identity
    pub host: String, // IP address (row key)
    pub port: u16,

    // Connection state
    pub ssh_connected: bool,

    // Service flags
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
    Refresh(String),      // host
    Delete(String),       // host
    RestartDocker(String), // host
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
}

impl Default for SshTab {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            loaded: false,
            load_error: None,
            show_add_dialog: false,
            add_host: String::new(),
            add_password: String::new(),
            add_private_key_path: String::new(),
            add_port: "22".to_string(),
            add_use_password: false,
            add_use_private_key: false,
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
                Ok((app_config, _)) => {
                    for host_config in &app_config.ssh_hosts {
                        let mut drawer_state = DrawerState::new();
                        drawer_state.set_ssh_host(host_config.host.clone());

                        self.rows.push(SshRow {
                            host: host_config.host.clone(),
                            port: host_config.port,
                            ssh_connected: false, // TODO: Determine from actual connection state
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

        // Add host dialog
        self.render_add_dialog(ui, profile);
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
            .auto_row_height(true)
            .min_row_height(70.0)
            .column("Host", 230.0 * width_ratio, false)
            .column("Operations", 510.0 * width_ratio, false);

        for row in self.rows.iter() {
            let row_for_cells = row.clone();
            let row_for_drawer = row.clone();
            let row_for_actions = row.clone();

            table = table.row(move |r| {
                r.cell_widget(move |ui| {
                        ui.label(&row_for_cells.host);
                    })
                    .cell_widget(move |ui| {
                        ui.horizontal(|ui| {
                            if ui.add(MaterialButton::outlined("Refresh").small()).clicked() {
                                ui.data_mut(|d| {
                                    d.insert_temp(
                                        egui::Id::new("ssh_action_refresh"),
                                        row_for_actions.host.clone(),
                                    )
                                });
                            }

                            if ui.add(MaterialButton::outlined("Delete").small()).clicked() {
                                ui.data_mut(|d| {
                                    d.insert_temp(
                                        egui::Id::new("ssh_action_delete"),
                                        row_for_actions.host.clone(),
                                    )
                                });
                            }

                            if row_for_actions.docker_installed {
                                if ui.add(MaterialButton::outlined("Restart Docker").small()).clicked() {
                                    ui.data_mut(|d| {
                                        d.insert_temp(
                                            egui::Id::new("ssh_action_restart_docker"),
                                            row_for_actions.host.clone(),
                                        )
                                    });
                                }
                            }
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
            // Refresh action - toggle drawer or reload data
            dure_info!("Refreshing SSH host: {}", host);
        }

        if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new("ssh_action_delete"))) {
            ui.data_mut(|d| d.remove::<String>(egui::Id::new("ssh_action_delete")));
            self.delete_host(&host, profile);
        }

        if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new("ssh_action_restart_docker"))) {
            ui.data_mut(|d| d.remove::<String>(egui::Id::new("ssh_action_restart_docker")));
            self.restart_docker(&host, profile);
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

    /// Reset add dialog fields
    fn reset_add_dialog(&mut self) {
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

    /// Restart Docker on SSH host
    fn restart_docker(
        &mut self,
        host: &str,
        _profile: &Option<crate::calc::profile::ProfileContext>,
    ) {
        dure_info!("Restarting Docker on {}", host);
        // TODO: Send command to SSH actor to restart Docker
        // TODO: Log audit with audit::push_gui()
    }
}
