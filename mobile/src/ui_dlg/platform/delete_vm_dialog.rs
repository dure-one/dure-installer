pub use super::delete_vm_dialog_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{dialog, MaterialButton};
use std::cell::Cell;

impl DeleteVmDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, platform: String, vm_list: Vec<(String, String, String)>) {
        self.platform = platform;
        self.vm_list = vm_list;
        self.selected = None;
        self.confirming = false;
        self.hard_delete = false;
        self.has_static_ip = false;
        self.ip_address = None;
        self.ip_name = None;
        self.release_ip = false;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.platform.clear();
        self.vm_list.clear();
        self.selected = None;
        self.confirming = false;
        self.hard_delete = false;
        self.has_static_ip = false;
        self.ip_address = None;
        self.ip_name = None;
        self.release_ip = false;
    }

    /// Show delete VM dialog, returns action result
    pub fn show(&mut self, ctx: &egui::Context) -> DeleteVmAction {
        if !self.open {
            return DeleteVmAction::None;
        }

        let next_clicked = Cell::new(false);
        let yes_clicked = Cell::new(false);
        let cancel_clicked = Cell::new(false);

        let is_confirming = self.confirming;
        let button_text = if is_confirming {
            tr!("platform-delete-vm-yes")
        } else {
            tr!("platform-delete-vm-next")
        };

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog("delete_vm_dialog", "Delete VM", &mut self.open)
            .min_width(350.0)
            .max_width(window_width - 120.0)
            .max_height(window_height - 120.0)
            .scrollable(true)
            .content(|ui, _open| {
                if self.confirming {
                    // Confirmation step
                    ui.heading(tr!("platform-delete-vm-confirm"));
                    ui.add_space(8.0);

                    if let Some(idx) = self.selected {
                        if let Some((name, zone, _)) = self.vm_list.get(idx).cloned() {
                            ui.label(tr!("platform-delete-vm-question", {name: name.clone()}));
                            ui.add_space(4.0);
                            ui.colored_label(
                                egui_material3::theme::get_global_color("error"),
                                "This action cannot be undone!",
                            );
                            ui.add_space(4.0);
                            ui.label(tr!("platform-delete-vm-zone", {zone: zone.clone()}));

                            ui.add_space(8.0);

                            // No graceful shutdown checkbox
                            ui.checkbox(&mut self.hard_delete, "Skip graceful shutdown");
                            ui.add_space(4.0);
                            ui.colored_label(
                                egui_material3::theme::get_global_color("onSurfaceVariant"),
                                "VM will be terminated immediately without graceful shutdown",
                            );

                            // Static IP section
                            if self.has_static_ip {
                                ui.add_space(8.0);
                                ui.separator();
                                ui.add_space(8.0);

                                ui.label(format!(
                                    "Static IP: {}",
                                    self.ip_address.as_ref().unwrap()
                                ));
                                ui.checkbox(
                                    &mut self.release_ip,
                                    "Release static IP address"
                                );
                                ui.label("(Default: Keep IP reserved for future use)");

                                ui.add_space(8.0);
                                if ui.add(MaterialButton::text(tr!("platform-delete-vm-manage-ips")).small()).clicked() {
                                    let url = format!(
                                        "https://console.cloud.google.com/networking/addresses/list?project={}",
                                        self.platform
                                    );
                                    let _ = webbrowser::open(&url);
                                }
                            }

                            ui.add_space(12.0);
                        }
                    }
                } else {
                    // VM selection step
                    ui.heading(tr!("platform-delete-vm-select"));
                    ui.add_space(8.0);

                    if self.vm_list.is_empty() {
                        ui.label(tr!("platform-delete-vm-none"));
                        ui.add_space(8.0);
                        ui.colored_label(
                            egui_material3::theme::get_global_color("onSurfaceVariant"),
                            "Note: Only VMs in common zones are shown.",
                        );
                    } else {
                        ui.label(tr!("platform-delete-vm-found", {count: self.vm_list.len()}));
                        ui.add_space(8.0);

                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                for (idx, (name, zone, status)) in
                                    self.vm_list.iter().enumerate()
                                {
                                    let is_selected = self.selected == Some(idx);
                                    if ui
                                        .selectable_label(
                                            is_selected,
                                            format!("{} ({}, {})", name, zone, status),
                                        )
                                        .clicked()
                                    {
                                        self.selected = Some(idx);
                                    }
                                }
                            });
                    }

                    ui.add_space(12.0);
                }
            })
            .action(&tr!("platform-delete-vm-cancel"), || {
                cancel_clicked.set(true);
            })
            .primary_action(&button_text, || {
                if is_confirming {
                    yes_clicked.set(true);
                } else {
                    next_clicked.set(true);
                }
            })
            .show(ctx);

        // Handle actions
        if cancel_clicked.get() {
            if self.confirming {
                self.confirming = false;
                return DeleteVmAction::None;
            } else {
                self.close();
                return DeleteVmAction::Cancel;
            }
        }

        if next_clicked.get() && self.selected.is_some() {
            self.confirming = true;
            return DeleteVmAction::None;
        }

        if yes_clicked.get() {
            if let Some(idx) = self.selected {
                if let Some((name, zone, _)) = self.vm_list.get(idx).cloned() {
                    let release_ip = if self.release_ip {
                        self.ip_name.as_ref().and_then(|addr_name| {
                            // Region = zone with trailing "-<letter>" stripped
                            let region = zone.rsplitn(2, '-').nth(1).unwrap_or(&zone).to_string();
                            Some((region, addr_name.clone()))
                        })
                    } else {
                        None
                    };

                    let result = DeleteVmAction::Delete {
                        vm_name: name,
                        zone,
                        force: self.hard_delete,
                        release_ip,
                    };
                    self.close();
                    return result;
                }
            }
        }

        DeleteVmAction::None
    }
}
