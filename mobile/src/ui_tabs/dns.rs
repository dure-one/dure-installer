//! DNS tab - Automated DNS management with Cloudflare OAuth
//!
//! This tab provides automated DNS setup through Cloudflare OAuth workflow:
//! 1. OAuth authentication with PKCE
//! 2. Domain registration/connection via Cloudflare Registrar API
//! 3. GCP VM integration for automatic NS record configuration
//! 4. DNS verification and HTTPS connection testing

use crate::{dure_debug, dure_error, dure_info, dure_warn};
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{badge, BadgeColor, BadgeSize, MaterialButton, dialog};
use egui_material3::datatable::data_table;
use poll_promise::Promise;

use crate::calc::profile::ProfileContext;

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

/// DNS tab state
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DnsTab {
    /// Selected domain for operations
    #[cfg_attr(feature = "serde", serde(skip))]
    selected_domain: Option<String>,

    /// Domain rows for table display
    #[cfg_attr(feature = "serde", serde(skip))]
    domain_rows: Vec<DnsDomainRow>,

    /// Show "Add DNS" wizard dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_add_dns_wizard: bool,

    /// DNS wizard state
    #[cfg_attr(feature = "serde", serde(skip))]
    dns_wizard: Option<crate::ui_dlg::dns_wizard::DnsWizard>,

    /// Show NS check dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_ns_check_dialog: bool,

    /// NS check results
    #[cfg_attr(feature = "serde", serde(skip))]
    ns_check_results: Vec<String>,

    /// Show DNS settings check dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_dns_settings_dialog: bool,

    /// DNS settings check results
    #[cfg_attr(feature = "serde", serde(skip))]
    dns_settings_results: Vec<DnsRecordCheck>,

    /// Show HTTPS connection check dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_https_check_dialog: bool,

    /// HTTPS check result
    #[cfg_attr(feature = "serde", serde(skip))]
    https_check_result: Option<HttpsCheckResult>,

    /// NS check promise
    #[cfg_attr(feature = "serde", serde(skip))]
    ns_check_promise: Option<Promise<Result<(NsCheckResult, Vec<String>), (NsCheckResult, Vec<String>)>>>,

    /// DNS settings check promise
    #[cfg_attr(feature = "serde", serde(skip))]
    dns_settings_promise: Option<Promise<Result<(SettingsCheckResult, Vec<String>), (SettingsCheckResult, Vec<String>)>>>,

    /// HTTPS check promise
    #[cfg_attr(feature = "serde", serde(skip))]
    https_check_promise: Option<Promise<Result<HttpsCheckResult, String>>>,

    /// Flag to track if domain rows have been loaded from config
    #[cfg_attr(feature = "serde", serde(skip))]
    loaded: bool,
}

/// Domain row data for table
#[derive(Clone, Debug)]
pub struct DnsDomainRow {
    pub provider: String,        // "cloudflare"
    pub domain: String,          // "dure.co"
    pub status: DomainStatus,    // Configured, Pending, Error

    // Platform/VM binding
    pub platform_id: Option<String>,
    pub platform_name: Option<String>,
    pub vm_name: Option<String>,
    pub vm_external_ip: Option<String>,

    // Provider metadata
    pub account_id: Option<String>,

    // Check results
    pub last_ns_check: Option<NsCheckResult>,
    pub last_settings_check: Option<SettingsCheckResult>,
    pub last_https_check: Option<HttpsCheckResult>,

    // Operation state tracking
    pub operation_state: OperationState,

    // Drawer state
    pub drawer_state: DnsDrawerState,
}

/// Domain status
#[derive(Clone, Debug, PartialEq)]
enum DomainStatus {
    Configured,
    Pending,
    Error(String),
}

/// DNS record check result
#[derive(Clone, Debug)]
struct DnsRecordCheck {
    record_type: String,     // "A", "CNAME", "SRV"
    name: String,            // "dure.co", "www.dure.co"
    expected: String,        // Expected value
    actual: Option<String>,  // Actual value from DNS
    valid: bool,             // true if matches expected
}

/// HTTPS connection check result
#[derive(Clone, Debug)]
pub struct HttpsCheckResult {
    pub checked_at: i64,
    pub resolves_to_vm: bool,
    pub resolved_ip: Option<String>,
    pub expected_ip: Option<String>,
    pub error: Option<String>,
}

/// NS check result
#[derive(Clone, Debug)]
pub struct NsCheckResult {
    pub checked_at: i64,
    pub uses_cloudflare: bool,
    pub nameservers: Vec<String>,
    pub error: Option<String>,
}

/// DNS settings check result
#[derive(Clone, Debug)]
pub struct SettingsCheckResult {
    pub checked_at: i64,
    pub records_valid: bool,
    pub missing_records: Vec<String>,
    pub error: Option<String>,
}

/// Drawer state for DNS domain
#[derive(Clone, Debug)]
pub struct DnsDrawerState {
    pub active_tab: DnsDrawerTab,
    pub logs: Vec<String>,
    pub operations: Vec<crate::storage::models::opslog::OperationLog>,
    pub loading: bool,
}

impl Default for DnsDrawerState {
    fn default() -> Self {
        Self {
            active_tab: DnsDrawerTab::Status,
            logs: Vec::new(),
            operations: Vec::new(),
            loading: false,
        }
    }
}

impl DnsDrawerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn switch_tab(&mut self, tab: DnsDrawerTab) {
        self.active_tab = tab;
    }

    pub fn set_logs(&mut self, logs: Vec<String>) {
        self.logs = logs;
        self.loading = false;
    }

    pub fn set_operations(&mut self, ops: Vec<crate::storage::models::opslog::OperationLog>) {
        self.operations = ops;
        self.loading = false;
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }
}

/// Drawer tabs for DNS domain
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DnsDrawerTab {
    #[default]
    Status,
    Logs,
    Operations,
}

impl DnsDrawerTab {
    pub fn to_index(&self) -> usize {
        match self {
            DnsDrawerTab::Status => 0,
            DnsDrawerTab::Logs => 1,
            DnsDrawerTab::Operations => 2,
        }
    }

    pub fn from_index(index: usize) -> Self {
        match index {
            0 => DnsDrawerTab::Status,
            1 => DnsDrawerTab::Logs,
            2 => DnsDrawerTab::Operations,
            _ => DnsDrawerTab::default(),
        }
    }
}

// Tab string constants (not user-facing, only for internal identification)
const TAB_STATUS: &str = "status";
const TAB_LOGS: &str = "logs";
const TAB_OPERATIONS: &str = "operations";

impl crate::ui_components::drawer::DrawerTabTrait for DnsDrawerTab {
    fn as_str(&self) -> &'static str {
        match self {
            DnsDrawerTab::Status => TAB_STATUS,
            DnsDrawerTab::Logs => TAB_LOGS,
            DnsDrawerTab::Operations => TAB_OPERATIONS,
        }
    }

    fn all() -> &'static [Self] {
        &[
            DnsDrawerTab::Status,
            DnsDrawerTab::Logs,
            DnsDrawerTab::Operations,
        ]
    }
}

impl Default for DnsTab {
    fn default() -> Self {
        Self {
            selected_domain: None,
            domain_rows: Vec::new(),
            show_add_dns_wizard: false,
            dns_wizard: None,
            show_ns_check_dialog: false,
            ns_check_results: Vec::new(),
            show_dns_settings_dialog: false,
            dns_settings_results: Vec::new(),
            show_https_check_dialog: false,
            https_check_result: None,
            ns_check_promise: None,
            dns_settings_promise: None,
            https_check_promise: None,
            loaded: false,
        }
    }
}

impl DnsTab {
    /// Render the DNS tab
    pub fn ui(&mut self, ui: &mut egui::Ui, profile: &ProfileContext) {
        // Load DNS domains from config on first render
        if !self.loaded {
            self.load_domain_rows(profile);
        }

        ui.vertical(|ui| {
            // Header with "Add DNS" button
            //ui.add_space(10.0);

            // Domain table
            self.render_domain_table(ui, profile);

            // Check for button click events stored in ui.data (consume with remove_temp)
            let check_ns = ui.data_mut(|d| d.remove_temp::<String>(egui::Id::new("dns_check_ns")));
            let check_settings = ui.data_mut(|d| d.remove_temp::<String>(egui::Id::new("dns_check_settings")));
            let check_https = ui.data_mut(|d| d.remove_temp::<String>(egui::Id::new("dns_check_https")));
            let delete_domain = ui.data_mut(|d| d.remove_temp::<String>(egui::Id::new("dns_delete_domain")));

            if let Some(domain) = check_ns {
                self.check_ns_server(&domain);
            }
            if let Some(domain) = check_settings {
                self.check_dns_settings(&domain);
            }
            if let Some(domain) = check_https {
                self.check_https_connection(&domain);
            }
            if let Some(domain) = delete_domain {
                self.delete_domain(&domain, profile);
            }

            // Handle drawer tab switches
            for row in &mut self.domain_rows {
                if let Some((domain, new_tab)) = ui.data_mut(|d| {
                    d.remove_temp::<(String, DnsDrawerTab)>(
                        egui::Id::new("dns_drawer_tab_switch").with(&row.domain)
                    )
                }) {
                    if domain == row.domain {
                        row.drawer_state.active_tab = new_tab;
                    }
                }
            }

            // Dialogs
            self.render_dialogs(ui, profile);
        });
    }

    /// Render domain table
    fn render_domain_table(&mut self, ui: &mut egui::Ui, profile: &ProfileContext) {
        // Calculate responsive column widths
        let available_width = ui.available_width() - 40.0;
        let base_width = 740.0;
        let width_ratio = (available_width / base_width).min(1.5).max(0.8);

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.heading(tr!("dns-main-heading"));
            ui.add_space(4.0);
            ui.label(tr!("dns-main-desc"));
            ui.add_space(8.0);

            if ui
                    .add(MaterialButton::filled(tr!("dns-button-add")))
                    .on_hover_text(tr!("dns-button-add-tooltip"))
                    .clicked()
                {
                    self.show_add_dns_wizard = true;
                    let mut wizard = crate::ui_dlg::dns_wizard::DnsWizard::new(
                        profile.name.clone(),
                    );

                    // Load platforms from config
                    let platforms = Self::load_platforms_from_config(&profile.config_file);
                    wizard.load_available_platforms(platforms);

                    self.dns_wizard = Some(wizard);
                }
            ui.add_space(8.0);

            if self.domain_rows.is_empty() {
                ui.label(tr!("dns-main-empty"));
                return;
            }

            // Build data table
            let table_id = egui::Id::new("dns_domains_table");

            let mut table = data_table()
                .id(table_id)
                .allow_selection(false)
                .allow_drawer(true)
                .min_row_height(68.0)
                .drawer_row_height(400.0)
                .column(tr!("dns-table-col-dns"), 230.0 * width_ratio, false)
                .column(tr!("dns-table-col-operations"), 510.0 * width_ratio, false);

            for row in self.domain_rows.clone().iter() {
                let row_for_cells = row.clone();
                let row_for_ops = row.clone();
                let row_for_drawer = row.clone();

                table = table.row(move |r| {
                    r.cell_widget(move |ui| {
                        // DNS column: badge + domain name
                        ui.vertical(|ui| {
                            ui.add(
                                badge(&row_for_cells.provider)
                                    .color(BadgeColor::Primary)
                                    .size(BadgeSize::Small)
                            );
                            use egui_material3::theme::get_global_color;
                            ui.label(egui::RichText::new(&row_for_cells.domain)
                                .size(16.0)
                                .strong()
                                .color(get_global_color("onSurface")));
                        });
                    })
                    .cell_widget(move |ui| {
                        // Operations column: buttons
                        use egui_material3::linear_progress;

                        ui.spacing_mut().item_spacing.x = BUTTON_HORIZONTAL_SPACING;
                        ui.spacing_mut().item_spacing.y = BUTTON_VERTICAL_SPACING;

                        let domain = row_for_ops.domain.clone();
                        let operation_in_progress = matches!(row_for_ops.operation_state, OperationState::InProgress { .. });

                        ui.vertical(|ui| {
                            let overlay_pos = ui.cursor().min;
                            let column_width = ui.available_width();

                            ui.horizontal_wrapped(|ui| {
                            if ui
                                .add(MaterialButton::outlined(tr!("dns-button-check-ns")).small())
                                .on_hover_text(tr!("dns-button-check-ns-tooltip"))
                                .clicked()
                            {
                                ui.data_mut(|d| d.insert_temp(
                                    egui::Id::new("dns_check_ns"),
                                    domain.clone()
                                ));
                            }

                            if ui
                                .add(MaterialButton::outlined(tr!("dns-button-check-settings")).small())
                                .on_hover_text(tr!("dns-button-check-settings-tooltip"))
                                .clicked()
                            {
                                ui.data_mut(|d| d.insert_temp(
                                    egui::Id::new("dns_check_settings"),
                                    domain.clone()
                                ));
                            }

                            if ui
                                .add(MaterialButton::outlined(tr!("dns-button-https-check")).small())
                                .on_hover_text(tr!("dns-button-https-check-tooltip"))
                                .clicked()
                            {
                                ui.data_mut(|d| d.insert_temp(
                                    egui::Id::new("dns_check_https"),
                                    domain.clone()
                                ));
                            }

                            if ui
                                .add(MaterialButton::outlined(tr!("dns-button-delete")).small())
                                .on_hover_text(tr!("dns-button-delete-tooltip"))
                                .clicked()
                            {
                                ui.data_mut(|d| d.insert_temp(
                                    egui::Id::new("dns_delete_domain"),
                                    domain.clone()
                                ));
                            }
                        }); // Close ui.horizontal_wrapped

                        // Overlay progress on foreground layer
                        if operation_in_progress {
                            let current_area_id = ui.layer_id().id;
                            let foreground_layer = egui::LayerId::new(egui::Order::Foreground, current_area_id);

                            ui.with_layer_id(foreground_layer, |ui| {
                                use egui_material3::theme::get_global_color;
                                let progress_rect = egui::Rect::from_min_size(overlay_pos, egui::vec2(column_width, 30.0));
                                let overlay_color = get_global_color("surfaceContainer");
                                ui.painter().rect_filled(progress_rect, egui::Rounding::same(4), overlay_color);

                                ui.allocate_ui_at_rect(progress_rect, |ui| {
                                    ui.add_space(2.0);
                                    if let OperationState::InProgress { operation, .. } = &row_for_ops.operation_state {
                                        ui.label(operation);
                                    }
                                    ui.add(linear_progress().indeterminate(true).width(column_width - 20.0).height(4.0).four_color_enabled(true));
                                });
                            });
                        }
                        }); // Close ui.vertical
                    })
                    .drawer(move |ui| {
                        // Drawer with tabs
                        let mut tab_switch = None;
                        crate::ui_tabs::dns_drawer::render_drawer(
                            ui,
                            &row_for_drawer,
                            &row_for_drawer.drawer_state,
                            &mut tab_switch,
                        );

                        if let Some(new_tab) = tab_switch {
                            ui.data_mut(|d| {
                                d.insert_temp(
                                    egui::Id::new("dns_drawer_tab_switch").with(&row_for_drawer.domain),
                                    (row_for_drawer.domain.clone(), new_tab),
                                );
                            });
                        }
                    })
                });
            }

            table.show(ui);
        });
    }

    /// Check NS server records using DNS over HTTPS
    fn check_ns_server(&mut self, domain: &str) {
        dure_info!("Checking NS server for domain: {}", domain);

        // Set operation state to InProgress
        if let Some(row) = self.domain_rows.iter_mut().find(|r| r.domain == domain) {
            row.operation_state = OperationState::start(tr!("dns-operation-check-ns"));
        }

        // Store which domain is being checked
        self.selected_domain = Some(domain.to_string());

        let domain = domain.to_string();
        let promise = Promise::spawn_thread("check_ns", move || {
            // Use DNS-over-HTTPS (Cloudflare 1.1.1.1)
            let url = format!("https://1.1.1.1/dns-query?name={}&type=NS", domain);

            match ureq::get(&url)
                .set("Accept", "application/dns-json")
                .timeout(std::time::Duration::from_secs(5))
                .call()
            {
                Ok(response) => {
                    #[derive(serde::Deserialize)]
                    struct DnsResponse {
                        #[serde(rename = "Answer")]
                        answer: Option<Vec<DnsAnswer>>,
                    }

                    #[derive(serde::Deserialize)]
                    struct DnsAnswer {
                        data: String,
                    }

                    match response.into_json::<DnsResponse>() {
                        Ok(dns_resp) => {
                            if let Some(answers) = dns_resp.answer {
                                let nameservers: Vec<String> = answers.iter()
                                    .map(|a| a.data.trim_end_matches('.').to_string())
                                    .collect();

                                // Check if using Cloudflare nameservers
                                let uses_cloudflare = nameservers.iter().any(|ns| {
                                    ns.ends_with(".ns.cloudflare.com")
                                });

                                let mut results = Vec::new();
                                results.push(tr!("dns-check-ns-found", {count: nameservers.len()}));
                                for ns in &nameservers {
                                    results.push(format!("  {}", ns));
                                }

                                Ok((NsCheckResult {
                                    checked_at: chrono::Utc::now().timestamp(),
                                    uses_cloudflare,
                                    nameservers,
                                    error: None,
                                }, results))
                            } else {
                                Err((
                                    NsCheckResult {
                                        checked_at: chrono::Utc::now().timestamp(),
                                        uses_cloudflare: false,
                                        nameservers: Vec::new(),
                                        error: Some(tr!("dns-check-ns-none")),
                                    },
                                    vec![format!("✗ {}", tr!("dns-check-ns-none"))]
                                ))
                            }
                        }
                        Err(e) => {
                            let err_msg = format!("Failed to parse DNS response: {}", e);
                            Err((
                                NsCheckResult {
                                    checked_at: chrono::Utc::now().timestamp(),
                                    uses_cloudflare: false,
                                    nameservers: Vec::new(),
                                    error: Some(err_msg.clone()),
                                },
                                vec![format!("✗ {}", err_msg)]
                            ))
                        }
                    }
                }
                Err(e) => {
                    let err_msg = format!("DNS query failed: {}", e);
                    Err((
                        NsCheckResult {
                            checked_at: chrono::Utc::now().timestamp(),
                            uses_cloudflare: false,
                            nameservers: Vec::new(),
                            error: Some(err_msg.clone()),
                        },
                        vec![format!("✗ {}", err_msg)]
                    ))
                }
            }
        });

        self.ns_check_promise = Some(promise);
        self.show_ns_check_dialog = true;
    }

    /// Check DNS settings via Cloudflare API
    fn check_dns_settings(&mut self, domain: &str) {
        dure_info!("Checking DNS settings for domain: {}", domain);

        // Set operation state to InProgress
        if let Some(row) = self.domain_rows.iter_mut().find(|r| r.domain == domain) {
            row.operation_state = OperationState::start(tr!("dns-operation-check-settings"));
        }

        // Find domain row to get zone_id, vm_ip, and API credentials
        let row = self.domain_rows.iter().find(|r| r.domain == domain);
        if row.is_none() {
            dure_warn!("Domain row not found for: {}", domain);
            return;
        }

        // TODO: Get Cloudflare API token and zone_id from config
        // For now, create placeholder promise
        let domain = domain.to_string();
        let vm_ip = row.and_then(|r| r.vm_external_ip.clone());

        self.selected_domain = Some(domain.clone());

        let promise = Promise::spawn_thread("check_dns_settings", move || {
            // TODO: Implement Cloudflare API check
            // For now, return error indicating not implemented
            let check_result = SettingsCheckResult {
                checked_at: chrono::Utc::now().timestamp(),
                records_valid: false,
                missing_records: vec![
                    tr!("dns-check-settings-root"),
                    tr!("dns-check-settings-www"),
                ],
                error: Some("Cloudflare API check not yet implemented - needs zone_id and API token".to_string()),
            };

            let display = vec![
                "⚠ Cloudflare API check not implemented".to_string(),
                "  Missing: A record for root domain".to_string(),
                "  Missing: A/CNAME for www subdomain".to_string(),
            ];

            Err((check_result, display))
        });

        self.dns_settings_promise = Some(promise);
        self.show_dns_settings_dialog = true;
    }

    /// Check HTTPS connection and verify IP resolution
    fn check_https_connection(&mut self, domain: &str) {
        dure_info!("Checking HTTPS connection for domain: {}", domain);

        // Set operation state to InProgress
        if let Some(row) = self.domain_rows.iter_mut().find(|r| r.domain == domain) {
            row.operation_state = OperationState::start(tr!("dns-operation-check-https"));
        }

        // Find domain row to get expected VM IP
        let row = self.domain_rows.iter().find(|r| r.domain == domain);
        let expected_ip = row.and_then(|r| r.vm_external_ip.clone());

        self.selected_domain = Some(domain.to_string());

        let domain = domain.to_string();
        let promise = Promise::spawn_thread("check_https", move || {
            // Resolve domain IP using DoH
            #[derive(serde::Deserialize)]
            struct DnsResponse {
                #[serde(rename = "Answer")]
                answer: Option<Vec<DnsAnswer>>,
            }

            #[derive(serde::Deserialize)]
            struct DnsAnswer {
                data: String,
            }

            let resolved_ip = match ureq::get(&format!("https://1.1.1.1/dns-query?name={}&type=A", domain))
                .set("Accept", "application/dns-json")
                .timeout(std::time::Duration::from_secs(5))
                .call()
            {
                Ok(resp) => {
                    match resp.into_json::<DnsResponse>() {
                        Ok(dns_resp) => {
                            dns_resp.answer.and_then(|answers| answers.first().map(|a| a.data.clone()))
                        }
                        Err(e) => {
                            return Ok(HttpsCheckResult {
                                checked_at: chrono::Utc::now().timestamp(),
                                resolves_to_vm: false,
                                resolved_ip: None,
                                expected_ip,
                                error: Some(format!("DNS parse error: {}", e)),
                            });
                        }
                    }
                }
                Err(e) => {
                    return Ok(HttpsCheckResult {
                        checked_at: chrono::Utc::now().timestamp(),
                        resolves_to_vm: false,
                        resolved_ip: None,
                        expected_ip,
                        error: Some(format!("DNS resolution failed: {}", e)),
                    });
                }
            };

            // Compare resolved IP with expected VM IP
            let resolves_to_vm = if let (Some(ref resolved), Some(ref expected)) = (&resolved_ip, &expected_ip) {
                resolved == expected
            } else {
                false
            };

            Ok(HttpsCheckResult {
                checked_at: chrono::Utc::now().timestamp(),
                resolves_to_vm,
                resolved_ip,
                expected_ip,
                error: None,
            })
        });

        self.https_check_promise = Some(promise);
        self.show_https_check_dialog = true;
    }

    /// Render all dialogs
    fn render_dialogs(&mut self, ui: &mut egui::Ui, profile: &ProfileContext) {
        // Add DNS wizard
        if self.show_add_dns_wizard {
            if let Some(wizard) = &mut self.dns_wizard {
                let mut open = true;
                egui::Window::new("Add DNS - Automated Setup")
                    .id(egui::Id::new("add_dns_wizard_dialog"))
                    .title_bar(false)
                    .open(&mut open)
                    .resizable(false)
                    .collapsible(false)
                    .scroll([false, true])
                    .min_width(400.0)
                    .max_width(400.0)
                    .min_height(460.0)
                    .resize(|r| {
                        r.default_size([400.0, 510.0])
                    })
                    .show(ui.ctx(), |ui| {
                        wizard.ui(ui, profile);
                    });

                // Close wizard if requested or window closed
                if !open || wizard.should_close {
                    // Check if wizard completed a domain setup
                    if let Some((provider, domain, zone_id, account_id)) = &wizard.completed_domain {
                        // Extract platform/VM info from wizard
                        let (platform_id, platform_name, vm_name, vm_external_ip) =
                            wizard.get_selected_platform_info();

                        // Add to domain rows if not already present
                        if !self.domain_rows.iter().any(|r| &r.domain == domain) {
                            self.domain_rows.push(DnsDomainRow {
                                provider: provider.clone(),
                                domain: domain.clone(),
                                status: DomainStatus::Configured,
                                platform_id: platform_id.clone(),
                                platform_name,
                                vm_name,
                                vm_external_ip,
                                account_id: account_id.clone(),
                                last_ns_check: None,
                                last_settings_check: None,
                                last_https_check: None,
                                operation_state: OperationState::Idle,
                                drawer_state: DnsDrawerState::default(),
                            });

                            // Save to config
                            Self::save_domain_to_config(
                                &profile.config_file,
                                provider,
                                domain,
                                zone_id,
                                account_id,
                                &platform_id,
                            );
                        }
                    }

                    self.show_add_dns_wizard = false;
                    self.dns_wizard = None;
                }
            }
        }

        // NS check dialog
        if self.show_ns_check_dialog {
            // Poll NS check promise
            if let Some(promise) = self.ns_check_promise.take() {
                if let Some(result) = promise.ready() {
                    match result {
                        Ok((check_result, display_results)) => {
                            self.ns_check_results = display_results.clone();
                            // Update domain row with check result and operation state
                            if let Some(domain) = &self.selected_domain {
                                if let Some(row) = self.domain_rows.iter_mut().find(|r| &r.domain == domain) {
                                    row.last_ns_check = Some(check_result.clone());
                                    row.operation_state = row.operation_state.clone().complete();
                                }
                            }
                        }
                        Err((check_result, display_results)) => {
                            self.ns_check_results = display_results.clone();
                            // Store error result and update operation state
                            if let Some(domain) = &self.selected_domain {
                                if let Some(row) = self.domain_rows.iter_mut().find(|r| &r.domain == domain) {
                                    row.last_ns_check = Some(check_result.clone());
                                    let error_msg = check_result.error.clone().unwrap_or_else(|| tr!("dns-error-unknown"));
                                    row.operation_state = row.operation_state.clone().fail(error_msg);
                                }
                            }
                        }
                    }
                } else {
                    ui.ctx().request_repaint();
                    self.ns_check_promise = Some(promise);
                }
            }

            let ns_check_results = self.ns_check_results.clone();

            dialog("ns_server_check_dialog", tr!("dns-dialog-ns-title"), &mut self.show_ns_check_dialog)
                .min_width(400.0)
                .max_width(400.0)
                .max_height(450.0)
                .scrollable(true)
                .content(|ui, open| {
                    if ns_check_results.is_empty() {
                        ui.spinner();
                        ui.label(tr!("dns-dialog-ns-querying"));
                    } else {
                        for result in &ns_check_results {
                            ui.label(result);
                        }
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(MaterialButton::filled(tr!("close"))).clicked() {
                            *open = false;
                        }
                    });
                })
                .show(ui.ctx());
        }

        // DNS settings check dialog
        if self.show_dns_settings_dialog {
            // Poll DNS settings check promise
            if let Some(promise) = self.dns_settings_promise.take() {
                if let Some(result) = promise.ready() {
                    match result {
                        Ok((check_result, display_results)) => {
                            // Store result in domain row and update operation state
                            if let Some(domain) = &self.selected_domain {
                                if let Some(row) = self.domain_rows.iter_mut().find(|r| &r.domain == domain) {
                                    row.last_settings_check = Some(check_result.clone());
                                    row.operation_state = row.operation_state.clone().complete();
                                }
                            }
                            // Convert display strings to DnsRecordCheck for legacy dialog
                            self.dns_settings_results = display_results.iter().map(|msg| {
                                DnsRecordCheck {
                                    record_type: "INFO".to_string(),
                                    name: msg.clone(),
                                    expected: String::new(),
                                    actual: None,
                                    valid: !msg.contains('✗'),
                                }
                            }).collect();
                        }
                        Err((check_result, display_results)) => {
                            // Store error result in domain row and update operation state
                            if let Some(domain) = &self.selected_domain {
                                if let Some(row) = self.domain_rows.iter_mut().find(|r| &r.domain == domain) {
                                    row.last_settings_check = Some(check_result.clone());
                                    let error_msg = check_result.error.clone().unwrap_or_else(|| tr!("dns-error-unknown"));
                                    row.operation_state = row.operation_state.clone().fail(error_msg);
                                }
                            }
                            // Convert display strings to DnsRecordCheck for legacy dialog
                            self.dns_settings_results = display_results.iter().map(|msg| {
                                DnsRecordCheck {
                                    record_type: "ERROR".to_string(),
                                    name: msg.clone(),
                                    expected: String::new(),
                                    actual: None,
                                    valid: false,
                                }
                            }).collect();
                        }
                    }
                } else {
                    ui.ctx().request_repaint();
                    self.dns_settings_promise = Some(promise);
                }
            }

            let dns_settings_results = self.dns_settings_results.clone();

            dialog("dns_settings_check_dialog", tr!("dns-dialog-settings-title"), &mut self.show_dns_settings_dialog)
                .min_width(600.0)
                .max_height(500.0)
                .scrollable(true)
                .content(|ui, open| {
                    if dns_settings_results.is_empty() {
                        ui.spinner();
                        ui.label(tr!("dns-dialog-settings-checking"));
                    } else {
                        ui.heading(tr!("dns-dialog-settings-verification"));
                        ui.add_space(10.0);

                        use egui_extras::{TableBuilder, Column};
                        TableBuilder::new(ui)
                            .striped(true)
                            .column(Column::exact(60.0))  // Type
                            .column(Column::remainder())  // Name
                            .column(Column::remainder())  // Expected
                            .column(Column::remainder())  // Actual
                            .column(Column::exact(60.0))  // Status
                            .header(20.0, |mut header| {
                                header.col(|ui| { ui.strong(tr!("dns-dialog-settings-col-type")); });
                                header.col(|ui| { ui.strong(tr!("dns-dialog-settings-col-name")); });
                                header.col(|ui| { ui.strong(tr!("dns-dialog-settings-col-expected")); });
                                header.col(|ui| { ui.strong(tr!("dns-dialog-settings-col-actual")); });
                                header.col(|ui| { ui.strong(tr!("dns-dialog-settings-col-status")); });
                            })
                            .body(|mut body| {
                                for check in &dns_settings_results {
                                    body.row(18.0, |mut row| {
                                        row.col(|ui| { ui.label(&check.record_type); });
                                        row.col(|ui| { ui.label(&check.name); });
                                        row.col(|ui| { ui.label(&check.expected); });
                                        row.col(|ui| {
                                            if let Some(actual) = &check.actual {
                                                ui.label(actual);
                                            } else {
                                                ui.weak(tr!("dns-dialog-settings-not-found"));
                                            }
                                        });
                                        row.col(|ui| {
                                            use egui_material3::theme::get_global_color;
                                            if check.valid {
                                                ui.label("✓");
                                            } else {
                                                ui.colored_label(get_global_color("error"), "✗");
                                            }
                                        });
                                    });
                                }
                            });

                        ui.add_space(8.0);

                        // Show "Set Correct Records" button if there are invalid records
                        if !dns_settings_results.is_empty() {
                            let all_valid = dns_settings_results.iter().all(|r| r.valid);
                            if !all_valid {
                                ui.horizontal(|ui| {
                                    if ui.add(MaterialButton::filled(tr!("dns-button-set-records"))).clicked() {
                                        // TODO: Implement Cloudflare API call to update DNS records
                                        dure_warn!("Set DNS records not implemented yet - needs Cloudflare API integration");
                                    }
                                    ui.label(tr!("dns-button-set-records-note"));
                                });
                            }
                        }
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(MaterialButton::filled("Close")).clicked() {
                            *open = false;
                        }
                    });
                })
                .show(ui.ctx());
        }

        // HTTPS check dialog
        if self.show_https_check_dialog {
            // Poll HTTPS check promise
            if let Some(promise) = self.https_check_promise.take() {
                if let Some(result) = promise.ready() {
                    match result {
                        Ok(check_result) => {
                            self.https_check_result = Some(check_result.clone());
                            // Store result in domain row and update operation state
                            if let Some(domain) = &self.selected_domain {
                                if let Some(row) = self.domain_rows.iter_mut().find(|r| &r.domain == domain) {
                                    row.last_https_check = Some(check_result.clone());
                                    row.operation_state = row.operation_state.clone().complete();
                                }
                            }
                        }
                        Err(err) => {
                            let error_result = HttpsCheckResult {
                                checked_at: chrono::Utc::now().timestamp(),
                                resolves_to_vm: false,
                                resolved_ip: None,
                                expected_ip: None,
                                error: Some(err.clone()),
                            };
                            self.https_check_result = Some(error_result.clone());
                            // Store error result in domain row and update operation state
                            if let Some(domain) = &self.selected_domain {
                                if let Some(row) = self.domain_rows.iter_mut().find(|r| &r.domain == domain) {
                                    row.last_https_check = Some(error_result);
                                    row.operation_state = row.operation_state.clone().fail(err);
                                }
                            }
                        }
                    }
                } else {
                    ui.ctx().request_repaint();
                    self.https_check_promise = Some(promise);
                }
            }

            let https_check_result = self.https_check_result.clone();

            dialog("https_connection_check_dialog", tr!("dns-dialog-https-title"), &mut self.show_https_check_dialog)
                .min_width(400.0)
                .max_width(400.0)
                .max_height(450.0)
                .scrollable(true)
                .content(|ui, open| {
                    if let Some(result) = &https_check_result {
                        ui.heading(if result.resolves_to_vm { tr!("dns-dialog-https-ok") } else { tr!("dns-dialog-https-mismatch") });
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.label(tr!("dns-dialog-https-resolved-ip"));
                            if let Some(ip) = &result.resolved_ip {
                                ui.label(ip);
                            } else {
                                ui.weak(tr!("dns-dialog-https-na"));
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label(tr!("dns-dialog-https-expected-ip"));
                            if let Some(ip) = &result.expected_ip {
                                ui.label(ip);
                            } else {
                                ui.weak(tr!("dns-dialog-https-na"));
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label(tr!("dns-dialog-https-matches-vm"));
                            if result.resolves_to_vm {
                                ui.label(tr!("dns-dialog-https-yes"));
                            } else {
                                ui.colored_label(egui::Color32::from_rgb(255, 193, 7), tr!("dns-dialog-https-no"));
                            }
                        });

                        if let Some(err_msg) = &result.error {
                            use egui_material3::theme::get_global_color;
                            ui.add_space(10.0);
                            ui.colored_label(get_global_color("error"), format!("{} {}", tr!("dns-dialog-https-error-prefix"), err_msg));
                        }
                    } else {
                        ui.spinner();
                        ui.label(tr!("dns-dialog-https-checking"));
                    }

                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.add(MaterialButton::filled(tr!("close"))).clicked() {
                            *open = false;
                        }
                    });
                })
                .show(ui.ctx());
        }
    }

    /// Load platforms from config file
    fn load_platforms_from_config(config_file: &std::path::Path) -> Vec<crate::ui_dlg::dns_wizard::PlatformVmInfo> {
        let mut platforms = Vec::new();

        if let Ok(config) = crate::config::AppConfig::load(&config_file.to_path_buf()) {
            for (idx, platform_config) in config.platforms.iter().enumerate() {
                if let Some(vm) = platform_config.vms.first() {
                    let project_id = platform_config.gcp_selected_project_id.clone()
                        .or_else(|| Some(vm.gcp_project_id.clone()))
                        .unwrap_or_else(|| format!("project-{}", idx));

                    let platform_name = format!("Platform {} ({})", idx, project_id);

                    platforms.push(crate::ui_dlg::dns_wizard::PlatformVmInfo {
                        project_id,
                        platform_name,
                        vm_name: vm.name.clone(),
                        vm_external_ip: vm.external_ip.clone().unwrap_or_default(),
                    });
                }
            }
        }

        platforms
    }

    /// Save DNS domain to config file
    fn save_domain_to_config(
        config_file: &std::path::Path,
        provider: &str,
        domain: &str,
        zone_id: &Option<String>,
        account_id: &Option<String>,
        platform_id: &Option<String>,
    ) {
        use crate::config::{AppConfig, DnsDomainConfig};

        // Load config
        let mut config = AppConfig::load(&config_file.to_path_buf())
            .unwrap_or_default();

        // Check if domain already exists
        if config.dns_domains.iter().any(|d| &d.domain == domain) {
            dure_warn!("Domain {} already in config, skipping", domain);
            return;
        }

        // Add new domain
        config.dns_domains.push(DnsDomainConfig {
            provider: provider.to_string(),
            domain: domain.to_string(),
            zone_id: zone_id.clone(),
            account_id: account_id.clone(),
            platform_id: platform_id.clone(),
        });

        // Save config
        if let Err(e) = config.save(&config_file.to_path_buf()) {
            dure_error!("Failed to save DNS domain to config: {}", e);
        } else {
            dure_info!("Saved DNS domain {} to config", domain);
        }
    }

    /// Delete DNS domain from config and memory
    fn delete_domain(&mut self, domain: &str, profile: &ProfileContext) {
        use crate::config::AppConfig;

        dure_info!("Deleting DNS domain: {}", domain);

        // Remove from memory
        self.domain_rows.retain(|row| row.domain != domain);

        // Remove from config file
        let mut config = AppConfig::load(&profile.config_file)
            .unwrap_or_default();

        let original_count = config.dns_domains.len();
        config.dns_domains.retain(|d| &d.domain != domain);

        if config.dns_domains.len() < original_count {
            if let Err(e) = config.save(&profile.config_file) {
                dure_error!("Failed to save config after deleting domain: {}", e);
            } else {
                dure_info!("Successfully deleted DNS domain {} from config", domain);
            }
        } else {
            dure_warn!("Domain {} not found in config", domain);
        }
    }

    /// Load DNS domains from config file
    #[cfg(not(target_arch = "wasm32"))]
    fn load_domain_rows(&mut self, profile: &ProfileContext) {
        use crate::config::AppConfig;

        // Clear existing rows
        self.domain_rows.clear();

        // Load config
        let config = match AppConfig::load(&profile.config_file) {
            Ok(config) => config,
            Err(e) => {
                dure_warn!("Failed to load config for DNS domains: {}", e);
                self.loaded = true;
                return;
            }
        };

        // Convert DnsDomainConfig to DnsDomainRow
        for dns_config in &config.dns_domains {
            // Lookup platform info from config if platform_id is set
            let (platform_id, platform_name, vm_name, vm_external_ip) = if let Some(pid) = &dns_config.platform_id {
                let platforms = Self::load_platforms_from_config(&profile.config_file);
                platforms.iter()
                    .find(|p| &p.project_id == pid)
                    .map(|p| (
                        Some(p.project_id.clone()),
                        Some(p.platform_name.clone()),
                        Some(p.vm_name.clone()),
                        Some(p.vm_external_ip.clone()),
                    ))
                    .unwrap_or_else(|| (Some(pid.clone()), None, None, None))
            } else {
                (None, None, None, None)
            };

            self.domain_rows.push(DnsDomainRow {
                provider: dns_config.provider.clone(),
                domain: dns_config.domain.clone(),
                status: DomainStatus::Configured,
                platform_id,
                platform_name,
                vm_name,
                vm_external_ip,
                account_id: dns_config.account_id.clone(),
                last_ns_check: None,
                last_settings_check: None,
                last_https_check: None,
                operation_state: OperationState::Idle,
                drawer_state: DnsDrawerState::default(),
            });
        }

        dure_debug!("Loaded {} DNS domains from config", self.domain_rows.len());
        self.loaded = true;
    }

    /// WASM stub for load_domain_rows
    #[cfg(target_arch = "wasm32")]
    fn load_domain_rows(&mut self, _profile: &ProfileContext) {
        self.loaded = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dns_drawer_state_default() {
        let state = DnsDrawerState::default();
        assert_eq!(state.active_tab, DnsDrawerTab::Status);
        assert!(state.logs.is_empty());
        assert!(state.operations.is_empty());
        assert!(!state.loading);
    }

    #[test]
    fn test_dns_drawer_state_new() {
        let state = DnsDrawerState::new();
        assert_eq!(state.active_tab, DnsDrawerTab::Status);
        assert!(state.logs.is_empty());
        assert!(state.operations.is_empty());
        assert!(!state.loading);
    }

    #[test]
    fn test_dns_drawer_state_switch_tab() {
        let mut state = DnsDrawerState::new();
        assert_eq!(state.active_tab, DnsDrawerTab::Status);

        state.switch_tab(DnsDrawerTab::Logs);
        assert_eq!(state.active_tab, DnsDrawerTab::Logs);

        state.switch_tab(DnsDrawerTab::Operations);
        assert_eq!(state.active_tab, DnsDrawerTab::Operations);
    }

    #[test]
    fn test_dns_drawer_state_set_logs() {
        let mut state = DnsDrawerState::new();
        state.loading = true;

        let logs = vec!["log1".to_string(), "log2".to_string()];
        state.set_logs(logs.clone());

        assert_eq!(state.logs, logs);
        assert!(!state.loading, "set_logs should clear loading flag");
    }

    #[test]
    fn test_dns_drawer_state_set_operations() {
        let mut state = DnsDrawerState::new();
        state.loading = true;

        let ops = vec![];
        state.set_operations(ops);

        assert!(state.operations.is_empty());
        assert!(!state.loading, "set_operations should clear loading flag");
    }

    #[test]
    fn test_dns_drawer_state_set_loading() {
        let mut state = DnsDrawerState::new();
        assert!(!state.loading);

        state.set_loading(true);
        assert!(state.loading);

        state.set_loading(false);
        assert!(!state.loading);
    }

    #[test]
    fn test_dns_drawer_tab_as_str() {
        assert_eq!(DnsDrawerTab::Status.as_str(), "status");
        assert_eq!(DnsDrawerTab::Logs.as_str(), "logs");
        assert_eq!(DnsDrawerTab::Operations.as_str(), "operations");
    }

    #[test]
    fn test_dns_drawer_tab_all() {
        let tabs = DnsDrawerTab::all();
        assert_eq!(tabs.len(), 3);
        assert_eq!(tabs[0], DnsDrawerTab::Status);
        assert_eq!(tabs[1], DnsDrawerTab::Logs);
        assert_eq!(tabs[2], DnsDrawerTab::Operations);
    }

    #[test]
    fn test_operation_state_lifecycle() {
        let state = OperationState::start("Check NS");

        match &state {
            OperationState::InProgress { operation, .. } => {
                assert_eq!(operation, "Check NS");
            }
            _ => panic!("Expected InProgress state"),
        }

        let state = state.complete();

        match &state {
            OperationState::Completed { operation, .. } => {
                assert_eq!(operation, "Check NS");
            }
            _ => panic!("Expected Completed state"),
        }
    }

    #[test]
    fn test_operation_state_failure() {
        let state = OperationState::start("HTTPS Check");
        let state = state.fail("DNS resolution failed");

        match &state {
            OperationState::Failed { operation, error, .. } => {
                assert_eq!(operation, "HTTPS Check");
                assert_eq!(error, "DNS resolution failed");
            }
            _ => panic!("Expected Failed state"),
        }
    }

    #[test]
    fn test_operation_state_default_is_idle() {
        let state = OperationState::default();
        assert_eq!(state, OperationState::Idle);
    }
}
