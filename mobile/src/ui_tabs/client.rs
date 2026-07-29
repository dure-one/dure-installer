//! Client tab - Audit | DNS Client | Crypt Codec | Key Mgmt Ops

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use eframe::egui;
use egui_material3::spreadsheet::{MaterialSpreadsheet, text_column};
use egui_material3::{MaterialButton, tabs_secondary};

// Sub-tab indices
const TAB_AUDIT: usize = 0;
const TAB_DNS_CLIENT: usize = 1;
const TAB_CRYPT_CODEC: usize = 2;
const TAB_KEY_MGMT: usize = 3;

/// Client tab state
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct ClientTab {
    selected_sub: usize,
    audit: AuditSubTab,
    dns_client: DnsClientSubTab,
    crypt_codec: CryptCodecSubTab,
    key_mgmt: KeyMgmtSubTab,
}

impl Default for ClientTab {
    fn default() -> Self {
        Self {
            selected_sub: TAB_AUDIT,
            audit: AuditSubTab::default(),
            dns_client: DnsClientSubTab::default(),
            crypt_codec: CryptCodecSubTab::default(),
            key_mgmt: KeyMgmtSubTab::default(),
        }
    }
}

impl ClientTab {
    /// Render the client tab UI
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.add(
            tabs_secondary(&mut self.selected_sub)
                .id_salt("client_secondary")
                .tab("Audit")
                .tab("DNS Client")
                .tab("Crypt Codec")
                .tab("Key Mgmt Ops"),
        );

        ui.add_space(8.0);

        match self.selected_sub {
            TAB_AUDIT => self.audit.ui(ui),
            TAB_DNS_CLIENT => self.dns_client.ui(ui),
            TAB_CRYPT_CODEC => self.crypt_codec.ui(ui),
            TAB_KEY_MGMT => self.key_mgmt.ui(ui),
            _ => {}
        }
    }
}

// ─── Audit sub-tab ───────────────────────────────────────────────────────────

/// Audit sub-tab: displays a spreadsheet of recent audit records.
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Default)]
pub struct AuditSubTab {
    #[cfg_attr(feature = "serde", serde(skip))]
    spreadsheet: Option<MaterialSpreadsheet>,
    #[cfg_attr(feature = "serde", serde(skip))]
    loaded: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    load_error: Option<String>,
}

impl AuditSubTab {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Audit");
        ui.add_space(4.0);
        ui.label(
            "Comprehensive records of user actions, authentication events, and privilege changes.",
        );
        ui.add_space(8.0);

        // Control buttons
        ui.horizontal(|ui| {
            if ui.add(MaterialButton::outlined("Refresh")).clicked() {
                self.load_audit_records();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Show the DB path
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let db_path = crate::calc::db::get_db_path();
                    ui.label(format!("db: {db_path}"));
                }
            });
        });

        ui.add_space(8.0);

        // Lazy-load from DB on first render
        if !self.loaded {
            self.load_audit_records();
            self.loaded = true;
        }

        if let Some(err) = &self.load_error {
            ui.colored_label(egui::Color32::RED, format!("⚠ {err}"));
            ui.add_space(4.0);
        }

        // Render spreadsheet
        if let Some(spreadsheet) = &mut self.spreadsheet {
            let rows_count = spreadsheet.rows().len();

            ui.group(|ui| {
                ui.set_min_height(400.0);
                ui.set_width(ui.available_width());
                spreadsheet.show(ui);
            });

            ui.add_space(10.0);
            ui.label(format!("Total records: {}", rows_count));
        } else {
            ui.colored_label(egui::Color32::YELLOW, "⚠ Spreadsheet not initialized");
        }
    }

    fn load_audit_records(&mut self) {
        self.load_error = None;

        // Define columns for both WASM and native
        let columns = vec![
            text_column("Record ID", 60.0),
            text_column("Timestamp", 170.0),
            text_column("Category", 120.0),
            text_column("Actor", 130.0),
            text_column("Surface", 80.0),
            text_column("Action", 180.0),
            text_column("Object", 150.0),
            text_column("Outcome", 90.0),
        ];

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::calc::audit;

            match audit::show(100) {
                Ok(records) => {
                    let mut data_rows = Vec::new();

                    for r in records {
                        data_rows.push(vec![
                            r.id.to_string(),
                            format_ts(r.timestamp),
                            r.category,
                            r.actor_id,
                            r.surface,
                            r.action,
                            r.object,
                            r.outcome,
                        ]);
                    }

                    dure_info!(" Loaded {} audit records from database", data_rows.len());

                    match MaterialSpreadsheet::new("dure_audit_log", columns) {
                        Ok(mut new_spreadsheet) => {
                            new_spreadsheet.set_striped(true);
                            new_spreadsheet.set_allow_selection(true);
                            new_spreadsheet.set_row_selection_enabled(true);
                            new_spreadsheet.init_with_data(data_rows);
                            self.spreadsheet = Some(new_spreadsheet);
                            dure_info!(" Spreadsheet refreshed with latest data");
                        }
                        Err(e) => {
                            dure_warn!(" Failed to create spreadsheet: {}", e);
                            self.load_error = Some(format!("Spreadsheet error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    dure_warn!(" Failed to load audit records: {}", e);
                    self.load_error = Some(format!("Failed to load audit records: {e}"));
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            // On WASM, create empty spreadsheet with message
            match MaterialSpreadsheet::new("dure_audit_log", columns) {
                Ok(mut new_spreadsheet) => {
                    new_spreadsheet.set_striped(true);
                    new_spreadsheet.set_allow_selection(true);
                    new_spreadsheet.set_row_selection_enabled(true);
                    // Initialize with empty data
                    new_spreadsheet.init_with_data(vec![]);
                    self.spreadsheet = Some(new_spreadsheet);
                    self.load_error = Some("Database not available on WASM".to_string());
                }
                Err(e) => {
                    self.load_error = Some(format!("Spreadsheet error: {}", e));
                }
            }
        }
    }
}

// ─── DNS Client sub-tab ──────────────────────────────────────────────────────

/// DNS Client sub-tab: displays DNS cache and allows queries
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DnsClientSubTab {
    #[cfg_attr(feature = "serde", serde(skip))]
    spreadsheet: Option<MaterialSpreadsheet>,
    #[cfg_attr(feature = "serde", serde(skip))]
    loaded: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    load_error: Option<String>,

    // Query form fields
    query_domain: String,
    query_type: String,

    // Nameserver management
    nameservers: Vec<String>,
    ns_input: String,
}

impl Default for DnsClientSubTab {
    fn default() -> Self {
        Self {
            spreadsheet: None,
            loaded: false,
            load_error: None,
            query_domain: String::new(),
            query_type: "A".to_string(),
            nameservers: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
            ns_input: String::new(),
        }
    }
}

impl DnsClientSubTab {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("DNS Client");
        ui.add_space(4.0);
        ui.label("Query DNS records with caching (Cloudflare DoH backend)");
        ui.add_space(8.0);

        // Query form
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label("New DNS Query");
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Domain:");
                ui.text_edit_singleline(&mut self.query_domain);

                ui.add_space(8.0);
                ui.label("Type:");
                egui::ComboBox::from_id_salt("dns_query_type")
                    .selected_text(&self.query_type)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.query_type, "A".to_string(), "A");
                        ui.selectable_value(&mut self.query_type, "AAAA".to_string(), "AAAA");
                        ui.selectable_value(&mut self.query_type, "TXT".to_string(), "TXT");
                        ui.selectable_value(&mut self.query_type, "NS".to_string(), "NS");
                    });

                ui.add_space(8.0);
                if ui.add(MaterialButton::filled("Query")).clicked() {
                    self.execute_query();
                }
            });
        });

        ui.add_space(12.0);

        // Control buttons
        ui.horizontal(|ui| {
            if ui.add(MaterialButton::outlined("Refresh Cache")).clicked() {
                self.load_dns_cache();
            }

            ui.add_space(8.0);
            if ui.add(MaterialButton::outlined("Clear Expired")).clicked() {
                self.clear_expired();
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let db_path = crate::calc::db::get_db_path();
                    ui.label(format!("db: {db_path}"));
                }
            });
        });

        ui.add_space(8.0);

        // Lazy-load from DB on first render
        if !self.loaded {
            self.load_dns_cache();
            self.loaded = true;
        }

        if let Some(err) = &self.load_error {
            ui.colored_label(egui::Color32::RED, format!("⚠ {err}"));
            ui.add_space(4.0);
        }

        // Render spreadsheet
        if let Some(spreadsheet) = &mut self.spreadsheet {
            let rows_count = spreadsheet.rows().len();

            ui.group(|ui| {
                ui.set_min_height(300.0);
                ui.set_width(ui.available_width());
                spreadsheet.show(ui);
            });

            ui.add_space(10.0);
            ui.label(format!("Total cached records: {}", rows_count));
        } else {
            ui.colored_label(egui::Color32::YELLOW, "⚠ Spreadsheet not initialized");
        }

        ui.add_space(16.0);

        // Nameserver management section
        ui.separator();
        ui.add_space(8.0);
        // self.nameserver_ui(ui);
    }

    fn nameserver_ui(&mut self, ui: &mut egui::Ui) {
        ui.label("Nameservers");
        ui.add_space(4.0);

        // Add new nameserver
        ui.horizontal(|ui| {
            ui.label("Add NS:");
            ui.text_edit_singleline(&mut self.ns_input);
            if ui.add(MaterialButton::outlined("Add")).clicked() && !self.ns_input.is_empty() {
                self.nameservers.push(self.ns_input.clone());
                self.ns_input.clear();
            }
        });

        ui.add_space(8.0);

        // List nameservers
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    let mut to_remove = None;
                    for (idx, ns) in self.nameservers.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(format!("{}. {}", idx + 1, ns));
                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.add(MaterialButton::outlined("Delete").small()).clicked()
                                    {
                                        to_remove = Some(idx);
                                    }
                                },
                            );
                        });
                    }

                    if let Some(idx) = to_remove {
                        self.nameservers.remove(idx);
                    }
                });
        });
    }

    fn execute_query(&mut self) {
        if self.query_domain.is_empty() {
            self.load_error = Some("Domain name is required".to_string());
            return;
        }

        self.load_error = None;

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::calc::db;
            use crate::calc::dns::{RecordType, resolve_dns};
            use crate::storage::models::dns::{
                cache_dns_records, get_cached_dns_records, init_dns_table,
            };

            let record_type = match self.query_type.as_str() {
                "A" => RecordType::A,
                "AAAA" => RecordType::AAAA,
                "TXT" => RecordType::TXT,
                "NS" => RecordType::NS,
                _ => {
                    self.load_error = Some(format!("Unknown record type: {}", self.query_type));
                    return;
                }
            };

            let mut conn = db::establish_connection();

            if let Err(e) = init_dns_table(&mut conn) {
                self.load_error = Some(format!("Failed to initialize DNS table: {}", e));
                return;
            }

            // Try cache first
            match get_cached_dns_records(&mut conn, &self.query_domain, record_type) {
                Ok(cached) if !cached.is_empty() => {
                    dure_debug!("✓ Using cached DNS results for {} {}", self.query_domain, record_type
                    );
                }
                _ => {
                    // Fetch fresh
                    dure_debug!("Fetching fresh DNS records for {} {}...", self.query_domain, record_type
                    );
                    match resolve_dns(&self.query_domain, record_type) {
                        Ok(records) => {
                            if !records.is_empty() {
                                if let Err(e) = cache_dns_records(&mut conn, &records) {
                                    self.load_error =
                                        Some(format!("Failed to cache records: {}", e));
                                    return;
                                }
                                dure_info!(" Cached {} records", records.len());
                            } else {
                                dure_warn!(" No records found");
                            }
                        }
                        Err(e) => {
                            self.load_error = Some(format!("DNS query failed: {}", e));
                            return;
                        }
                    }
                }
            }

            // Reload cache to show new results
            self.load_dns_cache();
        }
    }

    fn load_dns_cache(&mut self) {
        self.load_error = None;

        // Define columns for both WASM and native
        let columns = vec![
            text_column("Domain", 200.0),
            text_column("Type", 80.0),
            text_column("Value", 250.0),
            text_column("TTL (s)", 80.0),
            text_column("Cached At", 170.0),
        ];

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::calc::db;
            use diesel::prelude::*;

            let mut conn = db::establish_connection();

            // Query all DNS cache records
            let query = "SELECT domain, record_type, value, ttl, timestamp FROM dns_cache ORDER BY timestamp DESC";

            #[derive(QueryableByName)]
            struct DnsCacheRow {
                #[diesel(sql_type = diesel::sql_types::Text)]
                domain: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                record_type: String,
                #[diesel(sql_type = diesel::sql_types::Text)]
                value: String,
                #[diesel(sql_type = diesel::sql_types::BigInt)]
                ttl: i64,
                #[diesel(sql_type = diesel::sql_types::BigInt)]
                timestamp: i64,
            }

            match diesel::sql_query(query).load::<DnsCacheRow>(&mut conn) {
                Ok(rows) => {
                    let mut data_rows = Vec::new();

                    for r in rows {
                        data_rows.push(vec![
                            r.domain,
                            r.record_type,
                            r.value,
                            r.ttl.to_string(),
                            format_ts(r.timestamp),
                        ]);
                    }

                    dure_info!(" Loaded {} DNS cache records", data_rows.len());

                    match MaterialSpreadsheet::new("dure_dns_cache", columns) {
                        Ok(mut new_spreadsheet) => {
                            new_spreadsheet.set_striped(true);
                            new_spreadsheet.set_allow_selection(true);
                            new_spreadsheet.set_row_selection_enabled(true);
                            new_spreadsheet.init_with_data(data_rows);
                            self.spreadsheet = Some(new_spreadsheet);
                        }
                        Err(e) => {
                            self.load_error = Some(format!("Spreadsheet error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load DNS cache: {}", e));
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            // On WASM, create empty spreadsheet with message
            match MaterialSpreadsheet::new("dure_dns_cache", columns) {
                Ok(mut new_spreadsheet) => {
                    new_spreadsheet.set_striped(true);
                    new_spreadsheet.set_allow_selection(true);
                    new_spreadsheet.set_row_selection_enabled(true);
                    // Initialize with empty data
                    new_spreadsheet.init_with_data(vec![]);
                    self.spreadsheet = Some(new_spreadsheet);
                    self.load_error = Some("Database not available on WASM".to_string());
                }
                Err(e) => {
                    self.load_error = Some(format!("Spreadsheet error: {}", e));
                }
            }
        }
    }

    fn clear_expired(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::calc::db;
            use crate::storage::models::dns::clear_expired_dns_cache;

            let mut conn = db::establish_connection();

            match clear_expired_dns_cache(&mut conn) {
                Ok(deleted) => {
                    dure_info!(" Cleared {} expired DNS records", deleted);
                    self.load_dns_cache();
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to clear expired records: {}", e));
                }
            }
        }
    }
}

// ─── Helper functions ────────────────────────────────────────────────────────

fn format_ts(unix_secs: i64) -> String {
    if unix_secs == 0 {
        return "unknown".to_string();
    }
    let secs = unix_secs as u64;
    let days = secs / 86400;
    let rem = secs % 86400;
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let s = rem % 60;
    let (y, mo, d) = days_to_ymd(days);
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", y, mo, d, h, m, s)
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let ydays = if is_leap(year) { 366 } else { 365 };
        if days < ydays {
            break;
        }
        days -= ydays;
        year += 1;
    }
    let month_days: [u64; 12] = [
        31,
        if is_leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u64;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(y: u64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

// ─── Crypt Codec sub-tab ─────────────────────────────────────────────────────

/// Crypt Codec sub-tab: Encryption/Decryption test form
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Default)]
pub struct CryptCodecSubTab {
    // Current device info
    #[cfg_attr(feature = "serde", serde(skip))]
    device_pubkey: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    device_id: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    load_error: Option<String>,

    // Encryption form
    enc_recipient_pubkey: String,
    enc_plaintext: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    enc_result: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    enc_error: Option<String>,

    // Decryption form
    dec_sender_pubkey: String,
    dec_ciphertext: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    dec_result: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    dec_error: Option<String>,

    // Output format
    output_hex: bool,
}

impl CryptCodecSubTab {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Crypt Codec");
        ui.add_space(4.0);
        ui.label("Test encryption and decryption using X25519 + ChaCha20-Poly1305");
        ui.add_space(8.0);

        // Load device keys on first render
        if self.device_pubkey.is_none() && self.load_error.is_none() {
            self.load_device_keys();
        }

        // Display current device info
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label("Current Device");
            ui.add_space(4.0);

            if let Some(err) = &self.load_error {
                ui.colored_label(egui::Color32::RED, format!("⚠ {}", err));
            } else if let (Some(device_id), Some(pubkey)) = (&self.device_id, &self.device_pubkey) {
                ui.horizontal(|ui| {
                    ui.label("Device ID:");
                    ui.code(device_id);
                });
                ui.horizontal(|ui| {
                    ui.label("Public Key:");
                    ui.code(pubkey);
                    if ui
                        .small_button("📋")
                        .on_hover_text("Copy to clipboard")
                        .clicked()
                    {
                        ui.ctx().copy_text(pubkey.clone());
                    }
                });
            } else {
                ui.label("Loading device keys...");
            }
        });

        ui.add_space(12.0);

        // Encryption section
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label("Encrypt Data");
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Recipient Public Key:");
                ui.text_edit_singleline(&mut self.enc_recipient_pubkey)
                    .on_hover_text("Base64 or hex encoded public key (32 bytes)");
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Plaintext:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.enc_plaintext)
                        .desired_width(ui.available_width() - 100.0),
                );
            });

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.output_hex, "Use hex encoding (default: base64)");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(MaterialButton::filled("Encrypt")).clicked() {
                        self.execute_encrypt();
                    }
                });
            });

            ui.add_space(4.0);

            if let Some(err) = &self.enc_error {
                ui.colored_label(egui::Color32::RED, format!("⚠ Error: {}", err));
            }

            if let Some(result) = &self.enc_result {
                ui.add_space(4.0);
                ui.label("Encrypted Result:");
                ui.horizontal(|ui| {
                    let _response = ui.add(
                        egui::TextEdit::multiline(&mut result.as_str())
                            .desired_width(ui.available_width() - 40.0)
                            .desired_rows(3),
                    );
                    if ui
                        .small_button("📋")
                        .on_hover_text("Copy to clipboard")
                        .clicked()
                    {
                        ui.ctx().copy_text(result.clone());
                    }
                });
            }
        });

        ui.add_space(12.0);

        // Decryption section
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label("Decrypt Data");
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Sender Public Key:");
                ui.text_edit_singleline(&mut self.dec_sender_pubkey)
                    .on_hover_text("Optional - for display/verification only");
            });

            ui.add_space(4.0);

            ui.label("Ciphertext:");
            ui.add(
                egui::TextEdit::multiline(&mut self.dec_ciphertext)
                    .desired_width(ui.available_width())
                    .desired_rows(3)
                    .hint_text("Base64 or hex encoded encrypted data"),
            );

            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(MaterialButton::filled("Decrypt")).clicked() {
                        self.execute_decrypt();
                    }
                });
            });

            ui.add_space(4.0);

            if let Some(err) = &self.dec_error {
                ui.colored_label(egui::Color32::RED, format!("⚠ Error: {}", err));
            }

            if let Some(result) = &self.dec_result {
                ui.add_space(4.0);
                ui.label("Decrypted Result:");
                ui.horizontal(|ui| {
                    let _response = ui.add(
                        egui::TextEdit::multiline(&mut result.as_str())
                            .desired_width(ui.available_width() - 40.0)
                            .desired_rows(3),
                    );
                    if ui
                        .small_button("📋")
                        .on_hover_text("Copy to clipboard")
                        .clicked()
                    {
                        ui.ctx().copy_text(result.clone());
                    }
                });
            }
        });
    }

    fn load_device_keys(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::calc::crypt;
            use crate::calc::db;
            use crate::storage::models::crypt::get_current_device_keys;

            match db::establish_connection_result() {
                Ok(mut conn) => {
                    match get_current_device_keys(&mut conn) {
                        Ok(Some(keys)) => {
                            self.device_id = Some(keys.device_id.clone());
                            // Convert public key to base64 for display
                            self.device_pubkey = Some(crypt::encode_base64(&keys.public_key));
                            self.load_error = None;
                            dure_info!(" Loaded device keys for: {}", keys.device_id);
                        }
                        Ok(None) => {
                            self.load_error = Some(
                                "No device keys found. Please run 'dure key init' first."
                                    .to_string(),
                            );
                        }
                        Err(e) => {
                            self.load_error = Some(format!("Failed to load device keys: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.load_error = Some(format!("Database connection failed: {}", e));
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.load_error = Some("Device keys not available in WASM build".to_string());
        }
    }

    fn execute_encrypt(&mut self) {
        self.enc_error = None;
        self.enc_result = None;

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::calc::crypt;

            // Decode recipient public key
            let recipient_pubkey = match self.decode_input(&self.enc_recipient_pubkey) {
                Ok(key) => key,
                Err(e) => {
                    self.enc_error = Some(format!("Invalid recipient public key: {}", e));
                    return;
                }
            };

            // Encrypt the plaintext
            match crypt::encrypt(&recipient_pubkey, self.enc_plaintext.as_bytes()) {
                Ok(encrypted) => {
                    let encoded = if self.output_hex {
                        crypt::encode_hex(&encrypted)
                    } else {
                        crypt::encode_base64(&encrypted)
                    };
                    self.enc_result = Some(encoded);
                    dure_info!(" Encryption successful");
                }
                Err(e) => {
                    self.enc_error = Some(format!("Encryption failed: {}", e));
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.enc_error = Some("Encryption not available in WASM build".to_string());
        }
    }

    fn execute_decrypt(&mut self) {
        self.dec_error = None;
        self.dec_result = None;

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::calc::crypt;
            use crate::calc::db;
            use crate::storage::models::crypt::get_current_device_keys;

            // Get current device's private key
            let my_privkey = match db::establish_connection_result() {
                Ok(mut conn) => {
                    match get_current_device_keys(&mut conn) {
                        Ok(Some(keys)) => {
                            // Private key is already in binary form (Vec<u8>)
                            keys.private_key
                        }
                        Ok(None) => {
                            self.dec_error = Some(
                                "No device keys found. Please run 'dure key init' first."
                                    .to_string(),
                            );
                            return;
                        }
                        Err(e) => {
                            self.dec_error = Some(format!("Failed to load device keys: {}", e));
                            return;
                        }
                    }
                }
                Err(e) => {
                    self.dec_error = Some(format!("Database connection failed: {}", e));
                    return;
                }
            };

            // Decode encrypted data
            let encrypted = match self.decode_input(&self.dec_ciphertext) {
                Ok(data) => data,
                Err(e) => {
                    self.dec_error = Some(format!("Invalid ciphertext: {}", e));
                    return;
                }
            };

            // Decrypt the data
            match crypt::decrypt(&my_privkey, &encrypted) {
                Ok(decrypted) => match String::from_utf8(decrypted) {
                    Ok(plaintext) => {
                        self.dec_result = Some(plaintext);
                        dure_info!(" Decryption successful");
                    }
                    Err(_) => {
                        self.dec_error = Some("Decrypted data is not valid UTF-8 text".to_string());
                    }
                },
                Err(e) => {
                    self.dec_error = Some(format!("Decryption failed: {}", e));
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.dec_error = Some("Decryption not available in WASM build".to_string());
        }
    }

    /// Decode input from base64 or hex
    #[cfg(not(target_arch = "wasm32"))]
    fn decode_input(&self, input: &str) -> Result<Vec<u8>, String> {
        use crate::calc::crypt;

        let trimmed = input.trim();

        // Try base64 first
        if let Ok(decoded) = crypt::decode_base64(trimmed) {
            return Ok(decoded);
        }

        // Try hex
        if let Ok(decoded) = crypt::decode_hex(trimmed) {
            return Ok(decoded);
        }

        Err("Input must be valid base64 or hex encoding".to_string())
    }
}

// ─── Key Management sub-tab ──────────────────────────────────────────────────

/// Key Management sub-tab: Password manager with KeePass backend
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct KeyMgmtSubTab {
    #[cfg_attr(feature = "serde", serde(skip))]
    spreadsheet: Option<MaterialSpreadsheet>,
    #[cfg_attr(feature = "serde", serde(skip))]
    loaded: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    load_error: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    operation_message: Option<String>,

    // Add key form
    add_domain: String,
    add_username: String,
    add_password: String,

    // Delete key form
    del_domain: String,

    // Load/save paths
    load_path: String,
    save_path: String,
}

impl Default for KeyMgmtSubTab {
    fn default() -> Self {
        Self {
            spreadsheet: None,
            loaded: false,
            load_error: None,
            operation_message: None,
            add_domain: String::new(),
            add_username: String::new(),
            add_password: String::new(),
            del_domain: String::new(),
            load_path: String::new(),
            save_path: "exported_keys.kdbx".to_string(),
        }
    }
}

impl KeyMgmtSubTab {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Key Management");
        ui.add_space(4.0);
        ui.label("Password manager with KeePass backend (no database, file-based)");
        ui.add_space(8.0);

        // Display keyring paths
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::calc::keyring::{get_default_kdbx_path, get_default_kpkey_path};

            if let (Ok(kdbx_path), Ok(kpkey_path)) =
                (get_default_kdbx_path(), get_default_kpkey_path())
            {
                ui.group(|ui| {
                    ui.set_width(ui.available_width());
                    ui.label("Keyring Paths");
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("Database:");
                        ui.code(kdbx_path.display().to_string());
                    });
                    ui.horizontal(|ui| {
                        ui.label("KPKey:");
                        ui.code(kpkey_path.display().to_string());
                    });
                });
                ui.add_space(8.0);
            }
        }

        // Control buttons
        ui.horizontal(|ui| {
            if ui.add(MaterialButton::outlined("Refresh")).clicked() {
                self.load_keys();
            }

            ui.add_space(8.0);

            // Save button with path input
            ui.label("Save to:");
            ui.add(egui::TextEdit::singleline(&mut self.save_path).desired_width(150.0));
            if ui.add(MaterialButton::outlined("Save")).clicked() {
                self.execute_save();
            }

            ui.add_space(8.0);

            // Load button with path input
            ui.label("Load from:");
            ui.add(egui::TextEdit::singleline(&mut self.load_path).desired_width(150.0));
            if ui.add(MaterialButton::outlined("Load")).clicked() {
                self.execute_load();
            }
        });

        ui.add_space(8.0);

        // Operation message (success/error)
        if let Some(msg) = &self.operation_message {
            ui.colored_label(egui::Color32::from_rgb(100, 200, 100), format!("✓ {}", msg));
            ui.add_space(4.0);
        }

        // Load error
        if let Some(err) = &self.load_error {
            ui.colored_label(egui::Color32::RED, format!("⚠ {}", err));
            ui.add_space(4.0);
        }

        // Lazy-load keys on first render
        if !self.loaded {
            self.load_keys();
            self.loaded = true;
        }

        // Render spreadsheet
        if let Some(spreadsheet) = &mut self.spreadsheet {
            let rows_count = spreadsheet.rows().len();

            ui.group(|ui| {
                ui.set_min_height(300.0);
                ui.set_width(ui.available_width());
                spreadsheet.show(ui);
            });

            ui.add_space(10.0);
            ui.label(format!("Total keys: {}", rows_count));
        } else if self.load_error.is_none() {
            ui.colored_label(egui::Color32::YELLOW, "⚠ No keys loaded yet");
        }

        ui.add_space(12.0);

        // Add key form
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label("Add New Key");
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Domain:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.add_domain)
                        .hint_text("www.example.com")
                        .desired_width(200.0),
                );

                ui.add_space(8.0);
                ui.label("Username:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.add_username)
                        .hint_text("user@example.com")
                        .desired_width(200.0),
                );

                ui.add_space(8.0);
                ui.label("Password:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.add_password)
                        .password(true)
                        .hint_text("password")
                        .desired_width(150.0),
                );

                ui.add_space(8.0);
                if ui.add(MaterialButton::filled("Add")).clicked() {
                    self.execute_add();
                }
            });
        });

        ui.add_space(8.0);

        // Delete key form
        ui.group(|ui| {
            ui.set_width(ui.available_width());
            ui.label("Delete Key");
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Domain:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.del_domain)
                        .hint_text("www.example.com")
                        .desired_width(300.0),
                );

                ui.add_space(8.0);
                if ui.add(MaterialButton::outlined("Delete")).clicked() {
                    self.execute_delete();
                }
            });
        });
    }

    fn load_keys(&mut self) {
        self.load_error = None;
        self.operation_message = None;

        dure_info!(" Refreshing key management spreadsheet");

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::calc::keyring::{ensure_kdbx_exists, get_default_kpkey_path, list_keys};

            // Ensure database exists (auto-create if needed)
            let kdbx_path = match ensure_kdbx_exists() {
                Ok(path) => path,
                Err(e) => {
                    self.load_error = Some(format!("Failed to initialize keyring: {}", e));
                    return;
                }
            };

            let kpkey_path = match get_default_kpkey_path() {
                Ok(path) => path,
                Err(e) => {
                    self.load_error = Some(format!("Failed to get keyfile path: {}", e));
                    return;
                }
            };

            // List all keys
            match list_keys(&kdbx_path, Some(&kpkey_path)) {
                Ok(keys) => {
                    let mut data_rows = Vec::new();

                    for key in &keys {
                        let created = format_key_timestamp(key.created_at);
                        let last_mod = key
                            .last_modification
                            .map(|ts| format_key_timestamp(ts as u64))
                            .unwrap_or_else(|| "-".to_string());
                        let last_access = key
                            .last_access
                            .map(|ts| format_key_timestamp(ts as u64))
                            .unwrap_or_else(|| "-".to_string());
                        let notes = key.notes.as_deref().unwrap_or("-");
                        let ssh_key_status = if key.ssh_key.is_some() { "Yes" } else { "-" };

                        data_rows.push(vec![
                            key.domain.clone(),
                            key.username.clone(),
                            "••••••••".to_string(), // Hide password in display
                            created,
                            last_mod,
                            last_access,
                            notes.to_string(),
                            ssh_key_status.to_string(),
                        ]);
                    }

                    dure_info!(" Loaded {} keys from keyring", data_rows.len());

                    // Create spreadsheet
                    let columns = vec![
                        text_column("Domain", 200.0),
                        text_column("Username", 150.0),
                        text_column("Password", 80.0),
                        text_column("Created", 120.0),
                        text_column("LastModified", 120.0),
                        text_column("LastAccess", 120.0),
                        text_column("Notes", 150.0),
                        text_column("SSHKey", 60.0),
                    ];

                    match MaterialSpreadsheet::new("dure_keyring", columns) {
                        Ok(mut new_spreadsheet) => {
                            new_spreadsheet.set_striped(true);
                            new_spreadsheet.set_allow_selection(true);
                            new_spreadsheet.set_row_selection_enabled(true);
                            new_spreadsheet.init_with_data(data_rows);
                            self.spreadsheet = Some(new_spreadsheet);
                            dure_info!(" Spreadsheet refreshed with latest keys");
                        }
                        Err(e) => {
                            self.load_error = Some(format!("Spreadsheet error: {}", e));
                        }
                    }
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load keys: {}", e));
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.load_error = Some(
                "Key management not available in WASM build (desktop/Android only)".to_string(),
            );
        }
    }

    fn execute_add(&mut self) {
        self.operation_message = None;
        self.load_error = None;

        if self.add_domain.trim().is_empty() {
            self.load_error = Some("Domain cannot be empty".to_string());
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::calc::keyring::{add_key, get_default_kdbx_path, get_default_kpkey_path};

            let kdbx_path = match get_default_kdbx_path() {
                Ok(path) => path,
                Err(e) => {
                    self.load_error = Some(format!("Failed to get database path: {}", e));
                    return;
                }
            };

            let kpkey_path = match get_default_kpkey_path() {
                Ok(path) => path,
                Err(e) => {
                    self.load_error = Some(format!("Failed to get keyfile path: {}", e));
                    return;
                }
            };

            match add_key(
                &kdbx_path,
                Some(&kpkey_path),
                &self.add_domain,
                &self.add_username,
                &self.add_password,
            ) {
                Ok(()) => {
                    self.operation_message = Some(format!("Added key for '{}'", self.add_domain));
                    dure_info!(" Added key for: {}", self.add_domain);

                    // Clear form
                    self.add_domain.clear();
                    self.add_username.clear();
                    self.add_password.clear();

                    // Reload keys
                    self.load_keys();
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to add key: {}", e));
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.load_error = Some("Key management not available in WASM build".to_string());
        }
    }

    fn execute_delete(&mut self) {
        self.operation_message = None;
        self.load_error = None;

        if self.del_domain.trim().is_empty() {
            self.load_error = Some("Domain cannot be empty".to_string());
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::calc::keyring::{delete_key, get_default_kdbx_path, get_default_kpkey_path};

            let kdbx_path = match get_default_kdbx_path() {
                Ok(path) => path,
                Err(e) => {
                    self.load_error = Some(format!("Failed to get database path: {}", e));
                    return;
                }
            };

            let kpkey_path = match get_default_kpkey_path() {
                Ok(path) => path,
                Err(e) => {
                    self.load_error = Some(format!("Failed to get keyfile path: {}", e));
                    return;
                }
            };

            match delete_key(&kdbx_path, Some(&kpkey_path), &self.del_domain) {
                Ok(deleted) => {
                    if deleted {
                        self.operation_message =
                            Some(format!("Deleted key for '{}'", self.del_domain));
                        dure_info!(" Deleted key for: {}", self.del_domain);

                        // Clear form
                        self.del_domain.clear();

                        // Reload keys
                        self.load_keys();
                    } else {
                        self.load_error =
                            Some(format!("No key found with domain '{}'", self.del_domain));
                    }
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to delete key: {}", e));
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.load_error = Some("Key management not available in WASM build".to_string());
        }
    }

    fn execute_save(&mut self) {
        self.operation_message = None;
        self.load_error = None;

        if self.save_path.trim().is_empty() {
            self.load_error = Some("Save path cannot be empty".to_string());
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::calc::keyring::get_default_kdbx_path;
            use std::path::Path;

            let source_path = match get_default_kdbx_path() {
                Ok(path) => path,
                Err(e) => {
                    self.load_error = Some(format!("Failed to get database path: {}", e));
                    return;
                }
            };

            if !source_path.exists() {
                self.load_error = Some("No keyring found. Add keys first.".to_string());
                return;
            }

            let output_path = Path::new(&self.save_path);

            match std::fs::copy(&source_path, output_path) {
                Ok(_) => {
                    self.operation_message = Some(format!("Saved keyring to '{}'", self.save_path));
                    dure_info!(" Saved keyring to: {}", output_path.display());
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to save keyring: {}", e));
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.load_error = Some("Key management not available in WASM build".to_string());
        }
    }

    fn execute_load(&mut self) {
        self.operation_message = None;
        self.load_error = None;

        if self.load_path.trim().is_empty() {
            self.load_error = Some("Load path cannot be empty".to_string());
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::calc::keyring::get_default_kdbx_path;
            use std::path::Path;

            let input_path = Path::new(&self.load_path);

            if !input_path.exists() {
                self.load_error = Some(format!("File not found: '{}'", self.load_path));
                return;
            }

            let dest_path = match get_default_kdbx_path() {
                Ok(path) => path,
                Err(e) => {
                    self.load_error = Some(format!("Failed to get database path: {}", e));
                    return;
                }
            };

            // Ensure config directory exists
            if let Some(parent) = dest_path.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    self.load_error = Some(format!("Failed to create config directory: {}", e));
                    return;
                }
            }

            match std::fs::copy(input_path, &dest_path) {
                Ok(_) => {
                    self.operation_message =
                        Some(format!("Loaded keyring from '{}'", self.load_path));
                    dure_info!(" Loaded keyring from: {}", input_path.display());

                    // Clear load path
                    self.load_path.clear();

                    // Reload keys
                    self.load_keys();
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load keyring: {}", e));
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.load_error = Some("Key management not available in WASM build".to_string());
        }
    }
}

fn format_key_timestamp(timestamp: u64) -> String {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    let datetime = UNIX_EPOCH + Duration::from_secs(timestamp);
    let now = SystemTime::now();

    // Simple relative time formatting
    if let Ok(duration) = now.duration_since(datetime) {
        let days = duration.as_secs() / 86400;
        if days == 0 {
            return "Today".to_string();
        } else if days == 1 {
            return "Yesterday".to_string();
        } else if days < 7 {
            return format!("{} days ago", days);
        } else if days < 30 {
            return format!("{} weeks ago", days / 7);
        } else if days < 365 {
            return format!("{} months ago", days / 30);
        } else {
            return format!("{} years ago", days / 365);
        }
    }

    "Unknown".to_string()
}
