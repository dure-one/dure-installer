//! Mycart tab - Mycart admin settings management with data_table + drawer

use crate::ui_dlg::mycart::AddMycartDialog;
use crate::ui_dlg::mycart_crontab::{CrontabAction, CrontabDialog};
use crate::{dure_debug, dure_error, dure_info, dure_warn};
use crate::storage::models::opslog::{NewOperationLog, OperationStatus};
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
    InProgress {
        operation: String,
        started_at: i64,
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
    pub provider_id: Option<String>,
    pub host_ip: Option<String>,
}

/// Actions from mycart table rows
#[derive(Debug, Clone)]
enum MycartAction {
    OpenDrawer(String), // site_address
    Delete(String),     // site_address
    LoadStats(String),  // site_address
    LoadLogs(String),   // site_address
    // Phase 5: Crontab operations
    LoadCrontab(String), // site_address
    ToggleCronjob {
        site_address: String,
        job_id: String,
        enabled: bool,
    },
    SetCronjobInterval {
        site_address: String,
        job_id: String,
        interval: String,
    },
    UpdateCronjob {
        site_address: String,
        job_id: String,
        patch: crate::api::mycart::CrontabPatch,
    },
    // Phase 6: Service checks
    CheckDureMycart(String), // site_address
    CheckXmppProxy(String),  // site_address
    CheckFail2BanRS(String), // site_address
    // Phase 2: Crontab dialog
    OpenCrontabDialog(String), // site_address
}

/// Format Unix timestamp to readable date
fn format_created_at(timestamp: u64) -> String {
    use chrono::{DateTime, Utc};
    let dt = DateTime::<Utc>::from_timestamp(timestamp as i64, 0).unwrap_or_default();
    dt.format("%Y-%m-%d %H:%M").to_string()
}

/// Render site column with domain, platform_id, and ssh host
fn render_site_column(ui: &mut egui::Ui, row: &MycartRow) {
    use egui_material3::{badge, theme::get_global_color, BadgeColor, BadgeSize};

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 2.0;

        // Badges horizontal above domain
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            if row.platform_id_display != "—" {
                ui.add(
                    badge(&row.platform_id_display)
                        .color(BadgeColor::Primary)
                        .size(BadgeSize::Small),
                );
            }

            if let Some(provider) = &row.provider_id {
                ui.add(
                    badge(provider)
                        .color(BadgeColor::Primary)
                        .size(BadgeSize::Small),
                );
            }

            if let Some(host_ip) = &row.host_ip {
                ui.add(
                    badge(host_ip)
                        .color(BadgeColor::Primary)
                        .size(BadgeSize::Small),
                );
            }
        });

        // Domain (PRIMARY - big bold text)
        ui.label(
            egui::RichText::new(&row.domain)
                .size(16.0)
                .strong()
                .color(get_global_color("onSurface")),
        );
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
            if ui
                .add(MaterialButton::outlined("Crontab").small())
                .clicked()
            {
                *action_trigger = Some(MycartAction::OpenCrontabDialog(row.site_address.clone()));
            }

            if ui.add(MaterialButton::outlined("Delete").small()).clicked() {
                *action_trigger = Some(MycartAction::Delete(row.site_address.clone()));
            }
        });

        // Row 2: Service check buttons
        ui.horizontal(|ui| {
            if ui
                .add(MaterialButton::outlined("Check Mycart").small())
                .clicked()
            {
                *action_trigger = Some(MycartAction::CheckDureMycart(row.site_address.clone()));
            }

            if ui
                .add(MaterialButton::outlined("Check XMPPProxy").small())
                .clicked()
            {
                *action_trigger = Some(MycartAction::CheckXmppProxy(row.site_address.clone()));
            }

            if ui
                .add(MaterialButton::outlined("Check Fail2BanRS").small())
                .clicked()
            {
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
        &row.drawer_state.logs,
        row.drawer_state.logs_loading,
        &row.drawer_state.operations,
        false, // operations_loading
        &row.drawer_state.mycart_logs,
        row.drawer_state.mycart_loading,
        &row.drawer_state.xmppproxy_logs,
        row.drawer_state.xmppproxy_loading,
        &row.drawer_state.fail2banrs_logs,
        row.drawer_state.fail2banrs_loading,
    );

    // Handle tab switch (no auto-load - user clicks button to load)
    if let Some(new_tab) = tab_switch {
        // Persist tab switch (mirror platform.rs:1015)
        ui.data_mut(|d| d.insert_persisted(drawer_id, new_tab));

        // Backward compat: update row state
        row.drawer_state.active_tab = new_tab;
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
    add_dialog: Option<AddMycartDialog>,

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
            add_dialog: None,
            crontab_dialog: None,
        }
    }
}

impl MycartTab {
    pub fn ui(
        &mut self,
        current_profile: &Option<ProfileContext>,
        ui: &mut egui::Ui,
        mut vm: Option<&mut crate::viewmodel::ViewModel>,
        current_profile_kdbx: &Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) {
        // Poll for ViewModel events
        if let Some(ref mut viewmodel) = vm {
            let events = viewmodel.poll_events(ui.ctx());
            if !events.is_empty() {
                dure_info!("[UI::Mycart] 📬 Polled {} events from ViewModel", events.len());
            }
            for event in events {
                self.handle_event(event, current_profile);
            }
        }

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
                let mut dialog = AddMycartDialog::new();
                dialog.open();
                self.add_dialog = Some(dialog);
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
        self.render_table(current_profile, ui, table_height, vm.as_deref_mut(), current_profile_kdbx);

        // Add Mycart dialog
        #[cfg(not(target_arch = "wasm32"))]
        let add_action = if let Some(ref mut dialog) = self.add_dialog {
            let ssh_hosts = if let Some(config_path) = &self.config_path {
                let config = AppConfig::load_or_default(&config_path.to_path_buf());
                config.ssh_hosts
            } else {
                Vec::new()
            };
            dialog.show(ui.ctx(), &ssh_hosts)
        } else {
            crate::ui_dlg::mycart::AddMycartAction::None
        };

        #[cfg(target_arch = "wasm32")]
        let add_action = crate::ui_dlg::mycart::AddMycartAction::None;

        // Handle add dialog actions
        match add_action {
            crate::ui_dlg::mycart::AddMycartAction::Add {
                site_address,
                email,
                password,
                ssh_host,
                platform_id,
            } => {
                if let Some(ref mut dialog) = self.add_dialog {
                    dialog.set_in_progress(true);
                    dialog.set_status("Verifying credentials...".to_string());
                }

                // Execute async add operation
                if let Some(ref config_path) = self.config_path {
                    use crate::calc::mycart;
                    let platform_id_opt = if !platform_id.is_empty() {
                        Some(platform_id.as_str())
                    } else {
                        None
                    };
                    let ssh_host_opt = if !ssh_host.is_empty() {
                        Some(ssh_host.as_str())
                    } else {
                        None
                    };

                    let result = smol::block_on(mycart::add_mycart_site(
                        config_path,
                        &site_address,
                        &email,
                        &password,
                        platform_id_opt,
                        ssh_host_opt,
                    ));

                    if let Some(ref mut dialog) = self.add_dialog {
                        dialog.set_in_progress(false);
                        match result {
                            Ok(_) => {
                                dialog.set_status(format!("✓ Site '{}' added successfully", site_address));
                                self.loaded = false; // Trigger reload
                                // Close dialog after short delay
                                smol::block_on(async {
                                    smol::Timer::after(std::time::Duration::from_secs(1)).await;
                                });
                                self.add_dialog = None;
                            }
                            Err(e) => {
                                dialog.set_status(format!("✗ Error: {}", e));
                            }
                        }
                    }
                }
            }
            crate::ui_dlg::mycart::AddMycartAction::Cancel => {
                self.add_dialog = None;
            }
            crate::ui_dlg::mycart::AddMycartAction::None => {}
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
                            None,
                            current_profile_kdbx,
                        );
                        // Reload crontab to show updated state
                        self.handle_action(current_profile, MycartAction::OpenCrontabDialog(site), None, current_profile_kdbx);
                    }
                    CrontabAction::SetInterval { job_id, interval } => {
                        self.handle_action(
                            current_profile,
                            MycartAction::SetCronjobInterval {
                                site_address: site.clone(),
                                job_id,
                                interval,
                            },
                            None,
                            current_profile_kdbx,
                        );
                        // Reload crontab to show updated state
                        self.handle_action(current_profile, MycartAction::OpenCrontabDialog(site), None, current_profile_kdbx);
                    }
                    CrontabAction::UpdateJob { job_id, patch } => {
                        dure_info!(
                            "Mycart tab: Received UpdateJob action for site={}, job_id={}",
                            site,
                            job_id
                        );
                        self.handle_action(
                            current_profile,
                            MycartAction::UpdateCronjob {
                                site_address: site.clone(),
                                job_id,
                                patch,
                            },
                            None,
                            current_profile_kdbx,
                        );
                        // Reload crontab to show updated state
                        self.handle_action(current_profile, MycartAction::OpenCrontabDialog(site), None, current_profile_kdbx);
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
                                let platform =
                                    mycart::resolve_platform_for_site(&app_config, &site);
                                let ssh_host =
                                    mycart::resolve_ssh_host_for_site(&app_config, &site);

                                // Extract domain from site_address
                                let domain = site
                                    .site_address
                                    .split(':')
                                    .next()
                                    .unwrap_or(&site.site_address)
                                    .to_string();

                                // Format platform_id display
                                let platform_id_display = platform
                                    .and_then(|p| p.gcp_selected_project_id.as_ref())
                                    .map(|id| id.to_string())
                                    .unwrap_or_else(|| "—".to_string());

                                // Format SSH display (host:port)
                                let ssh_display = ssh_host
                                    .as_ref()
                                    .map(|h| {
                                        let host_part = h.host.split('@').last().unwrap_or(&h.host);
                                        format!("{}:{}", host_part, h.port)
                                    })
                                    .unwrap_or_else(|| "—".to_string());

                                // Extract host IP from ssh_display
                                let host_ip = if ssh_display != "—" {
                                    ssh_display.split(':').next().map(|s| s.to_string())
                                } else {
                                    None
                                };

                                // Resolve provider_id from domain
                                let provider_id = app_config
                                    .dns_domains
                                    .iter()
                                    .find(|d| {
                                        // Extract domain from site_address
                                        let site_domain = site
                                            .site_address
                                            .split(':')
                                            .next()
                                            .unwrap_or(&site.site_address);
                                        d.domain == site_domain
                                    })
                                    .map(|d| d.provider.clone());

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
                                    provider_id,
                                    host_ip,
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

    fn render_table(
        &mut self,
        current_profile: &Option<ProfileContext>,
        ui: &mut egui::Ui,
        _height: f32,
        vm: Option<&mut crate::viewmodel::ViewModel>,
        current_profile_kdbx: &Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) {
        // Calculate responsive column widths to fill available space
        let available_width = ui.available_width() - 40.0;
        let col1_width = 280.0_f32.max(available_width * 0.35);
        let col2_width = available_width - col1_width - 20.0; // 20px spacing

        // Build table with drawer support
        let table_id = egui::Id::new("mycart_sites_table");

        let mut table = data_table()
            .id(table_id)
            .allow_selection(false)
            .allow_drawer(true)
            .min_row_height(80.0)
            .drawer_row_height(500.0)
            .column("Site", col1_width, false)
            .column("Operations", col2_width, false);

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
        if let Some(action) =
            ui.data(|d| d.get_temp::<MycartAction>(egui::Id::new("mycart_pending_action")))
        {
            ui.data_mut(|d| d.remove::<MycartAction>(egui::Id::new("mycart_pending_action")));
            self.handle_action(current_profile, action, vm, current_profile_kdbx);
        }

        // Drawer action: Refresh operations
        if let Some(site_address) = ui.data(|d| {
            d.get_temp::<String>(egui::Id::new("drawer_action_refresh_operations"))
        }) {
            if let Some(row) = self.rows.iter_mut().find(|r| r.site_address == site_address) {
                // Load operations from DB
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Ok(mut conn) = crate::calc::db::establish_connection_result() {
                        match crate::storage::models::opslog::get_mycart_operations(
                            &mut conn,
                            &site_address,
                            100,
                        ) {
                            Ok(ops) => row.drawer_state.operations = ops,
                            Err(e) => {
                                dure_error!(
                                    "Failed to load operations for {}: {}",
                                    site_address,
                                    e
                                );
                            }
                        }
                    }
                }
            }
            ui.data_mut(|d| {
                d.remove::<String>(egui::Id::new("drawer_action_refresh_operations"))
            });
        }

        // Drawer action: Refresh logs
        if let Some(site_address) =
            ui.data(|d| d.get_temp::<String>(egui::Id::new("drawer_action_refresh_logs")))
        {
            if let Some(_row) = self.rows.iter_mut().find(|r| r.site_address == site_address) {
                // Load logs (placeholder - implement when log actor ready)
                // For now, just acknowledge the trigger
            }
            ui.data_mut(|d| d.remove::<String>(egui::Id::new("drawer_action_refresh_logs")));
        }
    }


    fn handle_event(
        &mut self,
        event: crate::viewmodel::common::ViewModelEvent,
        _current_profile: &Option<ProfileContext>,
    ) {
        use crate::viewmodel::mycart::events::MycartEvent;

        if let crate::viewmodel::common::ViewModelEvent::Mycart(mycart_event) = event {
            match mycart_event {
                MycartEvent::MycartLogsLoaded { site_address, logs } => {
                    dure_info!(project_id = site_address, "Mycart logs loaded for {}", site_address);
                    if let Some(row) = self.rows.iter_mut().find(|r| r.site_address == site_address) {
                        row.drawer_state.mycart_logs = logs;
                        row.drawer_state.mycart_loading = false;
                        row.drawer_state.active_tab = crate::viewmodel::mycart::DrawerTab::Mycart;
                        row.operation_state = OperationState::Completed {
                            operation: "Checking DureMycart".to_string(),
                            completed_at: chrono::Utc::now().timestamp(),
                        };
                    }
                }
                MycartEvent::XMPPProxyLogsLoaded { site_address, logs } => {
                    dure_info!(project_id = site_address, "XMPPProxy logs loaded for {}", site_address);
                    if let Some(row) = self.rows.iter_mut().find(|r| r.site_address == site_address) {
                        row.drawer_state.xmppproxy_logs = logs;
                        row.drawer_state.xmppproxy_loading = false;
                        row.drawer_state.active_tab = crate::viewmodel::mycart::DrawerTab::XMPPProxy;
                        row.operation_state = OperationState::Completed {
                            operation: "Checking XMPPProxy".to_string(),
                            completed_at: chrono::Utc::now().timestamp(),
                        };
                    }
                }
                MycartEvent::Fail2BanRSLogsLoaded { site_address, logs } => {
                    dure_info!(project_id = site_address, "Fail2BanRS logs loaded for {}", site_address);
                    if let Some(row) = self.rows.iter_mut().find(|r| r.site_address == site_address) {
                        row.drawer_state.fail2banrs_logs = logs;
                        row.drawer_state.fail2banrs_loading = false;
                        row.drawer_state.active_tab = crate::viewmodel::mycart::DrawerTab::Fail2BanRS;
                        row.operation_state = OperationState::Completed {
                            operation: "Checking Fail2BanRS".to_string(),
                            completed_at: chrono::Utc::now().timestamp(),
                        };
                    }
                }
                _ => {
                    // Other events not handled in UI yet
                }
            }
        }
    }

    fn handle_action(
        &mut self,
        current_profile: &Option<ProfileContext>,
        action: MycartAction,
        vm: Option<&mut crate::viewmodel::ViewModel>,
        current_profile_kdbx: &Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) {
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
                        dure_debug!("LoadCrontab: site={}, config_path={:?}", site, config_path);
                        match smol::block_on(mycart::get_site_crontab(config_path, &site)) {
                            Ok(jobs) => {
                                dure_debug!("LoadCrontab: loaded {} jobs", jobs.len());
                                if let Some(dialog) = &mut self.crontab_dialog {
                                    dialog.set_jobs(jobs);
                                }
                            }
                            Err(e) => {
                                dure_error!("LoadCrontab: failed with error: {:?}", e);
                                if let Some(dialog) = &mut self.crontab_dialog {
                                    dialog.set_error(format!("Failed to list crontab jobs: {}", e));
                                }
                            }
                        }
                    } else {
                        dure_error!("LoadCrontab: no config_path available");
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
                    if let (Some(config_path), Some(row)) = (
                        &self.config_path,
                        self.rows.iter_mut().find(|r| r.site_address == site),
                    ) {
                        row.operation_state = OperationState::InProgress {
                            operation: "Loading stats".to_string(),
                            started_at: chrono::Utc::now().timestamp(),
                        };

                        // Load stats and seller info in parallel
                        let stats_result =
                            smol::block_on(mycart::get_site_stats(config_path, &site));
                        let seller_result =
                            smol::block_on(mycart::get_seller_info(config_path, &site));

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
                    if let (Some(config_path), Some(row)) = (
                        &self.config_path,
                        self.rows.iter_mut().find(|r| r.site_address == site),
                    ) {
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
            MycartAction::ToggleCronjob {
                site_address,
                job_id,
                enabled,
            } => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let (Some(config_path), Some(row)) = (
                        &self.config_path,
                        self.rows
                            .iter_mut()
                            .find(|r| r.site_address == site_address),
                    ) {
                        match smol::block_on(mycart::update_site_cronjob_enabled(
                            config_path,
                            &site_address,
                            &job_id,
                            enabled,
                        )) {
                            Ok(()) => {
                                dure_info!("ToggleCronjob: Success - job list will be reloaded");
                                row.drawer_state.crontab_error = None;
                                // Job list will be reloaded by OpenCrontabDialog action
                            }
                            Err(e) => {
                                row.drawer_state.crontab_error = Some(e.to_string());
                            }
                        }
                    }
                }
            }
            MycartAction::SetCronjobInterval {
                site_address,
                job_id,
                interval,
            } => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let (Some(config_path), Some(row)) = (
                        &self.config_path,
                        self.rows
                            .iter_mut()
                            .find(|r| r.site_address == site_address),
                    ) {
                        match smol::block_on(mycart::update_site_cronjob_interval(
                            config_path,
                            &site_address,
                            &job_id,
                            &interval,
                        )) {
                            Ok(()) => {
                                dure_info!(
                                    "SetCronjobInterval: Success - job list will be reloaded"
                                );
                                row.drawer_state.crontab_error = None;
                                // Job list will be reloaded by OpenCrontabDialog action
                            }
                            Err(e) => {
                                row.drawer_state.crontab_error = Some(e.to_string());
                            }
                        }
                    }
                }
            }
            MycartAction::UpdateCronjob {
                site_address,
                job_id,
                patch,
            } => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let (Some(config_path), Some(row)) = (
                        &self.config_path,
                        self.rows
                            .iter_mut()
                            .find(|r| r.site_address == site_address),
                    ) {
                        dure_info!(
                            "UpdateCronjob: Calling API for site={}, job_id={}, patch enabled={:?}, interval={:?}",
                            site_address, job_id, patch.enabled, patch.interval
                        );

                        // Log operation
                        let log = NewOperationLog::new(&site_address, "crontab_patch", "mycart_api")
                            .status(OperationStatus::Running)
                            .details(format!("job_id={}", job_id));
                        if let Ok(mut conn) = crate::calc::db::establish_connection_result() {
                            let _ = crate::storage::models::opslog::create_log(&mut conn, log);
                        }

                        match smol::block_on(mycart::update_site_cronjob(
                            config_path,
                            &site_address,
                            &job_id,
                            patch,
                        )) {
                            Ok(()) => {
                                dure_info!("UpdateCronjob: Success - job list will be reloaded");
                                row.drawer_state.crontab_error = None;
                                // Job list will be reloaded by OpenCrontabDialog action
                            }
                            Err(e) => {
                                dure_error!("UpdateCronjob: Failed - {}", e);
                                row.drawer_state.crontab_error = Some(e.to_string());
                            }
                        }
                    }
                }
            }
            MycartAction::CheckDureMycart(site) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    // Log operation
                    let log = NewOperationLog::new(&site, "check_mycart", "ssh")
                        .status(OperationStatus::Running);
                    if let Ok(mut conn) = crate::calc::db::establish_connection_result() {
                        let _ = crate::storage::models::opslog::create_log(&mut conn, log);
                    }

                    if let Some(row) = self.rows.iter_mut().find(|r| r.site_address == site) {
                        row.drawer_state.mycart_loading = true;
                        row.operation_state = OperationState::InProgress {
                            operation: "Checking DureMycart".to_string(),
                            started_at: chrono::Utc::now().timestamp(),
                        };
                    }
                    if let Some(ref vm) = vm {
                        let _ = vm.load_mycart_logs(site.clone(), 100, (*current_profile_kdbx).clone());
                        dure_info!(project_id = site, "Mycart check triggered for {}", site);
                    }
                }
            }
            MycartAction::CheckXmppProxy(site) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    // Log operation
                    let log = NewOperationLog::new(&site, "check_xmppproxy", "ssh")
                        .status(OperationStatus::Running);
                    if let Ok(mut conn) = crate::calc::db::establish_connection_result() {
                        let _ = crate::storage::models::opslog::create_log(&mut conn, log);
                    }

                    if let Some(row) = self.rows.iter_mut().find(|r| r.site_address == site) {
                        row.drawer_state.xmppproxy_loading = true;
                        row.operation_state = OperationState::InProgress {
                            operation: "Checking XMPPProxy".to_string(),
                            started_at: chrono::Utc::now().timestamp(),
                        };
                    }
                    if let Some(ref vm) = vm {
                        let _ = vm.load_xmppproxy_logs(site.clone(), 100, (*current_profile_kdbx).clone());
                        dure_info!(project_id = site, "XMPPProxy check triggered for {}", site);
                    }
                }
            }
            MycartAction::CheckFail2BanRS(site) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    // Log operation
                    let log = NewOperationLog::new(&site, "check_fail2banrs", "ssh")
                        .status(OperationStatus::Running);
                    if let Ok(mut conn) = crate::calc::db::establish_connection_result() {
                        let _ = crate::storage::models::opslog::create_log(&mut conn, log);
                    }

                    if let Some(row) = self.rows.iter_mut().find(|r| r.site_address == site) {
                        row.drawer_state.fail2banrs_loading = true;
                        row.operation_state = OperationState::InProgress {
                            operation: "Checking Fail2BanRS".to_string(),
                            started_at: chrono::Utc::now().timestamp(),
                        };
                    }
                    if let Some(ref vm) = vm {
                        let _ = vm.load_fail2banrs_logs(site.clone(), 100, (*current_profile_kdbx).clone());
                        dure_info!(project_id = site, "Fail2BanRS check triggered for {}", site);
                    }
                }
            }
        }
    }
}
