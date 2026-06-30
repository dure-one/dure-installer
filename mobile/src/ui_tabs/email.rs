//! Email tab - Email management

use eframe::egui::{self, Id, Rect};
use egui_material3::{
    MaterialButton, MaterialIconButton, TooltipPosition, data_table, noto_emoji,
    show_tooltip_on_hover,
};

/// Email tab state
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct EmailTab {
    search_text: String,
    selected_row: Option<usize>,
    title_text: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    is_scrolled: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    search_open: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    menu_anchor_rect: Option<Rect>,
    menu_selected: String,
    // For button demo
    label: String,
    disabled: bool,
    soft_disabled: bool,
}

impl Default for EmailTab {
    fn default() -> Self {
        Self {
            search_text: String::new(),
            selected_row: None,
            title_text: "Platform".to_string(),
            is_scrolled: false,
            search_open: false,
            menu_anchor_rect: None,
            menu_selected: String::new(),
            label: String::new(),
            disabled: false,
            soft_disabled: false,
        }
    }
}

impl EmailTab {
    fn is_disabled(&self) -> bool {
        self.disabled || self.soft_disabled
    }

    fn label_or<'a>(&'a self, default: &'a str) -> &'a str {
        if self.label.is_empty() {
            default
        } else {
            &self.label
        }
    }

    /// Render the email tab UI
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // reference/egui-material3/examples/stories/button_window.rs
        // Small Buttons with SVG Icons
        ui.heading("Small Buttons with SVG Icons");

        let _disabled = self.is_disabled();

        #[cfg(feature = "svg_emoji")]
        {
            // Leading SVG icons
            ui.label("With leading SVG icon:");
            ui.horizontal_wrapped(|ui| {
                if let Some(&star_svg) = SOLAR_ICONS.get("star") {
                    let button = MaterialButton::filled(self.label_or("Star"))
                        .small()
                        .leading_svg(star_svg);
                    let button = if disabled {
                        button.enabled(false)
                    } else {
                        button
                    };
                    if ui.add(button).clicked() && !disabled {
                        println!("Small filled button with SVG clicked!");
                    }
                }

                if let Some(&heart_svg) = SOLAR_ICONS.get("heart") {
                    let button = MaterialButton::outlined(self.label_or("Like"))
                        .small()
                        .leading_svg(heart_svg);
                    let button = if disabled {
                        button.enabled(false)
                    } else {
                        button
                    };
                    if ui.add(button).clicked() && !disabled {
                        println!("Small outlined button with SVG clicked!");
                    }
                }

                if let Some(&bookmark_svg) = SOLAR_ICONS.get("bookmark") {
                    let button = MaterialButton::filled_tonal(self.label_or("Save"))
                        .small()
                        .leading_svg(bookmark_svg);
                    let button = if disabled {
                        button.enabled(false)
                    } else {
                        button
                    };
                    if ui.add(button).clicked() && !disabled {
                        println!("Small tonal button with SVG clicked!");
                    }
                }

                if let Some(&settings_svg) = SOLAR_ICONS.get("settings") {
                    let button = MaterialButton::elevated(self.label_or("Settings"))
                        .small()
                        .leading_svg(settings_svg);
                    let button = if disabled {
                        button.enabled(false)
                    } else {
                        button
                    };
                    if ui.add(button).clicked() && !disabled {
                        println!("Small elevated button with SVG clicked!");
                    }
                }
            });

            ui.add_space(8.0);

            // Trailing SVG icons
            ui.label("With trailing SVG icon:");
            ui.horizontal_wrapped(|ui| {
                if let Some(&arrow_right_svg) = SOLAR_ICONS.get("arrow-right") {
                    let button = MaterialButton::filled(self.label_or("Next"))
                        .small()
                        .trailing_svg(arrow_right_svg);
                    let button = if disabled {
                        button.enabled(false)
                    } else {
                        button
                    };
                    if ui.add(button).clicked() && !disabled {
                        println!("Small button with trailing SVG clicked!");
                    }
                }

                if let Some(&share_svg) = SOLAR_ICONS.get("share") {
                    let button = MaterialButton::text(self.label_or("Share"))
                        .small()
                        .trailing_svg(share_svg);
                    let button = if disabled {
                        button.enabled(false)
                    } else {
                        button
                    };
                    if ui.add(button).clicked() && !disabled {
                        println!("Small text button with trailing SVG clicked!");
                    }
                }
            });

            ui.add_space(8.0);

            // Both leading and trailing SVG icons
            ui.label("With both leading and trailing SVG icons:");
            ui.horizontal_wrapped(|ui| {
                if let (Some(&star_svg), Some(&arrow_right_svg)) =
                    (SOLAR_ICONS.get("star"), SOLAR_ICONS.get("arrow-right"))
                {
                    let button = MaterialButton::filled(self.label_or("Featured"))
                        .small()
                        .leading_svg(star_svg)
                        .trailing_svg(arrow_right_svg);
                    let button = if disabled {
                        button.enabled(false)
                    } else {
                        button
                    };
                    if ui.add(button).clicked() && !disabled {
                        println!("Small button with both SVG icons clicked!");
                    }
                }

                if let (Some(&download_svg), Some(&check_svg)) =
                    (SOLAR_ICONS.get("download"), SOLAR_ICONS.get("check"))
                {
                    let button = MaterialButton::outlined(self.label_or("Download"))
                        .small()
                        .leading_svg(download_svg)
                        .trailing_svg(check_svg);
                    let button = if disabled {
                        button.enabled(false)
                    } else {
                        button
                    };
                    if ui.add(button).clicked() && !disabled {
                        println!("Small outlined button with both SVG icons clicked!");
                    }
                }
            });
        }

        // reference/egui-material3/examples/stories/tooltip_window.rs
        // Icon Buttons with Tooltips
        ui.heading("Tooltips on Icon Buttons");
        ui.label("Icon buttons benefit from tooltips to explain their function:");

        ui.horizontal_wrapped(|ui| {
            let icon1 = ui.add(MaterialIconButton::standard(noto_emoji::HOUSE_BUILDING).size(40.0));
            show_tooltip_on_hover(ui, &icon1, "Home", TooltipPosition::Bottom);

            let icon2 = ui.add(
                MaterialIconButton::standard(noto_emoji::RIGHT_POINTING_MAGNIFYING_GLASS)
                    .size(40.0),
            );
            show_tooltip_on_hover(ui, &icon2, "Search", TooltipPosition::Bottom);

            let icon3 = ui.add(MaterialIconButton::standard(noto_emoji::GEAR).size(40.0));
            show_tooltip_on_hover(ui, &icon3, "Settings", TooltipPosition::Bottom);

            let icon4 =
                ui.add(MaterialIconButton::standard(noto_emoji::SPARKLING_HEART).size(40.0));
            show_tooltip_on_hover(ui, &icon4, "Favorite", TooltipPosition::Bottom);

            let icon5 = ui.add(
                MaterialIconButton::standard(
                    noto_emoji::ARROW_POINTING_RIGHTWARDS_THEN_CURVING_UPWARDS,
                )
                .size(40.0),
            );
            show_tooltip_on_hover(ui, &icon5, "Share", TooltipPosition::Bottom);

            let icon6 = ui.add(MaterialIconButton::standard(noto_emoji::WASTEBASKET).size(40.0));
            show_tooltip_on_hover(ui, &icon6, "Delete", TooltipPosition::Bottom);
        });

        ui.add_space(20.0);

        // Different Icon Button Styles
        ui.heading("Tooltips on Icon Button Variants");
        ui.label("All icon button variants support tooltips:");

        ui.horizontal_wrapped(|ui| {
            let icon1 =
                ui.add(MaterialIconButton::standard(noto_emoji::INFORMATION_SOURCE).size(40.0));
            show_tooltip_on_hover(ui, &icon1, "Standard icon button", TooltipPosition::Top);

            let icon2 =
                ui.add(MaterialIconButton::filled(noto_emoji::WHITE_HEAVY_CHECK_MARK).size(40.0));
            show_tooltip_on_hover(ui, &icon2, "Filled icon button", TooltipPosition::Top);

            let icon3 = ui.add(MaterialIconButton::filled_tonal(noto_emoji::BELL).size(40.0));
            show_tooltip_on_hover(ui, &icon3, "Filled tonal icon button", TooltipPosition::Top);

            let icon4 = ui.add(MaterialIconButton::outlined(noto_emoji::ENVELOPE).size(40.0));
            show_tooltip_on_hover(ui, &icon4, "Outlined icon button", TooltipPosition::Top);
        });

        ui.add_space(20.0);

        // Long Tooltip Text
        ui.heading("Long Tooltip Text");
        ui.label("Tooltips automatically wrap long text:");

        ui.horizontal_wrapped(|ui| {
            let btn = ui.add(MaterialButton::filled("Long Tooltip"));
            show_tooltip_on_hover(
                ui,
                &btn,
                "This is a much longer tooltip text that demonstrates how tooltips handle multiple lines of text. The tooltip will wrap the text to fit within the maximum width.",
                TooltipPosition::Top,
            );
        });

        // add compact datatable from reference/egui-material3/examples/stories/datatable_window.rs
        ui.heading("Compact Data Table");

        let compact_table = data_table()
            .id(Id::new("compact_data_table"))
            .column("ID", 60.0, true)
            .column("Name", 120.0, false)
            .column("Status", 80.0, false)
            .column("Progress", 100.0, true)
            .row(|row| {
                row.cell("001")
                    .cell("Task Alpha")
                    .cell("Active")
                    .cell("75%")
                    .id("compact_row_0")
            })
            .row(|row| {
                row.cell("002")
                    .cell("Task Beta")
                    .cell("Pending")
                    .cell("25%")
                    .id("compact_row_1")
            })
            .row(|row| {
                row.cell("003")
                    .cell("Task Gamma")
                    .cell("Complete")
                    .cell("100%")
                    .id("compact_row_2")
            });

        ui.add(compact_table);

        ui.add_space(20.0);
    }
}
