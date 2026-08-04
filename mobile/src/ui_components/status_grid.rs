//! Responsive status grid component

use crate::ui_components::emoji_loader::SvgEmoji;

/// Item state for visual indicators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemState {
    InProgress,
    Success,
    Error,
    Warning,
}

/// Grid item with emoji, label, value, and optional state
#[derive(Clone)]
pub struct StatusGridItem {
    pub emoji: SvgEmoji,
    pub label: String,
    pub value: String,
    pub state: Option<ItemState>,
}

/// Responsive key-value grid
pub struct StatusGrid {
    items: Vec<StatusGridItem>,
    min_column_width: f32,
}

impl StatusGrid {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            min_column_width: 200.0,
        }
    }

    pub fn with_min_column_width(width: f32) -> Self {
        Self {
            items: Vec::new(),
            min_column_width: width,
        }
    }

    pub fn add_item(
        &mut self,
        emoji: SvgEmoji,
        label: impl Into<String>,
        value: impl Into<String>,
        state: Option<ItemState>,
    ) {
        self.items.push(StatusGridItem {
            emoji,
            label: label.into(),
            value: value.into(),
            state,
        });
    }

    fn calculate_columns(&self, available_width: f32) -> usize {
        let cols = (available_width / self.min_column_width).floor() as usize;
        cols.clamp(1, 3)
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let available_width = ui.available_width();
        let columns = self.calculate_columns(available_width);

        // Get theme text color for proper visibility
        let text_color = ui.style().visuals.text_color();

        // Use egui Grid for layout
        egui::Grid::new("status_grid")
            .num_columns(columns * 3) // emoji + label + value per column
            .spacing([8.0, 4.0])
            .show(ui, |ui| {
                for (idx, item) in self.items.iter().enumerate() {
                    // Emoji (use Unicode for now, SVG in next iteration)
                    ui.label(egui::RichText::new(item.emoji.to_unicode()).color(text_color));

                    // Label with theme color
                    ui.label(egui::RichText::new(&item.label).strong().color(text_color));

                    // Value with state color (or theme color if no state)
                    let value_text = match item.state {
                        Some(ItemState::InProgress) => {
                            egui::RichText::new(&item.value).color(egui::Color32::from_rgb(255, 152, 0))
                        }
                        Some(ItemState::Success) => {
                            egui::RichText::new(&item.value).color(egui::Color32::from_rgb(76, 175, 80))
                        }
                        Some(ItemState::Error) => {
                            egui::RichText::new(&item.value).color(egui::Color32::from_rgb(244, 67, 54))
                        }
                        Some(ItemState::Warning) => {
                            egui::RichText::new(&item.value).color(egui::Color32::from_rgb(255, 193, 7))
                        }
                        None => egui::RichText::new(&item.value).color(text_color),
                    };
                    ui.label(value_text);

                    // End row after filling columns
                    if (idx + 1) % columns == 0 {
                        ui.end_row();
                    }
                }
            });
    }
}

impl Default for StatusGrid {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_columns() {
        let grid = StatusGrid::new();
        assert_eq!(grid.calculate_columns(700.0), 3);
        assert_eq!(grid.calculate_columns(450.0), 2);
        assert_eq!(grid.calculate_columns(250.0), 1);
        assert_eq!(grid.calculate_columns(100.0), 1); // Min 1
    }

    #[test]
    fn test_add_item() {
        let mut grid = StatusGrid::new();
        grid.add_item(SvgEmoji::Email, "Email", "test@example.com", None);
        grid.add_item(SvgEmoji::VM, "VM", "my-vm", Some(ItemState::Success));
        assert_eq!(grid.items.len(), 2);
        assert_eq!(grid.items[0].label, "Email");
        assert_eq!(grid.items[1].state, Some(ItemState::Success));
    }
}
