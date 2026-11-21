pub use super::firewall_dialog_stt::*;
use crate::ui_tabs::platform::FirewallRule;
use crate::viewmodel::platform::FullFirewallRule;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{dialog, list::ListItem, list::MaterialList, MaterialButton};
use std::cell::Cell;
use std::sync::{Arc, Mutex};

impl FirewallDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, platform: String) {
        self.platform = platform;
        self.rule_list.clear();
        self.rules.clear();
        self.selected = None;
        self.selected_rows.clear();
        self.edit_data.clear();
        self.current_ip = None;
        self.show_remove_ip_dialog = false;
        self.remove_ip_list.clear();
        self.remove_ip_selected.clear();
        self.show_flush_confirm = false;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.platform.clear();
        self.rule_list.clear();
        self.rules.clear();
        self.selected = None;
        self.selected_rows.clear();
        self.edit_data.clear();
        self.last_refresh_time = None;
        self.current_ip = None;
        self.show_remove_ip_dialog = false;
        self.remove_ip_list.clear();
        self.remove_ip_selected.clear();
        self.show_flush_confirm = false;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> FirewallAction {
        if !self.open {
            return FirewallAction::None;
        }

        let add_requested = Cell::new(false);
        let reset_requested = Cell::new(false);
        let edit_idx = Arc::new(Mutex::new(None::<usize>));
        let delete_idx = Arc::new(Mutex::new(None::<usize>));

        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog(
            "firewall_dialog",
            "Manage Firewall Rules",
            &mut self.open,
        )
        .min_width(350.0)
        .max_width(window_width - 120.0)
        .max_height(window_height - 120.0)
        .scrollable(true)
        .content(|ui, _open| {
            // Current IP display
            if let Some(ip) = &self.current_ip {
                ui.label(format!("Current IP: {}", ip));
            } else if self.rule_list.is_empty() {
                ui.label("Fetching current IP...");
            }

            ui.add_space(8.0);

            // Action buttons
            ui.horizontal(|ui| {
                if ui.add(MaterialButton::filled(tr!("firewall-button-add-row"))).clicked() {
                    add_requested.set(true);
                }
                if ui.add(MaterialButton::text(tr!("firewall-button-reset"))).clicked() {
                    reset_requested.set(true);
                }
            });

            ui.add_space(8.0);

            // Rules list
            if self.rule_list.is_empty() {
                ui.label("No firewall rules found");
            } else {
                ui.label(format!("{} firewall rules", self.rule_list.len()));
                ui.add_space(4.0);

                let mut list = MaterialList::new().id("firewall_rules_list");

                for (idx, rule) in self.rule_list.iter().enumerate() {
                    // Build secondary text with rule details
                    let mut secondary_parts = vec![
                        format!("Direction: {} | Priority: {}", rule.direction, rule.priority)
                    ];

                    if !rule.source_ranges.is_empty() {
                        secondary_parts.push(format!("Sources: {}", rule.source_ranges.join(", ")));
                    }

                    if !rule.allowed.is_empty() {
                        secondary_parts.push(format!("Allowed: {}", rule.allowed.join(", ")));
                    }

                    let secondary_text = secondary_parts.join("\n");

                    // Clone Arc for use in closure
                    let edit_idx_clone = Arc::clone(&edit_idx);
                    let delete_idx_clone = Arc::clone(&delete_idx);

                    let item = ListItem::new(&rule.name)
                        .secondary_text(&secondary_text)
                        .trailing_widget(move |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 8.0;

                                if ui.small_button("Edit").clicked() {
                                    if let Ok(mut edit) = edit_idx_clone.lock() {
                                        *edit = Some(idx);
                                    }
                                }

                                if ui.small_button("Delete").clicked() {
                                    if let Ok(mut delete) = delete_idx_clone.lock() {
                                        *delete = Some(idx);
                                    }
                                }
                            });
                        });

                    list = list.item(item);
                }

                ui.add(list);
            }
        })
        .show(ctx);

        // Handle actions
        if add_requested.get() {
            FirewallAction::AddRule
        } else if reset_requested.get() {
            FirewallAction::Reset
        } else {
            // Check edit action
            let edit_action = edit_idx.lock().ok().and_then(|mut e| e.take());
            if let Some(idx) = edit_action {
                if let Some(rule) = self.rule_list.get(idx) {
                    // Convert FullFirewallRule to FirewallRule
                    let rule_data = FirewallRule {
                        name: rule.name.clone(),
                        network: rule.network.clone(),
                        direction: rule.direction.clone(),
                        priority: rule.priority,
                        source_ranges: rule.source_ranges.clone(),
                        dest_ranges: rule.dest_ranges.clone(),
                        allowed: rule.allowed.clone(),
                        denied: rule.denied.clone(),
                        source_tags: rule.source_tags.clone(),
                        source_service_accounts: rule.source_service_accounts.clone(),
                        target_tags: rule.target_tags.clone(),
                        target_service_accounts: rule.target_service_accounts.clone(),
                    };
                    return FirewallAction::EditRule { rule_data, rule_idx: idx };
                }
            }

            // Check delete action
            let delete_action = delete_idx.lock().ok().and_then(|mut d| d.take());
            if let Some(idx) = delete_action {
                if let Some(rule) = self.rule_list.get(idx) {
                    return FirewallAction::DeleteRule {
                        rule_name: rule.name.clone(),
                        rule_idx: Some(idx),
                    };
                }
            }

            // Check close
            if !self.open {
                self.close();
                FirewallAction::Close
            } else {
                FirewallAction::None
            }
        }
    }
}
