//! Platform tab - Platform configuration and management

use eframe::egui;
use egui_material3::spreadsheet::{text_column, MaterialSpreadsheet};
use egui_material3::MaterialButton;

use crate::calc::audit;
use crate::calc::gcp_rest::BillingRecord;
use crate::config::{AppConfig, CloudPlatformConfig};

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use crate::ui_dlg::platform_gcp::GcpWizard;

/// Platform tab state
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PlatformTab {
    selected_row: Option<usize>,
    /// Cached platform/project/VM rows (platform_name, status, actions)
    #[cfg_attr(feature = "serde", serde(skip))]
    rows: Vec<[String; 3]>,
    #[cfg_attr(feature = "serde", serde(skip))]
    loaded: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    load_error: Option<String>,

    // Spreadsheet
    #[cfg_attr(feature = "serde", serde(skip))]
    spreadsheet: Option<MaterialSpreadsheet>,
    row_selection_enabled: bool,

    // Add dialog state
    #[cfg_attr(feature = "serde", serde(skip))]
    show_add_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_name: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_type: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_oauth_result: Option<crate::api::gcp_oauth::OAuthResult>,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_oauth_promise: Option<poll_promise::Promise<Result<crate::api::gcp_oauth::OAuthResult, String>>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_connected_email: Option<String>,

    // Init progress state
    #[cfg_attr(feature = "serde", serde(skip))]
    init_in_progress: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    init_platform_name: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    init_progress_log: Vec<String>,

    // GCP wizard
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    #[cfg_attr(feature = "serde", serde(skip))]
    gcp_wizard: Option<GcpWizard>,

    // Track if wizard was open in previous frame (to detect closure)
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    #[cfg_attr(feature = "serde", serde(skip))]
    wizard_was_open: bool,

    // Platform summary cache (platform_name -> summary)
    #[cfg_attr(feature = "serde", serde(skip))]
    platform_summaries: std::collections::HashMap<String, String>,

    // Delete VM dialog state
    #[cfg_attr(feature = "serde", serde(skip))]
    show_delete_vm_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_vm_platform: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_vm_list: Vec<(String, String, String)>, // (name, zone, status)
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_vm_selected: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_vm_confirming: bool,

    // Delete Platform dialog state
    #[cfg_attr(feature = "serde", serde(skip))]
    show_delete_platform_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_platform_name: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_platform_vm_count: usize,

    // Billing dialog state
    #[cfg_attr(feature = "serde", serde(skip))]
    show_billing_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    billing_data: Option<Vec<BillingRecord>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    billing_loading: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    billing_error: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    billing_dataset: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    billing_table: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    billing_project_id: String,
}

impl Default for PlatformTab {
    fn default() -> Self {
        let spreadsheet = {
            let columns = vec![
                text_column("Platform Name", 400.0),
                text_column("Status", 300.0),
                text_column("Actions", 250.0),
            ];

            // Create spreadsheet with theme-aware settings
            MaterialSpreadsheet::new("vm_spreadsheet", columns)
                .ok()
                .map(|mut s| {
                    // Enable striped rows for Material Design theme colors
                    s.set_striped(true);
                    // Enable row selection
                    s.set_row_selection_enabled(true);
                    // Enable selection for better visual feedback
                    s.set_allow_selection(true);
                    s
                })
        };

        Self {
            selected_row: None,
            rows: Vec::new(),
            loaded: false,
            load_error: None,
            spreadsheet,
            row_selection_enabled: true,
            show_add_dialog: false,
            add_platform_name: String::new(),
            add_platform_type: "gcp".to_string(),
            add_platform_oauth_result: None,
            add_platform_oauth_promise: None,
            add_platform_connected_email: None,
            init_in_progress: false,
            init_platform_name: None,
            init_progress_log: Vec::new(),
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            gcp_wizard: None,
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            wizard_was_open: false,
            platform_summaries: std::collections::HashMap::new(),
            show_delete_vm_dialog: false,
            delete_vm_platform: String::new(),
            delete_vm_list: Vec::new(),
            delete_vm_selected: None,
            delete_vm_confirming: false,
            show_delete_platform_dialog: false,
            delete_platform_name: String::new(),
            delete_platform_vm_count: 0,
            show_billing_dialog: false,
            billing_data: None,
            billing_loading: false,
            billing_error: None,
            billing_dataset: String::new(),
            billing_table: String::new(),
            billing_project_id: String::new(),
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

impl PlatformTab {
    /// Render the platform tab UI
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Cloud Platforms");
        ui.add_space(4.0);
        ui.label(
            "Manage cloud service platforms (GCP, Firebase, Supabase) for deployment and hosting.",
        );
        ui.add_space(8.0);

        // Get selected row for delete button state
        let selected_row_idx = self.spreadsheet.as_ref().and_then(|s| s.get_selected_row());
        let has_selection = selected_row_idx.is_some();

        // Action buttons
        ui.horizontal(|ui| {
            if ui.add(MaterialButton::filled("Add Platform")).clicked() {
                self.show_add_dialog = true;
                self.add_platform_name.clear();
                self.add_platform_type = "gcp".to_string();
            }

            // Delete Platform button - enabled only when a row is selected
            let delete_platform_button = MaterialButton::outlined("Delete Platform");
            let delete_platform_button = if has_selection {
                delete_platform_button
            } else {
                delete_platform_button.enabled(false)
            };

            if ui.add(delete_platform_button).clicked() {
                if let Some(idx) = selected_row_idx {
                    if idx < self.rows.len() {
                        let platform_name = self.rows[idx][0].clone();
                        self.show_delete_platform_confirmation(platform_name);
                    }
                }
            }

            ui.add_space(8.0);

            // Add VM button - always enabled if we have at least one platform
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            {
                // Check if we have any GCP platforms
                let has_gcp_platform = if let Ok((app_config, _)) = load_config() {
                    app_config.platforms.iter().any(|p| p.platform_type == "gcp")
                } else {
                    false
                };

                let add_vm_button = MaterialButton::outlined("Add VM");
                let add_vm_button = if has_gcp_platform {
                    add_vm_button
                } else {
                    add_vm_button.enabled(false)
                };

                if ui.add(add_vm_button).clicked() {
                    // Find first GCP platform
                    if let Ok((app_config, _)) = load_config() {
                        if let Some(platform) = app_config
                            .platforms
                            .iter()
                            .find(|p| p.platform_type == "gcp")
                        {
                            self.show_gcp_wizard(platform.name.clone());
                        }
                    }
                }

                // Delete VM button - enabled only when a VM row is selected
                let delete_button = MaterialButton::outlined("Delete VM");
                let delete_button = if has_selection {
                    delete_button
                } else {
                    delete_button.enabled(false)
                };

                if ui.add(delete_button).clicked() {
                    if let Some(idx) = selected_row_idx {
                        if idx < self.rows.len() {
                            let platform_name = self.rows[idx][0].clone();
                            let row_type = &self.rows[idx][1];

                            // Only delete if it's a VM row
                            if let Some(vm_name) = row_type.strip_prefix("vm:") {
                                let zone = self.rows[idx][2].clone();
                                self.show_delete_vm_confirmation(platform_name, vm_name.to_string(), zone);
                            }
                        }
                    }
                }

                // Estimated Billing button - enabled only when we have GCP platforms
                let billing_button = MaterialButton::outlined("Estimated Billing");
                let billing_button = if has_gcp_platform {
                    billing_button
                } else {
                    billing_button.enabled(false)
                };

                if ui.add(billing_button).clicked() {
                    self.show_billing_dialog = true;
                    self.fetch_billing_data();
                }
            }

            if ui.add(MaterialButton::outlined("Refresh")).clicked() {
                self.loaded = false;
                self.load_error = None;
            }

            // Show selected info
            if let Some(idx) = selected_row_idx {
                if idx < self.rows.len() {
                    let platform = &self.rows[idx][0];
                    let row_type = &self.rows[idx][1];

                    if let Some(vm_name) = row_type.strip_prefix("vm:") {
                        ui.label(format!("│ Selected: {} / {}", platform, vm_name));
                    } else if let Some(project_id) = row_type.strip_prefix("project:") {
                        ui.label(format!("│ Selected: {} / {}", platform, project_id));
                    } else {
                        ui.label(format!("│ Selected: {} (account)", platform));
                    }
                }
            }
        });
        ui.add_space(8.0);

        // Lazy-load from DB on first render or after refresh
        if !self.loaded {
            self.load_rows();
            self.loaded = true;
        }

        if let Some(err) = &self.load_error {
            ui.colored_label(egui::Color32::RED, format!("⚠ {err}"));
            ui.add_space(4.0);
        }

        // Platform spreadsheet - fill remaining space
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

        // Add platform dialog
        if self.show_add_dialog {
            self.render_add_dialog(ui.ctx());
        }

        // Delete Platform dialog
        if self.show_delete_platform_dialog {
            self.render_delete_platform_dialog(ui.ctx());
        }

        // Delete VM dialog
        if self.show_delete_vm_dialog {
            self.render_delete_vm_dialog(ui.ctx());
        }

        // Billing dialog
        if self.show_billing_dialog {
            self.render_billing_dialog(ui.ctx());
        }

        // Init progress display
        if self.init_in_progress {
            self.render_init_progress(ui);
        }

        // GCP wizard dialog
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            let wizard_is_open = self.gcp_wizard.is_some();

            if let Some(wizard) = &mut self.gcp_wizard {
                wizard.ui(ui.ctx());
            }

            // Detect wizard closure - if it was open and now closed, refresh
            if self.wizard_was_open && !wizard_is_open {
                eprintln!("✓ GCP wizard closed, refreshing platform spreadsheet");
                self.loaded = false;
            }

            self.wizard_was_open = wizard_is_open;
        }
    }

    fn load_rows(&mut self) {
        self.rows.clear();
        self.load_error = None;

        #[cfg(not(target_arch = "wasm32"))]
        {
            match load_config() {
                Ok((app_config, _)) => {
                    let mut data_rows = Vec::new();

                    // Build hierarchical rows: Account → Project → VMs
                    for platform in &app_config.platforms {
                        // Only show GCP platforms for now
                        if platform.platform_type != "gcp" {
                            continue;
                        }

                        let email = platform.gcp_connected_email.as_deref().unwrap_or("Not connected");

                        // Account row: "GCP: email" | "N Projects" | ""
                        let account_name = format!("GCP: {}", email);
                        let project_count = "Loading..."; // TODO: fetch from API

                        self.rows.push([
                            platform.name.clone(), // Internal platform name
                            format!("account:{}", platform.name), // Row type marker
                            String::new(), // Empty for internal use
                        ]);

                        data_rows.push(vec![
                            account_name,
                            project_count.to_string(),
                            String::new(), // No actions for account row
                        ]);

                        // Project row (only if project is selected)
                        if let Some(project_id) = &platform.gcp_selected_project_id {
                            let project_name = format!("  ├─ {}", project_id);
                            let vm_count = platform.vms.len();
                            let firewall_status = format!("{} VM\n✗ GCP Firewall Not Whitelisted", vm_count);

                            self.rows.push([
                                platform.name.clone(),
                                format!("project:{}", project_id),
                                String::new(),
                            ]);

                            data_rows.push(vec![
                                project_name,
                                firewall_status,
                                String::new(), // Actions will be rendered as buttons
                            ]);

                            // VM rows
                            for vm in &platform.vms {
                                let vm_name = format!("  └─── {}", vm.name);
                                let ssh_status = "🔄 SSH Connection Testing...".to_string();

                                self.rows.push([
                                    platform.name.clone(),
                                    format!("vm:{}", vm.name),
                                    vm.zone.clone(), // Store zone for operations
                                ]);

                                data_rows.push(vec![
                                    vm_name,
                                    ssh_status,
                                    String::new(), // Actions will be rendered as buttons
                                ]);
                            }
                        }
                    }

                    // Clear and update spreadsheet with fresh data
                    if let Some(spreadsheet) = &mut self.spreadsheet {
                        // Recreate spreadsheet with fresh data to avoid duplicates
                        let columns = vec![
                            text_column("Platform Name", 400.0),
                            text_column("Status", 300.0),
                            text_column("Actions", 250.0),
                        ];

                        if let Ok(mut new_spreadsheet) =
                            MaterialSpreadsheet::new("vm_spreadsheet", columns)
                        {
                            // Apply theme-aware settings
                            new_spreadsheet.set_striped(true);
                            new_spreadsheet.set_row_selection_enabled(self.row_selection_enabled);
                            new_spreadsheet.set_allow_selection(true);
                            new_spreadsheet.init_with_data(data_rows);
                            *spreadsheet = new_spreadsheet;
                        }
                    }
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {e}"));
                }
            }
        }
    }

    /// Format details for a VM: B: billing, P: project, Z: zone
    fn format_vm_details(&self, _platform: &CloudPlatformConfig, vm: &crate::config::VmInstance) -> String {
        let mut parts = Vec::new();

        // Billing account
        if let Some(billing) = &vm.gcp_billing_account {
            parts.push(format!("B: {}", billing));
        } else {
            parts.push("B: -".to_string());
        }

        // Project
        parts.push(format!("P: {}", vm.gcp_project_id));

        // Zone
        parts.push(format!("Z: {}", vm.zone));

        parts.join(", ")
    }

    /// Format details for a platform with no VMs
    fn format_platform_details(&self, platform: &CloudPlatformConfig, _vm: Option<&crate::config::VmInstance>) -> String {
        match platform.platform_type.as_str() {
            "gcp" => {
                // GCP details are now in VMs
                "Platform configured".to_string()
            }
            "firebase" => {
                if let Some(project) = &platform.firebase_project_id {
                    format!("P: {}", project)
                } else {
                    "Firebase platform".to_string()
                }
            }
            "supabase" => {
                if let Some(url) = &platform.supabase_api_url {
                    format!("URL: {}", url)
                } else {
                    "Supabase platform".to_string()
                }
            }
            _ => "Platform configured".to_string(),
        }
    }

    /// Fetch GCP account summary (billing accounts, projects, VMs)
    #[cfg(not(target_arch = "wasm32"))]
    fn fetch_gcp_summary(&mut self, platform: &CloudPlatformConfig) -> Option<String> {
        use crate::calc::gcp_rest::GcpRestClient;

        // Check if we have a cached summary
        if let Some(cached) = self.platform_summaries.get(&platform.name) {
            return Some(cached.clone());
        }

        // Get access token
        let access_token = platform.gcp_oauth_access_token.as_ref()?;

        // Check token expiry
        let now = chrono::Utc::now().timestamp();
        if let Some(expiry) = platform.gcp_oauth_token_expiry {
            if now >= expiry {
                return Some("OAuth expired".to_string());
            }
        }

        let client = GcpRestClient::new(access_token.clone());
        let mut summary_parts = Vec::new();

        // Fetch billing accounts
        if let Ok(billing_list) = client.list_billing_accounts() {
            let count = billing_list.billing_accounts.len();
            if count > 0 {
                let name = &billing_list.billing_accounts[0].display_name;
                if count == 1 {
                    summary_parts.push(format!("1 billing account({})", name));
                } else {
                    summary_parts.push(format!("{} billing accounts({}...)", count, name));
                }
            }
        }

        // Fetch projects
        if let Ok(project_list) = client.list_projects(None) {
            let count = project_list.projects.len();
            if count > 0 {
                let name = &project_list.projects[0].project_id;
                if count == 1 {
                    summary_parts.push(format!("1 project({})", name));
                } else {
                    summary_parts.push(format!("{} projects({}...)", count, name));
                }
            }
        }

        // Show configured VMs
        let vm_count = platform.vms.len();
        if vm_count > 0 {
            let name = &platform.vms[0].name;
            if vm_count == 1 {
                summary_parts.push(format!("1 vm({})", name));
            } else {
                summary_parts.push(format!("{} vms({}...)", vm_count, name));
            }
        }

        if summary_parts.is_empty() {
            None
        } else {
            let summary = summary_parts.join(", ");
            // Cache the summary
            self.platform_summaries.insert(platform.name.clone(), summary.clone());
            Some(summary)
        }
    }

    fn render_add_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.show_add_dialog;

        egui::Window::new("Add Platform")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Configure a new cloud platform:");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.add_platform_name);
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("platform_type_combo")
                        .selected_text(&self.add_platform_type)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.add_platform_type,
                                "gcp".to_string(),
                                "GCP (Google Cloud Platform)",
                            );
                            // TODO: Re-enable when Firebase/Supabase support is implemented
                            // ui.selectable_value(&mut self.add_platform_type, "firebase".to_string(), "Firebase");
                            // ui.selectable_value(&mut self.add_platform_type, "supabase".to_string(), "Supabase");
                        });
                });

                ui.add_space(12.0);

                // Show OAuth connection for GCP
                if self.add_platform_type == "gcp" {
                    ui.separator();
                    ui.add_space(8.0);

                    // Check for OAuth promise result
                    if let Some(promise) = &self.add_platform_oauth_promise {
                        if let Some(result) = promise.ready() {
                            match result {
                                Ok(oauth_result) => {
                                    self.add_platform_oauth_result = Some(oauth_result.clone());
                                    // Fetch account email
                                    self.fetch_connected_email();
                                    self.add_platform_oauth_promise = None;
                                }
                                Err(e) => {
                                    self.load_error = Some(format!("OAuth failed: {}", e));
                                    self.add_platform_oauth_promise = None;
                                }
                            }
                        }
                    }

                    if let Some(email) = &self.add_platform_connected_email {
                        ui.colored_label(
                            egui::Color32::from_rgb(72, 187, 120),
                            format!("✓ Connected as: {}", email),
                        );
                    } else if self.add_platform_oauth_promise.is_some() {
                        ui.spinner();
                        ui.label("Waiting for authorization...");
                        ui.label("Please complete the OAuth flow in your browser.");
                    } else {
                        if ui.add(MaterialButton::outlined("Connect to Google Cloud")).clicked() {
                            self.start_add_platform_oauth();
                        }
                        ui.add_space(4.0);
                        ui.colored_label(
                            egui::Color32::GRAY,
                            "⚠ Connection required for GCP platforms",
                        );
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                }

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.show_add_dialog = false;
                        self.add_platform_oauth_result = None;
                        self.add_platform_oauth_promise = None;
                        self.add_platform_connected_email = None;
                    }

                    let can_add = !self.add_platform_name.is_empty()
                        && (self.add_platform_type != "gcp"
                            || self.add_platform_connected_email.is_some());

                    ui.add_enabled_ui(can_add, |ui| {
                        if ui.button("Add").clicked() {
                            self.execute_add_platform();
                            self.show_add_dialog = false;
                            self.add_platform_oauth_result = None;
                            self.add_platform_oauth_promise = None;
                            self.add_platform_connected_email = None;
                        }
                    });

                    if !can_add {
                        if self.add_platform_name.is_empty() {
                            ui.label("⚠ Name required");
                        } else if self.add_platform_type == "gcp"
                            && self.add_platform_connected_email.is_none()
                        {
                            ui.label("⚠ Connect to Google Cloud first");
                        }
                    }
                });
            });

        if !open {
            self.show_add_dialog = false;
            self.add_platform_oauth_result = None;
            self.add_platform_oauth_promise = None;
            self.add_platform_connected_email = None;
        }
    }

    fn execute_add_platform(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match load_config() {
                Ok((mut app_config, config_path)) => {
                    // Check if platform already exists
                    if app_config
                        .platforms
                        .iter()
                        .any(|p| p.name == self.add_platform_name)
                    {
                        self.load_error = Some(format!(
                            "Platform '{}' already exists",
                            self.add_platform_name
                        ));
                        return;
                    }

                    // Create new platform with OAuth info if GCP
                    let (oauth_access, oauth_refresh, oauth_expiry, connected_email) =
                        if self.add_platform_type == "gcp" {
                            if let Some(oauth) = &self.add_platform_oauth_result {
                                (
                                    Some(oauth.access_token.clone()),
                                    Some(oauth.refresh_token.clone()),
                                    Some(oauth.expires_at as i64),
                                    self.add_platform_connected_email.clone(),
                                )
                            } else {
                                (None, None, None, None)
                            }
                        } else {
                            (None, None, None, None)
                        };

                    let platform = CloudPlatformConfig {
                        name: self.add_platform_name.clone(),
                        platform_type: self.add_platform_type.clone(),
                        gcp_oauth_access_token: oauth_access,
                        gcp_oauth_refresh_token: oauth_refresh,
                        gcp_oauth_token_expiry: oauth_expiry,
                        gcp_connected_email: connected_email,
                        gcp_selected_project_id: None,
                        firebase_project_id: None,
                        firebase_api_key: None,
                        supabase_project_ref: None,
                        supabase_api_url: None,
                        supabase_anon_key: None,
                        api_token: None,
                        service_account_json: None,
                        vms: Vec::new(),
                    };

                    // Add to config
                    app_config.platforms.push(platform);

                    // Save config
                    match app_config.save(&config_path) {
                        Ok(_) => {
                            // Record audit event
                            match audit::push_gui(
                                "system",
                                "desktop",
                                "platform add",
                                &self.add_platform_name,
                            ) {
                                Ok(audit_id) => {
                                    eprintln!("✓ Audit record created: ID {}", audit_id);
                                }
                                Err(e) => {
                                    eprintln!("⚠ Failed to record audit event: {}", e);
                                    self.load_error = Some(format!("Audit tracking failed: {}", e));
                                }
                            }

                            eprintln!("✓ Platform added, refreshing spreadsheet");
                            self.loaded = false; // Trigger reload
                            self.load_error = None;
                        }
                        Err(e) => {
                            self.load_error = Some(format!("Failed to save config: {e}"));
                        }
                    }
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {e}"));
                }
            }
        }
    }


    fn render_init_progress(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);
        ui.separator();
        ui.heading("Initialization Progress");

        if let Some(name) = &self.init_platform_name {
            ui.label(format!("Platform: {}", name));
        }

        ui.add_space(8.0);

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for log in &self.init_progress_log {
                    ui.label(log);
                }
            });
    }

    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn show_gcp_wizard(&mut self, platform_name: String) {
        let mut wizard = GcpWizard::new(platform_name);

        // Load OAuth from config if exists
        if let Ok((app_config, _)) = load_config() {
            wizard.load_oauth_from_config(&app_config);
        }

        wizard.show();
        self.gcp_wizard = Some(wizard);
    }

    fn show_delete_vm_confirmation(
        &mut self,
        platform_name: String,
        vm_name: String,
        zone: String,
    ) {
        self.delete_vm_platform = platform_name;
        self.delete_vm_list.clear();
        self.delete_vm_list
            .push((vm_name, zone, "".to_string()));
        self.delete_vm_selected = Some(0);
        self.delete_vm_confirming = true;
        self.show_delete_vm_dialog = true;
    }

    fn render_delete_vm_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.show_delete_vm_dialog;

        egui::Window::new("Delete VM")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                if self.delete_vm_confirming {
                    // Confirmation step
                    ui.heading("⚠ Confirm Deletion");
                    ui.add_space(8.0);

                    if let Some(idx) = self.delete_vm_selected {
                        if let Some((name, zone, _)) = self.delete_vm_list.get(idx).cloned() {
                            ui.label(format!("Are you sure you want to delete VM '{}'?", name));
                            ui.add_space(4.0);
                            ui.colored_label(
                                egui::Color32::from_rgb(245, 101, 101),
                                "This action cannot be undone!",
                            );
                            ui.add_space(4.0);
                            ui.label(format!("Zone: {}", zone));

                            ui.add_space(12.0);

                            let name_clone = name.clone();
                            let zone_clone = zone.clone();

                            ui.horizontal(|ui| {
                                if ui.button("No, Cancel").clicked() {
                                    self.delete_vm_confirming = false;
                                }

                                if ui
                                    .add(MaterialButton::filled("Yes, Delete"))
                                    .clicked()
                                {
                                    self.execute_delete_vm(name_clone, zone_clone);
                                    self.show_delete_vm_dialog = false;
                                }
                            });
                        }
                    }
                } else {
                    // VM selection step
                    ui.heading("Select VM to Delete");
                    ui.add_space(8.0);

                    if self.delete_vm_list.is_empty() {
                        ui.label("No VMs found in this project.");
                        ui.add_space(8.0);
                        ui.colored_label(
                            egui::Color32::GRAY,
                            "Note: Only VMs in common zones are shown.",
                        );
                    } else {
                        ui.label(format!("Found {} VM(s):", self.delete_vm_list.len()));
                        ui.add_space(8.0);

                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                for (idx, (name, zone, status)) in
                                    self.delete_vm_list.iter().enumerate()
                                {
                                    let is_selected = self.delete_vm_selected == Some(idx);
                                    if ui
                                        .selectable_label(
                                            is_selected,
                                            format!("{} ({}, {})", name, zone, status),
                                        )
                                        .clicked()
                                    {
                                        self.delete_vm_selected = Some(idx);
                                    }
                                }
                            });
                    }

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_delete_vm_dialog = false;
                        }

                        let can_delete = self.delete_vm_selected.is_some();
                        ui.add_enabled_ui(can_delete, |ui| {
                            if ui.add(MaterialButton::filled("Delete")).clicked() {
                                self.delete_vm_confirming = true;
                            }
                        });

                        if !can_delete {
                            ui.label("⚠ Select a VM");
                        }
                    });
                }
            });

        if !open {
            self.show_delete_vm_dialog = false;
            self.delete_vm_confirming = false;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn execute_delete_vm(&mut self, instance_name: String, zone: String) {
        if let Ok((mut app_config, config_path)) = load_config() {
            // Find platform and VM to get project_id
            let platform_idx = app_config
                .platforms
                .iter()
                .position(|p| p.name == self.delete_vm_platform);

            if let Some(idx) = platform_idx {
                let platform = &app_config.platforms[idx];

                // Find the VM to get its project_id
                let vm = platform.vms.iter().find(|vm| vm.name == instance_name);
                if vm.is_none() {
                    self.load_error = Some(format!("VM '{}' not found in config", instance_name));
                    return;
                }
                let project_id = vm.unwrap().gcp_project_id.clone();

                // Get valid access token (refresh if expired)
                let access_token = match self.get_valid_access_token(&mut app_config, idx, &config_path) {
                    Ok(token) => token,
                    Err(e) => {
                        self.load_error = Some(format!("Failed to get access token: {}", e));
                        return;
                    }
                };

                // Delete from GCP
                use crate::calc::gcp_rest::GcpRestClient;
                let client = GcpRestClient::new(access_token);

                match client.delete_instance(&project_id, &zone, &instance_name) {
                    Ok(_operation) => {
                        self.load_error = None;

                        // Record audit event
                        match audit::push_gui(
                            "system",
                            "desktop",
                            "vm delete",
                            &format!("{}:{}", project_id, instance_name),
                        ) {
                            Ok(audit_id) => {
                                eprintln!("✓ Audit record created: ID {}", audit_id);
                            }
                            Err(e) => {
                                eprintln!("⚠ Failed to record audit event: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        self.load_error = Some(format!("Failed to delete VM from GCP: {}", e));
                        return;
                    }
                }

                // Remove VM from config after successful deletion
                app_config.platforms[idx].vms.retain(|vm| vm.name != instance_name);

                // Save config
                if let Err(e) = app_config.save(&config_path) {
                    self.load_error = Some(format!("Failed to save config: {}", e));
                    return;
                }

                eprintln!("✓ VM deleted, refreshing spreadsheet");
                // Refresh the list
                self.loaded = false;
            } else {
                self.load_error = Some(format!("Platform '{}' not found", self.delete_vm_platform));
            }
        }
    }

    /// Get valid access token, refreshing if expired
    #[cfg(not(target_arch = "wasm32"))]
    fn get_valid_access_token(
        &self,
        app_config: &mut AppConfig,
        platform_idx: usize,
        config_path: &std::path::PathBuf,
    ) -> Result<String, String> {
        let platform = &app_config.platforms[platform_idx];

        // Check if token exists
        let access_token = platform.gcp_oauth_access_token.as_ref()
            .ok_or("No OAuth access token found")?;
        let refresh_token = platform.gcp_oauth_refresh_token.as_ref()
            .ok_or("No OAuth refresh token found")?;

        // Check if token is expired
        let now = chrono::Utc::now().timestamp();
        let is_expired = platform.gcp_oauth_token_expiry
            .map(|expiry| now >= expiry - 60) // Refresh 60 seconds before expiry
            .unwrap_or(true);

        if !is_expired {
            return Ok(access_token.clone());
        }

        // Token expired, refresh it
        eprintln!("Access token expired, refreshing...");

        use crate::api::gcp_oauth::{self, OAuthHandler};

        // Use embedded OAuth credentials
        let handler = OAuthHandler::default();
        let oauth_result = gcp_oauth::refresh_access_token(
            handler.client_id(),
            handler.client_secret(),
            refresh_token
        ).map_err(|e| format!("Failed to refresh token: {}", e))?;

        // Update config with new token
        let platform = &mut app_config.platforms[platform_idx];
        platform.gcp_oauth_access_token = Some(oauth_result.access_token.clone());
        platform.gcp_oauth_token_expiry = Some(oauth_result.expires_at as i64);

        // Save config
        app_config.save(config_path)
            .map_err(|e| format!("Failed to save refreshed token: {}", e))?;

        eprintln!("✓ Access token refreshed");
        Ok(oauth_result.access_token)
    }

    fn show_delete_platform_confirmation(&mut self, platform_name: String) {
        self.delete_platform_name = platform_name.clone();

        // Count VMs for this platform
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok((app_config, _)) = load_config() {
                if let Some(platform) = app_config.platforms.iter().find(|p| p.name == platform_name) {
                    self.delete_platform_vm_count = platform.vms.len();
                }
            }
        }

        self.show_delete_platform_dialog = true;
    }

    fn render_delete_platform_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.show_delete_platform_dialog;

        egui::Window::new("Delete Platform")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.heading("⚠ Confirm Platform Deletion");
                ui.add_space(8.0);

                ui.label(format!(
                    "Are you sure you want to delete platform '{}'?",
                    self.delete_platform_name
                ));
                ui.add_space(4.0);

                if self.delete_platform_vm_count > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(245, 101, 101),
                        format!(
                            "⚠ This will also remove {} VM(s) from config!",
                            self.delete_platform_vm_count
                        ),
                    );
                    ui.add_space(4.0);
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 152, 0),
                        "Note: VMs will be removed from config but NOT deleted from GCP.",
                    );
                } else {
                    ui.colored_label(
                        egui::Color32::GRAY,
                        "This platform has no VMs configured.",
                    );
                }

                ui.add_space(4.0);
                ui.colored_label(
                    egui::Color32::from_rgb(245, 101, 101),
                    "This action cannot be undone!",
                );

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui.button("No, Cancel").clicked() {
                        self.show_delete_platform_dialog = false;
                    }

                    if ui
                        .add(MaterialButton::filled("Yes, Delete Platform"))
                        .clicked()
                    {
                        self.execute_delete_platform();
                        self.show_delete_platform_dialog = false;
                    }
                });
            });

        if !open {
            self.show_delete_platform_dialog = false;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn execute_delete_platform(&mut self) {
        if let Ok((mut app_config, config_path)) = load_config() {
            // Find and remove platform
            if let Some(idx) = app_config
                .platforms
                .iter()
                .position(|p| p.name == self.delete_platform_name)
            {
                let platform = app_config.platforms.remove(idx);

                // Save config
                match app_config.save(&config_path) {
                    Ok(_) => {
                        self.load_error = None;

                        // Record audit event
                        match audit::push_gui(
                            "system",
                            "desktop",
                            "platform delete",
                            &format!("{} ({} VMs)", self.delete_platform_name, platform.vms.len()),
                        ) {
                            Ok(audit_id) => {
                                eprintln!("✓ Audit record created: ID {}", audit_id);
                            }
                            Err(e) => {
                                eprintln!("⚠ Failed to record audit event: {}", e);
                            }
                        }

                        eprintln!("✓ Platform deleted, refreshing spreadsheet");
                        // Refresh the list
                        self.loaded = false;
                    }
                    Err(e) => {
                        self.load_error = Some(format!("Failed to save config: {}", e));
                    }
                }
            } else {
                self.load_error = Some(format!("Platform '{}' not found", self.delete_platform_name));
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start_add_platform_oauth(&mut self) {
        use crate::api::gcp_oauth::OAuthHandler;
        use poll_promise::Promise;

        // Use embedded OAuth credentials (compiled into binary)
        let handler = OAuthHandler::default();

        self.add_platform_oauth_promise = Some(Promise::spawn_thread("gcp_oauth_add_platform", move || {
            handler.run_oauth_flow().map_err(|e| e.to_string())
        }));
    }


    #[cfg(not(target_arch = "wasm32"))]
    fn fetch_connected_email(&mut self) {
        if let Some(oauth) = &self.add_platform_oauth_result {
            use crate::calc::gcp_rest::GcpRestClient;

            let client = GcpRestClient::new(oauth.access_token.clone());

            // Get user info from OAuth2 userinfo endpoint
            match client.get_user_info() {
                Ok(user_info) => {
                    // Use email if available, fallback to name, or "Connected Account"
                    let display = if let Some(email) = user_info.email {
                        email
                    } else if let Some(name) = user_info.name {
                        name
                    } else {
                        "Connected Account".to_string()
                    };
                    self.add_platform_connected_email = Some(display);
                }
                Err(e) => {
                    eprintln!("Failed to fetch user info: {}", e);
                    self.add_platform_connected_email = Some("Connected Account".to_string());
                }
            }
        }
    }

    fn fetch_billing_data(&mut self) {
        self.billing_loading = true;
        self.billing_error = None;
        self.billing_data = None;

        // Load config to get GCP platform with OAuth
        let (app_config, _) = match load_config() {
            Ok(config) => config,
            Err(e) => {
                self.billing_error = Some(format!("Failed to load config: {}", e));
                self.billing_loading = false;
                return;
            }
        };

        // Find first GCP platform with OAuth token
        let platform = match app_config
            .platforms
            .iter()
            .find(|p| p.platform_type == "gcp" && p.gcp_oauth_access_token.is_some())
        {
            Some(p) => p,
            None => {
                self.billing_error = Some(
                    "No connected GCP platform found. Please add a GCP platform first.".to_string(),
                );
                self.billing_loading = false;
                return;
            }
        };

        // Get access token
        let access_token = match &platform.gcp_oauth_access_token {
            Some(token) => token.clone(),
            None => {
                self.billing_error = Some("No OAuth token found".to_string());
                self.billing_loading = false;
                return;
            }
        };

        // Get project ID from VMs
        let project_id = if !platform.vms.is_empty() {
            platform.vms[0].gcp_project_id.clone()
        } else {
            self.billing_error = Some(
                "No VMs found. Please create a VM to determine the project ID.".to_string(),
            );
            self.billing_loading = false;
            return;
        };

        // Create API client
        let client = crate::calc::gcp_rest::GcpRestClient::new(access_token);

        // Auto-discover billing table if not already configured
        if self.billing_dataset.is_empty() || self.billing_table.is_empty() {
            match client.discover_billing_table(&project_id) {
                Ok((dataset, table)) => {
                    self.billing_dataset = dataset;
                    self.billing_table = table;
                    self.billing_project_id = project_id.clone();
                }
                Err(e) => {
                    // Fall back to default names
                    self.billing_dataset = "billing_export".to_string();
                    self.billing_table = format!("gcp_billing_export_v1_{}", project_id.replace('-', "_"));
                    self.billing_project_id = project_id.clone();
                    self.billing_error = Some(format!(
                        "Auto-discovery failed: {}\n\nUsing default names. Please configure below if different.",
                        e
                    ));
                    self.billing_loading = false;
                    return;
                }
            }
        }

        // Fetch billing data
        match client.get_current_month_billing(&project_id, &self.billing_dataset, &self.billing_table) {
            Ok(records) => {
                self.billing_data = Some(records);
                self.billing_loading = false;
            }
            Err(e) => {
                self.billing_error = Some(format!(
                    "Failed to fetch billing data: {}\n\nCurrent settings:\n• Project: {}\n• Dataset: {}\n• Table: {}\n\nPlease verify these settings below.",
                    e, project_id, self.billing_dataset, self.billing_table
                ));
                self.billing_loading = false;
            }
        }
    }

    fn render_billing_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Estimated Billing")
            .collapsible(false)
            .resizable(true)
            .default_width(700.0)
            .show(ctx, |ui| {
                ui.heading("Current Month Billing Estimate");
                ui.add_space(8.0);

                // Configuration section
                ui.horizontal(|ui| {
                    ui.label("Project ID:");
                    ui.text_edit_singleline(&mut self.billing_project_id);
                });
                ui.horizontal(|ui| {
                    ui.label("Dataset:");
                    ui.text_edit_singleline(&mut self.billing_dataset);
                });
                ui.horizontal(|ui| {
                    ui.label("Table:");
                    ui.text_edit_singleline(&mut self.billing_table);
                });
                ui.add_space(4.0);
                ui.colored_label(
                    egui::Color32::GRAY,
                    "💡 Leave empty to auto-discover billing export table",
                );
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                if self.billing_loading {
                    ui.spinner();
                    ui.label("Loading billing data...");
                } else if let Some(error) = &self.billing_error {
                    ui.colored_label(egui::Color32::from_rgb(255, 82, 82), "Error:");
                    ui.add_space(4.0);
                    ui.label(error);
                    ui.add_space(16.0);

                    ui.label("To enable billing export:");
                    ui.label("1. Go to GCP Console → Billing → Billing export");
                    ui.label("2. Enable 'Standard usage cost' or 'Detailed usage cost'");
                    ui.label("3. Set project and dataset (e.g., 'billing_export')");
                    ui.label("4. Wait a few hours for data to appear");
                } else if let Some(records) = &self.billing_data {
                    if records.is_empty() {
                        ui.label("No billing data found for the current month.");
                        ui.add_space(8.0);
                        ui.label("This could mean:");
                        ui.label("• Billing export is not configured");
                        ui.label("• No costs have been incurred yet this month");
                        ui.label("• Data is still being processed (can take up to 5 days)");
                    } else {
                        // Calculate totals
                        let total_subtotal: f64 = records.iter().map(|r| r.subtotal).sum();
                        let total_list: f64 = records.iter().map(|r| r.list_cost).sum();
                        let total_savings: f64 = records.iter().map(|r| r.negotiated_savings).sum();
                        let total_discounts: f64 = records.iter().map(|r| r.discounts).sum();
                        let total_promotions: f64 = records.iter().map(|r| r.promotions).sum();

                        // Summary
                        ui.horizontal(|ui| {
                            ui.label("Total Cost:");
                            ui.label(format!("${:.2}", total_subtotal));
                        });
                        ui.horizontal(|ui| {
                            ui.label("List Cost:");
                            ui.label(format!("${:.2}", total_list));
                        });
                        if total_savings.abs() > 0.01 {
                            ui.horizontal(|ui| {
                                ui.label("Negotiated Savings:");
                                ui.colored_label(
                                    egui::Color32::from_rgb(72, 187, 120),
                                    format!("${:.2}", total_savings),
                                );
                            });
                        }
                        if total_discounts.abs() > 0.01 {
                            ui.horizontal(|ui| {
                                ui.label("Discounts:");
                                ui.colored_label(
                                    egui::Color32::from_rgb(72, 187, 120),
                                    format!("${:.2}", total_discounts),
                                );
                            });
                        }
                        if total_promotions.abs() > 0.01 {
                            ui.horizontal(|ui| {
                                ui.label("Promotions:");
                                ui.colored_label(
                                    egui::Color32::from_rgb(72, 187, 120),
                                    format!("${:.2}", total_promotions),
                                );
                            });
                        }

                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(4.0);

                        // Detailed breakdown
                        ui.label("Daily Breakdown:");
                        ui.add_space(4.0);

                        egui::ScrollArea::vertical()
                            .max_height(400.0)
                            .show(ui, |ui| {
                                // Group by day
                                let mut current_day = String::new();
                                for record in records {
                                    if record.day != current_day {
                                        if !current_day.is_empty() {
                                            ui.add_space(4.0);
                                        }
                                        current_day = record.day.clone();
                                        ui.label(format!("📅 {}", record.day));
                                        ui.separator();
                                    }

                                    ui.horizontal(|ui| {
                                        ui.label(format!("  {}", record.service));
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                ui.label(format!("${:.2}", record.subtotal));
                                                ui.label(" | ");
                                                if record.discounts.abs() > 0.01 || record.promotions.abs() > 0.01 {
                                                    ui.colored_label(
                                                        egui::Color32::from_rgb(72, 187, 120),
                                                        format!("💰${:.2}", record.discounts + record.promotions),
                                                    );
                                                    ui.label(" | ");
                                                }
                                                ui.label(format!("${:.2}", record.list_cost));
                                            },
                                        );
                                    });
                                }
                            });
                    }
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.add(MaterialButton::outlined("Refresh")).clicked() {
                        self.fetch_billing_data();
                    }

                    if ui.add(MaterialButton::outlined("Close")).clicked() {
                        self.show_billing_dialog = false;
                    }
                });
            });
    }
}
