//! SSH drawer UI with Status/Logs/Operations/Host/Docker/Dure tabs

use eframe::egui;

use crate::storage::models::opslog::OperationLog;
use crate::ui_components::drawer::{StatusLine, TabBar};
use crate::ui_tabs::ssh::{SshRow, OperationState};
use crate::viewmodel::ssh::{DrawerTab, DrawerState};

/// Render the SSH drawer with tabs
pub fn render_drawer(
    ui: &mut egui::Ui,
    row: &SshRow,
    drawer_state: &DrawerState,
    on_tab_switch: &mut Option<DrawerTab>,
) {
    // Tab bar (using shared component)
    let tab_bar = TabBar::<DrawerTab>::new();
    if let Some(new_tab) = tab_bar.show(ui, drawer_state.active_tab) {
        *on_tab_switch = Some(new_tab);
    }

    ui.separator();

    // Content area (500px from recent commit)
    egui::ScrollArea::vertical()
        .max_height(500.0)
        .show(ui, |ui| {
            match drawer_state.active_tab {
                DrawerTab::Status => render_status_tab(ui, row),
                DrawerTab::Logs => render_logs_tab(ui, drawer_state),
                DrawerTab::Operations => render_operations_tab(ui, drawer_state),
                DrawerTab::Host => render_host_tab(ui, drawer_state),
                DrawerTab::Docker => render_docker_tab(ui, drawer_state),
                DrawerTab::Dure => render_dure_tab(ui, drawer_state),
            }
        });
}

/// Render Status tab
fn render_status_tab(ui: &mut egui::Ui, row: &SshRow) {
    use crate::ui_components::ActionMenu;

    ui.add_space(8.0);

    // IP Address (no emoji)
    StatusLine::new("IP", &row.host).show(ui);

    // SSH Connection Status (inline error display)
    if let Some(check_result) = &row.last_ssh_check {
        if check_result.connected {
            StatusLine::new("SSH", "Connected").show(ui);
        } else if let Some(error) = &check_result.error {
            StatusLine::new("SSH", &format!("Failed ({})", error))
                .error()
                .show(ui);
        } else {
            StatusLine::new("SSH", "Disconnected")
                .warning()
                .show(ui);
        }
    } else {
        StatusLine::new("SSH", "Not checked").show(ui);
    }

    // Base Packages Status
    let base_status = if row.base_installed { "Installed" } else { "Not Installed" };
    let mut base_line = StatusLine::new("Base", base_status);
    if !row.base_installed {
        base_line = base_line.warning();
    }
    base_line.show(ui);

    // Show missing packages
    if let Some(check_result) = &row.last_base_check {
        if !check_result.missing_packages.is_empty() {
            ui.label(format!("  Missing: {}", check_result.missing_packages.join(", ")));
        }
    }

    // Docker Status
    let docker_status = if row.docker_installed { "Installed" } else { "Not Installed" };
    let mut docker_line = StatusLine::new("Docker", docker_status);
    if !row.docker_installed {
        docker_line = docker_line.warning();
    }
    docker_line.show(ui);

    // Show docker version
    if let Some(check_result) = &row.last_docker_check {
        if let Some(version) = &check_result.version {
            ui.label(format!("  Version: {}", version));
        }
    }

    // Dure Status
    let dure_status = if row.dure_installed { "Installed" } else { "Not Installed" };
    let mut dure_line = StatusLine::new("Dure", dure_status);
    if !row.dure_installed {
        dure_line = dure_line.warning();
    }
    dure_line.show(ui);

    ui.add_space(8.0);

    // Operation State
    match &row.operation_state {
        OperationState::InProgress { operation, started_at } => {
            let elapsed = chrono::Utc::now().timestamp() - started_at;
            StatusLine::new("Operation", &format!("{} ({}s)", operation, elapsed))
                .show(ui);
        }
        OperationState::Failed { operation, error, .. } => {
            StatusLine::new("Last Operation", &format!("{} failed", operation))
                .error()
                .show(ui);
            ui.label(format!("  Error: {}", error));
        }
        OperationState::Completed { operation, completed_at } => {
            let ago = chrono::Utc::now().timestamp() - completed_at;
            StatusLine::new("Last Operation", &format!("{} completed ({}s ago)", operation, ago))
                .show(ui);
        }
        OperationState::Idle => {}
    }

    // SSH action menu (copy key, etc.)
    if let Some(private_key) = &row.ssh_private_key {
        ui.add_space(8.0);

        let ssh_command = format!(
            "K=$(mktemp) && cat > $K <<'EOF'\n{}\nEOF\nchmod 600 $K && ssh -i $K root@{} && rm $K",
            private_key.trim(),
            row.host
        );

        let mut menu = ActionMenu::new("SSH");
        menu.add_action("Copy SSH Command");
        menu.add_action("Copy Private Key");
        menu.add_action("Copy IP Address");

        if let Some(action_idx) = menu.show(ui) {
            let text_to_copy = match action_idx {
                0 => &ssh_command,
                1 => private_key,
                2 => &row.host,
                _ => return,
            };

            ui.ctx().copy_text(text_to_copy.to_string());
        }
    }
}

/// Render Logs tab (stdout logs filtered by SSH host)
fn render_logs_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    // Auto-refresh: trigger log reload every 1 second (not every frame)
    if let Some(ref ssh_host) = drawer_state.ssh_host {
        let refresh_id = egui::Id::new("ssh_drawer_log_last_refresh");
        let now = std::time::Instant::now();
        let should_refresh = ui.data(|d| {
            d.get_temp::<std::time::Instant>(refresh_id)
                .map(|last| now.duration_since(last).as_secs() >= 1)
                .unwrap_or(true)
        });

        if should_refresh {
            ui.data_mut(|d| {
                d.insert_temp(refresh_id, now);
                d.insert_temp(
                    egui::Id::new("ssh_drawer_action_refresh_logs"),
                    ssh_host.clone(),
                );
            });
        }
    }

    ui.horizontal(|ui| {
        ui.heading("SSH Host Logs");
        ui.add_space(8.0);

        // New logs indicator
        let log_count_id = egui::Id::new("ssh_drawer_prev_log_count");
        let current_count = drawer_state.logs.len();
        let prev_count = ui.data(|d| d.get_temp::<usize>(log_count_id)).unwrap_or(0);

        if current_count > prev_count {
            ui.label(
                egui::RichText::new("🆕 NEW")
                    .color(egui::Color32::from_rgb(76, 175, 80))
                    .strong(),
            );
            ui.ctx().request_repaint();
        }

        ui.data_mut(|d| d.insert_temp(log_count_id, current_count));

        ui.add_space(8.0);

        // Log level filter
        ui.label("Level:");
        let filter_id = egui::Id::new("ssh_log_level_filter");
        let mut log_level_filter = ui
            .data(|d| d.get_temp::<String>(filter_id))
            .unwrap_or_else(|| "All".to_string());

        egui::ComboBox::from_label("")
            .selected_text(&log_level_filter)
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut log_level_filter, "All".to_string(), "All");
                ui.selectable_value(&mut log_level_filter, "DEBUG".to_string(), "DEBUG");
                ui.selectable_value(&mut log_level_filter, "INFO".to_string(), "INFO");
                ui.selectable_value(&mut log_level_filter, "WARN".to_string(), "WARN");
                ui.selectable_value(&mut log_level_filter, "ERROR".to_string(), "ERROR");
            });

        ui.data_mut(|d| d.insert_temp(filter_id, log_level_filter.clone()));
    });

    ui.add_space(8.0);

    if drawer_state.loading {
        ui.spinner();
        ui.label("Loading logs...");
        return;
    }

    if drawer_state.ssh_host.is_none() {
        ui.label("No SSH host selected");
        return;
    }

    if drawer_state.logs.is_empty() {
        ui.label("No logs available");
        return;
    }

    // Filter logs by level
    let log_level_filter = ui
        .data(|d| d.get_temp::<String>(egui::Id::new("ssh_log_level_filter")))
        .unwrap_or_else(|| "All".to_string());

    let filtered_logs: Vec<&String> = if log_level_filter == "All" {
        drawer_state.logs.iter().collect()
    } else {
        let filter_str = format!("[{}]", log_level_filter);
        drawer_state
            .logs
            .iter()
            .filter(|line| line.contains(&filter_str))
            .collect()
    };

    // Scrollable log view - 500px height
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(500.0)
        .show(ui, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.0));

            for line in filtered_logs {
                ui.label(line);
            }
        });
}

/// Render Operations tab (operation logs from SQLite)
fn render_operations_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    // Auto-refresh: trigger operation reload every 2 seconds
    if let Some(ref ssh_host) = drawer_state.ssh_host {
        let refresh_id = egui::Id::new("ssh_drawer_ops_last_refresh");
        let now = std::time::Instant::now();
        let should_refresh = ui.data(|d| {
            d.get_temp::<std::time::Instant>(refresh_id)
                .map(|last| now.duration_since(last).as_secs() >= 2)
                .unwrap_or(true)
        });

        if should_refresh {
            ui.data_mut(|d| {
                d.insert_temp(refresh_id, now);
                d.insert_temp(
                    egui::Id::new("ssh_drawer_action_refresh_operations"),
                    ssh_host.clone(),
                );
            });
        }
    }

    ui.heading("Operations History");
    ui.add_space(8.0);

    if drawer_state.loading {
        ui.spinner();
        ui.label("Loading operations...");
        return;
    }

    if drawer_state.ssh_host.is_none() {
        ui.label("No SSH host selected");
        return;
    }

    if drawer_state.operations.is_empty() {
        ui.label("No operations recorded");
        return;
    }

    // Operations table
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .max_height(500.0)
        .show(ui, |ui| {
            use egui_extras::{Column, TableBuilder};

            TableBuilder::new(ui)
                .striped(true)
                .column(Column::auto().resizable(true)) // Operation
                .column(Column::auto().resizable(true)) // Status
                .column(Column::auto().resizable(true)) // Started
                .column(Column::remainder())            // Details
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Operation");
                    });
                    header.col(|ui| {
                        ui.strong("Status");
                    });
                    header.col(|ui| {
                        ui.strong("Started");
                    });
                    header.col(|ui| {
                        ui.strong("Details");
                    });
                })
                .body(|mut body| {
                    for op in &drawer_state.operations {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.label(&op.operation_type);
                            });
                            row.col(|ui| {
                                let color = match op.status.as_str() {
                                    "success" => egui::Color32::from_rgb(76, 175, 80),
                                    "failed" => egui::Color32::from_rgb(244, 67, 54),
                                    _ => ui.style().visuals.text_color(),
                                };
                                ui.label(egui::RichText::new(&op.status).color(color));
                            });
                            row.col(|ui| {
                                let time = format_timestamp(op.started_at);
                                ui.label(time);
                            });
                            row.col(|ui| {
                                if let Some(details) = &op.details {
                                    ui.label(details);
                                } else if let Some(error) = &op.error_message {
                                    ui.label(
                                        egui::RichText::new(error)
                                            .color(egui::Color32::from_rgb(244, 67, 54)),
                                    );
                                }
                            });
                        });
                    }
                });
        });
}

/// Render Host tab (system information)
fn render_host_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    use egui_twemoji::EmojiLabel as TwemojiLabel;

    ui.heading("Host Information");
    ui.add_space(8.0);

    if drawer_state.loading {
        ui.spinner();
        ui.label("Loading host information...");
        return;
    }

    if let Some(ref info) = drawer_state.host_info {
        TwemojiLabel::new(format!("💻 OS: {}", info.os)).show(ui);
        ui.add_space(4.0);

        TwemojiLabel::new(format!("🕐 Uptime: {}", info.uptime)).show(ui);
        ui.add_space(4.0);

        TwemojiLabel::new(format!("🌐 External IP: {}", info.external_ip)).show(ui);
        ui.add_space(4.0);

        TwemojiLabel::new(format!("📊 Load Average: {}", info.load_average)).show(ui);
        ui.add_space(4.0);

        TwemojiLabel::new(format!("💾 Memory: {}", info.memory_usage)).show(ui);
        ui.add_space(4.0);

        TwemojiLabel::new(format!("💿 Disk: {}", info.disk_usage)).show(ui);
        ui.add_space(8.0);

        if !info.top_processes.is_empty() {
            ui.heading("Top Processes");
            ui.add_space(4.0);

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.0));
                    for process in &info.top_processes {
                        ui.label(process);
                    }
                });
        }
    } else {
        ui.label("No host information available");
        if ui.button("Load Host Info").clicked() {
            // TODO: Trigger host info load
        }
    }
}

/// Render Docker tab
fn render_docker_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    use egui_twemoji::EmojiLabel as TwemojiLabel;

    ui.heading("Docker Status");
    ui.add_space(8.0);

    if drawer_state.loading {
        ui.spinner();
        ui.label("Loading Docker information...");
        return;
    }

    if let Some(ref status) = drawer_state.docker_status {
        if status.installed {
            TwemojiLabel::new("🐳 Docker: Installed").show(ui);
            ui.add_space(4.0);

            if let Some(ref version) = status.version {
                TwemojiLabel::new(format!("📌 Version: {}", version)).show(ui);
                ui.add_space(8.0);
            }

            // Container list
            if !drawer_state.containers.is_empty() {
                ui.heading("Containers");
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        use egui_extras::{Column, TableBuilder};

                        TableBuilder::new(ui)
                            .striped(true)
                            .column(Column::auto().resizable(true)) // Name
                            .column(Column::auto().resizable(true)) // Image
                            .column(Column::auto().resizable(true)) // Status
                            .column(Column::remainder())            // Ports
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.strong("Name");
                                });
                                header.col(|ui| {
                                    ui.strong("Image");
                                });
                                header.col(|ui| {
                                    ui.strong("Status");
                                });
                                header.col(|ui| {
                                    ui.strong("Ports");
                                });
                            })
                            .body(|mut body| {
                                for container in &drawer_state.containers {
                                    body.row(20.0, |mut row| {
                                        row.col(|ui| {
                                            ui.label(&container.name);
                                        });
                                        row.col(|ui| {
                                            ui.label(&container.image);
                                        });
                                        row.col(|ui| {
                                            let color = if container.status.contains("running") {
                                                egui::Color32::from_rgb(76, 175, 80)
                                            } else {
                                                ui.style().visuals.text_color()
                                            };
                                            ui.label(
                                                egui::RichText::new(&container.status).color(color),
                                            );
                                        });
                                        row.col(|ui| {
                                            ui.label(container.ports.join(", "));
                                        });
                                    });
                                }
                            });
                    });
            } else {
                ui.label("No containers running");
            }
        } else {
            TwemojiLabel::new("🐳 Docker: Not Installed").show(ui);
        }
    } else {
        ui.label("No Docker information available");
        if ui.button("Load Docker Info").clicked() {
            // TODO: Trigger Docker info load
        }
    }
}

/// Render Dure tab
fn render_dure_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    use egui_twemoji::EmojiLabel as TwemojiLabel;

    ui.heading("Dure WSS Status");
    ui.add_space(8.0);

    if drawer_state.loading {
        ui.spinner();
        ui.label("Loading Dure information...");
        return;
    }

    if let Some(ref status) = drawer_state.dure_status {
        if status.installed {
            TwemojiLabel::new("📦 Dure: Installed").show(ui);
            ui.add_space(4.0);

            if let Some(ref version) = status.version {
                TwemojiLabel::new(format!("📌 Version: {}", version)).show(ui);
                ui.add_space(4.0);
            }

            let running_text = if status.running {
                ("✅ Status: Running".to_string(), egui::Color32::from_rgb(76, 175, 80))
            } else {
                ("❌ Status: Stopped".to_string(), egui::Color32::from_rgb(244, 67, 54))
            };
            TwemojiLabel::new(egui::RichText::new(running_text.0).color(running_text.1)).show(ui);
            ui.add_space(8.0);

            if status.running {
                if ui.button("Stop Dure").clicked() {
                    // TODO: Trigger Dure stop
                }
            } else if ui.button("Start Dure").clicked() {
                // TODO: Trigger Dure start
            }
        } else {
            TwemojiLabel::new("📦 Dure: Not Installed").show(ui);
        }
    } else {
        ui.label("No Dure information available");
        if ui.button("Load Dure Info").clicked() {
            // TODO: Trigger Dure info load
        }
    }
}

/// Format Unix timestamp to human-readable string
fn format_timestamp(timestamp: i64) -> String {
    use chrono::{TimeZone, Utc};
    Utc.timestamp_opt(timestamp, 0)
        .single()
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}
