pub use super::add_host_dialog_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{dialog, MaterialInput};
use std::cell::Cell;

impl AddHostDialog {
    pub fn new() -> Self {
        Self {
            port: "22".to_string(),
            ..Default::default()
        }
    }

    pub fn open(&mut self) {
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.host.clear();
        self.port = "22".to_string();
        self.use_password = false;
        self.password.clear();
        self.use_private_key = false;
        self.private_key_path.clear();
    }

    /// Show add host dialog, returns action result
    pub fn show(&mut self, ctx: &egui::Context) -> AddHostAction {
        if !self.open {
            return AddHostAction::None;
        }

        let add_requested = Cell::new(false);
        let cancel_requested = Cell::new(false);

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog(
            "add_ssh_host_dialog",
            &tr!("ssh-dialog-add-title"),
            &mut self.open,
        )
        .min_width(350.0)
        .max_width(window_width - 120.0)
        .max_height(window_height - 120.0)
        .scrollable(true)
        .content(|ui, _open| {
            ui.label(tr!("ssh-dialog-add-heading"));
            ui.add_space(8.0);

            ui.add(
                MaterialInput::outlined(&mut self.host).label(&tr!("ssh-dialog-add-host-label")),
            );
            ui.add_space(8.0);

            ui.add(
                MaterialInput::outlined(&mut self.port).label(&tr!("ssh-dialog-add-port-label")),
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
        .primary_action(tr!("button-add"), || {
            add_requested.set(true);
        })
        .show(ctx);

        if add_requested.get() {
            let result = AddHostAction::Add {
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
            AddHostAction::Cancel
        } else {
            AddHostAction::None
        }
    }
}
