//! Dropdown action menu component

use crate::ui_components::emoji_loader::SvgEmoji;

/// Dropdown action menu
pub struct ActionMenu {
    label: String,
    icon: Option<SvgEmoji>,
    actions: Vec<String>,
}

impl ActionMenu {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            actions: Vec::new(),
        }
    }

    pub fn with_icon(mut self, icon: SvgEmoji) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn add_action(&mut self, label: impl Into<String>) {
        self.actions.push(label.into());
    }

    /// Show menu and return index of clicked action
    pub fn show(&mut self, ui: &mut egui::Ui) -> Option<usize> {
        let button_text = if let Some(icon) = self.icon {
            format!("{} {}", icon.to_unicode(), self.label)
        } else {
            self.label.clone()
        };

        let mut selected = None;

        egui::menu::menu_button(ui, button_text, |ui| {
            for (idx, action) in self.actions.iter().enumerate() {
                if ui.button(action).clicked() {
                    selected = Some(idx);
                    ui.close_menu();
                }
            }
        });

        selected
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_menu_creation() {
        let mut menu = ActionMenu::new("Test Menu");
        menu.add_action("Action 1");
        menu.add_action("Action 2");
        menu.add_action("Action 3");

        assert_eq!(menu.actions.len(), 3);
        assert_eq!(menu.actions[0], "Action 1");
        assert_eq!(menu.label, "Test Menu");
    }

    #[test]
    fn test_with_icon() {
        let menu = ActionMenu::new("SSH")
            .with_icon(SvgEmoji::Terminal);
        assert_eq!(menu.icon, Some(SvgEmoji::Terminal));
    }
}
