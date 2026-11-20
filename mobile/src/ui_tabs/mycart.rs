//! Mycart tab - Mycart admin settings management with data_table + drawer

use crate::{dure_info, dure_warn};
use crate::ui_dlg::mycart_crontab::{CrontabDialog, CrontabAction};
use eframe::egui;
use egui_material3::{data_table, dialog, MaterialButton};
use std::cell::Cell;

#[cfg(not(target_arch = "wasm32"))]
use crate::calc::{mycart, profile::ProfileContext};
#[cfg(not(target_arch = "wasm32"))]
use crate::config::AppConfig;
#[cfg(not(target_arch = "wasm32"))]
use crate::viewmodel::mycart::{DrawerState, DrawerTab};

// Button spacing
const BUTTON_VERTICAL_SPACING: f32 = 4.0;
const BUTTON_HORIZONTAL_SPACING: f32 = 4.0;

/// Operation state for visual feedback
#[derive(Debug, Clone, PartialEq)]
pub enum OperationState {
    Idle,
    InProgress { operation: String, started_at: i64 },
    Completed { operation: String, completed_at: i64 },
    Failed { operation: String, error: String, failed_at: i64 },
}

impl Default for OperationState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Mycart row for data table
#[derive(Clone, Debug)]
pub struct MycartRow {
    pub site_address: String,
    pub email: String,
    pub created_at: u64,
    pub drawer_open: bool,
    pub drawer_state: DrawerState,
    pub seller_info: Option<crate::api::mycart::SellerInfo>,
    pub operation_state: OperationState,
    // Phase 2: Resolved display fields
    pub domain: String,
    pub platform_id_display: String,
    pub ssh_display: String,
}

/// Actions from mycart table rows
#[derive(Debug, Clone)]
enum MycartAction {
    OpenDrawer(String),     // site_address
    Delete(String),         // site_address
    LoadStats(String),      // site_address
    LoadLogs(String),       // site_address
    // Phase 5: Crontab operations
    LoadCrontab(String),    // site_address
    ToggleCronjob { site_address: String, job_id: String, enabled: bool },
    SetCronjobInterval { site_address: String, job_id: String, interval: String },
    // Phase 6: Service checks
    CheckDureMycart(String),    // site_address
    CheckXmppProxy(String),     // site_address
    CheckFail2BanRS(String),    // site_address
    // Phase 2: Crontab dialog
    OpenCrontabDialog(String),  // site_address
}

/// Format Unix timestamp to readable date
fn format_created_at(timestamp: u64) -> String {
    use chrono::{DateTime, Utc};
    let dt = DateTime::<Utc>::from_timestamp(timestamp as i64, 0).unwrap_or_default();
    dt.format("%Y-%m-%d %H:%M").to_string()
}

/// Render site column with domain, platform_id, and ssh host
fn render_site_column(ui: &mut egui::Ui, row: &MycartRow) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 2.0;

        // Line 1: Domain (strong)
        ui.label(egui::RichText::new(&row.domain).strong());

        // Line 2: Platform ID (small)
        ui.label(egui::RichText::new(format!("platform: {}", &row.platform_id_display)).small());

        // Line 3: SSH host:port (small, grey)
        ui.label(egui::RichText::new(format!("ssh: {}", &row.ssh_display)).small().color(egui::Color32::from_rgb(128, 128, 128)));
    });
}

/// Render operations column with action buttons
fn render_operations_column(
    ui: &mut egui::Ui,
    row: &MycartRow,
    action_trigger: &mut Option<MycartAction>,
) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = BUTTON_VERTICAL_SPACING;
        ui.spacing_mut().item_spacing.x = BUTTON_HORIZONTAL_SPACING;

        // Row 1: Crontab and Delete
        ui.horizontal(|ui| {
            if ui.add(MaterialButton::outlined("Crontab").small()).clicked() {
                *action_trigger = Some(MycartAction::OpenCrontabDialog(row.site_address.clone()));
            }

            if ui.add(MaterialButton::outlined("Delete").small()).clicked() {
                *action_trigger = Some(MycartAction::Delete(row.site_address.clone()));
            }
        });

        // Row 2: Service check buttons
        ui.horizontal(|ui| {
            if ui.add(MaterialButton::outlined("Check Mycart").small()).clicked() {
                *action_trigger = Some(MycartAction::CheckDureMycart(row.site_address.clone()));
            }

            if ui.add(MaterialButton::outlined("Check XMPPProxy").small()).clicked() {
                *action_trigger = Some(MycartAction::CheckXmppProxy(row.site_address.clone()));
            }

            if ui.add(MaterialButton::outlined("Check Fail2BanRS").small()).clicked() {
                *action_trigger = Some(MycartAction::CheckFail2BanRS(row.site_address.clone()));
            }
        });
    });
}

/// Render drawer for a specific row
fn render_drawer_for_row(
    ui: &mut egui::Ui,
    row: &mut MycartRow,
    action_trigger: &mut Option<MycartAction>,
) {
    use crate::ui_tabs::mycart_drawer;

    // Create unique drawer ID for persistent tab storage (mirror platform.rs:977)
    let drawer_id = egui::Id::new("mycart_drawer").with(&row.site_address);

    // Load active tab from persistent storage (mirror platform.rs:977)
    let active_tab: DrawerTab =
        ui.data_mut(|d| d.get_persisted(drawer_id).unwrap_or(DrawerTab::Status));

    let mut tab_switch: Option<DrawerTab> = None;
    let mut drawer_action: Option<mycart_drawer::DrawerAction> = None;

    mycart_drawer::render_drawer(
        ui,
        &row.site_address,
        &row.drawer_state.stats,
        &row.seller_info,
        &active_tab,
        &mut tab_switch,
        &row.drawer_state.crontab_jobs,
        row.drawer_state.crontab_loading,
        &row.drawer_state.crontab_error,
        &mut drawer_action,
        &row.drawer_state.mycart_logs,
        row.drawer_state.mycart_loading,
        &row.drawer_state.xmppproxy_logs,
        row.drawer_state.xmppproxy_loading,
        &row.drawer_state.fail2banrs_logs,
        row.drawer_state.fail2banrs_loading,
    );

    // Handle tab switch
    if let Some(new_tab) = tab_switch {
        // Persist tab switch (mirror platform.rs:1015)
        ui.data_mut(|d| d.insert_persisted(drawer_id, new_tab));

        // Backward compat: update row state
        row.drawer_state.active_tab = new_tab;

        // Load data when switching tabs
        match new_tab {
            DrawerTab::Status => {
                *action_trigger = Some(MycartAction::LoadStats(row.site_address.clone()));
            }
            DrawerTab::Operations => {
                *action_trigger = Some(MycartAction::LoadCrontab(row.site_address.clone()));
            }
            DrawerTab::Mycart => {
                *action_trigger = Some(MycartAction::CheckDureMycart(row.site_address.clone()));
            }
            DrawerTab::XMPPProxy => {
                *action_trigger = Some(MycartAction::CheckXmppProxy(row.site_address.clone()));
            }
            DrawerTab::Fail2BanRS => {
                *action_trigger = Some(MycartAction::CheckFail2BanRS(row.site_address.clone()));
            }
        }
    }

    // Handle drawer actions
    if let Some(action) = drawer_action {
        match action {
            mycart_drawer::DrawerAction::ToggleCronjob { job_id, enabled } => {
                *action_trigger = Some(MycartAction::ToggleCronjob {
                    site_address: row.site_address.clone(),
                    job_id,
                    enabled,
                });
            }
            mycart_drawer::DrawerAction::SetCronjobInterval { job_id, interval } => {
                *action_trigger = Some(MycartAction::SetCronjobInterval {
                    site_address: row.site_address.clone(),
                    job_id,
                    interval,
                });
            }
        }
    }
}

/// Mycart tab state
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct MycartTab {
    #[cfg_attr(feature = "serde", serde(skip))]
    rows: Vec<MycartRow>,
    #[cfg_attr(feature = "serde", serde(skip))]
    loaded: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    load_error: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    config_path: Option<std::path::PathBuf>,

    // Add dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_add_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_site: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_email: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_password: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_id: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_ssh_host: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_status: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_in_progress: bool,

    // Crontab dialog (Phase 3)
    #[cfg_attr(feature = "serde", serde(skip))]
    crontab_dialog: Option<CrontabDialog>,
}

impl Default for MycartTab {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            loaded: false,
            load_error: None,
            config_path: None,
            show_add_dialog: false,
            add_site: String::new(),
            add_email: String::new(),
            add_password: String::new(),
            add_platform_id: String::new(),
            add_ssh_host: String::new(),
            add_status: String::new(),
            add_in_progress: false,
            crontab_dialog: None,
        }
    }
}

impl MycartTab {
    pub fn ui(&mut self, current_profile: &Option<ProfileContext>, ui: &mut egui::Ui) {
        if current_profile.is_none() {
            ui.label("No profile selected");
            return;
        }

        // Store config path for async operations
        self.config_path = Some(current_profile.as_ref().unwrap().config_file.clone());

        ui.heading("Dure-Mycart");
        ui.add_space(4.0);
        ui.label("Manage Mycart admin settings with mycart API access.");
        ui.add_space(8.0);

        // Action buttons
        ui.horizontal(|ui| {
            if ui.add(MaterialButton::filled("Add Mycart")).clicked() {
                self.show_add_dialog = true;
                self.add_site.clear();
                self.add_email.clear();
                self.add_password.clear();
                self.add_status.clear();
            }

            if ui.add(MaterialButton::outlined("Refresh")).clicked() {
                self.loaded = false;
                self.load_error = None;
            }
        });
        ui.add_space(8.0);

        // Lazy load
        if !self.loaded {
            self.load_rows();
            self.loaded = true;
        }

        // Error display
        if let Some(err) = &self.load_error {
            ui.colored_label(egui::Color32::from_rgb(255, 0, 0), format!("⚠ {}", err));
            ui.add_space(4.0);
        }

        // Data table
        let table_height = ui.available_height() - 20.0;
        self.render_table(current_profile, ui, table_height);

        // Dialogs
        if self.show_add_dialog {
            self.render_add_dialog(ui.ctx());
        }

        // Crontab dialog (Phase 3)
        let crontab_action = if let Some(ref mut dialog) = self.crontab_dialog {
            let mut show_dialog = true;
            let action = dialog.render(ui.ctx(), &mut show_dialog);

            if !show_dialog {
                self.crontab_dialog = None;
            }

            action
        } else {
            None
        };

        // Handle crontab dialog actions outside the borrow
        if let Some(action) = crontab_action {
            if let Some(ref dialog) = self.crontab_dialog {
                let site = dialog.site_address.clone();

                match action {
                    CrontabAction::ToggleJob { job_id, enabled } => {
                        self.handle_action(
                            current_profile,
                            MycartAction::ToggleCronjob {
                                site_address: site.clone(),
                                job_id,
                                enabled,
                            },
                        );
                        // Reload crontab to show updated state
                        self.handle_action(
                            current_profile,
                            MycartAction::OpenCrontabDialog(site),
                        );
                    }
                    CrontabAction::SetInterval { job_id, interval } => {
                        self.handle_action(
                            current_profile,
                            MycartAction::SetCronjobInterval {
                                site_address: site.clone(),
                                job_id,
                                interval,
                            },
                        );
                        // Reload crontab to show updated state
                        self.handle_action(
                            current_profile,
                            MycartAction::OpenCrontabDialog(site),
                        );
                    }
                }
            }
        }
    }

    fn load_rows(&mut self) {
        self.rows.clear();
        self.load_error = None;

        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(config_path) = &self.config_path {
                // Load full config for resolver
                let app_config = AppConfig::load_or_default(&config_path.to_path_buf());

                match mycart::list_mycart_sites(config_path) {
                    Ok(sites) => {
                        self.rows = sites
                            .into_iter()
                            .map(|site| {
                                // Resolve platform and SSH host
                                let platform = mycart::resolve_platform_for_site(&app_config, &site);
                                let ssh_host = mycart::resolve_ssh_host_for_site(&app_config, &site);

                                // Extract domain from site_address
                                let domain = site.site_address.split(':').next().unwrap_or(&site.site_address).to_string();

                                // Format platform_id display
                                let platform_id_display = platform
                                    .and_then(|p| p.gcp_selected_project_id.as_ref())
                                    .map(|id| id.to_string())
                                    .unwrap_or_else(|| "—".to_string());

                                // Format SSH display (host:port)
                                let ssh_display = ssh_host
                                    .map(|h| {
                                        let host_part = h.host.split('@').last().unwrap_or(&h.host);
                                        format!("{}:{}", host_part, h.port)
                                    })
                                    .unwrap_or_else(|| "—".to_string());

                                MycartRow {
                                    site_address: site.site_address,
                                    email: site.email,
                                    created_at: site.created_at,
                                    drawer_open: false,
                                    drawer_state: DrawerState::default(),
                                    seller_info: None,
                                    operation_state: OperationState::Idle,
                                    domain,
                                    platform_id_display,
                                    ssh_display,
                                }
                            })
                            .collect();
                    }
                    Err(e) => {
                        self.load_error = Some(format!("Failed to load sites: {}", e));
                    }
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.load_error = Some("Mycart management not available on WASM".to_string());
        }
    }

    fn render_table(&mut self, current_profile: &Option<ProfileContext>, ui: &mut egui::Ui, _height: f32) {
        // Calculate responsive column widths
        let available_width = ui.available_width() - 40.0;
        let base_width = 740.0; // Base width for columns
        let width_ratio = (available_width / base_width).min(1.5).max(0.8);

        // Build table with drawer support
        let table_id = egui::Id::new("mycart_sites_table");

        let mut table = data_table()
            .id(table_id)
            .allow_selection(false)
            .allow_drawer(true)
            .min_row_height(80.0)
            .drawer_row_height(500.0)
            .column("Site", 280.0 * width_ratio, false)
            .column("Operations", 460.0 * width_ratio, false);

        for row in self.rows.iter() {
            let row_for_site = row.clone();
            let row_for_ops = row.clone();
            let row_for_drawer = row.clone();

            table = table.row(move |r| {
                r.cell_widget(move |ui| {
                    render_site_column(ui, &row_for_site);
                })
                .cell_widget(move |ui| {
                    let mut local_action = None;
                    render_operations_column(ui, &row_for_ops, &mut local_action);
                    if let Some(action) = local_action {
                        ui.data_mut(|d| {
                            d.insert_temp(egui::Id::new("mycart_pending_action"), action);
                        });
                    }
                })
                .drawer(move |ui| {
                    let mut local_action = None;
                    render_drawer_for_row(ui, &mut row_for_drawer.clone(), &mut local_action);
                    if let Some(action) = local_action {
                        ui.data_mut(|d| {
                            d.insert_temp(egui::Id::new("mycart_pending_action"), action);
                        });
                    }
                })
            });
        }

        table.show(ui);

        // Handle pending actions
        if let Some(action) = ui.data(|d| d.get_temp::<MycartAction>(egui::Id::new("mycart_pending_action"))) {
            ui.data_mut(|d| d.remove::<MycartAction>(egui::Id::new("mycart_pending_action")));
            self.handle_action(current_profile, action);
        }
    }

    fn render_add_dialog(&mut self, ctx: &egui::Context) {
        let add_clicked = Cell::new(false);
        let cancel_clicked = Cell::new(false);

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog("add_mycart_dialog", "Add Mycart Site", &mut self.show_add_dialog)
            .min_width(400.0)
            .max_width(window_width - 40.0)
            .max_height(window_height - 40.0)
            .scrollable(true)
            .content(|ui, _open| {
                ui.label("Add a new mycart site with admin credentials:");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Site Address:");
                    ui.text_edit_singleline(&mut self.add_site);
                });

                ui.horizontal(|ui| {
                    ui.label("Admin Email:");
                    ui.text_edit_singleline(&mut self.add_email);
                });

                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.add(egui::TextEdit::singleline(&mut self.add_password).password(true));
                });

                ui.horizontal(|ui| {
                    ui.label("Platform ID:");
                    ui.text_edit_singleline(&mut self.add_platform_id);
                });

                ui.horizontal(|ui| {
                    ui.label("SSH Host:");
                    ui.text_edit_singleline(&mut self.add_ssh_host);
                });

                if !self.add_status.is_empty() {
                    ui.add_space(8.0);
                    let color = if self.add_status.contains("success") {
                        egui::Color32::from_rgb(0, 200, 0)
                    } else {
                        egui::Color32::from_rgb(255, 0, 0)
                    };
                    ui.colored_label(color, &self.add_status);
                }

                if self.add_in_progress {
                    ui.add_space(8.0);
                    ui.spinner();
                    ui.label("Verifying credentials...");
                }

                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(!self.add_in_progress, MaterialButton::filled("Add"))
                        .clicked()
                    {
                        add_clicked.set(true);
                    }

                    if ui.add(MaterialButton::text("Cancel")).clicked() {
                        cancel_clicked.set(true);
                    }
                });
            })
            .show(ctx);

        if add_clicked.get() && !self.add_in_progress {
            self.execute_add_site();
        }

        if cancel_clicked.get() {
            self.show_add_dialog = false;
        }
    }

    fn execute_add_site(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if self.add_site.is_empty() || self.add_email.is_empty() || self.add_password.is_empty() {
                self.add_status = "All fields are required".to_string();
                return;
            }

            let Some(config_path) = &self.config_path else {
                self.add_status = "No profile selected".to_string();
                return;
            };

            let config_path = config_path.clone();
            let site = self.add_site.clone();
            let email = self.add_email.clone();
            let password = self.add_password.clone();
            let platform_id = (!self.add_platform_id.is_empty()).then(|| self.add_platform_id.as_str());
            let ssh_host = (!self.add_ssh_host.is_empty()).then(|| self.add_ssh_host.as_str());

            self.add_in_progress = true;
            self.add_status.clear();

            // Spawn async task
            let result = smol::block_on(mycart::add_mycart_site(&config_path, &site, &email, &password, platform_id, ssh_host));

            self.add_in_progress = false;

            match result {
                Ok(()) => {
                    self.add_status = "✓ Site added successfully".to_string();
                    self.loaded = false; // Trigger reload
                    // Clear form fields
                    self.add_site.clear();
                    self.add_email.clear();
                    self.add_password.clear();
                    self.add_platform_id.clear();
                    self.add_ssh_host.clear();
                    // Auto-close after success
                    self.show_add_dialog = false;
                }
                Err(e) => {
                    self.add_status = format!("✗ Failed: {}", e);
                }
            }
        }
    }

    fn handle_action(&mut self, current_profile: &Option<ProfileContext>, action: MycartAction) {
        match action {
            MycartAction::OpenDrawer(site) => {
                if let Some(row) = self.rows.iter_mut().find(|r| r.site_address == site) {
                    row.drawer_open = !row.drawer_open;
                }
            }
            MycartAction::OpenCrontabDialog(site) => {
                // Create dialog and trigger load
                self.crontab_dialog = Some(CrontabDialog::new(site.clone()));

                // Load crontab jobs
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(config_path) = &self.config_path {
                        match smol::block_on(mycart::get_site_crontab(config_path, &site)) {
                            Ok(jobs) => {
                                if let Some(dialog) = &mut self.crontab_dialog {
                                    dialog.set_jobs(jobs);
                                }
                            }
                            Err(e) => {
                                if let Some(dialog) = &mut self.crontab_dialog {
                                    dialog.set_error(e.to_string());
                                }
                            }
                        }
                    }
                }
            }
            MycartAction::Delete(site) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(config_path) = &self.config_path {
                        if let Err(e) = mycart::delete_mycart_site(config_path, &site) {
                            self.load_error = Some(format!("Failed to delete site: {}", e));
                        } else {
                            dure_info!("Deleted mycart site: {}", site);
                            self.loaded = false; // Trigger reload
                        }
                    }
                }
            }
            MycartAction::LoadStats(site) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let (Some(config_path), Some(row)) = (&self.config_path, self.rows.iter_mut().find(|r| r.site_address == site)) {
                        row.operation_state = OperationState::InProgress {
                            operation: "Loading stats".to_string(),
                            started_at: chrono::Utc::now().timestamp(),
                        };

                        // Load stats and seller info in parallel
                        let stats_result = smol::block_on(mycart::get_site_stats(config_path, &site));
                        let seller_result = smol::block_on(mycart::get_seller_info(config_path, &site));

                        match (stats_result, seller_result) {
                            (Ok(stats), Ok(seller)) => {
                                row.drawer_state.stats = Some(stats);
                                row.seller_info = Some(seller);
                                row.operation_state = OperationState::Completed {
                                    operation: "Loading stats".to_string(),
                                    completed_at: chrono::Utc::now().timestamp(),
                                };
                            }
                            (Err(e), _) | (_, Err(e)) => {
                                row.operation_state = OperationState::Failed {
                                    operation: "Loading stats".to_string(),
                                    error: e.to_string(),
                                    failed_at: chrono::Utc::now().timestamp(),
                                };
                            }
                        }
                    }
                }
            }
            // ponytail: LoadLogs removed - replaced by service-specific Check actions (Phase 4)
            MycartAction::LoadLogs(_site) => {}
            MycartAction::LoadCrontab(site) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let (Some(config_path), Some(row)) = (&self.config_path, self.rows.iter_mut().find(|r| r.site_address == site)) {
                        row.drawer_state.crontab_loading = true;
                        row.drawer_state.crontab_error = None;

                        match smol::block_on(mycart::get_site_crontab(config_path, &site)) {
                            Ok(jobs) => {
                                row.drawer_state.crontab_jobs = jobs;
                                row.drawer_state.crontab_loading = false;
                                row.drawer_state.active_tab = DrawerTab::Operations;
                            }
                            Err(e) => {
                                row.drawer_state.crontab_loading = false;
                                row.drawer_state.crontab_error = Some(e.to_string());
                            }
                        }
                    }
                }
            }
            MycartAction::ToggleCronjob { site_address, job_id, enabled } => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let (Some(config_path), Some(row)) = (&self.config_path, self.rows.iter_mut().find(|r| r.site_address == site_address)) {
                        match smol::block_on(mycart::update_site_cronjob_enabled(config_path, &site_address, &job_id, enabled)) {
                            Ok(updated_job) => {
                                // Update the job in the list
                                if let Some(job) = row.drawer_state.crontab_jobs.iter_mut().find(|j| j.job_id == job_id) {
                                    *job = updated_job;
                                }
                            }
                            Err(e) => {
                                row.drawer_state.crontab_error = Some(e.to_string());
                            }
                        }
                    }
                }
            }
            MycartAction::SetCronjobInterval { site_address, job_id, interval } => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let (Some(config_path), Some(row)) = (&self.config_path, self.rows.iter_mut().find(|r| r.site_address == site_address)) {
                        match smol::block_on(mycart::update_site_cronjob_interval(config_path, &site_address, &job_id, &interval)) {
                            Ok(updated_job) => {
                                // Update the job in the list
                                if let Some(job) = row.drawer_state.crontab_jobs.iter_mut().find(|j| j.job_id == job_id) {
                                    *job = updated_job;
                                }
                                row.drawer_state.crontab_error = None;
                            }
                            Err(e) => {
                                row.drawer_state.crontab_error = Some(e.to_string());
                            }
                        }
                    }
                }
            }
            MycartAction::CheckDureMycart(site) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(row) = self.rows.iter_mut().find(|r| r.site_address == site) {
                        row.drawer_state.mycart_loading = true;
                        row.operation_state = OperationState::InProgress {
                            operation: "Checking DureMycart".to_string(),
                            started_at: chrono::Utc::now().timestamp(),
                        };

                        if let Some(profile) = current_profile {
                            let config = AppConfig::load_or_default(&profile.config_file);
                            let ssh_host = config.ssh_hosts.iter().find(|h| h.host.contains(&site));

                            if let Some(host_config) = ssh_host {
                                match smol::block_on(mycart::get_duremycart_logs(host_config, None)) {
                                    Ok(logs) => {
                                        row.drawer_state.mycart_logs.extend(logs);
                                        row.drawer_state.mycart_loading = false;
                                        row.drawer_state.active_tab = DrawerTab::Mycart;
                                        row.operation_state = OperationState::Completed {
                                            operation: "Checking DureMycart".to_string(),
                                            completed_at: chrono::Utc::now().timestamp(),
                                        };
                                    }
                                    Err(e) => {
                                        row.drawer_state.mycart_loading = false;
                                        row.operation_state = OperationState::Failed {
                                            operation: "Checking DureMycart".to_string(),
                                            error: e.to_string(),
                                            failed_at: chrono::Utc::now().timestamp(),
                                        };
                                    }
                                }
                            } else {
                                row.drawer_state.mycart_loading = false;
                                row.operation_state = OperationState::Failed {
                                    operation: "Checking DureMycart".to_string(),
                                    error: format!("No SSH host configured for site: {}", site),
                                    failed_at: chrono::Utc::now().timestamp(),
                                };
                            }
                        } else {
                            row.drawer_state.mycart_loading = false;
                            row.operation_state = OperationState::Failed {
                                operation: "Checking DureMycart".to_string(),
                                error: "No profile selected".to_string(),
                                failed_at: chrono::Utc::now().timestamp(),
                            };
                        }
                    }
                }
            }
            MycartAction::CheckXmppProxy(site) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(row) = self.rows.iter_mut().find(|r| r.site_address == site) {
                        row.drawer_state.xmppproxy_loading = true;
                        row.operation_state = OperationState::InProgress {
                            operation: "Checking XmppProxy".to_string(),
                            started_at: chrono::Utc::now().timestamp(),
                        };

                        if let Some(profile) = current_profile {
                            let config = AppConfig::load_or_default(&profile.config_file);
                            let ssh_host = config.ssh_hosts.iter().find(|h| h.host.contains(&site));

                            if let Some(host_config) = ssh_host {
                                match smol::block_on(mycart::get_xmppproxy_logs(host_config, None)) {
                                    Ok(logs) => {
                                        row.drawer_state.xmppproxy_logs.extend(logs);
                                        row.drawer_state.xmppproxy_loading = false;
                                        row.drawer_state.active_tab = DrawerTab::XMPPProxy;
                                        row.operation_state = OperationState::Completed {
                                            operation: "Checking XmppProxy".to_string(),
                                            completed_at: chrono::Utc::now().timestamp(),
                                        };
                                    }
                                    Err(e) => {
                                        row.drawer_state.xmppproxy_loading = false;
                                        row.operation_state = OperationState::Failed {
                                            operation: "Checking XmppProxy".to_string(),
                                            error: e.to_string(),
                                            failed_at: chrono::Utc::now().timestamp(),
                                        };
                                    }
                                }
                            } else {
                                row.drawer_state.xmppproxy_loading = false;
                                row.operation_state = OperationState::Failed {
                                    operation: "Checking XmppProxy".to_string(),
                                    error: format!("No SSH host configured for site: {}", site),
                                    failed_at: chrono::Utc::now().timestamp(),
                                };
                            }
                        } else {
                            row.drawer_state.xmppproxy_loading = false;
                            row.operation_state = OperationState::Failed {
                                operation: "Checking XmppProxy".to_string(),
                                error: "No profile selected".to_string(),
                                failed_at: chrono::Utc::now().timestamp(),
                            };
                        }
                    }
                }
            }
            MycartAction::CheckFail2BanRS(site) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Some(row) = self.rows.iter_mut().find(|r| r.site_address == site) {
                        row.drawer_state.fail2banrs_loading = true;
                        row.operation_state = OperationState::InProgress {
                            operation: "Checking Fail2BanRS".to_string(),
                            started_at: chrono::Utc::now().timestamp(),
                        };

                        if let Some(profile) = current_profile {
                            let config = AppConfig::load_or_default(&profile.config_file);
                            let ssh_host = config.ssh_hosts.iter().find(|h| h.host.contains(&site));

                            if let Some(host_config) = ssh_host {
                                match smol::block_on(mycart::get_fail2banrs_logs(host_config, None)) {
                                    Ok(logs) => {
                                        row.drawer_state.fail2banrs_logs.extend(logs);
                                        row.drawer_state.fail2banrs_loading = false;
                                        row.drawer_state.active_tab = DrawerTab::Fail2BanRS;
                                        row.operation_state = OperationState::Completed {
                                            operation: "Checking Fail2BanRS".to_string(),
                                            completed_at: chrono::Utc::now().timestamp(),
                                        };
                                    }
                                    Err(e) => {
                                        row.drawer_state.fail2banrs_loading = false;
                                        row.operation_state = OperationState::Failed {
                                            operation: "Checking Fail2BanRS".to_string(),
                                            error: e.to_string(),
                                            failed_at: chrono::Utc::now().timestamp(),
                                        };
                                    }
                                }
                            } else {
                                row.drawer_state.fail2banrs_loading = false;
                                row.operation_state = OperationState::Failed {
                                    operation: "Checking Fail2BanRS".to_string(),
                                    error: format!("No SSH host configured for site: {}", site),
                                    failed_at: chrono::Utc::now().timestamp(),
                                };
                            }
                        } else {
                            row.drawer_state.fail2banrs_loading = false;
                            row.operation_state = OperationState::Failed {
                                operation: "Checking Fail2BanRS".to_string(),
                                error: "No profile selected".to_string(),
                                failed_at: chrono::Utc::now().timestamp(),
                            };
                        }
                    }
                }
            }
        }
    }
}
