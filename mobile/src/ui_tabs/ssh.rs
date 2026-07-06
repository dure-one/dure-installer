//! SSH tab - SSH host configuration and management

use eframe::egui;
use egui_material3::MaterialButton;

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
    docker_containers: Vec<crate::config::DockerContainerConfig>,
    ansible_roles: Vec<crate::config::AnsibleRoleConfig>,
    dure_wss_config: Option<crate::config::DureWssConfig>,

    // Connection state
    connection_status: ConnectionStatus,

    // Refresh state
    refreshing: bool,
    refresh_pending_count: u8,
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

    // Docker Install Dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_docker_install_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_install_host_idx: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_image_input: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_container_name: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_tag: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_metadata: Option<crate::calc::docker::DockerImageMetadata>,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_port_mappings: Vec<(String, String)>,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_env_vars: Vec<(String, String)>,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_validating: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_validation_error: Option<String>,

    // Ansible Install Dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_ansible_install_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_install_host_idx: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_role_input: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_instance_name: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_metadata: Option<crate::calc::ansible::AnsibleRoleMetadata>,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_ports: Vec<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_variables: Vec<(String, String)>,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_validating: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_validation_error: Option<String>,

    // Dure-WSS Install Dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_dure_wss_install_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    dure_wss_install_host_idx: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip))]
    dure_wss_domain: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    dure_wss_email: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    dure_wss_channel: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    dure_wss_variant: String,

    // Progress Dialogs
    #[cfg_attr(feature = "serde", serde(skip))]
    show_docker_progress: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_progress_host: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_progress_messages: Vec<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_progress_error: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    docker_progress_complete: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    show_ansible_progress: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_progress_host: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_progress_messages: Vec<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_progress_error: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    ansible_progress_complete: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    show_dure_wss_progress: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    dure_wss_progress_host: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    dure_wss_progress_messages: Vec<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    dure_wss_progress_error: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    dure_wss_progress_complete: bool,
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
            show_docker_install_dialog: false,
            docker_install_host_idx: None,
            docker_image_input: String::new(),
            docker_container_name: String::new(),
            docker_tag: "latest".to_string(),
            docker_metadata: None,
            docker_port_mappings: Vec::new(),
            docker_env_vars: Vec::new(),
            docker_validating: false,
            docker_validation_error: None,
            show_ansible_install_dialog: false,
            ansible_install_host_idx: None,
            ansible_role_input: String::new(),
            ansible_instance_name: String::new(),
            ansible_metadata: None,
            ansible_ports: Vec::new(),
            ansible_variables: Vec::new(),
            ansible_validating: false,
            ansible_validation_error: None,
            show_dure_wss_install_dialog: false,
            dure_wss_install_host_idx: None,
            dure_wss_domain: String::new(),
            dure_wss_email: String::new(),
            dure_wss_channel: "stable".to_string(),
            dure_wss_variant: "default".to_string(),
            show_docker_progress: false,
            docker_progress_host: String::new(),
            docker_progress_messages: Vec::new(),
            docker_progress_error: None,
            docker_progress_complete: false,
            show_ansible_progress: false,
            ansible_progress_host: String::new(),
            ansible_progress_messages: Vec::new(),
            ansible_progress_error: None,
            ansible_progress_complete: false,
            show_dure_wss_progress: false,
            dure_wss_progress_host: String::new(),
            dure_wss_progress_messages: Vec::new(),
            dure_wss_progress_error: None,
            dure_wss_progress_complete: false,
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
    /// Load SSH hosts from config and build row data
    fn load_rows(&mut self) {
        self.rows.clear();
        self.load_error = None;

        #[cfg(not(target_arch = "wasm32"))]
        {
            match load_config() {
                Ok((app_config, _)) => {
                    for host_config in &app_config.ssh_hosts {
                        // Resolve platform relationship
                        let (platform_name, platform_type) =
                            if let Some(pname) = &host_config.platform_name {
                                let ptype = app_config
                                    .platforms
                                    .iter()
                                    .find(|p| &p.name == pname)
                                    .map(|p| p.platform_type.clone());
                                (Some(pname.clone()), ptype)
                            } else {
                                (None, None)
                            };

                        // Find installed services for this host
                        let docker_containers = host_config.docker_containers.clone();
                        let ansible_roles = host_config.ansible_roles.clone();
                        let dure_wss_config = host_config.dure_wss_config.clone();

                        self.rows.push(SshRowData {
                            host: host_config.host.clone(),
                            port: host_config.port,
                            platform_name,
                            platform_type,
                            linux_detected: false,
                            linux_os: None,
                            ansible_enabled: false,
                            docker_enabled: false,
                            dure_wss_enabled: false,
                            linux_status: None,
                            docker_containers,
                            ansible_roles,
                            dure_wss_config,
                            connection_status: ConnectionStatus::Unknown,
                            refreshing: false,
                            refresh_pending_count: 0,
                        });
                    }
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {}", e));
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.load_error = Some("SSH management not available on WASM".to_string());
        }
    }

    /// Handle ViewModel events to update UI state
    fn handle_event(&mut self, event: crate::viewmodel::ViewModelEvent) {
        use crate::viewmodel::ViewModelEvent;
        use crate::viewmodel::ssh::SshEvent;

        match event {
            ViewModelEvent::Ssh(SshEvent::HostAdded { name }) => {
                eprintln!("✓ SSH host {} added", name);
                self.loaded = false; // Trigger reload
            }

            ViewModelEvent::Ssh(SshEvent::HostDeleted { name }) => {
                eprintln!("✓ SSH host {} deleted", name);

                // Remove from config
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok((mut app_config, config_path)) = load_config() {
                    app_config.ssh_hosts.retain(|h| h.host != name);
                    let _ = app_config.save(&config_path);
                }

                self.loaded = false; // Trigger reload
            }

            ViewModelEvent::Ssh(SshEvent::LinuxStatusRetrieved {
                name,
                uptime,
                external_ip,
                load_average,
                memory_usage,
                disk_usage,
                top_processes,
            }) => {
                eprintln!("✓ Linux status retrieved for {}", name);

                // Update row
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.linux_status = Some(LinuxStatus {
                        uptime,
                        external_ip,
                        load_average,
                        memory_usage,
                        disk_usage,
                        top_processes,
                    });
                    row.linux_detected = true;
                }
            }

            ViewModelEvent::Ssh(SshEvent::DockerInstalled { name }) => {
                eprintln!("✓ Docker installed on {}", name);

                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.docker_enabled = true;
                }

                // Update progress dialog
                if self.show_docker_progress && self.docker_progress_host == name {
                    self.docker_progress_messages.push("✓ Docker installed successfully".to_string());
                    self.docker_progress_complete = true;
                }
            }

            ViewModelEvent::Ssh(SshEvent::DockerStatusRetrieved {
                name,
                installed,
                running: _,
            }) => {
                eprintln!("✓ Docker status for {}: installed={}", name, installed);

                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.docker_enabled = installed;
                }
            }

            ViewModelEvent::Ssh(SshEvent::DockerUninstalled { name }) => {
                eprintln!("✓ Docker uninstalled from {}", name);

                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.docker_enabled = false;
                }
            }

            ViewModelEvent::Ssh(SshEvent::AnsibleInstalled { name }) => {
                eprintln!("✓ Ansible installed on {}", name);

                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.ansible_enabled = true;
                }

                // Update progress dialog
                if self.show_ansible_progress && self.ansible_progress_host == name {
                    self.ansible_progress_messages.push("✓ Ansible installed successfully".to_string());
                    self.ansible_progress_complete = true;
                }
            }

            ViewModelEvent::Ssh(SshEvent::AnsibleStatusRetrieved { name, installed }) => {
                eprintln!("✓ Ansible status for {}: installed={}", name, installed);

                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.ansible_enabled = installed;
                }
            }

            ViewModelEvent::Ssh(SshEvent::AnsibleUninstalled { name }) => {
                eprintln!("✓ Ansible uninstalled from {}", name);

                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.ansible_enabled = false;
                }
            }

            ViewModelEvent::Ssh(SshEvent::DureWssInstalled { name }) => {
                eprintln!("✓ Dure-WSS installed on {}", name);

                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.dure_wss_enabled = true;
                }

                // Update progress dialog
                if self.show_dure_wss_progress && self.dure_wss_progress_host == name {
                    self.dure_wss_progress_messages.push("✓ Dure-WSS installed successfully".to_string());
                }
            }

            ViewModelEvent::Ssh(SshEvent::DureWssStatusRetrieved { name, installed }) => {
                eprintln!("✓ Dure-WSS status for {}: installed={}", name, installed);

                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.dure_wss_enabled = installed;
                }
            }

            ViewModelEvent::Ssh(SshEvent::DureWssUninstalled { host_name }) => {
                eprintln!("✓ Dure-WSS uninstalled from {}", host_name);

                if let Some(row) = self.rows.iter_mut().find(|r| r.host == host_name) {
                    row.dure_wss_enabled = false;
                }
            }

            ViewModelEvent::Ssh(SshEvent::ServiceError {
                name,
                service,
                operation,
                error,
            }) => {
                self.load_error = Some(format!(
                    "Failed to {} {} on {}: {}",
                    operation, service, name, error
                ));

                // Update progress dialogs
                if service == "docker" && self.show_docker_progress && self.docker_progress_host == name {
                    self.docker_progress_error = Some(error.clone());
                    self.docker_progress_complete = true;
                }
                if service == "ansible" && self.show_ansible_progress && self.ansible_progress_host == name {
                    self.ansible_progress_error = Some(error.clone());
                    self.ansible_progress_complete = true;
                }
                if service == "dure-wss" && self.show_dure_wss_progress && self.dure_wss_progress_host == name {
                    self.dure_wss_progress_error = Some(error.clone());
                    self.dure_wss_progress_complete = true;
                }
            }

            ViewModelEvent::Ssh(SshEvent::Error { operation, error }) => {
                // Docker validation errors
                if operation.contains("Docker") && self.show_docker_install_dialog {
                    self.docker_validation_error = Some(error.clone());
                    self.docker_validating = false;
                }
                // Ansible validation errors
                if operation.contains("Ansible") && self.show_ansible_install_dialog {
                    self.ansible_validation_error = Some(error.clone());
                    self.ansible_validating = false;
                }
                self.load_error = Some(format!("SSH operation '{}' failed: {}", operation, error));
            }

            ViewModelEvent::Ssh(SshEvent::DockerImageValidated { image, metadata }) => {
                eprintln!("✓ Docker image {} validated", image);
                self.docker_metadata = Some(metadata.clone());

                // Populate port mappings from metadata
                self.docker_port_mappings.clear();
                for port in &metadata.exposed_ports {
                    self.docker_port_mappings.push((port.to_string(), port.to_string()));
                }

                // Populate env vars from metadata
                self.docker_env_vars.clear();
                for env in &metadata.env_vars {
                    if let Some(idx) = env.find('=') {
                        let (key, value) = env.split_at(idx);
                        self.docker_env_vars.push((key.to_string(), value[1..].to_string()));
                    }
                }

                self.docker_validating = false;
                self.docker_validation_error = None;
            }

            ViewModelEvent::Ssh(SshEvent::DockerImageInstalled { host_name, container_name }) => {
                eprintln!("✓ Docker container {} installed on {}", container_name, host_name);
                self.show_docker_install_dialog = false;
                self.loaded = false; // Trigger reload
            }

            ViewModelEvent::Ssh(SshEvent::DockerDaemonInstalled { host_name }) => {
                eprintln!("✓ Docker daemon installed on {}", host_name);
            }

            ViewModelEvent::Ssh(SshEvent::AnsibleRoleValidated { role, metadata }) => {
                eprintln!("✓ Ansible role {} validated", role);
                self.ansible_metadata = Some(metadata.clone());

                // Populate ports from metadata
                self.ansible_ports.clear();
                for port in &metadata.suggested_ports {
                    self.ansible_ports.push(port.to_string());
                }

                // Populate variables from metadata
                self.ansible_variables.clear();
                for (key, value) in &metadata.variables {
                    self.ansible_variables.push((key.clone(), value.clone()));
                }

                self.ansible_validating = false;
                self.ansible_validation_error = None;
            }

            ViewModelEvent::Ssh(SshEvent::AnsibleRoleInstalled { host_name, instance_name }) => {
                eprintln!("✓ Ansible role {} installed on {}", instance_name, host_name);
                self.show_ansible_install_dialog = false;
                self.loaded = false; // Trigger reload
            }

            ViewModelEvent::Ssh(SshEvent::AnsibleDaemonInstalled { host_name }) => {
                eprintln!("✓ Ansible daemon installed on {}", host_name);
            }

            ViewModelEvent::Ssh(SshEvent::DureWssServiceInstalled { host_name, domain }) => {
                eprintln!("✓ Dure-WSS service installed on {} with domain {}", host_name, domain);
                self.show_dure_wss_install_dialog = false;
                self.loaded = false; // Trigger reload

                // Update progress dialog
                if self.show_dure_wss_progress && self.dure_wss_progress_host == host_name {
                    self.dure_wss_progress_messages.push(format!("✓ Dure-WSS installed successfully with domain {}", domain));
                    self.dure_wss_progress_complete = true;
                }
            }

            ViewModelEvent::Ssh(SshEvent::DureWssStarted { host_name }) => {
                eprintln!("✓ Dure-WSS started on {}", host_name);
            }

            ViewModelEvent::Ssh(SshEvent::DureWssStopped { host_name }) => {
                eprintln!("✓ Dure-WSS stopped on {}", host_name);
            }

            ViewModelEvent::Ssh(SshEvent::Progress { operation, progress: _, status }) => {
                // Update progress dialogs based on operation
                if operation.contains("install_docker") && self.show_docker_progress {
                    self.docker_progress_messages.push(status.clone());
                }
                if operation.contains("install_ansible") && self.show_ansible_progress {
                    self.ansible_progress_messages.push(status.clone());
                }
                if operation.contains("install_dure_wss") && self.show_dure_wss_progress {
                    self.dure_wss_progress_messages.push(status);
                }
            }

            // Keep existing event handlers (ConnectionTested, HostInitialized, etc.)
            _ => {}
        }
    }

    /// Process action triggers from operation buttons
    fn process_action_triggers(
        &mut self,
        ui: &mut egui::Ui,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        let Some(vm) = vm else { return };

        // Check all possible action IDs
        for (idx, _row) in self.rows.iter().enumerate() {
            // Refresh
            if let Some(host) =
                ui.data(|d| d.get_temp::<String>(egui::Id::new(format!("ssh_refresh_{}", idx))))
            {
                let _ = vm.get_linux_status(host.clone());
                let _ = vm.get_docker_status(host.clone());
                let _ = vm.get_ansible_status(host.clone());
                let _ = vm.get_dure_wss_status(host);
            }

            // Docker operations
            let docker_install_id = egui::Id::new(format!("ssh_install_docker_{}", idx));
            if let Some(host) = ui.data(|d| d.get_temp::<String>(docker_install_id)) {
                // Clear the temp data immediately to prevent continuous triggering
                ui.data_mut(|d| d.remove::<String>(docker_install_id));

                eprintln!("🔴 UI: Install Docker button clicked for host: {}", host);

                // Check if already in progress
                if self.show_docker_progress && !self.docker_progress_complete {
                    eprintln!("⚠️  Docker installation already in progress, ignoring click");
                } else {
                    // Show progress dialog immediately
                    self.show_docker_progress = true;
                    self.docker_progress_host = host.clone();
                    self.docker_progress_messages.clear();
                    self.docker_progress_error = None;
                    self.docker_progress_complete = false;
                    self.docker_progress_messages.push("Starting Docker installation...".to_string());

                    let _ = vm.install_docker(host);
                }
            }
            if let Some(host) = ui
                .data(|d| d.get_temp::<String>(egui::Id::new(format!("ssh_docker_status_{}", idx))))
            {
                let _ = vm.get_docker_status(host);
            }
            if let Some(_host) = ui.data(|d| {
                d.get_temp::<String>(egui::Id::new(format!("ssh_install_docker_image_{}", idx)))
            }) {
                self.show_docker_install_dialog = true;
                self.docker_install_host_idx = Some(idx);
                self.docker_image_input.clear();
                self.docker_container_name.clear();
                self.docker_tag = "latest".to_string();
                self.docker_metadata = None;
                self.docker_port_mappings.clear();
                self.docker_env_vars.clear();
                self.docker_validating = false;
                self.docker_validation_error = None;
            }
            if let Some(host) = ui.data(|d| {
                d.get_temp::<String>(egui::Id::new(format!("ssh_uninstall_docker_{}", idx)))
            }) {
                let _ = vm.uninstall_docker(host);
            }

            // Ansible operations
            let ansible_install_id = egui::Id::new(format!("ssh_install_ansible_{}", idx));
            if let Some(host) = ui.data(|d| d.get_temp::<String>(ansible_install_id)) {
                // Clear the temp data immediately to prevent continuous triggering
                ui.data_mut(|d| d.remove::<String>(ansible_install_id));

                eprintln!("🔵 UI: Install Ansible button clicked for host: {}", host);

                // Check if already in progress
                if self.show_ansible_progress && !self.ansible_progress_complete {
                    eprintln!("⚠️  Ansible installation already in progress, ignoring click");
                } else {
                    // Show progress dialog immediately
                    self.show_ansible_progress = true;
                    self.ansible_progress_host = host.clone();
                    self.ansible_progress_messages.clear();
                    self.ansible_progress_error = None;
                    self.ansible_progress_complete = false;
                    self.ansible_progress_messages.push("Starting Ansible installation...".to_string());

                    let _ = vm.install_ansible(host);
                }
            }
            if let Some(host) = ui.data(|d| {
                d.get_temp::<String>(egui::Id::new(format!("ssh_ansible_status_{}", idx)))
            }) {
                let _ = vm.get_ansible_status(host);
            }
            if let Some(_host) = ui.data(|d| {
                d.get_temp::<String>(egui::Id::new(format!("ssh_install_ansible_role_{}", idx)))
            }) {
                self.show_ansible_install_dialog = true;
                self.ansible_install_host_idx = Some(idx);
                self.ansible_role_input.clear();
                self.ansible_instance_name.clear();
                self.ansible_metadata = None;
                self.ansible_ports.clear();
                self.ansible_variables.clear();
                self.ansible_validating = false;
                self.ansible_validation_error = None;
            }
            if let Some(host) = ui.data(|d| {
                d.get_temp::<String>(egui::Id::new(format!("ssh_uninstall_ansible_{}", idx)))
            }) {
                let _ = vm.uninstall_ansible(host);
            }

            // Dure-WSS operations
            if let Some(_host) = ui.data(|d| {
                d.get_temp::<String>(egui::Id::new(format!("ssh_install_dure_wss_{}", idx)))
            }) {
                self.show_dure_wss_install_dialog = true;
                self.dure_wss_install_host_idx = Some(idx);
                self.dure_wss_domain.clear();
                self.dure_wss_email.clear();
                self.dure_wss_channel = "stable".to_string();
                self.dure_wss_variant = "default".to_string();
            }
            if let Some(host) = ui.data(|d| {
                d.get_temp::<String>(egui::Id::new(format!("ssh_dure_wss_status_{}", idx)))
            }) {
                let _ = vm.get_dure_wss_status(host);
            }
            if let Some(host) = ui.data(|d| {
                d.get_temp::<String>(egui::Id::new(format!("ssh_uninstall_dure_wss_{}", idx)))
            }) {
                let _ = vm.uninstall_dure_wss(host);
            }

            // Delete
            if let Some(host) =
                ui.data(|d| d.get_temp::<String>(egui::Id::new(format!("ssh_delete_{}", idx))))
            {
                let _ = vm.delete_ssh_host(host);
            }
        }
    }

    /// Render the SSH hosts table with drawers
    fn render_table(&mut self, ui: &mut egui::Ui, vm: Option<&mut crate::viewmodel::ViewModel>) {
        use egui_material3::data_table;

        let table_id = egui::Id::new("ssh_table");

        // Initialize drawer state (all closed by default)
        use egui_material3::datatable::DataTableState;
        let state: DataTableState = ui.data_mut(|d| {
            d.get_persisted::<DataTableState>(table_id)
                .unwrap_or_default()
        });
        ui.data_mut(|d| d.insert_persisted(table_id, state));

        // Build table
        let mut table = data_table()
            .id(table_id)
            .allow_selection(false)
            .allow_drawer(true)
            .column("Host (Port)", 200.0, false)
            .column("Platform", 150.0, false)
            .column("Status", 300.0, false)
            .column("Operations", 350.0, false);

        for (idx, row) in self.rows.iter().enumerate() {
            let row_for_cells = row.clone();
            let row_for_drawer = row.clone();
            let row_for_ops = row.clone();

            table = table.row(move |r| {
                r.cell(&format!("{}:{}", row_for_cells.host, row_for_cells.port))
                    .cell(&format_platform(&row_for_cells))
                    .cell(&format_status(&row_for_cells))
                    .widget_cell(move |ui| {
                        render_operations(ui, &row_for_ops, idx);
                    })
                    .drawer(move |ui| {
                        render_drawer_content(ui, &row_for_drawer);
                    })
            });
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            table.show(ui);
        });

        // Process action triggers from operations buttons
        self.process_action_triggers(ui, vm);
    }

    /// Render Docker image installation dialog
    fn render_docker_install_dialog(
        &mut self,
        ctx: &egui::Context,
        mut vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        use egui_material3::MaterialButton;

        let mut dialog_open = self.show_docker_install_dialog;

        egui::Window::new("Install Docker Image")
            .collapsible(false)
            .resizable(true)
            .default_width(600.0)
            .open(&mut dialog_open)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(8.0);

                    // Image input with validation
                    ui.horizontal(|ui| {
                        ui.label("Image:");
                        let response = ui.text_edit_singleline(&mut self.docker_image_input);
                        if response.lost_focus() && !self.docker_image_input.is_empty() {
                            if let Some(ref mut vm) = vm {
                                self.docker_validating = true;
                                self.docker_validation_error = None;
                                vm.validate_docker_image(self.docker_image_input.clone());
                            }
                        }
                    });
                    ui.label("Format: owner/repository or library/image");
                    ui.add_space(4.0);

                    // Validation status
                    if self.docker_validating {
                        ui.spinner();
                        ui.label("Validating image...");
                    } else if let Some(error) = &self.docker_validation_error {
                        ui.colored_label(egui::Color32::RED, format!("⚠ {}", error));
                    } else if let Some(ref metadata) = self.docker_metadata {
                        ui.colored_label(egui::Color32::GREEN, "✓ Image validated");
                        ui.label(format!("Description: {}", metadata.description));
                    }
                    ui.add_space(8.0);

                    // Tag selection
                    ui.horizontal(|ui| {
                        ui.label("Tag:");
                        egui::ComboBox::from_id_salt("docker_tag")
                            .selected_text(&self.docker_tag)
                            .show_ui(ui, |ui| {
                                if let Some(ref metadata) = self.docker_metadata {
                                    for tag in &metadata.tags {
                                        ui.selectable_value(&mut self.docker_tag, tag.clone(), tag);
                                    }
                                } else {
                                    ui.selectable_value(&mut self.docker_tag, "latest".to_string(), "latest");
                                }
                            });
                    });
                    ui.add_space(8.0);

                    // Container name
                    ui.horizontal(|ui| {
                        ui.label("Container Name:");
                        ui.text_edit_singleline(&mut self.docker_container_name);
                    });
                    ui.add_space(8.0);

                    // Port mappings
                    ui.label("Port Mappings:");
                    ui.add_space(4.0);

                    let mut to_remove = None;
                    for (idx, (host_port, container_port)) in self.docker_port_mappings.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label("Host:");
                            ui.add(egui::TextEdit::singleline(host_port).desired_width(80.0));
                            ui.label("→ Container:");
                            ui.add(egui::TextEdit::singleline(container_port).desired_width(80.0));
                            if ui.button("−").clicked() {
                                to_remove = Some(idx);
                            }
                        });
                    }
                    if let Some(idx) = to_remove {
                        self.docker_port_mappings.remove(idx);
                    }

                    if ui.add(MaterialButton::text("+ Add Port Mapping")).clicked() {
                        self.docker_port_mappings.push(("".to_string(), "".to_string()));
                    }
                    ui.add_space(8.0);

                    // Environment variables
                    ui.label("Environment Variables:");
                    ui.add_space(4.0);

                    let mut to_remove_env = None;
                    for (idx, (key, value)) in self.docker_env_vars.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(key).desired_width(150.0).hint_text("KEY"));
                            ui.label("=");
                            ui.add(egui::TextEdit::singleline(value).desired_width(200.0).hint_text("value"));
                            if ui.button("−").clicked() {
                                to_remove_env = Some(idx);
                            }
                        });
                    }
                    if let Some(idx) = to_remove_env {
                        self.docker_env_vars.remove(idx);
                    }

                    if ui.add(MaterialButton::text("+ Add Environment Variable")).clicked() {
                        self.docker_env_vars.push(("".to_string(), "".to_string()));
                    }
                    ui.add_space(16.0);

                    // Action buttons
                    ui.horizontal(|ui| {
                        let can_install = !self.docker_image_input.is_empty()
                            && !self.docker_container_name.is_empty()
                            && self.docker_metadata.is_some()
                            && !self.docker_validating;

                        if ui
                            .add_enabled(can_install, MaterialButton::filled("Install"))
                            .clicked()
                        {
                            if let Some(ref mut vm) = vm {
                                if let Some(host_idx) = self.docker_install_host_idx {
                                    if let Some(row) = self.rows.get(host_idx) {
                                        // Parse port mappings
                                        let ports: Vec<(u16, u16)> = self
                                            .docker_port_mappings
                                            .iter()
                                            .filter_map(|(h, c)| {
                                                let host = h.parse::<u16>().ok()?;
                                                let container = c.parse::<u16>().ok()?;
                                                Some((host, container))
                                            })
                                            .collect();

                                        // Filter out empty env vars
                                        let env: Vec<(String, String)> = self
                                            .docker_env_vars
                                            .iter()
                                            .filter(|(k, _)| !k.is_empty())
                                            .cloned()
                                            .collect();

                                        vm.install_docker_image(
                                            row.host.clone(),
                                            self.docker_container_name.clone(),
                                            self.docker_image_input.clone(),
                                            self.docker_tag.clone(),
                                            ports,
                                            env,
                                        );
                                    }
                                }
                            }
                        }

                        if ui.add(MaterialButton::text("Cancel")).clicked() {
                            self.show_docker_install_dialog = false;
                        }
                    });
                });
            });

        self.show_docker_install_dialog = dialog_open;
    }

    /// Render Ansible role installation dialog
    fn render_ansible_install_dialog(
        &mut self,
        ctx: &egui::Context,
        mut vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        use egui_material3::MaterialButton;

        let mut dialog_open = self.show_ansible_install_dialog;

        egui::Window::new("Install Ansible Role")
            .collapsible(false)
            .resizable(true)
            .default_width(600.0)
            .open(&mut dialog_open)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(8.0);

                    // Role input with validation
                    ui.horizontal(|ui| {
                        ui.label("Galaxy Role:");
                        let response = ui.text_edit_singleline(&mut self.ansible_role_input);
                        if response.lost_focus() && !self.ansible_role_input.is_empty() {
                            if let Some(ref mut vm) = vm {
                                self.ansible_validating = true;
                                self.ansible_validation_error = None;
                                vm.validate_ansible_role(self.ansible_role_input.clone());
                            }
                        }
                    });
                    ui.label("Format: namespace.role_name");
                    ui.add_space(4.0);

                    // Validation status
                    if self.ansible_validating {
                        ui.spinner();
                        ui.label("Validating role...");
                    } else if let Some(error) = &self.ansible_validation_error {
                        ui.colored_label(egui::Color32::RED, format!("⚠ {}", error));
                    } else if let Some(ref metadata) = self.ansible_metadata {
                        ui.colored_label(egui::Color32::GREEN, "✓ Role validated");
                        ui.label(format!("Description: {}", metadata.description));
                    }
                    ui.add_space(8.0);

                    // Instance name
                    ui.horizontal(|ui| {
                        ui.label("Instance Name:");
                        ui.text_edit_singleline(&mut self.ansible_instance_name);
                    });
                    ui.label("Unique name for this instance");
                    ui.add_space(8.0);

                    // Ports
                    ui.label("Exposed Ports:");
                    ui.add_space(4.0);

                    let mut to_remove = None;
                    for (idx, port) in self.ansible_ports.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label("Port:");
                            ui.add(egui::TextEdit::singleline(port).desired_width(80.0));
                            if ui.button("−").clicked() {
                                to_remove = Some(idx);
                            }
                        });
                    }
                    if let Some(idx) = to_remove {
                        self.ansible_ports.remove(idx);
                    }

                    if ui.add(MaterialButton::text("+ Add Port")).clicked() {
                        self.ansible_ports.push("".to_string());
                    }
                    ui.add_space(8.0);

                    // Variables
                    ui.label("Variables:");
                    ui.add_space(4.0);

                    let mut to_remove_var = None;
                    for (idx, (key, value)) in self.ansible_variables.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            ui.add(egui::TextEdit::singleline(key).desired_width(150.0).hint_text("variable"));
                            ui.label("=");
                            ui.add(egui::TextEdit::singleline(value).desired_width(200.0).hint_text("value"));
                            if ui.button("−").clicked() {
                                to_remove_var = Some(idx);
                            }
                        });
                    }
                    if let Some(idx) = to_remove_var {
                        self.ansible_variables.remove(idx);
                    }

                    if ui.add(MaterialButton::text("+ Add Variable")).clicked() {
                        self.ansible_variables.push(("".to_string(), "".to_string()));
                    }
                    ui.add_space(16.0);

                    // Action buttons
                    ui.horizontal(|ui| {
                        let can_install = !self.ansible_role_input.is_empty()
                            && !self.ansible_instance_name.is_empty()
                            && self.ansible_metadata.is_some()
                            && !self.ansible_validating;

                        if ui
                            .add_enabled(can_install, MaterialButton::filled("Install"))
                            .clicked()
                        {
                            if let Some(ref mut vm) = vm {
                                if let Some(host_idx) = self.ansible_install_host_idx {
                                    if let Some(row) = self.rows.get(host_idx) {
                                        // Parse ports
                                        let ports: Vec<u16> = self
                                            .ansible_ports
                                            .iter()
                                            .filter_map(|p| p.parse::<u16>().ok())
                                            .collect();

                                        // Filter out empty variables
                                        let variables: Vec<(String, String)> = self
                                            .ansible_variables
                                            .iter()
                                            .filter(|(k, _)| !k.is_empty())
                                            .cloned()
                                            .collect();

                                        vm.install_ansible_role(
                                            row.host.clone(),
                                            self.ansible_instance_name.clone(),
                                            self.ansible_role_input.clone(),
                                            variables,
                                            ports,
                                        );
                                    }
                                }
                            }
                        }

                        if ui.add(MaterialButton::text("Cancel")).clicked() {
                            self.show_ansible_install_dialog = false;
                        }
                    });
                });
            });

        self.show_ansible_install_dialog = dialog_open;
    }

    /// Render Dure-WSS service installation dialog
    fn render_dure_wss_install_dialog(
        &mut self,
        ctx: &egui::Context,
        mut vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        use egui_material3::MaterialButton;

        let mut dialog_open = self.show_dure_wss_install_dialog;

        egui::Window::new("Install Dure-WSS Service")
            .collapsible(false)
            .resizable(true)
            .default_width(600.0)
            .open(&mut dialog_open)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(8.0);

                    // Domain input
                    ui.horizontal(|ui| {
                        ui.label("Domain:");
                        ui.text_edit_singleline(&mut self.dure_wss_domain);
                    });
                    ui.label("Full domain name (e.g., shop.example.com)");
                    ui.add_space(8.0);

                    // Email input (for ACME)
                    ui.horizontal(|ui| {
                        ui.label("Email:");
                        ui.text_edit_singleline(&mut self.dure_wss_email);
                    });
                    ui.label("Email for ACME certificate notifications");
                    ui.add_space(8.0);

                    // Channel selection
                    ui.horizontal(|ui| {
                        ui.label("Channel:");
                        egui::ComboBox::from_id_salt("dure_wss_channel")
                            .selected_text(&self.dure_wss_channel)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.dure_wss_channel, "stable".to_string(), "stable");
                                ui.selectable_value(&mut self.dure_wss_channel, "beta".to_string(), "beta");
                                ui.selectable_value(&mut self.dure_wss_channel, "nightly".to_string(), "nightly");
                            });
                    });
                    ui.label("Release channel");
                    ui.add_space(8.0);

                    // Variant selection
                    ui.horizontal(|ui| {
                        ui.label("Variant:");
                        egui::ComboBox::from_id_salt("dure_wss_variant")
                            .selected_text(&self.dure_wss_variant)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.dure_wss_variant, "default".to_string(), "default");
                                ui.selectable_value(&mut self.dure_wss_variant, "minimal".to_string(), "minimal");
                            });
                    });
                    ui.label("Installation variant");
                    ui.add_space(16.0);

                    // Action buttons
                    ui.horizontal(|ui| {
                        let can_install = !self.dure_wss_domain.is_empty()
                            && !self.dure_wss_email.is_empty();

                        if ui
                            .add_enabled(can_install, MaterialButton::filled("Install"))
                            .clicked()
                        {
                            if let Some(ref mut vm) = vm {
                                if let Some(host_idx) = self.dure_wss_install_host_idx {
                                    if let Some(row) = self.rows.get(host_idx) {
                                        // Show progress dialog
                                        self.show_dure_wss_progress = true;
                                        self.dure_wss_progress_host = row.host.clone();
                                        self.dure_wss_progress_messages.clear();
                                        self.dure_wss_progress_error = None;
                                        self.dure_wss_progress_messages.push("Starting Dure-WSS installation...".to_string());

                                        vm.install_dure_wss(
                                            row.host.clone(),
                                            self.dure_wss_domain.clone(),
                                            self.dure_wss_email.clone(),
                                            self.dure_wss_channel.clone(),
                                            self.dure_wss_variant.clone(),
                                        );

                                        // Close the install dialog
                                        self.show_dure_wss_install_dialog = false;
                                    }
                                }
                            }
                        }

                        if ui.add(MaterialButton::text("Cancel")).clicked() {
                            self.show_dure_wss_install_dialog = false;
                        }
                    });
                });
            });

        self.show_dure_wss_install_dialog = dialog_open;
    }

    /// Render Docker installation progress dialog
    fn render_docker_progress(&mut self, ctx: &egui::Context) {
        use egui_material3::MaterialButton;

        let mut close_clicked = false;
        egui::Window::new("Installing Docker")
            .collapsible(false)
            .resizable(false)
            .default_width(500.0)
            .open(&mut self.show_docker_progress)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.label(format!("Host: {}", self.docker_progress_host));
                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for msg in &self.docker_progress_messages {
                            ui.label(msg);
                        }
                    });

                ui.add_space(8.0);

                if let Some(error) = &self.docker_progress_error {
                    ui.colored_label(egui::Color32::RED, format!("✗ Error: {}", error));
                    ui.add_space(8.0);
                }

                // Only allow closing when installation is complete or has error
                if self.docker_progress_complete {
                    if ui.add(MaterialButton::text("Close")).clicked() {
                        close_clicked = true;
                    }
                } else {
                    ui.add_enabled(false, MaterialButton::text("Installing..."));
                    ui.label("Please wait while Docker is being installed.");
                }
            });

        if close_clicked {
            self.show_docker_progress = false;
        }
    }

    /// Render Ansible installation progress dialog
    fn render_ansible_progress(&mut self, ctx: &egui::Context) {
        use egui_material3::MaterialButton;

        let mut close_clicked = false;
        egui::Window::new("Installing Ansible")
            .collapsible(false)
            .resizable(false)
            .default_width(500.0)
            .open(&mut self.show_ansible_progress)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.label(format!("Host: {}", self.ansible_progress_host));
                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for msg in &self.ansible_progress_messages {
                            ui.label(msg);
                        }
                    });

                ui.add_space(8.0);

                if let Some(error) = &self.ansible_progress_error {
                    ui.colored_label(egui::Color32::RED, format!("✗ Error: {}", error));
                    ui.add_space(8.0);
                }

                // Only allow closing when installation is complete or has error
                if self.ansible_progress_complete {
                    if ui.add(MaterialButton::text("Close")).clicked() {
                        close_clicked = true;
                    }
                } else {
                    ui.add_enabled(false, MaterialButton::text("Installing..."));
                    ui.label("Please wait while Ansible is being installed.");
                }
            });

        if close_clicked {
            self.show_ansible_progress = false;
        }
    }

    /// Render Dure-WSS installation progress dialog
    fn render_dure_wss_progress(&mut self, ctx: &egui::Context) {
        use egui_material3::MaterialButton;

        let mut close_clicked = false;
        egui::Window::new("Installing Dure-WSS")
            .collapsible(false)
            .resizable(false)
            .default_width(500.0)
            .open(&mut self.show_dure_wss_progress)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.label(format!("Host: {}", self.dure_wss_progress_host));
                ui.add_space(8.0);

                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for msg in &self.dure_wss_progress_messages {
                            ui.label(msg);
                        }
                    });

                ui.add_space(8.0);

                if let Some(error) = &self.dure_wss_progress_error {
                    ui.colored_label(egui::Color32::RED, format!("✗ Error: {}", error));
                    ui.add_space(8.0);
                }

                // Only allow closing when installation is complete or has error
                if self.dure_wss_progress_complete {
                    if ui.add(MaterialButton::text("Close")).clicked() {
                        close_clicked = true;
                    }
                } else {
                    ui.add_enabled(false, MaterialButton::text("Installing..."));
                    ui.label("Please wait while Dure-WSS is being installed.");
                }
            });

        if close_clicked {
            self.show_dure_wss_progress = false;
        }
    }

    /// Render the SSH tab UI
    pub fn ui(&mut self, ui: &mut egui::Ui, mut vm: Option<&mut crate::viewmodel::ViewModel>) {
        use egui_material3::MaterialButton;

        // 1. Process ViewModel events
        if let Some(ref mut vm) = vm {
            let events = vm.poll_events(ui.ctx());
            for event in events {
                self.handle_event(event);
            }

            // 2. Show active operations with progress bars
            for (_op_id, progress) in vm.active_operations() {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::ProgressBar::new(progress.progress)
                            .text(format!("{}: {}", progress.operation, progress.status))
                            .desired_width(400.0),
                    );
                });
            }

            // 3. Show recent errors
            if let Some(error) = vm
                .recent_errors()
                .iter()
                .filter(|e| e.actor == "ssh")
                .rev()
                .next()
            {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 100, 100),
                    format!("⚠ Error in {}: {}", error.operation, error.error),
                );
                ui.add_space(4.0);
            }
        }

        // 4. Header
        ui.heading("SSH Hosts");
        ui.add_space(4.0);
        ui.label("Manage SSH hosts for remote server deployment and management.");
        ui.add_space(8.0);

        // 5. Add Host button
        if ui.add(MaterialButton::filled("Add Host")).clicked() {
            self.show_add_dialog = true;
            self.add_host.clear();
            self.add_password.clear();
            self.add_private_key_path.clear();
            self.add_port = "22".to_string();
            self.add_use_password = false;
            self.add_use_private_key = false;
        }
        ui.add_space(8.0);

        // 6. Load rows on demand
        if !self.loaded {
            self.load_rows();
            self.loaded = true;
        }

        // 7. Error display
        if let Some(error) = &self.load_error {
            ui.colored_label(egui::Color32::RED, format!("⚠ {}", error));
            ui.add_space(4.0);
        }

        // 8. Render table or empty state
        if self.rows.is_empty() {
            ui.label("No SSH hosts configured. Click 'Add Host' to get started.");
        } else {
            self.render_table(ui, vm.as_deref_mut());
        }

        // 9. Dialogs
        if self.show_add_dialog {
            self.render_add_dialog(ui.ctx(), vm.as_deref_mut());
        }
        if self.show_docker_install_dialog {
            self.render_docker_install_dialog(ui.ctx(), vm.as_deref_mut());
        }
        if self.show_ansible_install_dialog {
            self.render_ansible_install_dialog(ui.ctx(), vm.as_deref_mut());
        }
        if self.show_dure_wss_install_dialog {
            self.render_dure_wss_install_dialog(ui.ctx(), vm.as_deref_mut());
        }

        // Progress dialogs
        if self.show_docker_progress {
            self.render_docker_progress(ui.ctx());
        }
        if self.show_ansible_progress {
            self.render_ansible_progress(ui.ctx());
        }
        if self.show_dure_wss_progress {
            self.render_dure_wss_progress(ui.ctx());
        }
    }

    /* OLD ui() implementation - removed in Task 14
    fn old_ui_commented_out() {
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
    }
    */

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

    fn render_add_dialog(
        &mut self,
        ctx: &egui::Context,
        mut vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
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

            let ssh_key_path = if self.add_use_private_key && !self.add_private_key_path.is_empty()
            {
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

    fn execute_delete_host(
        &mut self,
        host: String,
        mut vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
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
            "  (status not loaded - click Refresh to load)",
        );
    }

    ui.add_space(4.0);

    // Docker containers
    ui.label(egui::RichText::new("docker:").strong());
    if row.docker_containers.is_empty() {
        ui.colored_label(ui.visuals().weak_text_color(), "  (no containers installed)");
    } else {
        for container in &row.docker_containers {
            ui.label(format!("  • {} ({}:{})",
                container.name,
                container.image,
                container.tag
            ));
            if !container.ports.is_empty() {
                let ports: Vec<String> = container.ports.iter()
                    .map(|(h, c)| format!("{}→{}", h, c))
                    .collect();
                ui.label(format!("    ports: {}", ports.join(", ")));
            }
        }
    }

    ui.add_space(4.0);

    // Ansible roles
    ui.label(egui::RichText::new("ansible:").strong());
    if row.ansible_roles.is_empty() {
        ui.colored_label(ui.visuals().weak_text_color(), "  (no roles installed)");
    } else {
        for role in &row.ansible_roles {
            ui.label(format!("  • {} ({})",
                role.name,
                role.galaxy_name
            ));
            if !role.ports.is_empty() {
                let ports: Vec<String> = role.ports.iter()
                    .map(|p| p.to_string())
                    .collect();
                ui.label(format!("    ports: {}", ports.join(", ")));
            }
        }
    }

    ui.add_space(4.0);

    // Dure-WSS service
    ui.label(egui::RichText::new("dure-wss:").strong());
    if let Some(config) = &row.dure_wss_config {
        ui.label(format!("  • domain: {}", config.domain));
        ui.label(format!("    channel: {}, variant: {}", config.channel, config.variant));
    } else {
        ui.colored_label(ui.visuals().weak_text_color(), "  (not installed)");
    }

    ui.add_space(4.0);
}

/// Render dynamic operation buttons based on service state
fn render_operations(ui: &mut egui::Ui, row: &SshRowData, idx: usize) {
    use egui_material3::MaterialButton;

    egui::ScrollArea::horizontal()
        .id_salt(format!("operations_scroll_{}", idx))
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                ui.style_mut().spacing.button_padding = egui::vec2(6.0, 2.0);

                // Refresh - always available
                if ui
                    .add(MaterialButton::outlined("Refresh").small())
                    .on_hover_text("Refresh host status")
                    .clicked()
                {
                    ui.data_mut(|d| {
                        d.insert_temp(
                            egui::Id::new(format!("ssh_refresh_{}", idx)),
                            row.host.clone(),
                        )
                    });
                }

                // Docker operations - dynamic based on state
                if !row.docker_enabled {
                    if ui
                        .add(MaterialButton::outlined("Install Docker").small())
                        .on_hover_text("Install Docker")
                        .clicked()
                    {
                        ui.data_mut(|d| {
                            d.insert_temp(
                                egui::Id::new(format!("ssh_install_docker_{}", idx)),
                                row.host.clone(),
                            )
                        });
                    }
                } else {
                    if ui
                        .add(MaterialButton::outlined("Docker Status").small())
                        .on_hover_text("Check Docker status")
                        .clicked()
                    {
                        ui.data_mut(|d| {
                            d.insert_temp(
                                egui::Id::new(format!("ssh_docker_status_{}", idx)),
                                row.host.clone(),
                            )
                        });
                    }
                    if ui
                        .add(MaterialButton::outlined("Install Image").small())
                        .on_hover_text("Install Docker image")
                        .clicked()
                    {
                        ui.data_mut(|d| {
                            d.insert_temp(
                                egui::Id::new(format!("ssh_install_docker_image_{}", idx)),
                                row.host.clone(),
                            )
                        });
                    }
                    if ui
                        .add(MaterialButton::outlined("Uninstall Docker").small())
                        .on_hover_text("Uninstall Docker")
                        .clicked()
                    {
                        ui.data_mut(|d| {
                            d.insert_temp(
                                egui::Id::new(format!("ssh_uninstall_docker_{}", idx)),
                                row.host.clone(),
                            )
                        });
                    }
                }

                // Ansible operations - similar pattern
                if !row.ansible_enabled {
                    if ui
                        .add(MaterialButton::outlined("Install Ansible").small())
                        .on_hover_text("Install Ansible (placeholder)")
                        .clicked()
                    {
                        ui.data_mut(|d| {
                            d.insert_temp(
                                egui::Id::new(format!("ssh_install_ansible_{}", idx)),
                                row.host.clone(),
                            )
                        });
                    }
                } else {
                    if ui
                        .add(MaterialButton::outlined("Ansible Status").small())
                        .clicked()
                    {
                        ui.data_mut(|d| {
                            d.insert_temp(
                                egui::Id::new(format!("ssh_ansible_status_{}", idx)),
                                row.host.clone(),
                            )
                        });
                    }
                    if ui
                        .add(MaterialButton::outlined("Install Role").small())
                        .on_hover_text("Install Ansible role")
                        .clicked()
                    {
                        ui.data_mut(|d| {
                            d.insert_temp(
                                egui::Id::new(format!("ssh_install_ansible_role_{}", idx)),
                                row.host.clone(),
                            )
                        });
                    }
                    if ui
                        .add(MaterialButton::outlined("Uninstall Ansible").small())
                        .clicked()
                    {
                        ui.data_mut(|d| {
                            d.insert_temp(
                                egui::Id::new(format!("ssh_uninstall_ansible_{}", idx)),
                                row.host.clone(),
                            )
                        });
                    }
                }

                // Dure-WSS operations - similar pattern
                if !row.dure_wss_enabled {
                    if ui
                        .add(MaterialButton::outlined("Install Dure-WSS").small())
                        .on_hover_text("Install Dure-WSS (placeholder)")
                        .clicked()
                    {
                        ui.data_mut(|d| {
                            d.insert_temp(
                                egui::Id::new(format!("ssh_install_dure_wss_{}", idx)),
                                row.host.clone(),
                            )
                        });
                    }
                } else {
                    if ui
                        .add(MaterialButton::outlined("Dure-WSS Status").small())
                        .clicked()
                    {
                        ui.data_mut(|d| {
                            d.insert_temp(
                                egui::Id::new(format!("ssh_dure_wss_status_{}", idx)),
                                row.host.clone(),
                            )
                        });
                    }
                    if ui
                        .add(MaterialButton::outlined("Uninstall Dure-WSS").small())
                        .clicked()
                    {
                        ui.data_mut(|d| {
                            d.insert_temp(
                                egui::Id::new(format!("ssh_uninstall_dure_wss_{}", idx)),
                                row.host.clone(),
                            )
                        });
                    }
                }

                // Delete - always available
                if ui
                    .add(MaterialButton::outlined("Delete").small())
                    .on_hover_text("Delete SSH host")
                    .clicked()
                {
                    ui.data_mut(|d| {
                        d.insert_temp(
                            egui::Id::new(format!("ssh_delete_{}", idx)),
                            row.host.clone(),
                        )
                    });
                }
            });
        });
}
