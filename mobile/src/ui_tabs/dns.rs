//! DNS tab - Automated DNS management with Cloudflare OAuth
//!
//! This tab provides automated DNS setup through Cloudflare OAuth workflow:
//! 1. OAuth authentication with PKCE
//! 2. Domain registration/connection via Cloudflare Registrar API
//! 3. GCP VM integration for automatic NS record configuration
//! 4. DNS verification and HTTPS connection testing

use crate::{dure_debug, dure_info, dure_warn};
use eframe::egui;
use egui_i18n::tr;
use egui_material3::MaterialButton;
use poll_promise::Promise;

use crate::calc::profile::ProfileContext;

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
    ns_check_promise: Option<Promise<Result<Vec<String>, String>>>,

    /// DNS settings check promise
    #[cfg_attr(feature = "serde", serde(skip))]
    dns_settings_promise: Option<Promise<Result<Vec<DnsRecordCheck>, String>>>,

    /// HTTPS check promise
    #[cfg_attr(feature = "serde", serde(skip))]
    https_check_promise: Option<Promise<Result<HttpsCheckResult, String>>>,
}

/// Domain row data for table
#[derive(Clone, Debug)]
struct DnsDomainRow {
    provider: String,        // "cloudflare"
    domain: String,          // "dure.co"
    status: DomainStatus,    // Configured, Pending, Error
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
struct HttpsCheckResult {
    success: bool,
    status_code: Option<u16>,
    ssl_valid: bool,
    response_time_ms: u64,
    error: Option<String>,
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
        }
    }
}

impl DnsTab {
    /// Render the DNS tab
    pub fn ui(&mut self, ui: &mut egui::Ui, profile: &ProfileContext) {
        ui.vertical(|ui| {
            // Header with "Add DNS" button
            ui.horizontal(|ui| {
                ui.heading("DNS Management");
                ui.add_space(10.0);

                if ui
                    .add(MaterialButton::filled("Add DNS"))
                    .on_hover_text("Add new DNS with automated Cloudflare setup")
                    .clicked()
                {
                    self.show_add_dns_wizard = true;
                    self.dns_wizard = Some(crate::ui_dlg::dns_wizard::DnsWizard::new(
                        profile.name.clone(),
                    ));
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Domain table
            self.render_domain_table(ui);

            // Check for button click events stored in ui.data
            let check_ns = ui.data(|d| d.get_temp::<String>(egui::Id::new("dns_check_ns")));
            let check_settings = ui.data(|d| d.get_temp::<String>(egui::Id::new("dns_check_settings")));
            let check_https = ui.data(|d| d.get_temp::<String>(egui::Id::new("dns_check_https")));

            if let Some(domain) = check_ns {
                self.check_ns_server(&domain);
            }
            if let Some(domain) = check_settings {
                self.check_dns_settings(&domain);
            }
            if let Some(domain) = check_https {
                self.check_https_connection(&domain);
            }

            // Dialogs
            self.render_dialogs(ui, profile);
        });
    }

    /// Render domain table
    fn render_domain_table(&mut self, ui: &mut egui::Ui) {
        if self.domain_rows.is_empty() {
            ui.vertical_centered(|ui| {
                ui.add_space(50.0);
                ui.label("No DNS domains configured");
                ui.label("Click 'Add DNS' to get started with automated Cloudflare setup");
            });
            return;
        }

        use egui_extras::{Column, TableBuilder};

        TableBuilder::new(ui)
            .striped(true)
            .column(Column::exact(150.0)) // Provider
            .column(Column::remainder())   // Domain
            .column(Column::exact(400.0))  // Operations
            .header(30.0, |mut header| {
                header.col(|ui| {
                    ui.heading("Provider");
                });
                header.col(|ui| {
                    ui.heading("DNS");
                });
                header.col(|ui| {
                    ui.heading("Operations");
                });
            })
            .body(|mut body| {
                let rows = self.domain_rows.clone(); // Clone to avoid borrow conflict
                for row in &rows {
                    body.row(40.0, |mut table_row| {
                        // Provider column
                        table_row.col(|ui| {
                            ui.label(&row.provider);
                        });

                        // Domain column
                        table_row.col(|ui| {
                            ui.label(&row.domain);
                        });

                        // Operations column (capture row domain for button callbacks)
                        let domain = row.domain.clone();
                        table_row.col(|ui| {
                            ui.horizontal(|ui| {
                                if ui
                                    .add(MaterialButton::outlined("Check NS Server"))
                                    .on_hover_text("Check nameserver records using dig")
                                    .clicked()
                                {
                                    ui.data_mut(|d| d.insert_temp(
                                        egui::Id::new("dns_check_ns"),
                                        domain.clone()
                                    ));
                                }

                                if ui
                                    .add(MaterialButton::outlined("Check NS Settings"))
                                    .on_hover_text("Verify all DNS records are configured correctly")
                                    .clicked()
                                {
                                    ui.data_mut(|d| d.insert_temp(
                                        egui::Id::new("dns_check_settings"),
                                        domain.clone()
                                    ));
                                }

                                if ui
                                    .add(MaterialButton::outlined("Connection Check"))
                                    .on_hover_text("Test HTTPS connection to domain")
                                    .clicked()
                                {
                                    ui.data_mut(|d| d.insert_temp(
                                        egui::Id::new("dns_check_https"),
                                        domain.clone()
                                    ));
                                }
                            });
                        });
                    });
                }
            });
    }

    /// Render operation buttons for a domain
    fn render_operations(&mut self, ui: &mut egui::Ui, row: &DnsDomainRow) {
        ui.horizontal(|ui| {
            if ui
                .add(MaterialButton::outlined("Check NS Server"))
                .on_hover_text("Check nameserver records using dig")
                .clicked()
            {
                self.check_ns_server(&row.domain);
            }

            if ui
                .add(MaterialButton::outlined("Check NS Settings"))
                .on_hover_text("Verify all DNS records are configured correctly")
                .clicked()
            {
                self.check_dns_settings(&row.domain);
            }

            if ui
                .add(MaterialButton::outlined("Connection Check"))
                .on_hover_text("Test HTTPS connection to domain")
                .clicked()
            {
                self.check_https_connection(&row.domain);
            }
        });
    }

    /// Check NS server records
    fn check_ns_server(&mut self, domain: &str) {
        dure_info!("Checking NS server for domain: {}", domain);

        let domain = domain.to_string();
        let promise = Promise::spawn_thread("check_ns", move || {
            let mut results = Vec::new();
            results.push(format!("Querying NS records for {}...", domain));

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
                                if answers.is_empty() {
                                    results.push("✗ No NS records found".to_string());
                                } else {
                                    results.push(format!("✓ Found {} NS record(s):", answers.len()));
                                    for answer in answers {
                                        results.push(format!("  {}", answer.data));
                                    }
                                }
                            } else {
                                results.push("✗ No NS records found".to_string());
                            }
                            Ok(results)
                        }
                        Err(e) => {
                            results.push(format!("✗ Failed to parse DNS response: {}", e));
                            Err(format!("Parse error: {}", e))
                        }
                    }
                }
                Err(e) => {
                    results.push(format!("✗ DNS query failed: {}", e));
                    Err(format!("Query failed: {}", e))
                }
            }
        });

        self.ns_check_promise = Some(promise);
        self.show_ns_check_dialog = true;
    }

    /// Check DNS settings (all 7 default records)
    fn check_dns_settings(&mut self, domain: &str) {
        dure_info!("Checking DNS settings for domain: {}", domain);

        let domain = domain.to_string();
        let promise = Promise::spawn_thread("check_dns_settings", move || {
            #[derive(serde::Deserialize)]
            struct DnsResponse {
                #[serde(rename = "Answer")]
                answer: Option<Vec<DnsAnswer>>,
            }

            #[derive(serde::Deserialize)]
            struct DnsAnswer {
                data: String,
            }

            let query_dns = |name: &str, qtype: &str| -> Option<String> {
                let url = format!("https://1.1.1.1/dns-query?name={}&type={}", name, qtype);
                ureq::get(&url)
                    .set("Accept", "application/dns-json")
                    .timeout(std::time::Duration::from_secs(5))
                    .call()
                    .ok()?
                    .into_json::<DnsResponse>()
                    .ok()?
                    .answer?
                    .first()
                    .map(|a| a.data.clone())
            };

            let mut checks = Vec::new();

            // 1. Check A record for domain
            let expected_ip = query_dns(&domain, "A");
            checks.push(DnsRecordCheck {
                record_type: "A".to_string(),
                name: domain.clone(),
                expected: expected_ip.clone().unwrap_or_else(|| "IP address".to_string()),
                actual: expected_ip.clone(),
                valid: expected_ip.is_some(),
            });

            // 2. Check A record for xmpp.domain
            let xmpp_domain = format!("xmpp.{}", domain);
            let xmpp_ip = query_dns(&xmpp_domain, "A");
            let valid = if let (Some(ref ip1), Some(ref ip2)) = (&expected_ip, &xmpp_ip) {
                ip1 == ip2
            } else {
                false
            };
            checks.push(DnsRecordCheck {
                record_type: "A".to_string(),
                name: xmpp_domain,
                expected: expected_ip.clone().unwrap_or_default(),
                actual: xmpp_ip,
                valid,
            });

            // 3. Check CNAME for www.domain
            let www_domain = format!("www.{}", domain);
            let cname = query_dns(&www_domain, "CNAME");
            let valid = cname.as_ref().map_or(false, |c| c.contains(&domain));
            checks.push(DnsRecordCheck {
                record_type: "CNAME".to_string(),
                name: www_domain,
                expected: domain.clone(),
                actual: cname,
                valid,
            });

            // 4-7. Check SRV records for XMPP
            let srv_records = [
                ("_xmpp-client._tcp", 5222),
                ("_xmpp-server._tcp", 5269),
                ("_xmpps-client._tcp", 5223),
                ("_xmpps-server._tcp", 5270),
            ];

            for (service, port) in srv_records {
                let srv_name = format!("{}.{}", service, domain);
                let srv_data = query_dns(&srv_name, "SRV");
                let valid = srv_data.as_ref().map_or(false, |s| s.contains(&port.to_string()));
                checks.push(DnsRecordCheck {
                    record_type: "SRV".to_string(),
                    name: srv_name,
                    expected: format!("5 {} xmpp.{}", port, domain),
                    actual: srv_data,
                    valid,
                });
            }

            Ok(checks)
        });

        self.dns_settings_promise = Some(promise);
        self.show_dns_settings_dialog = true;
    }

    /// Check HTTPS connection
    fn check_https_connection(&mut self, domain: &str) {
        dure_info!("Checking HTTPS connection for domain: {}", domain);

        let domain = domain.to_string();
        let promise = Promise::spawn_thread("check_https", move || {
            let url = format!("https://{}", domain);
            let start = std::time::Instant::now();

            match ureq::get(&url).timeout(std::time::Duration::from_secs(10)).call() {
                Ok(response) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    Ok(HttpsCheckResult {
                        success: true,
                        status_code: Some(response.status()),
                        ssl_valid: true, // ureq with rustls validates SSL automatically
                        response_time_ms: elapsed,
                        error: None,
                    })
                }
                Err(ureq::Error::Status(code, _response)) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    Ok(HttpsCheckResult {
                        success: false,
                        status_code: Some(code),
                        ssl_valid: true, // Connected but got error status
                        response_time_ms: elapsed,
                        error: Some(format!("HTTP error: {}", code)),
                    })
                }
                Err(ureq::Error::Transport(e)) => {
                    let elapsed = start.elapsed().as_millis() as u64;
                    let is_ssl_error = e.to_string().contains("tls") || e.to_string().contains("ssl");
                    Ok(HttpsCheckResult {
                        success: false,
                        status_code: None,
                        ssl_valid: !is_ssl_error,
                        response_time_ms: elapsed,
                        error: Some(e.to_string()),
                    })
                }
            }
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
                    .open(&mut open)
                    .resizable(true)
                    .default_width(600.0)
                    .show(ui.ctx(), |ui| {
                        wizard.ui(ui, profile);
                    });

                if !open {
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
                        Ok(results) => self.ns_check_results = results.clone(),
                        Err(err) => self.ns_check_results = vec![format!("✗ Error: {}", err)],
                    }
                } else {
                    ui.ctx().request_repaint();
                    self.ns_check_promise = Some(promise);
                }
            }

            let mut open = true;
            egui::Window::new("NS Server Check")
                .open(&mut open)
                .resizable(true)
                .default_width(500.0)
                .show(ui.ctx(), |ui| {
                    if self.ns_check_results.is_empty() {
                        ui.spinner();
                        ui.label("Querying NS records...");
                    } else {
                        for result in &self.ns_check_results {
                            ui.label(result);
                        }
                    }
                });

            if !open {
                self.show_ns_check_dialog = false;
            }
        }

        // DNS settings check dialog
        if self.show_dns_settings_dialog {
            // Poll DNS settings check promise
            if let Some(promise) = self.dns_settings_promise.take() {
                if let Some(result) = promise.ready() {
                    match result {
                        Ok(checks) => self.dns_settings_results = checks.clone(),
                        Err(err) => {
                            dure_warn!("DNS settings check error: {}", err);
                            self.dns_settings_results = vec![DnsRecordCheck {
                                record_type: "ERROR".to_string(),
                                name: err.clone(),
                                expected: String::new(),
                                actual: None,
                                valid: false,
                            }];
                        }
                    }
                } else {
                    ui.ctx().request_repaint();
                    self.dns_settings_promise = Some(promise);
                }
            }

            let mut open = true;
            egui::Window::new("DNS Settings Check")
                .open(&mut open)
                .resizable(true)
                .default_width(700.0)
                .show(ui.ctx(), |ui| {
                    if self.dns_settings_results.is_empty() {
                        ui.spinner();
                        ui.label("Checking all DNS records...");
                    } else {
                        ui.heading("DNS Record Verification");
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
                                header.col(|ui| { ui.strong("Type"); });
                                header.col(|ui| { ui.strong("Name"); });
                                header.col(|ui| { ui.strong("Expected"); });
                                header.col(|ui| { ui.strong("Actual"); });
                                header.col(|ui| { ui.strong("Status"); });
                            })
                            .body(|mut body| {
                                for check in &self.dns_settings_results {
                                    body.row(18.0, |mut row| {
                                        row.col(|ui| { ui.label(&check.record_type); });
                                        row.col(|ui| { ui.label(&check.name); });
                                        row.col(|ui| { ui.label(&check.expected); });
                                        row.col(|ui| {
                                            if let Some(actual) = &check.actual {
                                                ui.label(actual);
                                            } else {
                                                ui.weak("(not found)");
                                            }
                                        });
                                        row.col(|ui| {
                                            if check.valid {
                                                ui.colored_label(egui::Color32::GREEN, "✓");
                                            } else {
                                                ui.colored_label(egui::Color32::RED, "✗");
                                            }
                                        });
                                    });
                                }
                            });
                    }
                });

            if !open {
                self.show_dns_settings_dialog = false;
            }
        }

        // HTTPS check dialog
        if self.show_https_check_dialog {
            // Poll HTTPS check promise
            if let Some(promise) = self.https_check_promise.take() {
                if let Some(result) = promise.ready() {
                    match result {
                        Ok(check_result) => self.https_check_result = Some(check_result.clone()),
                        Err(err) => {
                            self.https_check_result = Some(HttpsCheckResult {
                                success: false,
                                status_code: None,
                                ssl_valid: false,
                                response_time_ms: 0,
                                error: Some(err.clone()),
                            });
                        }
                    }
                } else {
                    ui.ctx().request_repaint();
                    self.https_check_promise = Some(promise);
                }
            }

            let mut open = true;
            egui::Window::new("HTTPS Connection Check")
                .open(&mut open)
                .resizable(true)
                .default_width(500.0)
                .show(ui.ctx(), |ui| {
                    if let Some(result) = &self.https_check_result {
                        ui.heading(if result.success { "✓ Connection Successful" } else { "✗ Connection Failed" });
                        ui.add_space(10.0);

                        ui.horizontal(|ui| {
                            ui.label("Status:");
                            if let Some(code) = result.status_code {
                                ui.label(format!("{}", code));
                            } else {
                                ui.weak("N/A");
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("SSL Valid:");
                            if result.ssl_valid {
                                ui.colored_label(egui::Color32::GREEN, "Yes");
                            } else {
                                ui.colored_label(egui::Color32::RED, "No");
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Response Time:");
                            ui.label(format!("{} ms", result.response_time_ms));
                        });

                        if let Some(error) = &result.error {
                            ui.add_space(10.0);
                            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                        }
                    } else {
                        ui.spinner();
                        ui.label("Testing HTTPS connection...");
                    }
                });

            if !open {
                self.show_https_check_dialog = false;
            }
        }
    }
}
