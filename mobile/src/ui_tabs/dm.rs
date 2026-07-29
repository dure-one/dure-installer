//! DM tab - Direct messaging

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use eframe::egui;

/// DM tab state
#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct DMTab {
    // Add state here as needed
}

impl DMTab {
    /// Render the DM tab UI
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Direct Messages");
        ui.separator();

        ui.label("This is DM tab!");
        // TODO: Add DM tab implementation
    }
}
