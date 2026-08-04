//! Emoji progress bar component

use crate::ui_components::emoji_loader::SvgEmoji;

/// Progress step state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressState {
    Completed,
    InProgress,
    Pending,
    Failed,
}

/// Single progress step
#[derive(Clone)]
pub struct ProgressStep {
    pub label: String,
    pub state: ProgressState,
}

/// Emoji progress bar
pub struct EmojiProgressBar {
    steps: Vec<ProgressStep>,
    compact: bool,
}

impl EmojiProgressBar {
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
            compact: true,
        }
    }

    pub fn add_step(&mut self, label: impl Into<String>, state: ProgressState) {
        self.steps.push(ProgressStep {
            label: label.into(),
            state,
        });
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    fn emoji_for_state(&self, state: ProgressState) -> SvgEmoji {
        match state {
            ProgressState::Completed => SvgEmoji::Checkmark,
            ProgressState::InProgress => SvgEmoji::Progress,
            ProgressState::Pending => SvgEmoji::Circle,
            ProgressState::Failed => SvgEmoji::Cross,
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) -> egui::Response {
        if self.compact {
            // Inline: emoji → emoji → emoji (with labels on hover)
            ui.horizontal(|ui| {
                for (idx, step) in self.steps.iter().enumerate() {
                    let emoji = self.emoji_for_state(step.state);
                    ui.label(emoji.to_unicode())
                        .on_hover_text(&step.label);

                    // Arrow between steps (except last)
                    if idx < self.steps.len() - 1 {
                        ui.label("→");
                    }
                }
            })
            .response
        } else {
            // Stacked: emoji + label per row
            ui.vertical(|ui| {
                for step in &self.steps {
                    ui.horizontal(|ui| {
                        let emoji = self.emoji_for_state(step.state);
                        ui.label(emoji.to_unicode());
                        ui.label(&step.label);
                    });
                }
            })
            .response
        }
    }

    /// Create progress bar from PlatformRow state
    pub fn from_platform_row(row: &crate::ui_tabs::platform::PlatformRow) -> Self {
        let mut bar = Self::new();

        // Step 1: OAuth
        let oauth_state = if row.gcp_connected {
            ProgressState::Completed
        } else {
            ProgressState::Pending
        };
        bar.add_step("OAuth", oauth_state);

        // Step 2: Project
        let project_state = if row.project_selected {
            ProgressState::Completed
        } else if row.gcp_connected {
            ProgressState::Pending
        } else {
            ProgressState::Pending
        };
        bar.add_step("Project", project_state);

        // Step 3: VM
        let vm_state = if row.vm_created {
            ProgressState::Completed
        } else if row.project_selected {
            // Check if VM operation in progress
            match &row.operation_state {
                crate::ui_tabs::platform::OperationState::InProgress { operation, .. }
                    if operation.to_lowercase().contains("vm") =>
                {
                    ProgressState::InProgress
                }
                _ => ProgressState::Pending,
            }
        } else {
            ProgressState::Pending
        };
        bar.add_step("VM", vm_state);

        // Step 4: Firewall
        let firewall_state = if row.firewall_updated {
            ProgressState::Completed
        } else if row.vm_created {
            match &row.operation_state {
                crate::ui_tabs::platform::OperationState::InProgress { operation, .. }
                    if operation.to_lowercase().contains("firewall") =>
                {
                    ProgressState::InProgress
                }
                _ => ProgressState::Pending,
            }
        } else {
            ProgressState::Pending
        };
        bar.add_step("Firewall", firewall_state);

        // Step 5: SSH
        let ssh_state = if row.ssh_ready {
            ProgressState::Completed
        } else if row.vm_created {
            ProgressState::Pending
        } else {
            ProgressState::Pending
        };
        bar.add_step("SSH", ssh_state);

        bar
    }
}

impl Default for EmojiProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_step() {
        let mut bar = EmojiProgressBar::new();
        bar.add_step("OAuth", ProgressState::Completed);
        bar.add_step("Project", ProgressState::InProgress);
        bar.add_step("VM", ProgressState::Pending);

        assert_eq!(bar.steps.len(), 3);
        assert_eq!(bar.steps[0].label, "OAuth");
        assert_eq!(bar.steps[1].state, ProgressState::InProgress);
    }

    #[test]
    fn test_emoji_mapping() {
        let bar = EmojiProgressBar::new();
        assert_eq!(bar.emoji_for_state(ProgressState::Completed), SvgEmoji::Checkmark);
        assert_eq!(bar.emoji_for_state(ProgressState::InProgress), SvgEmoji::Progress);
        assert_eq!(bar.emoji_for_state(ProgressState::Pending), SvgEmoji::Circle);
        assert_eq!(bar.emoji_for_state(ProgressState::Failed), SvgEmoji::Cross);
    }

    #[test]
    fn test_from_platform_row_all_completed() {
        use crate::ui_tabs::platform::{PlatformRow, OperationState};

        let row = PlatformRow {
            project_id: "test".to_string(),
            project_display_name: "Test".to_string(),
            platform_type: "GCP".to_string(),
            gcp_connected: true,
            project_selected: true,
            vm_created: true,
            firewall_updated: true,
            ssh_ready: true,
            operation_state: OperationState::Idle,
            email: None,
            total_project_count: 0,
            selected_project_id: None,
            vm_name: None,
            vm_external_ip: None,
            ssh_private_key: None,
            ssh_public_key: None,
            ssh_keyring_domain: None,
            firewall_status: "".to_string(),
            ssh_status: "".to_string(),
            last_refresh_time: None,
            has_vm: false,
            vm_zone: None,
        };

        let bar = EmojiProgressBar::from_platform_row(&row);
        assert_eq!(bar.steps.len(), 5);
        // All steps should be Completed
        for step in &bar.steps {
            assert_eq!(step.state, ProgressState::Completed);
        }
    }

    #[test]
    fn test_from_platform_row_with_operation() {
        use crate::ui_tabs::platform::{PlatformRow, OperationState};

        let row = PlatformRow {
            project_id: "test".to_string(),
            project_display_name: "Test".to_string(),
            platform_type: "GCP".to_string(),
            gcp_connected: true,
            project_selected: true,
            vm_created: false,
            firewall_updated: false,
            ssh_ready: false,
            operation_state: OperationState::InProgress {
                operation: "Creating VM".to_string(),
                started_at: 0,
            },
            email: None,
            total_project_count: 0,
            selected_project_id: None,
            vm_name: None,
            vm_external_ip: None,
            ssh_private_key: None,
            ssh_public_key: None,
            ssh_keyring_domain: None,
            firewall_status: "".to_string(),
            ssh_status: "".to_string(),
            last_refresh_time: None,
            has_vm: false,
            vm_zone: None,
        };

        let bar = EmojiProgressBar::from_platform_row(&row);
        assert_eq!(bar.steps[2].state, ProgressState::InProgress); // VM step
    }
}
