//! SSH drawer UI with Status/Logs/Operations/Host/Docker/Dure tabs

use eframe::egui;
use egui_i18n::tr;
use egui_material3::tabs_primary;

use crate::storage::models::opslog::OperationLog;
use crate::ui_components::drawer::{StatusLine, LogsRenderer, OperationsRenderer};
use crate::ui_tabs::ssh::{SshRow, OperationState};
use crate::viewmodel::ssh::{DrawerTab, DrawerState};

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
            .tab(&tr!("ssh-drawer-tab-dure"))
    );
    if temp_selected != drawer_state.active_tab.to_index() {
        *on_tab_switch = Some(DrawerTab::from_index(temp_selected));
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
    StatusLine::new(&tr!("ssh-drawer-status-ip"), &row.host).show(ui);

    // SSH Connection Status (inline error display)
    if let Some(check_result) = &row.last_ssh_check {
        if check_result.connected {
            StatusLine::new(&tr!("ssh-drawer-status-ssh"), tr!("ssh-drawer-status-connected")).show(ui);
        } else if let Some(error) = &check_result.error {
            StatusLine::new(&tr!("ssh-drawer-status-ssh"), &format!("Failed ({})", error))
                .error()
                .show(ui);
        } else {
            StatusLine::new(&tr!("ssh-drawer-status-ssh"), tr!("ssh-drawer-status-disconnected"))
                .warning()
                .show(ui);
        }
    } else {
        StatusLine::new(&tr!("ssh-drawer-status-ssh"), tr!("ssh-drawer-status-not-checked")).show(ui);
    }

    // Base Packages Status
    let base_status = if row.base_installed { tr!("ssh-drawer-status-installed") } else { tr!("ssh-drawer-status-not-installed") };
    let mut base_line = StatusLine::new(&tr!("ssh-drawer-status-base"), base_status);
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
    let dure_status = if row.dure_installed { tr!("ssh-drawer-status-installed") } else { tr!("ssh-drawer-status-not-installed") };
    let mut dure_line = StatusLine::new(&tr!("ssh-drawer-status-dure"), dure_status);
    if !row.dure_installed {
        dure_line = dure_line.warning();
    }
    dure_line.show(ui);

    ui.add_space(8.0);

    // Operation State
    match &row.operation_state {
        OperationState::InProgress { operation, started_at } => {
            let elapsed = chrono::Utc::now().timestamp() - started_at;
            StatusLine::new(&tr!("ssh-drawer-status-operation"), &tr!("ssh-op-in-progress-text", {operation: operation.clone(), seconds: elapsed}))
                .show(ui);
        }
        OperationState::Failed { operation, error, .. } => {
            StatusLine::new(&tr!("ssh-drawer-status-last-operation"), &tr!("ssh-op-failed-text", {operation: operation.clone()}))
                .error()
                .show(ui);
            ui.label(format!("  Error: {}", error));
        }
        OperationState::Completed { operation, completed_at } => {
            let ago = chrono::Utc::now().timestamp() - completed_at;
            StatusLine::new(&tr!("ssh-drawer-status-last-operation"), &tr!("ssh-op-completed-ago", {operation: operation.clone(), seconds: ago}))
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
    let renderer = LogsRenderer::new(&tr!("ssh-logs-title"))
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

/// Render Dure tab
fn render_dure_tab(ui: &mut egui::Ui, drawer_state: &DrawerState) {
    if !drawer_state.dure_compose_logs.is_empty() {
        egui::ScrollArea::vertical()
            .max_height(480.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.style_mut().override_font_id = Some(egui::FontId::monospace(11.0));
                for line in &drawer_state.dure_compose_logs {
                    ui.label(line);
                }
            });
    } else {
        ui.label(tr!("ssh-dure-placeholder"));
    }
}

