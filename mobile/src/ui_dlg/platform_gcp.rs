//! GCP Platform Wizard Dialog
//!
//! Multi-step wizard for GCP Compute Engine setup:
//! 1. Connect Account (OAuth)
//! 2. Configure Project (enter project ID)
//! 3. Configure Server (region, machine type, etc.)
//! 4. Create Server (VM instance creation)
//! 5. Complete (show connection info)

use eframe::egui;
use egui_material3::MaterialButton;
use poll_promise::Promise;
use serde::{Deserialize, Serialize};

use crate::api::gcp::oauth::{OAuthHandler, OAuthResult};
use crate::calc::gcp::{Instance, MachineType, Region, get_common_machine_types};
use crate::api::gcp::GcpRestClient;
use crate::api::gcp::compute::{InstanceRequest, Metadata, MetadataItem};
use crate::calc::keyring;
use crate::config::{AppConfig, CloudPlatformConfig};

#[cfg(not(target_arch = "wasm32"))]
use base64::Engine;

/// GCP wizard state machine
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum WizardState {
    ConnectAccount,
    SelectProject,
    ConfigureServer,
    CreatingServer,
    Complete,
    Error(String),
}

/// GCP Platform Wizard
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GcpWizard {
    /// Current wizard state
    state: WizardState,

    /// Platform name (from parent)
    platform_name: String,

    /// OAuth result
    #[cfg_attr(feature = "serde", serde(skip))]
    oauth_result: Option<OAuthResult>,

    /// Selected project ID (user input, not loaded from API)
    selected_project_id: String,

    /// Selected region
    selected_region: String,

    /// Selected zone
    selected_zone: String,

    /// Selected machine type
    selected_machine_type: String,

    /// Instance name
    instance_name: String,

    /// Created instance
    #[cfg_attr(feature = "serde", serde(skip))]
    created_instance: Option<Instance>,

    /// Available regions (cached)
    #[cfg_attr(feature = "serde", serde(skip))]
    available_regions: Vec<Region>,

    /// Available machine types (cached)
    #[cfg_attr(feature = "serde", serde(skip))]
    available_machine_types: Vec<MachineType>,

    /// Available projects (loaded from GCP API)
    #[cfg_attr(feature = "serde", serde(skip))]
    available_projects: Vec<crate::api::gcp::resourcemanager::Project>,

    /// Project loading state
    #[cfg_attr(feature = "serde", serde(skip))]
    projects_loaded: bool,

    /// Project loading error
    #[cfg_attr(feature = "serde", serde(skip))]
    projects_load_error: Option<String>,

    /// New project name for creation (when no projects exist)
    new_project_name: String,

    /// Whether user chose "Create New Project" option in combo box
    create_new_project_selected: bool,

    /// OAuth promise (for async OAuth flow)
    #[cfg_attr(feature = "serde", serde(skip))]
    oauth_promise: Option<Promise<Result<OAuthResult, String>>>,

    /// Create promise (for async VM creation)
    #[cfg_attr(feature = "serde", serde(skip))]
    create_promise: Option<Promise<Result<Instance, String>>>,

    /// Progress messages
    #[cfg_attr(feature = "serde", serde(skip))]
    progress_log: Vec<String>,

    /// Show wizard dialog
    show: bool,

    /// Available platforms with GCP connection
    #[cfg_attr(feature = "serde", serde(skip))]
    available_platforms: Vec<CloudPlatformConfig>,

    /// Selected platform email for VM creation
    selected_platform_email: String,
}

impl Default for GcpWizard {
    fn default() -> Self {
        // Generate default project ID with timestamp
        let default_project_id = format!("dure-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));

        Self {
            state: WizardState::ConnectAccount,
            platform_name: String::new(),
            oauth_result: None,
            selected_project_id: default_project_id,
            selected_region: "us-central1".to_string(),
            selected_zone: "us-central1-a".to_string(),
            selected_machine_type: "e2-micro".to_string(),
            instance_name: "dure-server".to_string(),
            created_instance: None,
            available_regions: Vec::new(),
            available_machine_types: get_common_machine_types(),
            available_projects: Vec::new(),
            projects_loaded: false,
            projects_load_error: None,
            new_project_name: "Dure Server".to_string(),
            create_new_project_selected: false,
            oauth_promise: None,
            create_promise: None,
            progress_log: Vec::new(),
            show: false,
            available_platforms: Vec::new(),
            selected_platform_email: String::new(),
        }
    }
}

impl GcpWizard {
    /// Create new wizard for a platform
    pub fn new(platform_name: String) -> Self {
        Self {
            platform_name: platform_name.clone(),
            instance_name: format!("{}-server", platform_name.to_lowercase().replace(' ', "-")),
            ..Default::default()
        }
    }

    /// Load OAuth from config if exists (deprecated - now uses platform selection UI)
    pub fn load_oauth_from_config(&mut self, _config: &AppConfig) {
        // No-op: Platform selection is now handled in the ConnectAccount UI step
    }

    /// Save OAuth to config
    pub fn save_oauth_to_config(&self, config: &mut AppConfig) -> anyhow::Result<()> {
        if let Some(oauth) = &self.oauth_result {
            // Find or create GCP platform config
            let gcp_config = config
                .platforms
                .iter_mut()
                .find(|p| p.platform_type == "gcp")
                .cloned();

            let mut platform_config = gcp_config.unwrap_or_else(|| CloudPlatformConfig {
                name: "GCP".to_string(),
                platform_type: "gcp".to_string(),
                ..Default::default()
            });

            // Update OAuth fields
            platform_config.gcp_oauth_access_token = Some(oauth.access_token.clone());
            platform_config.gcp_oauth_refresh_token = Some(oauth.refresh_token.clone());
            platform_config.gcp_oauth_token_expiry = Some(oauth.expires_at as i64);

            // Update or add to platforms list
            if let Some(existing) = config
                .platforms
                .iter_mut()
                .find(|p| p.platform_type == "gcp")
            {
                *existing = platform_config;
            } else {
                config.platforms.push(platform_config);
            }
        }
        Ok(())
    }

    /// Clear OAuth from config
    pub fn clear_oauth_from_config(&self, config: &mut AppConfig) {
        if let Some(gcp_config) = config
            .platforms
            .iter_mut()
            .find(|p| p.platform_type == "gcp")
        {
            gcp_config.gcp_oauth_access_token = None;
            gcp_config.gcp_oauth_refresh_token = None;
            gcp_config.gcp_oauth_token_expiry = None;
        }
    }

    /// Show the wizard
    pub fn show(&mut self) {
        self.show = true;
        self.state = WizardState::ConnectAccount;
        self.progress_log.clear();
    }

    /// Hide the wizard
    pub fn hide(&mut self) {
        self.show = false;
    }

    /// Check if wizard is visible
    pub fn is_visible(&self) -> bool {
        self.show
    }

    /// Render the wizard UI
    pub fn ui(&mut self, ctx: &egui::Context) {
        if !self.show {
            return;
        }

        let mut open = true;

        egui::Window::new("GCP Server Setup")
            .open(&mut open)
            .resizable(true)
            .default_width(600.0)
            .default_height(500.0)
            .collapsible(false)
            .show(ctx, |ui| {
                // Progress indicator
                self.render_progress_indicator(ui);

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);

                // Render current step
                match self.state.clone() {
                    WizardState::ConnectAccount => self.render_connect_account(ui),
                    WizardState::SelectProject => self.render_select_project(ui),
                    WizardState::ConfigureServer => self.render_configure_server(ui),
                    WizardState::CreatingServer => self.render_creating_server(ui),
                    WizardState::Complete => self.render_complete(ui),
                    WizardState::Error(err) => self.render_error(ui, &err),
                }
            });

        if !open {
            self.hide();
        }
    }

    fn render_progress_indicator(&self, ui: &mut egui::Ui) {
        let steps = [
            ("Connect", WizardState::ConnectAccount),
            ("Project", WizardState::SelectProject),
            ("Configure", WizardState::ConfigureServer),
            ("Create", WizardState::CreatingServer),
            ("Complete", WizardState::Complete),
        ];

        ui.horizontal(|ui| {
            for (i, (label, step_state)) in steps.iter().enumerate() {
                if i > 0 {
                    ui.label("→");
                }

                let is_current =
                    std::mem::discriminant(&self.state) == std::mem::discriminant(step_state);
                let is_past = self.is_past_step(step_state);

                let color = if is_current {
                    egui::Color32::from_rgb(103, 126, 234) // Primary color
                } else if is_past {
                    egui::Color32::from_rgb(72, 187, 120) // Green
                } else {
                    egui::Color32::GRAY
                };

                ui.colored_label(
                    color,
                    if is_current {
                        format!("● {}", label)
                    } else {
                        label.to_string()
                    },
                );
            }
        });
    }

    fn is_past_step(&self, step: &WizardState) -> bool {
        use WizardState::*;
        match (&self.state, step) {
            (SelectProject, ConnectAccount) => true,
            (ConfigureServer, ConnectAccount | SelectProject) => true,
            (CreatingServer, ConnectAccount | SelectProject | ConfigureServer) => true,
            (Complete, ConnectAccount | SelectProject | ConfigureServer | CreatingServer) => true,
            _ => false,
        }
    }

    fn render_connect_account(&mut self, ui: &mut egui::Ui) {
        ui.heading("Select Google Cloud Account");
        ui.add_space(8.0);

        // Load available platforms on first render
        if self.available_platforms.is_empty() {
            self.load_available_platforms();
        }

        // Check if any platforms are available
        if self.available_platforms.is_empty() {
            ui.colored_label(
                egui::Color32::from_rgb(255, 152, 0),
                "⚠ No connected Google Cloud platforms found",
            );
            ui.add_space(8.0);

            ui.label("You need to add a GCP platform first before creating VMs.");
            ui.add_space(4.0);
            ui.label("Steps:");
            ui.label("  1. Go to the Platform tab");
            ui.label("  2. Click 'Add Platform'");
            ui.label("  3. Select 'GCP' and connect your Google account");
            ui.label("  4. Return here to create a VM");

            ui.add_space(16.0);

            if ui.button("Cancel").clicked() {
                self.hide();
            }

            return;
        }

        ui.label("Select which Google Cloud account to use for this VM:");
        ui.add_space(8.0);

        // Platform selection combobox
        egui::ComboBox::from_label("GCP Account")
            .selected_text(&self.selected_platform_email)
            .show_ui(ui, |ui| {
                for platform in &self.available_platforms {
                    if let Some(email) = &platform.gcp_connected_email {
                        let selected = self.selected_platform_email == *email;
                        if ui.selectable_label(selected, email).clicked() {
                            self.selected_platform_email = email.clone();
                        }
                    }
                }
            });

        ui.add_space(16.0);

        // Next button
        ui.horizontal(|ui| {
            if ui.add(MaterialButton::filled("Next →")).clicked() {
                // Load OAuth from selected platform and refresh if expired
                if let Some(platform) = self
                    .available_platforms
                    .iter()
                    .find(|p| p.gcp_connected_email.as_ref() == Some(&self.selected_platform_email))
                {
                    if let (Some(access_token), Some(refresh_token)) = (
                        &platform.gcp_oauth_access_token,
                        &platform.gcp_oauth_refresh_token,
                    ) {
                        let expires_at = platform
                            .gcp_oauth_token_expiry
                            .map(|exp| exp as u64)
                            .unwrap_or(chrono::Utc::now().timestamp() as u64 + 3600);

                        // Check if token is expired (with 60 second buffer)
                        let now = chrono::Utc::now().timestamp() as u64;
                        let is_expired = now >= expires_at.saturating_sub(60);

                        if is_expired {
                            // Refresh token
                            log::info!("OAuth token expired, refreshing...");
                            match self.refresh_token_sync(refresh_token) {
                                Ok(new_oauth) => {
                                    self.oauth_result = Some(new_oauth);
                                    self.state = WizardState::SelectProject;
                                }
                                Err(e) => {
                                    log::error!("Failed to refresh token: {}", e);
                                    // Show error but allow retry
                                }
                            }
                        } else {
                            // Token still valid
                            self.oauth_result = Some(OAuthResult {
                                access_token: access_token.clone(),
                                refresh_token: refresh_token.clone(),
                                expires_at,
                            });
                            self.state = WizardState::SelectProject;
                        }
                    }
                }
            }

            ui.add_space(8.0);

            if ui.button("Cancel").clicked() {
                self.hide();
            }
        });
    }

    fn render_select_project(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configure Project");
        ui.add_space(8.0);

        // Load projects on first render
        if !self.projects_loaded {
            self.load_projects();
        }

        // Show error if project loading failed
        if let Some(error) = &self.projects_load_error {
            // Check if it's the API not enabled error
            if error.contains("Cloud Resource Manager API") && error.contains("not been used") {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 152, 0),
                    "⚠ Cloud Resource Manager API Not Enabled",
                );
                ui.add_space(8.0);

                ui.label("The Cloud Resource Manager API needs to be enabled to list projects.");
                ui.add_space(4.0);

                // Extract project number from error
                let project_num = error
                    .split("project ")
                    .nth(1)
                    .and_then(|s| s.split_whitespace().next())
                    .unwrap_or("YOUR_PROJECT");

                let enable_url = format!(
                    "https://console.developers.google.com/apis/api/cloudresourcemanager.googleapis.com/overview?project={}",
                    project_num
                );

                ui.hyperlink_to("→ Click here to enable the API", &enable_url);
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.button("🔄 Retry Loading Projects").clicked() {
                        self.projects_loaded = false;
                        self.projects_load_error = None;
                    }

                    ui.colored_label(egui::Color32::GRAY, "After enabling, click Retry");
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);

                ui.label("Or proceed without loading (enter project ID manually):");
                ui.add_space(8.0);
            } else {
                ui.colored_label(
                    egui::Color32::from_rgb(245, 101, 101),
                    format!("⚠ {}", error),
                );
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.button("🔄 Retry").clicked() {
                        self.projects_loaded = false;
                        self.projects_load_error = None;
                    }
                });
                ui.add_space(8.0);
                ui.label("Or proceed by entering project ID manually:");
                ui.add_space(8.0);
            }
        }

        // Only show active projects
        let active_projects: Vec<_> = self
            .available_projects
            .iter()
            .filter(|p| p.is_active())
            .collect();

        // Show simple info about active projects found
        if self.projects_loaded && !active_projects.is_empty() {
            ui.colored_label(
                egui::Color32::GRAY,
                format!(
                    "ℹ Found {} active project{}",
                    active_projects.len(),
                    if active_projects.len() == 1 { "" } else { "s" }
                ),
            );
            ui.add_space(4.0);
        } else if !self.projects_loaded
            && self.oauth_result.is_some()
            && self.projects_load_error.is_none()
        {
            ui.colored_label(egui::Color32::GRAY, "⏳ Loading projects...");
            ui.add_space(4.0);
        }

        // Always show combo box with existing projects + "Create New Project" option
        ui.label("Select a project:");
        ui.add_space(8.0);

        // Build combo box options
        const CREATE_NEW_ID: &str = "__CREATE_NEW_PROJECT__";

        // Determine what to show in combo box
        let combo_display = if self.create_new_project_selected {
            "➕ Create New Project".to_string()
        } else if !self.selected_project_id.is_empty() && self.selected_project_id != CREATE_NEW_ID
        {
            // Find matching project for display
            active_projects
                .iter()
                .find(|p| p.project_id == self.selected_project_id)
                .map(|p| {
                    let display = p.display_name();
                    if display != p.project_id {
                        format!("{} ({})", display, p.project_id)
                    } else {
                        p.project_id.clone()
                    }
                })
                .unwrap_or_else(|| self.selected_project_id.clone())
        } else if !active_projects.is_empty() {
            // Default to first project
            let first = active_projects[0];
            self.selected_project_id = first.project_id.clone();
            self.create_new_project_selected = false;
            let display = first.display_name();
            if display != first.project_id {
                format!("{} ({})", display, first.project_id)
            } else {
                first.project_id.clone()
            }
        } else {
            // No active projects - default to create new
            self.create_new_project_selected = true;
            "➕ Create New Project".to_string()
        };

        ui.horizontal(|ui| {
            ui.label("Project:");

            let combo_width = if !active_projects.is_empty() {
                400.0
            } else {
                300.0
            };

            egui::ComboBox::from_id_source("project_selector")
                .selected_text(&combo_display)
                .width(combo_width)
                .show_ui(ui, |ui| {
                    // Show active projects only
                    for project in &active_projects {
                        let display = project.display_name();
                        let label = if display != project.project_id {
                            format!("{} ({})", display, project.project_id)
                        } else {
                            project.project_id.clone()
                        };

                        if ui
                            .selectable_label(
                                self.selected_project_id == project.project_id
                                    && !self.create_new_project_selected,
                                label,
                            )
                            .clicked()
                        {
                            self.selected_project_id = project.project_id.clone();
                            self.create_new_project_selected = false;
                        }
                    }

                    // Add separator if there are existing projects
                    if !active_projects.is_empty() {
                        ui.separator();
                    }

                    // Always show "Create New Project" option
                    if ui
                        .selectable_label(self.create_new_project_selected, "➕ Create New Project")
                        .clicked()
                    {
                        self.create_new_project_selected = true;
                        // Generate new project ID with timestamp
                        self.selected_project_id =
                            format!("dure-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S"));
                        self.new_project_name = "Dure Server".to_string();
                    }
                });
        });

        ui.add_space(12.0);

        // Compute validation before closures to avoid borrow issues
        let project_id_for_validation = self.selected_project_id.clone();
        let create_new_selected = self.create_new_project_selected;

        // Show input fields when "Create New Project" is selected
        if create_new_selected {
            let is_valid = self.validate_project_id(&project_id_for_validation);
            let is_empty = project_id_for_validation.is_empty();

            ui.group(|ui| {
                ui.set_width(ui.available_width());
                ui.vertical(|ui| {
                    ui.label("New Project Details:");
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("Project ID:");
                        ui.text_edit_singleline(&mut self.selected_project_id);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Display Name:");
                        ui.text_edit_singleline(&mut self.new_project_name);
                    });

                    ui.add_space(4.0);
                    ui.colored_label(
                        egui::Color32::GRAY,
                        "💡 Project ID: 6-30 characters, lowercase letters, digits, hyphens",
                    );

                    // Show validation error
                    if !is_valid && !is_empty {
                        ui.colored_label(
                            egui::Color32::from_rgb(245, 101, 101),
                            "⚠ Invalid project ID format",
                        );
                    }
                });
            });
        }

        ui.add_space(16.0);

        // Determine state before entering closure to avoid borrow issues
        let can_proceed = !self.selected_project_id.is_empty()
            && (self.create_new_project_selected || !active_projects.is_empty());
        let has_load_error = self.projects_load_error.is_some();
        let is_new_project = self.create_new_project_selected;

        ui.horizontal(|ui| {
            if ui.button("← Back").clicked() {
                self.state = WizardState::ConnectAccount;
            }

            ui.add_enabled_ui(can_proceed, |ui| {
                if ui.add(MaterialButton::filled("Next →")).clicked() {
                    // If we couldn't load projects due to API error, try to proceed anyway
                    // The project might exist even if we couldn't list it
                    if has_load_error && !is_new_project {
                        // Don't try to create - just proceed and let region loading handle it
                        self.load_regions();
                        self.state = WizardState::ConfigureServer;
                    } else if is_new_project && can_proceed {
                        // It's a new project - create it first
                        self.create_project_and_proceed();
                    } else {
                        // Existing project - just proceed
                        self.load_regions();
                        self.state = WizardState::ConfigureServer;
                    }
                }
            });

            if !can_proceed {
                ui.label("⚠ Select or create a project");
            } else if is_new_project {
                ui.colored_label(
                    egui::Color32::from_rgb(100, 181, 246),
                    "ℹ Will create new project",
                );
            }

            if ui.button("Cancel").clicked() {
                self.hide();
            }
        });
    }

    fn render_configure_server(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configure Server");
        ui.add_space(8.0);

        // Instance name with validation
        ui.horizontal(|ui| {
            ui.label("Instance Name:");
            ui.text_edit_singleline(&mut self.instance_name);
        });

        // Show validation hint
        if !self.instance_name.is_empty() {
            let is_valid = self.validate_instance_name(&self.instance_name);
            if is_valid {
                ui.colored_label(egui::Color32::from_rgb(72, 187, 120), "✓ Valid name");
            } else {
                ui.colored_label(
                    egui::Color32::from_rgb(245, 101, 101),
                    "⚠ Name must start with letter, contain only lowercase letters, numbers, hyphens"
                );
            }
        }

        ui.add_space(8.0);

        // Region selection
        ui.horizontal(|ui| {
            ui.label("Region:");
            // Find current region to show friendly name
            let selected_display = self
                .available_regions
                .iter()
                .find(|r| r.name == self.selected_region)
                .map(|r| format!("{} ({})", r.location, r.name))
                .unwrap_or_else(|| self.selected_region.clone());

            egui::ComboBox::from_id_salt("region_combo")
                .selected_text(&selected_display)
                .show_ui(ui, |ui| {
                    for region in &self.available_regions {
                        ui.selectable_value(
                            &mut self.selected_region,
                            region.name.clone(),
                            format!("{} ({})", region.location, region.name),
                        );
                    }
                });
        });

        // Zone selection (based on selected region)
        if let Some(region) = self
            .available_regions
            .iter()
            .find(|r| r.name == self.selected_region)
        {
            ui.horizontal(|ui| {
                ui.label("Zone:");
                egui::ComboBox::from_id_salt("zone_combo")
                    .selected_text(&self.selected_zone)
                    .show_ui(ui, |ui| {
                        for zone in &region.zones {
                            ui.selectable_value(&mut self.selected_zone, zone.clone(), zone);
                        }
                    });
            });
        }

        ui.add_space(8.0);

        // Machine type selection
        ui.horizontal(|ui| {
            ui.label("Machine Type:");
            egui::ComboBox::from_id_salt("machine_type_combo")
                .selected_text(&self.selected_machine_type)
                .show_ui(ui, |ui| {
                    for machine_type in &self.available_machine_types {
                        ui.selectable_value(
                            &mut self.selected_machine_type,
                            machine_type.name.clone(),
                            format!("{} - {}", machine_type.name, machine_type.description),
                        );
                    }
                });
        });

        ui.add_space(16.0);

        ui.horizontal(|ui| {
            if ui.button("← Back").clicked() {
                self.state = WizardState::SelectProject;
            }

            let can_create = !self.instance_name.is_empty()
                && self.validate_instance_name(&self.instance_name)
                && !self.selected_region.is_empty()
                && !self.selected_zone.is_empty()
                && !self.selected_machine_type.is_empty();

            let create_button = MaterialButton::filled("Create Server");
            ui.add_enabled_ui(can_create, |ui| {
                if ui.add(create_button).clicked() {
                    self.start_server_creation();
                }
            });

            if !can_create {
                ui.label("⚠ Complete all fields");
            }

            if ui.button("Cancel").clicked() {
                self.hide();
            }
        });
    }

    fn render_creating_server(&mut self, ui: &mut egui::Ui) {
        ui.heading("Creating Server");
        ui.add_space(8.0);

        ui.spinner();
        ui.label("Please wait while we create your GCP Compute Engine instance...");

        ui.add_space(8.0);
        ui.colored_label(egui::Color32::GRAY, "This usually takes 1-2 minutes...");

        ui.add_space(16.0);

        // Show progress log
        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for log in &self.progress_log {
                    ui.label(log);
                }
            });

        // Check for creation promise result
        if let Some(promise) = &self.create_promise {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(instance) => {
                        self.created_instance = Some(instance.clone());
                        self.progress_log
                            .push("✓ Server created successfully!".to_string());
                        self.state = WizardState::Complete;
                        self.create_promise = None;
                    }
                    Err(e) => {
                        self.state = WizardState::Error(e.clone());
                        self.create_promise = None;
                    }
                }
            }
        }
    }

    fn render_complete(&mut self, ui: &mut egui::Ui) {
        ui.heading("✓ Setup Complete!");
        ui.add_space(8.0);

        if let Some(instance) = &self.created_instance {
            // Check if instance is actually running
            let is_running = instance.status.to_uppercase() == "RUNNING";

            if is_running {
                ui.colored_label(
                    egui::Color32::from_rgb(72, 187, 120),
                    "✓ Instance is RUNNING and ready to use!",
                );
            } else {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 193, 7),
                    format!(
                        "⏳ Instance status: {} (waiting for RUNNING)",
                        instance.status
                    ),
                );
            }

            ui.add_space(12.0);

            ui.group(|ui| {
                ui.set_width(ui.available_width());

                ui.label(format!("📦 Instance Name: {}", instance.name));
                ui.label(format!("🆔 Instance ID: {}", instance.id));
                ui.label(format!("⚙️  Machine Type: {}", instance.machine_type));
                ui.label(format!("📍 Zone: {}", instance.zone));

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                let status_color = if is_running {
                    egui::Color32::from_rgb(72, 187, 120)
                } else {
                    egui::Color32::from_rgb(255, 193, 7)
                };
                ui.colored_label(status_color, format!("📊 Status: {}", instance.status));

                if let Some(ip) = &instance.external_ip {
                    ui.colored_label(
                        egui::Color32::from_rgb(103, 126, 234),
                        format!("🌐 External IP: {}", ip),
                    );

                    // SSH command hint
                    ui.add_space(4.0);
                    ui.label(format!("💻 SSH: ssh root@{}", ip));
                } else {
                    ui.label("🌐 External IP: (assigning...)");
                }

                if let Some(internal_ip) = &instance.internal_ip {
                    ui.label(format!("🔒 Internal IP: {}", internal_ip));
                }
            });

            // Save VM to config (do this once when state transitions to Complete)
            self.save_vm_to_config();

            ui.add_space(12.0);

            ui.label("📋 Next steps:");
            if !is_running {
                ui.label("  1. ⏳ Wait for instance to reach RUNNING status");
            }
            ui.label("  2. 🔑 SSH key already configured for root access");
            ui.label("  3. 🔐 Set up firewall rules if needed");
            ui.label("  4. 📦 Install your application");

            ui.add_space(8.0);
            ui.label("💡 View in GCP Console:");
            ui.hyperlink_to(
                format!(
                    "https://console.cloud.google.com/compute/instances?project={}",
                    self.selected_project_id
                ),
                format!(
                    "https://console.cloud.google.com/compute/instances?project={}",
                    self.selected_project_id
                ),
            );
        } else {
            ui.colored_label(egui::Color32::RED, "⚠ Instance information not available");
            ui.label("The creation may have failed. Check the GCP Console.");
        }

        ui.add_space(16.0);

        if ui.add(MaterialButton::filled("Close")).clicked() {
            self.hide();
        }
    }

    /// Save VM instance to config
    fn save_vm_to_config(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(instance) = &self.created_instance {
                if let Ok((mut app_config, config_path)) = self.load_config_file() {
                    // Find the platform
                    if let Some(platform) = app_config
                        .platforms
                        .iter_mut()
                        .find(|p| p.name == self.platform_name)
                    {
                        // Check if VM already exists
                        if platform.vms.iter().any(|vm| vm.instance_id == instance.id) {
                            return; // Already saved
                        }

                        // Extract region from zone (e.g., "us-central1-a" -> "us-central1")
                        let region = self
                            .selected_zone
                            .rsplitn(2, '-')
                            .nth(1)
                            .unwrap_or(&self.selected_zone)
                            .to_string();

                        // Fetch billing account name
                        let billing_account_name = if let Some(oauth) = &self.oauth_result {
                            use crate::api::gcp::GcpRestClient;
                            let client = GcpRestClient::new(oauth.access_token.clone());

                            match client.list_billing_accounts() {
                                Ok(list) => {
                                    if let Some(ba) = list.billing_accounts.first() {
                                        // Extract billing account ID from name (e.g., "billingAccounts/012345-ABCDEF-678901" -> "012345-ABCDEF-678901")
                                        let account_id = ba
                                            .name
                                            .strip_prefix("billingAccounts/")
                                            .unwrap_or(&ba.name)
                                            .to_string();

                                        self.progress_log.push(format!(
                                            "✓ Found billing account: {} ({})",
                                            ba.display_name, account_id
                                        ));
                                        Some(account_id)
                                    } else {
                                        self.progress_log
                                            .push("⚠ No billing accounts found".to_string());
                                        None
                                    }
                                }
                                Err(e) => {
                                    self.progress_log
                                        .push(format!("⚠ Failed to fetch billing account: {}", e));
                                    None
                                }
                            }
                        } else {
                            self.progress_log.push(
                                "⚠ No OAuth token available for billing account fetch".to_string(),
                            );
                            None
                        };

                        // Create VM instance entry
                        let ssh_key_name = format!("gcp.{}.{}", self.platform_name, instance.name);
                        let vm = crate::config::VmInstance {
                            name: instance.name.clone(),
                            instance_id: instance.id.clone(),
                            zone: self.selected_zone.clone(),
                            gcp_region: region,
                            machine_type: self.selected_machine_type.clone(),
                            status: instance.status.clone(),
                            external_ip: instance.external_ip.clone(),
                            internal_ip: instance.internal_ip.clone(),
                            gcp_project_id: self.selected_project_id.clone(),
                            gcp_billing_account: billing_account_name,
                            created_at: chrono::Utc::now().timestamp(),
                            ssh_key_name: Some(ssh_key_name.clone()),
                        };

                        // Add VM to platform
                        platform.vms.push(vm);

                        // Add SSH host entry if external IP is available
                        if let Some(external_ip) = &instance.external_ip {
                            let ssh_host = crate::config::SshHostConfig {
                                host: format!("root@{}", external_ip),
                                password: None,
                                private_key_path: None,
                                keyring_domain: Some(ssh_key_name.clone()),
                                port: 22,
                                initialized: true,
                                last_status: None,
                                platform_name: Some(self.platform_name.clone()),
                                docker_containers: Vec::new(),
                                ansible_roles: Vec::new(),
                                dure_wss_config: None,
                            };

                            // Check if SSH host already exists
                            if !app_config.ssh_hosts.iter().any(|h| h.host == ssh_host.host) {
                                app_config.ssh_hosts.push(ssh_host);
                                self.progress_log
                                    .push(format!("✓ SSH host added: root@{}", external_ip));
                            }
                        } else {
                            self.progress_log
                                .push("⚠ No external IP, SSH host not added".to_string());
                        }

                        // Save config
                        if let Err(e) = app_config.save(&config_path) {
                            self.progress_log
                                .push(format!("⚠ Failed to save VM to config: {}", e));
                        } else {
                            self.progress_log.push("✓ VM saved to config".to_string());
                        }
                    }
                }
            }
        }
    }

    fn render_error(&mut self, ui: &mut egui::Ui, error: &str) {
        ui.heading("❌ Error");
        ui.add_space(8.0);

        // Check for specific API not enabled errors
        let is_compute_api_error =
            error.contains("Compute Engine API") && error.contains("is not enabled");

        if is_compute_api_error {
            // Extract project ID from error message
            let project_id = if let Some(pos) = error.find("project '") {
                let start = pos + 9; // Length of "project '"
                error[start..]
                    .find('\'')
                    .map(|end_pos| &error[start..start + end_pos])
            } else {
                None
            };

            ui.colored_label(
                egui::Color32::from_rgb(255, 152, 0), // Orange
                error.split('\n').next().unwrap_or(error),
            );

            ui.add_space(12.0);

            ui.label("To fix this (one-time setup):");
            ui.add_space(4.0);

            if let Some(proj_id) = project_id {
                let enable_url = format!(
                    "https://console.developers.google.com/apis/api/compute.googleapis.com/overview?project={}",
                    proj_id
                );

                ui.label("1. Open: ");
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    if ui.link(&enable_url).clicked() {
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let _ = webbrowser::open(&enable_url);
                        }
                    }
                });
            } else {
                ui.label("1. Open the GCP Console for your project");
            }

            ui.add_space(4.0);
            ui.label("2. Click 'Enable API'");
            ui.add_space(4.0);
            ui.label("3. Wait a few minutes for changes to propagate");
            ui.add_space(4.0);
            ui.label("4. Return here and click 'Create Server' again");

            ui.add_space(12.0);
            ui.colored_label(
                egui::Color32::GRAY,
                "Note: This needs to be done once per GCP project.",
            );

            ui.add_space(16.0);

            ui.horizontal(|ui| {
                if ui
                    .add(MaterialButton::filled("← Back to Configure"))
                    .clicked()
                {
                    self.state = WizardState::ConfigureServer;
                }

                if ui.button("Close").clicked() {
                    self.hide();
                }
            });
        } else {
            // Generic error display
            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    ui.colored_label(egui::Color32::RED, error);
                });

            ui.add_space(16.0);

            ui.horizontal(|ui| {
                if ui.button("← Start Over").clicked() {
                    self.state = WizardState::ConnectAccount;
                    self.progress_log.clear();
                }

                if ui.button("Close").clicked() {
                    self.hide();
                }
            });
        }
    }

    fn start_oauth(&mut self) {
        self.progress_log.push("Starting OAuth flow...".to_string());

        // Use embedded OAuth credentials (compiled into binary)
        let handler = OAuthHandler::default();

        self.oauth_promise = Some(Promise::spawn_thread("gcp_oauth", move || {
            handler.run_oauth_flow().map_err(|e| e.to_string())
        }));
    }

    /// Get config file path
    #[cfg(not(target_arch = "wasm32"))]
    fn get_config_path(&self) -> Result<std::path::PathBuf, String> {
        directories::ProjectDirs::from("pe", "nikescar", "dure")
            .map(|proj_dirs| proj_dirs.config_dir().join("config.yml"))
            .ok_or_else(|| "Failed to get config directory".to_string())
    }

    /// Load config file
    #[cfg(not(target_arch = "wasm32"))]
    fn load_config_file(&self) -> Result<(AppConfig, std::path::PathBuf), String> {
        let config_path = self.get_config_path()?;
        let app_config = AppConfig::load_or_default(&config_path);
        Ok((app_config, config_path))
    }

    fn load_available_platforms(&mut self) {
        if let Ok((app_config, _)) = self.load_config_file() {
            // Filter platforms with GCP connection (has gcp_connected_email)
            self.available_platforms = app_config
                .platforms
                .into_iter()
                .filter(|p| {
                    p.platform_type == "gcp"
                        && p.gcp_connected_email.is_some()
                        && p.gcp_oauth_access_token.is_some()
                })
                .collect();

            // Auto-select first platform if available
            if !self.available_platforms.is_empty() && self.selected_platform_email.is_empty() {
                if let Some(email) = &self.available_platforms[0].gcp_connected_email {
                    self.selected_platform_email = email.clone();
                }
            }
        }
    }

    fn refresh_token_sync(&self, refresh_token: &str) -> Result<OAuthResult, String> {
        use crate::api::gcp::oauth::{self, OAuthHandler};

        // Use embedded OAuth credentials
        let handler = OAuthHandler::default();
        let oauth_result = oauth::refresh_access_token(
            handler.client_id(),
            handler.client_secret(),
            refresh_token,
        )
        .map_err(|e| format!("Failed to refresh token: {}", e))?;

        // Update config with new token
        if let Ok((mut app_config, config_path)) = self.load_config_file() {
            if let Some(platform) = app_config
                .platforms
                .iter_mut()
                .find(|p| p.gcp_connected_email.as_ref() == Some(&self.selected_platform_email))
            {
                platform.gcp_oauth_access_token = Some(oauth_result.access_token.clone());
                platform.gcp_oauth_token_expiry = Some(oauth_result.expires_at as i64);

                // Save config
                app_config
                    .save(&config_path)
                    .map_err(|e| format!("Failed to save refreshed token: {}", e))?;

                log::info!("✓ OAuth token refreshed and saved");
            }
        }

        Ok(oauth_result)
    }

    fn store_oauth_token(&self, oauth_result: &OAuthResult) -> Result<(), String> {
        // Store refresh token in keyring (overwrites existing if present)
        let domain = format!("gcp.{}", self.platform_name);
        let username = "oauth";
        let password = &oauth_result.refresh_token;

        // Get keyring paths
        let kdbx_path = keyring::get_default_kdbx_path()
            .map_err(|e| format!("Failed to get kdbx path: {}", e))?;
        let kpkey_path = keyring::get_default_kpkey_path()
            .map_err(|e| format!("Failed to get KPKey path: {}", e))?;

        keyring::update_key(&kdbx_path, Some(&kpkey_path), &domain, username, password)
            .map_err(|e| format!("Failed to store OAuth token: {}", e))?;

        Ok(())
    }

    fn load_regions(&mut self) {
        if let Some(oauth) = &self.oauth_result {
            let client = GcpRestClient::new(oauth.access_token.clone());

            match client.list_regions(&self.selected_project_id) {
                Ok(region_list) => {
                    self.available_regions = region_list
                        .items
                        .into_iter()
                        .map(|r| Region {
                            name: r.name.clone(),
                            location: r.description,
                            zones: r
                                .zones
                                .iter()
                                .filter_map(|z| z.split('/').next_back().map(String::from))
                                .collect(),
                        })
                        .collect();

                    self.progress_log
                        .push(format!("✓ Loaded {} regions", self.available_regions.len()));
                }
                Err(e) => {
                    // Check if it's a "not found" error for a newly created project
                    let error_msg = e.to_string();
                    let is_new_project_not_ready = error_msg.contains("404")
                        || error_msg.contains("NOT_FOUND")
                        || error_msg.contains("was not found");

                    if is_new_project_not_ready {
                        self.progress_log.push(
                            "ℹ Project not fully ready yet - using default regions".to_string(),
                        );
                    } else {
                        self.progress_log
                            .push(format!("⚠ Failed to load regions: {}", e));
                    }

                    // Fallback to static list
                    self.available_regions = vec![
                        Region {
                            name: "us-central1".to_string(),
                            location: "Iowa, USA".to_string(),
                            zones: vec!["us-central1-a".to_string(), "us-central1-b".to_string()],
                        },
                        Region {
                            name: "us-east1".to_string(),
                            location: "South Carolina, USA".to_string(),
                            zones: vec!["us-east1-b".to_string(), "us-east1-c".to_string()],
                        },
                        Region {
                            name: "asia-northeast3".to_string(),
                            location: "Seoul, South Korea".to_string(),
                            zones: vec!["asia-northeast3-a".to_string()],
                        },
                    ];

                    self.progress_log.push(format!(
                        "✓ Using {} default regions",
                        self.available_regions.len()
                    ));
                }
            }
        }
    }

    fn load_projects(&mut self) {
        if self.projects_loaded {
            return;
        }

        if let Some(oauth) = &self.oauth_result {
            let client = GcpRestClient::new(oauth.access_token.clone());

            match client.list_projects(None) {
                Ok(project_list) => {
                    self.available_projects = project_list.projects;
                    self.projects_loaded = true;
                    self.projects_load_error = None;

                    // Log project details for debugging
                    log::info!(
                        "Loaded {} projects from GCP API",
                        self.available_projects.len()
                    );
                    for proj in &self.available_projects {
                        let state_str = proj.state.as_deref().unwrap_or("<no state field>");
                        log::debug!(
                            "  Project: {} ({}), state: {:?}",
                            proj.display_name(),
                            proj.project_id,
                            state_str
                        );
                    }

                    let active_count = self
                        .available_projects
                        .iter()
                        .filter(|p| p.is_active())
                        .count();
                    log::info!(
                        "  {} active/usable projects, {} total",
                        active_count,
                        self.available_projects.len()
                    );

                    // Auto-select first active project if available
                    if let Some(project) = self.available_projects.iter().find(|p| p.is_active()) {
                        self.selected_project_id = project.project_id.clone();
                        log::info!("Auto-selected first active project: {}", project.project_id);
                    } else {
                        log::warn!(
                            "No active projects found among {} total projects",
                            self.available_projects.len()
                        );
                    }
                }
                Err(e) => {
                    self.projects_load_error = Some(format!("Failed to load projects: {}", e));
                    self.projects_loaded = true;
                    log::error!("Failed to load projects: {}", e);
                }
            }
        }
    }

    fn create_project_and_proceed(&mut self) {
        if let Some(oauth) = &self.oauth_result {
            let client = GcpRestClient::new(oauth.access_token.clone());

            let project_id = self.selected_project_id.clone();
            let display_name = self.new_project_name.clone();

            match client.create_project(&project_id, &display_name) {
                Ok(_operation) => {
                    // Project creation initiated successfully
                    // Note: Project creation is async and may take time
                    // For now, we'll proceed immediately and let region loading fail if needed
                    self.progress_log
                        .push(format!("✓ Project '{}' creation initiated", project_id));

                    // Add a small delay to allow project to be created
                    // In production, we should poll the operation status
                    self.progress_log
                        .push("ℹ Waiting for project to be ready...".to_string());

                    // Proceed to next step
                    self.load_regions();
                    self.state = WizardState::ConfigureServer;
                }
                Err(e) => {
                    // Show error but don't transition state
                    self.projects_load_error = Some(format!("Failed to create project: {}", e));
                }
            }
        }
    }

    fn start_server_creation(&mut self) {
        self.state = WizardState::CreatingServer;
        self.progress_log.push("Creating server...".to_string());

        let project_id = self.selected_project_id.clone();
        let zone = self.selected_zone.clone();
        let instance_name = self.instance_name.clone();
        let machine_type = self.selected_machine_type.clone();
        let platform_name = self.platform_name.clone();

        let access_token = self
            .oauth_result
            .as_ref()
            .map(|o| o.access_token.clone())
            .unwrap_or_default();

        self.create_promise = Some(Promise::spawn_thread("gcp_create_vm", move || {
            let client = GcpRestClient::new(access_token);

            // Create firewall rule if it doesn't exist
            Self::ensure_firewall_exists(&client, &project_id)
                .map_err(|e| format!("Failed to ensure firewall: {}", e))?;

            // Generate SSH key pair for this instance
            let (ssh_private_key, ssh_public_key, _raw_private, _raw_public) =
                Self::generate_ssh_key_pair()
                    .map_err(|e| format!("Failed to generate SSH key: {}", e))?;

            // Store private key in keyring
            Self::store_ssh_key_in_keyring(&instance_name, &platform_name, &ssh_private_key)
                .map_err(|e| format!("Failed to store SSH key: {}", e))?;

            // Create instance request with startup script
            let mut instance_req =
                InstanceRequest::debian_micro(instance_name.clone(), zone.clone());

            // Customize machine type if not default
            if machine_type != "e2-micro" {
                instance_req.machine_type = format!("zones/{}/machineTypes/{}", zone, machine_type);
            }

            // Generate and add startup script metadata
            let startup_script = Self::generate_startup_script(&ssh_public_key);
            instance_req.metadata = Some(Metadata {
                items: vec![MetadataItem {
                    key: "startup-script".to_string(),
                    value: startup_script,
                }],
            });

            // Create the instance
            let operation = client
                .create_instance(&project_id, &zone, &instance_req)
                .map_err(|e| format!("Failed to create instance: {}", e))?;

            // Wait for operation to complete (10 minute timeout)
            let result = client
                .wait_for_operation(&project_id, &zone, &operation.name, 600)
                .map_err(|e| format!("Operation failed: {}", e))?;

            if let Some(error) = result.error {
                return Err(format!("Creation failed: {:?}", error.errors));
            }

            // Get instance details
            let instance = client
                .get_instance(&project_id, &zone, &instance_name)
                .map_err(|e| format!("Failed to get instance: {}", e))?;

            // Extract IPs before moving other fields
            let external_ip = instance.external_ip();
            let internal_ip = instance.internal_ip();

            // Convert to our Instance type
            Ok(Instance {
                id: instance.id,
                name: instance.name,
                machine_type: instance.machine_type,
                zone: instance.zone,
                status: instance.status,
                external_ip,
                internal_ip,
                creation_timestamp: String::new(),
            })
        }));
    }

    /// Ensure Dure firewall rule exists
    fn ensure_firewall_exists(client: &GcpRestClient, project_id: &str) -> Result<(), String> {
        const FIREWALL_NAME: &str = "dure";
        const FIREWALL_TAG: &str = "dure";

        // Check if firewall already exists
        match client.list_firewalls(project_id, Some(FIREWALL_NAME)) {
            Ok(response) => {
                if response.items.is_some() && !response.items.unwrap().is_empty() {
                    // Firewall already exists
                    return Ok(());
                }
            }
            Err(_) => {
                // Error listing, will try to create anyway
            }
        }

        // Create firewall rule
        use crate::api::gcp::compute::{FirewallAllowed, FirewallRequest};

        let firewall_req = FirewallRequest {
            name: FIREWALL_NAME.to_string(),
            description: Some("Dure VM access - allows all traffic".to_string()),
            direction: "INGRESS".to_string(),
            priority: 1000,
            target_tags: vec![FIREWALL_TAG.to_string()],
            allowed: vec![FirewallAllowed {
                ip_protocol: "all".to_string(),
                ports: None,
            }],
            source_ranges: vec!["0.0.0.0/0".to_string()],
        };

        client
            .create_firewall(project_id, &firewall_req)
            .map_err(|e| format!("Failed to create firewall: {}", e))?;

        Ok(())
    }

    /// Generate Ed25519 SSH key pair
    /// Returns (private_key_pem, public_key_openssh, raw_private, raw_public)
    fn generate_ssh_key_pair() -> Result<(String, String, Vec<u8>, Vec<u8>), String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use go_webauthn_client::GoWebAuthnClient;

            let mut client = GoWebAuthnClient::new(None)
                .map_err(|e| format!("Failed to create WebAuthn client: {}", e))?;

            let keypair = client
                .ed25519_generate_key()
                .map_err(|e| format!("Failed to generate key: {}", e))?;

            // Convert to SSH format
            let private_key =
                Self::ed25519_to_openssh_private(&keypair.private_key, &keypair.public_key)?;
            let public_key = Self::ed25519_to_openssh_public(&keypair.public_key)?;

            Ok((
                private_key,
                public_key,
                keypair.private_key,
                keypair.public_key,
            ))
        }

        #[cfg(target_arch = "wasm32")]
        {
            Err("SSH key generation not supported on WASM".to_string())
        }
    }

    /// Convert Ed25519 private key to OpenSSH format
    fn ed25519_to_openssh_private(private_key: &[u8], public_key: &[u8]) -> Result<String, String> {
        // Proper OpenSSH private key format (RFC 4253 + OpenSSH extensions)
        // Reference: https://github.com/openssh/openssh-portable/blob/master/PROTOCOL.key

        // Ed25519 private key can be 32 bytes (seed) or 64 bytes (seed + public)
        let private_seed = if private_key.len() == 64 {
            // Take first 32 bytes if 64 bytes provided
            &private_key[0..32]
        } else if private_key.len() == 32 {
            private_key
        } else {
            return Err(format!(
                "Ed25519 private key must be 32 or 64 bytes, got {}",
                private_key.len()
            ));
        };

        if public_key.len() != 32 {
            return Err(format!(
                "Ed25519 public key must be 32 bytes, got {}",
                public_key.len()
            ));
        }

        let mut key_data = Vec::new();

        // Magic bytes "openssh-key-v1\0"
        key_data.extend_from_slice(b"openssh-key-v1\0");

        // Cipher name (none = unencrypted)
        Self::write_string(&mut key_data, b"none");

        // KDF name (none)
        Self::write_string(&mut key_data, b"none");

        // KDF options (empty string)
        Self::write_string(&mut key_data, b"");

        // Number of keys (1)
        key_data.extend_from_slice(&1u32.to_be_bytes());

        // Public key blob
        let mut public_blob = Vec::new();
        Self::write_string(&mut public_blob, b"ssh-ed25519");
        Self::write_string(&mut public_blob, public_key);
        Self::write_string(&mut key_data, &public_blob);

        // Private key section
        let mut private_section = Vec::new();

        // Check bytes (random, repeated)
        let check = 0x12345678u32;
        private_section.extend_from_slice(&check.to_be_bytes());
        private_section.extend_from_slice(&check.to_be_bytes());

        // Key type
        Self::write_string(&mut private_section, b"ssh-ed25519");

        // Public key
        Self::write_string(&mut private_section, public_key);

        // Private key (64 bytes: 32 private seed + 32 public for Ed25519)
        let mut full_private = Vec::new();
        full_private.extend_from_slice(private_seed);
        full_private.extend_from_slice(public_key);
        Self::write_string(&mut private_section, &full_private);

        // Comment
        Self::write_string(&mut private_section, b"dure-vm-key");

        // Padding to block size (8 bytes for unencrypted)
        let padding_len = 8 - (private_section.len() % 8);
        for i in 1..=padding_len {
            private_section.push(i as u8);
        }

        // Write private section length and data
        Self::write_string(&mut key_data, &private_section);

        // Base64 encode and wrap in PEM format
        let encoded = base64::engine::general_purpose::STANDARD.encode(&key_data);

        // Split into 70-character lines
        let mut pem = String::from("-----BEGIN OPENSSH PRIVATE KEY-----\n");
        for chunk in encoded.as_bytes().chunks(70) {
            pem.push_str(std::str::from_utf8(chunk).unwrap());
            pem.push('\n');
        }
        pem.push_str("-----END OPENSSH PRIVATE KEY-----\n");

        Ok(pem)
    }

    /// Helper to write a string with length prefix (SSH string format)
    fn write_string(buf: &mut Vec<u8>, data: &[u8]) {
        buf.extend_from_slice(&(data.len() as u32).to_be_bytes());
        buf.extend_from_slice(data);
    }

    /// Convert Ed25519 public key to OpenSSH format
    fn ed25519_to_openssh_public(public_key: &[u8]) -> Result<String, String> {
        if public_key.len() != 32 {
            return Err(format!(
                "Ed25519 public key must be 32 bytes, got {}",
                public_key.len()
            ));
        }

        // OpenSSH public key format: ssh-ed25519 <base64-encoded-blob> comment
        // The blob contains: algorithm-name-length | algorithm-name | key-length | key-bytes
        let mut blob = Vec::new();

        // Write algorithm name with length prefix
        Self::write_string(&mut blob, b"ssh-ed25519");

        // Write public key with length prefix
        Self::write_string(&mut blob, public_key);

        // Base64 encode the blob
        let encoded = base64::engine::general_purpose::STANDARD.encode(&blob);

        Ok(format!("ssh-ed25519 {} dure-vm-key", encoded))
    }

    /// Store SSH private key in keyring
    fn store_ssh_key_in_keyring(
        instance_name: &str,
        platform_name: &str,
        private_key: &str,
    ) -> Result<(), String> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let domain = format!("gcp.{}.{}", platform_name, instance_name);
            let username = "root";

            // Ensure KeePass database exists (will create KPKey if needed)
            let kdbx_path = keyring::ensure_kdbx_exists()
                .map_err(|e| format!("Failed to initialize keyring: {}", e))?;
            let kpkey_path = keyring::get_default_kpkey_path()
                .map_err(|e| format!("Failed to get KPKey path: {}", e))?;

            // Store SSH key as binary attachment, not in password field
            keyring::update_key_with_ssh(
                &kdbx_path,
                Some(&kpkey_path),
                &domain,
                username,
                "",                             // Empty password field
                Some(private_key.as_bytes()),   // SSH key as binary attachment
                Some("GCP VM SSH private key"), // Notes
            )
            .map_err(|e| format!("Failed to store SSH key: {}", e))?;

            Ok(())
        }

        #[cfg(target_arch = "wasm32")]
        {
            Err("Keyring not supported on WASM".to_string())
        }
    }

    /// Generate startup script for VM initialization
    fn generate_startup_script(ssh_public_key: &str) -> String {
        format!(
            r#"#!/bin/bash
# Don't exit on errors - we want SSH config to run even if swap fails
set -uo pipefail

# Log output
exec &> /var/log/dure-startup.log
echo "Starting Dure VM initialization at $(date)"

# Enable BBR congestion control
echo "Enabling BBR..."
cat >> /etc/sysctl.conf << 'EOF'

# Added by Dure
net.core.default_qdisc=fq
net.ipv4.tcp_congestion_control=bbr
EOF
sysctl -p

# Add swap if memory is less than 8GB
TOTAL_MEM_KB=$(grep MemTotal /proc/meminfo | awk '{{print $2}}')
TOTAL_MEM_GB=$((TOTAL_MEM_KB / 1024 / 1024))
DISK_AVAIL_GB=$(df -BG / | awk 'NR==2 {{print $4}}' | sed 's/G//')

if [ $TOTAL_MEM_GB -lt 8 ]; then
    # Determine swap size based on available disk space
    # Reserve 2GB for system, use the rest for swap (up to 8GB max)
    if [ $DISK_AVAIL_GB -gt 10 ]; then
        SWAP_SIZE_GB=8
    elif [ $DISK_AVAIL_GB -gt 2 ]; then
        SWAP_SIZE_GB=$((DISK_AVAIL_GB - 2))
    else
        echo "Insufficient disk space (${{DISK_AVAIL_GB}}GB available), skipping swap"
        SWAP_SIZE_GB=0
    fi

    if [ $SWAP_SIZE_GB -gt 0 ]; then
        echo "Total memory is ${{TOTAL_MEM_GB}}GB, disk available ${{DISK_AVAIL_GB}}GB"
        echo "Creating ${{SWAP_SIZE_GB}}GB swap..."

        # Try fallocate first, fall back to dd
        if fallocate -l "${{SWAP_SIZE_GB}}G" /swapfile 2>/dev/null; then
            echo "Created ${{SWAP_SIZE_GB}}GB swap with fallocate"
        elif dd if=/dev/zero of=/swapfile bs=1G count=$SWAP_SIZE_GB 2>/dev/null; then
            echo "Created ${{SWAP_SIZE_GB}}GB swap with dd"
        else
            echo "WARNING: Failed to create swap file, continuing without swap..."
        fi

        # Only configure swap if file was created successfully
        if [ -f /swapfile ] && [ -s /swapfile ]; then
            chmod 600 /swapfile
            mkswap /swapfile
            swapon /swapfile
            echo '/swapfile none swap sw 0 0' >> /etc/fstab
            echo "Swap added successfully: ${{SWAP_SIZE_GB}}GB"
        fi
    fi
else
    echo "Memory is ${{TOTAL_MEM_GB}}GB, no swap needed"
fi

# Configure SSH for root access with key authentication
echo "Configuring SSH..."

# Create .ssh directory for root
mkdir -p /root/.ssh
chmod 700 /root/.ssh

# Add SSH public key to authorized_keys (append to preserve GCP Console SSH access)
touch /root/.ssh/authorized_keys
chmod 600 /root/.ssh/authorized_keys
echo "{}" >> /root/.ssh/authorized_keys

# Configure sshd
sed -i 's/#*PermitRootLogin.*/PermitRootLogin prohibit-password/' /etc/ssh/sshd_config
sed -i 's/#*PasswordAuthentication.*/PasswordAuthentication no/' /etc/ssh/sshd_config
sed -i 's/#*PubkeyAuthentication.*/PubkeyAuthentication yes/' /etc/ssh/sshd_config

# Restart sshd
systemctl restart sshd || systemctl restart ssh

echo "Dure VM initialization completed at $(date)"
"#,
            ssh_public_key
        )
    }

    /// Validate GCP project ID
    /// Rules: 6-30 characters, lowercase letters, digits, hyphens, start with letter
    fn validate_project_id(&self, id: &str) -> bool {
        if id.is_empty() || id.len() < 6 || id.len() > 30 {
            return false;
        }

        // Must start with a lowercase letter
        if !id
            .chars()
            .next()
            .map(|c| c.is_ascii_lowercase())
            .unwrap_or(false)
        {
            return false;
        }

        // Can only contain lowercase letters, digits, and hyphens
        id.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    }

    /// Validate GCP instance name
    /// Rules: 1-63 characters, lowercase letters, numbers, hyphens, start with letter
    fn validate_instance_name(&self, name: &str) -> bool {
        if name.is_empty() || name.len() > 63 {
            return false;
        }

        // Must start with a lowercase letter
        if !name
            .chars()
            .next()
            .map(|c| c.is_ascii_lowercase())
            .unwrap_or(false)
        {
            return false;
        }

        // Can only contain lowercase letters, numbers, and hyphens
        name.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    }
}

/// Validate disk size (10 GB minimum, 65536 GB maximum)
fn validate_disk_size(input: &str) -> Result<u32, String> {
    let size = input.parse::<u32>()
        .map_err(|_| "Must be a valid number".to_string())?;

    if size < 10 {
        return Err("Minimum disk size is 10 GB".to_string());
    }

    if size > 65536 {
        return Err("Maximum disk size is 65536 GB".to_string());
    }

    Ok(size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_project_id() {
        let wizard = GcpWizard::new("test-platform".to_string());

        // Valid project IDs
        assert!(wizard.validate_project_id("my-project"));
        assert!(wizard.validate_project_id("project123"));
        assert!(wizard.validate_project_id("a".repeat(30).as_str()));

        // Invalid: too short
        assert!(!wizard.validate_project_id("short"));

        // Invalid: too long
        assert!(!wizard.validate_project_id(&"a".repeat(31)));

        // Invalid: doesn't start with letter
        assert!(!wizard.validate_project_id("123project"));
        assert!(!wizard.validate_project_id("-project"));

        // Invalid: uppercase
        assert!(!wizard.validate_project_id("MyProject"));

        // Invalid: special characters
        assert!(!wizard.validate_project_id("my_project"));
        assert!(!wizard.validate_project_id("my.project"));
    }

    #[test]
    fn test_validate_instance_name() {
        let wizard = GcpWizard::new("test-platform".to_string());

        // Valid instance names
        assert!(wizard.validate_instance_name("my-instance"));
        assert!(wizard.validate_instance_name("instance123"));
        assert!(wizard.validate_instance_name("a"));

        // Invalid: empty
        assert!(!wizard.validate_instance_name(""));

        // Invalid: too long
        assert!(!wizard.validate_instance_name(&"a".repeat(64)));

        // Invalid: doesn't start with letter
        assert!(!wizard.validate_instance_name("123instance"));

        // Invalid: uppercase
        assert!(!wizard.validate_instance_name("MyInstance"));
    }

    #[test]
    fn test_write_string() {
        let mut buf = Vec::new();
        GcpWizard::write_string(&mut buf, b"test");

        // Should write length (4 bytes) + data
        assert_eq!(buf.len(), 8);
        assert_eq!(&buf[0..4], &[0, 0, 0, 4]); // Length = 4 in big-endian
        assert_eq!(&buf[4..8], b"test");
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_ed25519_to_openssh_public() {
        // Valid 32-byte public key
        let public_key = vec![1u8; 32];
        let result = GcpWizard::ed25519_to_openssh_public(&public_key);

        assert!(result.is_ok());
        let ssh_key = result.unwrap();

        // Should start with "ssh-ed25519 "
        assert!(ssh_key.starts_with("ssh-ed25519 "));

        // Should end with " dure-vm-key"
        assert!(ssh_key.ends_with(" dure-vm-key"));
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_ed25519_to_openssh_public_invalid_length() {
        // Invalid key length (not 32 bytes)
        let public_key = vec![1u8; 30];
        let result = GcpWizard::ed25519_to_openssh_public(&public_key);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be 32 bytes"));
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_ed25519_to_openssh_private() {
        // Valid keys (32-byte private, 32-byte public)
        let private_key = vec![1u8; 32];
        let public_key = vec![2u8; 32];

        let result = GcpWizard::ed25519_to_openssh_private(&private_key, &public_key);

        assert!(result.is_ok());
        let ssh_key = result.unwrap();

        // Should start with PEM header
        assert!(ssh_key.starts_with("-----BEGIN OPENSSH PRIVATE KEY-----"));

        // Should end with PEM footer
        assert!(ssh_key.ends_with("-----END OPENSSH PRIVATE KEY-----\n"));

        // Should contain base64 data
        assert!(ssh_key.contains('\n'));
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_ed25519_to_openssh_private_64byte_key() {
        // Valid 64-byte private key (should use first 32 bytes)
        let private_key = vec![1u8; 64];
        let public_key = vec![2u8; 32];

        let result = GcpWizard::ed25519_to_openssh_private(&private_key, &public_key);

        assert!(result.is_ok());
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_ed25519_to_openssh_private_invalid_lengths() {
        // Invalid private key length
        let private_key = vec![1u8; 30];
        let public_key = vec![2u8; 32];

        let result = GcpWizard::ed25519_to_openssh_private(&private_key, &public_key);
        assert!(result.is_err());

        // Invalid public key length
        let private_key = vec![1u8; 32];
        let public_key = vec![2u8; 30];

        let result = GcpWizard::ed25519_to_openssh_private(&private_key, &public_key);
        assert!(result.is_err());
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_generate_ssh_key_pair() {
        // This test requires go-webauthn-cli to be built
        // Run with: PATH="$PWD/crates/go-webauthn/bin:$PATH" cargo test test_generate_ssh_key_pair -- --ignored --nocapture

        let result = GcpWizard::generate_ssh_key_pair();

        // Should succeed
        assert!(
            result.is_ok(),
            "Failed to generate SSH key pair: {:?}",
            result
        );

        let (private_key, public_key, raw_private, raw_public) = result.unwrap();

        // Check private key format
        assert!(private_key.starts_with("-----BEGIN OPENSSH PRIVATE KEY-----"));
        assert!(private_key.ends_with("-----END OPENSSH PRIVATE KEY-----\n"));

        // Check public key format
        assert!(public_key.starts_with("ssh-ed25519 "));
        assert!(public_key.ends_with(" dure-vm-key"));

        // Check raw key lengths
        assert_eq!(raw_public.len(), 32, "Public key should be 32 bytes");
        assert_eq!(raw_private.len(), 64, "Private key should be 64 bytes");
    }

    #[test]
    #[cfg(target_arch = "wasm32")]
    fn test_generate_ssh_key_pair_wasm() {
        let result = GcpWizard::generate_ssh_key_pair();

        // Should fail on WASM
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not supported on WASM"));
    }

    #[test]
    fn test_generate_startup_script() {
        let ssh_key = "ssh-ed25519 AAAAC3... test@example.com";
        let script = GcpWizard::generate_startup_script(ssh_key);

        // Should be a bash script
        assert!(script.starts_with("#!/bin/bash"));

        // Should contain the SSH key
        assert!(script.contains(ssh_key));

        // Should set up SSH
        assert!(script.contains("mkdir -p /root/.ssh"));
        assert!(script.contains("authorized_keys"));

        // Should create swap
        assert!(script.contains("swap"));
    }

    #[test]
    fn test_validate_disk_size() {
        // Valid sizes
        assert!(validate_disk_size("10").is_ok());
        assert_eq!(validate_disk_size("10").unwrap(), 10);
        assert!(validate_disk_size("100").is_ok());
        assert!(validate_disk_size("65536").is_ok());

        // Invalid: too small
        assert_eq!(
            validate_disk_size("9").unwrap_err(),
            "Minimum disk size is 10 GB"
        );

        // Invalid: too large
        assert_eq!(
            validate_disk_size("65537").unwrap_err(),
            "Maximum disk size is 65536 GB"
        );

        // Invalid: not a number
        assert_eq!(
            validate_disk_size("abc").unwrap_err(),
            "Must be a valid number"
        );
        assert!(validate_disk_size("").is_err());
        assert!(validate_disk_size("-5").is_err());
    }
}
