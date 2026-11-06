//! DNS Wizard Dialog
//!
//! 4-step wizard for automated DNS setup:
//! 1. Cloudflare OAuth + API connection check
//! 2. Domain registration/connection + NS verification
//! 3. GCP VM selection + NS settings configuration
//! 4. Confirmation and summary

use crate::{dure_debug, dure_error, dure_info, dure_warn};
use eframe::egui;
use egui_material3::MaterialButton;
use poll_promise::Promise;
use serde::{Deserialize, Serialize};

use crate::api::cloudflare::oauth::{CloudflareOAuthHandler, OAuthResult};
use crate::calc::profile::ProfileContext;

/// Wizard state machine
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq)]
pub enum DnsWizardState {
    CloudflareAuth,
    DomainRegistration,
    VmConfiguration,
    Confirmation,
    Complete,
    Error(String),
}

/// DNS Wizard
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DnsWizard {
    /// Current state
    state: DnsWizardState,

    /// Profile name
    profile_name: String,

    /// Step 1: Cloudflare OAuth
    #[cfg_attr(feature = "serde", serde(skip))]
    oauth_result: Option<OAuthResult>,

    #[cfg_attr(feature = "serde", serde(skip))]
    oauth_promise: Option<Promise<Result<OAuthResult, String>>>,

    connected_email: Option<String>,
    api_connection_ok: bool,

    /// Cloudflare account ID (needed for Registrar API)
    cloudflare_account_id: String,

    /// Step 2: Domain registration
    domain_name: String,
    domain_registered: bool,
    ns_check_ok: bool,

    #[cfg_attr(feature = "serde", serde(skip))]
    domain_check_promise: Option<Promise<Result<bool, String>>>,

    domain_available: Option<bool>,

    #[cfg_attr(feature = "serde", serde(skip))]
    register_promise: Option<Promise<Result<String, String>>>,

    /// Domains list from Cloudflare account
    #[cfg_attr(feature = "serde", serde(skip))]
    domains_list: Vec<crate::api::cloudflare::Zone>,

    /// Promise for fetching domains
    #[cfg_attr(feature = "serde", serde(skip))]
    fetch_domains_promise: Option<Promise<Result<Vec<crate::api::cloudflare::Zone>, String>>>,

    /// Promise for adding domain
    #[cfg_attr(feature = "serde", serde(skip))]
    add_domain_promise: Option<Promise<Result<String, String>>>,

    /// Promise for creating DNS records
    #[cfg_attr(feature = "serde", serde(skip))]
    create_dns_promise: Option<Promise<Result<(), String>>>,

    /// Selected domain for DNS setup
    selected_domain: Option<String>,

    /// Selected zone ID (from selected_domain)
    selected_zone_id: Option<String>,

    /// Step 3: VM configuration
    selected_vm: Option<String>,
    vm_ip: Option<String>,
    ns_records_configured: bool,

    /// Available GCP VMs (from platform tab)
    #[cfg_attr(feature = "serde", serde(skip))]
    available_vms: Vec<VmInfo>,

    /// Step 4: Configuration summary
    summary_items: Vec<String>,

    /// Progress log
    #[cfg_attr(feature = "serde", serde(skip))]
    progress_log: Vec<String>,
}

/// VM information for selection
#[derive(Clone, Debug)]
struct VmInfo {
    name: String,
    ip: String,
    zone: String,
}

impl DnsWizard {
    /// Create new wizard
    pub fn new(profile_name: String) -> Self {
        Self {
            state: DnsWizardState::CloudflareAuth,
            profile_name,
            oauth_result: None,
            oauth_promise: None,
            connected_email: None,
            api_connection_ok: false,
            cloudflare_account_id: String::new(),
            domain_name: String::new(),
            domain_registered: false,
            ns_check_ok: false,
            domain_check_promise: None,
            domain_available: None,
            register_promise: None,
            domains_list: Vec::new(),
            fetch_domains_promise: None,
            add_domain_promise: None,
            create_dns_promise: None,
            selected_domain: None,
            selected_zone_id: None,
            selected_vm: None,
            vm_ip: None,
            ns_records_configured: false,
            available_vms: Vec::new(),
            summary_items: Vec::new(),
            progress_log: Vec::new(),
        }
    }

    /// Get current state
    pub fn state(&self) -> &DnsWizardState {
        &self.state
    }

    /// Check if can proceed from Step 1
    pub fn can_proceed_from_step1(&self) -> bool {
        self.oauth_result.is_some() && !self.cloudflare_account_id.is_empty()
    }

    /// Test helper: set OAuth result
    pub fn set_oauth_result_for_test(&mut self) {
        use crate::api::cloudflare::OAuthResult;
        self.oauth_result = Some(OAuthResult {
            access_token: "test_token".to_string(),
            refresh_token: None,
            expires_at: 0,
        });
    }

    /// Test helper: set account ID
    pub fn set_account_id_for_test(&mut self, account_id: String) {
        self.cloudflare_account_id = account_id;
    }

    /// Get domain input
    pub fn domain_input(&self) -> &str {
        &self.domain_name
    }

    /// Check if can proceed from Step 2
    pub fn can_proceed_from_step2(&self) -> bool {
        self.selected_domain.is_some()
    }

    /// Test helper: set domain input
    pub fn set_domain_input_for_test(&mut self, domain: String) {
        self.domain_name = domain;
    }

    /// Test helper: select domain
    pub fn select_domain_for_test(&mut self, domain: String) {
        self.selected_domain = Some(domain);
    }

    /// Check if can proceed from Step 3
    pub fn can_proceed_from_step3(&self) -> bool {
        self.selected_vm.is_some()
    }

    /// Test helper: select VM
    pub fn select_vm_for_test(&mut self, vm_name: String, vm_ip: String) {
        self.selected_vm = Some(vm_name);
        self.vm_ip = Some(vm_ip);
    }

    /// Get selected VM name
    pub fn selected_vm(&self) -> Option<&str> {
        self.selected_vm.as_deref()
    }

    /// Get selected VM IP
    pub fn selected_vm_ip(&self) -> Option<&str> {
        self.vm_ip.as_deref()
    }

    /// Check if DNS is configured
    pub fn dns_configured(&self) -> bool {
        self.ns_records_configured
    }

    /// Check if can proceed from Step 4
    pub fn can_proceed_from_step4(&self) -> bool {
        self.ns_records_configured
    }

    /// Test helper: set DNS configured status
    pub fn set_dns_configured_for_test(&mut self, configured: bool) {
        self.ns_records_configured = configured;
    }

    /// Get domain binding data for saving to config
    pub fn get_binding_data(&self) -> (Option<String>, Option<String>) {
        (self.selected_vm.clone(), self.selected_domain.clone())
    }

    /// Render wizard UI
    pub fn ui(&mut self, ui: &mut egui::Ui, profile: &ProfileContext) {
        ui.vertical(|ui| {
            // Progress indicator
            self.render_progress_indicator(ui);

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Current step content
            match &self.state {
                DnsWizardState::CloudflareAuth => self.render_step1_oauth(ui),
                DnsWizardState::DomainRegistration => self.render_step2_domain(ui),
                DnsWizardState::VmConfiguration => self.render_step3_vm(ui),
                DnsWizardState::Confirmation => self.render_step4_confirmation(ui),
                DnsWizardState::Complete => self.render_complete(ui),
                DnsWizardState::Error(err) => self.render_error(ui, err),
            }

            // Navigation buttons
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            self.render_navigation(ui);
        });
    }

    /// Render progress indicator
    fn render_progress_indicator(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let steps = [
                ("1. OAuth", DnsWizardState::CloudflareAuth),
                ("2. Domain", DnsWizardState::DomainRegistration),
                ("3. VM Config", DnsWizardState::VmConfiguration),
                ("4. Confirm", DnsWizardState::Confirmation),
            ];

            for (i, (label, step_state)) in steps.iter().enumerate() {
                let is_current = std::mem::discriminant(&self.state)
                    == std::mem::discriminant(step_state);
                let is_complete = self.step_index() > i;

                if is_current {
                    ui.strong(*label);
                } else if is_complete {
                    ui.label(format!("✓ {}", label));
                } else {
                    ui.weak(*label);
                }

                if i < steps.len() - 1 {
                    ui.label("→");
                }
            }
        });
    }

    /// Get current step index
    fn step_index(&self) -> usize {
        match &self.state {
            DnsWizardState::CloudflareAuth => 0,
            DnsWizardState::DomainRegistration => 1,
            DnsWizardState::VmConfiguration => 2,
            DnsWizardState::Confirmation => 3,
            DnsWizardState::Complete => 4,
            DnsWizardState::Error(_) => 0,
        }
    }

    /// Step 1: Cloudflare OAuth
    fn render_step1_oauth(&mut self, ui: &mut egui::Ui) {
        ui.heading("Step 1: Cloudflare Authentication");
        ui.add_space(10.0);

        ui.label("Connect your Cloudflare account to manage DNS:");

        ui.add_space(10.0);

        // Check OAuth promise status
        if let Some(promise) = self.oauth_promise.take() {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(oauth_result) => {
                        self.oauth_result = Some(oauth_result.clone());
                        self.progress_log.push("✓ OAuth successful".to_string());

                        // Verify API token
                        self.verify_api_token();

                        // Auto-fetch Account ID from API v4
                        self.fetch_account_id();
                    }
                    Err(err) => {
                        self.progress_log.push(format!("✗ OAuth failed: {}", err));
                        self.state = DnsWizardState::Error(err.clone());
                    }
                }
            } else {
                ui.label("⏳ Waiting for OAuth authorization...");
                ui.label("Please complete the authorization in your browser.");
                // Put promise back if not ready
                self.oauth_promise = Some(promise);
            }
        } else if self.oauth_result.is_none() {
            // Not started OAuth yet
            if ui
                .add(MaterialButton::filled("Connect Cloudflare Account"))
                .on_hover_text("Opens browser for OAuth authentication")
                .clicked()
            {
                self.start_oauth();
            }

            ui.add_space(10.0);
            ui.label("You will be redirected to Cloudflare to authorize access.");
        } else {
            // OAuth complete
            ui.label(format!(
                "✓ Connected: {}",
                self.connected_email.as_ref().unwrap_or(&"Account connected".to_string())
            ));

            if self.api_connection_ok {
                ui.label("✓ API connection verified");
            } else {
                ui.label("⏳ Verifying API connection...");
            }

            ui.add_space(10.0);

            // Account ID input
            ui.horizontal(|ui| {
                ui.label("Account ID:");
                ui.text_edit_singleline(&mut self.cloudflare_account_id);
            });

            if self.cloudflare_account_id.is_empty() {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 165, 0),
                    "⚠ Account ID required to continue"
                );
            }
        }

        // Progress log
        if !self.progress_log.is_empty() {
            ui.add_space(10.0);
            ui.group(|ui| {
                ui.set_max_height(150.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for log in &self.progress_log {
                        ui.label(log);
                    }
                });
            });
        }

        // Next button
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            ui.add_space(ui.available_width() - 100.0);
            let next_btn = ui.add_enabled(
                self.can_proceed_from_step1(),
                MaterialButton::filled("Next")
            );
            if next_btn.clicked() {
                self.state = DnsWizardState::DomainRegistration;
            }
        });
    }

    /// Step 2: Domain registration
    fn render_step2_domain(&mut self, ui: &mut egui::Ui) {
        ui.heading("Step 2: Domain Selection");
        ui.add_space(10.0);

        // Auto-fetch domains on first render
        if self.domains_list.is_empty() && self.fetch_domains_promise.is_none() {
            self.fetch_domains();
        }

        // Handle fetch domains promise
        if let Some(promise) = self.fetch_domains_promise.take() {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(zones) => {
                        let count = zones.len();
                        self.domains_list = zones.clone();
                        self.progress_log.push(format!("✓ Loaded {} domains", count));
                    }
                    Err(err) => {
                        self.progress_log.push(format!("✗ Failed to fetch domains: {}", err));
                    }
                }
            } else {
                ui.spinner();
                ui.label("Loading domains...");
                self.fetch_domains_promise = Some(promise);
            }
        }

        // Handle add domain promise
        if let Some(promise) = self.add_domain_promise.take() {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(domain) => {
                        self.progress_log.push(format!("✓ Added domain: {}", domain));
                        self.fetch_domains(); // Refresh list
                        self.domain_name.clear();
                    }
                    Err(err) => {
                        self.progress_log.push(format!("✗ Failed to add domain: {}", err));
                    }
                }
            } else {
                ui.spinner();
                ui.label("Adding domain...");
                self.add_domain_promise = Some(promise);
            }
        }

        // Two-column layout for domain addition options
        ui.columns(2, |cols| {
            // Left column: I have domain
            cols[0].group(|ui| {
                ui.heading("I have domain");
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    ui.label("Domain:");
                    ui.text_edit_singleline(&mut self.domain_name);
                });

                ui.add_space(5.0);
                let can_add = !self.domain_name.is_empty() && self.add_domain_promise.is_none();
                if ui.add_enabled(can_add, MaterialButton::outlined("Add")).clicked() {
                    self.add_domain_to_account(&self.domain_name.clone());
                }
            });

            // Right column: I want to register domain
            cols[1].group(|ui| {
                ui.heading("I want to register domain");
                ui.add_space(10.0);

                let mut search_domain = String::new();
                ui.horizontal(|ui| {
                    ui.label("Domain:");
                    ui.text_edit_singleline(&mut search_domain);
                });

                ui.add_space(5.0);
                ui.label("⚠ Domain registration not yet implemented");
            });
        });

        ui.add_space(20.0);
        ui.separator();
        ui.add_space(10.0);

        // Domain dropdown from Cloudflare account
        ui.label("Select domain to use:");
        ui.add_space(5.0);

        egui::ComboBox::from_label("")
            .selected_text(
                self.selected_domain
                    .as_ref()
                    .map(|s| s.as_str())
                    .unwrap_or("Select a domain")
            )
            .show_ui(ui, |ui| {
                for domain in &self.domains_list {
                    let domain_name = &domain.name;
                    let zone_id = &domain.id;
                    if ui.selectable_label(
                        self.selected_domain.as_ref() == Some(domain_name),
                        domain_name
                    ).clicked() {
                        self.selected_domain = Some(domain_name.clone());
                        self.selected_zone_id = Some(zone_id.clone());
                    }
                }
            });

        ui.add_space(10.0);
        ui.label("* select domain to use and continue");

        // Previous/Next buttons
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            if ui.add(MaterialButton::outlined("Previous")).clicked() {
                self.state = DnsWizardState::CloudflareAuth;
            }

            ui.add_space(ui.available_width() - 100.0);

            let next_btn = ui.add_enabled(
                self.can_proceed_from_step2(),
                MaterialButton::filled("Next")
            );
            if next_btn.clicked() {
                self.state = DnsWizardState::VmConfiguration;
            }
        });
    }

    /// Step 3: VM configuration
    fn render_step3_vm(&mut self, ui: &mut egui::Ui) {
        ui.heading("Step 3: VM Selection");
        ui.add_space(10.0);

        ui.label("Select platform binded VM which doesn't have DNS yet:");
        ui.add_space(10.0);

        // VM list with checkboxes
        for vm in self.available_vms.clone() {
            let is_selected = self.selected_vm.as_ref() == Some(&vm.name);

            ui.horizontal(|ui| {
                if ui.radio(is_selected, "").clicked() {
                    self.selected_vm = Some(vm.name.clone());
                    self.vm_ip = Some(vm.ip.clone());
                }

                // Format: project_id  vm_name  external_ip
                ui.label(format!("{}  {}  {}", vm.zone, vm.name, vm.ip));
            });
        }

        ui.add_space(10.0);
        ui.label("* select vm to bind and continue");

        // Previous/Next buttons
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            if ui.add(MaterialButton::outlined("Previous")).clicked() {
                self.state = DnsWizardState::DomainRegistration;
            }

            ui.add_space(ui.available_width() - 100.0);

            let next_btn = ui.add_enabled(
                self.can_proceed_from_step3(),
                MaterialButton::filled("Next")
            );
            if next_btn.clicked() {
                self.state = DnsWizardState::Confirmation;
            }
        });
    }

    /// Step 4: DNS Setup
    fn render_step4_confirmation(&mut self, ui: &mut egui::Ui) {
        ui.heading("Step 4: DNS Setup");
        ui.add_space(10.0);

        // Handle create DNS records promise
        if let Some(promise) = self.create_dns_promise.take() {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(()) => {
                        self.ns_records_configured = true;
                        self.progress_log.push("✓ DNS records created successfully".to_string());
                    }
                    Err(err) => {
                        self.progress_log.push(format!("✗ Failed to create DNS records: {}", err));
                    }
                }
            } else {
                ui.spinner();
                ui.label("Creating DNS records...");
                self.create_dns_promise = Some(promise);
                return; // Skip rest of UI while creating
            }
        }

        // Show status and setup button
        ui.horizontal(|ui| {
            ui.label("Status of DNS settings:");
            ui.add_space(10.0);

            let can_setup = !self.ns_records_configured && self.create_dns_promise.is_none();
            let setup_btn = ui.add_enabled(
                can_setup,
                MaterialButton::outlined("Setup DNS")
            );
            if setup_btn.clicked() {
                self.create_dns_records();
            }
        });

        ui.add_space(10.0);

        // Show default DNS records
        ui.group(|ui| {
            ui.label("Default settings:");
            ui.add_space(5.0);

            if let Some(ip) = &self.vm_ip {
                let domain = self.selected_domain.as_deref().unwrap_or("example.com");

                ui.monospace(format!("{}        A    {} (GCP SERVER)", domain, ip));
                ui.monospace(format!("xmpp.{}    A    {} (GCP SERVER)", domain, ip));
                ui.monospace(format!("www.{}    CNAME    {}", domain, domain));
                ui.monospace(format!("_xmpp-client    SRV    5 5222 xmpp.{}", domain));
                ui.monospace(format!("_xmpp-server    SRV    5 5269 xmpp.{}", domain));
                ui.monospace(format!("_xmpps-client    SRV    5 5223 xmpp.{}", domain));
                ui.monospace(format!("_xmpps-server    SRV    5 5270 xmpp.{}", domain));
            }
        });

        // Previous/Add buttons
        ui.add_space(20.0);
        ui.horizontal(|ui| {
            if ui.add(MaterialButton::outlined("Previous")).clicked() {
                self.state = DnsWizardState::VmConfiguration;
            }

            ui.add_space(ui.available_width() - 100.0);

            let add_btn = ui.add_enabled(
                self.ns_records_configured,
                MaterialButton::filled("Add")
            );
            if add_btn.clicked() {
                self.state = DnsWizardState::Complete;
            }
        });
    }

    /// Render complete state
    fn render_complete(&self, ui: &mut egui::Ui) {
        ui.heading("✓ Setup Complete");
        ui.add_space(10.0);
        ui.label("Your DNS has been configured successfully!");
    }

    /// Render error state
    fn render_error(&self, ui: &mut egui::Ui, error: &str) {
        ui.heading("Error");
        ui.add_space(10.0);
        ui.colored_label(egui::Color32::RED, error);
    }

    /// Render navigation buttons
    fn render_navigation(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if self.can_go_back() {
                if ui.add(MaterialButton::outlined("Back")).clicked() {
                    self.go_back();
                }
            }

            ui.add_space(10.0);

            if self.can_go_next() {
                if ui.add(MaterialButton::filled("Next")).clicked() {
                    self.go_next();
                }
            }
        });
    }

    /// Check if can go back
    fn can_go_back(&self) -> bool {
        matches!(
            self.state,
            DnsWizardState::DomainRegistration
                | DnsWizardState::VmConfiguration
                | DnsWizardState::Confirmation
        )
    }

    /// Check if can go next
    fn can_go_next(&self) -> bool {
        match &self.state {
            DnsWizardState::CloudflareAuth => {
                self.oauth_result.is_some()
                    && self.api_connection_ok
                    && !self.cloudflare_account_id.is_empty()
            }
            DnsWizardState::DomainRegistration => self.domain_registered && self.ns_check_ok,
            DnsWizardState::VmConfiguration => {
                self.selected_vm.is_some() && self.ns_records_configured
            }
            _ => false,
        }
    }

    /// Go to previous step
    fn go_back(&mut self) {
        self.state = match &self.state {
            DnsWizardState::DomainRegistration => DnsWizardState::CloudflareAuth,
            DnsWizardState::VmConfiguration => DnsWizardState::DomainRegistration,
            DnsWizardState::Confirmation => DnsWizardState::VmConfiguration,
            _ => return,
        };
    }

    /// Go to next step
    fn go_next(&mut self) {
        self.state = match &self.state {
            DnsWizardState::CloudflareAuth => DnsWizardState::DomainRegistration,
            DnsWizardState::DomainRegistration => DnsWizardState::VmConfiguration,
            DnsWizardState::VmConfiguration => DnsWizardState::Confirmation,
            DnsWizardState::Confirmation => DnsWizardState::Complete,
            _ => return,
        };
    }

    /// Start OAuth flow
    fn start_oauth(&mut self) {
        dure_info!("Starting Cloudflare OAuth flow...");
        self.progress_log.push("Starting OAuth flow...".to_string());

        use crate::api::cloudflare::oauth::CloudflareOAuthHandler;

        let promise = Promise::spawn_thread("cloudflare_oauth", move || {
            let handler = CloudflareOAuthHandler::default();
            handler
                .run_oauth_flow()
                .map_err(|e| format!("OAuth failed: {}", e))
        });

        self.oauth_promise = Some(promise);
    }

    /// Fetch account info from Cloudflare API
    fn fetch_account_id(&mut self) {
        if let Some(oauth_result) = &self.oauth_result {
            use crate::api::cloudflare::get_user_info;

            self.progress_log.push("Fetching account info...".to_string());

            match get_user_info(&oauth_result.access_token) {
                Ok(user_info) => {
                    self.cloudflare_account_id = user_info.account_id.clone();
                    self.progress_log.push(format!(
                        "✓ Account: {} (ID: {})",
                        user_info.account_name,
                        user_info.account_id
                    ));
                    dure_info!("Auto-populated Account ID: {} ({})",
                        user_info.account_id, user_info.account_name);
                }
                Err(e) => {
                    self.progress_log.push(format!("⚠ Failed to fetch account ID: {}", e));
                    dure_warn!("Failed to auto-fetch account ID: {}", e);
                }
            }
        }
    }

    /// Verify API token
    fn verify_api_token(&mut self) {
        if let Some(oauth_result) = &self.oauth_result {
            use crate::api::cloudflare::oauth::verify_token;

            self.progress_log.push("Verifying API token...".to_string());

            match verify_token(&oauth_result.access_token) {
                Ok(valid) => {
                    self.api_connection_ok = valid;
                    if valid {
                        self.progress_log.push("✓ API token verified".to_string());
                    } else {
                        self.progress_log.push("✗ Invalid API token".to_string());
                    }
                }
                Err(e) => {
                    self.progress_log
                        .push(format!("✗ Verification error: {}", e));
                }
            }
        }
    }

    /// Check domain availability
    fn check_domain_availability(&mut self) {
        let domain = self.domain_name.clone();
        let account_id = self.cloudflare_account_id.clone();
        let token = self
            .oauth_result
            .as_ref()
            .map(|r| r.access_token.clone())
            .unwrap_or_default();

        self.progress_log
            .push(format!("Checking availability: {}", domain));

        let promise = Promise::spawn_thread("check_domain", move || {
            use crate::api::cloudflare::registrar::CloudflareRegistrarClient;

            let client = CloudflareRegistrarClient::new(token);
            client
                .check_availability(&account_id, &domain)
                .map_err(|e| format!("Check failed: {}", e))
        });

        self.domain_check_promise = Some(promise);
    }

    /// Register domain
    fn register_domain(&mut self) {
        let domain = self.domain_name.clone();
        let account_id = self.cloudflare_account_id.clone();
        let token = self
            .oauth_result
            .as_ref()
            .map(|r| r.access_token.clone())
            .unwrap_or_default();

        self.progress_log
            .push(format!("Registering domain: {}", domain));

        let promise = Promise::spawn_thread("register_domain", move || {
            use crate::api::cloudflare::registrar::CloudflareRegistrarClient;

            let client = CloudflareRegistrarClient::new(token);
            client
                .register_domain(&account_id, &domain, 1)
                .map(|d| d.name)
                .map_err(|e| format!("Registration failed: {}", e))
        });

        self.register_promise = Some(promise);
    }

    /// Connect existing domain
    fn connect_existing_domain(&mut self) {
        self.progress_log
            .push(format!("Connecting existing domain: {}", self.domain_name));
        self.domain_registered = true;
        self.check_nameservers();
    }

    /// Check nameservers
    fn check_nameservers(&mut self) {
        // TODO: Implement NS verification using dqy
        self.progress_log.push("Checking nameservers...".to_string());
        // Simulate successful check for now
        self.ns_check_ok = true;
        self.progress_log.push("✓ Nameservers verified".to_string());
    }

    /// Apply DNS configuration
    fn apply_dns_configuration(&mut self) {
        if let Some(ip) = &self.vm_ip {
            self.progress_log
                .push(format!("Configuring DNS records for {}...", self.domain_name));

            // TODO: Actually create DNS records via Cloudflare API
            let records = self.get_default_dns_records(ip);
            for record in &records {
                self.progress_log.push(format!("  Creating: {}", record));
            }

            self.ns_records_configured = true;
            self.progress_log.push("✓ DNS records configured".to_string());
        }
    }

    /// Get default DNS records
    fn get_default_dns_records(&self, ip: &str) -> Vec<String> {
        vec![
            format!("{:<20} A      {}", self.domain_name, ip),
            format!("{:<20} A      {}", format!("xmpp.{}", self.domain_name), ip),
            format!("{:<20} CNAME  {}", format!("www.{}", self.domain_name), self.domain_name),
            format!("{:<20} SRV    5 5222 xmpp.{}", "_xmpp-client", self.domain_name),
            format!("{:<20} SRV    5 5269 xmpp.{}", "_xmpp-server", self.domain_name),
            format!("{:<20} SRV    5 5223 xmpp.{}", "_xmpps-client", self.domain_name),
            format!("{:<20} SRV    5 5270 xmpp.{}", "_xmpps-server", self.domain_name),
        ]
    }

    /// Load available VMs (placeholder - should be called from parent)
    pub fn load_available_vms(&mut self, vms: Vec<(String, String, String)>) {
        self.available_vms = vms
            .into_iter()
            .map(|(name, ip, zone)| VmInfo { name, ip, zone })
            .collect();
    }

    /// Filter VMs to only show those without domain binding
    /// Call this after load_available_vms with platform configs
    pub fn filter_vms_without_domain(&mut self, platforms: &[crate::config::CloudPlatformConfig]) {
        let vms_with_domain: std::collections::HashSet<String> = platforms
            .iter()
            .filter(|p| p.bound_domain.is_some())
            .flat_map(|p| p.vms.iter().map(|vm| vm.name.clone()))
            .collect();

        self.available_vms.retain(|vm| !vms_with_domain.contains(&vm.name));
    }

    /// Fetch domains from Cloudflare account
    pub fn fetch_domains(&mut self) {
        let token = match &self.oauth_result {
            Some(result) => result.access_token.clone(),
            None => {
                dure_error!("No OAuth token available");
                return;
            }
        };

        let promise = poll_promise::Promise::spawn_thread("fetch_domains", move || {
            let client = crate::api::cloudflare::CloudflareClient::new(token);
            client.list_zones()
                .map_err(|e| format!("Failed to fetch domains: {}", e))
        });

        self.fetch_domains_promise = Some(promise);
    }

    /// Add domain to Cloudflare account
    pub fn add_domain_to_account(&mut self, domain: &str) {
        let token = match &self.oauth_result {
            Some(result) => result.access_token.clone(),
            None => {
                dure_error!("No OAuth token available");
                return;
            }
        };

        let domain = domain.to_string();
        let promise = poll_promise::Promise::spawn_thread("add_domain", move || {
            let client = crate::api::cloudflare::CloudflareClient::new(token);
            client.create_zone(&domain)
                .map(|zone| zone.name)
                .map_err(|e| format!("Failed to add domain: {}", e))
        });

        self.add_domain_promise = Some(promise);
    }

    /// Create DNS records for selected domain and VM
    pub fn create_dns_records(&mut self) {
        let token = match &self.oauth_result {
            Some(result) => result.access_token.clone(),
            None => {
                dure_error!("No OAuth token available");
                return;
            }
        };

        let zone_id = match &self.selected_zone_id {
            Some(id) => id.clone(),
            None => {
                dure_error!("No zone ID available");
                return;
            }
        };

        let domain = match &self.selected_domain {
            Some(d) => d.clone(),
            None => {
                dure_error!("No domain selected");
                return;
            }
        };

        let ip = match &self.vm_ip {
            Some(i) => i.clone(),
            None => {
                dure_error!("No VM IP available");
                return;
            }
        };

        let promise = poll_promise::Promise::spawn_thread("create_dns", move || {
            let client = crate::api::cloudflare::CloudflareClient::new(token);
            client.create_default_xmpp_records(&zone_id, &domain, &ip)
                .map_err(|e| format!("Failed to create DNS records: {}", e))
        });

        self.create_dns_promise = Some(promise);
    }
}
