//! Platform tab - Platform configuration and management

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use eframe::egui;
use egui_material3::{MaterialButton, data_table};

use crate::api::gcp::bigquery::BillingRecord;
use crate::calc::audit;
use crate::config::{AppConfig, CloudPlatformConfig};

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use crate::ui_dlg::platform_gcp::GcpWizard;

use crate::ui_components::{StatusGrid, ItemState, ActionMenu, SvgEmoji, EmojiProgressBar};

/// Platform row data for data table
#[derive(Clone, Debug)]
pub struct PlatformRow {
    // Identity (CHANGED: project_id replaces platform_name)
    pub project_id: String,              // GCP project ID (platform identifier)
    pub project_display_name: String,    // Display name (may differ from ID)
    pub platform_type: String,           // "GCP"

    // Connection state flags (for Steps column)
    pub gcp_connected: bool,    // Has OAuth access token
    pub project_selected: bool, // Has gcp_selected_project_id
    pub vm_created: bool,       // vms.len() > 0
    pub firewall_updated: bool, // Current IP is whitelisted
    pub ssh_ready: bool,        // VM has external_ip.is_some()

    // Drawer content data
    pub email: Option<String>,      // Connected Google account
    pub total_project_count: usize, // Cached from config (not 0!)
    pub selected_project_id: Option<String>,
    pub vm_name: Option<String>,            // First VM name
    pub vm_external_ip: Option<String>,     // First VM external IP (from cache or VM)
    pub ssh_private_key: Option<String>,    // SSH private key from KeePass
    pub ssh_public_key: Option<String>,     // Derived SSH public key for verification
    pub ssh_keyring_domain: Option<String>, // Keyring domain for SSH key
    pub firewall_status: String,            // Cached from config
    pub ssh_status: String,                 // "✓ Ready" or "? No external IP"

    // NEW: Status cache metadata
    pub last_refresh_time: Option<i64>,     // For staleness indicator

    // Operation state tracking (for visual feedback)
    pub operation_state: OperationState,

    // Action button state
    pub has_vm: bool,            // Enable/disable VM operation buttons
    pub vm_zone: Option<String>, // For VM operations (delete, restart, regen)
}

/// Actions that can be triggered from platform table rows
#[derive(Debug, Clone)]
enum PlatformAction {
    UpdateFirewall(String), // project_id
    SelectProject(String),  // project_id
    DeleteVM {
        project_id: String,  // CHANGED from platform_name
        vm_name: String,
        vm_zone: String,
    },
    RegenVM(String),        // project_id
    RestartVM(String),      // project_id
    DeletePlatform(String), // project_id
    Refresh(String),        // NEW: project_id for manual refresh
}

/// Operation state for visual feedback with timestamps
#[derive(Debug, Clone, PartialEq)]
pub enum OperationState {
    Idle,
    InProgress {
        operation: String,
        started_at: i64,
    },
    Completed {
        operation: String,
        completed_at: i64,
    },
    Failed {
        operation: String,
        error: String,
        failed_at: i64,
    },
}

impl Default for OperationState {
    fn default() -> Self {
        Self::Idle
    }
}

/// Platform tab state
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct PlatformTab {
    /// Platform rows for data table
    #[cfg_attr(feature = "serde", serde(skip))]
    rows: Vec<PlatformRow>,

    #[cfg_attr(feature = "serde", serde(skip))]
    loaded: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    load_error: Option<String>,

    // Add dialog state
    #[cfg_attr(feature = "serde", serde(skip))]
    show_add_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_type: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_oauth_url: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_oauth_result: Option<crate::api::gcp::oauth::OAuthResult>,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_oauth_promise:
        Option<poll_promise::Promise<Result<crate::api::gcp::oauth::OAuthResult, String>>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_connected_email: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_project_list: Vec<(String, String)>,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_selected_project: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_create_new: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_new_project_id: String,

    // Init progress state
    #[cfg_attr(feature = "serde", serde(skip))]
    init_in_progress: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    init_platform_name: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    init_progress_log: Vec<String>,

    // GCP wizard
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    #[cfg_attr(feature = "serde", serde(skip))]
    gcp_wizard: Option<GcpWizard>,

    // Track if wizard was open in previous frame (to detect closure)
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    #[cfg_attr(feature = "serde", serde(skip))]
    wizard_was_open: bool,

    // Platform summary cache (platform_name -> summary)
    #[cfg_attr(feature = "serde", serde(skip))]
    platform_summaries: std::collections::HashMap<String, String>,

    // Delete VM dialog state
    #[cfg_attr(feature = "serde", serde(skip))]
    show_delete_vm_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_vm_platform: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_vm_list: Vec<(String, String, String)>, // (name, zone, status)
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_vm_selected: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_vm_confirming: bool,

    // Delete Platform dialog state
    #[cfg_attr(feature = "serde", serde(skip))]
    show_delete_platform_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_platform_name: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_platform_vm_count: usize,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_platform_delete_vms: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_platform_delete_project: bool,

    // Billing dialog state
    #[cfg_attr(feature = "serde", serde(skip))]
    show_billing_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    billing_data: Option<Vec<BillingRecord>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    billing_loading: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    billing_error: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    billing_dataset: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    billing_table: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    billing_project_id: String,

    // Select Project dialog state
    #[cfg_attr(feature = "serde", serde(skip))]
    show_select_project_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    select_project_platform_name: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    select_project_list: Vec<(String, String)>, // (project_id, project_name)
    #[cfg_attr(feature = "serde", serde(skip))]
    select_project_selected: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip))]
    select_project_loading: bool,

    // SSH connection test state (per platform)
    #[cfg_attr(feature = "serde", serde(skip))]
    ssh_test_promises: std::collections::HashMap<
        String,
        poll_promise::Promise<Result<crate::calc::ssh::SshConnectionResult, String>>,
    >,
    #[cfg_attr(feature = "serde", serde(skip))]
    ssh_test_results:
        std::collections::HashMap<String, Result<crate::calc::ssh::SshConnectionResult, String>>,

    // Status refresh state (per platform, keyed by project_id)
    #[cfg_attr(feature = "serde", serde(skip))]
    refresh_promises: std::collections::HashMap<String, poll_promise::Promise<Result<(), String>>>,
}

impl Default for PlatformTab {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            loaded: false,
            load_error: None,
            show_add_dialog: false,
            add_platform_type: "gcp".to_string(),
            add_platform_oauth_url: None,
            add_platform_oauth_result: None,
            add_platform_oauth_promise: None,
            add_platform_connected_email: None,
            add_platform_project_list: Vec::new(),
            add_platform_selected_project: None,
            add_platform_create_new: false,
            add_platform_new_project_id: String::new(),
            init_in_progress: false,
            init_platform_name: None,
            init_progress_log: Vec::new(),
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            gcp_wizard: None,
            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            wizard_was_open: false,
            platform_summaries: std::collections::HashMap::new(),
            show_delete_vm_dialog: false,
            delete_vm_platform: String::new(),
            delete_vm_list: Vec::new(),
            delete_vm_selected: None,
            delete_vm_confirming: false,
            show_delete_platform_dialog: false,
            delete_platform_name: String::new(),
            delete_platform_vm_count: 0,
            delete_platform_delete_vms: false,
            delete_platform_delete_project: false,
            show_billing_dialog: false,
            billing_data: None,
            billing_loading: false,
            billing_error: None,
            billing_dataset: String::new(),
            billing_table: String::new(),
            billing_project_id: String::new(),
            show_select_project_dialog: false,
            select_project_platform_name: String::new(),
            select_project_list: Vec::new(),
            select_project_selected: None,
            select_project_loading: false,
            ssh_test_promises: std::collections::HashMap::new(),
            ssh_test_results: std::collections::HashMap::new(),
            refresh_promises: std::collections::HashMap::new(),
        }
    }
}

/// Get config file path
#[cfg(not(target_arch = "wasm32"))]
fn get_config_path() -> Result<std::path::PathBuf, String> {
    let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
        .ok_or_else(|| "Failed to get project directories".to_string())?;
    Ok(proj_dirs.config_dir().join("config.yml"))
}

/// Load application config with V1 to V2 migration
#[cfg(not(target_arch = "wasm32"))]
fn load_config() -> Result<(AppConfig, std::path::PathBuf), String> {
    use crate::config_migration::{AppConfigV1, backup_config, migrate_config_v1_to_v2};

    let config_path = get_config_path()?;

    if !config_path.exists() {
        // No config exists, create default
        let default_config = AppConfig::default();
        return Ok((default_config, config_path));
    }

    let contents = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    // Try loading as V2 first (current format)
    match serde_yaml::from_str::<AppConfig>(&contents) {
        Ok(config) => {
            // Already V2 format
            Ok((config, config_path))
        }
        Err(_v2_err) => {
            // V2 parse failed, try V1 (with 'name' field)
            match serde_yaml::from_str::<AppConfigV1>(&contents) {
                Ok(v1_config) => {
                    dure_info!(" Detected V1 config, migrating to V2...");

                    // Create backup before migration
                    backup_config(&config_path)?;

                    // Migrate
                    let v2_config = migrate_config_v1_to_v2(v1_config)?;

                    // Save migrated config immediately
                    v2_config.save(&config_path)
                        .map_err(|e| format!("Failed to save migrated config: {}", e))?;

                    dure_info!(" Migrated {} platform(s) to V2 format", v2_config.platforms.len());

                    Ok((v2_config, config_path))
                }
                Err(v1_err) => {
                    // Both V1 and V2 failed - config is corrupted
                    Err(format!("Failed to parse config as V1 or V2: V2 error: {:?}, V1 error: {:?}",
                        _v2_err, v1_err))
                }
            }
        }
    }
}

/// Format connection progress steps with status indicators
// REMOVED: format_steps() - replaced by EmojiProgressBar component
// The visual progress is now shown using emoji indicators via EmojiProgressBar::from_platform_row()

/// Compute firewall whitelist status for a platform
///
/// # Arguments
/// * `access_token` - Valid (non-expired) OAuth access token
/// * `project_id` - GCP project ID to check
fn compute_firewall_status(access_token: Option<&str>, project_id: Option<&str>) -> String {
    if let Some(project) = project_id {
        if let Some(token) = access_token {
            use crate::api::gcp::{GcpRestClient, get_current_ip};

            let client = GcpRestClient::new(token.to_string());

            match get_current_ip() {
                Ok(current_ip) => match client.check_ip_whitelisted(project, &current_ip) {
                    Ok(true) => format!("✓ Whitelisted ({})", current_ip),
                    Ok(false) => "✗ Not whitelisted".to_string(),
                    Err(_) => "? Status unknown".to_string(),
                },
                Err(_) => "? Failed to get IP".to_string(),
            }
        } else {
            "Not connected".to_string()
        }
    } else {
        "No project".to_string()
    }
}

/// Compute SSH readiness status for a platform's VM
///
/// # Arguments
/// * `platform` - Platform configuration
/// * `test_result` - Optional SSH connection test result
fn compute_ssh_status(
    platform: &CloudPlatformConfig,
    test_result: Option<&Result<crate::calc::ssh::SshConnectionResult, String>>,
) -> String {
    // If we have a test result, show it
    if let Some(result) = test_result {
        return match result {
            Ok(conn_result) => {
                if conn_result.success {
                    "✓ Connected".to_string()
                } else {
                    format!("✗ {}", conn_result.message)
                }
            }
            Err(e) => format!("✗ Failed: {}", e),
        };
    }

    // Otherwise, show basic readiness check
    if let Some(vm) = platform.vms.first() {
        if vm.external_ip.is_some() {
            "? Not tested".to_string()
        } else {
            "✗ No external IP".to_string()
        }
    } else {
        "No VM".to_string()
    }
}

/// Helper function to write length-prefixed strings (SSH wire format)
#[cfg(not(target_arch = "wasm32"))]
fn write_ssh_string(buf: &mut Vec<u8>, data: &[u8]) {
    buf.extend_from_slice(&(data.len() as u32).to_be_bytes());
    buf.extend_from_slice(data);
}

/// Convert raw Ed25519 private key bytes (32 bytes) to OpenSSH format
#[cfg(not(target_arch = "wasm32"))]
fn convert_ed25519_to_openssh(raw_bytes: &[u8]) -> Option<String> {
    use ed25519_dalek::SigningKey;

    // Check if it's raw 32-byte Ed25519 key
    if raw_bytes.len() != 32 {
        return None;
    }

    let signing_key = SigningKey::from_bytes(raw_bytes.try_into().ok()?);
    let verifying_key = signing_key.verifying_key();

    let mut key_blob = Vec::new();

    // Magic bytes
    key_blob.extend_from_slice(b"openssh-key-v1\0");

    // Cipher name (none for unencrypted)
    write_ssh_string(&mut key_blob, b"none");

    // KDF name (none for unencrypted)
    write_ssh_string(&mut key_blob, b"none");

    // KDF options (empty for unencrypted)
    write_ssh_string(&mut key_blob, b"");

    // Number of keys (1)
    key_blob.extend_from_slice(&1u32.to_be_bytes());

    // Public key blob
    let mut public_key_blob = Vec::new();
    write_ssh_string(&mut public_key_blob, b"ssh-ed25519");
    write_ssh_string(&mut public_key_blob, verifying_key.as_bytes());
    write_ssh_string(&mut key_blob, &public_key_blob);

    // Private key blob
    let mut private_key_blob = Vec::new();

    // Check bytes (same value twice for unencrypted keys)
    let check = 0x12345678u32;
    private_key_blob.extend_from_slice(&check.to_be_bytes());
    private_key_blob.extend_from_slice(&check.to_be_bytes());

    // Key type
    write_ssh_string(&mut private_key_blob, b"ssh-ed25519");

    // Public key
    write_ssh_string(&mut private_key_blob, verifying_key.as_bytes());

    // Private key (Ed25519 format: 32 bytes private + 32 bytes public)
    let mut ed25519_keypair = Vec::new();
    ed25519_keypair.extend_from_slice(raw_bytes);
    ed25519_keypair.extend_from_slice(verifying_key.as_bytes());
    write_ssh_string(&mut private_key_blob, &ed25519_keypair);

    // Comment (empty)
    write_ssh_string(&mut private_key_blob, b"");

    // Padding (block size is 8 bytes)
    let padding_len = 8 - (private_key_blob.len() % 8);
    for i in 1..=padding_len {
        private_key_blob.push(i as u8);
    }

    // Write private key blob
    write_ssh_string(&mut key_blob, &private_key_blob);

    // Base64 encode
    let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &key_blob);

    // Wrap in PEM format with 70-character lines
    let mut result = String::from("-----BEGIN OPENSSH PRIVATE KEY-----\n");
    for chunk in encoded.as_bytes().chunks(70) {
        result.push_str(std::str::from_utf8(chunk).ok()?);
        result.push('\n');
    }
    result.push_str("-----END OPENSSH PRIVATE KEY-----\n");

    Some(result)
}

/// Read SSH string (length-prefixed) from buffer
#[cfg(not(target_arch = "wasm32"))]
fn read_ssh_string(data: &[u8], offset: &mut usize) -> Option<Vec<u8>> {
    if *offset + 4 > data.len() {
        return None;
    }

    let len = u32::from_be_bytes([
        data[*offset],
        data[*offset + 1],
        data[*offset + 2],
        data[*offset + 3],
    ]) as usize;
    *offset += 4;

    if *offset + len > data.len() {
        return None;
    }

    let result = data[*offset..*offset + len].to_vec();
    *offset += len;
    Some(result)
}

/// Extract public key from OpenSSH private key format
#[cfg(not(target_arch = "wasm32"))]
fn extract_pubkey_from_openssh(openssh_key: &str) -> Option<String> {
    // Remove PEM headers and decode base64
    let key_data = openssh_key
        .lines()
        .filter(|line| !line.contains("BEGIN") && !line.contains("END") && !line.trim().is_empty())
        .collect::<String>();

    let decoded = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        key_data.as_bytes(),
    )
    .ok()?;

    let mut offset = 0;

    // Skip magic bytes "openssh-key-v1\0"
    if decoded.len() < 15 || &decoded[0..15] != b"openssh-key-v1\0" {
        return None;
    }
    offset = 15;

    // Skip cipher name, kdf name, kdf options
    read_ssh_string(&decoded, &mut offset)?;
    read_ssh_string(&decoded, &mut offset)?;
    read_ssh_string(&decoded, &mut offset)?;

    // Skip number of keys (should be 1)
    if offset + 4 > decoded.len() {
        return None;
    }
    offset += 4;

    // Read public key blob
    let public_key_blob = read_ssh_string(&decoded, &mut offset)?;

    // Encode as SSH public key
    let public_key_b64 =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &public_key_blob);
    Some(format!("ssh-ed25519 {} dure-generated", public_key_b64))
}

/// Derive SSH public key from raw Ed25519 private key bytes or OpenSSH format
#[cfg(not(target_arch = "wasm32"))]
fn derive_public_key_from_raw(raw_bytes: &[u8]) -> Option<String> {
    use ed25519_dalek::SigningKey;

    // Try to interpret as OpenSSH format first
    if let Ok(key_str) = String::from_utf8(raw_bytes.to_vec()) {
        if key_str.contains("BEGIN") && key_str.contains("PRIVATE KEY") {
            dure_debug!("Extracting public key from OpenSSH format");
            return extract_pubkey_from_openssh(&key_str);
        }
    }

    // Otherwise, treat as raw 32-byte Ed25519 key
    if raw_bytes.len() != 32 {
        dure_debug!(
            "Key is neither OpenSSH format nor raw 32 bytes (length: {})",
            raw_bytes.len()
        );
        return None;
    }

    let signing_key = SigningKey::from_bytes(raw_bytes.try_into().ok()?);
    let verifying_key = signing_key.verifying_key();
    let public_key_bytes = verifying_key.as_bytes();

    // Encode in SSH public key format
    let mut public_key_ssh = Vec::new();
    write_ssh_string(&mut public_key_ssh, b"ssh-ed25519");
    write_ssh_string(&mut public_key_ssh, public_key_bytes);

    let public_key_b64 =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &public_key_ssh);
    Some(format!("ssh-ed25519 {} dure-generated", public_key_b64))
}

/// Load SSH private key from KeePass database and derive public key
///
/// # Arguments
/// * `keyring_domain` - The domain/identifier for the SSH key in KeePass
///
/// Returns (private_key, public_key)
#[cfg(not(target_arch = "wasm32"))]
fn load_ssh_key_from_keyring(keyring_domain: &Option<String>) -> (Option<String>, Option<String>) {
    use crate::calc::keyring;

    let domain = match keyring_domain.as_ref() {
        Some(d) => {
            dure_debug!("Loading SSH key for domain: {}", d);
            d
        }
        None => {
            dure_debug!("No keyring domain provided");
            return (None, None);
        }
    };

    let kdbx_path = match keyring::get_default_kdbx_path() {
        Ok(p) => {
            dure_debug!("KeePass DB path: {}", p.display());
            p
        }
        Err(e) => {
            dure_debug!("Failed to get kdbx path: {}", e);
            return (None, None);
        }
    };
    let kpkey_path = match keyring::get_default_kpkey_path() {
        Ok(p) => {
            dure_debug!("KPKey path: {}", p.display());
            p
        }
        Err(e) => {
            dure_debug!("Failed to get kpkey path: {}", e);
            return (None, None);
        }
    };

    let keys = match keyring::list_keys(&kdbx_path, Some(&kpkey_path)) {
        Ok(k) => {
            dure_debug!("Found {} keys in keyring", k.len());
            for key in &k {
                dure_debug!("  - Domain: {}, Username: {}, Has SSH: {}", key.domain,
                    key.username,
                    key.ssh_key.is_some()
                );
            }
            k
        }
        Err(e) => {
            dure_debug!("Failed to list keys: {}", e);
            return (None, None);
        }
    };

    // Find the key with matching domain
    let key_entry = match keys.iter().find(|k| &k.domain == domain) {
        Some(e) => {
            dure_debug!("Found matching key entry");
            e
        }
        None => {
            dure_debug!("No key found for domain: {}", domain);
            return (None, None);
        }
    };

    // Try to get SSH key from binary attachment
    if let Some(ssh_key_bytes) = &key_entry.ssh_key {
        dure_debug!("SSH key bytes length: {}", ssh_key_bytes.len());

        // Derive public key from raw bytes
        let public_key = derive_public_key_from_raw(ssh_key_bytes);
        if let Some(ref pk) = public_key {
            dure_debug!("Derived public key: {}", pk);
        } else {
            dure_debug!("Failed to derive public key");
        }

        // Try to interpret as UTF-8 string first (already in OpenSSH format)
        if let Ok(key_str) = String::from_utf8(ssh_key_bytes.clone()) {
            if key_str.contains("BEGIN") && key_str.contains("PRIVATE KEY") {
                dure_debug!("Key already in OpenSSH format");
                return (Some(key_str), public_key);
            }
        }

        // Otherwise, try to convert raw Ed25519 bytes to OpenSSH format
        dure_debug!("Converting raw bytes to OpenSSH format");
        let private_key = convert_ed25519_to_openssh(ssh_key_bytes);
        (private_key, public_key)
    } else {
        dure_debug!("Key entry has no SSH key attachment");
        (None, None)
    }
}

#[cfg(target_arch = "wasm32")]
fn load_ssh_key_from_keyring(_keyring_domain: &Option<String>) -> (Option<String>, Option<String>) {
    (None, None)
}

/// Fetch total project count from GCP API
///
/// # Arguments
/// * `access_token` - Valid (non-expired) OAuth access token. Caller should use get_valid_access_token() to ensure token is fresh.
fn fetch_project_count(access_token: Option<&str>) -> usize {
    if let Some(token) = access_token {
        use crate::api::gcp::GcpRestClient;
        let client = GcpRestClient::new(token.to_string());

        match client.list_projects(None) {
            Ok(list) => list.projects.len(),
            Err(e) => {
                dure_debug!("Failed to fetch project count: {}", e);
                0
            }
        }
    } else {
        0
    }
}

/// Render drawer content showing platform hierarchy
fn render_drawer_content(ui: &mut egui::Ui, row: &PlatformRow) {
    ui.add_space(8.0);

    let mut grid = StatusGrid::new();

    // Connection info
    if let Some(email) = &row.email {
        grid.add_item(
            SvgEmoji::Email,
            "Email",
            format!("{} ({} projects)", email, row.total_project_count),
            None,
        );
    } else {
        grid.add_item(SvgEmoji::Email, "Email", "Not connected", None);
    }

    // Project info
    if let Some(project_id) = &row.selected_project_id {
        grid.add_item(SvgEmoji::Project, "Project", project_id, None);

        // Refresh staleness
        if let Some(last_refresh) = row.last_refresh_time {
            let elapsed = chrono::Utc::now().timestamp() - last_refresh;
            let (time_str, state) = if elapsed < 60 {
                ("just now".to_string(), None)
            } else if elapsed < 3600 {
                (format!("{} min ago", elapsed / 60), None)
            } else if elapsed < 86400 {
                (format!("{} hours ago", elapsed / 3600), Some(ItemState::Warning))
            } else {
                (format!("{} days ago", elapsed / 86400), Some(ItemState::Warning))
            };
            grid.add_item(SvgEmoji::Clock, "Refreshed", time_str, state);
        }

        // VM details
        if let Some(vm_name) = &row.vm_name {
            grid.add_item(SvgEmoji::VM, "VM", vm_name, None);

            // IP address
            grid.add_item(
                SvgEmoji::Network,
                "IP",
                row.vm_external_ip
                    .as_deref()
                    .unwrap_or("⚠ No external IP"),
                if row.vm_external_ip.is_none() {
                    Some(ItemState::Warning)
                } else {
                    None
                },
            );

            // Firewall status (check operation state)
            let (firewall_value, firewall_state) = match &row.operation_state {
                OperationState::InProgress { operation, .. }
                    if operation.to_lowercase().contains("firewall") =>
                {
                    ("Updating...".to_string(), Some(ItemState::InProgress))
                }
                OperationState::Failed { operation, error, .. }
                    if operation.to_lowercase().contains("firewall") =>
                {
                    (error.clone(), Some(ItemState::Error))
                }
                _ => (row.firewall_status.clone(), None),
            };
            grid.add_item(SvgEmoji::Firewall, "Firewall", firewall_value, firewall_state);

            // SSH status
            grid.add_item(SvgEmoji::Key, "SSH", &row.ssh_status, None);
        } else {
            grid.add_item(SvgEmoji::VM, "VM", "— No VM created", None);
        }
    } else {
        grid.add_item(SvgEmoji::Project, "Project", "— No project selected", None);
    }

    grid.show(ui);

    // SSH action menu (if available)
    if let (Some(external_ip), Some(private_key)) =
        (&row.vm_external_ip, &row.ssh_private_key)
    {
        ui.add_space(8.0);

        let ssh_command = format!(
            "K=$(mktemp) && cat > $K <<'EOF'\n{}\nEOF\nchmod 600 $K && ssh -i $K root@{} && rm $K",
            private_key.trim(),
            external_ip
        );

        let mut menu = ActionMenu::new("📋 SSH").with_icon(SvgEmoji::Terminal);
        menu.add_action("Copy SSH Command");
        menu.add_action("Copy Private Key");
        menu.add_action("Copy IP Address");

        if let Some(action_idx) = menu.show(ui) {
            let text_to_copy = match action_idx {
                0 => &ssh_command,
                1 => private_key,
                2 => external_ip,
                _ => return,
            };

            ui.ctx().copy_text(text_to_copy.to_string());
        }
    } else if row.vm_external_ip.is_some() && row.ssh_keyring_domain.is_some() {
        ui.add_space(8.0);
        ui.colored_label(
            egui::Color32::from_rgb(255, 152, 0),
            "⚠ SSH key not found in keyring",
        );
    }
}

impl PlatformTab {
    /// Render the platform tab UI
    pub fn ui(&mut self, ui: &mut egui::Ui, mut vm: Option<&mut crate::viewmodel::ViewModel>) {
        // ViewModel event processing (MVVM pattern)
        if let Some(ref mut vm) = vm {
            // Process events first
            let events = vm.poll_events(ui.ctx());
            for event in events {
                use crate::viewmodel::ViewModelEvent;
                use crate::viewmodel::platform::PlatformEvent;

                match event {
                    ViewModelEvent::Platform(PlatformEvent::BillingFetched { records, .. }) => {
                        self.billing_data = Some(records);
                        self.billing_loading = false;
                    }
                    ViewModelEvent::Platform(PlatformEvent::FirewallUpdated {
                        platform_name,
                        whitelisted_ip,
                    }) => {
                        dure_debug!("✓ Successfully added {} to firewall whitelist", whitelisted_ip);

                        // Incremental update: Find and update specific row
                        if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                            row.operation_state = OperationState::Completed {
                                operation: "firewall".to_string(),
                                completed_at: chrono::Utc::now().timestamp(),
                            };
                            row.firewall_status = format!("✅ Whitelisted ({})", whitelisted_ip);
                            row.firewall_updated = true;
                        }
                        // Note: NO self.loaded = false! Incremental update only
                    }
                    ViewModelEvent::Platform(PlatformEvent::VMRestarted { platform_name, vm_name }) => {
                        dure_info!("✓ VM {} restarted successfully", vm_name);

                        // Incremental update
                        if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                            row.operation_state = OperationState::Completed {
                                operation: "restart".to_string(),
                                completed_at: chrono::Utc::now().timestamp(),
                            };
                        }
                        // Note: NO self.loaded = false!
                    }
                    ViewModelEvent::Platform(PlatformEvent::VMRegenerated {
                        vm_name,
                        message,
                        ..
                    }) => {
                        dure_info!(" {}", message);
                        // Refresh to show updated VM details
                        self.loaded = false;
                        self.load_error = None;
                    }
                    ViewModelEvent::Platform(PlatformEvent::VMsScanned {
                        platform_name,
                        vm_count,
                    }) => {
                        dure_info!("✓ Scanned and imported {} VMs for platform '{}'", vm_count, platform_name);

                        // Update operation state
                        if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                            row.operation_state = OperationState::Completed {
                                operation: "scan".to_string(),
                                completed_at: chrono::Utc::now().timestamp(),
                            };
                        }

                        // Trigger reload to show imported VMs
                        self.loaded = false;
                        self.load_error = None;
                    }
                    ViewModelEvent::Platform(PlatformEvent::VMCreated {
                        platform_name,
                        vm_name,
                        external_ip,
                    }) => {
                        dure_info!("✓ VM '{}' created successfully with IP {}", vm_name, external_ip);

                        // Incremental update
                        if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                            row.operation_state = OperationState::Completed {
                                operation: "vm".to_string(),
                                completed_at: chrono::Utc::now().timestamp(),
                            };
                            row.vm_name = Some(vm_name);
                            row.vm_external_ip = Some(external_ip);
                            row.vm_created = true;
                            row.has_vm = true;
                        }
                        // Trigger full reload to update config-backed data
                        self.loaded = false;
                    }
                    ViewModelEvent::Platform(PlatformEvent::VMDeleted {
                        platform_name,
                        vm_name,
                    }) => {
                        dure_info!("✓ VM {} deleted successfully", vm_name);

                        // Update operation state before reload
                        if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                            row.operation_state = OperationState::Completed {
                                operation: "delete_vm".to_string(),
                                completed_at: chrono::Utc::now().timestamp(),
                            };
                        }

                        // Keep config update and reload logic
                        if let Ok((mut app_config, config_path)) = load_config() {
                            if let Some(platform) = app_config
                                .platforms
                                .iter_mut()
                                .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
                            {
                                platform.vms.retain(|vm| vm.name != vm_name);

                                if let Err(e) = app_config.save(&config_path) {
                                    self.load_error = Some(format!("Failed to save config: {}", e));
                                } else {
                                    dure_info!("✓ Config updated, refreshing table");
                                    self.loaded = false;
                                    self.load_error = None;
                                }
                            }
                        }
                    }
                    ViewModelEvent::Platform(PlatformEvent::ProjectsListed {
                        platform_name,
                        projects,
                    }) => {
                        dure_debug!("✓ Projects listed for {}: {} projects", platform_name,
                            projects.len()
                        );
                        self.select_project_list = projects;
                        self.select_project_loading = false;

                        // Show the dialog now that projects are loaded
                        if !self.show_select_project_dialog {
                            self.show_select_project_dialog = true;
                        }
                    }
                    ViewModelEvent::Platform(PlatformEvent::ProjectSelected {
                        project_id, ..
                    }) => {
                        dure_info!(" Project selected: {}", project_id);
                        // Refresh spreadsheet to show updated project
                        self.loaded = false;
                        self.load_error = None;
                    }
                    ViewModelEvent::Platform(PlatformEvent::PlatformAdded {
                        platform_name,
                        platform_type,
                    }) => {
                        dure_info!(" Platform '{}' ({}) added", platform_name, platform_type);
                        // Refresh spreadsheet to show new platform
                        self.loaded = false;
                        self.load_error = None;
                    }
                    ViewModelEvent::Platform(PlatformEvent::PlatformDeleted {
                        platform_name,
                        vm_count,
                    }) => {
                        dure_info!(" Platform '{}' deleted ({} VMs)", platform_name, vm_count);
                        // Refresh spreadsheet to show removal
                        self.loaded = false;
                        self.load_error = None;
                    }
                    ViewModelEvent::Platform(PlatformEvent::Error { operation, error }) => {
                        if operation == "fetch_billing" {
                            self.billing_error = Some(error);
                            self.billing_loading = false;
                        } else if operation == "update_firewall" {
                            self.load_error = Some(format!("Failed to update firewall: {}", error));
                        } else if operation == "restart_vm" {
                            self.load_error = Some(format!("Failed to restart VM: {}", error));
                        } else if operation == "delete_vm" {
                            self.load_error = Some(format!("Failed to delete VM: {}", error));
                        }
                    }
                    ViewModelEvent::Platform(PlatformEvent::OperationFailed {
                        platform_name,
                        operation,
                        error,
                    }) => {
                        dure_error!("✗ Operation '{}' failed for {}: {}", operation, platform_name, error);

                        // Update row to show error state
                        if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                            row.operation_state = OperationState::Failed {
                                operation: operation.clone(),
                                error: error.clone(),
                                failed_at: chrono::Utc::now().timestamp(),
                            };
                        }
                    }
                    _ => {}
                }
            }

            // Show active operations with progress bars
            for (_op_id, progress) in vm.active_operations() {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::ProgressBar::new(progress.progress)
                            .text(format!("{}: {}", progress.operation, progress.status))
                            .desired_width(400.0),
                    );
                });
            }

            // Show recent errors
            if let Some(error) = vm
                .recent_errors()
                .iter()
                .filter(|e| e.actor == "platform")
                .rev()
                .next()
            {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 100, 100),
                    format!("⚠ Error in {}: {}", error.operation, error.error),
                );
                ui.add_space(4.0);
            }
        }

        // Auto-clear Completed/Failed operation states
        let now = chrono::Utc::now().timestamp();
        for row in &mut self.rows {
            match &row.operation_state {
                OperationState::Completed { completed_at, .. } if now - completed_at > 3 => {
                    row.operation_state = OperationState::Idle;
                }
                OperationState::Failed { failed_at, .. } if now - failed_at > 10 => {
                    row.operation_state = OperationState::Idle;
                }
                _ => {}
            }
        }

        // Request repaint to update UI when states auto-clear
        ui.ctx().request_repaint_after(std::time::Duration::from_secs(1));

        ui.heading("Cloud Platforms");
        ui.add_space(4.0);
        ui.label(
            "Manage cloud service platforms (GCP, Firebase, Supabase) for deployment and hosting.",
        );
        ui.add_space(8.0);

        // Poll SSH test promises (TODO: Replace with ViewModel event processing)
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut completed = Vec::new();
            for (platform_name, promise) in &self.ssh_test_promises {
                if let Some(result) = promise.ready() {
                    completed.push(platform_name.clone());
                    self.ssh_test_results
                        .insert(platform_name.clone(), result.clone());
                }
            }
            for platform_name in completed {
                self.ssh_test_promises.remove(&platform_name);
            }
        }

        // Poll refresh promises
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut completed_refreshes = Vec::new();
            for (project_id, promise) in &self.refresh_promises {
                if let Some(result) = promise.ready() {
                    match result {
                        Ok(_) => {
                            // Reload data to show fresh status
                            self.loaded = false;
                        }
                        Err(e) => {
                            dure_debug!("Refresh failed for {}: {}", project_id, e);
                        }
                    }
                    completed_refreshes.push(project_id.clone());
                }
            }
            for project_id in completed_refreshes {
                self.refresh_promises.remove(&project_id);
            }
        }

        // Action buttons
        if ui.add(MaterialButton::filled("Add Platform")).clicked() {
            self.show_add_dialog = true;
            self.add_platform_type = "gcp".to_string();
        }
        ui.add_space(8.0);

        // Table rendering
        if !self.loaded {
            self.load_rows();
        }

        if let Some(error) = &self.load_error {
            ui.colored_label(
                egui::Color32::from_rgb(255, 0, 0),
                format!("Error: {}", error),
            );
        } else if self.rows.is_empty() {
            ui.label("No platforms configured. Click 'Add Platform' to get started.");
        } else {
            // Calculate responsive column widths
            // Reserve space for borders, padding, and scrollbar
            let available_width = ui.available_width() - 40.0;
            let base_width = 740.0; // Base design width (reduced to fit with borders)
            let width_ratio = (available_width / base_width).min(1.5).max(0.8);

            // Build data table
            let table_id = egui::Id::new("platform_table");

            // Initialize drawer state (all open by default on first load)
            use egui_material3::datatable::DataTableState;
            let state: DataTableState = ui.data_mut(|d| {
                let existing = d.get_persisted::<DataTableState>(table_id);
                match existing {
                    Some(state) => state,
                    None => {
                        // First load - initialize with all drawers open
                        let mut state = DataTableState::default();
                        state.drawer_open_rows = (0..self.rows.len()).collect();
                        state
                    }
                }
            });

            // Store state back
            ui.data_mut(|d| d.insert_persisted(table_id, state));

            let mut table = data_table()
                .id(table_id)
                .allow_selection(false)
                .allow_drawer(true)
                .column("Project", 150.0 * width_ratio, false)
                .column("Type", 80.0 * width_ratio, false)
                .column("Steps", 250.0 * width_ratio, false)
                .column("Operations", 260.0 * width_ratio, false);

            for (idx, row) in self.rows.iter().enumerate() {
                let row_for_cells = row.clone();
                let row_for_drawer = row.clone();
                let row_for_actions = row.clone();

                table = table.row(move |r| {
                    r.cell(&row_for_cells.project_id)
                        .cell(&row_for_cells.platform_type)
                        .widget_cell(move |ui| {
                            let progress = EmojiProgressBar::from_platform_row(&row_for_cells)
                                .compact(true);
                            progress.show(ui);
                        })
                        .widget_cell(move |ui| {
                            egui::ScrollArea::horizontal()
                                .id_salt(format!("operations_scroll_{}", idx))
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.spacing_mut().item_spacing.x = 2.0;
                                        ui.style_mut().spacing.button_padding =
                                            egui::vec2(6.0, 2.0);

                                        // Check if any operation in progress
                                        let operation_in_progress = matches!(
                                            row_for_actions.operation_state,
                                            OperationState::InProgress { .. }
                                        );

                                        // 0. Refresh (always enabled)
                                        if ui
                                            .add(MaterialButton::outlined("Refresh").small())
                                            .on_hover_text("Refresh platform data")
                                            .clicked()
                                        {
                                            ui.data_mut(|d| {
                                                d.insert_temp(
                                                    egui::Id::new("platform_action_refresh"),
                                                    row_for_actions.project_id.clone(),
                                                )
                                            });
                                        }

                                        // Disable other buttons during operations
                                        ui.add_enabled_ui(!operation_in_progress, |ui| {
                                        // 1. Add VM
                                        #[cfg(not(any(
                                            target_os = "android",
                                            target_arch = "wasm32"
                                        )))]
                                        if ui
                                            .add_enabled(
                                                !row_for_actions.has_vm
                                                    && row_for_actions.project_selected,
                                                MaterialButton::outlined("Add VM").small(),
                                            )
                                            .on_hover_text("Add VM")
                                            .clicked()
                                        {
                                            ui.data_mut(|d| {
                                                d.insert_temp(
                                                    egui::Id::new("platform_action_add_vm"),
                                                    row_for_actions.project_id.clone(),
                                                )
                                            });
                                        }

                                        // 1.5. Scan VMs
                                        if ui
                                            .add_enabled(
                                                row_for_actions.project_selected,
                                                MaterialButton::outlined("Scan VMs").small(),
                                            )
                                            .on_hover_text("Scan and import existing VMs from GCP")
                                            .clicked()
                                        {
                                            ui.data_mut(|d| {
                                                d.insert_temp(
                                                    egui::Id::new("platform_action_scan_vms"),
                                                    row_for_actions.project_id.clone(),
                                                )
                                            });
                                        }

                                        // 2. Firewall
                                        if ui
                                            .add_enabled(
                                                row_for_actions.project_selected
                                                    && !row_for_actions.firewall_updated,
                                                MaterialButton::outlined("Firewall").small(),
                                            )
                                            .on_hover_text("Update Firewall")
                                            .clicked()
                                        {
                                            ui.data_mut(|d| {
                                                d.insert_temp(
                                                    egui::Id::new(
                                                        "platform_action_update_firewall",
                                                    ),
                                                    row_for_actions.project_id.clone(),
                                                )
                                            });
                                        }

                                        // 3. Restart
                                        if ui
                                            .add_enabled(
                                                row_for_actions.has_vm,
                                                MaterialButton::outlined("Restart").small(),
                                            )
                                            .on_hover_text("Restart VM")
                                            .clicked()
                                        {
                                            ui.data_mut(|d| {
                                                d.insert_temp(
                                                    egui::Id::new("platform_action_restart_vm"),
                                                    row_for_actions.project_id.clone(),
                                                )
                                            });
                                        }

                                        // 4. Del VM
                                        if ui
                                            .add_enabled(
                                                row_for_actions.has_vm,
                                                MaterialButton::outlined("Del VM").small(),
                                            )
                                            .on_hover_text("Delete VM")
                                            .clicked()
                                        {
                                            ui.data_mut(|d| {
                                                d.insert_temp(
                                                    egui::Id::new("platform_action_delete_vm"),
                                                    (
                                                        row_for_actions.project_id.clone(),
                                                        row_for_actions
                                                            .vm_name
                                                            .clone()
                                                            .unwrap_or_default(),
                                                        row_for_actions
                                                            .vm_zone
                                                            .clone()
                                                            .unwrap_or_default(),
                                                    ),
                                                )
                                            });
                                        }

                                        // 5. Regen
                                        // if ui.add_enabled(row_for_actions.has_vm,
                                        //     MaterialButton::outlined("Regen").small()).on_hover_text("Regenerate VM").clicked() {
                                        //     ui.data_mut(|d| d.insert_temp(
                                        //         egui::Id::new("platform_action_regen_vm"),
                                        //         row_for_actions.project_id.clone()
                                        //     ));
                                        // }

                                        // 6. Billing
                                        #[cfg(not(any(
                                            target_os = "android",
                                            target_arch = "wasm32"
                                        )))]
                                        if ui
                                            .add_enabled(
                                                row_for_actions.project_selected && row_for_actions.selected_project_id.is_some(),
                                                MaterialButton::outlined("Billing").small(),
                                            )
                                            .on_hover_text("Estimated Billing")
                                            .clicked()
                                        {
                                            ui.data_mut(|d| {
                                                d.insert_temp(
                                                    egui::Id::new("platform_action_billing_name"),
                                                    row_for_actions.project_id.clone(),
                                                );
                                                if let Some(project_id) = &row_for_actions.selected_project_id {
                                                    d.insert_temp(
                                                        egui::Id::new("platform_action_billing_project"),
                                                        project_id.clone(),
                                                    );
                                                }
                                            });
                                        }

                                        // 7. Delete
                                        if ui
                                            .add(MaterialButton::outlined("Delete").small())
                                            .on_hover_text("Delete Platform")
                                            .clicked()
                                        {
                                            ui.data_mut(|d| {
                                                d.insert_temp(
                                                    egui::Id::new(
                                                        "platform_action_delete_platform",
                                                    ),
                                                    row_for_actions.project_id.clone(),
                                                )
                                            });
                                        }
                                        }); // End add_enabled_ui
                                    });
                                });
                        })
                        .drawer(move |ui| {
                            render_drawer_content(ui, &row_for_drawer);
                        })
                });
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                table.show(ui);
            });

            // Process pending actions from button clicks
            // Refresh action (available on all platforms)
            if let Some(platform_name) =
                ui.data(|d| d.get_temp::<String>(egui::Id::new("platform_action_refresh")))
            {
                self.loaded = false;

                // Trigger SSH connection test for this platform
                #[cfg(not(target_arch = "wasm32"))]
                self.execute_test_connection(platform_name.clone());

                ui.data_mut(|d| d.remove::<String>(egui::Id::new("platform_action_refresh")));
            }

            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            {
                if let Some(platform_name) = ui.data(|d| {
                    d.get_temp::<String>(egui::Id::new("platform_action_update_firewall"))
                }) {
                    // Optimistic update: Set InProgress immediately
                    if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                        row.operation_state = OperationState::InProgress {
                            operation: "Updating firewall".to_string(),
                            started_at: chrono::Utc::now().timestamp(),
                        };
                    }

                    self.update_firewall(platform_name, vm.as_deref_mut());
                    ui.data_mut(|d| {
                        d.remove::<String>(egui::Id::new("platform_action_update_firewall"))
                    });
                }

                if let Some(platform_name) = ui.data(|d| {
                    d.get_temp::<String>(egui::Id::new("platform_action_scan_vms"))
                }) {
                    // Optimistic update
                    if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                        row.operation_state = OperationState::InProgress {
                            operation: "Scanning VMs".to_string(),
                            started_at: chrono::Utc::now().timestamp(),
                        };
                    }

                    self.scan_vms(platform_name, vm.as_deref_mut());
                    ui.data_mut(|d| {
                        d.remove::<String>(egui::Id::new("platform_action_scan_vms"))
                    });
                }

                if let Some((platform_name, vm_name, vm_zone)) = ui.data(|d| {
                    d.get_temp::<(String, String, String)>(egui::Id::new(
                        "platform_action_delete_vm",
                    ))
                }) {
                    // Optimistic update
                    if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                        row.operation_state = OperationState::InProgress {
                            operation: format!("Deleting VM {}", vm_name),
                            started_at: chrono::Utc::now().timestamp(),
                        };
                    }

                    self.show_delete_vm_confirmation(platform_name, vm_name, vm_zone);
                    ui.data_mut(|d| {
                        d.remove::<(String, String, String)>(egui::Id::new(
                            "platform_action_delete_vm",
                        ))
                    });
                }

                if let Some(platform_name) =
                    ui.data(|d| d.get_temp::<String>(egui::Id::new("platform_action_regen_vm")))
                {
                    // Find platform and get vm_name
                    if let Ok((app_config, _)) = load_config() {
                        if let Some(platform) = app_config
                            .platforms
                            .iter()
                            .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
                        {
                            if let Some(vm_cfg) = platform.vms.first() {
                                self.regenerate_vm(
                                    platform_name,
                                    vm_cfg.name.clone(),
                                    vm.as_deref_mut(),
                                );
                            }
                        }
                    }
                    ui.data_mut(|d| d.remove::<String>(egui::Id::new("platform_action_regen_vm")));
                }

                if let Some(platform_name) =
                    ui.data(|d| d.get_temp::<String>(egui::Id::new("platform_action_restart_vm")))
                {
                    // Optimistic update
                    if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                        row.operation_state = OperationState::InProgress {
                            operation: "Restarting VM".to_string(),
                            started_at: chrono::Utc::now().timestamp(),
                        };
                    }

                    // Find platform and get vm_name and zone
                    if let Ok((app_config, _)) = load_config() {
                        if let Some(platform) = app_config
                            .platforms
                            .iter()
                            .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
                        {
                            if let Some(vm_config) = platform.vms.first() {
                                self.restart_vm(
                                    platform_name.clone(),
                                    vm_config.name.clone(),
                                    vm_config.zone.clone(),
                                    vm.as_deref_mut(),
                                );
                            }
                        }
                    }
                    ui.data_mut(|d| {
                        d.remove::<String>(egui::Id::new("platform_action_restart_vm"))
                    });
                }

                if let Some(platform_name) =
                    ui.data(|d| d.get_temp::<String>(egui::Id::new("platform_action_add_vm")))
                {
                    self.show_gcp_wizard(platform_name);
                    ui.data_mut(|d| d.remove::<String>(egui::Id::new("platform_action_add_vm")));
                }

                if let Some(platform_name) =
                    ui.data(|d| d.get_temp::<String>(egui::Id::new("platform_action_billing_name")))
                {
                    if let Some(project_id) =
                        ui.data(|d| d.get_temp::<String>(egui::Id::new("platform_action_billing_project")))
                    {
                        self.show_billing_dialog = true;
                        self.billing_project_id = project_id.clone();
                        self.fetch_billing_data(vm.as_deref_mut(), Some(project_id));
                        ui.data_mut(|d| {
                            d.remove::<String>(egui::Id::new("platform_action_billing_name"));
                            d.remove::<String>(egui::Id::new("platform_action_billing_project"));
                        });
                    }
                }
            }

            if let Some(platform_name) =
                ui.data(|d| d.get_temp::<String>(egui::Id::new("platform_action_delete_platform")))
            {
                self.show_delete_platform_confirmation(platform_name);
                ui.data_mut(|d| {
                    d.remove::<String>(egui::Id::new("platform_action_delete_platform"))
                });
            }
        }

        // Add platform dialog
        if self.show_add_dialog {
            self.render_add_dialog(ui.ctx(), vm.as_deref_mut());
        }

        // Delete Platform dialog
        if self.show_delete_platform_dialog {
            self.render_delete_platform_dialog(ui.ctx(), vm.as_deref_mut());
        }

        // Select Project dialog
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        if self.show_select_project_dialog {
            self.render_select_project_dialog(ui.ctx(), vm.as_deref_mut());
        }

        // Delete VM dialog
        if self.show_delete_vm_dialog {
            self.render_delete_vm_dialog(ui.ctx(), vm.as_deref_mut());
        }

        // Billing dialog
        if self.show_billing_dialog {
            self.render_billing_dialog(ui.ctx(), vm.as_deref_mut());
        }

        // Init progress display
        if self.init_in_progress {
            self.render_init_progress(ui);
        }

        // GCP wizard dialog
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        {
            let wizard_is_open = self.gcp_wizard.is_some();

            if let Some(wizard) = &mut self.gcp_wizard {
                wizard.ui(ui.ctx());
            }

            // Detect wizard closure - if it was open and now closed, refresh
            if self.wizard_was_open && !wizard_is_open {
                dure_info!(" GCP wizard closed, refreshing platform spreadsheet");
                self.loaded = false;
            }

            self.wizard_was_open = wizard_is_open;
        }
    }

    fn load_rows(&mut self) {
        self.rows.clear();
        self.load_error = None;

        #[cfg(not(target_arch = "wasm32"))]
        {
            match load_config() {
                Ok((mut app_config, config_path)) => {
                    // Iterate by index to avoid borrow checker issues
                    let platform_count = app_config.platforms.len();
                    for idx in 0..platform_count {
                        // Only show GCP platforms for now
                        if app_config.platforms[idx].platform_type != "gcp" {
                            continue;
                        }

                        // Get valid access token (refreshes if expired)
                        // Note: get_valid_access_token() saves config if it refreshes the token
                        let access_token = if app_config.platforms[idx]
                            .gcp_oauth_access_token
                            .is_some()
                        {
                            match self.get_valid_access_token(&mut app_config, idx, &config_path) {
                                Ok(token) => Some(token),
                                Err(e) => {
                                    let project_id = app_config.platforms[idx].gcp_selected_project_id.as_deref().unwrap_or("unknown");
                                    dure_debug!("Failed to get valid access token for project '{}': {}", project_id, e
                                    );
                                    None
                                }
                            }
                        } else {
                            None
                        };

                        // Borrow platform after get_valid_access_token
                        let platform = &app_config.platforms[idx];

                        // Compute firewall status (synchronous check)
                        let firewall_status_str = compute_firewall_status(
                            access_token.as_deref(),
                            platform.gcp_selected_project_id.as_deref(),
                        );

                        // firewall_updated is true if IP is whitelisted (status starts with ✓)
                        let firewall_updated = firewall_status_str.starts_with("✓");

                        // Compute SSH status from cached results only (no blocking)
                        let project_id_key = platform.gcp_selected_project_id.as_ref()
                            .map(|s| s.as_str()).unwrap_or("unknown");
                        let ssh_status_str =
                            compute_ssh_status(platform, self.ssh_test_results.get(project_id_key));

                        // ssh_ready should match ssh_status: only true if actually connected
                        let ssh_ready =
                            if let Some(result) = self.ssh_test_results.get(project_id_key) {
                                matches!(result, Ok(conn_result) if conn_result.success)
                            } else {
                                false
                            };

                        // Load SSH private key from KeePass if VM exists (quick local operation)
                        let (ssh_private_key, ssh_public_key, ssh_keyring_domain) =
                            if let Some(vm) = platform.vms.first() {
                                let keyring_domain = vm.ssh_key_name.clone();
                                let (private_key, public_key) =
                                    load_ssh_key_from_keyring(&keyring_domain);
                                (private_key, public_key, keyring_domain)
                            } else {
                                (None, None, None)
                            };

                        let row = PlatformRow {
                            // NEW: Use project_id as identifier
                            project_id: platform.gcp_selected_project_id.clone()
                                .unwrap_or_else(|| "unknown".to_string()),
                            project_display_name: platform.gcp_selected_project_id.clone()
                                .unwrap_or_else(|| "unknown".to_string()),
                            platform_type: "GCP".to_string(),

                            // Compute state flags
                            gcp_connected: platform.gcp_oauth_access_token.is_some(),
                            project_selected: platform.gcp_selected_project_id.is_some(),
                            vm_created: !platform.vms.is_empty(),
                            firewall_updated,
                            ssh_ready,

                            // Extract drawer data
                            email: platform.gcp_connected_email.clone(),

                            // FIX: Use cached project count (not 0!)
                            total_project_count: platform.cached_total_project_count.unwrap_or(0),

                            selected_project_id: platform.gcp_selected_project_id.clone(),
                            vm_name: platform.vms.first().map(|vm| vm.name.clone()),

                            // Use cached external IP if available, fall back to VM data
                            vm_external_ip: platform.cached_vm_external_ip.clone()
                                .or_else(|| platform.vms.first().and_then(|vm| vm.external_ip.clone())),

                            ssh_private_key,
                            ssh_public_key,
                            ssh_keyring_domain,

                            // Use cached firewall status
                            firewall_status: platform.cached_firewall_status.clone()
                                .unwrap_or_else(|| firewall_status_str),

                            ssh_status: ssh_status_str,

                            // NEW: Cache metadata
                            last_refresh_time: platform.last_status_refresh,

                            // Operation state tracking
                            operation_state: OperationState::Idle,

                            // Action button state
                            has_vm: !platform.vms.is_empty(),
                            vm_zone: platform.vms.first().map(|vm| vm.zone.clone()),
                        };

                        self.rows.push(row);

                        // Trigger SSH connection test in background if VM has external IP
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let project_id = platform.gcp_selected_project_id.clone()
                                .unwrap_or_else(|| "unknown".to_string());
                            if platform
                                .vms
                                .first()
                                .and_then(|vm| vm.external_ip.as_ref())
                                .is_some()
                                && !self.ssh_test_results.contains_key(&project_id)
                                && !self.ssh_test_promises.contains_key(&project_id)
                            {
                                self.execute_test_connection(project_id);
                            }
                        }
                    }

                    // Mark as loaded immediately - background checks will update rows
                    self.loaded = true;
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {}", e));
                    self.loaded = true; // Still mark as loaded to show error
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.load_error = Some("WASM platform not supported".to_string());
            self.loaded = true;
        }
    }

    /// Format details for a VM: B: billing, P: project, Z: zone
    fn format_vm_details(
        &self,
        _platform: &CloudPlatformConfig,
        vm: &crate::config::VmInstance,
    ) -> String {
        let mut parts = Vec::new();

        // Billing account
        if let Some(billing) = &vm.gcp_billing_account {
            parts.push(format!("B: {}", billing));
        } else {
            parts.push("B: -".to_string());
        }

        // Project
        parts.push(format!("P: {}", vm.gcp_project_id));

        // Zone
        parts.push(format!("Z: {}", vm.zone));

        parts.join(", ")
    }

    /// Format details for a platform with no VMs
    fn format_platform_details(
        &self,
        platform: &CloudPlatformConfig,
        _vm: Option<&crate::config::VmInstance>,
    ) -> String {
        match platform.platform_type.as_str() {
            "gcp" => {
                // GCP details are now in VMs
                "Platform configured".to_string()
            }
            "firebase" => {
                if let Some(project) = &platform.firebase_project_id {
                    format!("P: {}", project)
                } else {
                    "Firebase platform".to_string()
                }
            }
            "supabase" => {
                if let Some(url) = &platform.supabase_api_url {
                    format!("URL: {}", url)
                } else {
                    "Supabase platform".to_string()
                }
            }
            _ => "Platform configured".to_string(),
        }
    }

    /// Fetch GCP account summary (billing accounts, projects, VMs)
    #[cfg(not(target_arch = "wasm32"))]
    fn fetch_gcp_summary(&mut self, platform: &CloudPlatformConfig) -> Option<String> {
        use crate::api::gcp::GcpRestClient;

        // Check if we have a cached summary (keyed by project_id)
        let project_id = platform.gcp_selected_project_id.as_ref()?;
        if let Some(cached) = self.platform_summaries.get(project_id) {
            return Some(cached.clone());
        }

        // Get access token
        let access_token = platform.gcp_oauth_access_token.as_ref()?;

        // Check token expiry
        let now = chrono::Utc::now().timestamp();
        if let Some(expiry) = platform.gcp_oauth_token_expiry {
            if now >= expiry {
                return Some("OAuth expired".to_string());
            }
        }

        let client = GcpRestClient::new(access_token.clone());
        let mut summary_parts = Vec::new();

        // Fetch billing accounts
        if let Ok(billing_list) = client.list_billing_accounts() {
            let count = billing_list.billing_accounts.len();
            if count > 0 {
                let name = &billing_list.billing_accounts[0].display_name;
                if count == 1 {
                    summary_parts.push(format!("1 billing account({})", name));
                } else {
                    summary_parts.push(format!("{} billing accounts({}...)", count, name));
                }
            }
        }

        // Fetch projects
        if let Ok(project_list) = client.list_projects(None) {
            let count = project_list.projects.len();
            if count > 0 {
                let name = &project_list.projects[0].project_id;
                if count == 1 {
                    summary_parts.push(format!("1 project({})", name));
                } else {
                    summary_parts.push(format!("{} projects({}...)", count, name));
                }
            }
        }

        // Show configured VMs
        let vm_count = platform.vms.len();
        if vm_count > 0 {
            let name = &platform.vms[0].name;
            if vm_count == 1 {
                summary_parts.push(format!("1 vm({})", name));
            } else {
                summary_parts.push(format!("{} vms({}...)", vm_count, name));
            }
        }

        if summary_parts.is_empty() {
            None
        } else {
            let summary = summary_parts.join(", ");
            // Cache the summary (keyed by project_id)
            if let Some(project_id) = &platform.gcp_selected_project_id {
                self.platform_summaries
                    .insert(project_id.clone(), summary.clone());
            }
            Some(summary)
        }
    }

    fn render_add_dialog(
        &mut self,
        ctx: &egui::Context,
        mut vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        let mut open = self.show_add_dialog;

        egui::Window::new("Add Platform")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Configure a new cloud platform:");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    egui::ComboBox::from_id_salt("platform_type_combo")
                        .selected_text(&self.add_platform_type)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.add_platform_type,
                                "gcp".to_string(),
                                "GCP (Google Cloud Platform)",
                            );
                            // TODO: Re-enable when Firebase/Supabase support is implemented
                            // ui.selectable_value(&mut self.add_platform_type, "firebase".to_string(), "Firebase");
                            // ui.selectable_value(&mut self.add_platform_type, "supabase".to_string(), "Supabase");
                        });
                });

                ui.add_space(12.0);

                // Show OAuth connection for GCP
                if self.add_platform_type == "gcp" {
                    ui.separator();
                    ui.add_space(8.0);

                    // Check for OAuth promise result
                    if let Some(promise) = &self.add_platform_oauth_promise {
                        if let Some(result) = promise.ready() {
                            match result {
                                Ok(oauth_result) => {
                                    self.add_platform_oauth_result = Some(oauth_result.clone());
                                    // Fetch account email
                                    self.fetch_connected_email();
                                    self.add_platform_oauth_promise = None;
                                }
                                Err(e) => {
                                    self.load_error = Some(format!("OAuth failed: {}", e));
                                    self.add_platform_oauth_promise = None;
                                }
                            }
                        }
                    }

                    if let Some(email) = &self.add_platform_connected_email {
                        ui.colored_label(
                            egui::Color32::from_rgb(72, 187, 120),
                            format!("✓ Connected as: {}", email),
                        );

                        ui.add_space(8.0);

                        // Fetch projects if not already fetched
                        if self.add_platform_project_list.is_empty() {
                            if let Some(oauth_result) = &self.add_platform_oauth_result {
                                use crate::api::gcp::GcpRestClient;
                                let client = GcpRestClient::new(oauth_result.access_token.clone());
                                match client.list_projects(None) {
                                    Ok(project_list) => {
                                        self.add_platform_project_list = project_list
                                            .projects
                                            .into_iter()
                                            .filter(|p| p.is_active())
                                            .map(|p| {
                                                (p.id().to_string(), p.display_name().to_string())
                                            })
                                            .collect();
                                    }
                                    Err(e) => {
                                        dure_debug!("Failed to fetch projects: {}", e);
                                    }
                                }
                            }
                        }

                        // Show project selection or creation
                        ui.label("Project:");
                        ui.add_space(4.0);

                        // Radio buttons for select vs create
                        ui.horizontal(|ui| {
                            ui.radio_value(&mut self.add_platform_create_new, false, "Select Existing");
                            ui.radio_value(&mut self.add_platform_create_new, true, "Create New");
                        });
                        ui.add_space(8.0);

                        if self.add_platform_create_new {
                            // Create new project UI
                            ui.label("New Project ID:");
                            ui.add_space(4.0);
                            ui.text_edit_singleline(&mut self.add_platform_new_project_id);
                            ui.add_space(4.0);
                            ui.colored_label(
                                egui::Color32::GRAY,
                                "ⓘ Project ID must be 6-30 characters, lowercase letters, digits, hyphens",
                            );
                        } else {
                            // Select existing project UI
                            ui.label(format!(
                                "Select Project ({} available):",
                                self.add_platform_project_list.len()
                            ));
                            ui.add_space(4.0);

                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    for (idx, (project_id, project_name)) in
                                        self.add_platform_project_list.iter().enumerate()
                                    {
                                        let is_selected =
                                            self.add_platform_selected_project == Some(idx);
                                        if ui
                                            .selectable_label(
                                                is_selected,
                                                format!("{} ({})", project_name, project_id),
                                            )
                                            .clicked()
                                        {
                                            self.add_platform_selected_project = Some(idx);
                                        }
                                    }
                                });
                        }
                    } else if self.add_platform_oauth_promise.is_some() {
                        ui.spinner();
                        ui.label("Waiting for authorization...");
                        ui.label("Please complete the OAuth flow in your browser.");
                    } else {
                        if ui
                            .add(MaterialButton::outlined("Connect to Google Cloud"))
                            .clicked()
                        {
                            self.start_add_platform_oauth();
                        }
                        ui.add_space(4.0);
                        ui.colored_label(
                            egui::Color32::GRAY,
                            "⚠ Connection required for GCP platforms",
                        );

                        // Show OAuth URL if available
                        if let Some(ref oauth_url) = self.add_platform_oauth_url {
                            ui.add_space(8.0);
                            ui.label("OAuth URL (copy to browser if needed):");
                            ui.add_space(4.0);
                            egui::ScrollArea::vertical()
                                .max_height(60.0)
                                .show(ui, |ui| {
                                    ui.text_edit_multiline(&mut oauth_url.as_str());
                                });
                        }
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                }

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.show_add_dialog = false;
                        self.add_platform_oauth_url = None;
                        self.add_platform_oauth_result = None;
                        self.add_platform_oauth_promise = None;
                        self.add_platform_connected_email = None;
                        self.add_platform_project_list.clear();
                        self.add_platform_selected_project = None;
                        self.add_platform_create_new = false;
                        self.add_platform_new_project_id.clear();
                    }

                    let can_add = self.add_platform_type != "gcp"
                        || (self.add_platform_connected_email.is_some()
                            && (self.add_platform_selected_project.is_some()
                                || (!self.add_platform_new_project_id.is_empty())));

                    ui.add_enabled_ui(can_add, |ui| {
                        if ui.button("Add").clicked() {
                            self.execute_add_platform(vm.as_deref_mut());
                            self.show_add_dialog = false;
                            self.add_platform_oauth_url = None;
                            self.add_platform_oauth_result = None;
                            self.add_platform_oauth_promise = None;
                            self.add_platform_connected_email = None;
                            self.add_platform_project_list.clear();
                            self.add_platform_selected_project = None;
                            self.add_platform_create_new = false;
                            self.add_platform_new_project_id.clear();
                        }
                    });

                    if !can_add {
                        if self.add_platform_type == "gcp"
                            && self.add_platform_connected_email.is_none()
                        {
                            ui.label("⚠ Connect to Google Cloud first");
                        } else if self.add_platform_type == "gcp" {
                            if self.add_platform_create_new {
                                ui.label("⚠ Enter project ID");
                            } else {
                                ui.label("⚠ Select a project");
                            }
                        }
                    }
                });
            });

        if !open {
            self.show_add_dialog = false;
            self.add_platform_oauth_result = None;
            self.add_platform_oauth_promise = None;
            self.add_platform_connected_email = None;
        }
    }

    fn execute_add_platform(&mut self, vm: Option<&mut crate::viewmodel::ViewModel>) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Extract OAuth info if GCP
            let (oauth_access, oauth_refresh, oauth_expiry, connected_email, selected_project) =
                if self.add_platform_type == "gcp" {
                    if let Some(oauth) = &self.add_platform_oauth_result {
                        let project_id = if self.add_platform_create_new {
                            // Create new project
                            if !self.add_platform_new_project_id.is_empty() {
                                // TODO: Call GCP API to create project
                                // For now, just use the entered project ID
                                // In production, you'd call:
                                // let client = GcpRestClient::new(oauth.access_token.clone());
                                // client.create_project(&self.add_platform_new_project_id, &self.add_platform_new_project_id)?;
                                Some(self.add_platform_new_project_id.clone())
                            } else {
                                None
                            }
                        } else {
                            // Select existing project
                            self.add_platform_selected_project
                                .and_then(|idx| self.add_platform_project_list.get(idx))
                                .map(|(id, _)| id.clone())
                        };
                        (
                            Some(oauth.access_token.clone()),
                            Some(oauth.refresh_token.clone()),
                            Some(oauth.expires_at as i64),
                            self.add_platform_connected_email.clone(),
                            project_id,
                        )
                    } else {
                        (None, None, None, None, None)
                    }
                } else {
                    (None, None, None, None, None)
                };

            if let Some(vm) = vm {
                match vm.add_platform(
                    self.add_platform_type.clone(),
                    oauth_access,
                    oauth_refresh,
                    oauth_expiry,
                    connected_email,
                    selected_project,
                ) {
                    Ok(_) => {
                        dure_info!(" Platform add command sent");
                    }
                    Err(e) => {
                        self.load_error = Some(format!("Failed to add platform: {}", e));
                    }
                }
            }
        }
    }

    fn render_init_progress(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);
        ui.separator();
        ui.heading("Initialization Progress");

        if let Some(name) = &self.init_platform_name {
            ui.label(format!("Platform: {}", name));
        }

        ui.add_space(8.0);

        egui::ScrollArea::vertical()
            .max_height(200.0)
            .show(ui, |ui| {
                for log in &self.init_progress_log {
                    ui.label(log);
                }
            });
    }

    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn show_gcp_wizard(&mut self, platform_name: String) {
        // Try to load config and find platform with OAuth + project
        let mut wizard = if let Ok((app_config, _)) = load_config() {
            // Find platform by name
            if let Some(platform) = app_config.platforms.iter()
                .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
            {
                // Check if platform has OAuth tokens and project ID
                if let (Some(access_token), Some(refresh_token), Some(token_expiry), Some(project_id)) = (
                    &platform.gcp_oauth_access_token,
                    &platform.gcp_oauth_refresh_token,
                    platform.gcp_oauth_token_expiry,
                    &platform.gcp_selected_project_id,
                ) {
                    // Construct OAuthResult and use with_platform_context
                    let oauth_result = crate::api::gcp::oauth::OAuthResult {
                        access_token: access_token.clone(),
                        refresh_token: refresh_token.clone(),
                        expires_at: token_expiry as u64,
                    };

                    GcpWizard::with_platform_context(
                        platform_name,
                        project_id.clone(),
                        oauth_result,
                    )
                } else {
                    // Missing OAuth or project, use full wizard
                    GcpWizard::new(platform_name)
                }
            } else {
                // Platform not found in config, use full wizard
                GcpWizard::new(platform_name)
            }
        } else {
            // Config load failed, use full wizard
            GcpWizard::new(platform_name)
        };

        wizard.show();
        self.gcp_wizard = Some(wizard);
    }

    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn restart_vm(
        &mut self,
        platform_name: String,
        vm_name: String,
        zone: String,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        // ViewModel-based implementation
        if let Some(vm) = vm {
            // Send command to ViewModel
            if let Err(e) = vm.restart_vm(platform_name, vm_name, zone) {
                self.load_error = Some(format!("Failed to start VM restart: {}", e));
            }
            // Note: UI will be updated by event processing when VMRestarted event arrives
        } else {
            // Fallback: no ViewModel available
            self.load_error = Some("ViewModel not available".to_string());
        }
    }

    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn regenerate_vm(
        &mut self,
        platform_name: String,
        vm_name: String,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        if let Some(vm) = vm {
            // Get zone from config
            let zone = match load_config() {
                Ok((app_config, _)) => {
                    if let Some(platform) = app_config
                        .platforms
                        .iter()
                        .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
                    {
                        if let Some(vm_cfg) = platform.vms.iter().find(|v| v.name == vm_name) {
                            vm_cfg.zone.clone()
                        } else {
                            self.load_error = Some(format!("VM not found: {}", vm_name));
                            return;
                        }
                    } else {
                        self.load_error = Some(format!("Platform not found: {}", platform_name));
                        return;
                    }
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {}", e));
                    return;
                }
            };

            if let Err(e) = vm.regenerate_vm(platform_name.clone(), vm_name.clone(), zone) {
                self.load_error = Some(format!("Failed to start VM regeneration: {}", e));
            }
            // Result will be delivered via VMRegenerated event
        } else {
            self.load_error = Some("ViewModel not available".to_string());
        }
    }

    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn update_firewall(
        &mut self,
        platform_name: String,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        use crate::api::gcp::get_current_ip;

        dure_info!(" Firewall update requested for '{}'", platform_name);

        // ViewModel-based implementation
        if let Some(vm) = vm {
            // Get current IP
            let current_ip = match get_current_ip() {
                Ok(ip) => {
                    dure_info!(" Current IP detected: {}", ip);
                    ip
                },
                Err(e) => {
                    dure_error!("Failed to get current IP: {}", e);
                    self.load_error = Some(format!("Failed to get current IP: {}", e));
                    return;
                }
            };

            // Send command to ViewModel
            if let Err(e) = vm.update_firewall(platform_name.clone(), current_ip.clone()) {
                dure_error!("Failed to send firewall update command: {}", e);
                self.load_error = Some(format!("Failed to start firewall update: {}", e));
            } else {
                dure_info!(" Firewall update command sent successfully");
            }
            // Note: UI will be updated by event processing when FirewallUpdated event arrives
        } else {
            // Fallback: no ViewModel available
            dure_error!("ViewModel not available for firewall update");
            self.load_error = Some("ViewModel not available".to_string());
        }
    }

    fn scan_vms(
        &mut self,
        platform_name: String,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        // ViewModel-based implementation
        if let Some(vm) = vm {
            dure_info!(" Scanning VMs for platform '{}'...", platform_name);

            // Send command to ViewModel
            if let Err(e) = vm.scan_existing_vms(platform_name) {
                self.load_error = Some(format!("Failed to start VM scan: {}", e));
            }
            // Note: UI will be updated when VMsScanned event arrives
        } else {
            // Fallback: no ViewModel available
            self.load_error = Some("ViewModel not available".to_string());
        }
    }

    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn show_select_project_dialog(
        &mut self,
        platform_name: String,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        self.select_project_platform_name = platform_name.clone();
        self.select_project_list.clear();
        self.select_project_selected = None;
        self.select_project_loading = true;

        // Trigger ViewModel to fetch projects
        if let Some(vm) = vm {
            if let Err(e) = vm.list_projects(platform_name.clone()) {
                self.load_error = Some(format!("Failed to start project listing: {}", e));
                self.select_project_loading = false;
            }
            // Dialog will be shown when ProjectsListed event arrives
        } else {
            // Fallback: No ViewModel available, show error
            self.load_error = Some("ViewModel not available".to_string());
            self.select_project_loading = false;
        }
    }

    fn show_delete_vm_confirmation(
        &mut self,
        platform_name: String,
        vm_name: String,
        zone: String,
    ) {
        self.delete_vm_platform = platform_name;
        self.delete_vm_list.clear();
        self.delete_vm_list.push((vm_name, zone, "".to_string()));
        self.delete_vm_selected = Some(0);
        self.delete_vm_confirming = true;
        self.show_delete_vm_dialog = true;
    }

    fn render_delete_vm_dialog(
        &mut self,
        ctx: &egui::Context,
        mut vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        let mut open = self.show_delete_vm_dialog;

        egui::Window::new("Delete VM")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                if self.delete_vm_confirming {
                    // Confirmation step
                    ui.heading("⚠ Confirm Deletion");
                    ui.add_space(8.0);

                    if let Some(idx) = self.delete_vm_selected {
                        if let Some((name, zone, _)) = self.delete_vm_list.get(idx).cloned() {
                            ui.label(format!("Are you sure you want to delete VM '{}'?", name));
                            ui.add_space(4.0);
                            ui.colored_label(
                                egui::Color32::from_rgb(245, 101, 101),
                                "This action cannot be undone!",
                            );
                            ui.add_space(4.0);
                            ui.label(format!("Zone: {}", zone));

                            ui.add_space(12.0);

                            let name_clone = name.clone();
                            let zone_clone = zone.clone();

                            ui.horizontal(|ui| {
                                if ui.button("No, Cancel").clicked() {
                                    self.delete_vm_confirming = false;
                                }

                                if ui.add(MaterialButton::filled("Yes, Delete")).clicked() {
                                    self.execute_delete_vm(name_clone, zone_clone, vm);
                                    self.show_delete_vm_dialog = false;
                                }
                            });
                        }
                    }
                } else {
                    // VM selection step
                    ui.heading("Select VM to Delete");
                    ui.add_space(8.0);

                    if self.delete_vm_list.is_empty() {
                        ui.label("No VMs found in this project.");
                        ui.add_space(8.0);
                        ui.colored_label(
                            egui::Color32::GRAY,
                            "Note: Only VMs in common zones are shown.",
                        );
                    } else {
                        ui.label(format!("Found {} VM(s):", self.delete_vm_list.len()));
                        ui.add_space(8.0);

                        egui::ScrollArea::vertical()
                            .max_height(300.0)
                            .show(ui, |ui| {
                                for (idx, (name, zone, status)) in
                                    self.delete_vm_list.iter().enumerate()
                                {
                                    let is_selected = self.delete_vm_selected == Some(idx);
                                    if ui
                                        .selectable_label(
                                            is_selected,
                                            format!("{} ({}, {})", name, zone, status),
                                        )
                                        .clicked()
                                    {
                                        self.delete_vm_selected = Some(idx);
                                    }
                                }
                            });
                    }

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_delete_vm_dialog = false;
                        }

                        let can_delete = self.delete_vm_selected.is_some();
                        ui.add_enabled_ui(can_delete, |ui| {
                            if ui.add(MaterialButton::filled("Delete")).clicked() {
                                self.delete_vm_confirming = true;
                            }
                        });

                        if !can_delete {
                            ui.label("⚠ Select a VM");
                        }
                    });
                }
            });

        if !open {
            self.show_delete_vm_dialog = false;
            self.delete_vm_confirming = false;
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn execute_delete_vm(
        &mut self,
        instance_name: String,
        zone: String,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        // ViewModel-based implementation
        if let Some(vm) = vm {
            // Send command to ViewModel
            if let Err(e) =
                vm.delete_vm(self.delete_vm_platform.clone(), instance_name.clone(), zone)
            {
                self.load_error = Some(format!("Failed to start VM deletion: {}", e));
                return;
            }

            // Record audit event
            // TODO: This should be moved to the actor/calc layer
            if let Ok((app_config, _)) = load_config() {
                if let Some(platform) = app_config
                    .platforms
                    .iter()
                    .find(|p| p.gcp_selected_project_id.as_ref() == Some(&self.delete_vm_platform))
                {
                    if let Some(vm_config) = platform.vms.iter().find(|v| v.name == instance_name) {
                        let project_id = &vm_config.gcp_project_id;
                        match audit::push_gui(
                            "system",
                            "desktop",
                            "vm delete",
                            &format!("{}:{}", project_id, instance_name),
                        ) {
                            Ok(audit_id) => {
                                dure_info!(" Audit record created: ID {}", audit_id);
                            }
                            Err(e) => {
                                dure_warn!(" Failed to record audit event: {}", e);
                            }
                        }
                    }
                }
            }

            // Note: Config will be updated when VMDeleted event arrives
        } else {
            // Fallback: no ViewModel available
            self.load_error = Some("ViewModel not available".to_string());
        }
    }

    /// Get valid access token, refreshing if expired
    #[cfg(not(target_arch = "wasm32"))]
    fn get_valid_access_token(
        &self,
        app_config: &mut AppConfig,
        platform_idx: usize,
        config_path: &std::path::PathBuf,
    ) -> Result<String, String> {
        let platform = &app_config.platforms[platform_idx];

        // Check if token exists
        let access_token = platform
            .gcp_oauth_access_token
            .as_ref()
            .ok_or("No OAuth access token found")?;
        let refresh_token = platform
            .gcp_oauth_refresh_token
            .as_ref()
            .ok_or("No OAuth refresh token found")?;

        // Check if token is expired
        let now = chrono::Utc::now().timestamp();
        let is_expired = platform
            .gcp_oauth_token_expiry
            .map(|expiry| now >= expiry - 60) // Refresh 60 seconds before expiry
            .unwrap_or(true);

        if !is_expired {
            return Ok(access_token.clone());
        }

        // Token expired, refresh it
        dure_debug!("Access token expired, refreshing...");

        use crate::api::gcp::oauth::{self, OAuthHandler};

        // Use embedded OAuth credentials
        let handler = OAuthHandler::default();
        let oauth_result = oauth::refresh_access_token(
            handler.client_id(),
            handler.client_secret(),
            refresh_token,
        )
        .map_err(|e| format!("Failed to refresh token: {}", e))?;

        // Update config with new token
        let platform = &mut app_config.platforms[platform_idx];
        platform.gcp_oauth_access_token = Some(oauth_result.access_token.clone());
        platform.gcp_oauth_token_expiry = Some(oauth_result.expires_at as i64);

        // Save config
        app_config
            .save(config_path)
            .map_err(|e| format!("Failed to save refreshed token: {}", e))?;

        dure_info!(" Access token refreshed");
        Ok(oauth_result.access_token)
    }

    /// Execute SSH connection test for a platform's VM
    #[cfg(not(target_arch = "wasm32"))]
    fn execute_test_connection(&mut self, platform_name: String) {
        // Load config and find the VM for this platform (platform_name is actually project_id)
        let (vm_host, keyring_domain) = match load_config() {
            Ok((app_config, _)) => {
                let platform = app_config
                    .platforms
                    .iter()
                    .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name));

                if let Some(platform) = platform {
                    if let Some(vm) = platform.vms.first() {
                        if let Some(external_ip) = &vm.external_ip {
                            // Construct SSH host from VM info
                            let host = format!("root@{}", external_ip);
                            (Some(host), vm.ssh_key_name.clone())
                        } else {
                            self.ssh_test_results
                                .insert(platform_name, Err("VM has no external IP".to_string()));
                            return;
                        }
                    } else {
                        self.ssh_test_results
                            .insert(platform_name, Err("No VM found".to_string()));
                        return;
                    }
                } else {
                    self.ssh_test_results
                        .insert(platform_name.clone(), Err("Platform not found".to_string()));
                    return;
                }
            }
            Err(e) => {
                self.ssh_test_results
                    .insert(platform_name, Err(format!("Failed to load config: {}", e)));
                return;
            }
        };

        let Some(host) = vm_host else {
            return;
        };

        // Build SSH host config
        let host_config = crate::config::SshHostConfig {
            host: host.clone(),
            password: None,
            private_key_path: None,
            keyring_domain,
            port: 22,
            initialized: false,
            last_status: None,
            platform_name: None,
            docker_containers: Vec::new(),
            ansible_roles: Vec::new(),
            dure_wss_config: None,
        };

        // Spawn connection test in background thread
        let platform_name_clone = platform_name.clone();
        let promise = poll_promise::Promise::spawn_thread("ssh_test_platform", move || {
            use crate::calc::ssh;
            // russh uses tokio internally, wrap with async-compat for smol
            smol::block_on(async {
                async_compat::Compat::new(ssh::test_connection(&host_config))
                    .await
                    .map_err(|e| format!("{}", e))
            })
        });

        self.ssh_test_promises.insert(platform_name, promise);
    }

    /// Get valid access token, refreshing if expired (standalone helper)
    fn refresh_or_get_token(platform: &mut crate::config::CloudPlatformConfig) -> Result<String, String> {
        use crate::api::gcp::oauth::refresh_access_token;

        let token = platform.gcp_oauth_access_token.as_ref()
            .ok_or_else(|| "No OAuth access token".to_string())?;

        let refresh_token = platform.gcp_oauth_refresh_token.as_ref()
            .ok_or_else(|| "No OAuth refresh token".to_string())?;

        let expiry = platform.gcp_oauth_token_expiry
            .ok_or_else(|| "No token expiry".to_string())?;

        // Check if expired (with 60 second buffer)
        let now = chrono::Utc::now().timestamp();
        if now >= expiry - 60 {
            // Refresh token
            let oauth_handler = crate::api::gcp::oauth::OAuthHandler::default();
            let new_oauth = refresh_access_token(
                oauth_handler.client_id(),
                oauth_handler.client_secret(),
                refresh_token,
            ).map_err(|e| format!("Failed to refresh token: {}", e))?;

            // Update platform
            platform.gcp_oauth_access_token = Some(new_oauth.access_token.clone());
            platform.gcp_oauth_token_expiry = Some(new_oauth.expires_at as i64);

            Ok(new_oauth.access_token)
        } else {
            Ok(token.clone())
        }
    }

    /// Execute status refresh for a platform
    #[cfg(not(target_arch = "wasm32"))]
    fn execute_refresh(&mut self, project_id: String) {
        use crate::api::gcp::{GcpRestClient, get_current_ip};

        let project_id_clone = project_id.clone();
        let promise = poll_promise::Promise::spawn_thread("refresh_status", move || {
            // Load config
            let (mut config, config_path) = load_config()
                .map_err(|e| format!("Failed to load config: {}", e))?;

            // Find platform
            let platform = config.platforms.iter_mut()
                .find(|p| p.gcp_selected_project_id.as_ref() == Some(&project_id_clone))
                .ok_or_else(|| format!("Platform {} not found", project_id_clone))?;

            // Get valid access token
            let token = Self::refresh_or_get_token(platform)?;
            let client = GcpRestClient::new(token);

            // Fetch VM status if VM exists
            if let Some(vm) = platform.vms.first() {
                match client.get_instance(&project_id_clone, &vm.zone, &vm.name) {
                    Ok(instance) => {
                        platform.cached_vm_status = Some(instance.status.clone());

                        // Extract external IP from network interfaces
                        if let Some(ni) = instance.network_interfaces.first() {
                            if let Some(ac) = ni.access_configs.first() {
                                platform.cached_vm_external_ip = ac.nat_ip.clone();
                            }
                        }
                    }
                    Err(e) => {
                        dure_debug!("Failed to fetch VM status: {}", e);
                    }
                }
            }

            // Fetch firewall status
            match get_current_ip() {
                Ok(current_ip) => {
                    match client.check_ip_whitelisted(&project_id_clone, &current_ip) {
                        Ok(whitelisted) => {
                            platform.cached_firewall_status = Some(
                                if whitelisted {
                                    format!("✓ Whitelisted ({})", current_ip)
                                } else {
                                    "✗ Not whitelisted".to_string()
                                }
                            );
                        }
                        Err(e) => {
                            dure_debug!("Failed to check firewall: {}", e);
                        }
                    }
                }
                Err(e) => {
                    dure_debug!("Failed to get current IP: {}", e);
                }
            }

            // Fetch total project count
            match client.list_projects(None) {
                Ok(list) => {
                    platform.cached_total_project_count = Some(list.projects.len());
                }
                Err(e) => {
                    dure_debug!("Failed to fetch project count: {}", e);
                }
            }

            // Update refresh timestamp
            platform.last_status_refresh = Some(chrono::Utc::now().timestamp());

            // Save config
            config.save(&config_path)
                .map_err(|e| format!("Failed to save config: {}", e))?;

            Ok(())
        });

        self.refresh_promises.insert(project_id, promise);
    }

    #[cfg(target_arch = "wasm32")]
    fn execute_refresh(&mut self, _project_id: String) {
        // WASM not supported
    }

    fn show_delete_platform_confirmation(&mut self, platform_name: String) {
        self.delete_platform_name = platform_name.clone();

        // Count VMs for this platform
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok((app_config, _)) = load_config() {
                if let Some(platform) = app_config
                    .platforms
                    .iter()
                    .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
                {
                    self.delete_platform_vm_count = platform.vms.len();
                }
            }
        }

        self.show_delete_platform_dialog = true;
    }

    fn render_delete_platform_dialog(
        &mut self,
        ctx: &egui::Context,
        mut vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        let mut open = self.show_delete_platform_dialog;

        egui::Window::new("Delete Platform")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.heading("⚠ Confirm Platform Deletion");
                ui.add_space(8.0);

                ui.label(format!(
                    "Are you sure you want to delete platform '{}'?",
                    self.delete_platform_name
                ));
                ui.add_space(4.0);

                if self.delete_platform_vm_count > 0 {
                    ui.colored_label(
                        egui::Color32::from_rgb(245, 101, 101),
                        format!(
                            "⚠ This will also remove {} VM(s) from config!",
                            self.delete_platform_vm_count
                        ),
                    );
                    ui.add_space(4.0);
                    ui.colored_label(
                        egui::Color32::from_rgb(255, 152, 0),
                        "Note: VMs will be removed from config but NOT deleted from GCP.",
                    );
                } else {
                    ui.colored_label(egui::Color32::GRAY, "This platform has no VMs configured.");
                }

                ui.add_space(4.0);
                ui.colored_label(
                    egui::Color32::from_rgb(245, 101, 101),
                    "This action cannot be undone!",
                );

                ui.add_space(12.0);

                // Delete options
                ui.checkbox(&mut self.delete_platform_delete_vms, "Delete VMs from GCP");
                ui.checkbox(
                    &mut self.delete_platform_delete_project,
                    "Delete GCP project",
                );

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui.button("No, Cancel").clicked() {
                        self.show_delete_platform_dialog = false;
                        self.delete_platform_delete_vms = false;
                        self.delete_platform_delete_project = false;
                    }

                    if ui
                        .add(MaterialButton::filled("Yes, Delete Platform"))
                        .clicked()
                    {
                        self.execute_delete_platform(vm.as_deref_mut());
                        self.show_delete_platform_dialog = false;
                        self.delete_platform_delete_vms = false;
                        self.delete_platform_delete_project = false;
                    }
                });
            });

        if !open {
            self.show_delete_platform_dialog = false;
            self.delete_platform_delete_vms = false;
            self.delete_platform_delete_project = false;
        }
    }

    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn render_select_project_dialog(
        &mut self,
        ctx: &egui::Context,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        let mut open = self.show_select_project_dialog;

        egui::Window::new("Select GCP Project")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .default_width(500.0)
            .show(ctx, |ui| {
                ui.heading("Select Project");
                ui.add_space(8.0);

                if self.select_project_list.is_empty() {
                    ui.colored_label(egui::Color32::from_rgb(255, 152, 0), "No projects found");
                } else {
                    ui.label(format!(
                        "Found {} projects:",
                        self.select_project_list.len()
                    ));
                    ui.add_space(8.0);

                    egui::ScrollArea::vertical()
                        .max_height(300.0)
                        .show(ui, |ui| {
                            for (idx, (project_id, project_name)) in
                                self.select_project_list.iter().enumerate()
                            {
                                let is_selected = self.select_project_selected == Some(idx);
                                if ui
                                    .selectable_label(
                                        is_selected,
                                        format!("{} ({})", project_name, project_id),
                                    )
                                    .clicked()
                                {
                                    self.select_project_selected = Some(idx);
                                }
                            }
                        });
                }

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.show_select_project_dialog = false;
                    }

                    let select_enabled = self.select_project_selected.is_some();
                    let select_button = MaterialButton::filled("Select");
                    let select_button = if select_enabled {
                        select_button
                    } else {
                        select_button.enabled(false)
                    };

                    if ui.add(select_button).clicked() {
                        self.execute_select_project(vm);
                        self.show_select_project_dialog = false;
                    }
                });
            });

        if !open {
            self.show_select_project_dialog = false;
        }
    }

    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn execute_select_project(&mut self, vm: Option<&mut crate::viewmodel::ViewModel>) {
        if let Some(selected_idx) = self.select_project_selected {
            if selected_idx < self.select_project_list.len() {
                let (project_id, _) = &self.select_project_list[selected_idx];
                let platform_name = self.select_project_platform_name.clone();

                if let Some(vm) = vm {
                    if let Err(e) = vm.select_project(platform_name.clone(), project_id.clone()) {
                        self.load_error = Some(format!("Failed to select project: {}", e));
                    }
                    // Result will be delivered via ProjectSelected event
                } else {
                    self.load_error = Some("ViewModel not available".to_string());
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn execute_delete_platform(&mut self, vm: Option<&mut crate::viewmodel::ViewModel>) {
        if let Some(vm) = vm {
            let delete_options = crate::viewmodel::platform::DeleteOptions {
                delete_vms: self.delete_platform_delete_vms,
                delete_project: self.delete_platform_delete_project,
            };
            match vm.delete_platform(self.delete_platform_name.clone(), delete_options) {
                Ok(_) => {
                    dure_info!(" Platform delete command sent");
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to delete platform: {}", e));
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start_add_platform_oauth(&mut self) {
        use crate::api::gcp::oauth::OAuthHandler;
        use poll_promise::Promise;

        // Use embedded OAuth credentials (compiled into binary)
        let handler = OAuthHandler::default();

        self.add_platform_oauth_promise =
            Some(Promise::spawn_thread("gcp_oauth_add_platform", move || {
                handler.run_oauth_flow().map_err(|e| e.to_string())
            }));
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn fetch_connected_email(&mut self) {
        if let Some(oauth) = &self.add_platform_oauth_result {
            use crate::api::gcp::GcpRestClient;

            let client = GcpRestClient::new(oauth.access_token.clone());

            // Get user info from OAuth2 userinfo endpoint
            match client.get_user_info() {
                Ok(user_info) => {
                    // Use email if available, fallback to name, or "Connected Account"
                    let display = if let Some(email) = user_info.email {
                        email
                    } else if let Some(name) = user_info.name {
                        name
                    } else {
                        "Connected Account".to_string()
                    };
                    self.add_platform_connected_email = Some(display);
                }
                Err(e) => {
                    dure_debug!("Failed to fetch user info: {}", e);
                    self.add_platform_connected_email = Some("Connected Account".to_string());
                }
            }
        }
    }

    fn fetch_billing_data(&mut self, vm: Option<&mut crate::viewmodel::ViewModel>, project_id_param: Option<String>) {
        // ViewModel-based implementation
        if let Some(vm) = vm {
            self.billing_loading = true;
            self.billing_error = None;
            self.billing_data = None;

            // Load config to get GCP platform with OAuth
            let (mut app_config, config_path) = match load_config() {
                Ok(config) => config,
                Err(e) => {
                    self.billing_error = Some(format!("Failed to load config: {}", e));
                    self.billing_loading = false;
                    return;
                }
            };

            // Find first GCP platform with OAuth token (need index for token refresh)
            let platform_idx = match app_config
                .platforms
                .iter()
                .enumerate()
                .find(|(_, p)| p.platform_type == "gcp" && p.gcp_oauth_access_token.is_some())
                .map(|(idx, _)| idx)
            {
                Some(idx) => idx,
                None => {
                    self.billing_error = Some(
                        "No connected GCP platform found. Please add a GCP platform first."
                            .to_string(),
                    );
                    self.billing_loading = false;
                    return;
                }
            };

            // Get valid (possibly refreshed) access token
            let access_token =
                match self.get_valid_access_token(&mut app_config, platform_idx, &config_path) {
                    Ok(token) => token,
                    Err(e) => {
                        self.billing_error =
                            Some(format!("Failed to get valid OAuth token: {}", e));
                        self.billing_loading = false;
                        return;
                    }
                };

            // Get platform reference after token refresh
            let platform = &app_config.platforms[platform_idx];

            // Get project ID: use provided parameter, or try platform config, or fall back to VMs
            let project_id = if let Some(pid) = project_id_param {
                pid
            } else if let Some(pid) = &platform.gcp_selected_project_id {
                pid.clone()
            } else if !platform.vms.is_empty() {
                platform.vms[0].gcp_project_id.clone()
            } else {
                self.billing_error = Some(
                    "No project ID available. Please select a project first.".to_string(),
                );
                self.billing_loading = false;
                return;
            };

            // Auto-discover billing table if not configured
            if self.billing_dataset.is_empty() || self.billing_table.is_empty() {
                let client = crate::api::gcp::GcpRestClient::new(access_token.clone());

                match client.discover_billing_table(&project_id) {
                    Ok((dataset, table)) => {
                        self.billing_dataset = dataset;
                        self.billing_table = table;
                        self.billing_project_id = project_id.clone();
                    }
                    Err(e) => {
                        // Fall back to default names
                        self.billing_dataset = "billing_export".to_string();
                        self.billing_table =
                            format!("gcp_billing_export_v1_{}", project_id.replace('-', "_"));
                        self.billing_project_id = project_id.clone();
                        self.billing_error = Some(format!(
                            "Auto-discovery failed: {}\n\nUsing default names. Please configure below if different.",
                            e
                        ));
                        self.billing_loading = false;
                        return;
                    }
                }
            }

            if self.billing_project_id.is_empty() {
                self.billing_project_id = project_id.clone();
            }

            // Send command to ViewModel
            if let Err(e) = vm.fetch_billing(
                project_id.clone(),  // Use project_id as platform identifier
                project_id,
                self.billing_dataset.clone(),
                self.billing_table.clone(),
            ) {
                self.billing_error = Some(format!("Failed to start billing fetch: {}", e));
                self.billing_loading = false;
            }
            // Note: billing_data will be set by event processing when BillingFetched event arrives
        } else {
            // Fallback: no ViewModel available (shouldn't happen in normal operation)
            self.billing_error = Some("ViewModel not available".to_string());
            self.billing_loading = false;
        }
    }

    fn render_billing_dialog(
        &mut self,
        ctx: &egui::Context,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        egui::Window::new("Monthly Billing")
            .collapsible(false)
            .resizable(true)
            .default_width(600.0)
            .show(ctx, |ui| {
                ui.heading("Monthly Total Cost (Last 3 Months)");
                ui.add_space(8.0);

                // Configuration section
                ui.horizontal(|ui| {
                    ui.label("Project ID:");
                    ui.text_edit_singleline(&mut self.billing_project_id);
                });
                ui.horizontal(|ui| {
                    ui.label("Dataset:");
                    ui.text_edit_singleline(&mut self.billing_dataset);
                });
                ui.horizontal(|ui| {
                    ui.label("Table:");
                    ui.text_edit_singleline(&mut self.billing_table);
                });
                ui.add_space(4.0);
                ui.colored_label(
                    egui::Color32::GRAY,
                    "💡 Leave empty to auto-discover billing export table",
                );
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                if self.billing_loading {
                    ui.spinner();
                    ui.label("Loading billing data...");
                } else if let Some(error) = &self.billing_error {
                    // Check if this is a "table not found" error (billing export not configured)
                    let is_table_not_found =
                        error.contains("was not found") || error.contains("Not found");

                    if is_table_not_found {
                        ui.colored_label(
                            egui::Color32::from_rgb(255, 152, 0),
                            "⚠ Billing Export Not Configured",
                        );
                        ui.add_space(4.0);
                        ui.label("BigQuery billing export table does not exist yet.");
                    } else {
                        ui.colored_label(egui::Color32::from_rgb(255, 82, 82), "Error:");
                        ui.add_space(4.0);
                        ui.label(error);
                    }
                    ui.add_space(16.0);

                    ui.label("To enable BigQuery billing export:");
                    ui.label("1. Go to GCP Console → Billing → Billing export");
                    ui.label("2. Select 'Detailed cost data' tab");
                    ui.label("3. Select or create a BigQuery dataset");
                    ui.label("4. Wait a few hours for data to populate");
                } else if let Some(records) = &self.billing_data {
                    if records.is_empty() {
                        ui.label("No billing data found for the current month.");
                        ui.add_space(8.0);
                        ui.label("This could mean:");
                        ui.label("• Billing export is not configured");
                        ui.label("• No costs have been incurred yet this month");
                        ui.label("• Data is still being processed (can take up to 5 days)");
                    } else {
                        // Display monthly totals
                        ui.label(format!(
                            "Monthly Total Net Cost (including all credits) - {} record(s):",
                            records.len()
                        ));
                        ui.add_space(8.0);

                        egui::ScrollArea::vertical()
                            .max_height(400.0)
                            .show(ui, |ui| {
                                for record in records {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(format!(
                                                "📅 {} ({})",
                                                record.month, record.currency
                                            ));

                                            // Helper to format currency with thousand separators
                                            let format_currency =
                                                |amount: f64, currency: &str| -> String {
                                                    // Determine if currency uses decimals
                                                    let uses_decimals = match currency {
                                                        "KRW" | "JPY" | "VND" | "IDR" => false,
                                                        _ => true,
                                                    };

                                                    // Get currency symbol
                                                    let symbol = match currency {
                                                        "USD" => "$",
                                                        "EUR" => "€",
                                                        "GBP" => "£",
                                                        "JPY" => "¥",
                                                        "KRW" => "₩",
                                                        "CNY" => "¥",
                                                        "INR" => "₹",
                                                        "AUD" => "A$",
                                                        "CAD" => "C$",
                                                        "SGD" => "S$",
                                                        "HKD" => "HK$",
                                                        "TWD" => "NT$",
                                                        "THB" => "฿",
                                                        "VND" => "₫",
                                                        "IDR" => "Rp",
                                                        "BRL" => "R$",
                                                        _ => currency, // Fallback to currency code
                                                    };

                                                    if uses_decimals {
                                                        format!("{}{:.2}", symbol, amount)
                                                    } else {
                                                        let int_amount = amount as i64;
                                                        let abs_amount = int_amount.abs();
                                                        let formatted = abs_amount
                                                            .to_string()
                                                            .as_bytes()
                                                            .rchunks(3)
                                                            .rev()
                                                            .map(std::str::from_utf8)
                                                            .collect::<Result<Vec<&str>, _>>()
                                                            .unwrap()
                                                            .join(",");
                                                        if int_amount < 0 {
                                                            format!("-{}{}", symbol, formatted)
                                                        } else {
                                                            format!("{}{}", symbol, formatted)
                                                        }
                                                    }
                                                };

                                            let (color, text) = if record.total_net_cost < 0.0 {
                                                (
                                                    egui::Color32::from_rgb(72, 187, 120),
                                                    format!(
                                                        "{} (credit)",
                                                        format_currency(
                                                            record.total_net_cost,
                                                            &record.currency
                                                        )
                                                    ),
                                                )
                                            } else if record.total_net_cost == 0.0 {
                                                (
                                                    egui::Color32::GRAY,
                                                    format!(
                                                        "{} (no charges)",
                                                        format_currency(0.0, &record.currency)
                                                    ),
                                                )
                                            } else {
                                                (
                                                    egui::Color32::from_rgb(255, 200, 87),
                                                    format_currency(
                                                        record.total_net_cost,
                                                        &record.currency,
                                                    ),
                                                )
                                            };

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.colored_label(color, &text);
                                                },
                                            );
                                        });
                                    });
                                    ui.add_space(4.0);
                                }
                            });
                    }
                }

                ui.add_space(16.0);
                ui.separator();
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    if ui.add(MaterialButton::outlined("Refresh")).clicked() {
                        self.fetch_billing_data(vm, None);
                    }

                    if ui.add(MaterialButton::outlined("Close")).clicked() {
                        self.show_billing_dialog = false;
                    }
                });
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_options_default() {
        let state = PlatformTab::default();
        assert!(!state.delete_platform_delete_vms);
        assert!(!state.delete_platform_delete_project);
    }
}
