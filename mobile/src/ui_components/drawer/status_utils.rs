use eframe::egui::{self, Color32, RichText, Ui};

/// A status line with optional warning/error coloring (no emojis)
pub struct StatusLine {
    label: String,
    value: String,
    color: Option<Color32>,
}

impl StatusLine {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            color: None,
        }
    }

    pub fn warning(mut self) -> Self {
        self.color = Some(Color32::from_rgb(255, 193, 7));
        self
    }

    pub fn error(mut self) -> Self {
        self.color = Some(Color32::from_rgb(244, 67, 54));
        self
    }

    pub fn show(&self, ui: &mut Ui) {
        let text = format!("{}: {}", self.label, self.value);
        if let Some(color) = self.color {
            ui.label(RichText::new(text).color(color));
        } else {
            ui.label(text);
        }
        ui.add_space(4.0);
    }
}

/// Format elapsed time in human-readable form
pub fn format_elapsed(seconds: i64) -> String {
    if seconds < 60 {
        "just now".to_string()
    } else if seconds < 3600 {
        format!("{} min ago", seconds / 60)
    } else if seconds < 86400 {
        format!("{} hours ago", seconds / 3600)
    } else {
        format!("{} days ago", seconds / 86400)
    }
}

/// Get color based on staleness (for refresh timestamps)
pub fn staleness_color(seconds: i64, ui: &Ui) -> Color32 {
    if seconds < 3600 {
        ui.style().visuals.text_color()
    } else {
        Color32::from_rgb(255, 193, 7) // Warning color
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_elapsed() {
        assert_eq!(format_elapsed(30), "just now");
        assert_eq!(format_elapsed(120), "2 min ago");
        assert_eq!(format_elapsed(7200), "2 hours ago");
        assert_eq!(format_elapsed(172800), "2 days ago");
    }

    #[test]
    fn test_format_elapsed_edge_cases() {
        // Just under 60 seconds
        assert_eq!(format_elapsed(59), "just now");
        // Exactly 60 seconds
        assert_eq!(format_elapsed(60), "1 min ago");
        // Just under 1 hour
        assert_eq!(format_elapsed(3599), "59 min ago");
        // Exactly 1 hour
        assert_eq!(format_elapsed(3600), "1 hours ago");
        // Just under 1 day
        assert_eq!(format_elapsed(86399), "23 hours ago");
        // Exactly 1 day
        assert_eq!(format_elapsed(86400), "1 days ago");
    }

    #[test]
    fn test_status_line_new() {
        let status = StatusLine::new("Status", "Connected");
        assert_eq!(status.label, "Status");
        assert_eq!(status.value, "Connected");
        assert_eq!(status.color, None);
    }

    #[test]
    fn test_status_line_warning() {
        let status = StatusLine::new("Status", "Warning").warning();
        assert_eq!(status.label, "Status");
        assert_eq!(status.value, "Warning");
        assert_eq!(status.color, Some(Color32::from_rgb(255, 193, 7)));
    }

    #[test]
    fn test_status_line_error() {
        let status = StatusLine::new("Status", "Error").error();
        assert_eq!(status.label, "Status");
        assert_eq!(status.value, "Error");
        assert_eq!(status.color, Some(Color32::from_rgb(244, 67, 54)));
    }

    #[test]
    fn test_status_line_no_emojis() {
        let status = StatusLine::new("Connection", "Failed").error();
        let text = format!("{}: {}", status.label, status.value);
        // Verify no emoji characters are present
        assert!(!text.contains('✓'));
        assert!(!text.contains('✗'));
        assert!(!text.contains('⚠'));
        assert!(!text.contains('❌'));
        assert!(!text.contains('✅'));
    }

    #[test]
    fn test_format_elapsed_no_emojis() {
        let formatted = format_elapsed(30);
        assert!(!formatted.contains('✓'));
        assert!(!formatted.contains('⏱'));
        assert!(!formatted.contains('⌚'));
    }
}
