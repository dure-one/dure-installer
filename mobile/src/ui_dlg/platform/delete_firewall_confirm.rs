pub use super::delete_firewall_confirm_stt::*;
use eframe::egui;
use egui_material3::{dialog, MaterialButton};
use std::cell::Cell;

impl DeleteFirewallConfirmDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, rule_name: String, platform: String, rule_idx: Option<usize>) {
        self.rule_name = rule_name;
        self.platform = platform;
        self.rule_idx = rule_idx;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.rule_name.clear();
        self.platform.clear();
        self.rule_idx = None;
    }

    /// Show delete firewall confirm dialog, returns action result
    pub fn show(&mut self, ctx: &egui::Context) -> DeleteFirewallConfirmAction {
        if !self.open {
            return DeleteFirewallConfirmAction::None;
        }

        let confirmed = Cell::new(false);
        let cancelled = Cell::new(false);

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog("delete_firewall_confirm", "Confirm Delete", &mut self.open)
            .min_width(400.0)
            .max_width(window_width - 120.0)
            .max_height(window_height - 120.0)
            .scrollable(true)
            .content(|ui, _open| {
                ui.label(format!("Delete firewall rule '{}'?", self.rule_name));
                ui.add_space(8.0);
                ui.label("This action cannot be undone.");
                ui.add_space(16.0);

                // Keep custom styled button in content for now
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui
                        .add(
                            MaterialButton::filled("Delete")
                                .text_color(egui::Color32::from_rgb(255, 82, 82)),
                        )
                        .clicked()
                    {
                        confirmed.set(true);
                    }
                    if ui.add(MaterialButton::outlined("Cancel")).clicked() {
                        cancelled.set(true);
                    }
                });
            })
            .show(ctx);

        if confirmed.get() {
            let result = DeleteFirewallConfirmAction::Delete {
                rule_name: self.rule_name.clone(),
                platform: self.platform.clone(),
                rule_idx: self.rule_idx,
            };
            self.close();
            result
        } else if cancelled.get() || !self.open {
            self.close();
            DeleteFirewallConfirmAction::Cancel
        } else {
            DeleteFirewallConfirmAction::None
        }
    }
}
