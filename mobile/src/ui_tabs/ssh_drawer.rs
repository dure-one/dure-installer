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
    ui.horizontal(|ui| {
        ui.heading("Host Information");
        ui.add_space(8.0);

        if ui.button("Refresh").clicked() {
            ui.data_mut(|d| {
                d.insert_temp(
                    egui::Id::new("ssh_drawer_action_refresh_host"),
                    drawer_state.ssh_host.clone().unwrap_or_default(),
                );
            });
        }
    });

    ui.separator();

    if let Some(host_info) = &drawer_state.host_info {
        ui.label(format!("OS: {}", host_info.os));
        ui.label(format!("Uptime: {}", host_info.uptime));
        ui.label(format!("External IP: {}", host_info.external_ip));
        ui.label(format!("Load: {}", host_info.load_average));
        ui.label(format!("Memory: {}", host_info.memory_usage));
        ui.label(format!("Disk: {}", host_info.disk_usage));

        ui.add_space(8.0);
        ui.separator();
        ui.heading("Network Log (last 100 lines)");
        ui.separator();

        egui::ScrollArea::vertical()
            .max_height(300.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.style_mut().override_font_id = Some(egui::FontId::monospace(12.0));
                for line in &drawer_state.network_log {
                    ui.label(line);
                }
            });
    } else {
        ui.label("Loading host information...");
    }
}

/// Render Docker tab
fn render_docker_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    ui.horizontal(|ui| {
        ui.heading("Docker Containers");
        ui.add_space(8.0);

        if ui.button("Refresh").clicked() {
            ui.data_mut(|d| {
                d.insert_temp(
                    egui::Id::new("ssh_drawer_action_refresh_docker"),
                    drawer_state.ssh_host.clone().unwrap_or_default(),
                );
            });
        }
    });

    ui.separator();

    if let Some(docker_status) = &drawer_state.docker_status {
        if docker_status.installed {
            if let Some(version) = &docker_status.version {
                ui.label(format!("Docker Version: {}", version));
            }

            ui.add_space(8.0);
            ui.separator();

            // Raw docker ps -a output (monospace)
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::monospace(11.0));

                    if let Some(raw_output) = &drawer_state.docker_ps_raw {
                        ui.label(raw_output);
                    }
                });
        } else {
            ui.label("Docker is not installed on this host.");
        }
    } else {
        ui.label("Loading docker information...");
    }
}

/// Render Dure tab
fn render_dure_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    ui.horizontal(|ui| {
        ui.heading("Dure (mycart) Status");
        ui.add_space(8.0);

        if ui.button("Refresh").clicked() {
            ui.data_mut(|d| {
                d.insert_temp(
                    egui::Id::new("ssh_drawer_action_refresh_dure"),
                    drawer_state.ssh_host.clone().unwrap_or_default(),
                );
            });
        }
    });

    ui.separator();

    if let Some(dure_status) = &drawer_state.dure_status {
        if dure_status.installed {
            ui.label(format!("Installed: Yes"));
            ui.label(format!("Running: {}", if dure_status.running { "Yes" } else { "No" }));

            ui.add_space(8.0);
            ui.separator();
            ui.heading("docker-compose ps");
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::monospace(11.0));

                    if let Some(status) = &drawer_state.dure_compose_status {
                        ui.label(status);
                    }
                });

            ui.add_space(8.0);
            ui.separator();
            ui.heading("docker-compose logs (last 50 lines)");
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::monospace(11.0));

                    for line in &drawer_state.dure_compose_logs {
                        ui.label(line);
                    }
                });
        } else {
            ui.label("Dure is not installed on this host.");
            ui.label("Use 'Install Dure' button in the operations column.");
        }
    } else {
        ui.label("Loading dure information...");
    }
}

