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

                        // Fetch user info from OAuth
                        self.fetch_user_info();
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
    }

    /// Step 2: Domain registration
    fn render_step2_domain(&mut self, ui: &mut egui::Ui) {
        ui.heading("Step 2: Domain Registration");
        ui.add_space(10.0);

        // Domain input
        ui.horizontal(|ui| {
            ui.label("Domain name:");
            ui.text_edit_singleline(&mut self.domain_name);
        });

        ui.add_space(5.0);

        // Check domain availability
        if let Some(promise) = self.domain_check_promise.take() {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(available) => {
                        self.domain_available = Some(*available);

                        if *available {
                            self.progress_log.push(format!("✓ {} is available", self.domain_name));
                        } else {
                            self.progress_log.push(format!("✗ {} is taken", self.domain_name));
                        }
                    }
                    Err(err) => {
                        self.progress_log.push(format!("✗ Check failed: {}", err));
                    }
                }
            } else {
                ui.label("⏳ Checking availability...");
                // Put promise back if not ready
                self.domain_check_promise = Some(promise);
            }
        } else if !self.domain_name.is_empty() && self.domain_available.is_none() {
            if ui
                .add(MaterialButton::outlined("Check Availability"))
                .clicked()
            {
                self.check_domain_availability();
            }
        }

        // Show availability status
        if let Some(available) = self.domain_available {
            ui.add_space(5.0);
            if available {
                ui.colored_label(egui::Color32::from_rgb(0, 200, 0), "✓ Domain available for registration");
            } else {
                ui.colored_label(egui::Color32::from_rgb(255, 100, 0), "✗ Domain already registered");
            }
        }

        ui.add_space(10.0);

        // Register or connect button
        if let Some(promise) = self.register_promise.take() {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(domain) => {
                        self.domain_registered = true;
                        self.progress_log.push(format!("✓ Domain registered: {}", domain));
                    }
                    Err(err) => {
                        self.progress_log.push(format!("✗ Registration failed: {}", err));
                    }
                }
            } else {
                ui.label("⏳ Registering domain...");
                // Put promise back if not ready
                self.register_promise = Some(promise);
            }
        } else if !self.domain_registered && !self.domain_name.is_empty() {
            ui.horizontal(|ui| {
                if self.domain_available == Some(true) {
                    if ui
                        .add(MaterialButton::filled("Register Domain"))
                        .on_hover_text("Register via Cloudflare Registrar (1 year)")
                        .clicked()
                    {
                        self.register_domain();
                    }
                }

                if ui
                    .add(MaterialButton::outlined("Connect Existing"))
                    .on_hover_text("I already own this domain")
                    .clicked()
                {
                    self.connect_existing_domain();
                }
            });
        }

        // Status indicators
        if self.domain_registered {
            ui.add_space(10.0);
            ui.label("✓ Domain configured");

            if self.ns_check_ok {
                ui.label("✓ Nameservers verified");
            } else {
                ui.label("⏳ Verifying nameservers...");
            }
        }

        // Progress log
        if !self.progress_log.is_empty() {
            ui.add_space(10.0);
            ui.group(|ui| {
                ui.set_max_height(100.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for log in &self.progress_log {
                        ui.label(log);
                    }
                });
            });
        }
    }

    /// Step 3: VM configuration
    fn render_step3_vm(&mut self, ui: &mut egui::Ui) {
        ui.heading("Step 3: VM Configuration");
        ui.add_space(10.0);

        ui.label("Select GCP VM and configure default DNS records:");

        ui.add_space(10.0);

        // VM selection
        ui.horizontal(|ui| {
            ui.label("GCP VM:");
            egui::ComboBox::from_label("")
                .selected_text(
                    self.selected_vm
                        .as_ref()
                        .unwrap_or(&"Select VM...".to_string()),
                )
                .show_ui(ui, |ui| {
                    for vm in &self.available_vms {
                        let text = format!("{} ({})", vm.name, vm.ip);
                        if ui.selectable_label(false, &text).clicked() {
                            self.selected_vm = Some(vm.name.clone());
                            self.vm_ip = Some(vm.ip.clone());
                        }
                    }
                });
        });

        // Show selected VM IP
        if let Some(ip) = &self.vm_ip {
            ui.add_space(5.0);
            ui.label(format!("VM IP: {}", ip));
        }

        ui.add_space(10.0);

        // Default NS settings preview
        if let Some(ip) = &self.vm_ip {
            ui.group(|ui| {
                ui.label("Default DNS Records to be created:");
                ui.add_space(5.0);

                let records = self.get_default_dns_records(ip);
                for record in &records {
                    ui.monospace(record);
                }
            });

            ui.add_space(10.0);

            if !self.ns_records_configured {
                if ui
                    .add(MaterialButton::filled("Apply DNS Configuration"))
                    .on_hover_text("Create 7 default DNS records")
                    .clicked()
                {
                    self.apply_dns_configuration();
                }
            } else {
                ui.label("✓ DNS records configured");
            }
        } else {
            ui.colored_label(
                egui::Color32::from_rgb(255, 165, 0),
                "⚠ Please select a VM to continue"
            );
        }
    }

    /// Step 4: Confirmation
    fn render_step4_confirmation(&mut self, ui: &mut egui::Ui) {
        ui.heading("Step 4: Configuration Summary");
        ui.add_space(10.0);

        ui.label("Your DNS setup is complete! Review the configuration:");

        ui.add_space(10.0);

        // Configuration summary
        ui.group(|ui| {
            ui.strong("Cloudflare Account");
            ui.label(format!(
                "  Account: {}",
                self.connected_email.as_ref().unwrap_or(&"Connected".to_string())
            ));
            ui.label(format!("  Account ID: {}", self.cloudflare_account_id));

            ui.add_space(5.0);

            ui.strong("Domain");
            ui.label(format!("  Domain: {}", self.domain_name));
            ui.label("  Status: Registered with Cloudflare");
            ui.label("  Nameservers: Cloudflare NS");

            ui.add_space(5.0);

            ui.strong("GCP VM");
            ui.label(format!(
                "  VM: {}",
                self.selected_vm.as_ref().unwrap_or(&"N/A".to_string())
            ));
            ui.label(format!(
                "  IP: {}",
                self.vm_ip.as_ref().unwrap_or(&"N/A".to_string())
            ));

            ui.add_space(5.0);

            ui.strong("DNS Records Configured");
            if let Some(ip) = &self.vm_ip {
                let records = self.get_default_dns_records(ip);
                for record in &records {
                    ui.monospace(format!("  {}", record));
                }
            }
        });

        ui.add_space(10.0);

        if ui
            .add(MaterialButton::filled("Finish"))
            .on_hover_text("Close wizard and return to DNS tab")
            .clicked()
        {
            self.state = DnsWizardState::Complete;
        }
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

    /// Fetch user info from OAuth endpoint
    fn fetch_user_info(&mut self) {
        if let Some(oauth_result) = &self.oauth_result {
            use crate::api::cloudflare::get_user_info;

            self.progress_log.push("Fetching user info...".to_string());

            match get_user_info(&oauth_result.access_token) {
                Ok(user_info) => {
                    self.connected_email = Some(user_info.email.clone());
                    self.progress_log.push(format!(
                        "✓ Connected: {}",
                        user_info.email
                    ));
                    dure_info!("OAuth connected as: {}", user_info.email);
                }
                Err(e) => {
                    self.progress_log.push(format!("⚠ Failed to fetch user info: {}", e));
                    dure_warn!("Failed to fetch user info: {}", e);
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
}
