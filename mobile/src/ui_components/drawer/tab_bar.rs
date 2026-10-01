//! TabBar component for drawer tab management with trait-based design

use eframe::egui;

/// Trait for drawer tab types that can be used with TabBar
pub trait DrawerTabTrait: Copy + PartialEq {
    /// Display name for the tab
    fn as_str(&self) -> &'static str;

    /// All available tabs for this type
    fn all() -> &'static [Self];
}

/// TabBar component for rendering and managing drawer tabs
pub struct TabBar<T: DrawerTabTrait + 'static> {
    spacing: f32,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: DrawerTabTrait + 'static> TabBar<T> {
    /// Create a new TabBar with default spacing
    pub fn new() -> Self {
        Self {
            spacing: 8.0,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Show tab bar and return clicked tab if any
    pub fn show(&self, ui: &mut egui::Ui, current_tab: T) -> Option<T> {
        let mut clicked_tab = None;

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = self.spacing;

            for tab in T::all() {
                let is_selected = current_tab == *tab;
                if ui.selectable_label(is_selected, tab.as_str()).clicked() && !is_selected {
                    clicked_tab = Some(*tab);
                }
            }
        });

        clicked_tab
    }
}

impl<T: DrawerTabTrait + 'static> Default for TabBar<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test enum implementing DrawerTabTrait
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestTab {
        First,
        Second,
        Third,
    }

    impl DrawerTabTrait for TestTab {
        fn as_str(&self) -> &'static str {
            match self {
                TestTab::First => "First",
                TestTab::Second => "Second",
                TestTab::Third => "Third",
            }
        }

        fn all() -> &'static [Self] {
            &[TestTab::First, TestTab::Second, TestTab::Third]
        }
    }

    #[test]
    fn test_tabbar_creation() {
        let tabbar: TabBar<TestTab> = TabBar::new();
        assert_eq!(tabbar.spacing, 8.0);
    }

    #[test]
    fn test_tabbar_default() {
        let tabbar1: TabBar<TestTab> = TabBar::new();
        let tabbar2: TabBar<TestTab> = TabBar::default();
        assert_eq!(tabbar1.spacing, tabbar2.spacing);
    }

    #[test]
    fn test_trait_as_str() {
        assert_eq!(TestTab::First.as_str(), "First");
        assert_eq!(TestTab::Second.as_str(), "Second");
        assert_eq!(TestTab::Third.as_str(), "Third");
    }

    #[test]
    fn test_trait_all() {
        let tabs = TestTab::all();
        assert_eq!(tabs.len(), 3);
        assert_eq!(tabs[0], TestTab::First);
        assert_eq!(tabs[1], TestTab::Second);
        assert_eq!(tabs[2], TestTab::Third);
    }

    #[test]
    fn test_trait_copy() {
        let tab1 = TestTab::First;
        let tab2 = tab1;
        assert_eq!(tab1, tab2);
    }

    #[test]
    fn test_trait_partialeq() {
        assert_eq!(TestTab::First, TestTab::First);
        assert_ne!(TestTab::First, TestTab::Second);
    }
}
