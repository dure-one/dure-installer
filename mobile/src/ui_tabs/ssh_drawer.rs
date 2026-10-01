//! SSH drawer UI with Status/Logs/Operations/Host/Docker/Dure tabs

use eframe::egui;

use crate::storage::models::opslog::OperationLog;
use crate::ui_components::drawer::{StatusLine, TabBar, LogsRenderer, OperationsRenderer};
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
    let renderer = LogsRenderer::new("SSH Host Logs")
        .refresh_interval(1);  // 1 second auto-refresh

    let refresh_id = egui::Id::new("ssh_drawer_action_refresh_logs");
    renderer.show(ui, &drawer_state.logs, refresh_id);
}

/// Render Operations tab (operation logs from SQLite)
fn render_operations_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    let renderer = OperationsRenderer::new();
    renderer.show(ui, &drawer_state.operations);
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

