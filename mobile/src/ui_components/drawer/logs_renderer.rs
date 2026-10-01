use eframe::egui::{self, Color32, Id, RichText, ScrollArea, Ui};

/// Auto-refreshing log display component
pub struct LogsRenderer {
    title: String,
    refresh_interval_secs: u64,
    max_height: f32,
}

impl LogsRenderer {
    /// Create a new LogsRenderer with a title
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            refresh_interval_secs: 1,
            max_height: 400.0,
        }
    }

    /// Set the auto-refresh interval in seconds (default 1)
    pub fn refresh_interval(mut self, secs: u64) -> Self {
        self.refresh_interval_secs = secs;
        self
    }

    /// Set the maximum height of the log area (default 400.0)
    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = height;
        self
    }

    /// Show logs with auto-refresh, returns true if should refresh
    pub fn show(&self, ui: &mut Ui, logs: &[String], refresh_trigger_id: Id) -> bool {
        let mut should_refresh = false;

        // Auto-refresh logic
        let refresh_id = ui.id().with("log_refresh");
        let now = std::time::Instant::now();
        should_refresh = ui.data(|d| {
            d.get_temp::<std::time::Instant>(refresh_id)
                .map(|last| now.duration_since(last).as_secs() >= self.refresh_interval_secs)
                .unwrap_or(true)
        });

        if should_refresh {
            ui.data_mut(|d| {
                d.insert_temp(refresh_id, now);
                d.insert_temp(refresh_trigger_id, ());
            });
        }

        // Header with new logs indicator
        ui.horizontal(|ui| {
            ui.heading(&self.title);
            ui.add_space(8.0);

            let log_count_id = ui.id().with("prev_log_count");
            let current_count = logs.len();
            let prev_count = ui.data(|d| d.get_temp::<usize>(log_count_id).unwrap_or(0));

            if current_count > prev_count {
                let new_count = current_count - prev_count;
                ui.label(
                    RichText::new(format!("^ {} new", new_count))
                        .color(Color32::from_rgb(76, 175, 80))
                );
                ui.data_mut(|d| d.insert_temp(log_count_id, current_count));
            }
        });

        ui.separator();

        ScrollArea::vertical()
            .max_height(self.max_height)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                if logs.is_empty() {
                    ui.label("No logs yet");
                } else {
                    for line in logs {
                        ui.label(line);
                    }
                }
            });

        should_refresh
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logs_renderer_new() {
        let renderer = LogsRenderer::new("Test Logs");
        assert_eq!(renderer.title, "Test Logs");
        assert_eq!(renderer.refresh_interval_secs, 1);
        assert_eq!(renderer.max_height, 400.0);
    }

    #[test]
    fn test_logs_renderer_refresh_interval() {
        let renderer = LogsRenderer::new("Logs").refresh_interval(5);
        assert_eq!(renderer.refresh_interval_secs, 5);
    }

    #[test]
    fn test_logs_renderer_max_height() {
        let renderer = LogsRenderer::new("Logs").max_height(600.0);
        assert_eq!(renderer.max_height, 600.0);
    }

    #[test]
    fn test_logs_renderer_builder_chain() {
        let renderer = LogsRenderer::new("Logs")
            .refresh_interval(2)
            .max_height(500.0);
        assert_eq!(renderer.title, "Logs");
        assert_eq!(renderer.refresh_interval_secs, 2);
        assert_eq!(renderer.max_height, 500.0);
    }

    #[test]
    fn test_logs_renderer_title_from_string() {
        let renderer = LogsRenderer::new("SSH Logs".to_string());
        assert_eq!(renderer.title, "SSH Logs");
    }

    #[test]
    fn test_logs_renderer_no_emojis_in_title() {
        let renderer = LogsRenderer::new("Logs");
        assert!(!renderer.title.contains('✓'));
        assert!(!renderer.title.contains('✗'));
        assert!(!renderer.title.contains('⚠'));
    }
}
