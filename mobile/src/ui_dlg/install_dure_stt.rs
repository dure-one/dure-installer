//! Install Dure wizard state
//!
//! Separates state from UI rendering logic

use crate::api::github::GitHubTag;
use crate::calc::env_parser::EnvField;
use poll_promise::Promise;
use serde::{Deserialize, Serialize};

/// Installation step in the wizard flow
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InstallStep {
    /// Step 1: Select version and load .env.example
    VersionSelect,
    /// Step 2: Edit configuration (form or editor)
    ConfigEdit,
    /// Step 3: Installing to remote host
    Installing,
    /// Step 4: Installation complete
    Complete,
    /// Error state with message
    Error(String),
}

impl Default for InstallStep {
    fn default() -> Self {
        Self::VersionSelect
    }
}

/// Install Dure wizard state
#[derive(Serialize, Deserialize)]
pub struct InstallDureState {
    // Host information
    pub host: String, // Format: "root@IP"

    // Step 1: Version selection
    pub step: InstallStep,
    pub available_versions: Vec<String>, // ["v0.0.15", "v0.0.14", "v0.0.13", "main", "test"]
    pub selected_version: Option<usize>, // Index into available_versions

    #[serde(skip)]
    pub fetch_tags_promise: Option<Promise<Result<Vec<GitHubTag>, String>>>,
    pub tags_fetched: bool,

    // Step 2: Configuration
    pub env_editor_text: String,   // Raw .env content for code editor tab
    pub env_fields: Vec<EnvField>, // Parsed fields for form tab
    pub active_tab: usize,         // 0 = Input Form, 1 = Code Editor
    pub image_tag: String,         // "latest" | "test" | "dev"

    #[serde(skip)]
    pub download_env_promise: Option<Promise<Result<String, String>>>,

    // Step 3: Installation (via ViewModel, no promise needed)
    #[serde(skip)]
    pub progress_log: Vec<String>, // Installation progress messages

    // Step 4: Completion
    pub install_success: bool,
    pub error_message: Option<String>,

    // Dialog control
    pub should_close: bool,

    // Pending action (deferred to avoid closure borrow issues)
    pub pending_install: bool,
}

impl InstallDureState {
    /// Create new wizard state for given host
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            step: InstallStep::VersionSelect,
            available_versions: Vec::new(),
            selected_version: None,
            fetch_tags_promise: None,
            tags_fetched: false,
            env_editor_text: String::new(),
            env_fields: Vec::new(),
            active_tab: 0,
            image_tag: "latest".to_string(),
            download_env_promise: None,
            progress_log: Vec::new(),
            install_success: false,
            error_message: None,
            should_close: false,
            pending_install: false,
        }
    }

    /// Start fetching GitHub tags
    pub fn start_fetch_tags(&mut self) {
        if self.fetch_tags_promise.is_some() {
            return;
        }

        let promise = Promise::spawn_thread("fetch_tags", move || {
            smol::block_on(async move {
                match crate::api::github::fetch_latest_tags("dure-one/dure-mycart", 3).await {
                    Ok(tags) => Ok(tags),
                    Err(e) => Err(format!("Failed to fetch tags: {}", e)),
                }
            })
        });

        self.fetch_tags_promise = Some(promise);
    }

    /// Poll fetch tags promise and update state
    pub fn poll_fetch_tags(&mut self) {
        if let Some(promise) = &self.fetch_tags_promise {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(tags) => {
                        // Extract tag names and add main/test branches
                        let mut versions: Vec<String> =
                            tags.iter().map(|t| t.name.clone()).collect();
                        versions.push("main".to_string());
                        versions.push("test".to_string());

                        self.available_versions = versions;
                        self.tags_fetched = true;
                    }
                    Err(e) => {
                        self.step = InstallStep::Error(e.clone());
                    }
                }
                self.fetch_tags_promise = None;
            }
        }
    }

    /// Start downloading .env.example for selected version
    pub fn start_download_env(&mut self) {
        if self.download_env_promise.is_some() {
            return;
        }

        let Some(idx) = self.selected_version else {
            self.step = InstallStep::Error("No version selected".to_string());
            return;
        };

        let version = self.available_versions[idx].clone();
        let promise = Promise::spawn_thread("download_env", move || {
            smol::block_on(async move {
                match crate::api::github::download_env_example(&version).await {
                    Ok(content) => Ok(content),
                    Err(e) => Err(format!("Failed to download .env.example: {}", e)),
                }
            })
        });

        self.download_env_promise = Some(promise);
    }

    /// Poll download promise and update state
    pub fn poll_download_env(&mut self) {
        if let Some(promise) = &self.download_env_promise {
            if let Some(result) = promise.ready() {
                match result {
                    Ok(content) => {
                        self.env_editor_text = content.clone();
                        self.sync_editor_to_form();
                        self.step = InstallStep::ConfigEdit;
                    }
                    Err(e) => {
                        self.step = InstallStep::Error(e.clone());
                    }
                }
                self.download_env_promise = None;
            }
        }
    }

    /// Sync editor text to form fields (parse .env)
    pub fn sync_editor_to_form(&mut self) {
        self.env_fields = crate::calc::env_parser::parse_env_file(&self.env_editor_text);
    }

    /// Sync form fields back to editor text (generate .env)
    pub fn sync_form_to_editor(&mut self) {
        self.env_editor_text = crate::calc::env_parser::generate_env_file(&self.env_fields);
    }

    /// Reset wizard to initial state
    pub fn reset(&mut self) {
        let host = self.host.clone();
        *self = Self::new(host);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_state() {
        let state = InstallDureState::new("root@1.2.3.4");
        assert_eq!(state.host, "root@1.2.3.4");
        assert_eq!(state.step, InstallStep::VersionSelect);
        assert!(state.selected_version.is_none());
        assert_eq!(state.active_tab, 0);
    }

    #[test]
    fn test_sync_editor_to_form() {
        let mut state = InstallDureState::new("root@1.2.3.4");
        state.env_editor_text = "KEY1=value1\nKEY2=value2\n".to_string();
        state.sync_editor_to_form();

        assert_eq!(state.env_fields.len(), 2);
        assert_eq!(state.env_fields[0].key, "KEY1");
        assert_eq!(state.env_fields[1].value, "value2");
    }

    #[test]
    fn test_sync_form_to_editor() {
        let mut state = InstallDureState::new("root@1.2.3.4");
        state.env_fields = vec![
            crate::calc::env_parser::EnvField::new("KEY1", "value1"),
            crate::calc::env_parser::EnvField::new("KEY2", "value2"),
        ];
        state.sync_form_to_editor();

        assert!(state.env_editor_text.contains("KEY1=value1"));
        assert!(state.env_editor_text.contains("KEY2=value2"));
    }

    #[test]
    fn test_state_transitions() {
        let mut state = InstallDureState::new("root@1.2.3.4");

        // Initial state
        assert_eq!(state.step, InstallStep::VersionSelect);

        // After version selected and download started
        state.available_versions = vec!["v0.0.15".to_string(), "main".to_string()];
        state.selected_version = Some(0);
        state.step = InstallStep::ConfigEdit;

        assert_eq!(state.step, InstallStep::ConfigEdit);

        // After install completes
        state.install_success = true;
        state.step = InstallStep::Complete;

        assert_eq!(state.step, InstallStep::Complete);
        assert!(state.install_success);
    }

    #[test]
    fn test_reset() {
        let mut state = InstallDureState::new("root@1.2.3.4");
        state.step = InstallStep::Complete;
        state.install_success = true;

        state.reset();

        assert_eq!(state.step, InstallStep::VersionSelect);
        assert!(!state.install_success);
        assert_eq!(state.host, "root@1.2.3.4");
    }
}
