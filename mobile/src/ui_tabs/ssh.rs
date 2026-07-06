//! SSH tab - SSH host configuration and management

use eframe::egui;
use egui_material3::MaterialButton;
use egui_material3::spreadsheet::{MaterialSpreadsheet, text_column};

use crate::calc::audit;
use crate::calc::ssh;
use crate::config::{AppConfig, SshHostConfig};

/// Linux system status information
#[derive(Clone, Debug, Default)]
struct LinuxStatus {
    uptime: String,
    external_ip: String,
    load_average: String,
    memory_usage: String,
    disk_usage: String,
    top_processes: Vec<String>,
}

/// SSH connection state
#[derive(Clone, Debug, PartialEq)]
enum ConnectionStatus {
    Connected,
    Offline,
    Testing,
    Unknown,
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        ConnectionStatus::Unknown
    }
}

/// Display data for SSH table row + drawer
#[derive(Clone, Debug, Default)]
struct SshRowData {
    // Identity
    host: String,
    port: u16,

    // Platform relationship
    platform_name: Option<String>,
    platform_type: Option<String>,

    // Service status flags
    linux_detected: bool,
    linux_os: Option<String>,
    ansible_enabled: bool,
    docker_enabled: bool,
    dure_wss_enabled: bool,

    // Drawer content
    linux_status: Option<LinuxStatus>,

    // Connection state
    connection_status: ConnectionStatus,
}

/// SSH tab state
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SshTab {
    /// Display data
    #[cfg_attr(feature = "serde", serde(skip))]
    rows: Vec<SshRowData>,

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
fn get_config_path() -> Result<std::path::PathBuf, String> {
    let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
        .ok_or_else(|| "Failed to get project directories".to_string())?;
    Ok(proj_dirs.config_dir().join("config.yml"))
}

/// Load application config
#[cfg(not(target_arch = "wasm32"))]
fn load_config() -> Result<(AppConfig, std::path::PathBuf), String> {
    let config_path = get_config_path()?;
    let app_config = AppConfig::load_or_default(&config_path);
    Ok((app_config, config_path))
}

impl SshTab {
    /// Render the SSH tab UI
    pub fn ui(&mut self, ui: &mut egui::Ui, mut vm: Option<&mut crate::viewmodel::ViewModel>) {
        /* OLD ui() - being rewritten in Tasks 8-14
        // ViewModel event processing (MVVM pattern)
        if let Some(ref mut vm) = vm {
            let events = vm.poll_events(ui.ctx());
            if !events.is_empty() {
                eprintln!("🔍 SSH UI: Polling events, found {} events", events.len());
            }
            for event in events {
                use crate::viewmodel::ViewModelEvent;
                use crate::viewmodel::ssh::SshEvent;

                eprintln!("🔍 SSH UI: Processing event: {:?}", event);
                match event {
                    ViewModelEvent::Ssh(SshEvent::HostAdded { name }) => {
                        eprintln!("✓ SSH host {} added successfully", name);

                        // Refresh the list
                        self.loaded = false;
                        self.load_error = None;
                    }
                    ViewModelEvent::Ssh(SshEvent::HostDeleted { name }) => {
                        eprintln!("✓ SSH host {} deleted successfully", name);

                        // Remove host from config
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Ok((mut app_config, config_path)) = load_config() {
                            app_config.ssh_hosts.retain(|h| h.host != name);

                            if let Err(e) = app_config.save(&config_path) {
                                self.load_error = Some(format!("Failed to save config: {}", e));
                            } else {
                                eprintln!("✓ Config updated, refreshing list");
                                self.loaded = false;
                                self.selected_row = None;
                                self.load_error = None;
                            }
                        }
                    }
                    ViewModelEvent::Ssh(SshEvent::ConnectionTested { name, success, latency_ms }) => {
                        eprintln!("🔍 SSH UI: Received ConnectionTested event - name: {}, success: {}, latency: {:?}", name, success, latency_ms);
                        if success {
                            let latency_str = if let Some(latency) = latency_ms {
                                format!(" ({}ms)", latency)
                            } else {
                                String::new()
                            };
                            self.test_result = Some(Ok(format!("✓ Connection successful{}", latency_str)));
                            eprintln!("✓ SSH UI: Set test result to success");
                        } else {
                            self.test_result = Some(Err(format!("✗ Connection failed to {}", name)));
                            eprintln!("✗ SSH UI: Set test result to failure");
                        }
                        self.test_in_progress = false;
                    }
                    ViewModelEvent::Ssh(SshEvent::HostInitialized { name, success }) => {
                        if success {
                            self.init_progress_log.push(format!("✓ Host '{}' initialized successfully", name));
                        } else {
                            self.init_progress_log.push(format!("✗ Host '{}' initialization failed", name));
                        }
                        self.init_in_progress = false;
                    }
                    ViewModelEvent::Ssh(SshEvent::Error { operation, error }) => {
                        if operation == "add_host" {
                            self.load_error = Some(format!("Failed to add host: {}", error));
                        } else if operation == "delete_host" {
                            self.load_error = Some(format!("Failed to delete host: {}", error));
                        } else if operation == "test_connection" {
                            self.test_result = Some(Err(format!("Connection test failed: {}", error)));
                            self.test_in_progress = false;
                        }
                    }
                    _ => {}
                }
            }
        }

        ui.heading("SSH Hosts");
        ui.add_space(4.0);
        ui.label("Manage SSH hosts for remote server deployment and management.");
        ui.add_space(8.0);

        // Get selected row for action buttons
        let selected_row_idx = self.spreadsheet.as_ref().and_then(|s| s.get_selected_row());
        let has_selection = selected_row_idx.is_some();

        // Action buttons
        ui.horizontal(|ui| {
            if ui.add(MaterialButton::filled("Add Host")).clicked() {
                self.show_add_dialog = true;
                self.add_host.clear();
                self.add_password.clear();
                self.add_private_key_path.clear();
                self.add_port = "22".to_string();
                self.add_use_password = false;
                self.add_use_private_key = false;
            }

            // Delete button - enabled only when a row is selected
            let delete_button = MaterialButton::outlined("Delete Selected");
            let delete_button = if has_selection {
                delete_button
            } else {
                delete_button.enabled(false)
            };

            if ui.add(delete_button).clicked() {
                if let Some(idx) = selected_row_idx {
                    if idx < self.rows.len() {
                        let host = self.rows[idx][0].clone();
                        self.execute_delete_host(host, vm.as_deref_mut());
                    }
                }
            }

            // Check Connection button - enabled only when a row is selected
            let check_button = MaterialButton::outlined("Check Connection");
            let check_button = if has_selection && !self.test_in_progress {
                check_button
            } else {
                check_button.enabled(false)
            };

            if ui.add(check_button).clicked() {
                eprintln!("🔍 Check Connection button clicked");
                if let Some(idx) = selected_row_idx {
                    eprintln!("🔍 Selected row index: {}", idx);
                    if idx < self.rows.len() {
                        let host = self.rows[idx][0].clone();
                        eprintln!("🔍 Testing connection to host: {}", host);
                        self.execute_test_connection(host, vm.as_deref_mut());
                    }
                } else {
                    eprintln!("⚠️ No row selected");
                }
            }

            // Initialize button - enabled only when a row is selected
            let init_button = MaterialButton::outlined("Initialize");
            let init_button = if has_selection && !self.init_in_progress {
                init_button
            } else {
                init_button.enabled(false)
            };

            if ui.add(init_button).clicked() {
                if let Some(idx) = selected_row_idx {
                    if idx < self.rows.len() {
                        let host = self.rows[idx][0].clone();
                        self.execute_init_host(host, vm.as_deref_mut());
                    }
                }
            }

            if ui.add(MaterialButton::outlined("Refresh")).clicked() {
                self.loaded = false;
                self.load_error = None;
            }

            // Show selected host info
            if let Some(idx) = selected_row_idx {
                if idx < self.rows.len() {
                    ui.label(format!("│ Selected: {}", self.rows[idx][0]));
                }
            }
        });
        ui.add_space(8.0);

        // Lazy-load from config on first render or after refresh
        if !self.loaded {
            self.load_rows();
            self.loaded = true;
        }

        if let Some(err) = &self.load_error {
            ui.colored_label(egui::Color32::RED, format!("⚠ {err}"));
            ui.add_space(4.0);
        }

        // SSH hosts spreadsheet - fill remaining space
        if let Some(spreadsheet) = &mut self.spreadsheet {
            let available_height = ui.available_height();

            ui.group(|ui| {
                // Set the group to fill available space
                ui.set_min_height(available_height - 20.0); // Leave some padding
                ui.set_width(ui.available_width());

                egui::ScrollArea::vertical()
                    .max_height(available_height - 20.0)
                    .show(ui, |ui| {
                        spreadsheet.show(ui);
                    });
            });
        }

        // Add host dialog
        if self.show_add_dialog {
            self.render_add_dialog(ui.ctx(), vm.as_deref_mut());
        }

        // Init progress display
        if self.init_in_progress {
            self.render_init_progress(ui);
        }

        // Poll for connection test completion (DEPRECATED - using ViewModel events now)
        // self.poll_connection_test();

        // Show connection test result
        if let Some(result) = self.test_result.clone() {
            self.render_test_result(ui.ctx(), &result);
        }
        */

        // Temporary placeholder during refactoring
        ui.heading("SSH Hosts");
        ui.add_space(8.0);
        ui.label("SSH tab UI is being refactored to use data_table with drawer pattern.");
        ui.label("This will be complete after Tasks 8-14.");

        let _ = vm; // Suppress unused warning
    }

    /* OLD load_rows - will be rewritten in Task 9
    fn load_rows(&mut self) {
        self.rows.clear();
        self.load_error = None;

        #[cfg(not(target_arch = "wasm32"))]
        {
            match load_config() {
                Ok((app_config, _)) => {
                    let mut data_rows = Vec::new();

                    for host_config in &app_config.ssh_hosts {
                        let auth_type = if host_config.private_key_path.is_some() {
                            "Private Key"
                        } else if host_config.password.is_some() {
                            "Password"
                        } else {
                            "SSH Agent"
                        };

                        let status = if host_config.initialized {
                            "Initialized"
                        } else {
                            "Not Initialized"
                        };

                        self.rows.push([
                            host_config.host.clone(),
                            host_config.port.to_string(),
                            auth_type.to_string(),
                            status.to_string(),
                        ]);

                        data_rows.push(vec![
                            host_config.host.clone(),
                            host_config.port.to_string(),
                            auth_type.to_string(),
                            status.to_string(),
                        ]);
                    }

                    // Clear and update spreadsheet with fresh data
                    if let Some(spreadsheet) = &mut self.spreadsheet {
                        // Recreate spreadsheet with fresh data to avoid duplicates
                        let columns = vec![
                            text_column("Host", 250.0),
                            text_column("Port", 80.0),
                            text_column("Auth", 150.0),
                            text_column("Status", 150.0),
                        ];

                        match MaterialSpreadsheet::new("ssh_spreadsheet", columns) {
                            Ok(mut new_spreadsheet) => {
                                new_spreadsheet.set_striped(true);
                                new_spreadsheet.set_row_selection_enabled(true);
                                new_spreadsheet.set_allow_selection(true);
                                new_spreadsheet.init_with_data(data_rows);
                                *spreadsheet = new_spreadsheet;
                            }
                            Err(e) => {
                                self.load_error =
                                    Some(format!("Failed to create spreadsheet: {e}"));
                            }
                        }
                    }
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {e}"));
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.load_error = Some("SSH management not available on WASM".to_string());
        }
    }
    */

    fn render_add_dialog(&mut self, ctx: &egui::Context, mut vm: Option<&mut crate::viewmodel::ViewModel>) {
        let mut open = true;

        egui::Window::new("Add SSH Host")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Configure a new SSH host:");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Host:");
                    ui.text_edit_singleline(&mut self.add_host)
                        .on_hover_text("Format: username@hostname (e.g., root@dure.com)");
                });

                ui.horizontal(|ui| {
                    ui.label("Port:");
                    ui.text_edit_singleline(&mut self.add_port);
                });

                ui.add_space(8.0);
                ui.label("Authentication:");

                ui.checkbox(&mut self.add_use_password, "Use password");
                if self.add_use_password {
                    ui.horizontal(|ui| {
                        ui.label("Password:");
                        ui.add(egui::TextEdit::singleline(&mut self.add_password).password(true));
                    });
                }

                ui.checkbox(&mut self.add_use_private_key, "Use private key");
                if self.add_use_private_key {
                    ui.horizontal(|ui| {
                        ui.label("Key path:");
                        ui.text_edit_singleline(&mut self.add_private_key_path)
                            .on_hover_text("Path to private key file (e.g., ~/.ssh/id_rsa)");
                    });
                }

                if !self.add_use_password && !self.add_use_private_key {
                    ui.label(
                        egui::RichText::new("Will use SSH agent if no auth method selected")
                            .color(ui.visuals().weak_text_color()),
                    );
                }

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.show_add_dialog = false;
                    }

                    if ui.button("Add").clicked() {
                        if self.add_host.is_empty() {
                            self.load_error =
                                Some("Host is required (format: username@hostname)".to_string());
                        } else if !self.add_host.contains('@') {
                            self.load_error =
                                Some("Invalid host format. Use: username@hostname".to_string());
                        } else {
                            self.execute_add_host(vm.as_deref_mut());
                            self.show_add_dialog = false;
                        }
                    }
                });
            });

        if !open {
            self.show_add_dialog = false;
        }
    }

    fn execute_add_host(&mut self, vm: Option<&mut crate::viewmodel::ViewModel>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Parse port
            let port = match self.add_port.parse::<u16>() {
                Ok(p) => p,
                Err(_) => {
                    self.load_error = Some("Invalid port number".to_string());
                    return;
                }
            };

            // Parse user@host format
            let (user, host) = if self.add_host.contains('@') {
                let parts: Vec<&str> = self.add_host.split('@').collect();
                if parts.len() == 2 {
                    (parts[0].to_string(), parts[1].to_string())
                } else {
                    self.load_error = Some("Invalid format. Use: username@hostname".to_string());
                    return;
                }
            } else {
                self.load_error = Some("Invalid format. Use: username@hostname".to_string());
                return;
            };

            let ssh_key_path = if self.add_use_private_key && !self.add_private_key_path.is_empty() {
                shellexpand::tilde(&self.add_private_key_path).to_string()
            } else {
                String::new()
            };

            // ViewModel-based implementation
            if let Some(vm) = vm {
                match vm.add_ssh_host(
                    self.add_host.clone(), // name (full user@host)
                    host,
                    port,
                    user,
                    ssh_key_path,
                ) {
                    Ok(_) => {
                        // Record audit event
                        let _ = audit::push_gui("system", "desktop", "ssh add", &self.add_host);
                        // Config will be updated when HostAdded event arrives
                    }
                    Err(e) => {
                        self.load_error = Some(format!("Failed to add SSH host: {}", e));
                    }
                }
            } else {
                self.load_error = Some("ViewModel not available".to_string());
            }
        }
    }

    fn execute_delete_host(&mut self, host: String, mut vm: Option<&mut crate::viewmodel::ViewModel>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // ViewModel-based implementation
            if let Some(ref mut vm) = vm {
                // Send command to ViewModel
                if let Err(e) = vm.delete_ssh_host(host.clone()) {
                    self.load_error = Some(format!("Failed to start host deletion: {}", e));
                    return;
                }

                // Record audit event
                let _ = audit::push_gui("system", "desktop", "ssh del", &host);

                // Note: Config will be updated when HostDeleted event arrives
            } else {
                // Fallback: no ViewModel available
                self.load_error = Some("ViewModel not available".to_string());
            }
        }
    }

    /* OLD - will be replaced in later tasks
    fn execute_init_host(&mut self, host: String, vm: Option<&mut crate::viewmodel::ViewModel>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(vm) = vm {
                self.init_in_progress = true;
                self.init_host = Some(host.clone());
                self.init_progress_log.clear();
                self.init_progress_log
                    .push(format!("Initializing SSH host: {}", host));

                match vm.init_ssh_host(host) {
                    Ok(_) => {
                        eprintln!("✓ SSH host init command sent");
                    }
                    Err(e) => {
                        self.init_progress_log.push(format!("✗ Failed to start initialization: {}", e));
                        self.init_in_progress = false;
                    }
                }
            }
        }
    }

    */

    /* OLD methods using removed fields - commented out during Task 8
    fn execute_test_connection(&mut self, host: String, mut vm: Option<&mut crate::viewmodel::ViewModel>) {
        eprintln!("🔍 execute_test_connection called for host: {}", host);
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(ref mut vm) = vm {
                eprintln!("🔍 ViewModel available, sending test command");
                self.test_in_progress = true;
                self.test_result = None;

                match vm.test_ssh_connection(host.clone()) {
                    Ok(_) => {
                        eprintln!("✓ Test command sent successfully");
                    }
                    Err(e) => {
                        eprintln!("✗ Failed to send test command: {}", e);
                        self.test_result = Some(Err(format!("Failed to start connection test: {}", e)));
                        self.test_in_progress = false;
                    }
                }
                // Result will be delivered via ConnectionTested event
            } else {
                eprintln!("✗ ViewModel not available");
                self.test_result = Some(Err("ViewModel not available".to_string()));
                self.test_in_progress = false;
            }
        }
    }

    fn poll_connection_test(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(promise) = &self.test_promise {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(conn_result) => {
                        self.test_result = Some(Ok(conn_result.message.clone()));
                    }
                    Err(e) => {
                        self.test_result = Some(Err(e.clone()));
                    }
                }

                self.test_promise = None;
                self.test_in_progress = false;
            }
        }
    }

    fn render_test_result(&mut self, ctx: &egui::Context, result: &Result<String, String>) {
        let mut open = true;

        egui::Window::new("Connection Test Result")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                match result {
                    Ok(msg) => {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("✓")
                                    .color(egui::Color32::GREEN)
                                    .size(20.0),
                            );
                            ui.label(egui::RichText::new("Connection successful").strong());
                        });
                        ui.add_space(8.0);
                        ui.label(msg);
                    }
                    Err(msg) => {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("✗")
                                    .color(egui::Color32::RED)
                                    .size(20.0),
                            );
                            ui.label(egui::RichText::new("Connection failed").strong());
                        });
                        ui.add_space(8.0);
                        ui.colored_label(egui::Color32::RED, msg);
                    }
                }

                ui.add_space(12.0);

                if ui.button("Close").clicked() {
                    self.test_result = None;
                }
            });

        if !open {
            self.test_result = None;
        }
    }

    fn render_init_progress(&mut self, ui: &mut egui::Ui) {
        // Poll for completion
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(promise) = &self.init_promise {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(progress_log) => {
                        self.init_progress_log.extend(progress_log.clone());

                        // Mark host as initialized and save config
                        if let Some(host) = &self.init_host {
                            if let Ok((mut app_config, config_path)) = load_config() {
                                if let Some(host_config) =
                                    app_config.ssh_hosts.iter_mut().find(|h| &h.host == host)
                                {
                                    host_config.initialized = true;

                                    match app_config.save(&config_path) {
                                        Ok(_) => {
                                            eprintln!(
                                                "✓ SSH host initialized, refreshing spreadsheet"
                                            );
                                            self.loaded = false; // Trigger reload
                                            self.init_progress_log
                                                .push("✓ Configuration saved".to_string());
                                        }
                                        Err(e) => {
                                            self.init_progress_log
                                                .push(format!("⚠ Failed to save config: {e}"));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        self.init_progress_log
                            .push(format!("✗ Initialization failed: {}", e));
                    }
                }

                self.init_promise = None;
            }
        }

        ui.add_space(12.0);
        ui.separator();
        ui.heading("Initialization Progress");

        if let Some(host) = &self.init_host {
            ui.label(format!("Host: {}", host));
        }

        ui.add_space(8.0);

        // Show spinner if still in progress
        #[cfg(not(target_arch = "wasm32"))]
        if self.init_promise.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Initialization in progress...");
            });
            ui.add_space(8.0);
        }

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for log in &self.init_progress_log {
                    ui.label(log);
                }
            });

        ui.add_space(8.0);

        let can_close = self.init_promise.is_none();
        if ui
            .add_enabled(can_close, egui::Button::new("Close"))
            .clicked()
        {
            self.init_in_progress = false;
            self.init_host = None;
            self.init_progress_log.clear();
        }

        if !can_close {
            ui.colored_label(
                egui::Color32::GRAY,
                "Please wait for initialization to complete",
            );
        }
    }
    */
}

/// Format platform relationship for display
fn format_platform(row: &SshRowData) -> String {
    match (&row.platform_name, &row.platform_type) {
        (Some(name), Some(ptype)) => format!("{}({})", name, ptype),
        _ => "manual".to_string(),
    }
}

/// Format status column showing only enabled services
fn format_status(row: &SshRowData) -> String {
    let mut parts = Vec::new();

    // Show Linux with OS if available
    if row.linux_detected {
        if let Some(os) = &row.linux_os {
            parts.push(format!("✓ linux({})", os));
        } else {
            parts.push("✓ linux".to_string());
        }
    }

    // Show enabled services
    if row.ansible_enabled {
        parts.push("✓ ansible".to_string());
    }

    if row.docker_enabled {
        parts.push("✓ docker".to_string());
    }

    if row.dure_wss_enabled {
        parts.push("✓ dure-wss".to_string());
    }

    if parts.is_empty() {
        "—".to_string()
    } else {
        parts.join(" ")
    }
}

/// Render drawer content with Linux status and service placeholders
fn render_drawer_content(ui: &mut egui::Ui, row: &SshRowData) {
    ui.add_space(8.0);

    // Linux status (detailed)
    ui.label(egui::RichText::new("linux:").strong());
    if let Some(status) = &row.linux_status {
        ui.label(format!("  uptime: {}", status.uptime));
        ui.label(format!("  ip: {}", status.external_ip));
        ui.label(format!("  load: {}", status.load_average));
        ui.label(format!("  memory: {}", status.memory_usage));
        ui.label(format!("  disk: {}", status.disk_usage));

        let processes = if status.top_processes.is_empty() {
            "none".to_string()
        } else {
            status.top_processes.join(", ")
        };
        ui.label(format!("  ps: {}", processes));
    } else {
        ui.colored_label(
            ui.visuals().weak_text_color(),
            "  (status not loaded - click Refresh to load)"
        );
    }

    ui.add_space(4.0);

    // Ansible placeholder
    ui.label(egui::RichText::new("ansible:").strong());
    ui.colored_label(ui.visuals().weak_text_color(), "  —");

    ui.add_space(4.0);

    // Docker placeholder
    ui.label(egui::RichText::new("docker:").strong());
    ui.colored_label(ui.visuals().weak_text_color(), "  —");

    ui.add_space(4.0);

    // Dure-WSS placeholder
    ui.label(egui::RichText::new("dure-wss:").strong());
    ui.colored_label(ui.visuals().weak_text_color(), "  —");

    ui.add_space(4.0);
}
