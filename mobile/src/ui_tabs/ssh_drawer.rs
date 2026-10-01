//! SSH drawer UI with Status/Logs/Operations/Host/Docker/Dure tabs

use eframe::egui;

use crate::storage::models::opslog::OperationLog;
use crate::ui_tabs::ssh::SshRow;
use crate::viewmodel::ssh::{DrawerTab, DrawerState};

/// Render the SSH drawer with tabs
pub fn render_drawer(
    ui: &mut egui::Ui,
    row: &SshRow,
    drawer_state: &DrawerState,
    on_tab_switch: &mut Option<DrawerTab>,
) {
    // Tab bar at top
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;

        for tab in DrawerTab::all() {
            let is_selected = drawer_state.active_tab == tab;

            if ui.selectable_label(is_selected, tab.as_str()).clicked() && !is_selected {
                *on_tab_switch = Some(tab);
            }
        }
    });

    ui.separator();

    // Content area with static height
    egui::ScrollArea::vertical()
        .max_height(500.0)
        .show(ui, |ui| {
            match drawer_state.active_tab {
                DrawerTab::Status => render_status_tab(ui, row, drawer_state),
                DrawerTab::Logs => render_logs_tab(ui, drawer_state),
                DrawerTab::Operations => render_operations_tab(ui, drawer_state),
                DrawerTab::Host => render_host_tab(ui, drawer_state),
                DrawerTab::Docker => render_docker_tab(ui, drawer_state),
                DrawerTab::Dure => render_dure_tab(ui, drawer_state),
            }
        });
}

/// Render Status tab (simplified - no action buttons)
fn render_status_tab(ui: &mut egui::Ui, row: &SshRow, _drawer_state: &DrawerState) {
    use crate::ui_components::ActionMenu;
    use egui_twemoji::EmojiLabel as TwemojiLabel;

    ui.add_space(8.0);

    // IP address
    TwemojiLabel::new(format!("🌐 IP: {}", row.host)).show(ui);
    ui.add_space(4.0);

    // Base packages status
    let base_text = if row.base_installed {
        ("📦 Base: Installed".to_string(), ui.style().visuals.text_color())
    } else {
        (
            "📦 Base: Not Installed".to_string(),
            egui::Color32::from_rgb(255, 152, 0),
        )
    };
    TwemojiLabel::new(egui::RichText::new(base_text.0).color(base_text.1)).show(ui);
    ui.add_space(4.0);

    // Docker status
    let docker_text = if row.docker_installed {
        ("🐳 Docker: Installed".to_string(), ui.style().visuals.text_color())
    } else {
        (
            "🐳 Docker: Not Installed".to_string(),
            egui::Color32::from_rgb(255, 152, 0),
        )
    };
    TwemojiLabel::new(egui::RichText::new(docker_text.0).color(docker_text.1)).show(ui);
    ui.add_space(4.0);

    // Dure status
    let dure_text = if row.dure_installed {
        ("🚀 Dure: Installed".to_string(), ui.style().visuals.text_color())
    } else {
        (
            "🚀 Dure: Not Installed".to_string(),
            egui::Color32::from_rgb(255, 152, 0),
        )
    };
    TwemojiLabel::new(egui::RichText::new(dure_text.0).color(dure_text.1)).show(ui);
    ui.add_space(8.0);

    // SSH action menu (copy key like platform tab)
    if let Some(private_key) = &row.ssh_private_key {
        ui.add_space(8.0);

        let ssh_command = format!(
            "K=$(mktemp) && cat > $K <<'EOF'\n{}\nEOF\nchmod 600 $K && ssh -i $K root@{} && rm $K",
            private_key.trim(),
            row.host
        );

        let mut menu = ActionMenu::new("💻 SSH");
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
    // Auto-refresh: trigger log reload every 2 seconds
    if let Some(ref ssh_host) = drawer_state.ssh_host {
        let refresh_id = egui::Id::new("host_log_last_refresh");
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
                    egui::Id::new("host_action_refresh_network_log"),
                    ssh_host.clone(),
                );
            });
        }
    }

    ui.heading("/var/log/dure-network.log");
    ui.add_space(8.0);

    if drawer_state.loading {
        ui.spinner();
        ui.label("Loading network log...");
        return;
    }

    if drawer_state.ssh_host.is_none() {
        ui.label("No SSH host selected");
        return;
    }

    // Show network log from host_info (reusing existing field temporarily)
    // TODO: Add dedicated network_log field to DrawerState
    if let Some(ref info) = drawer_state.host_info {
        if !info.top_processes.is_empty() {
            // Temporarily using top_processes to store log lines
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .max_height(500.0)
                .show(ui, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.0));

                    for line in &info.top_processes {
                        ui.label(line);
                    }
                });
        } else {
            ui.label("No network log entries");
        }
    } else {
        ui.label("Network log not loaded");
        ui.add_space(8.0);
        ui.label("Click Refresh in the operations column to load the latest network activity log.");
    }
}

/// Render Docker tab (docker ps -a output)
fn render_docker_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    use egui_twemoji::EmojiLabel as TwemojiLabel;

    // Auto-refresh: trigger docker ps reload every 2 seconds
    if let Some(ref ssh_host) = drawer_state.ssh_host {
        let refresh_id = egui::Id::new("docker_ps_last_refresh");
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
                    egui::Id::new("docker_action_refresh_ps"),
                    ssh_host.clone(),
                );
            });
        }
    }

    ui.heading("Docker Containers (docker ps -a)");
    ui.add_space(8.0);

    if drawer_state.loading {
        ui.spinner();
        ui.label("Loading Docker containers...");
        return;
    }

    if drawer_state.ssh_host.is_none() {
        ui.label("No SSH host selected");
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

            // Container list from docker ps -a
            if !drawer_state.containers.is_empty() {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
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

    // Auto-refresh: trigger docker compose ps reload every 2 seconds
    if let Some(ref ssh_host) = drawer_state.ssh_host {
        let refresh_id = egui::Id::new("dure_compose_last_refresh");
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
                    egui::Id::new("dure_action_refresh_compose"),
                    ssh_host.clone(),
                );
            });
        }
    }

    ui.heading("Dure Docker Compose Status");
    ui.add_space(4.0);
    ui.label("Location: /srv/dure-mycart/xmpp-proxy-stack");
    ui.add_space(8.0);

    if drawer_state.loading {
        ui.spinner();
        ui.label("Loading Dure status...");
        return;
    }

    if drawer_state.ssh_host.is_none() {
        ui.label("No SSH host selected");
        return;
    }

    if let Some(ref status) = drawer_state.dure_status {
        if status.installed {
            TwemojiLabel::new("🚀 Dure: Installed").show(ui);
            ui.add_space(4.0);

            if let Some(ref version) = status.version {
                TwemojiLabel::new(format!("📌 Version: {}", version)).show(ui);
                ui.add_space(4.0);
            }

            let running_text = if status.running {
                ("✅ Status: Running".to_string(), egui::Color32::from_rgb(76, 175, 80))
            } else {
                ("⚠️ Status: Stopped".to_string(), egui::Color32::from_rgb(255, 152, 0))
            };
            TwemojiLabel::new(egui::RichText::new(running_text.0).color(running_text.1)).show(ui);
            ui.add_space(8.0);

            // Show docker compose ps output (stored in containers list)
            // TODO: Backend should populate with compose service status
            if !drawer_state.containers.is_empty() {
                ui.heading("Services");
                ui.add_space(4.0);

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(400.0)
                    .show(ui, |ui| {
                        use egui_extras::{Column, TableBuilder};

                        TableBuilder::new(ui)
                            .striped(true)
                            .column(Column::auto().resizable(true)) // Service
                            .column(Column::auto().resizable(true)) // Status
                            .column(Column::remainder())            // Ports
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.strong("Service");
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
                                            ui.label(&container.status);
                                        });
                                        row.col(|ui| {
                                            ui.label(container.ports.join(", "));
                                        });
                                    });
                                }
                            });
                    });
            } else {
                ui.label("No services found");
            }
        } else {
            TwemojiLabel::new("🚀 Dure: Not Installed").show(ui);
            ui.add_space(8.0);
            ui.label("Use the Install Dure button in the operations column to set up Dure.");
        }
    } else {
        ui.label("Dure status not loaded");
        ui.add_space(8.0);
        ui.label("Click Refresh in the operations column to check Dure installation status.");
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
