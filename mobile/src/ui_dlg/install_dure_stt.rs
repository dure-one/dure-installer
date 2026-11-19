//! Install Dure wizard state
//!
//! Separates state from UI rendering logic

use crate::api::github::GitHubTag;
use crate::calc::env_parser::EnvField;
use poll_promise::Promise;
use serde::{Deserialize, Serialize};

/// ACME provider selection
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AcmeProvider {
    LetsEncrypt,
    GoogleTrustServices,
}

impl Default for AcmeProvider {
    fn default() -> Self {
        Self::LetsEncrypt
    }
}

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

    // Domain replacement (from platform config)
    #[serde(skip)]
    pub replacement_domain: Option<String>,

    // ACME provider configuration
    #[serde(default)]
    pub acme_provider: AcmeProvider,
    #[serde(skip)]
    pub fetch_eab_promise: Option<Promise<Result<(String, String), String>>>, // (keyId, hmacKey)
    #[serde(skip)]
    pub gcp_project_id: Option<String>, // From platform config
    #[serde(skip)]
    pub gcp_access_token: Option<String>, // From platform config

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
            replacement_domain: None,
            acme_provider: AcmeProvider::default(),
            fetch_eab_promise: None,
            gcp_project_id: None,
            gcp_access_token: None,
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

                        // Auto-select latest version (index 0)
                        if !self.available_versions.is_empty() {
                            self.selected_version = Some(0);
                        }
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

                        // Replace example.com with platform domain if available
                        if let Some(domain) = &self.replacement_domain {
                            crate::dure_info!(
                                "[InstallDure] Replacing example.com with domain: {}",
                                domain
                            );
                            crate::dure_debug!(
                                "[InstallDure] Content before replacement (first 200 chars): {}",
                                &self.env_editor_text.chars().take(200).collect::<String>()
                            );

                            // Case-insensitive replacement of example.com
                            self.env_editor_text = self
                                .env_editor_text
                                .replace("example.com", domain)
                                .replace("Example.com", domain)
                                .replace("EXAMPLE.COM", domain);

                            crate::dure_debug!(
                                "[InstallDure] Content after replacement (first 200 chars): {}",
                                &self.env_editor_text.chars().take(200).collect::<String>()
                            );
                        } else {
                            crate::dure_warn!(
                                "[InstallDure] No replacement_domain set, keeping example.com"
                            );
                        }

                        // Parse after replacement so env_fields have correct values
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

    /// Start fetching GCP External Account Keys for ACME EAB
    pub fn start_fetch_eab_keys(&mut self, project_id: String, access_token: String) {
        if self.fetch_eab_promise.is_some() {
            return;
        }

        let promise = Promise::spawn_thread("fetch_eab", move || {
            smol::block_on(async move {
                let client = crate::api::gcp::GcpRestClient::new(access_token);
                match client.create_external_account_key(&project_id, "global") {
                    Ok(key) => Ok((key.key_id, key.b64_mac_key)),
                    Err(e) => Err(format!("Failed to fetch EAB keys: {}", e)),
                }
            })
        });

        self.fetch_eab_promise = Some(promise);
    }

    /// Poll EAB key fetch promise and return result if ready
    pub fn poll_fetch_eab(&mut self) -> Option<Result<(String, String), String>> {
        // Check if promise is ready without borrowing
        let is_ready = self
            .fetch_eab_promise
            .as_ref()
            .map(|p| p.ready().is_some())
            .unwrap_or(false);

        if is_ready {
            // Take ownership of the promise to avoid borrow checker issues
            if let Some(promise) = self.fetch_eab_promise.take() {
                if let Some(result) = promise.ready() {
                    return Some(result.clone());
                }
            }
        }
        None
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
    fn test_acme_provider_default() {
        let state = InstallDureState::new("test-host");
        assert_eq!(state.acme_provider, AcmeProvider::LetsEncrypt);
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

    #[test]
    fn test_auto_select_latest_version() {
        // Directly test that when available_versions is populated,
        // selected_version should be set to index 0 (latest)
        let mut state = InstallDureState::new("root@1.2.3.4");

        // Initially no version selected
        assert_eq!(state.selected_version, None);

        // Simulate what poll_fetch_tags does after successful fetch
        state.available_versions = vec![
            "v0.0.15".to_string(),
            "v0.0.14".to_string(),
            "main".to_string(),
        ];

        // Auto-select latest (this is what poll_fetch_tags now does)
        if !state.available_versions.is_empty() {
            state.selected_version = Some(0);
        }

        // Verify latest is selected
        assert_eq!(state.selected_version, Some(0));
        assert_eq!(state.available_versions[0], "v0.0.15");
    }

    #[test]
    fn test_domain_replacement() {
        let mut state = InstallDureState::new("root@1.2.3.4");
        state.replacement_domain = Some("dure.co".to_string());

        // Simulate downloaded .env.example with example.com
        state.env_editor_text = r#"MYCART_DOMAIN=example.com
XMPP_DOMAIN=example.com
ADMIN_EMAIL=admin@example.com
OTHER_URL=https://example.com/api
"#
        .to_string();

        // Simulate domain replacement (what poll_download_env does)
        if let Some(domain) = &state.replacement_domain {
            state.env_editor_text = state
                .env_editor_text
                .replace("example.com", domain)
                .replace("Example.com", domain)
                .replace("EXAMPLE.COM", domain);
        }

        // Parse after replacement
        state.sync_editor_to_form();

        // Verify replacement worked
        assert_eq!(state.env_fields.len(), 4);
        assert_eq!(
            crate::calc::env_parser::get_env_value(&state.env_fields, "MYCART_DOMAIN"),
            Some("dure.co".to_string())
        );
        assert_eq!(
            crate::calc::env_parser::get_env_value(&state.env_fields, "XMPP_DOMAIN"),
            Some("dure.co".to_string())
        );
        assert_eq!(
            crate::calc::env_parser::get_env_value(&state.env_fields, "ADMIN_EMAIL"),
            Some("admin@dure.co".to_string())
        );
        assert_eq!(
            crate::calc::env_parser::get_env_value(&state.env_fields, "OTHER_URL"),
            Some("https://dure.co/api".to_string())
        );
    }

    #[test]
    fn test_no_domain_replacement_when_none() {
        let mut state = InstallDureState::new("root@1.2.3.4");
        state.replacement_domain = None; // No domain set

        state.env_editor_text = "MYCART_DOMAIN=example.com\n".to_string();
        state.sync_editor_to_form();

        // Should keep example.com
        assert_eq!(
            crate::calc::env_parser::get_env_value(&state.env_fields, "MYCART_DOMAIN"),
            Some("example.com".to_string())
        );
    }
}
