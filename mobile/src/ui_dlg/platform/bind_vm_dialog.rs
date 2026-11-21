pub use super::bind_vm_dialog_stt::*;
use crate::{dure_debug, dure_error};
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{dialog, ListItem, MaterialList};
use std::cell::Cell;

impl BindVmDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, platform: String, vm_list: Vec<(String, String, String, String)>) {
        self.platform = platform;
        self.vm_list = vm_list;
        self.selected = None;
        self.loading = false;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.platform.clear();
        self.vm_list.clear();
        self.selected = None;
        self.loading = false;
    }

    /// Show bind VM dialog, returns action result
    pub fn show(&mut self, ctx: &egui::Context) -> BindVmAction {
        if !self.open {
            return BindVmAction::None;
        }

        let bind_requested = Cell::new(false);
        let cancel_requested = Cell::new(false);

        dialog(
            "bind_vm_dialog",
            &tr!("platform-dialog-bind-title"),
            &mut self.open,
        )
        .min_width(500.0)
        .max_height(450.0)
        .scrollable(true)
        .content(|ui, _open| {
            ui.heading(tr!("platform-dialog-bind-heading"));
            ui.add_space(12.0);

            if self.loading {
                ui.spinner();
                ui.label(tr!("platform-dialog-bind-loading"));
            } else if self.vm_list.is_empty() {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 193, 7),
                    tr!("platform-dialog-bind-no-vms"),
                );
            } else {
                ui.label(tr!("platform-dialog-bind-select", {count: self.vm_list.len()}));
                ui.add_space(8.0);

                let mut vm_list = MaterialList::new().id("vm_binding_list").fill_width(true);

                for (idx, (project_id, vm_name, zone, external_ip)) in
                    self.vm_list.iter().enumerate()
                {
                    let is_selected = self.selected == Some(idx);
                    let secondary = if !external_ip.is_empty() {
                        format!("{}, {}, {}", project_id, zone, external_ip)
                    } else {
                        format!("{}, {}", project_id, zone)
                    };

                    let item = ListItem::new(vm_name)
                        .secondary_text(&secondary)
                        .leading_icon("computer".to_string())
                        .dense(true)
                        .selected(is_selected)
                        .on_click({
                            let idx_copy = idx;
                            move || {
                                dure_debug!("VM {} clicked", idx_copy);
                            }
                        });

                    vm_list = vm_list.item(item);
                }

                let _list_response = ui.add(vm_list);

                // Handle selection via clicking on list items
                for (idx, _) in self.vm_list.iter().enumerate() {
                    let _item_id = egui::Id::new("vm_binding_list").with(idx);
                    if ui
                        .ctx()
                        .input(|i| i.pointer.button_clicked(egui::PointerButton::Primary))
                    {
                        if let Some(_pos) = ui.ctx().input(|i| i.pointer.interact_pos()) {
                            self.selected = Some(idx);
                            break;
                        }
                    }
                }
            }
        })
        .action(tr!("button-cancel"), || {
            cancel_requested.set(true);
        })
        .primary_action(tr!("button-bind"), || {
            bind_requested.set(true);
        })
        .show(ctx);

        if bind_requested.get() {
            let result = BindVmAction::Bind {
                platform: self.platform.clone(),
            };
            self.close();
            result
        } else if cancel_requested.get() || !self.open {
            self.close();
            BindVmAction::Cancel
        } else {
            BindVmAction::None
        }
    }
}
