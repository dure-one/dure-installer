pub use super::billing_dialog_stt::*;
use eframe::egui;
use egui_i18n::tr;
use egui_material3::dialog;
use std::cell::Cell;

impl BillingDialog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, project_id: String) {
        self.project_id = project_id;
        self.data = None;
        self.loading = false;
        self.error = None;
        self.open = true;
    }

    pub fn close(&mut self) {
        self.open = false;
        self.project_id.clear();
        self.data = None;
        self.loading = false;
        self.error = None;
    }

    /// Show billing dialog, returns action result
    pub fn show(&mut self, ctx: &egui::Context) -> BillingAction {
        if !self.open {
            return BillingAction::None;
        }

        let refresh_requested = Cell::new(false);

        dialog(
            "billing_dialog",
            &tr!("platform-billing-title"),
            &mut self.open,
        )
        .max_width(600.0)
        .max_height(600.0)
        .scrollable(true)
        .content(|ui, _open| {
            ui.heading(tr!("platform-billing-heading"));
            ui.add_space(8.0);

            // Project ID display with GCP console link
            ui.horizontal(|ui| {
                ui.label(tr!("platform-billing-project-id"));
                ui.label(&self.project_id);
                ui.add_space(8.0);
                ui.hyperlink_to(
                    "GCP Console",
                    format!(
                        "https://console.cloud.google.com/billing?project={}",
                        self.project_id
                    ),
                );
            });
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);

            if self.loading {
                ui.spinner();
                ui.label(tr!("platform-billing-loading"));
            } else if let Some(error) = &self.error {
                let is_table_not_found =
                    error.contains("was not found") || error.contains("Not found");

                if is_table_not_found {
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 193, 7),
                        "⚠️ Billing Export Not Configured",
                    );
                    ui.add_space(4.0);
                    ui.label(tr!("platform-billing-no-table"));
                } else {
                    ui.colored_label(
                        egui_material3::theme::get_global_color("error"),
                        tr!("platform-billing-error"),
                    );
                    ui.add_space(4.0);
                    ui.label(error);
                }
                ui.add_space(16.0);

                ui.label(tr!("platform-billing-help"));
                ui.label("1. Go to GCP Console → Billing → Billing export");
                ui.label("2. Select 'Detailed cost data' tab");
                ui.label("3. Select or create a BigQuery dataset");
                ui.label("4. Wait a few hours for data to populate");
            } else if let Some(records) = &self.data {
                if records.is_empty() {
                    ui.label(tr!("platform-billing-no-data"));
                    ui.add_space(8.0);
                    ui.label("This could mean:");
                    ui.label("• Billing export is not configured");
                    ui.label("• No costs have been incurred yet this month");
                    ui.label("• Data is still being processed (can take up to 5 days)");
                } else {
                    ui.label(tr!("platform-billing-monthly-total", {count: records.len()}));
                    ui.add_space(8.0);

                    for record in records {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(format!("📅 {} ({})", record.month, record.currency));

                                let format_currency = |amount: f64, currency: &str| -> String {
                                    let uses_decimals = match currency {
                                        "KRW" | "JPY" | "VND" | "IDR" => false,
                                        _ => true,
                                    };

                                    let symbol = match currency {
                                        "USD" => "$",
                                        "EUR" => "€",
                                        "GBP" => "£",
                                        "JPY" => "¥",
                                        "KRW" => "₩",
                                        "CNY" => "¥",
                                        "INR" => "₹",
                                        "AUD" => "A$",
                                        "CAD" => "C$",
                                        "SGD" => "S$",
                                        "HKD" => "HK$",
                                        "TWD" => "NT$",
                                        "THB" => "฿",
                                        "VND" => "₫",
                                        "IDR" => "Rp",
                                        "BRL" => "R$",
                                        _ => currency,
                                    };

                                    if uses_decimals {
                                        format!("{}{:.2}", symbol, amount)
                                    } else {
                                        let int_amount = amount as i64;
                                        let abs_amount = int_amount.abs();
                                        let formatted = abs_amount
                                            .to_string()
                                            .as_bytes()
                                            .rchunks(3)
                                            .rev()
                                            .map(std::str::from_utf8)
                                            .collect::<Result<Vec<&str>, _>>()
                                            .unwrap()
                                            .join(",");
                                        if int_amount < 0 {
                                            format!("-{}{}", symbol, formatted)
                                        } else {
                                            format!("{}{}", symbol, formatted)
                                        }
                                    }
                                };

                                let (color, text) = if record.total_net_cost < 0.0 {
                                    (
                                        egui::Color32::from_rgb(76, 175, 80),
                                        format!(
                                            "{} (credit)",
                                            format_currency(record.total_net_cost, &record.currency)
                                        ),
                                    )
                                } else if record.total_net_cost == 0.0 {
                                    (
                                        egui_material3::theme::get_global_color("onSurfaceVariant"),
                                        format_currency(0.0, &record.currency),
                                    )
                                } else {
                                    (
                                        egui_material3::theme::get_global_color("error"),
                                        format_currency(record.total_net_cost, &record.currency),
                                    )
                                };

                                ui.colored_label(color, text);
                            });
                        });
                        ui.add_space(4.0);
                    }
                }
            }

            ui.add_space(16.0);
        })
        .action(tr!("button-close"), || {})
        .primary_action(tr!("platform-billing-refresh"), || {
            refresh_requested.set(true);
        })
        .show(ctx);

        if refresh_requested.get() {
            BillingAction::Refresh
        } else if !self.open {
            self.close();
            BillingAction::Close
        } else {
            BillingAction::None
        }
    }
}
