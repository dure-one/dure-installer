//! DNS Wizard Dialog
//!
//! 4-step wizard for automated DNS setup:
//! 1. Cloudflare OAuth + API connection check
//! 2. Domain registration/connection + NS verification
//! 3. GCP VM selection + NS settings configuration
//! 4. Confirmation and summary

use crate::{dure_debug, dure_error, dure_info, dure_warn};
use eframe::egui;
use egui_i18n::tr;
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

    /// Promise for verifying DNS records
    #[cfg_attr(feature = "serde", serde(skip))]
    verify_dns_promise: Option<Promise<Result<bool, String>>>,

    /// DNS verification result (true if records match desired state)
    dns_verified: Option<bool>,

    /// DNS operation log (separate from general progress_log)
    #[cfg_attr(feature = "serde", serde(skip))]
    dns_operation_log: Vec<String>,

    /// Selected domain for DNS setup
    selected_domain: Option<String>,

    /// Selected zone ID (from selected_domain)
    selected_zone_id: Option<String>,

    /// Domain search query for browser link
    search_query: String,

    /// Step 3: VM configuration
    selected_vm: Option<String>,
    vm_ip: Option<String>,
    ns_records_configured: bool,

    /// Available platforms with VMs (from platform tab)
    #[cfg_attr(feature = "serde", serde(skip))]
    available_platforms: Vec<PlatformVmInfo>,

    /// Step 4: Configuration summary
    summary_items: Vec<String>,

    /// Progress log
    #[cfg_attr(feature = "serde", serde(skip))]
    progress_log: Vec<String>,

    /// Should close wizard
    pub should_close: bool,

    /// Completed domain setup - used by DNS tab to add row
    /// (provider, domain, zone_id, account_id, access_token, refresh_token, expires_at)
    pub completed_domain: Option<(
        String,           // provider
        String,           // domain
        Option<String>,   // zone_id
        Option<String>,   // account_id
        Option<String>,   // oauth_access_token
        Option<String>,   // oauth_refresh_token
        Option<u64>,      // oauth_token_expiry
    )>,
}

/// Platform with VM information for selection
#[derive(Clone, Debug)]
pub struct PlatformVmInfo {
    pub project_id: String,
    pub platform_name: String,
    pub vm_name: String,
    pub vm_external_ip: String,
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
            verify_dns_promise: None,
            dns_verified: None,
            dns_operation_log: Vec::new(),
            selected_domain: None,
            selected_zone_id: None,
            search_query: String::new(),
            selected_vm: None,
            vm_ip: None,
            ns_records_configured: false,
            available_platforms: Vec::new(),
            summary_items: Vec::new(),
            progress_log: Vec::new(),
            should_close: false,
            completed_domain: None,
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
            ui.add_space(4.0);
            // Progress indicator (always visible at top)
            self.render_progress_indicator(ui);

            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Current step content
            match &self.state {
                DnsWizardState::CloudflareAuth => self.render_step1_oauth(ui),
                DnsWizardState::DomainRegistration => self.render_step2_domain(ui),
                DnsWizardState::VmConfiguration => self.render_step3_vm(ui),
                DnsWizardState::Confirmation => self.render_step4_confirmation(ui),
                DnsWizardState::Complete => self.render_complete(ui),
                DnsWizardState::Error(err) => self.render_error(ui, err),
            }
        });
    }

    /// Render progress indicator
    fn render_progress_indicator(&mut self, ui: &mut egui::Ui) {
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
                    // Current step: filled small button (not clickable)
                    ui.add(MaterialButton::filled(*label).small());
                } else if is_complete {
                    // Completed step: outlined small button (clickable to jump back)
                    if ui.add(MaterialButton::outlined(*label).small()).clicked() {
                        self.state = step_state.clone();
                    }
                } else {
                    // Future step: disabled outlined small button
                    ui.add(MaterialButton::outlined(*label).small().enabled(false));
                }

                if i < steps.len() - 1 {
                    ui.add_space(5.0);
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
        ui.heading(tr!("dns-wizard-step1-title"));
        ui.add_space(4.0);

        ui.label(tr!("dns-wizard-step1-desc"));

        ui.add_space(4.0);

        // Check OAuth promise status
        if let Some(promise) = self.oauth_promise.take() {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(oauth_result) => {
                        self.oauth_result = Some(oauth_result.clone());
                        self.progress_log.push("OAuth completed successfully".to_string());
                        self.progress_log.push("Verifying API token...".to_string());
                        ui.ctx().request_repaint();

                        // Verify API token
                        self.verify_api_token();

                        // Auto-fetch Account ID from API v4
                        self.fetch_account_id();
                    }
                    Err(err) => {
                        // Enhanced error message with troubleshooting hints
                        let error_msg = if err.contains("OAuth authorization failed") {
                            format!(
                                "❌ {}\n\n\
                                 Troubleshooting:\n\
                                 1. Check Cloudflare OAuth app redirect URI: http://localhost:51082/cf_callback\n\
                                 2. Verify all required scopes are granted\n\
                                 3. Try the OAuth flow again",
                                err
                            )
                        } else if err.contains("Failed to bind to port") {
                            format!(
                                "❌ {}\n\n\
                                 Port 51082 is already in use.\n\
                                 Close any other applications using this port and try again.",
                                err
                            )
                        } else {
                            format!("❌ OAuth failed: {}", err)
                        };

                        self.progress_log.push(error_msg.clone());
                        ui.ctx().request_repaint();
                        self.state = DnsWizardState::Error(error_msg);
                    }
                }
            } else {
                ui.label(tr!("dns-wizard-progress-loading"));
                ui.label(tr!("dns-wizard-step1-oauth-complete"));
                ui.ctx().request_repaint();
                // Put promise back if not ready
                self.oauth_promise = Some(promise);
            }
        } else if self.oauth_result.is_none() {
            // Not started OAuth yet
            if ui
                .add(MaterialButton::filled(tr!("dns-wizard-oauth-button")))
                .on_hover_text("Opens browser for OAuth authentication")
                .clicked()
            {
                self.start_oauth();
            }

            ui.add_space(4.0);
            ui.label("You will be redirected to Cloudflare to authorize access.");
        } else {
            // OAuth complete
            ui.label(format!(
                "Connected: {}",
                self.connected_email.as_ref().unwrap_or(&"Account connected".to_string())
            ));

            if self.api_connection_ok {
                ui.label(tr!("dns-wizard-step1-account-verified"));
            } else {
                ui.label(tr!("dns-wizard-step1-account-verifying"));
            }

            ui.add_space(4.0);

            // Account ID input
            ui.horizontal(|ui| {
                ui.label(tr!("dns-wizard-step1-account-id"));
                ui.text_edit_singleline(&mut self.cloudflare_account_id);
            });

            if self.cloudflare_account_id.is_empty() {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 193, 7), // Warning amber
                    "Warning: Account ID required to continue"
                );
            }
        }
        
        ui.add_space(4.0);
        
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            // Bottom padding (rendered first = appears at bottom)
            ui.add_space(30.0);

            // Buttons at bottom (rendered first in bottom_up layout)
            ui.horizontal(|ui| {
                if ui.add(MaterialButton::outlined(&tr!("dns-wizard-button-close"))).clicked() {
                    self.should_close = true;
                }

                ui.add_space(4.0);

                // Show "Open OAuth Screen" button if OAuth was attempted
                if self.oauth_promise.is_some() || self.oauth_result.is_some() {
                    if ui.add(MaterialButton::outlined(&tr!("dns-wizard-button-oauth"))).clicked() {
                        self.start_oauth();
                    }
                    ui.add_space(4.0);
                }

                let next_btn = ui.add_enabled(
                    self.can_proceed_from_step1(),
                    MaterialButton::filled(&tr!("dns-wizard-button-next"))
                );
                if next_btn.clicked() {
                    self.state = DnsWizardState::DomainRegistration;
                }
            });

            // Content area (reversed order in bottom_up layout)
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.add_space(170.0);
                // Progress log (always visible, full-width, attached to bottom)
                // ui.set_width(ui.available_width());
                egui::ScrollArea::vertical()
                    .max_height(60.0)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        if self.progress_log.is_empty() {
                            ui.label(tr!("dns-wizard-waiting"));
                        } else {
                            for log in &self.progress_log {
                                ui.label(log);
                            }
                        }
                    });
                ui.add_space(4.0);
            });
        });
    }

    /// Step 2: Domain registration
    fn render_step2_domain(&mut self, ui: &mut egui::Ui) {
        // Auto-fetch domains on first render
        if self.domains_list.is_empty() && self.fetch_domains_promise.is_none() {
            self.fetch_domains();
        }
        
        // ui starts
        ui.heading("Step 2: Domain Selection");
        ui.add_space(4.0);

        // Handle fetch domains promise
        if let Some(promise) = self.fetch_domains_promise.take() {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(zones) => {
                        let count = zones.len();
                        self.domains_list = zones.clone();
                        self.progress_log.push(format!("Loaded {} domains", count));
                        ui.ctx().request_repaint();
                    }
                    Err(err) => {
                        self.progress_log.push(format!("Error: Failed to fetch domains: {}", err));
                        ui.ctx().request_repaint();
                    }
                }
            } else {
                ui.spinner();
                ui.label("Loading domains...");
                ui.ctx().request_repaint();
                self.fetch_domains_promise = Some(promise);
            }
        }

        // Handle add domain promise
        if let Some(promise) = self.add_domain_promise.take() {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(domain) => {
                        self.progress_log.push(format!("Added domain: {}", domain));
                        ui.ctx().request_repaint();
                        self.fetch_domains(); // Refresh list
                        self.domain_name.clear();
                    }
                    Err(err) => {
                        self.progress_log.push(format!("Error: Failed to add domain: {}", err));
                        ui.ctx().request_repaint();
                    }
                }
            } else {
                ui.spinner();
                ui.label("Adding domain...");
                ui.ctx().request_repaint();
                self.add_domain_promise = Some(promise);
            }
        }

        // I have domain section
        ui.heading("I have domain");
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Domain:");
            ui.text_edit_singleline(&mut self.domain_name);

            ui.add_space(4.0);
            let can_add = !self.domain_name.is_empty() && self.add_domain_promise.is_none();
            if ui.add_enabled(can_add, MaterialButton::filled("Add").small()).clicked() {
                self.add_domain_to_account(&self.domain_name.clone());
            }
        });
        ui.add_space(4.0);
        ui.separator();
        ui.add_space(4.0);

        // I want to register domain section
        ui.heading("I want to register domain");
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Domain:");
            ui.text_edit_singleline(&mut self.search_query);

            if ui.add(MaterialButton::filled("Search").small()).clicked() {
                if !self.search_query.is_empty() {
                    let url = format!(
                        "https://dash.cloudflare.com/{}/domains/registrations/purchase?query={}",
                        self.cloudflare_account_id,
                        urlencoding::encode(&self.search_query)
                    );
                    if let Err(e) = webbrowser::open(&url) {
                        self.progress_log.push(format!("Error: Failed to open browser: {}", e));
                    } else {
                        self.progress_log.push("Opened Cloudflare dashboard".to_string());
                    }
                }
            }
        });

        ui.add_space(8.0);
        ui.separator();
        ui.add_space(4.0);

        // Domain dropdown from Cloudflare account
        ui.horizontal(|ui| {
            ui.label("Select domain to use:");

            ui.add_space(8.0);

            if ui.add(MaterialButton::outlined(&tr!("refresh")).small()).clicked() {
                self.fetch_domains();
                dure_info!("Refreshing domain list from Cloudflare");
            }
        });
        ui.add_space(4.0);

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
        
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            // Bottom padding (rendered first = appears at bottom)
            ui.add_space(30.0);

            // Buttons at bottom (rendered first in bottom_up layout)
            ui.horizontal(|ui| {
                if ui.add(MaterialButton::outlined(&tr!("dns-wizard-button-close"))).clicked() {
                    self.should_close = true;
                }

                ui.add_space(4.0);

                if ui.add(MaterialButton::outlined(&tr!("dns-wizard-button-previous"))).clicked() {
                    self.state = DnsWizardState::CloudflareAuth;
                }

                ui.add_space(4.0);

                let next_btn = ui.add_enabled(
                    self.can_proceed_from_step2(),
                    MaterialButton::filled(&tr!("dns-wizard-button-next"))
                );
                if next_btn.clicked() {
                    self.state = DnsWizardState::VmConfiguration;
                }
            });

            // Content area (reversed order in bottom_up layout)
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.add_space(55.0);
                
                // Progress log (full-width, no border)
                ui.set_width(ui.available_width());
                egui::ScrollArea::vertical()
                    .max_height(60.0)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        if self.progress_log.is_empty() {
                            ui.label(tr!("dns-wizard-waiting"));
                        } else {
                            for log in self.progress_log.iter().rev().take(5).rev() {
                                ui.label(log);
                            }
                        }
                    }); 
            });
        });
    }

    /// Step 3: VM configuration
    fn render_step3_vm(&mut self, ui: &mut egui::Ui) {
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            // Bottom padding (rendered first = appears at bottom)
            ui.add_space(30.0);

            // Buttons at bottom (rendered first in bottom_up layout)
            ui.horizontal(|ui| {
                if ui.add(MaterialButton::outlined(&tr!("dns-wizard-button-close"))).clicked() {
                    self.should_close = true;
                }

                ui.add_space(4.0);

                if ui.add(MaterialButton::outlined(&tr!("dns-wizard-button-previous"))).clicked() {
                    self.state = DnsWizardState::DomainRegistration;
                }

                ui.add_space(4.0);

                let next_btn = ui.add_enabled(
                    self.can_proceed_from_step3(),
                    MaterialButton::filled(&tr!("dns-wizard-button-next"))
                );
                if next_btn.clicked() {
                    self.state = DnsWizardState::Confirmation;
                }
            });

            ui.add_space(4.0);

            // Content (reversed in bottom_up)
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.heading("Step 3: VM Selection");
                ui.add_space(4.0);

                ui.label("Select VM to bind with DNS:");
                ui.add_space(4.0);

                // Platform list with dense styling
                if self.available_platforms.is_empty() {
                    ui.label("No platforms available. Please add a platform in Platform tab first.");
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for platform in &self.available_platforms.clone() {
                                let is_selected = self.selected_vm.as_ref() == Some(&platform.project_id);

                                use egui_material3::theme::get_global_color;
                                let bg_color = if is_selected {
                                    get_global_color("primaryContainer")
                                } else {
                                    get_global_color("surface")
                                };

                                let frame = egui::Frame::default()
                                    .fill(bg_color)
                                    .inner_margin(egui::Margin::symmetric(12, 8))
                                    .rounding(4.0);

                                let response = frame.show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.add_space(4.0);
                                        ui.label("☁");
                                        ui.add_space(8.0);
                                        ui.vertical(|ui| {
                                            ui.label(
                                                egui::RichText::new(&platform.platform_name)
                                                    .strong()
                                                    .color(get_global_color("onSurface"))
                                            );
                                            ui.label(
                                                egui::RichText::new(format!(
                                                    "VM: {} | IP: {}",
                                                    platform.vm_name,
                                                    platform.vm_external_ip
                                                ))
                                                .size(12.0)
                                                .color(get_global_color("onSurfaceVariant"))
                                            );
                                        });
                                    });
                                }).response;

                                if response.interact(egui::Sense::click()).clicked() {
                                    self.selected_vm = Some(platform.project_id.clone());
                                    self.vm_ip = Some(platform.vm_external_ip.clone());
                                    dure_info!("Selected VM: {} ({})", platform.platform_name, platform.project_id);
                                }

                                ui.add_space(2.0);
                            }
                        });
                }
            });
        });
    }

    /// Step 4: DNS Setup
    fn render_step4_confirmation(&mut self, ui: &mut egui::Ui) {
        // Auto-verify DNS on first render of this step
        if self.verify_dns_promise.is_none() && self.dns_verified.is_none() {
            self.verify_dns_records();
        }

        // Handle verify DNS promise (early return for spinner)
        if let Some(promise) = self.verify_dns_promise.take() {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(verified) => {
                        self.dns_verified = Some(*verified);
                        if *verified {
                            self.dns_operation_log.push("DNS records verified - all settings correct".to_string());
                            self.ns_records_configured = true;
                        } else {
                            self.dns_operation_log.push("DNS records missing or incorrect - setup required".to_string());
                        }
                        ui.ctx().request_repaint();
                    }
                    Err(err) => {
                        self.dns_operation_log.push(format!("Error: Verification failed: {}", err));
                        ui.ctx().request_repaint();
                    }
                }
            } else {
                ui.spinner();
                ui.label("Verifying DNS records...");
                ui.ctx().request_repaint();
                self.verify_dns_promise = Some(promise);
                return;
            }
        }

        // Handle create DNS records promise (early return for spinner)
        if let Some(promise) = self.create_dns_promise.take() {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(()) => {
                        self.ns_records_configured = true;
                        self.dns_verified = Some(true);
                        self.dns_operation_log.push("DNS records created or updated successfully".to_string());
                        ui.ctx().request_repaint();
                    }
                    Err(err) => {
                        if err.contains("already exists") || err.contains("duplicate") {
                            self.ns_records_configured = true;
                            self.dns_verified = Some(true);
                            self.dns_operation_log.push("DNS records already exist (no changes needed)".to_string());
                        } else {
                            self.dns_operation_log.push(format!("Error: Failed to setup DNS records: {}", err));
                        }
                        ui.ctx().request_repaint();
                    }
                }
            } else {
                ui.spinner();
                ui.label("Setting up DNS records...");
                ui.ctx().request_repaint();
                self.create_dns_promise = Some(promise);
                return;
            }
        }
        
        // ui starts
        ui.heading("Step 4: DNS Setup");
        ui.add_space(4.0);

        ui.label("Status of DNS settings:");
        ui.add_space(4.0);

        let dns_needs_setup = self.dns_verified != Some(true);
        let can_setup = dns_needs_setup && self.create_dns_promise.is_none();

        let setup_btn = ui.add_enabled(
            can_setup,
            MaterialButton::filled("Setup DNS").small()
        );
        if setup_btn.clicked() {
            self.dns_operation_log.push("Starting DNS setup...".to_string());
            self.create_dns_records();
        }

        ui.add_space(4.0);
        ui.label("setup dns will overwrite to next values for your domain.");
        ui.add_space(4.0);

        ui.group(|ui| {
            ui.label("Default settings:");
            ui.add_space(4.0);

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
        
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            // Bottom padding (rendered first = appears at bottom)
            ui.add_space(30.0);

            // Buttons at bottom (rendered first in bottom_up layout)
            ui.horizontal(|ui| {
                if ui.add(MaterialButton::outlined(&tr!("dns-wizard-button-close"))).clicked() {
                    self.should_close = true;
                }

                ui.add_space(4.0);

                if ui.add(MaterialButton::outlined(&tr!("dns-wizard-button-previous"))).clicked() {
                    self.state = DnsWizardState::VmConfiguration;
                }

                ui.add_space(4.0);

                let add_btn = ui.add_enabled(
                    self.ns_records_configured,
                    MaterialButton::filled("Add")
                );
                if add_btn.clicked() {
                    if let Some(domain) = &self.selected_domain {
                        // Extract OAuth tokens from oauth_result
                        let (access_token, refresh_token, expires_at) = self.oauth_result
                            .as_ref()
                            .map(|r| (
                                Some(r.access_token.clone()),
                                r.refresh_token.clone(),
                                Some(r.expires_at),
                            ))
                            .unwrap_or((None, None, None));

                        self.completed_domain = Some((
                            "cloudflare".to_string(),
                            domain.clone(),
                            self.selected_zone_id.clone(),
                            Some(self.cloudflare_account_id.clone()),
                            access_token,
                            refresh_token,
                            expires_at,
                        ));
                    }
                    self.should_close = true;
                }
            });

            // Content area (reversed order in bottom_up layout)
            ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                ui.add_space(55.0);
                
                // DNS operation log (always visible, full-width, attached to bottom)
                ui.set_width(ui.available_width());
                egui::ScrollArea::vertical()
                    .max_height(60.0)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        if self.dns_operation_log.is_empty() {
                            ui.label("Waiting for DNS operations...");
                        } else {
                            for log in self.dns_operation_log.iter().rev().take(10).rev() {
                                ui.label(log);
                            }
                        }
                    });
            });
        });
    }

    /// Render complete state
    fn render_complete(&self, ui: &mut egui::Ui) {
        ui.heading("Setup Complete");
        ui.add_space(4.0);
        ui.label("Your DNS has been configured successfully!");
    }

    /// Render error state
    fn render_error(&self, ui: &mut egui::Ui, error: &str) {
        ui.heading("Error");
        ui.add_space(4.0);
        ui.colored_label(egui_material3::theme::get_global_color("error"), error);
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
            dure_info!("Fetching Cloudflare account info");

            match get_user_info(&oauth_result.access_token) {
                Ok(user_info) => {
                    self.cloudflare_account_id = user_info.account_id.clone();
                    self.progress_log.push(format!(
                        "✓ Account: {} ({})",
                        user_info.account_name,
                        user_info.account_id
                    ));
                    dure_info!("Auto-populated Account ID: {} ({})",
                        user_info.account_id, user_info.account_name);
                }
                Err(e) => {
                    self.progress_log.push(format!("⚠ Could not auto-fetch account: {}", e));
                    dure_warn!("Failed to auto-fetch account ID: {}", e);
                }
            }
        }
    }

    /// Verify API token
    fn verify_api_token(&mut self) {
        if let Some(oauth_result) = &self.oauth_result {
            use crate::api::cloudflare::oauth::verify_token;

            dure_info!("Verifying Cloudflare API token");

            match verify_token(&oauth_result.access_token) {
                Ok(valid) => {
                    self.api_connection_ok = valid;
                    if valid {
                        self.progress_log.push("✓ API token verified successfully".to_string());
                        dure_info!("API token verified successfully");
                    } else {
                        self.progress_log.push("✗ Invalid API token".to_string());
                        dure_warn!("Invalid API token");
                    }
                }
                Err(e) => {
                    self.progress_log
                        .push(format!("✗ Token verification failed: {}", e));
                    dure_error!("Token verification error: {}", e);
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
        // ponytail: Stub NS check - returns true for now
        // Real implementation: use hickory-dns or trust-dns-resolver to query NS records
        // Upgrade when: NS misconfig becomes actual issue (domain pointing elsewhere)
        self.progress_log.push("Checking nameservers...".to_string());
        self.ns_check_ok = true;
        self.progress_log.push("Nameservers verified (stub)".to_string());
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

    /// Load available platforms with VMs
    pub fn load_available_platforms(&mut self, platforms: Vec<PlatformVmInfo>) {
        self.available_platforms = platforms;
    }

    /// Check if platforms need to be loaded
    pub fn needs_platform_data(&self) -> bool {
        self.available_platforms.is_empty()
    }

    /// Fetch domains from Cloudflare account
    pub fn fetch_domains(&mut self) {
        dure_info!("Fetching domains from Cloudflare account");

        let token = match &self.oauth_result {
            Some(result) => result.access_token.clone(),
            None => {
                dure_error!("No OAuth token available for fetching domains");
                return;
            }
        };

        let promise = poll_promise::Promise::spawn_thread("fetch_domains", move || {
            dure_debug!("Starting fetch_domains thread");
            let client = crate::api::cloudflare::CloudflareClient::new(token);
            let result = client.list_zones()
                .map_err(|e| format!("Failed to fetch domains: {}", e));

            match &result {
                Ok(zones) => dure_info!("Fetched {} domains successfully", zones.len()),
                Err(e) => dure_error!("Failed to fetch domains: {}", e),
            }

            result
        });

        self.fetch_domains_promise = Some(promise);
    }

    /// Add domain to Cloudflare account
    pub fn add_domain_to_account(&mut self, domain: &str) {
        dure_info!("Adding domain to Cloudflare account: {}", domain);

        let token = match &self.oauth_result {
            Some(result) => result.access_token.clone(),
            None => {
                dure_error!("No OAuth token available for adding domain");
                return;
            }
        };

        let domain = domain.to_string();
        let promise = poll_promise::Promise::spawn_thread("add_domain", move || {
            dure_debug!("Starting add_domain thread for: {}", domain);
            let client = crate::api::cloudflare::CloudflareClient::new(token);
            let result = client.create_zone(&domain)
                .map(|zone| zone.name)
                .map_err(|e| format!("Failed to add domain: {}", e));

            match &result {
                Ok(name) => dure_info!("Domain added successfully: {}", name),
                Err(e) => dure_error!("Failed to add domain: {}", e),
            }

            result
        });

        self.add_domain_promise = Some(promise);
    }

    /// Create DNS records for selected domain and VM
    pub fn create_dns_records(&mut self) {
        dure_info!("Starting DNS records creation");

        let token = match &self.oauth_result {
            Some(result) => result.access_token.clone(),
            None => {
                dure_error!("No OAuth token available for DNS creation");
                return;
            }
        };

        let zone_id = match &self.selected_zone_id {
            Some(id) => id.clone(),
            None => {
                dure_error!("No zone ID available for DNS creation");
                return;
            }
        };

        let domain = match &self.selected_domain {
            Some(d) => d.clone(),
            None => {
                dure_error!("No domain selected for DNS creation");
                return;
            }
        };

        let ip = match &self.vm_ip {
            Some(i) => i.clone(),
            None => {
                dure_error!("No VM IP available for DNS creation");
                return;
            }
        };

        dure_info!("Creating DNS records - Domain: {}, Zone ID: {}, VM IP: {}", domain, zone_id, ip);

        let promise = poll_promise::Promise::spawn_thread("create_dns", move || {
            dure_debug!("Starting create_dns thread");
            let client = crate::api::cloudflare::CloudflareClient::new(token);
            let result = client.create_default_xmpp_records(&zone_id, &domain, &ip)
                .map_err(|e| format!("Failed to create DNS records: {}", e));

            match &result {
                Ok(_) => dure_info!("DNS records created/updated successfully"),
                Err(e) => dure_error!("Failed to create DNS records: {}", e),
            }

            result
        });

        self.create_dns_promise = Some(promise);
    }

    /// Verify existing DNS records match desired configuration
    pub fn verify_dns_records(&mut self) {
        dure_info!("Starting DNS records verification");

        let token = match &self.oauth_result {
            Some(result) => result.access_token.clone(),
            None => {
                dure_error!("No OAuth token available for DNS verification");
                return;
            }
        };

        let zone_id = match &self.selected_zone_id {
            Some(id) => id.clone(),
            None => {
                dure_error!("No zone ID available for DNS verification");
                return;
            }
        };

        let domain = match &self.selected_domain {
            Some(d) => d.clone(),
            None => {
                dure_error!("No domain selected for DNS verification");
                return;
            }
        };

        let ip = match &self.vm_ip {
            Some(i) => i.clone(),
            None => {
                dure_error!("No VM IP available for DNS verification");
                return;
            }
        };

        dure_info!("Verifying DNS records - Domain: {}, Zone ID: {}, VM IP: {}", domain, zone_id, ip);

        let promise = poll_promise::Promise::spawn_thread("verify_dns", move || {
            dure_debug!("Starting verify_dns thread");
            let client = crate::api::cloudflare::CloudflareClient::new(token);
            let result = client.verify_default_xmpp_records(&zone_id, &domain, &ip)
                .map_err(|e| format!("Failed to verify DNS records: {}", e));

            match &result {
                Ok(verified) => {
                    if *verified {
                        dure_info!("DNS records verification passed - all records correct");
                    } else {
                        dure_warn!("DNS records verification failed - records missing or incorrect");
                    }
                }
                Err(e) => dure_error!("DNS verification error: {}", e),
            }

            result
        });

        self.verify_dns_promise = Some(promise);
    }

    /// Get selected platform/VM info for DNS tab binding
    pub fn get_selected_platform_info(&self) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
        // Find the selected platform from available_platforms based on selected_vm (which stores project_id)
        if let Some(ref project_id) = self.selected_vm {
            if let Some(platform) = self.available_platforms.iter().find(|p| &p.project_id == project_id) {
                return (
                    Some(platform.project_id.clone()),
                    Some(platform.platform_name.clone()),
                    Some(platform.vm_name.clone()),
                    self.vm_ip.clone(),
                );
            }
        }
        (None, None, None, None)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_selected_platform_info_with_valid_selection() {
        let mut wizard = DnsWizard::new();

        // Add test platforms
        wizard.available_platforms = vec![
            PlatformVmInfo {
                project_id: "project-123".to_string(),
                platform_name: "Test Platform".to_string(),
                vm_name: "test-vm".to_string(),
                vm_external_ip: "1.2.3.4".to_string(),
            },
            PlatformVmInfo {
                project_id: "project-456".to_string(),
                platform_name: "Other Platform".to_string(),
                vm_name: "other-vm".to_string(),
                vm_external_ip: "5.6.7.8".to_string(),
            },
        ];

        // Select first platform (selected_vm stores project_id)
        wizard.selected_vm = Some("project-123".to_string());

        let (platform_id, platform_name, vm_name, vm_ip) = wizard.get_selected_platform_info();

        assert_eq!(platform_id, Some("project-123".to_string()));
        assert_eq!(platform_name, Some("Test Platform".to_string()));
        assert_eq!(vm_name, Some("test-vm".to_string()));
        assert_eq!(vm_ip, None); // vm_ip is separate in wizard
    }

    #[test]
    fn test_get_selected_platform_info_no_selection() {
        let wizard = DnsWizard::new();

        let (platform_id, platform_name, vm_name, vm_ip) = wizard.get_selected_platform_info();

        assert_eq!(platform_id, None);
        assert_eq!(platform_name, None);
        assert_eq!(vm_name, None);
        assert_eq!(vm_ip, None);
    }

    #[test]
    fn test_get_selected_platform_info_invalid_selection() {
        let mut wizard = DnsWizard::new();

        wizard.available_platforms = vec![PlatformVmInfo {
            project_id: "project-123".to_string(),
            platform_name: "Test Platform".to_string(),
            vm_name: "test-vm".to_string(),
            vm_external_ip: "1.2.3.4".to_string(),
        }];

        // Select a non-existent platform
        wizard.selected_vm = Some("project-999".to_string());

        let (platform_id, platform_name, vm_name, vm_ip) = wizard.get_selected_platform_info();

        assert_eq!(platform_id, None);
        assert_eq!(platform_name, None);
        assert_eq!(vm_name, None);
        assert_eq!(vm_ip, None);
    }
}
