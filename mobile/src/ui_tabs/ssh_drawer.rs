//! SSH drawer UI with Status/Logs/Operations/Host/Docker/Dure tabs

use eframe::egui;
use egui_i18n::tr;
use egui_material3::{tabs_primary, tabs_secondary};

use crate::storage::models::opslog::OperationLog;
use crate::ui_components::drawer::{LogsRenderer, OperationsRenderer, StatusLine};
use crate::ui_tabs::ssh::{OperationState, SshRow};
use crate::viewmodel::ssh::{DrawerState, DrawerTab};

/// Render the SSH drawer with tabs
pub fn render_drawer(
    ui: &mut egui::Ui,
    row: &SshRow,
    drawer_state: &DrawerState,
    on_tab_switch: &mut Option<DrawerTab>,
) {
    // Tab bar (Material3 compact tabs)
    let mut temp_selected = drawer_state.active_tab.to_index();
    ui.add(
        tabs_primary(&mut temp_selected)
            .id_salt("ssh_drawer_tabs")
            .height(32.0)
            .tab(&tr!("ssh-drawer-tab-status"))
            .tab(&tr!("ssh-drawer-tab-logs"))
            .tab(&tr!("ssh-drawer-tab-operations"))
            .tab(&tr!("ssh-drawer-tab-host"))
            .tab(&tr!("ssh-drawer-tab-docker"))
            .tab(&tr!("ssh-drawer-tab-dure")),
    );
    if temp_selected != drawer_state.active_tab.to_index() {
        *on_tab_switch = Some(DrawerTab::from_index(temp_selected));
    }

    ui.separator();

    // Content area (500px from recent commit)
    egui::ScrollArea::vertical()
        .max_height(500.0)
        .show(ui, |ui| match drawer_state.active_tab {
            DrawerTab::Status => render_status_tab(ui, row),
            DrawerTab::Logs => render_logs_tab(ui, drawer_state),
            DrawerTab::Operations => render_operations_tab(ui, drawer_state),
            DrawerTab::Host => render_host_tab(ui, drawer_state),
            DrawerTab::Docker => render_docker_tab(ui, drawer_state),
            DrawerTab::Dure => render_dure_tab(ui, drawer_state),
        });
}

/// Render Status tab
fn render_status_tab(ui: &mut egui::Ui, row: &SshRow) {
    use crate::ui_components::ActionMenu;

    ui.add_space(8.0);

    // IP Address (no emoji)
    StatusLine::new(&tr!("ssh-drawer-status-ip"), &row.host).show(ui);

    // SSH Connection Status (inline error display)
    if let Some(check_result) = &row.last_ssh_check {
        if check_result.connected {
            StatusLine::new(
                &tr!("ssh-drawer-status-ssh"),
                tr!("ssh-drawer-status-connected"),
            )
            .show(ui);
        } else if let Some(error) = &check_result.error {
            StatusLine::new(
                &tr!("ssh-drawer-status-ssh"),
                &format!("Failed ({})", error),
            )
            .error()
            .show(ui);
        } else {
            StatusLine::new(
                &tr!("ssh-drawer-status-ssh"),
                tr!("ssh-drawer-status-disconnected"),
            )
            .warning()
            .show(ui);
        }
    } else {
        StatusLine::new(
            &tr!("ssh-drawer-status-ssh"),
            tr!("ssh-drawer-status-not-checked"),
        )
        .show(ui);
    }

    // Base Packages Status
    let host_status = if row.host_installed {
        tr!("ssh-drawer-status-installed")
    } else {
        tr!("ssh-drawer-status-not-installed")
    };
    let mut base_line = StatusLine::new(&tr!("ssh-drawer-status-base"), host_status);
    if !row.host_installed {
        base_line = base_line.warning();
    }
    base_line.show(ui);

    // Show missing packages
    if let Some(check_result) = &row.last_host_check {
        if !check_result.missing_packages.is_empty() {
            ui.label(format!(
                "  Missing: {}",
                check_result.missing_packages.join(", ")
            ));
        }
    }

    // Docker Status (single line with version)
    let docker_status = if row.docker_installed {
        if let Some(check_result) = &row.last_docker_check {
            if let Some(version) = &check_result.version {
                format!("Installed ({})", version)
            } else {
                tr!("ssh-drawer-status-installed").to_string()
            }
        } else {
            tr!("ssh-drawer-status-installed").to_string()
        }
    } else {
        tr!("ssh-drawer-status-not-installed").to_string()
    };
    let mut docker_line = StatusLine::new(&tr!("ssh-drawer-status-docker"), &docker_status);
    if !row.docker_installed {
        docker_line = docker_line.warning();
    }
    docker_line.show(ui);

    // Dure Status
    let dure_status = if row.dure_installed {
        tr!("ssh-drawer-status-installed")
    } else {
        tr!("ssh-drawer-status-not-installed")
    };
    let mut dure_line = StatusLine::new(&tr!("ssh-drawer-status-dure"), dure_status);
    if !row.dure_installed {
        dure_line = dure_line.warning();
    }
    dure_line.show(ui);

    ui.add_space(8.0);

    // Operation State
    match &row.operation_state {
        OperationState::InProgress {
            operation,
            started_at,
        } => {
            let elapsed = chrono::Utc::now().timestamp() - started_at;
            StatusLine::new(
                &tr!("ssh-drawer-status-operation"),
                &tr!("ssh-op-in-progress-text", {operation: operation.clone(), seconds: elapsed}),
            )
            .show(ui);
        }
        OperationState::Failed {
            operation, error, ..
        } => {
            StatusLine::new(
                &tr!("ssh-drawer-status-last-operation"),
                &tr!("ssh-op-failed-text", {operation: operation.clone()}),
            )
            .error()
            .show(ui);
            ui.label(format!("  Error: {}", error));
        }
        OperationState::Completed {
            operation,
            completed_at,
        } => {
            let ago = chrono::Utc::now().timestamp() - completed_at;
            StatusLine::new(
                &tr!("ssh-drawer-status-last-operation"),
                &tr!("ssh-op-completed-ago", {operation: operation.clone(), seconds: ago}),
            )
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
    let renderer = LogsRenderer::new(&tr!("ssh-logs-title")).refresh_interval(1); // 1 second auto-refresh

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
    if let Some(raw_output) = &drawer_state.host_status_raw {
        egui::ScrollArea::vertical()
            .max_height(480.0)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.style_mut().override_font_id = Some(egui::FontId::monospace(11.0));
                ui.label(raw_output);
            });
    } else {
        ui.label(tr!("ssh-host-placeholder"));
    }
}

/// Render Docker tab
fn render_docker_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    if let Some(raw_output) = &drawer_state.docker_ps_raw {
        egui::ScrollArea::vertical()
            .max_height(480.0)
            .show(ui, |ui| {
                ui.style_mut().override_font_id = Some(egui::FontId::monospace(11.0));
                ui.label(raw_output);
            });
    } else {
        ui.label(tr!("ssh-docker-placeholder"));
    }
}

/// Render Dure tab with per-container subtabs
fn render_dure_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    if drawer_state.dure_containers.is_empty() {
        ui.label(tr!("ssh-dure-placeholder"));
        return;
    }

    ui.vertical(|ui| {
        // Sub-tab bar: Status + one tab per container
        let mut active_subtab = drawer_state.dure_active_subtab;
        let mut tab_builder = tabs_secondary(&mut active_subtab)
            .id_salt("dure_subtabs")
            .height(32.0)
            .tab("Status");

        for container in &drawer_state.dure_containers {
            tab_builder = tab_builder.tab(&container.name);
        }

        ui.add(tab_builder);

        // Store selected tab in temp data if changed
        if active_subtab != drawer_state.dure_active_subtab {
            if let Some(ref host) = drawer_state.ssh_host {
                ui.data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new("dure_subtab_selection").with(host),
                        (host.clone(), active_subtab),
                    );
                });
            }
        }

        ui.separator();
        ui.add_space(8.0);

        // Content based on selected sub-tab
        let focus_applied = if active_subtab == 0 {
            // Tab 0: Container status table
            render_dure_status_subtab(ui, drawer_state)
        } else {
            // Tab 1+: Per-container logs
            let container_index = active_subtab - 1;
            if let Some(container) = drawer_state.dure_containers.get(container_index) {
                render_dure_container_logs_subtab(ui, drawer_state, &container.name)
            } else {
                ui.label("Container not found");
                false
            }
        };

        // Signal focus applied via temp data
        if focus_applied {
            if let Some(ref host) = drawer_state.ssh_host {
                ui.data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new("dure_subtab_focus_applied").with(host),
                        host.clone(),
                    );
                });
            }
        }
    });
}

/// Render Status sub-tab: container status table
fn render_dure_status_subtab(ui: &mut egui::Ui, drawer_state: &DrawerState) -> bool {
    ui.heading("Container Status");
    ui.add_space(8.0);

    let grid_response = egui::Grid::new("dure_containers_grid")
        .striped(true)
        .show(ui, |ui| {
            ui.strong("Name");
            ui.strong("Status");
            ui.strong("Image");
            ui.end_row();

            for container in &drawer_state.dure_containers {
                ui.label(&container.name);
                ui.label(&container.status);
                ui.label(&container.image);
                ui.end_row();
            }
        })
        .response;

    if drawer_state.dure_subtab_should_focus {
        grid_response.request_focus();
        return true;
    }
    false
}

/// Render container logs sub-tab
fn render_dure_container_logs_subtab(
    ui: &mut egui::Ui,
    drawer_state: &DrawerState,
    container_name: &str,
) -> bool {
    ui.heading(format!("Logs: {}", container_name));
    ui.add_space(8.0);

    if let Some(logs) = drawer_state.dure_container_logs.get(container_name) {
        let scroll_response = egui::ScrollArea::vertical()
            .max_height(400.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.style_mut().override_font_id = Some(egui::FontId::monospace(11.0));
                for line in logs {
                    ui.label(line);
                }
            });

        if drawer_state.dure_subtab_should_focus {
            ui.memory_mut(|mem| mem.request_focus(scroll_response.id));
            return true;
        }
    } else {
        ui.label("No logs available");
    }
    false
}
