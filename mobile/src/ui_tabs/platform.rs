//! Platform tab - Platform configuration and management

use crate::{dure_debug, dure_error, dure_info, dure_warn};
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{
    badge, data_table, dialog, linear_progress, notification, BadgeColor, BadgeSize, DataTableCell,
    MaterialButton, NotificationAlign,
};
use egui_twemoji::EmojiLabel;
use std::cell::Cell;

use crate::api::gcp::bigquery::BillingRecord;
use crate::calc::audit;
use crate::config::{AppConfig, CloudPlatformConfig};
use crate::ui_dlg::platform::{
    AddFirewallDialog, AddPlatformDialog, BillingDialog, BindVmDialog, DeleteFirewallConfirmDialog,
    DeletePlatformConfirmDialog, DeleteVmDialog, FirewallDialog, PlatformErrorDialog,
    RestartVmConfirmDialog, SelectProjectDialog,
};

#[cfg(not(target_arch = "wasm32"))]
use crate::ui_dlg::platform_gcp::GcpWizard;

use crate::ui_components::{ActionMenu, EmojiProgressBar, SvgEmoji};

// Button spacing matching Platform tab
const BUTTON_VERTICAL_SPACING: f32 = 4.0;
const BUTTON_HORIZONTAL_SPACING: f32 = 4.0;

/// Firewall rule information for firewall dialog (comprehensive GCP schema)
#[derive(Clone, Debug)]
pub struct FirewallRule {
    pub name: String,
    pub network: String,
    pub direction: String, // "INGRESS" or "EGRESS"
    pub priority: u32,
    pub source_ranges: Vec<String>,
    pub dest_ranges: Vec<String>,
    pub allowed: Vec<String>, // e.g., ["tcp:22", "udp:53", "icmp"]
    pub denied: Vec<String>,
    pub source_tags: Vec<String>,
    pub source_service_accounts: Vec<String>,
    pub target_tags: Vec<String>,
    pub target_service_accounts: Vec<String>,
}

impl Default for FirewallRule {
    fn default() -> Self {
        Self {
            name: String::new(),
            network: "default".to_string(),
            direction: "INGRESS".to_string(),
            priority: 1000,
            source_ranges: Vec::new(),
            dest_ranges: Vec::new(),
            allowed: Vec::new(),
            denied: Vec::new(),
            source_tags: Vec::new(),
            source_service_accounts: Vec::new(),
            target_tags: Vec::new(),
            target_service_accounts: Vec::new(),
        }
    }
}

/// Simplified firewall rule info (for backward compatibility during migration)
#[derive(Clone, Debug)]
struct FirewallRuleInfo {
    name: String,
    source_ranges: Vec<String>,
    ip_included: bool,
}

/// Firewall validation results
#[derive(Clone, Debug, Default)]
struct FirewallValidation {
    current_ip_missing: bool,
    full_access_count: usize,
    full_access_ips: Vec<String>,
    ssh_access_ips: Vec<String>,
    http_exposed: bool,
    https_exposed: bool,
}

impl FirewallValidation {
    fn has_warnings(&self) -> bool {
        self.current_ip_missing
            || self.full_access_count > 1
            || !self.ssh_access_ips.is_empty()
            || !self.http_exposed
            || !self.https_exposed
    }
}

/// Validate firewall rules against security best practices
fn validate_firewall_rules(rules: &[FirewallRule], current_ip: &str) -> FirewallValidation {
    let mut validation = FirewallValidation::default();

    // Check if current IP is in any rule
    let current_ip_with_cidr = format!("{}/32", current_ip);
    let mut current_ip_found = false;

    for rule in rules {
        if rule.direction != "INGRESS" {
            continue; // Only validate ingress rules
        }

        // Check if current IP is whitelisted
        for range in &rule.source_ranges {
            if range == current_ip || range == &current_ip_with_cidr {
                current_ip_found = true;
            }

            // Check for full access (0.0.0.0/0)
            if range == "0.0.0.0/0" {
                validation.full_access_count += 1;
                validation.full_access_ips.push(range.clone());
            }
        }

        // Check for SSH access (port 22)
        for allowed in &rule.allowed {
            if allowed.contains("tcp:22") || allowed == "tcp:0-65535" || allowed == "tcp" {
                for ip in &rule.source_ranges {
                    if ip != "0.0.0.0/0" && !validation.ssh_access_ips.contains(ip) {
                        validation.ssh_access_ips.push(ip.clone());
                    }
                }
            }

            // Check for HTTP/HTTPS ports
            if allowed.contains("tcp:80") || allowed.contains("tcp:0-65535") {
                validation.http_exposed = true;
            }
            if allowed.contains("tcp:443") || allowed.contains("tcp:0-65535") {
                validation.https_exposed = true;
            }
        }
    }

    validation.current_ip_missing = !current_ip_found;
    validation
}

/// Platform row data for data table
#[derive(Clone, Debug)]
pub struct PlatformRow {
    // Identity (CHANGED: project_id replaces platform_name)
    pub project_id: String,           // GCP project ID (platform identifier)
    pub project_display_name: String, // Display name (may differ from ID)
    pub platform_type: String,        // "GCP"

    // Connection state flags (for Steps column)
    pub gcp_connected: bool,    // Has OAuth access token
    pub project_selected: bool, // Has gcp_selected_project_id
    pub vm_created: bool,       // vms.len() > 0
    pub firewall_updated: bool, // Current IP is whitelisted
    pub ssh_ready: bool,        // VM has external_ip.is_some()

    // Drawer content data
    pub email: Option<String>,              // Connected Google account
    pub total_project_count: Option<usize>, // Cached project count (None = not yet loaded)
    pub selected_project_id: Option<String>,
    pub vm_name: Option<String>,            // First VM name
    pub vm_external_ip: Option<String>,     // First VM external IP (from cache or VM)
    pub ssh_private_key: Option<String>,    // SSH private key from KeePass
    pub ssh_public_key: Option<String>,     // Derived SSH public key for verification
    pub ssh_keyring_domain: Option<String>, // Keyring domain for SSH key
    pub firewall_status: String,            // Cached from config
    pub ssh_status: String,                 // "✅ Ready" or "? No external IP"

    // NEW: Status cache metadata
    pub last_refresh_time: Option<i64>, // For staleness indicator

    // Operation state tracking (for visual feedback)
    pub operation_state: OperationState,

    // Action button state
    pub has_vm: bool,               // Enable/disable VM operation buttons
    pub vm_zone: Option<String>,    // For VM operations (delete, restart, regen)
    pub oauth_refresh_failed: bool, // OAuth token refresh failed

    // DNS wizard domain binding
    pub bound_domain: Option<String>,
}

/// Actions that can be triggered from platform table rows
#[derive(Debug, Clone)]
enum PlatformAction {
    UpdateFirewall(String), // project_id
    SelectProject(String),  // project_id
    DeleteVM {
        project_id: String, // CHANGED from platform_name
        vm_name: String,
        vm_zone: String,
    },
    RegenVM(String),        // project_id
    RestartVM(String),      // project_id
    DeletePlatform(String), // project_id
    Refresh(String),        // NEW: project_id for manual refresh
    Reauth(String),         // NEW: project_id for re-authentication
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

    /// Error dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    error_dialog: PlatformErrorDialog,

    /// Config file last modified time (to detect changes)
    #[cfg_attr(feature = "serde", serde(skip))]
    config_last_modified: Option<std::time::SystemTime>,

    /// Add Platform dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_dialog: AddPlatformDialog,

    // Add platform wizard state
    #[cfg_attr(feature = "serde", serde(skip))]
    wizard_dialog_open: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    wizard_current_step: usize,

    // Init progress state
    #[cfg_attr(feature = "serde", serde(skip))]
    init_in_progress: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    init_platform_name: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    init_progress_log: Vec<String>,

    // GCP wizard
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg_attr(feature = "serde", serde(skip))]
    gcp_wizard: Option<GcpWizard>,

    // Track if wizard was open in previous frame (to detect closure)
    #[cfg(not(target_arch = "wasm32"))]
    #[cfg_attr(feature = "serde", serde(skip))]
    wizard_was_open: bool,

    // Platform summary cache (platform_name -> summary)
    #[cfg_attr(feature = "serde", serde(skip))]
    platform_summaries: std::collections::HashMap<String, String>,

    /// Delete VM dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_vm_dialog: DeleteVmDialog,

    /// Delete Platform confirm dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_platform_confirm_dialog: DeletePlatformConfirmDialog,

    // Billing dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    billing_dialog: BillingDialog,

    /// Select Project dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    select_project_dialog: SelectProjectDialog,

    /// Bind VM dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    bind_vm_dialog: BindVmDialog,

    // Firewall dialogs
    #[cfg_attr(feature = "serde", serde(skip))]
    firewall_dialog: FirewallDialog,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_firewall_dialog: AddFirewallDialog,

    /// Delete firewall confirm dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_firewall_confirm_dialog: DeleteFirewallConfirmDialog,

    /// Restart VM confirm dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    restart_vm_confirm_dialog: RestartVmConfirmDialog,

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

    // Track platforms that have been auto-refreshed (to prevent infinite loop)
    #[cfg_attr(feature = "serde", serde(skip))]
    auto_refreshed_platforms: std::collections::HashSet<String>,

    // Drawer state (for platform details drawer with tabs)
    #[cfg_attr(feature = "serde", serde(skip))]
    drawer_state: crate::viewmodel::platform::DrawerState,

    // SSH creation notification state
    #[cfg_attr(feature = "serde", serde(skip))]
    show_ssh_creation_notification: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    ssh_created_count: usize,
    #[cfg_attr(feature = "serde", serde(skip))]
    ssh_creation_notification_time: Option<std::time::Instant>,
}

impl Default for PlatformTab {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            loaded: false,
            error_dialog: PlatformErrorDialog::new(),
            config_last_modified: None,
            add_platform_dialog: AddPlatformDialog::new(),
            wizard_dialog_open: false,
            wizard_current_step: 0,
            init_in_progress: false,
            init_platform_name: None,
            init_progress_log: Vec::new(),
            #[cfg(not(target_arch = "wasm32"))]
            gcp_wizard: None,
            #[cfg(not(target_arch = "wasm32"))]
            wizard_was_open: false,
            platform_summaries: std::collections::HashMap::new(),
            delete_vm_dialog: DeleteVmDialog::new(),
            delete_platform_confirm_dialog: DeletePlatformConfirmDialog::new(),
            billing_dialog: BillingDialog::new(),
            select_project_dialog: SelectProjectDialog::new(),
            bind_vm_dialog: BindVmDialog::new(),
            firewall_dialog: FirewallDialog::new(),
            add_firewall_dialog: AddFirewallDialog::new(),
            delete_firewall_confirm_dialog: DeleteFirewallConfirmDialog::new(),
            restart_vm_confirm_dialog: RestartVmConfirmDialog::new(),
            ssh_test_promises: std::collections::HashMap::new(),
            ssh_test_results: std::collections::HashMap::new(),
            refresh_promises: std::collections::HashMap::new(),
            auto_refreshed_platforms: std::collections::HashSet::new(),
            drawer_state: crate::viewmodel::platform::DrawerState::new(),
            show_ssh_creation_notification: false,
            ssh_created_count: 0,
            ssh_creation_notification_time: None,
        }
    }
}

/// Get config file path
#[cfg(not(target_arch = "wasm32"))]
fn get_config_path(
    profile: &Option<crate::calc::profile::ProfileContext>,
) -> Result<std::path::PathBuf, String> {
    match profile {
        Some(ctx) => Ok(ctx.config_file.clone()),
        None => Err(tr!("platform-error-no-profile")),
    }
}

/// Load application config with V1 to V2 migration
#[cfg(not(target_arch = "wasm32"))]
fn load_config(
    profile: &Option<crate::calc::profile::ProfileContext>,
) -> Result<(AppConfig, std::path::PathBuf), String> {
    use crate::config_migration::{backup_config, migrate_config_v1_to_v2, AppConfigV1};

    let config_path = get_config_path(profile)?;

    if !config_path.exists() {
        // No config exists, create default
        let default_config = AppConfig::default();
        return Ok((default_config, config_path));
    }

    let contents = std::fs::read_to_string(&config_path)
        .map_err(|e| tr!("platform-error-read-config", {error: e.to_string()}))?;

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
                    v2_config
                        .save(&config_path)
                        .map_err(|e| tr!("platform-error-save-config", {error: e.to_string()}))?;

                    dure_info!(
                        " Migrated {} platform(s) to V2 format",
                        v2_config.platforms.len()
                    );

                    Ok((v2_config, config_path))
                }
                Err(v1_err) => {
                    // Both V1 and V2 failed - config is corrupted
                    Err(
                        tr!("platform-error-parse-config", {v2_error: format!("{:?}", _v2_err), v1_error: format!("{:?}", v1_err)}),
                    )
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
            use crate::api::gcp::{get_current_ip, GcpRestClient};

            let client = GcpRestClient::new(token.to_string());

            // Check ONLY direct IP access (ignore IAP - this app uses direct SSH)
            match get_current_ip() {
                Ok(current_ip) => match client.check_ip_whitelisted(project, &current_ip) {
                    Ok(true) => format!("✅ Whitelisted ({})", current_ip),
                    Ok(false) => format!("❌ Not whitelisted ({})", current_ip),
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
                    "✅ Connected".to_string()
                } else {
                    format!("❌ {}", conn_result.message)
                }
            }
            Err(e) => format!("❌ Failed: {}", e),
        };
    }

    // Otherwise, show basic readiness check
    if let Some(vm) = platform.vms.first() {
        if vm.external_ip.is_some() {
            "? Not tested".to_string()
        } else {
            "❌ No external IP".to_string()
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
pub fn load_ssh_key_from_keyring(
    project_id: &str,
    keyring_domain: &Option<String>,
    kdbx_handle: &Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
) -> (Option<String>, Option<String>) {
    use crate::calc::keyring;

    let domain = match keyring_domain.as_ref() {
        Some(d) => d,
        None => return (None, None),
    };

    // Require profile keyring handle
    let handle = match kdbx_handle {
        Some(h) => h,
        None => return (None, None),
    };

    let keys = match keyring::list_keys_from_handle(handle) {
        Ok(k) => k,
        Err(e) => {
            dure_debug!(
                project_id = project_id,
                "Failed to list keys from profile keyring: {}",
                e
            );
            return (None, None);
        }
    };

    // Find the key with matching domain
    let key_entry = match keys.iter().find(|k| &k.domain == domain) {
        Some(e) => e,
        None => return (None, None),
    };

    // Try to get SSH key from binary attachment
    if let Some(ssh_key_bytes) = &key_entry.ssh_key {
        // Derive public key from raw bytes
        let public_key = derive_public_key_from_raw(ssh_key_bytes);

        // Try to interpret as UTF-8 string first (already in OpenSSH format)
        if let Ok(key_str) = String::from_utf8(ssh_key_bytes.clone()) {
            if key_str.contains("BEGIN") && key_str.contains("PRIVATE KEY") {
                return (Some(key_str), public_key);
            }
        }

        // Otherwise, try to convert raw Ed25519 bytes to OpenSSH format
        let private_key = convert_ed25519_to_openssh(ssh_key_bytes);
        (private_key, public_key)
    } else {
        (None, None)
    }
}

#[cfg(target_arch = "wasm32")]
fn load_ssh_key_from_keyring(
    _project_id: &str,
    _keyring_domain: &Option<String>,
    _kdbx_handle: &Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
) -> (Option<String>, Option<String>) {
    (None, None)
}

/// Fetch total project count from GCP API
///
/// # Arguments
/// * `access_token` - Valid (non-expired) OAuth access token. Caller should use get_valid_access_token() to ensure token is fresh.
/// Format project count for display
fn format_project_count_display(count: Option<usize>) -> String {
    match count {
        None => "—".to_string(),
        Some(0) => "0 projects".to_string(),
        Some(1) => "1 project".to_string(),
        Some(n) => format!("{} projects", n),
    }
}

/// Render drawer content showing platform hierarchy
fn render_drawer_content(
    ui: &mut egui::Ui,
    row: &PlatformRow,
    current_profile_kdbx: &Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
) {
    use crate::ui_tabs::platform_drawer;
    use crate::viewmodel::platform::{DrawerState, DrawerTab};

    // Create unique ID for this row's drawer state
    // Include last_refresh_time to invalidate cache when platform refreshes
    let drawer_id = egui::Id::new("platform_drawer")
        .with(&row.project_id)
        .with(row.last_refresh_time.unwrap_or(0));

    // Load active tab from persistent storage (default to Status)
    let active_tab: DrawerTab =
        ui.data_mut(|d| d.get_persisted(drawer_id).unwrap_or(DrawerTab::Status));

    // Create drawer state with persisted tab
    let mut drawer_state = DrawerState::new();
    drawer_state.active_tab = active_tab;
    if let Some(project_id) = &row.selected_project_id {
        drawer_state.set_project(project_id);

        // Load logs from temp storage if available
        if let Some(logs) = ui
            .ctx()
            .data(|d| d.get_temp::<Vec<String>>(drawer_id.with("logs")))
        {
            drawer_state.set_logs(logs);
        }

        // Load operations from temp storage if available
        if let Some(operations) = ui.ctx().data(|d| {
            d.get_temp::<Vec<crate::storage::models::opslog::OperationLog>>(
                drawer_id.with("operations"),
            )
        }) {
            drawer_state.set_operations(operations);
        }
    }

    let mut tab_switch: Option<DrawerTab> = None;

    platform_drawer::render_drawer(
        ui,
        row,
        &drawer_state,
        &mut tab_switch,
        current_profile_kdbx,
    );

    // Persist tab switch and trigger auto-load
    if let Some(new_tab) = tab_switch {
        ui.data_mut(|d| d.insert_persisted(drawer_id, new_tab));

        // Auto-load logs when switching to Logs tab
        if new_tab == DrawerTab::Logs {
            if let Some(ref project_id) = row.selected_project_id {
                ui.data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new("drawer_action_load_logs_on_switch"),
                        project_id.clone(),
                    );
                });
            }
        }

        // Auto-load operations when switching to Operations tab
        if new_tab == DrawerTab::Operations {
            if let Some(ref project_id) = row.selected_project_id {
                ui.data_mut(|d| {
                    d.insert_temp(
                        egui::Id::new("drawer_action_load_operations_on_switch"),
                        project_id.clone(),
                    );
                });
            }
        }
    }
}

impl PlatformTab {
    /// Reset loaded flag to force config reload on next render
    pub fn reset_loaded(&mut self) {
        self.loaded = false;
    }

    /// Check if config file has changed since last load
    #[cfg(not(target_arch = "wasm32"))]
    pub fn should_reload_config(
        &self,
        profile: &Option<crate::calc::profile::ProfileContext>,
    ) -> bool {
        // If never loaded, should reload
        if self.config_last_modified.is_none() {
            return true;
        }

        // Get current config file metadata
        if let Ok(config_path) = get_config_path(profile) {
            if let Ok(metadata) = std::fs::metadata(&config_path) {
                if let Ok(current_modified) = metadata.modified() {
                    // Compare with cached metadata
                    return Some(current_modified) != self.config_last_modified;
                }
            }
        }

        // If we can't get metadata, don't reload (ponytail: fail safe)
        false
    }

    #[cfg(target_arch = "wasm32")]
    pub fn should_reload_config(
        &self,
        _profile: &Option<crate::calc::profile::ProfileContext>,
    ) -> bool {
        // WASM doesn't have config files
        false
    }

    /// Save SSH test results to config (Task 6)
    #[cfg(not(target_arch = "wasm32"))]
    fn save_ssh_results_to_config(
        &self,
        completed: &[(
            String,
            Result<crate::calc::ssh::SshConnectionResult, String>,
        )],
        profile: &crate::calc::profile::ProfileContext,
    ) {
        if let Ok((mut config, config_path)) = load_config(&Some(profile.clone())) {
            for (project_id, result) in completed {
                if let Some(platform) = config
                    .platforms
                    .iter_mut()
                    .find(|p| p.gcp_selected_project_id.as_deref() == Some(project_id))
                {
                    match result {
                        Ok(conn_result) => {
                            platform.cached_ssh_connection_success = Some(conn_result.success);
                            platform.cached_ssh_connection_message =
                                Some(conn_result.message.clone());
                            dure_info!(
                                "Saved SSH test result for {}: {}",
                                project_id,
                                if conn_result.success {
                                    "success"
                                } else {
                                    "failed"
                                }
                            );
                        }
                        Err(err) => {
                            platform.cached_ssh_connection_success = Some(false);
                            platform.cached_ssh_connection_message = Some(err.clone());
                            dure_warn!("Saved SSH test error for {}: {}", project_id, err);
                        }
                    }
                }
            }
            if let Err(e) = config.save(&config_path) {
                dure_warn!("Failed to save SSH test results to config: {}", e);
            }
        }
    }

    /// Save firewall status to config (Task 6)
    #[cfg(not(target_arch = "wasm32"))]
    fn save_firewall_status_to_config(
        &self,
        project_id: &str,
        firewall_status: &str,
        profile: &crate::calc::profile::ProfileContext,
    ) {
        if let Ok((mut config, config_path)) = load_config(&Some(profile.clone())) {
            if let Some(platform) = config
                .platforms
                .iter_mut()
                .find(|p| p.gcp_selected_project_id.as_deref() == Some(project_id))
            {
                platform.cached_firewall_status = Some(firewall_status.to_string());
                dure_info!(
                    "Saved firewall status for {}: {}",
                    project_id,
                    firewall_status
                );

                if let Err(e) = config.save(&config_path) {
                    dure_warn!("Failed to save firewall status to config: {}", e);
                }
            }
        }
    }

    /// Render the platform tab UI
    pub fn ui(
        &mut self,
        current_profile: &Option<crate::calc::profile::ProfileContext>,
        current_profile_kdbx: &Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
        ui: &mut egui::Ui,
        mut vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        // Check if profile is selected
        if current_profile.is_none() {
            ui.label(tr!("no-profile-selected"));
            return;
        }

        // Unwrap profile (safe after guard above)
        let current_profile = current_profile.as_ref().unwrap();

        // ViewModel event processing (MVVM pattern)
        if let Some(ref mut vm) = vm {
            // Track if we need to fetch billing after discovery
            let mut pending_billing_fetch: Option<(String, String, String)> = None;

            // Process events first
            let events = vm.poll_events(ui.ctx());
            for event in events {
                use crate::viewmodel::platform::PlatformEvent;
                use crate::viewmodel::ViewModelEvent;

                match event {
                    ViewModelEvent::Platform(PlatformEvent::BillingFetched { records, .. }) => {
                        self.billing_dialog.data = Some(records);
                        self.billing_dialog.loading = false;
                    }
                    ViewModelEvent::Platform(PlatformEvent::BillingTableDiscovered {
                        platform_name,
                        dataset,
                        table,
                    }) => {
                        dure_info!(
                            "[BILLING] Discovered billing table: dataset={}, table={}",
                            dataset,
                            table
                        );
                        self.billing_dialog.dataset = dataset.clone();
                        self.billing_dialog.table = table.clone();

                        // Store for fetch after event loop
                        pending_billing_fetch = Some((platform_name, dataset, table));
                    }
                    ViewModelEvent::Platform(PlatformEvent::FirewallUpdated {
                        platform_name,
                        whitelisted_ip,
                        focus_tab,
                    }) => {
                        dure_debug!(
                            "✅ Successfully added {} to firewall whitelist",
                            whitelisted_ip
                        );

                        // Auto-focus drawer tab if specified
                        if let Some(tab) = focus_tab {
                            self.drawer_state.active_tab = tab;
                        }

                        let firewall_status = format!("✅ Whitelisted ({})", whitelisted_ip);

                        // Incremental update: Find and update specific row
                        if let Some(row) =
                            self.rows.iter_mut().find(|r| r.project_id == platform_name)
                        {
                            row.operation_state = OperationState::Completed {
                                operation: "firewall".to_string(),
                                completed_at: chrono::Utc::now().timestamp(),
                            };
                            row.firewall_status = firewall_status.clone();
                            row.firewall_updated = true;
                        }

                        // Save firewall status to config (Task 6)
                        #[cfg(not(target_arch = "wasm32"))]
                        self.save_firewall_status_to_config(
                            &platform_name,
                            &firewall_status,
                            current_profile,
                        );

                        // Note: NO self.loaded = false! Incremental update only
                    }
                    ViewModelEvent::Platform(PlatformEvent::FirewallRulesFetched {
                        platform_name,
                        rules,
                        current_ip,
                    }) => {
                        dure_info!(
                            "✅ [UI::PLATFO] Firewall rules fetched for {}: {} rules, IP: {}",
                            platform_name,
                            rules.len(),
                            current_ip
                        );

                        // Check if this is for the firewall dialog
                        if self.firewall_dialog.open && self.firewall_dialog.platform == platform_name {
                            dure_info!(
                                "[UI::PLATFO] Updating UI with {} firewall rules",
                                rules.len()
                            );
                            self.firewall_dialog.current_ip = Some(current_ip);
                            // Use full data directly - no conversion needed!
                            self.firewall_dialog.rule_list = rules;
                        } else {
                            dure_info!("[UI::PLATFO] Ignoring fetched rules: dialog_open={}, platform_match={}",
                                self.firewall_dialog.open, self.firewall_dialog.platform == platform_name);
                        }
                    }
                    ViewModelEvent::Platform(PlatformEvent::FirewallRuleUpdated {
                        platform_name,
                        rule_name,
                        operation,
                        focus_tab,
                    }) => {
                        dure_info!("✅ Firewall rule '{}' updated ({})", rule_name, operation);

                        // Auto-focus drawer tab if specified
                        if let Some(tab) = focus_tab {
                            self.drawer_state.active_tab = tab;
                        }

                        // Force immediate refresh for save operations, debounce for others
                        let force_refresh = self.add_firewall_dialog.save_in_progress
                            && (operation == "create" || operation == "delete");

                        // Refresh firewall data to show updated state
                        if self.firewall_dialog.open && self.firewall_dialog.platform == platform_name {
                            dure_info!("[UI::PLATFO] Triggering firewall data refresh after {} operation (force={})",
                                operation, force_refresh);
                            let profile_config_path = current_profile.config_file.clone();
                            self.refresh_firewall_list_if_needed(
                                profile_config_path,
                                platform_name,
                                Some(vm),
                                force_refresh,
                            );
                        } else {
                            dure_info!(
                                "[UI::PLATFO] Skipping refresh: dialog_open={}, platform_match={}",
                                self.firewall_dialog.open,
                                self.firewall_dialog.platform == platform_name
                            );
                        }

                        // Close add/edit dialog if save was in progress
                        if self.add_firewall_dialog.save_in_progress
                            && (operation == "create" || operation == "delete")
                        {
                            dure_info!("Closing Add/Edit Firewall Rule dialog after successful {} and immediate list refresh", operation);
                            self.add_firewall_dialog.open = false;
                            self.add_firewall_dialog.save_in_progress = false;
                            self.add_firewall_dialog.save_error = None;
                            self.add_firewall_dialog.edit_idx = None;
                            self.add_firewall_dialog.form_data = FirewallRule::default();

                            // Clear persisted string state
                            let state_id = egui::Id::new("firewall_form_strings");
                            ui.ctx().data_mut(|d| {
                                d.remove::<(
                                    String,
                                    String,
                                    String,
                                    String,
                                    String,
                                    String,
                                    String,
                                    String,
                                    String,
                                )>(state_id)
                            });
                        }

                        // Close subdialogs
                        self.firewall_dialog.show_remove_ip_dialog = false;
                        self.firewall_dialog.show_flush_confirm = false;
                    }
                    ViewModelEvent::Platform(PlatformEvent::VMRestarted {
                        platform_name,
                        vm_name,
                        focus_tab,
                    }) => {
                        // Auto-focus drawer tab if specified
                        if let Some(tab) = focus_tab {
                            self.drawer_state.active_tab = tab;
                        }

                        // Log with project_id context for filtering
                        crate::viewmodel::logs::append_log(
                            Some(platform_name.clone()),
                            crate::viewmodel::logs::LogLevel::Info,
                            format!("✅ VM {} restarted successfully", vm_name),
                        );

                        // Incremental update
                        if let Some(row) =
                            self.rows.iter_mut().find(|r| r.project_id == platform_name)
                        {
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
                        self.error_dialog.close();
                    }
                    ViewModelEvent::Platform(PlatformEvent::VMsScanned {
                        platform_name,
                        vm_count,
                    }) => {
                        dure_info!(
                            "✅ Scanned and imported {} VMs for platform '{}'",
                            vm_count,
                            platform_name
                        );

                        // Update operation state
                        if let Some(row) =
                            self.rows.iter_mut().find(|r| r.project_id == platform_name)
                        {
                            row.operation_state = OperationState::Completed {
                                operation: "scan".to_string(),
                                completed_at: chrono::Utc::now().timestamp(),
                            };
                        }

                        // Trigger reload to show imported VMs
                        self.loaded = false;
                        self.error_dialog.close();
                    }
                    ViewModelEvent::Platform(PlatformEvent::VMCreated {
                        platform_name,
                        vm_name,
                        external_ip,
                        focus_tab,
                    }) => {
                        // Auto-focus drawer tab if specified
                        if let Some(tab) = focus_tab {
                            self.drawer_state.active_tab = tab;
                        }

                        dure_info!(
                            "✅ VM '{}' created successfully with IP {}",
                            vm_name,
                            external_ip
                        );

                        // Incremental update
                        if let Some(row) =
                            self.rows.iter_mut().find(|r| r.project_id == platform_name)
                        {
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
                        focus_tab,
                    }) => {
                        // Auto-focus drawer tab if specified
                        if let Some(tab) = focus_tab {
                            self.drawer_state.active_tab = tab;
                        }

                        dure_info!("✅ VM {} deleted successfully", vm_name);

                        // Update operation state before reload
                        if let Some(row) =
                            self.rows.iter_mut().find(|r| r.project_id == platform_name)
                        {
                            row.operation_state = OperationState::Completed {
                                operation: "delete_vm".to_string(),
                                completed_at: chrono::Utc::now().timestamp(),
                            };
                        }

                        // Keep config update and reload logic
                        if let Ok((mut app_config, config_path)) =
                            load_config(&Some(current_profile.clone()))
                        {
                            if let Some(platform) = app_config.platforms.iter_mut().find(|p| {
                                p.gcp_selected_project_id.as_ref() == Some(&platform_name)
                            }) {
                                platform.vms.retain(|vm| vm.name != vm_name);

                                if let Err(e) = app_config.save(&config_path) {
                                    self.error_dialog.open(
                                        tr!("platform-error-save-config", {error: e.to_string()})
                                    );
                                } else {
                                    dure_info!("✅ Config updated, refreshing table");
                                    self.loaded = false;
                                    self.error_dialog.close();
                                }
                            }
                        }
                    }
                    ViewModelEvent::Platform(PlatformEvent::VmStaticIpChecked {
                        platform_name: _,
                        vm_name: _,
                        has_static_ip,
                        ip_address,
                        ip_name,
                    }) => {
                        dure_info!("✅ Static IP check complete: has_static={}", has_static_ip);
                        self.delete_vm_dialog.has_static_ip = has_static_ip;
                        self.delete_vm_dialog.ip_address = ip_address;
                        self.delete_vm_dialog.ip_name = ip_name;
                    }
                    ViewModelEvent::Platform(PlatformEvent::ProjectsListed {
                        platform_name,
                        projects,
                    }) => {
                        dure_debug!(
                            "✅ Projects listed for {}: {} projects",
                            platform_name,
                            projects.len()
                        );

                        // Populate project lists for any open dialogs
                        if self.add_platform_dialog.open || self.wizard_dialog_open {
                            self.add_platform_dialog.project_list = projects.clone();
                        }
                        if self.select_project_dialog.open {
                            self.select_project_dialog.project_list = projects;
                        }
                        // Note: Don't auto-open Select Project dialog - user opens it explicitly when needed
                    }
                    ViewModelEvent::Platform(PlatformEvent::VMsListed { platform_name, vms }) => {
                        dure_debug!("✅ VMs listed for {}: {} VMs", platform_name, vms.len());

                        // Check if this is for the bind VM dialog
                        if self.bind_vm_dialog.open && self.bind_vm_dialog.platform == platform_name {
                            let vm_list: Vec<_> = vms
                                .into_iter()
                                .map(|vm| {
                                    let zone_name =
                                        crate::api::gcp::compute::extract_zone_name(&vm.zone);
                                    (
                                        vm.project_id,
                                        vm.name,
                                        zone_name,
                                        vm.external_ip.unwrap_or_default(),
                                    )
                                })
                                .collect();
                            self.bind_vm_dialog.open(platform_name, vm_list);
                        }
                    }
                    ViewModelEvent::Platform(PlatformEvent::ProjectSelected {
                        project_id, ..
                    }) => {
                        dure_info!(" Project selected: {}", project_id);
                        // Refresh spreadsheet to show updated project
                        self.loaded = false;
                        self.error_dialog.close();
                    }
                    ViewModelEvent::Platform(PlatformEvent::PlatformAdded {
                        platform_name,
                        platform_type,
                    }) => {
                        dure_info!(" Platform '{}' ({}) added", platform_name, platform_type);
                        // Refresh spreadsheet to show new platform
                        self.loaded = false;
                        self.error_dialog.close();
                    }
                    ViewModelEvent::Platform(PlatformEvent::PlatformDeleted {
                        platform_name,
                        vm_count,
                    }) => {
                        dure_info!(" Platform '{}' deleted ({} VMs)", platform_name, vm_count);
                        // Refresh spreadsheet to show removal
                        self.loaded = false;
                        self.error_dialog.close();
                    }
                    ViewModelEvent::Platform(PlatformEvent::Error { operation, error }) => {
                        if operation == "fetch_billing" {
                            self.billing_dialog.error = Some(error);
                            self.billing_dialog.loading = false;
                        } else if operation == "update_firewall" {
                            self.error_dialog.open(format!("Failed to update firewall: {}", error));
                        } else if operation == "create_firewall_rule"
                            || operation == "delete_firewall_rule"
                        {
                            // Show error in dialog, keep it open
                            self.add_firewall_dialog.save_error = Some(error);
                            self.add_firewall_dialog.save_in_progress = false;
                        } else if operation == "restart_vm" {
                            self.error_dialog.open(format!("Failed to restart VM: {}", error));
                        } else if operation == "delete_vm" {
                            self.error_dialog.open(format!("Failed to delete VM: {}", error));
                        } else if operation == "list_vms" {
                            dure_error!("Failed to list VMs: {}", error);
                            self.bind_vm_dialog.loading = false;
                            self.bind_vm_dialog.vm_list = Vec::new();
                        } else if operation == "fetch_firewall_rules" {
                            dure_error!("Failed to fetch firewall rules: {}", error);
                            self.firewall_dialog.current_ip = None;
                            self.firewall_dialog.rule_list = Vec::new();
                        }
                    }
                    ViewModelEvent::Platform(PlatformEvent::RefreshCompleted {
                        platform_name,
                        vm_status,
                        firewall_status,
                        ssh_status,
                        project_count,
                        focus_tab,
                    }) => {
                        // Auto-focus drawer tab if specified
                        if let Some(tab) = focus_tab {
                            self.drawer_state.active_tab = tab;
                        }

                        dure_info!("✅ Refresh completed for {}", platform_name);
                        dure_info!(
                            "🔍 RefreshCompleted event: firewall_status.whitelisted = {}",
                            firewall_status.whitelisted
                        );
                        dure_info!(
                            "🔍 RefreshCompleted event: firewall_status.current_ip = {:?}",
                            firewall_status.current_ip
                        );

                        // Find and update the row
                        if let Some(row) =
                            self.rows.iter_mut().find(|r| r.project_id == platform_name)
                        {
                            // Update VM status
                            row.vm_created = vm_status.exists;
                            row.vm_name = vm_status.name;
                            row.vm_external_ip = vm_status.external_ip;
                            row.has_vm = vm_status.exists;
                            if let Some(zone) = vm_status.zone {
                                row.vm_zone = Some(zone);
                            }

                            // Update firewall status
                            row.firewall_updated = firewall_status.whitelisted;
                            if let Some(current_ip) = firewall_status.current_ip {
                                if firewall_status.whitelisted {
                                    dure_info!(
                                        "🔍 Setting row.firewall_status = '✅ Whitelisted ({})'",
                                        current_ip
                                    );
                                    row.firewall_status =
                                        format!("✅ Whitelisted ({})", current_ip);
                                } else {
                                    dure_info!("🔍 Setting row.firewall_status = '❌ Not whitelisted ({})'", current_ip);
                                    row.firewall_status =
                                        format!("❌ Not whitelisted ({})", current_ip);
                                }
                            } else {
                                dure_info!("🔍 Setting row.firewall_status = '? Status unknown'");
                                row.firewall_status = "? Status unknown".to_string();
                            }

                            // Update SSH status
                            row.ssh_ready = ssh_status.connected;
                            if ssh_status.connected {
                                row.ssh_status = "✅ Ready".to_string();
                            } else if let Some(error) = &ssh_status.error {
                                row.ssh_status = format!("❌ {}", error);
                            } else {
                                row.ssh_status = "? Unknown".to_string();
                            }

                            // Update project count
                            row.total_project_count = project_count;

                            // Clear operation state (refresh complete)
                            row.operation_state = OperationState::Completed {
                                operation: "refresh".to_string(),
                                completed_at: chrono::Utc::now().timestamp(),
                            };

                            // Update last refresh time
                            row.last_refresh_time = Some(chrono::Utc::now().timestamp());
                        }
                    }
                    ViewModelEvent::Platform(PlatformEvent::IpWhitelistChecked {
                        platform_name,
                        whitelisted,
                        current_ip,
                    }) => {
                        // Update row firewall status
                        if let Some(row) =
                            self.rows.iter_mut().find(|r| r.project_id == platform_name)
                        {
                            row.firewall_updated = whitelisted;
                            row.firewall_status = if whitelisted {
                                format!("✅ Whitelisted ({})", current_ip)
                            } else {
                                format!("❌ Not whitelisted ({})", current_ip)
                            };
                        }
                    }
                    ViewModelEvent::Platform(PlatformEvent::OperationFailed {
                        platform_name,
                        operation,
                        error,
                    }) => {
                        dure_error!(
                            "❌ Operation '{}' failed for {}: {}",
                            operation,
                            platform_name,
                            error
                        );

                        // Update row to show error state
                        if let Some(row) =
                            self.rows.iter_mut().find(|r| r.project_id == platform_name)
                        {
                            row.operation_state = OperationState::Failed {
                                operation: operation.clone(),
                                error: error.clone(),
                                failed_at: chrono::Utc::now().timestamp(),
                            };
                        }
                    }
                    ViewModelEvent::Drawer(drawer_event) => {
                        use crate::viewmodel::platform::DrawerEvent;
                        match drawer_event {
                            DrawerEvent::LogsLoaded { project_id, lines } => {
                                // Store logs in egui temp storage using drawer_id
                                // Find row to get last_refresh_time for cache invalidation
                                if let Some(row) =
                                    self.rows.iter().find(|r| r.project_id == project_id)
                                {
                                    let drawer_id = egui::Id::new("platform_drawer")
                                        .with(&project_id)
                                        .with(row.last_refresh_time.unwrap_or(0));
                                    ui.ctx().data_mut(|d| {
                                        d.insert_temp(drawer_id.with("logs"), lines.clone());
                                    });
                                }
                            }
                            DrawerEvent::OperationsLoaded { project_id, logs } => {
                                // Store operations in egui temp storage using drawer_id
                                // Find row to get last_refresh_time for cache invalidation
                                if let Some(row) =
                                    self.rows.iter().find(|r| r.project_id == project_id)
                                {
                                    let drawer_id = egui::Id::new("platform_drawer")
                                        .with(&project_id)
                                        .with(row.last_refresh_time.unwrap_or(0));
                                    ui.ctx().data_mut(|d| {
                                        d.insert_temp(drawer_id.with("operations"), logs.clone());
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }

            // Auto-fetch billing after discovery completes
            if let Some((platform_name, dataset, table)) = pending_billing_fetch {
                dure_info!(
                    "[BILLING] Auto-fetching billing data after discovery: platform={}, dataset={}, table={}",
                    platform_name,
                    dataset,
                    table
                );

                if let Err(e) = vm.fetch_billing(
                    current_profile.config_file.clone(),
                    platform_name.clone(),
                    platform_name,
                    dataset,
                    table,
                ) {
                    dure_error!("[BILLING] Auto-fetch failed: {}", e);
                    self.billing_dialog.error = Some(format!("Auto-fetch failed: {}", e));
                    self.billing_dialog.loading = false;
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
                use egui_material3::theme::get_global_color;
                EmojiLabel::new(
                    egui::RichText::new(format!(
                        "⚠️ Error in {}: {}",
                        error.operation, error.error
                    ))
                    .color(get_global_color("error")),
                )
                .show(ui);
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
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs(1));

        // Poll SSH test promises (TODO: Replace with ViewModel event processing)
        #[cfg(not(target_arch = "wasm32"))]
        {
            let mut completed = Vec::new();
            for (platform_name, promise) in &self.ssh_test_promises {
                if let Some(result) = promise.ready() {
                    completed.push((platform_name.clone(), result.clone()));
                    self.ssh_test_results
                        .insert(platform_name.clone(), result.clone());
                }
            }
            if !completed.is_empty() {
                // Save SSH test results to config (Task 6)
                self.save_ssh_results_to_config(&completed, &current_profile);

                // Reload rows to update SSH status and keys in drawer
                self.loaded = false;
            }
            for (platform_name, _) in completed {
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

        // Table rendering
        if !self.loaded {
            self.load_rows(&Some(current_profile.clone()), current_profile_kdbx);
        }

        // Auto-trigger refresh for rows with missing project count cache (once per platform)
        #[cfg(not(target_arch = "wasm32"))]
        {
            for row in &self.rows {
                if row.total_project_count.is_none()
                    && !self.auto_refreshed_platforms.contains(&row.project_id)
                    && !self.refresh_promises.contains_key(&row.project_id)
                {
                    if let Some(ref vm) = vm {
                        let profile_config_path = current_profile.config_file.clone();
                        if let Ok(_) = vm.refresh_platform(
                            profile_config_path,
                            row.project_id.clone(),
                            current_profile_kdbx.clone(),
                        ) {
                            // Mark as auto-refreshed to prevent repeat triggers
                            self.auto_refreshed_platforms.insert(row.project_id.clone());
                        } else {
                            dure_debug!("Auto-refresh failed for {}", row.project_id);
                        }
                    }
                }
            }
        }

        // Show error dialog
        self.error_dialog.show(ui.ctx());

        if self.rows.is_empty() && !self.error_dialog.open {
            ui.heading(tr!("platform-main-heading"));
            ui.add_space(4.0);
            ui.label(tr!("platform-main-desc"));
            ui.add_space(8.0);

            if ui
                .add(MaterialButton::filled(tr!("platform-button-add")))
                .clicked()
            {
                self.wizard_dialog_open = true;
                self.wizard_current_step = 0;
                self.add_platform_dialog.platform_type = "gcp".to_string();
            }
            ui.add_space(8.0);
            ui.label(tr!("platform-main-empty"));
        } else if !self.rows.is_empty() {
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
                .min_row_height(68.0)
                .drawer_row_height(500.0)
                .column(
                    &tr!("platform-table-col-project"),
                    230.0 * width_ratio,
                    false,
                )
                .column(
                    &tr!("platform-table-col-operations"),
                    510.0 * width_ratio,
                    false,
                );

            for (idx, row) in self.rows.iter().enumerate() {
                let row_for_cells = row.clone();
                let row_for_drawer = row.clone();
                let row_for_actions = row.clone();
                let kdbx_for_drawer = current_profile_kdbx.clone();

                table = table.row(move |r| {
                    r.cell_widget(move |ui| {
                        ui.vertical(|ui| {
                            ui.add(
                                badge(&row_for_cells.platform_type)
                                    .color(BadgeColor::Primary)
                                    .size(BadgeSize::Small),
                            );
                            use egui_material3::theme::get_global_color;
                            ui.label(
                                egui::RichText::new(&row_for_cells.project_id)
                                    .size(16.0)
                                    .strong()
                                    .color(get_global_color("onSurface")),
                            );

                            // Display domain badge if bound
                            if let Some(domain) = &row_for_cells.bound_domain {
                                ui.add_space(4.0);
                                ui.add(
                                    badge(domain)
                                        .color(BadgeColor::Success)
                                        .size(BadgeSize::Small),
                                );
                            }
                        });
                    })
                    .cell_widget(move |ui| {
                        // Calculate needed height for button wrapping
                        let column_width = 260.0 * width_ratio;
                        let button_width = 90.0; // Approximate width including spacing
                        let button_height = 32.0; // Button height + vertical spacing

                        // Count buttons (platform-dependent)
                        let button_count =
                            if cfg!(any(target_os = "android", target_arch = "wasm32")) {
                                6 // Mobile/WASM: Refresh, Scan VMs, Firewall, Restart, Del VM, Delete
                            } else {
                                8 // Desktop: + Add VM, Billing
                            };

                        let buttons_per_row = (column_width / button_width).floor().max(1.0);
                        let needed_rows = (button_count as f32 / buttons_per_row).ceil();
                        let needed_height = needed_rows * button_height + 8.0; // Extra padding

                        // Set minimum height before rendering
                        ui.set_min_height(needed_height);

                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = BUTTON_VERTICAL_SPACING;
                            ui.spacing_mut().item_spacing.x = BUTTON_HORIZONTAL_SPACING;
                            ui.style_mut().spacing.button_padding = egui::vec2(6.0, 2.0);

                            // Check if any operation in progress
                            let operation_in_progress = matches!(
                                row_for_actions.operation_state,
                                OperationState::InProgress { .. }
                            );

                            // Store top position for foreground overlay
                            let overlay_pos = ui.cursor().min;

                            // Row 1: Refresh, Re-authenticate (if needed), Billing, Delete
                            ui.horizontal(|ui| {
                                // Refresh (always enabled)
                                if ui
                                    .add(
                                        MaterialButton::outlined(tr!("platform-button-refresh"))
                                            .small(),
                                    )
                                    .on_hover_text(tr!("platform-button-refresh-tooltip"))
                                    .clicked()
                                {
                                    ui.data_mut(|d| {
                                        d.insert_temp(
                                            egui::Id::new("platform_action_refresh"),
                                            row_for_actions.project_id.clone(),
                                        )
                                    });
                                }

                                // Re-authenticate (shown when OAuth refresh fails)
                                if row_for_actions.oauth_refresh_failed {
                                    if ui
                                        .add(
                                            MaterialButton::filled(tr!("platform-button-reauth"))
                                                .small(),
                                        )
                                        .on_hover_text(tr!("platform-button-reauth-tooltip"))
                                        .clicked()
                                    {
                                        ui.data_mut(|d| {
                                            d.insert_temp(
                                                egui::Id::new("platform_action_reauth"),
                                                row_for_actions.project_id.clone(),
                                            )
                                        });
                                    }
                                }

                                ui.add_enabled_ui(
                                    !operation_in_progress && !row_for_actions.oauth_refresh_failed,
                                    |ui| {
                                        // Billing
                                        #[cfg(not(any(
                                            target_os = "android",
                                            target_arch = "wasm32"
                                        )))]
                                        if ui
                                            .add_enabled(
                                                row_for_actions.project_selected
                                                    && row_for_actions
                                                        .selected_project_id
                                                        .is_some(),
                                                MaterialButton::outlined(tr!(
                                                    "platform-button-billing"
                                                ))
                                                .small(),
                                            )
                                            .on_hover_text(tr!("platform-button-billing-tooltip"))
                                            .clicked()
                                        {
                                            ui.data_mut(|d| {
                                                d.insert_temp(
                                                    egui::Id::new("platform_action_billing_name"),
                                                    row_for_actions.project_id.clone(),
                                                );
                                                if let Some(project_id) =
                                                    &row_for_actions.selected_project_id
                                                {
                                                    d.insert_temp(
                                                        egui::Id::new(
                                                            "platform_action_billing_project",
                                                        ),
                                                        project_id.clone(),
                                                    );
                                                }
                                            });
                                        }

                                        // Delete
                                        if ui
                                            .add(
                                                MaterialButton::outlined(tr!(
                                                    "platform-button-delete"
                                                ))
                                                .small(),
                                            )
                                            .on_hover_text(tr!("platform-button-delete-tooltip"))
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
                                    },
                                );
                            });

                            // Row 2: Add VM, Scan VMs, Firewall, Restart, DelVM (disabled when OAuth fails)
                            ui.horizontal(|ui| {
                                ui.add_enabled_ui(
                                    !operation_in_progress && !row_for_actions.oauth_refresh_failed,
                                    |ui| {
                                        // Add VM
                                        #[cfg(not(any(
                                            target_os = "android",
                                            target_arch = "wasm32"
                                        )))]
                                        if ui
                                            .add_enabled(
                                                !row_for_actions.has_vm
                                                    && row_for_actions.project_selected,
                                                MaterialButton::outlined(tr!(
                                                    "platform-button-add-vm"
                                                ))
                                                .small(),
                                            )
                                            .on_hover_text(tr!("platform-button-add-vm-tooltip"))
                                            .clicked()
                                        {
                                            ui.data_mut(|d| {
                                                d.insert_temp(
                                                    egui::Id::new("platform_action_add_vm"),
                                                    row_for_actions.project_id.clone(),
                                                )
                                            });
                                        }

                                        // Bind VM
                                        if ui
                                            .add_enabled(
                                                row_for_actions.project_selected,
                                                MaterialButton::outlined(tr!(
                                                    "platform-button-bind-vm"
                                                ))
                                                .small(),
                                            )
                                            .on_hover_text(tr!("platform-button-bind-vm-tooltip"))
                                            .clicked()
                                        {
                                            ui.data_mut(|d| {
                                                d.insert_temp(
                                                    egui::Id::new("platform_action_bind_vm"),
                                                    row_for_actions.project_id.clone(),
                                                )
                                            });
                                        }

                                        // Firewall
                                        if ui
                                            .add_enabled(
                                                row_for_actions.project_selected
                                                    && row_for_actions.gcp_connected,
                                                MaterialButton::outlined(tr!(
                                                    "platform-button-firewall"
                                                ))
                                                .small(),
                                            )
                                            .on_hover_text("Manage Firewall Rules")
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

                                        // Restart
                                        if ui
                                            .add_enabled(
                                                row_for_actions.has_vm,
                                                MaterialButton::outlined(tr!(
                                                    "platform-button-restart"
                                                ))
                                                .small(),
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

                                        // Del VM
                                        if ui
                                            .add_enabled(
                                                row_for_actions.has_vm,
                                                MaterialButton::outlined(tr!(
                                                    "platform-button-del-vm"
                                                ))
                                                .small(),
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
                                    },
                                );
                            });

                            // Render progress indicator on foreground layer (overlay)
                            if operation_in_progress {
                                let current_area_id = ui.layer_id().id;
                                let foreground_layer =
                                    egui::LayerId::new(egui::Order::Foreground, current_area_id);

                                ui.with_layer_id(foreground_layer, |ui| {
                                    // Position at stored overlay position
                                    let progress_rect = egui::Rect::from_min_size(
                                        overlay_pos,
                                        egui::vec2(column_width, 30.0),
                                    );

                                    // Semi-transparent background
                                    use egui_material3::theme::get_global_color;
                                    let overlay_color = get_global_color("surfaceContainer");
                                    ui.painter().rect_filled(
                                        progress_rect,
                                        egui::Rounding::same(4),
                                        overlay_color,
                                    );

                                    // Draw progress label and bar
                                    ui.allocate_ui_at_rect(progress_rect, |ui| {
                                        ui.add_space(2.0);

                                        if let OperationState::InProgress { operation, .. } =
                                            &row_for_actions.operation_state
                                        {
                                            ui.label(operation);
                                        }

                                        ui.add(
                                            linear_progress()
                                                .indeterminate(true)
                                                .width(column_width - 20.0)
                                                .height(4.0)
                                                .four_color_enabled(true),
                                        );
                                    });
                                });
                            }
                        });
                    })
                    .drawer(move |ui| {
                        render_drawer_content(ui, &row_for_drawer, &kdbx_for_drawer);
                    })
                });
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading(tr!("platform-main-heading"));
                ui.add_space(4.0);
                ui.label(tr!("platform-main-desc"));
                ui.add_space(8.0);

                if ui
                    .add(MaterialButton::filled(tr!("platform-button-add")))
                    .clicked()
                {
                    self.wizard_dialog_open = true;
                    self.wizard_current_step = 0;
                    self.add_platform_dialog.platform_type = "gcp".to_string();
                }
                ui.add_space(8.0);

                table.show(ui);
            });

            // Process pending actions from button clicks
            // Refresh action (available on all platforms)
            if let Some(platform_name) =
                ui.data(|d| d.get_temp::<String>(egui::Id::new("platform_action_refresh")))
            {
                // Optimistic update: Show "Refreshing..." state
                if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                    row.operation_state = OperationState::InProgress {
                        operation: tr!("refresh"),
                        started_at: chrono::Utc::now().timestamp(),
                    };
                }

                // Refresh access token if expired (before making API calls)
                #[cfg(not(target_arch = "wasm32"))]
                {
                    if let Ok((mut app_config, config_path)) =
                        load_config(&Some(current_profile.clone()))
                    {
                        if let Some((platform_idx, _)) =
                            app_config.platforms.iter().enumerate().find(|(_, p)| {
                                p.gcp_selected_project_id.as_ref() == Some(&platform_name)
                            })
                        {
                            // Check/refresh token (this will log "Access token refreshed" if needed)
                            if let Err(e) = self.get_valid_access_token(
                                &mut app_config,
                                platform_idx,
                                &config_path,
                            ) {
                                dure_warn!("Failed to refresh access token: {}", e);
                                // Continue anyway - the API call might still work or will fail with proper error
                            }
                        }
                    }
                }

                // Send RefreshPlatform command to ViewModel
                if let Some(ref vm) = vm {
                    let profile_config_path = current_profile.config_file.clone();
                    if let Err(e) = vm.refresh_platform(
                        profile_config_path,
                        platform_name.clone(),
                        current_profile_kdbx.clone(),
                    ) {
                        dure_error!("Failed to send refresh command: {}", e);
                        if let Some(row) =
                            self.rows.iter_mut().find(|r| r.project_id == platform_name)
                        {
                            row.operation_state = OperationState::Failed {
                                operation: "refresh".to_string(),
                                error: format!("Failed to start refresh: {}", e),
                                failed_at: chrono::Utc::now().timestamp(),
                            };
                        }
                    } else {
                        dure_info!(" Refresh command sent successfully");
                    }
                }

                ui.data_mut(|d| d.remove::<String>(egui::Id::new("platform_action_refresh")));
            }

            // Re-authenticate action (when OAuth token refresh fails)
            // TODO: Implement reauth via wizard
            if let Some(_platform_name) =
                ui.data(|d| d.get_temp::<String>(egui::Id::new("platform_action_reauth")))
            {
                dure_warn!("Re-authentication not yet implemented via wizard");
                ui.data_mut(|d| d.remove::<String>(egui::Id::new("platform_action_reauth")));
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                // Firewall action - open dialog to manage firewall rules
                if let Some(platform_name) = ui.data(|d| {
                    d.get_temp::<String>(egui::Id::new("platform_action_update_firewall"))
                }) {
                    self.firewall_dialog.platform = platform_name.clone();
                    self.firewall_dialog.open = true;

                    // Fetch current IP and firewall rules
                    let profile_config_path = current_profile.config_file.clone();
                    self.fetch_firewall_data(profile_config_path, platform_name, vm.as_deref_mut());

                    ui.data_mut(|d| {
                        d.remove::<String>(egui::Id::new("platform_action_update_firewall"))
                    });
                }

                // Bind VM action - open dialog to select VM
                if let Some(platform_name) =
                    ui.data(|d| d.get_temp::<String>(egui::Id::new("platform_action_bind_vm")))
                {
                    self.bind_vm_dialog.platform = platform_name.clone();
                    self.bind_vm_dialog.loading = true;
                    self.bind_vm_dialog.open = true;

                    // Fetch VM list from GCP
                    let profile_config_path = current_profile.config_file.clone();
                    self.fetch_vm_list_for_binding(
                        profile_config_path,
                        platform_name,
                        vm.as_deref_mut(),
                    );

                    ui.data_mut(|d| d.remove::<String>(egui::Id::new("platform_action_bind_vm")));
                }
            }

            // Drawer action: Refresh logs
            if let Some(project_id) =
                ui.data(|d| d.get_temp::<String>(egui::Id::new("drawer_action_refresh_logs")))
            {
                if let Some(ref vm) = vm {
                    use crate::viewmodel::platform::DrawerCommand;
                    if let Err(e) = vm.send_drawer_command(DrawerCommand::LoadLogs {
                        project_id: project_id.clone(),
                        limit: 1000,
                    }) {
                        dure_error!("Failed to send LoadLogs command: {}", e);
                    }
                }
                ui.data_mut(|d| d.remove::<String>(egui::Id::new("drawer_action_refresh_logs")));
            }

            // Drawer action: Refresh operations
            if let Some(project_id) =
                ui.data(|d| d.get_temp::<String>(egui::Id::new("drawer_action_refresh_operations")))
            {
                if let Some(ref vm) = vm {
                    use crate::viewmodel::platform::DrawerCommand;
                    if let Err(e) = vm.send_drawer_command(DrawerCommand::LoadOperations {
                        project_id: project_id.clone(),
                        limit: 100,
                    }) {
                        dure_error!("Failed to send LoadOperations command: {}", e);
                    }
                }
                ui.data_mut(|d| {
                    d.remove::<String>(egui::Id::new("drawer_action_refresh_operations"))
                });
            }

            // Drawer action: Auto-load logs on tab switch
            if let Some(project_id) = ui
                .data(|d| d.get_temp::<String>(egui::Id::new("drawer_action_load_logs_on_switch")))
            {
                if let Some(ref vm) = vm {
                    use crate::viewmodel::platform::DrawerCommand;
                    if let Err(e) = vm.send_drawer_command(DrawerCommand::LoadLogs {
                        project_id: project_id.clone(),
                        limit: 1000,
                    }) {
                        dure_error!("Failed to auto-load logs on tab switch: {}", e);
                    }
                }
                ui.data_mut(|d| {
                    d.remove::<String>(egui::Id::new("drawer_action_load_logs_on_switch"))
                });
            }

            // Drawer action: Auto-load operations on tab switch
            if let Some(project_id) = ui.data(|d| {
                d.get_temp::<String>(egui::Id::new("drawer_action_load_operations_on_switch"))
            }) {
                if let Some(ref vm) = vm {
                    use crate::viewmodel::platform::DrawerCommand;
                    if let Err(e) = vm.send_drawer_command(DrawerCommand::LoadOperations {
                        project_id: project_id.clone(),
                        limit: 100,
                    }) {
                        dure_error!("Failed to auto-load operations on tab switch: {}", e);
                    }
                }
                ui.data_mut(|d| {
                    d.remove::<String>(egui::Id::new("drawer_action_load_operations_on_switch"))
                });
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Some((platform_name, vm_name, vm_zone)) = ui.data(|d| {
                    d.get_temp::<(String, String, String)>(egui::Id::new(
                        "platform_action_delete_vm",
                    ))
                }) {
                    // Optimistic update
                    if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name)
                    {
                        row.operation_state = OperationState::InProgress {
                            operation: format!("Deleting VM {}", vm_name),
                            started_at: chrono::Utc::now().timestamp(),
                        };
                    }

                    self.show_delete_vm_confirmation(
                        &Some(current_profile.clone()),
                        vm.as_deref_mut(),
                        platform_name,
                        vm_name,
                        vm_zone,
                    );
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
                    if let Ok((app_config, _)) = load_config(&Some(current_profile.clone())) {
                        if let Some(platform) = app_config
                            .platforms
                            .iter()
                            .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
                        {
                            if let Some(vm_cfg) = platform.vms.first() {
                                self.regenerate_vm(
                                    current_profile,
                                    platform_name,
                                    vm_cfg.name.clone(),
                                    current_profile_kdbx.clone(),
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
                    // Find platform and get vm_name to show confirmation
                    if let Ok((app_config, _)) = load_config(&Some(current_profile.clone())) {
                        if let Some(platform) = app_config
                            .platforms
                            .iter()
                            .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
                        {
                            if let Some(vm_config) = platform.vms.first() {
                                dure_info!("Restart VM button clicked for '{}' - showing confirmation", vm_config.name);
                                self.restart_vm_confirm_dialog.open(
                                    vm_config.name.clone(),
                                    platform_name.clone(),
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
                    self.show_gcp_wizard(current_profile, current_profile_kdbx, platform_name);
                    ui.data_mut(|d| d.remove::<String>(egui::Id::new("platform_action_add_vm")));
                }

                if let Some(platform_name) =
                    ui.data(|d| d.get_temp::<String>(egui::Id::new("platform_action_billing_name")))
                {
                    if let Some(project_id) = ui.data(|d| {
                        d.get_temp::<String>(egui::Id::new("platform_action_billing_project"))
                    }) {
                        self.billing_dialog.open(project_id.clone());
                        self.fetch_billing_data(
                            current_profile,
                            vm.as_deref_mut(),
                            Some(project_id),
                        );
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
                self.show_delete_platform_confirmation(current_profile, platform_name);
                ui.data_mut(|d| {
                    d.remove::<String>(egui::Id::new("platform_action_delete_platform"))
                });
            }
        }

        // Add platform dialog - check OAuth promise first
        if self.add_platform_dialog.open {
            // Check for OAuth promise result
            if let Some(promise) = &self.add_platform_dialog.oauth_promise {
                if let Some(result) = promise.ready() {
                    match result {
                        Ok(oauth_result) => {
                            self.add_platform_dialog.oauth_result = Some(oauth_result.clone());
                            self.add_platform_dialog.connected_email = Some("Connected Account".to_string());
                            self.add_platform_dialog.oauth_promise = None;
                        }
                        Err(e) => {
                            self.error_dialog.open(format!("OAuth failed: {}", e));
                            self.add_platform_dialog.oauth_promise = None;
                        }
                    }
                }
            }

            // Fetch projects if OAuth succeeded and list is empty
            if let Some(ref oauth_result) = self.add_platform_dialog.oauth_result {
                if self.add_platform_dialog.connected_email.is_some()
                    && !self.add_platform_dialog.reauth_mode
                    && self.add_platform_dialog.project_list.is_empty()
                {
                    if let Some(ref vm) = vm {
                        let _ = vm.list_projects_with_token(
                            oauth_result.access_token.clone(),
                            "temp-platform".to_string(),
                        );
                    }
                }
            }

        }

        // Add Platform Wizard
        if self.wizard_dialog_open {
            self.render_add_platform_wizard(current_profile, ui.ctx(), vm.as_deref_mut());
        }

        // Delete Platform dialog
        match self.delete_platform_confirm_dialog.show(ui.ctx()) {
            crate::ui_dlg::platform::DeletePlatformConfirmAction::Delete {
                platform_name,
                delete_vms,
                delete_project,
            } => {
                self.handle_delete_platform(
                    current_profile,
                    platform_name,
                    delete_vms,
                    delete_project,
                    vm.as_deref_mut(),
                );
            }
            _ => {}
        }

        // Select Project dialog
        #[cfg(not(target_arch = "wasm32"))]
        match self.select_project_dialog.show(ui.ctx()) {
            crate::ui_dlg::platform::SelectProjectAction::Select {
                platform_name,
                project_id,
            } => {
                self.handle_select_project(current_profile, platform_name, project_id, vm.as_deref_mut());
            }
            _ => {}
        }

        // Delete VM dialog
        match self.delete_vm_dialog.show(ui.ctx()) {
            crate::ui_dlg::platform::DeleteVmAction::Delete {
                vm_name,
                zone,
                force,
                release_ip,
            } => {
                self.handle_delete_vm(current_profile, vm_name, zone, force, release_ip, vm.as_deref_mut());
            }
            _ => {}
        }

        // Billing dialog
        match self.billing_dialog.show(ui.ctx()) {
            crate::ui_dlg::platform::BillingAction::Refresh => {
                self.handle_billing_refresh(current_profile, vm.as_deref_mut());
            }
            _ => {}
        }

        // Bind VM dialog
        match self.bind_vm_dialog.show(ui.ctx()) {
            crate::ui_dlg::platform::BindVmAction::Bind { platform } => {
                self.handle_bind_vm(current_profile, platform, vm.as_deref_mut());
            }
            _ => {}
        }

        // Firewall dialog (skip if Add/Edit or Delete confirmation is open to maintain z-order)
        if self.firewall_dialog.open && !self.add_firewall_dialog.open && !self.delete_firewall_confirm_dialog.open {
            match self.firewall_dialog.show(ui.ctx()) {
                crate::ui_dlg::platform::FirewallAction::AddIp => {
                    self.handle_firewall_add_ip(current_profile, vm.as_deref_mut());
                }
                crate::ui_dlg::platform::FirewallAction::AddRule => {
                    self.handle_firewall_add_rule();
                }
                crate::ui_dlg::platform::FirewallAction::EditRule { rule_data, rule_idx } => {
                    self.handle_firewall_edit_rule(rule_data, rule_idx);
                }
                crate::ui_dlg::platform::FirewallAction::DeleteRule { rule_name, rule_idx } => {
                    self.handle_firewall_delete_rule(rule_name, rule_idx);
                }
                crate::ui_dlg::platform::FirewallAction::Reset => {
                    self.handle_firewall_reset(current_profile, vm.as_deref_mut());
                }
                crate::ui_dlg::platform::FirewallAction::Close => {
                    self.firewall_dialog.last_refresh_time = None;
                }
                _ => {}
            }
        } else if !self.firewall_dialog.open {
            // Clear debounce state when dialog is closed
            self.firewall_dialog.last_refresh_time = None;
        }

        // Add/Edit Firewall Rule dialog (renders on top when open)
        match self.add_firewall_dialog.show(ui.ctx()) {
            crate::ui_dlg::platform::AddFirewallAction::Save {
                rule_data,
                edit_idx,
            } => {
                self.handle_add_firewall_save(
                    current_profile,
                    rule_data,
                    edit_idx,
                    vm.as_deref_mut(),
                );
            }
            _ => {}
        }

        // Delete Firewall Rule confirmation dialog
        match self.delete_firewall_confirm_dialog.show(ui.ctx()) {
            crate::ui_dlg::platform::DeleteFirewallConfirmAction::Delete {
                rule_name,
                platform,
                rule_idx,
            } => {
                self.handle_delete_firewall_rule(
                    current_profile,
                    rule_name,
                    platform,
                    rule_idx,
                    vm.as_deref_mut(),
                );
            }
            _ => {}
        }

        // Restart VM confirmation dialog
        match self.restart_vm_confirm_dialog.show(ui.ctx()) {
            crate::ui_dlg::platform::RestartVmConfirmAction::Restart {
                vm_name,
                platform,
            } => {
                dure_info!("Restart confirmed for VM '{}' on platform '{}'", vm_name, platform);

                // Optimistic update
                if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform) {
                    row.operation_state = OperationState::InProgress {
                        operation: "Restarting VM".to_string(),
                        started_at: chrono::Utc::now().timestamp(),
                    };
                }

                // Get zone and restart
                if let Ok((app_config, _)) = load_config(&Some(current_profile.clone())) {
                    if let Some(plat) = app_config
                        .platforms
                        .iter()
                        .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform))
                    {
                        if let Some(vm_config) = plat.vms.first() {
                            self.restart_vm(
                                current_profile,
                                platform,
                                vm_name,
                                vm_config.zone.clone(),
                                vm.as_deref_mut(),
                            );
                        }
                    }
                }
            }
            _ => {}
        }

        // Init progress display
        if self.init_in_progress {
            self.render_init_progress(ui);
        }

        // GCP wizard dialog
        #[cfg(not(target_arch = "wasm32"))]
        {
            let wizard_is_open = self.gcp_wizard.is_some();

            if let Some(wizard) = &mut self.gcp_wizard {
                wizard.ui(ui.ctx());

                // Operation logging integration
                #[cfg(not(target_arch = "wasm32"))]
                {
                    use crate::storage::models::opslog::{NewOperationLog, OperationStatus};

                    // Create operation log when VM creation starts
                    if wizard.is_creating_vm() && wizard.get_operation_log_id().is_none() {
                        if let Ok(mut conn) = crate::calc::db::establish_connection_result() {
                            let log =
                                NewOperationLog::new(wizard.get_project_id(), "create_vm", "gcp");

                            if let Ok(log_id) =
                                crate::storage::models::opslog::create_log(&mut conn, log)
                            {
                                wizard.set_operation_log_id(log_id);
                                dure_info!("Created operation log for VM creation: {}", log_id);
                            }
                        }
                    }

                    // Update operation log on completion
                    if wizard.vm_creation_completed() {
                        if let Some(log_id) = wizard.get_operation_log_id() {
                            if let Ok(mut conn) = crate::calc::db::establish_connection_result() {
                                let _ = crate::storage::models::opslog::update_log_status(
                                    &mut conn,
                                    log_id,
                                    OperationStatus::Success,
                                    None,
                                );
                                wizard.mark_operation_log_updated();
                                dure_info!("Updated operation log {} to success", log_id);
                            }
                        }
                    }

                    // Update operation log on failure
                    if let Some(error) = wizard.vm_creation_failed() {
                        if let Some(log_id) = wizard.get_operation_log_id() {
                            if let Ok(mut conn) = crate::calc::db::establish_connection_result() {
                                let _ = crate::storage::models::opslog::update_log_status(
                                    &mut conn,
                                    log_id,
                                    OperationStatus::Failed,
                                    Some(error),
                                );
                                wizard.mark_operation_log_updated();
                                dure_info!("Updated operation log {} to failed", log_id);
                            }
                        }
                    }

                    // Trigger refresh when VM creation completes
                    if wizard.needs_refresh() {
                        dure_info!("VM creation completed, refreshing platform data");
                        self.loaded = false;
                        wizard.reset_refresh_flag();
                    }
                }
            }

            // Detect wizard closure - if it was open and now closed, refresh
            if self.wizard_was_open && !wizard_is_open {
                dure_info!(" GCP wizard closed, refreshing platform spreadsheet");
                self.loaded = false;
            }

            self.wizard_was_open = wizard_is_open;
        }

        // SSH creation notification (auto-dismiss after 5 seconds)
        if self.show_ssh_creation_notification {
            if let Some(show_time) = self.ssh_creation_notification_time {
                if show_time.elapsed() >= std::time::Duration::from_secs(5) {
                    self.show_ssh_creation_notification = false;
                }
            }

            let response = ui.add(
                notification()
                    .title("SSH Entries Created")
                    .text(&format!(
                        "Created {} new SSH entr{} from VM scan",
                        self.ssh_created_count,
                        if self.ssh_created_count == 1 {
                            "y"
                        } else {
                            "ies"
                        }
                    ))
                    .icon("check")
                    .align(NotificationAlign::Center)
                    .opened(true),
            );

            if response.clicked() {
                self.show_ssh_creation_notification = false;
            }
        }
    }

    fn load_rows(
        &mut self,
        current_profile: &Option<crate::calc::profile::ProfileContext>,
        current_profile_kdbx: &Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
    ) {
        self.rows.clear();
        self.error_dialog.close();

        #[cfg(not(target_arch = "wasm32"))]
        {
            match load_config(current_profile) {
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
                        let (access_token, oauth_refresh_failed) = if app_config.platforms[idx]
                            .gcp_oauth_access_token
                            .is_some()
                        {
                            match self.get_valid_access_token(&mut app_config, idx, &config_path) {
                                Ok(token) => (Some(token), false),
                                Err(e) => {
                                    let project_id = app_config.platforms[idx]
                                        .gcp_selected_project_id
                                        .as_deref()
                                        .unwrap_or("unknown");
                                    dure_warn!(
                                        "Failed to get valid access token for project '{}': {}",
                                        project_id,
                                        e
                                    );
                                    (None, true)
                                }
                            }
                        } else {
                            (None, false)
                        };

                        // Borrow platform after get_valid_access_token
                        let platform = &app_config.platforms[idx];

                        // Use only cached firewall status (no synchronous network call)
                        let firewall_status_str = platform
                            .cached_firewall_status
                            .clone()
                            .unwrap_or_else(|| "? Not checked".to_string());

                        // firewall_updated is true if IP is whitelisted (status starts with ✅)
                        let firewall_updated = firewall_status_str.starts_with("✅");

                        // Compute SSH status from cached results only (no blocking)
                        let project_id_key = platform
                            .gcp_selected_project_id
                            .as_ref()
                            .map(|s| s.as_str())
                            .unwrap_or("unknown");

                        // Load cached SSH result from config if no live test result (Task 6)
                        let ssh_test_result = self
                            .ssh_test_results
                            .get(project_id_key)
                            .cloned()
                            .or_else(|| {
                                // Check config cache
                                if let (Some(success), Some(message)) = (
                                    platform.cached_ssh_connection_success,
                                    &platform.cached_ssh_connection_message,
                                ) {
                                    Some(Ok(crate::calc::ssh::SshConnectionResult {
                                        success,
                                        message: message.clone(),
                                    }))
                                } else {
                                    None
                                }
                            });

                        let ssh_status_str = compute_ssh_status(platform, ssh_test_result.as_ref());

                        // ssh_ready should match ssh_status: only true if actually connected
                        let ssh_ready = if let Some(result) = &ssh_test_result {
                            matches!(result, Ok(conn_result) if conn_result.success)
                        } else {
                            false
                        };

                        // Store keyring domain only (no blocking I/O during rendering)
                        let ssh_keyring_domain =
                            platform.vms.first().and_then(|vm| vm.ssh_key_name.clone());

                        let row = PlatformRow {
                            // NEW: Use project_id as identifier
                            project_id: platform
                                .gcp_selected_project_id
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string()),
                            project_display_name: platform
                                .gcp_selected_project_id
                                .clone()
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

                            // Use cached project count (None = not yet loaded, shows "—")
                            total_project_count: platform.cached_total_project_count,

                            selected_project_id: platform.gcp_selected_project_id.clone(),
                            vm_name: platform.vms.first().map(|vm| vm.name.clone()),

                            // Use cached external IP if available, fall back to VM data
                            vm_external_ip: platform.cached_vm_external_ip.clone().or_else(|| {
                                platform.vms.first().and_then(|vm| vm.external_ip.clone())
                            }),

                            ssh_private_key: None, // Loaded on-demand in drawer
                            ssh_public_key: None,
                            ssh_keyring_domain,

                            // Use cached firewall status
                            firewall_status: platform
                                .cached_firewall_status
                                .clone()
                                .unwrap_or_else(|| firewall_status_str),

                            ssh_status: ssh_status_str,

                            // NEW: Cache metadata
                            last_refresh_time: platform.last_status_refresh,

                            // Operation state tracking
                            operation_state: OperationState::Idle,

                            // Action button state
                            has_vm: !platform.vms.is_empty(),
                            vm_zone: platform.vms.first().map(|vm| vm.zone.clone()),
                            oauth_refresh_failed,

                            // DNS wizard domain binding
                            bound_domain: platform.bound_domain.clone(),
                        };

                        self.rows.push(row);

                        // Trigger SSH connection test in background if VM has external IP
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            let project_id = platform
                                .gcp_selected_project_id
                                .clone()
                                .unwrap_or_else(|| "unknown".to_string());
                            if platform
                                .vms
                                .first()
                                .and_then(|vm| vm.external_ip.as_ref())
                                .is_some()
                                && !self.ssh_test_results.contains_key(&project_id)
                                && !self.ssh_test_promises.contains_key(&project_id)
                            {
                                self.execute_test_connection(
                                    current_profile.as_ref().unwrap(),
                                    current_profile_kdbx,
                                    project_id,
                                );
                            }
                        }
                    }

                    // Auto-create SSH entries for VMs not in SSH hosts
                    let mut new_ssh_hosts = Vec::new();
                    for platform in &app_config.platforms {
                        if platform.platform_type != "gcp" {
                            continue;
                        }

                        let platform_name = platform
                            .gcp_selected_project_id
                            .clone()
                            .unwrap_or_else(|| "unknown".to_string());

                        for vm in &platform.vms {
                            if let Some(external_ip) = &vm.external_ip {
                                // Check if SSH host already exists with this IP
                                let exists = app_config.ssh_hosts.iter().any(|ssh_host| {
                                    // Parse host to extract IP (format: "root@IP" or "user@IP" or just "IP")
                                    let host_ip =
                                        ssh_host.host.split('@').last().unwrap_or(&ssh_host.host);
                                    host_ip == external_ip
                                });

                                if !exists {
                                    new_ssh_hosts.push(crate::config::SshHostConfig {
                                        host: format!("root@{}", external_ip),
                                        password: None,
                                        private_key_path: None,
                                        keyring_domain: vm.ssh_key_name.clone(),
                                        port: 22,
                                        initialized: false,
                                        last_status: None,
                                        platform_id: Some(platform_name.clone()),
                                        docker_containers: Vec::new(),
                                        ..Default::default()
                                    });
                                }
                            }
                        }
                    }

                    // Add new SSH hosts to config and save
                    if !new_ssh_hosts.is_empty() {
                        app_config.ssh_hosts.extend(new_ssh_hosts.clone());
                        if let Err(e) = app_config.save(&config_path) {
                            dure_error!("Failed to save SSH hosts: {}", e);
                        } else {
                            // Show notification
                            self.show_ssh_creation_notification = true;
                            self.ssh_created_count = new_ssh_hosts.len();
                            self.ssh_creation_notification_time = Some(std::time::Instant::now());
                        }
                    }

                    // Cache config file metadata
                    if let Ok(metadata) = std::fs::metadata(&config_path) {
                        if let Ok(modified) = metadata.modified() {
                            self.config_last_modified = Some(modified);
                        }
                    }

                    // Mark as loaded immediately - background checks will update rows
                    self.loaded = true;
                }
                Err(e) => {
                    self.error_dialog.open(format!("Failed to load config: {}", e));
                    self.loaded = true; // Still mark as loaded to show error
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.error_dialog.open("WASM platform not supported".to_string());
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
            _ => "Platform configured".to_string(),
        }
    }

    /// Fetch GCP account summary (billing accounts, projects, VMs)

    fn handle_add_platform(
        &mut self,
        current_profile: &crate::calc::profile::ProfileContext,
        platform_type: String,
        project_id: String,
        oauth_result: crate::api::gcp::oauth::OAuthResult,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Some(vm) = vm {
                let config_path = current_profile.config_file.clone();

                if self.add_platform_dialog.reauth_mode {
                    // Re-auth mode: update existing platform OAuth
                    match vm.update_platform_oauth(
                        config_path,
                        project_id,
                        oauth_result.access_token,
                        oauth_result.refresh_token,
                        oauth_result.expires_at as i64,
                        self.add_platform_dialog.connected_email.clone().unwrap_or_default(),
                    ) {
                        Ok(_) => {
                            dure_info!("Platform OAuth update command sent");
                        }
                        Err(e) => {
                            self.error_dialog.open(format!("Failed to update platform OAuth: {}", e));
                        }
                    }
                } else {
                    // Add mode: add new platform
                    match vm.add_platform(
                        config_path,
                        platform_type,
                        Some(oauth_result.access_token),
                        Some(oauth_result.refresh_token),
                        Some(oauth_result.expires_at as i64),
                        self.add_platform_dialog.connected_email.clone(),
                        Some(project_id),
                    ) {
                        Ok(_) => {
                            dure_info!("Platform add command sent");
                        }
                        Err(e) => {
                            self.error_dialog.open(format!("Failed to add platform: {}", e));
                        }
                    }
                }
            }
        }
    }


    /// Render add platform wizard dialog
    fn render_add_platform_wizard(
        &mut self,
        current_profile: &crate::calc::profile::ProfileContext,
        ctx: &egui::Context,
        mut vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        let mut open = self.wizard_dialog_open;
        let window_width = ctx.viewport_rect().width();
        let window_height = ctx.viewport_rect().height();

        dialog(
            "add_platform_wizard",
            &tr!("platform-wizard-title"),
            &mut open,
        )
        .min_width(500.0)
        .max_width(window_width - 40.0)
        .max_height(window_height - 40.0)
        .scrollable(true)
        .content(|ui, open| {
            // Step indicator
            ui.add_space(10.0);
            self.render_wizard_step_indicator(ui);
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Step content
            match self.wizard_current_step {
                0 => self.render_wizard_oauth_step(ui, vm.as_deref_mut()),
                1 => self.render_wizard_project_step(ui),
                2 => self.render_wizard_confirm_step(ui),
                3 => self.render_wizard_success_step(ui),
                _ => {}
            }

            // Navigation buttons
            ui.add_space(20.0);
            ui.separator();
            ui.add_space(10.0);

            self.render_wizard_navigation(ui, open, current_profile, vm);
        })
        .show(ctx);

        self.wizard_dialog_open = open;
    }

    /// Render wizard step indicator (progress bar)
    fn render_wizard_step_indicator(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let steps = [
                ("1. OAuth", 0),
                ("2. Project", 1),
                ("3. Confirm", 2),
                ("4. Done", 3),
            ];

            for (i, (label, step_num)) in steps.iter().enumerate() {
                let is_current = self.wizard_current_step == *step_num;
                let is_complete = self.wizard_current_step > *step_num;

                if is_current {
                    ui.add(MaterialButton::filled(*label).small());
                } else if is_complete {
                    ui.add(MaterialButton::outlined(*label).small());
                } else {
                    ui.add(MaterialButton::outlined(*label).small().enabled(false));
                }

                if i < steps.len() - 1 {
                    ui.add_space(5.0);
                }
            }
        });
    }

    /// Step 0: OAuth connection
    fn render_wizard_oauth_step(
        &mut self,
        ui: &mut egui::Ui,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        ui.heading(tr!("platform-wizard-oauth-title"));
        ui.add_space(10.0);
        ui.label(tr!("platform-wizard-oauth-desc"));
        ui.add_space(15.0);

        // Check if OAuth is complete
        if let Some(email) = &self.add_platform_dialog.connected_email {
            ui.colored_label(
                egui_material3::theme::get_global_color("success"),
                format!("✓ Connected as: {}", email),
            );
        } else if let Some(promise) = &self.add_platform_dialog.oauth_promise {
            // Poll OAuth promise
            if let Some(result) = promise.ready() {
                match result {
                    Ok(oauth_result) => {
                        // Clone token before clearing promise (borrow checker)
                        let access_token = oauth_result.access_token.clone();

                        self.add_platform_dialog.oauth_result = Some(oauth_result.clone());
                        self.add_platform_dialog.connected_email = Some("Connected Account".to_string());
                        self.add_platform_dialog.oauth_promise = None;

                        // Fetch projects using OAuth token directly
                        if self.add_platform_dialog.project_list.is_empty() {
                            if let Some(ref vm) = vm {
                                let _ = vm.list_projects_with_token(
                                    access_token,
                                    "temp-platform".to_string(),
                                );
                            }
                        }
                    }
                    Err(e) => {
                        self.error_dialog.open(format!("OAuth failed: {}", e));
                        self.add_platform_dialog.oauth_promise = None;
                    }
                }
            } else {
                ui.spinner();
                ui.label(tr!("platform-add-oauth-waiting"));
            }
        } else {
            if ui
                .add(MaterialButton::filled(tr!("platform-add-oauth-connect")))
                .clicked()
            {
                self.start_add_platform_oauth();
            }
        }
    }

    /// Step 1: Project selection
    fn render_wizard_project_step(&mut self, ui: &mut egui::Ui) {
        ui.heading(tr!("platform-wizard-project-title"));
        ui.add_space(10.0);
        ui.label(tr!("platform-wizard-project-desc"));
        ui.add_space(15.0);

        // Reuse existing project selection logic
        ui.radio_value(
            &mut self.add_platform_dialog.create_new,
            false,
            tr!("platform-add-project-existing"),
        );

        if !self.add_platform_dialog.create_new {
            ui.add_space(5.0);
            for (idx, (project_id, project_name)) in
                self.add_platform_dialog.project_list.iter().enumerate()
            {
                ui.radio_value(
                    &mut self.add_platform_dialog.selected_project,
                    Some(idx),
                    format!("{} ({})", project_name, project_id),
                );
            }
        }

        ui.add_space(10.0);
        ui.radio_value(
            &mut self.add_platform_dialog.create_new,
            true,
            tr!("platform-add-project-new"),
        );

        if self.add_platform_dialog.create_new {
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label(tr!("platform-add-project-id"));
                ui.text_edit_singleline(&mut self.add_platform_dialog.new_project_id);
            });
        }
    }

    /// Step 2: Confirmation
    fn render_wizard_confirm_step(&self, ui: &mut egui::Ui) {
        ui.heading(tr!("platform-wizard-confirm-title"));
        ui.add_space(10.0);
        ui.label(tr!("platform-wizard-confirm-desc"));
        ui.add_space(15.0);

        ui.label(format!(
            "Email: {}",
            self.add_platform_dialog.connected_email
                .as_ref()
                .unwrap_or(&"Not connected".to_string())
        ));

        if self.add_platform_dialog.create_new {
            ui.label(format!(
                "Project: New - {}",
                if self.add_platform_dialog.new_project_id.is_empty() {
                    "Not specified"
                } else {
                    &self.add_platform_dialog.new_project_id
                }
            ));
        } else {
            let project_info = self
                .add_platform_dialog.selected_project
                .and_then(|idx| self.add_platform_dialog.project_list.get(idx))
                .map(|(id, name)| format!("{} ({})", name, id))
                .unwrap_or_else(|| "Not selected".to_string());
            ui.label(format!("Project: {}", project_info));
        }
    }

    /// Step 3: Success
    fn render_wizard_success_step(&mut self, ui: &mut egui::Ui) {
        ui.heading(tr!("platform-wizard-success-title"));
        ui.add_space(10.0);
        ui.label(tr!("platform-wizard-success-desc"));
        ui.add_space(15.0);

        ui.colored_label(
            egui_material3::theme::get_global_color("success"),
            "✓ Platform added successfully!",
        );
    }

    /// Render wizard navigation buttons
    fn render_wizard_navigation(
        &mut self,
        ui: &mut egui::Ui,
        open: &mut bool,
        current_profile: &crate::calc::profile::ProfileContext,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        ui.horizontal(|ui| {
            // Close button
            if ui.add(MaterialButton::outlined(tr!("cancel"))).clicked() {
                *open = false;
                self.reset_wizard_state();
            }

            ui.add_space(10.0);

            // Previous button (disabled on first step)
            let can_go_back = self.wizard_current_step > 0 && self.wizard_current_step < 3;
            if ui
                .add(MaterialButton::outlined(tr!("previous")).enabled(can_go_back))
                .clicked()
            {
                if self.wizard_current_step > 0 {
                    self.wizard_current_step -= 1;
                }
            }

            ui.add_space(10.0);

            // Next/Finish button
            if self.wizard_current_step < 3 {
                // Validate current step before allowing Next
                let can_proceed = self.can_proceed_wizard();

                if ui
                    .add(MaterialButton::filled(tr!("next")).enabled(can_proceed))
                    .clicked()
                {
                    // Step 2 → 3: Execute add platform command
                    if self.wizard_current_step == 2 {
                        // Extract data from dialog and call handle_add_platform
                        if let Some(oauth_result) = self.add_platform_dialog.oauth_result.clone() {
                            let project_id = if self.add_platform_dialog.create_new {
                                self.add_platform_dialog.new_project_id.clone()
                            } else {
                                self.add_platform_dialog.selected_project
                                    .and_then(|idx| self.add_platform_dialog.project_list.get(idx))
                                    .map(|(id, _)| id.clone())
                                    .unwrap_or_default()
                            };
                            self.handle_add_platform(
                                current_profile,
                                self.add_platform_dialog.platform_type.clone(),
                                project_id,
                                oauth_result,
                                vm,
                            );
                        }
                        // Force reload on next frame
                        self.loaded = false;
                    }
                    self.wizard_current_step += 1;
                }
            } else {
                // Final step - show Finish button
                if ui.add(MaterialButton::filled(tr!("finish"))).clicked() {
                    *open = false;
                    self.reset_wizard_state();
                }
            }
        });
    }

    /// Check if wizard can proceed to next step
    fn can_proceed_wizard(&self) -> bool {
        match self.wizard_current_step {
            0 => self.add_platform_dialog.connected_email.is_some(), // OAuth connected
            1 => {
                // Project selected
                if self.add_platform_dialog.create_new {
                    !self.add_platform_dialog.new_project_id.is_empty()
                } else {
                    self.add_platform_dialog.selected_project.is_some()
                }
            }
            2 => true, // Confirm step - always allow
            _ => false,
        }
    }

    /// Reset wizard state
    fn reset_wizard_state(&mut self) {
        self.wizard_dialog_open = false;
        self.wizard_current_step = 0;
        self.add_platform_dialog.close();
    }

    fn render_init_progress(&mut self, ui: &mut egui::Ui) {
        ui.add_space(12.0);
        ui.separator();
        ui.heading(tr!("platform-progress-heading"));

        if let Some(name) = &self.init_platform_name {
            ui.label(tr!("platform-progress-platform", {name: name.clone()}));
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

    #[cfg(not(target_arch = "wasm32"))]
    fn show_gcp_wizard(
        &mut self,
        profile: &crate::calc::profile::ProfileContext,
        kdbx_handle: &Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
        platform_name: String,
    ) {
        // Try to load config and find platform with OAuth + project
        let mut wizard = if let Ok((app_config, _)) = load_config(&Some(profile.clone())) {
            // Find platform by name
            if let Some(platform) = app_config
                .platforms
                .iter()
                .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
            {
                // Check if platform has OAuth tokens and project ID
                if let (
                    Some(access_token),
                    Some(refresh_token),
                    Some(token_expiry),
                    Some(project_id),
                ) = (
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
                        platform_name.clone(),
                        project_id.clone(),
                        oauth_result,
                    )
                } else {
                    // Missing OAuth or project, use full wizard
                    GcpWizard::new(platform_name.clone())
                }
            } else {
                // Platform not found in config, use full wizard
                GcpWizard::new(platform_name.clone())
            }
        } else {
            // Config load failed, use full wizard
            GcpWizard::new(platform_name)
        };

        // Set profile keyring handle before showing wizard
        if let Some(handle) = kdbx_handle {
            wizard.set_profile_keyring(handle.clone());
        }

        wizard.show();
        self.gcp_wizard = Some(wizard);
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn restart_vm(
        &mut self,
        profile: &crate::calc::profile::ProfileContext,
        platform_name: String,
        vm_name: String,
        zone: String,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        // ViewModel-based implementation
        if let Some(vm) = vm {
            // Send command to ViewModel with profile-specific config path
            let profile_config_path = profile.config_file.clone();
            if let Err(e) = vm.restart_vm(profile_config_path, platform_name, vm_name, zone) {
                self.error_dialog.open(format!("Failed to start VM restart: {}", e));
            }
            // Note: UI will be updated by event processing when VMRestarted event arrives
        } else {
            // Fallback: no ViewModel available
            self.error_dialog.open("ViewModel not available".to_string());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn regenerate_vm(
        &mut self,
        profile: &crate::calc::profile::ProfileContext,
        platform_name: String,
        vm_name: String,
        profile_kdbx: Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        if let Some(vm) = vm {
            // Get zone from config
            let zone = match load_config(&Some(profile.clone())) {
                Ok((app_config, _)) => {
                    if let Some(platform) = app_config
                        .platforms
                        .iter()
                        .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
                    {
                        if let Some(vm_cfg) = platform.vms.iter().find(|v| v.name == vm_name) {
                            vm_cfg.zone.clone()
                        } else {
                            self.error_dialog.open(format!("VM not found: {}", vm_name));
                            return;
                        }
                    } else {
                        self.error_dialog.open(format!("Platform not found: {}", platform_name));
                        return;
                    }
                }
                Err(e) => {
                    self.error_dialog.open(format!("Failed to load config: {}", e));
                    return;
                }
            };

            let profile_config_path = profile.config_file.clone();
            if let Err(e) = vm.regenerate_vm(
                profile_config_path,
                platform_name.clone(),
                vm_name.clone(),
                zone,
                profile_kdbx,
            ) {
                self.error_dialog.open(format!("Failed to start VM regeneration: {}", e));
            }
            // Result will be delivered via VMRegenerated event
        } else {
            self.error_dialog.open("ViewModel not available".to_string());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn update_firewall(
        &mut self,
        current_profile: &crate::calc::profile::ProfileContext,
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
                }
                Err(e) => {
                    dure_error!("Failed to get current IP: {}", e);
                    self.error_dialog.open(format!("Failed to get current IP: {}", e));
                    return;
                }
            };

            // Send command to ViewModel
            let config_path = get_config_path(&Some(current_profile.clone())).unwrap_or_default();
            if let Err(e) = vm.update_firewall(config_path, platform_name.clone(), current_ip.clone()) {
                dure_error!("Failed to send firewall update command: {}", e);
                self.error_dialog.open(format!("Failed to start firewall update: {}", e));
            } else {
                dure_info!(" Firewall update command sent successfully");
            }
            // Note: UI will be updated by event processing when FirewallUpdated event arrives
        } else {
            // Fallback: no ViewModel available
            dure_error!("ViewModel not available for firewall update");
            self.error_dialog.open("ViewModel not available".to_string());
        }
    }

    fn scan_vms(
        &mut self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        // ViewModel-based implementation
        if let Some(vm) = vm {
            dure_info!(" Scanning VMs for platform '{}'...", platform_name);

            // Send command to ViewModel with profile-specific config path
            if let Err(e) = vm.scan_existing_vms(profile_config_path, platform_name) {
                self.error_dialog.open(format!("Failed to start VM scan: {}", e));
            }
            // Note: UI will be updated when VMsScanned event arrives
        } else {
            // Fallback: no ViewModel available
            self.error_dialog.open("ViewModel not available".to_string());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn show_select_project_dialog(
        &mut self,
        current_profile: &crate::calc::profile::ProfileContext,
        platform_name: String,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        self.select_project_dialog.platform_name = platform_name.clone();
        self.select_project_dialog.project_list.clear();
        self.select_project_dialog.selected = None;
        self.select_project_dialog.loading = true;

        // Trigger ViewModel to fetch projects
        if let Some(vm) = vm {
            let config_path = get_config_path(&Some(current_profile.clone())).unwrap_or_default();
            if let Err(e) = vm.list_projects(config_path, platform_name.clone()) {
                self.error_dialog.open(format!("Failed to start project listing: {}", e));
                self.select_project_dialog.loading = false;
            }
            // Dialog will be shown when ProjectsListed event arrives
        } else {
            // Fallback: No ViewModel available, show error
            self.error_dialog.open("ViewModel not available".to_string());
            self.select_project_dialog.loading = false;
        }
    }

    fn show_delete_vm_confirmation(
        &mut self,
        profile: &Option<crate::calc::profile::ProfileContext>,
        vm: Option<&mut crate::viewmodel::ViewModel>,
        platform_name: String,
        vm_name: String,
        zone: String,
    ) {
        // Open dialog with single VM pre-selected and go directly to confirmation step
        self.delete_vm_dialog.open(
            platform_name.clone(),
            vec![(vm_name.clone(), zone.clone(), "".to_string())],
        );
        self.delete_vm_dialog.selected = Some(0);
        self.delete_vm_dialog.confirming = true;

        // Check if VM has static IP (requires access token and config)
        #[cfg(not(target_arch = "wasm32"))]
        {
            dure_info!("=== Static IP Detection START for VM: {} ===", vm_name);
            if let Ok((app_config, _)) = load_config(profile) {
                dure_info!("✓ Config loaded");
                if let Some(platform) = app_config
                    .platforms
                    .iter()
                    .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
                {
                    dure_info!("✓ Platform found: {}", platform_name);
                    if let Some(token) = platform.gcp_oauth_access_token.as_ref() {
                        dure_info!("✓ OAuth token exists");
                        if let Some(vm_config) = platform
                            .vms
                            .iter()
                            .find(|v| v.name == vm_name && v.zone == zone)
                        {
                            dure_info!("✓ VM found: {}", vm_name);
                            if let Some(external_ip) = &vm_config.external_ip {
                                dure_info!("✓ VM has external_ip: {}", external_ip);
                                // Trigger async static IP check via actor
                                if let Some(vm) = vm {
                                    let config_path = get_config_path(profile).unwrap_or_default();
                                    let _ = vm.check_vm_static_ip(
                                        config_path,
                                        platform_name.clone(),
                                        vm_name.clone(),
                                        zone.clone(),
                                        external_ip.clone(),
                                    );
                                    dure_info!("✓ Static IP check triggered via actor");
                                }
                            } else {
                                dure_info!("✗ VM has no external_ip field");
                            }
                        } else {
                            dure_info!("✗ VM not found in config");
                        }
                    } else {
                        dure_info!("✗ No OAuth token");
                    }
                } else {
                    dure_info!("✗ Platform not found");
                }
            } else {
                dure_info!("✗ Config load failed");
            }
            dure_info!(
                "=== Static IP Detection END (has_static_ip={}) ===",
                self.delete_vm_dialog.has_static_ip
            );
        }
    }


    #[cfg(not(target_arch = "wasm32"))]
    fn handle_delete_vm(
        &mut self,
        profile: &crate::calc::profile::ProfileContext,
        instance_name: String,
        zone: String,
        force: bool,
        release_ip: Option<(String, String)>,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        if let Some(vm) = vm {
            let profile_config_path = profile.config_file.clone();
            let platform = self.delete_vm_dialog.platform.clone();

            if let Err(e) = vm.delete_vm(
                profile_config_path,
                platform.clone(),
                instance_name.clone(),
                zone.clone(),
                force,
                release_ip,
            ) {
                self.error_dialog.open(format!("Failed to start VM deletion: {}", e));
                return;
            }

            // Record audit event
            if let Ok((app_config, _)) = load_config(&Some(profile.clone())) {
                if let Some(plat) = app_config
                    .platforms
                    .iter()
                    .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform))
                {
                    if let Some(vm_config) = plat.vms.iter().find(|v| v.name == instance_name) {
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
        } else {
            self.error_dialog.open("ViewModel not available".to_string());
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
    fn execute_test_connection(
        &mut self,
        profile: &crate::calc::profile::ProfileContext,
        kdbx_handle: &Option<std::sync::Arc<crate::calc::keyring::DatabaseHandle>>,
        platform_name: String,
    ) {
        // Load config and find the VM for this platform (platform_name is actually project_id)
        let (vm_host, keyring_domain) = match load_config(&Some(profile.clone())) {
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

        // Load SSH private key from profile keyring before spawning thread
        let (private_key, _public_key) =
            load_ssh_key_from_keyring(&platform_name, &keyring_domain, kdbx_handle);

        let Some(private_key_pem) = private_key else {
            self.ssh_test_results.insert(
                platform_name,
                Err("SSH key not found in profile keyring".to_string()),
            );
            return;
        };

        // Parse IP from host (format: "root@IP")
        let ip = host.split('@').last().unwrap_or(&host).to_string();

        // Spawn connection test in background thread
        let promise = poll_promise::Promise::spawn_thread("ssh_test_platform", move || {
            use crate::calc::ssh;
            smol::block_on(async {
                ssh::test_connection_simple(&ip, &private_key_pem, 22, 10000)
                    .await
                    .map(|_| crate::calc::ssh::SshConnectionResult {
                        success: true,
                        message: format!("Successfully connected to {}", host),
                    })
                    .map_err(|e| format!("{}", e))
            })
        });

        self.ssh_test_promises.insert(platform_name, promise);
    }

    /// Get valid access token, refreshing if expired (standalone helper)
    fn refresh_or_get_token(
        platform: &mut crate::config::CloudPlatformConfig,
    ) -> Result<String, String> {
        use crate::api::gcp::oauth::refresh_access_token;

        let token = platform
            .gcp_oauth_access_token
            .as_ref()
            .ok_or_else(|| "No OAuth access token".to_string())?;

        let refresh_token = platform
            .gcp_oauth_refresh_token
            .as_ref()
            .ok_or_else(|| "No OAuth refresh token".to_string())?;

        let expiry = platform
            .gcp_oauth_token_expiry
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
            )
            .map_err(|e| format!("Failed to refresh token: {}", e))?;

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
    fn execute_refresh(
        &mut self,
        profile: &crate::calc::profile::ProfileContext,
        project_id: String,
    ) {
        use crate::api::gcp::{get_current_ip, GcpRestClient};

        let project_id_clone = project_id.clone();
        let profile_clone = profile.clone();
        let promise = poll_promise::Promise::spawn_thread("refresh_status", move || {
            // Load config
            let (mut config, config_path) = load_config(&Some(profile_clone))
                .map_err(|e| format!("Failed to load config: {}", e))?;

            // Find platform
            let platform = config
                .platforms
                .iter_mut()
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
                            platform.cached_firewall_status = Some(if whitelisted {
                                format!("✅ Whitelisted ({})", current_ip)
                            } else {
                                "❌ Not whitelisted".to_string()
                            });
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
            config
                .save(&config_path)
                .map_err(|e| format!("Failed to save config: {}", e))?;

            Ok(())
        });

        self.refresh_promises.insert(project_id, promise);
    }

    #[cfg(target_arch = "wasm32")]
    fn execute_refresh(&mut self, _project_id: String) {
        // WASM not supported
    }

    fn show_delete_platform_confirmation(
        &mut self,
        profile: &crate::calc::profile::ProfileContext,
        platform_name: String,
    ) {
        // Count VMs for this platform
        let vm_count = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                if let Ok((app_config, _)) = load_config(&Some(profile.clone())) {
                    if let Some(platform) = app_config
                        .platforms
                        .iter()
                        .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
                    {
                        platform.vms.len()
                    } else {
                        0
                    }
                } else {
                    0
                }
            }
            #[cfg(target_arch = "wasm32")]
            {
                0
            }
        };

        self.delete_platform_confirm_dialog.open(platform_name, vm_count);
    }


    #[cfg(not(target_arch = "wasm32"))]

    #[cfg(not(target_arch = "wasm32"))]
    fn handle_select_project(
        &mut self,
        current_profile: &crate::calc::profile::ProfileContext,
        platform_name: String,
        project_id: String,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        if let Some(vm) = vm {
            let config_path = current_profile.config_file.clone();
            if let Err(e) = vm.select_project(config_path, platform_name, project_id) {
                self.error_dialog.open(format!("Failed to select project: {}", e));
            }
            // Result will be delivered via ProjectSelected event
        } else {
            self.error_dialog.open("ViewModel not available".to_string());
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn handle_delete_platform(
        &mut self,
        current_profile: &crate::calc::profile::ProfileContext,
        platform_name: String,
        delete_vms: bool,
        delete_project: bool,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        if let Some(vm) = vm {
            let config_path = current_profile.config_file.clone();
            let delete_options = crate::viewmodel::platform::DeleteOptions {
                delete_vms,
                delete_project,
            };
            match vm.delete_platform(config_path, platform_name, delete_options) {
                Ok(_) => {
                    dure_info!(" Platform delete command sent");
                }
                Err(e) => {
                    self.error_dialog.open(format!("Failed to delete platform: {}", e));
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

        self.add_platform_dialog.oauth_promise =
            Some(Promise::spawn_thread("gcp_oauth_add_platform", move || {
                handler.run_oauth_flow().map_err(|e| e.to_string())
            }));
    }

    fn fetch_billing_data(
        &mut self,
        profile: &crate::calc::profile::ProfileContext,
        vm: Option<&mut crate::viewmodel::ViewModel>,
        project_id_param: Option<String>,
    ) {
        dure_debug!(
            "[BILLING] fetch_billing_data called, project_id_param={:?}",
            project_id_param
        );

        // ViewModel-based implementation
        if let Some(vm) = vm {
            dure_debug!("[BILLING] Setting billing_loading = true");
            self.billing_dialog.loading = true;
            self.billing_dialog.error = None;
            self.billing_dialog.data = None;

            // Load config to get GCP platform with OAuth
            dure_debug!("[BILLING] Loading config...");
            let (mut app_config, config_path) = match load_config(&Some(profile.clone())) {
                Ok(config) => {
                    dure_debug!("[BILLING] Config loaded successfully");
                    config
                }
                Err(e) => {
                    dure_error!("[BILLING] Failed to load config: {}", e);
                    self.billing_dialog.error = Some(format!("Failed to load config: {}", e));
                    self.billing_dialog.loading = false;
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
                    self.billing_dialog.error = Some(
                        "No connected GCP platform found. Please add a GCP platform first."
                            .to_string(),
                    );
                    self.billing_dialog.loading = false;
                    return;
                }
            };

            // Get valid (possibly refreshed) access token
            let access_token =
                match self.get_valid_access_token(&mut app_config, platform_idx, &config_path) {
                    Ok(token) => token,
                    Err(e) => {
                        self.billing_dialog.error =
                            Some(format!("Failed to get valid OAuth token: {}", e));
                        self.billing_dialog.loading = false;
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
                self.billing_dialog.error =
                    Some("No project ID available. Please select a project first.".to_string());
                self.billing_dialog.loading = false;
                return;
            };

            // Auto-discover billing table if not configured
            if self.billing_dialog.dataset.is_empty() || self.billing_dialog.table.is_empty() {
                dure_info!(
                    "[BILLING] Auto-discovering billing table for project: {}",
                    project_id
                );
                self.billing_dialog.project_id = project_id.clone();

                // Trigger async discovery via actor
                let config_path = get_config_path(&Some(profile.clone())).unwrap_or_default();
                if let Err(e) = vm.discover_billing_table(config_path, project_id.clone(), project_id.clone()) {
                    dure_error!("[BILLING] Failed to start billing discovery: {}", e);
                    self.billing_dialog.error = Some(format!("Failed to start discovery: {}", e));
                    self.billing_dialog.loading = false;
                    return;
                }

                // Discovery will trigger BillingTableDiscovered event which will auto-fetch
                dure_info!("[BILLING] Waiting for billing table discovery to complete...");
                return;
            }

            // Dataset and table are already configured, fetch immediately
            dure_debug!(
                "[BILLING] Using existing dataset={}, table={}",
                self.billing_dialog.dataset,
                self.billing_dialog.table
            );

            if self.billing_dialog.project_id.is_empty() {
                self.billing_dialog.project_id = project_id.clone();
            }

            // Send command to ViewModel
            dure_info!(
                "[BILLING] Sending fetch_billing command: project={}, dataset={}, table={}",
                project_id,
                self.billing_dialog.dataset,
                self.billing_dialog.table
            );
            if let Err(e) = vm.fetch_billing(
                profile.config_file.clone(),
                project_id.clone(), // Use project_id as platform identifier
                project_id,
                self.billing_dialog.dataset.clone(),
                self.billing_dialog.table.clone(),
            ) {
                dure_error!("[BILLING] Failed to send fetch_billing command: {}", e);
                self.billing_dialog.error = Some(format!("Failed to start billing fetch: {}", e));
                self.billing_dialog.loading = false;
            } else {
                dure_info!("[BILLING] fetch_billing command sent successfully");
            }
            // Note: billing_data and billing_loading will be updated by BillingFetched or Error event handlers
        } else {
            dure_warn!("[BILLING] No ViewModel available");
            // Fallback: no ViewModel available (shouldn't happen in normal operation)
            self.billing_dialog.error = Some("ViewModel not available".to_string());
            self.billing_dialog.loading = false;
        }
    }

    fn handle_billing_refresh(
        &mut self,
        profile: &crate::calc::profile::ProfileContext,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        self.fetch_billing_data(profile, vm, None);
    }

    fn fetch_vm_list_for_binding(
        &mut self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        if let Some(vm) = vm {
            if let Err(e) = vm.list_vms(profile_config_path, platform_name) {
                dure_error!("Failed to list VMs for binding: {}", e);
                self.bind_vm_dialog.loading = false;
                self.bind_vm_dialog.vm_list = Vec::new();
            }
            // Results will come via ViewModelEvent::VMsListed in update_from_viewmodel
            // bind_vm_dialog.loading stays true until event arrives
        } else {
            self.bind_vm_dialog.loading = false;
            self.bind_vm_dialog.vm_list = Vec::new();
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn handle_bind_vm(
        &mut self,
        current_profile: &crate::calc::profile::ProfileContext,
        platform: String,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        if let Some(vm_ref) = vm {
            let profile_config_path = current_profile.config_file.clone();
            if let Err(e) = vm_ref.scan_existing_vms(profile_config_path, platform) {
                self.error_dialog.open(format!("Failed to bind VMs: {}", e));
            }
        }
    }


    fn fetch_firewall_data(
        &mut self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        dure_info!(
            "[UI::PLATFO] fetch_firewall_data called for platform: {}",
            platform_name
        );
        if let Some(vm) = vm {
            if let Err(e) = vm.fetch_firewall_rules(profile_config_path, platform_name.clone()) {
                dure_error!("[UI::PLATFO] Failed to fetch firewall rules: {}", e);
                self.firewall_dialog.current_ip = None;
                self.firewall_dialog.rule_list = Vec::new();
            } else {
                dure_info!("[UI::PLATFO] Firewall rules fetch command sent, awaiting FirewallRulesFetched event");
            }
            // Results will come via ViewModelEvent::FirewallRulesFetched
        } else {
            dure_info!("[UI::PLATFO] No VM available, clearing firewall data");
            self.firewall_dialog.current_ip = None;
            self.firewall_dialog.rule_list = Vec::new();
        }
    }

    /// Refresh firewall rules with debounce (prevents duplicate refreshes within 1 second)
    fn refresh_firewall_list_if_needed(
        &mut self,
        profile_config_path: std::path::PathBuf,
        platform_name: String,
        vm: Option<&mut crate::viewmodel::ViewModel>,
        force: bool, // If true, skip debounce check
    ) {
        // Debounce: skip if refreshed within last 1 second (unless force=true)
        if !force {
            if let Some(last_refresh) = self.firewall_dialog.last_refresh_time {
                if last_refresh.elapsed() < std::time::Duration::from_secs(1) {
                    dure_info!("[UI::PLATFO] Skipping duplicate refresh (debounced)");
                    return;
                }
            }
        }

        dure_info!(
            "[UI::PLATFO] Refreshing firewall rules for {}",
            platform_name
        );
        self.firewall_dialog.last_refresh_time = Some(std::time::Instant::now());
        self.fetch_firewall_data(profile_config_path, platform_name, vm);
    }


    fn handle_add_firewall_save(
        &mut self,
        current_profile: &crate::calc::profile::ProfileContext,
        rule_data: FirewallRule,
        edit_idx: Option<usize>,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        use crate::{dure_error, dure_info};

        self.add_firewall_dialog.save_error = None;
        self.add_firewall_dialog.save_in_progress = true;

        if let Some(idx) = edit_idx {
            dure_info!("Update firewall rule at index {}: {}", idx, rule_data.name);
            if let Some(vm_ref) = vm.as_ref() {
                // Get original rule name for deletion
                if let Some(old_rule) = self.firewall_dialog.rule_list.get(idx) {
                    let old_rule_name = old_rule.name.clone();
                    // Delete old rule first
                    if let Err(e) = vm_ref.delete_firewall_rule(
                        current_profile.config_file.clone(),
                        self.add_firewall_dialog.platform.clone(),
                        old_rule_name.clone(),
                    ) {
                        dure_error!(
                            "Failed to delete old firewall rule '{}': {}",
                            old_rule_name,
                            e
                        );
                    } else {
                        // Create new rule with updated data
                        let rule = crate::viewmodel::platform::FirewallRuleData {
                            name: rule_data.name.clone(),
                            network: rule_data.network.clone(),
                            direction: rule_data.direction.clone(),
                            priority: rule_data.priority,
                            source_ranges: rule_data.source_ranges.clone(),
                            dest_ranges: rule_data.dest_ranges.clone(),
                            allowed: rule_data.allowed.clone(),
                            denied: rule_data.denied.clone(),
                            source_tags: rule_data.source_tags.clone(),
                            source_service_accounts: rule_data.source_service_accounts.clone(),
                            target_tags: rule_data.target_tags.clone(),
                            target_service_accounts: rule_data.target_service_accounts.clone(),
                        };

                        if let Err(e) = vm_ref.create_firewall_rule(
                            current_profile.config_file.clone(),
                            self.add_firewall_dialog.platform.clone(),
                            rule,
                        ) {
                            dure_error!("Failed to recreate firewall rule: {}", e);
                            self.add_firewall_dialog.save_error =
                                Some(format!("Failed to update rule: {}", e));
                            self.add_firewall_dialog.save_in_progress = false;
                        }
                    }
                }
            }
        } else {
            dure_info!("Create new firewall rule: {}", rule_data.name);
            if let Some(vm_ref) = vm.as_ref() {
                let rule = crate::viewmodel::platform::FirewallRuleData {
                    name: rule_data.name.clone(),
                    network: rule_data.network.clone(),
                    direction: rule_data.direction.clone(),
                    priority: rule_data.priority,
                    source_ranges: rule_data.source_ranges.clone(),
                    dest_ranges: rule_data.dest_ranges.clone(),
                    allowed: rule_data.allowed.clone(),
                    denied: rule_data.denied.clone(),
                    source_tags: rule_data.source_tags.clone(),
                    source_service_accounts: rule_data.source_service_accounts.clone(),
                    target_tags: rule_data.target_tags.clone(),
                    target_service_accounts: rule_data.target_service_accounts.clone(),
                };

                if let Err(e) = vm_ref.create_firewall_rule(
                    current_profile.config_file.clone(),
                    self.add_firewall_dialog.platform.clone(),
                    rule,
                ) {
                    dure_error!("Failed to create firewall rule: {}", e);
                    self.add_firewall_dialog.save_error =
                        Some(format!("Failed to create rule: {}", e));
                    self.add_firewall_dialog.save_in_progress = false;
                }
            }
        }
    }

    fn handle_firewall_add_ip(
        &mut self,
        current_profile: &crate::calc::profile::ProfileContext,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        use crate::dure_error;

        if let (Some(vm_ref), Some(ip)) = (vm.as_ref(), &self.firewall_dialog.current_ip) {
            if let Some(selected_idx) = self.firewall_dialog.selected {
                if let Some(rule) = self.firewall_dialog.rule_list.get(selected_idx) {
                    if let Err(e) = vm_ref.update_firewall_rule(
                        current_profile.config_file.clone(),
                        self.firewall_dialog.platform.clone(),
                        rule.name.clone(),
                        ip.clone(),
                    ) {
                        dure_error!("Failed to add IP to firewall rule: {}", e);
                    }
                }
            }
        }
    }

    fn handle_firewall_add_rule(&mut self) {
        use crate::dure_info;

        dure_info!("Add Row button clicked - opening Add Firewall Rule dialog");
        self.add_firewall_dialog.form_data = FirewallRule::default();
        self.add_firewall_dialog.edit_idx = None;
        self.add_firewall_dialog.platform = self.firewall_dialog.platform.clone();
        self.add_firewall_dialog.open = true;
    }

    fn handle_firewall_edit_rule(
        &mut self,
        rule_data: FirewallRule,
        rule_idx: usize,
    ) {
        use crate::dure_info;

        dure_info!("Edit firewall rule at index {}", rule_idx);
        self.add_firewall_dialog.form_data = rule_data;
        self.add_firewall_dialog.edit_idx = Some(rule_idx);
        self.add_firewall_dialog.platform = self.firewall_dialog.platform.clone();
        self.add_firewall_dialog.open = true;
    }

    fn handle_firewall_delete_rule(
        &mut self,
        rule_name: String,
        rule_idx: Option<usize>,
    ) {
        use crate::dure_info;

        dure_info!(
            "Delete button clicked for rule '{}' - showing confirmation",
            rule_name
        );
        self.delete_firewall_confirm_dialog.open(
            rule_name,
            self.firewall_dialog.platform.clone(),
            rule_idx,
        );
    }

    fn handle_firewall_reset(
        &mut self,
        current_profile: &crate::calc::profile::ProfileContext,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        use crate::dure_info;

        dure_info!("Reset firewall rules from GCP (forced refresh)");
        self.refresh_firewall_list_if_needed(
            current_profile.config_file.clone(),
            self.firewall_dialog.platform.clone(),
            vm,
            true, // Force refresh, skip debounce
        );
    }

    fn handle_delete_firewall_rule(
        &mut self,
        current_profile: &crate::calc::profile::ProfileContext,
        rule_name: String,
        platform: String,
        _rule_idx: Option<usize>,
        vm: Option<&mut crate::viewmodel::ViewModel>,
    ) {
        dure_info!("Delete confirmed for firewall rule: {}", rule_name);
        if let Some(vm_ref) = vm.as_ref() {
            if let Err(e) = vm_ref.delete_firewall_rule(
                current_profile.config_file.clone(),
                platform,
                rule_name.clone(),
            ) {
                dure_error!("Failed to delete firewall rule '{}': {}", rule_name, e);
            }
        }
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

    #[test]
    fn test_format_project_count_display_with_none() {
        // When project count is not yet loaded, should show "—"
        assert_eq!(format_project_count_display(None), "—");
    }

    #[test]
    fn test_format_project_count_display_with_zero() {
        // When project count is 0, should show "0 projects"
        assert_eq!(format_project_count_display(Some(0)), "0 projects");
    }

    #[test]
    fn test_format_project_count_display_with_one() {
        // When project count is 1, should show "1 project" (singular)
        assert_eq!(format_project_count_display(Some(1)), "1 project");
    }

    #[test]
    fn test_format_project_count_display_with_many() {
        // When project count is >1, should show "N projects" (plural)
        assert_eq!(format_project_count_display(Some(5)), "5 projects");
    }

    #[test]
    fn test_auto_refresh_tracking() {
        // Verify auto-refresh tracking prevents duplicate refreshes
        let mut tab = PlatformTab::default();
        let project_id = "test-project".to_string();

        // Initially empty
        assert!(!tab.auto_refreshed_platforms.contains(&project_id));

        // Mark as auto-refreshed
        tab.auto_refreshed_platforms.insert(project_id.clone());

        // Should now contain the project
        assert!(tab.auto_refreshed_platforms.contains(&project_id));

        // Attempting to insert again should not cause issues
        tab.auto_refreshed_platforms.insert(project_id.clone());
        assert_eq!(tab.auto_refreshed_platforms.len(), 1);
    }

    #[test]
    fn test_firewall_rule_default() {
        // Verify FirewallRule has sensible defaults
        let rule = FirewallRule::default();
        assert_eq!(rule.name, "");
        assert_eq!(rule.network, "default");
        assert_eq!(rule.direction, "INGRESS");
        assert_eq!(rule.priority, 1000);
        assert!(rule.source_ranges.is_empty());
        assert!(rule.allowed.is_empty());
    }

    #[test]
    fn test_firewall_validation_has_warnings() {
        // Validation with no warnings
        let validation = FirewallValidation {
            current_ip_missing: false,
            full_access_count: 1,
            full_access_ips: vec![],
            ssh_access_ips: vec![],
            http_exposed: true,
            https_exposed: true,
        };
        assert!(!validation.has_warnings());

        // Validation with current IP missing
        let validation = FirewallValidation {
            current_ip_missing: true,
            ..Default::default()
        };
        assert!(validation.has_warnings());

        // Validation with multiple full access
        let validation = FirewallValidation {
            full_access_count: 2,
            ..Default::default()
        };
        assert!(validation.has_warnings());
    }

    #[test]
    fn test_firewall_save_progress_state() {
        let mut tab = PlatformTab::default();

        // Initially not in progress
        assert!(!tab.firewall_save_in_progress);
        assert!(tab.firewall_save_error.is_none());

        // Mark as in progress
        tab.firewall_save_in_progress = true;
        assert!(tab.firewall_save_in_progress);

        // Set error
        tab.firewall_save_error = Some("Test error".to_string());
        assert_eq!(tab.firewall_save_error.as_deref(), Some("Test error"));
    }

    #[test]
    fn test_firewall_error_handling() {
        let mut tab = PlatformTab::default();
        tab.firewall_save_in_progress = true;

        // Simulate error
        tab.firewall_save_error = Some("API failed".to_string());
        tab.firewall_save_in_progress = false;

        // Dialog should stay open with error
        assert!(!tab.firewall_save_in_progress);
        assert!(tab.firewall_save_error.is_some());
    }

    #[test]
    fn test_firewall_refresh_debounce() {
        let mut tab = PlatformTab::default();

        // Initially no refresh time
        assert!(tab.firewall_last_refresh_time.is_none());

        // Set refresh time
        tab.firewall_last_refresh_time = Some(std::time::Instant::now());
        assert!(tab.firewall_last_refresh_time.is_some());

        // Check debounce window
        let last_refresh = tab.firewall_last_refresh_time.unwrap();
        let elapsed = last_refresh.elapsed();
        assert!(elapsed < std::time::Duration::from_secs(2));
    }

    #[test]
    fn test_delete_firewall_confirmation_state() {
        let mut tab = PlatformTab::default();

        // Initially no confirmation shown
        assert!(!tab.show_delete_firewall_confirm);
        assert_eq!(tab.delete_firewall_rule_name, "");
        assert!(tab.delete_firewall_rule_idx.is_none());

        // Trigger delete confirmation
        tab.show_delete_firewall_confirm = true;
        tab.delete_firewall_rule_name = "test-rule".to_string();
        tab.delete_firewall_rule_idx = Some(5);

        assert!(tab.show_delete_firewall_confirm);
        assert_eq!(tab.delete_firewall_rule_name, "test-rule");
        assert_eq!(tab.delete_firewall_rule_idx, Some(5));

        // Cancel confirmation
        tab.show_delete_firewall_confirm = false;
        tab.delete_firewall_rule_name.clear();
        tab.delete_firewall_rule_idx = None;

        assert!(!tab.show_delete_firewall_confirm);
        assert_eq!(tab.delete_firewall_rule_name, "");
        assert!(tab.delete_firewall_rule_idx.is_none());
    }

    #[test]
    fn test_firewall_refresh_state_cleanup() {
        let mut tab = PlatformTab::default();
        tab.firewall_last_refresh_time = Some(std::time::Instant::now());

        // Clear state
        tab.firewall_last_refresh_time = None;
        assert!(tab.firewall_last_refresh_time.is_none());
    }

    // Wizard tests
    #[test]
    fn test_wizard_initial_state() {
        let tab = PlatformTab::default();
        assert!(!tab.wizard_dialog_open);
        assert_eq!(tab.wizard_current_step, 0);
        assert!(tab.add_platform_connected_email.is_none());
    }

    #[test]
    fn test_wizard_step_validation_oauth() {
        let mut tab = PlatformTab::default();

        // Step 0: OAuth required
        tab.wizard_current_step = 0;
        assert!(!tab.can_proceed_wizard());

        // After OAuth connection
        tab.add_platform_connected_email = Some("test@example.com".to_string());
        assert!(tab.can_proceed_wizard());
    }

    #[test]
    fn test_wizard_step_validation_project_existing() {
        let mut tab = PlatformTab::default();
        tab.wizard_current_step = 1;
        tab.add_platform_create_new = false;

        // No project selected
        assert!(!tab.can_proceed_wizard());

        // Project selected
        tab.add_platform_project_list = vec![("project-1".to_string(), "Project 1".to_string())];
        tab.add_platform_selected_project = Some(0);
        assert!(tab.can_proceed_wizard());
    }

    #[test]
    fn test_wizard_step_validation_project_new() {
        let mut tab = PlatformTab::default();
        tab.wizard_current_step = 1;
        tab.add_platform_create_new = true;

        // Empty project ID
        assert!(!tab.can_proceed_wizard());

        // Project ID entered
        tab.add_platform_new_project_id = "new-project".to_string();
        assert!(tab.can_proceed_wizard());
    }

    #[test]
    fn test_wizard_step_validation_confirm() {
        let mut tab = PlatformTab::default();
        tab.wizard_current_step = 2;

        // Confirm step always allows proceeding
        assert!(tab.can_proceed_wizard());
    }

    #[test]
    fn test_wizard_state_reset() {
        let mut tab = PlatformTab::default();

        // Set wizard state
        tab.wizard_dialog_open = true;
        tab.wizard_current_step = 2;
        tab.add_platform_connected_email = Some("test@example.com".to_string());
        tab.add_platform_project_list = vec![("project-1".to_string(), "Project 1".to_string())];

        // Reset
        tab.reset_wizard_state();

        // Verify reset
        assert!(!tab.wizard_dialog_open);
        assert_eq!(tab.wizard_current_step, 0);
        assert!(tab.add_platform_connected_email.is_none());
        assert!(tab.add_platform_project_list.is_empty());
    }

    #[test]
    fn test_wizard_navigation_forward() {
        let mut tab = PlatformTab::default();

        // Start at step 0
        assert_eq!(tab.wizard_current_step, 0);

        // Complete OAuth
        tab.add_platform_connected_email = Some("test@example.com".to_string());

        // Can proceed → simulate Next click
        assert!(tab.can_proceed_wizard());
        tab.wizard_current_step += 1;

        // Now at step 1
        assert_eq!(tab.wizard_current_step, 1);
    }

    #[test]
    fn test_wizard_navigation_backward() {
        let mut tab = PlatformTab::default();

        // Start at step 1
        tab.wizard_current_step = 1;

        // Go back
        tab.wizard_current_step -= 1;

        // Now at step 0
        assert_eq!(tab.wizard_current_step, 0);
    }
}
