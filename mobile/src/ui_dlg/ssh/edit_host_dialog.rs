pub use super::edit_host_dialog_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{dialog, MaterialInput};
use std::cell::Cell;

impl EditHostDialog {
    pub fn new() -> Self {
        Self {
            port: "22".to_string(),
            ..Default::default()
        }
    }

    pub fn open(&mut self, original_host: String, host: String, port: u16, password: Option<String>, private_key_path: Option<String>) {
        self.original_host = original_host;
        self.host = host;
        self.port = port.to_string();
        self.use_password = password.is_some();
        self.password = password.unwrap_or_default();
        self.use_private_key = private_key_path.is_some();
        self.private_key_path = private_key_path.unwrap_or_default();
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.original_host.clear();
        self.host.clear();
        self.port = "22".to_string();
        self.use_password = false;
        self.password.clear();
        self.use_private_key = false;
        self.private_key_path.clear();
    }

    /// Show edit host dialog, returns action result
    pub fn show(&mut self, ctx: &egui::Context) -> EditHostAction {
        if !self.open {
            return EditHostAction::None;
        }

        let save_requested = Cell::new(false);
        let cancel_requested = Cell::new(false);

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog(
            "edit_ssh_host_dialog",
            &tr!("ssh-dialog-edit-title"),
            &mut self.open,
        )
        .min_width(350.0)
        .max_width(window_width - 120.0)
        .max_height(window_height - 120.0)
        .scrollable(true)
        .content(|ui, _open| {
            ui.label(tr!("ssh-dialog-edit-heading"));
            ui.add_space(8.0);

            ui.add(
                MaterialInput::outlined(&mut self.host)
                    .label(&tr!("ssh-dialog-add-host-label")),
            );
            ui.add_space(8.0);

            ui.add(
                MaterialInput::outlined(&mut self.port)
                    .label(&tr!("ssh-dialog-add-port-label")),
            );
            ui.add_space(8.0);

            ui.checkbox(&mut self.use_password, &tr!("ssh-dialog-add-use-password"));
            if self.use_password {
                ui.add(
                    MaterialInput::outlined(&mut self.password)
                        .label(&tr!("ssh-dialog-add-password-label"))
                        .password(true),
                );
            }
            ui.add_space(8.0);

            ui.checkbox(&mut self.use_private_key, &tr!("ssh-dialog-add-use-key"));
            if self.use_private_key {
                ui.add(
                    MaterialInput::outlined(&mut self.private_key_path)
                        .label(&tr!("ssh-dialog-add-key-path-label")),
                );
            }
        })
        .action(tr!("button-cancel"), || {
            cancel_requested.set(true);
        })
        .primary_action(tr!("button-save"), || {
            save_requested.set(true);
        })
        .show(ctx);

        if save_requested.get() {
            let result = EditHostAction::Save {
                original_host: self.original_host.clone(),
                host: self.host.clone(),
                port: self.port.clone(),
                use_password: self.use_password,
                password: self.password.clone(),
                use_private_key: self.use_private_key,
                private_key_path: self.private_key_path.clone(),
            };
            self.close();
            result
        } else if cancel_requested.get() || !self.open {
            self.close();
            EditHostAction::Cancel
        } else {
            EditHostAction::None
        }
    }
}
