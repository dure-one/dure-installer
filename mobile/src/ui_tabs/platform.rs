//! Platform tab - Platform configuration and management

use eframe::egui;
use egui_material3::{MaterialButton, data_table};

use crate::calc::audit;
use crate::calc::gcp_rest::BillingRecord;
use crate::config::{AppConfig, CloudPlatformConfig};

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use crate::ui_dlg::platform_gcp::GcpWizard;

/// Platform row data for data table
#[derive(Clone, Debug)]
struct PlatformRow {
    // Identity
    platform_name: String, // Internal platform name from config
    platform_type: String, // "GCP"

    // Connection state flags (for Steps column)
    gcp_connected: bool,    // Has OAuth access token
    project_selected: bool, // Has gcp_selected_project_id
    vm_created: bool,       // vms.len() > 0
    firewall_updated: bool, // Current IP is whitelisted
    ssh_ready: bool,        // VM has external_ip.is_some()

    // Drawer content data
    email: Option<String>,           // Connected Google account
    total_project_count: usize,      // Fetched from GCP API
    selected_project_id: Option<String>,
    vm_name: Option<String>,         // First VM name
    vm_external_ip: Option<String>,  // First VM external IP
    ssh_private_key: Option<String>, // SSH private key from KeePass
    ssh_public_key: Option<String>,  // Derived SSH public key for verification
    ssh_keyring_domain: Option<String>, // Keyring domain for SSH key
    firewall_status: String,         // "✓ Whitelisted (IP)" or "✗ Not whitelisted"
    ssh_status: String,              // "✓ Ready" or "? No external IP"

    // Action button state
    has_vm: bool,            // Enable/disable VM operation buttons
    vm_zone: Option<String>, // For VM operations (delete, restart, regen)
}

/// Actions that can be triggered from platform table rows
#[derive(Debug, Clone)]
enum PlatformAction {
    UpdateFirewall(String), // platform_name
    SelectProject(String),  // platform_name
    DeleteVM {
        platform_name: String,
        vm_name: String,
        vm_zone: String,
    },
    RegenVM(String),        // platform_name
    RestartVM(String),      // platform_name
    DeletePlatform(String), // platform_name
    Refresh,                // Refresh table data
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
    add_platform_name: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_type: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_oauth_result: Option<crate::api::gcp_oauth::OAuthResult>,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_oauth_promise:
        Option<poll_promise::Promise<Result<crate::api::gcp_oauth::OAuthResult, String>>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_connected_email: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_project_list: Vec<(String, String)>,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_platform_selected_project: Option<usize>,

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

    // SSH connection test state (per platform)
    #[cfg_attr(feature = "serde", serde(skip))]
    ssh_test_promises: std::collections::HashMap<String, poll_promise::Promise<Result<crate::calc::ssh::SshConnectionResult, String>>>,
    #[cfg_attr(feature = "serde", serde(skip))]
    ssh_test_results: std::collections::HashMap<String, Result<crate::calc::ssh::SshConnectionResult, String>>,
}

impl Default for PlatformTab {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            loaded: false,
            load_error: None,
            show_add_dialog: false,
            add_platform_name: String::new(),
            add_platform_type: "gcp".to_string(),
            add_platform_oauth_result: None,
            add_platform_oauth_promise: None,
            add_platform_connected_email: None,
            add_platform_project_list: Vec::new(),
            add_platform_selected_project: None,
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
            ssh_test_promises: std::collections::HashMap::new(),
            ssh_test_results: std::collections::HashMap::new(),
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

/// Load application config
#[cfg(not(target_arch = "wasm32"))]
fn load_config() -> Result<(AppConfig, std::path::PathBuf), String> {
    let config_path = get_config_path()?;
    let app_config = AppConfig::load_or_default(&config_path);
    Ok((app_config, config_path))
}

/// Format connection progress steps with status indicators
fn format_steps(row: &PlatformRow) -> String {
    let gcp = if row.gcp_connected { "✓" } else { "✗" };
    let proj = if row.project_selected { "✓" } else { "✗" };
    let vm = if row.vm_created { "✓" } else { "✗" };
    let firewall = if row.firewall_updated { "✓" } else { "✗" };
    let ssh = if row.ssh_ready { "✓" } else { "✗" };

    format!(
        "{} GCP Connected → {} Project Created → {} VM Created → {} Firewall Rules Updated → {} SSH Connected",
        gcp, proj, vm, firewall, ssh
    )
}

/// Compute firewall whitelist status for a platform
///
/// # Arguments
/// * `access_token` - Valid (non-expired) OAuth access token
/// * `project_id` - GCP project ID to check
fn compute_firewall_status(access_token: Option<&str>, project_id: Option<&str>) -> String {
    if let Some(project) = project_id {
        if let Some(token) = access_token {
            use crate::calc::gcp_rest::{GcpRestClient, get_current_ip};

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

    let len = u32::from_be_bytes([data[*offset], data[*offset + 1], data[*offset + 2], data[*offset + 3]]) as usize;
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
        key_data.as_bytes()
    ).ok()?;

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
    let public_key_b64 = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &public_key_blob
    );
    Some(format!("ssh-ed25519 {} dure-generated", public_key_b64))
}

/// Derive SSH public key from raw Ed25519 private key bytes or OpenSSH format
#[cfg(not(target_arch = "wasm32"))]
fn derive_public_key_from_raw(raw_bytes: &[u8]) -> Option<String> {
    use ed25519_dalek::SigningKey;

    // Try to interpret as OpenSSH format first
    if let Ok(key_str) = String::from_utf8(raw_bytes.to_vec()) {
        if key_str.contains("BEGIN") && key_str.contains("PRIVATE KEY") {
            eprintln!("DEBUG: Extracting public key from OpenSSH format");
            return extract_pubkey_from_openssh(&key_str);
        }
    }

    // Otherwise, treat as raw 32-byte Ed25519 key
    if raw_bytes.len() != 32 {
        eprintln!("DEBUG: Key is neither OpenSSH format nor raw 32 bytes (length: {})", raw_bytes.len());
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
            eprintln!("DEBUG: Loading SSH key for domain: {}", d);
            d
        }
        None => {
            eprintln!("DEBUG: No keyring domain provided");
            return (None, None);
        }
    };

    let kdbx_path = match keyring::get_default_kdbx_path() {
        Ok(p) => {
            eprintln!("DEBUG: KeePass DB path: {}", p.display());
            p
        }
        Err(e) => {
            eprintln!("DEBUG: Failed to get kdbx path: {}", e);
            return (None, None);
        }
    };
    let kpkey_path = match keyring::get_default_kpkey_path() {
        Ok(p) => {
            eprintln!("DEBUG: KPKey path: {}", p.display());
            p
        }
        Err(e) => {
            eprintln!("DEBUG: Failed to get kpkey path: {}", e);
            return (None, None);
        }
    };

    let keys = match keyring::list_keys(&kdbx_path, Some(&kpkey_path)) {
        Ok(k) => {
            eprintln!("DEBUG: Found {} keys in keyring", k.len());
            for key in &k {
                eprintln!("  - Domain: {}, Username: {}, Has SSH: {}",
                    key.domain, key.username, key.ssh_key.is_some());
            }
            k
        }
        Err(e) => {
            eprintln!("DEBUG: Failed to list keys: {}", e);
            return (None, None);
        }
    };

    // Find the key with matching domain
    let key_entry = match keys.iter().find(|k| &k.domain == domain) {
        Some(e) => {
            eprintln!("DEBUG: Found matching key entry");
            e
        }
        None => {
            eprintln!("DEBUG: No key found for domain: {}", domain);
            return (None, None);
        }
    };

    // Try to get SSH key from binary attachment
    if let Some(ssh_key_bytes) = &key_entry.ssh_key {
        eprintln!("DEBUG: SSH key bytes length: {}", ssh_key_bytes.len());

        // Derive public key from raw bytes
        let public_key = derive_public_key_from_raw(ssh_key_bytes);
        if let Some(ref pk) = public_key {
            eprintln!("DEBUG: Derived public key: {}", pk);
        } else {
            eprintln!("DEBUG: Failed to derive public key");
        }

        // Try to interpret as UTF-8 string first (already in OpenSSH format)
        if let Ok(key_str) = String::from_utf8(ssh_key_bytes.clone()) {
            if key_str.contains("BEGIN") && key_str.contains("PRIVATE KEY") {
                eprintln!("DEBUG: Key already in OpenSSH format");
                return (Some(key_str), public_key);
            }
        }

        // Otherwise, try to convert raw Ed25519 bytes to OpenSSH format
        eprintln!("DEBUG: Converting raw bytes to OpenSSH format");
        let private_key = convert_ed25519_to_openssh(ssh_key_bytes);
        (private_key, public_key)
    } else {
        eprintln!("DEBUG: Key entry has no SSH key attachment");
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
        use crate::calc::gcp_rest::GcpRestClient;
        let client = GcpRestClient::new(token.to_string());

        match client.list_projects(None) {
            Ok(list) => list.projects.len(),
            Err(e) => {
                eprintln!("Failed to fetch project count: {}", e);
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

    // Level 1: Email + project count
    if let Some(email) = &row.email {
        ui.label(format!(
            "{} ({} projects total)",
            email, row.total_project_count
        ));
    } else {
        ui.label("Not connected");
    }

    // Level 2: Selected project
    if let Some(project_id) = &row.selected_project_id {
        ui.label(format!("  └─ Project: {} (selected)", project_id));

        // Level 3: VM details
        if let Some(vm_name) = &row.vm_name {
            let vm_display = if let Some(external_ip) = &row.vm_external_ip {
                format!("     └─ VM: {}({})", vm_name, external_ip)
            } else {
                format!("     └─ VM: {} (no external IP)", vm_name)
            };
            ui.label(vm_display);
            ui.label(format!("        • Firewall: {}", row.firewall_status));
            ui.label(format!("        • SSH: {}", row.ssh_status));

            // Show derived public key for verification
            // if let Some(public_key) = &row.ssh_public_key {
            //     ui.add_space(4.0);
            //     ui.label("        • Public Key (from keyring):");
            //     egui::ScrollArea::horizontal()
            //         .id_salt(format!("pubkey_{}", vm_name))
            //         .max_width(ui.available_width() - 32.0)
            //         .show(ui, |ui| {
            //             ui.add(
            //                 egui::TextEdit::singleline(&mut public_key.as_str())
            //                     .font(egui::TextStyle::Monospace)
            //                     .desired_width(f32::INFINITY)
            //             );
            //         });
            //     ui.label("          (Compare this with /root/.ssh/authorized_keys on VM)");
            // }

            // Show SSH connection command if we have the key and external IP
            if let (Some(external_ip), Some(ssh_key)) = (&row.vm_external_ip, &row.ssh_private_key) {
                ui.add_space(4.0);
                ui.label("        • SSH Connect:");

                // Create temp file, set permissions, connect, then cleanup
                let ssh_command = format!(
                    "K=$(mktemp) && cat > $K <<'EOF'\n{}\nEOF\nchmod 600 $K && ssh -i $K root@{} && rm $K",
                    ssh_key.trim(),
                    external_ip
                );

                // Show in a scrollable, selectable text area
                egui::ScrollArea::horizontal()
                    .id_salt(format!("ssh_cmd_{}", vm_name))
                    .max_width(ui.available_width() - 32.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut ssh_command.as_str())
                                .font(egui::TextStyle::Monospace)
                                .desired_width(f32::INFINITY)
                                .desired_rows(3)
                                .interactive(true)
                        );
                    });

                ui.add_space(2.0);
                ui.label("          (Copy and paste into terminal - key auto-deletes after use)");
            } else if row.vm_external_ip.is_some() && row.ssh_keyring_domain.is_some() {
                ui.add_space(4.0);
                ui.colored_label(
                    egui::Color32::from_rgb(255, 152, 0),
                    "        ⚠ SSH key not found in keyring"
                );
            }
        } else {
            ui.label("     └─ No VM created");
        }
    } else {
        ui.label("  └─ No project selected");
    }
}

impl PlatformTab {
    /// Render the platform tab UI
    pub fn ui(&mut self, ui: &mut egui::Ui, vm: Option<&mut crate::viewmodel::ViewModel>) {
        // ViewModel event processing (MVVM pattern)
        if let Some(vm) = vm {
            // Show active operations with progress bars
            for (_op_id, progress) in vm.active_operations() {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::ProgressBar::new(progress.progress)
                            .text(format!("{}: {}", progress.operation, progress.status))
                            .desired_width(400.0)
                    );
                });
            }

            // Show recent errors
            if let Some(error) = vm.recent_errors()
                .iter()
                .filter(|e| e.actor == "platform")
                .rev()
                .next()
            {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 100, 100),
                    format!("⚠ Error in {}: {}", error.operation, error.error)
                );
                ui.add_space(4.0);
            }
        }

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
                    self.ssh_test_results.insert(platform_name.clone(), result.clone());
                }
            }
            for platform_name in completed {
                self.ssh_test_promises.remove(&platform_name);
            }
        }

        // Action buttons
        if ui.add(MaterialButton::filled("Add Platform")).clicked() {
            self.show_add_dialog = true;
            self.add_platform_name.clear();
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
                .column("Platform", 150.0 * width_ratio, false)
                .column("Type", 80.0 * width_ratio, false)
                .column("Steps", 250.0 * width_ratio, false)
                .column("Operations", 260.0 * width_ratio, false);

            for (idx, row) in self.rows.iter().enumerate() {
                let row_for_cells = row.clone();
                let row_for_drawer = row.clone();
                let row_for_actions = row.clone();

                table = table.row(move |r| {
                    r.cell(&row_for_cells.platform_name)
                        .cell(&row_for_cells.platform_type)
                        .cell(&format_steps(&row_for_cells))
                        .widget_cell(move |ui| {
                            egui::ScrollArea::horizontal()
                                .id_salt(format!("operations_scroll_{}", idx))
                                .auto_shrink([false, true])
                                .show(ui, |ui| {
                                    ui.horizontal_wrapped(|ui| {
                                        ui.spacing_mut().item_spacing.x = 2.0;
                                        ui.style_mut().spacing.button_padding = egui::vec2(6.0, 2.0);

                                        // 0. Refresh
                                        if ui.add(MaterialButton::outlined("Refresh").small()).on_hover_text("Refresh platform data").clicked() {
                                            ui.data_mut(|d| d.insert_temp(
                                                egui::Id::new("platform_action_refresh"),
                                                row_for_actions.platform_name.clone()
                                            ));
                                        }

                                        // 1. Add VM
                                        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
                                        if ui.add_enabled(!row_for_actions.has_vm && row_for_actions.project_selected,
                                            MaterialButton::outlined("Add VM").small()).on_hover_text("Add VM").clicked() {
                                            ui.data_mut(|d| d.insert_temp(
                                                egui::Id::new("platform_action_add_vm"),
                                                row_for_actions.platform_name.clone()
                                            ));
                                        }

                                        // 2. Firewall
                                        if ui.add_enabled(row_for_actions.project_selected && !row_for_actions.firewall_updated,
                                            MaterialButton::outlined("Firewall").small()).on_hover_text("Update Firewall").clicked() {
                                            ui.data_mut(|d| d.insert_temp(
                                                egui::Id::new("platform_action_update_firewall"),
                                                row_for_actions.platform_name.clone()
                                            ));
                                        }

                                        // 3. Restart
                                        if ui.add_enabled(row_for_actions.has_vm,
                                            MaterialButton::outlined("Restart").small()).on_hover_text("Restart VM").clicked() {
                                            ui.data_mut(|d| d.insert_temp(
                                                egui::Id::new("platform_action_restart_vm"),
                                                row_for_actions.platform_name.clone()
                                            ));
                                        }

                                        // 4. Del VM
                                        if ui.add_enabled(row_for_actions.has_vm,
                                            MaterialButton::outlined("Del VM").small()).on_hover_text("Delete VM").clicked() {
                                            ui.data_mut(|d| d.insert_temp(
                                                egui::Id::new("platform_action_delete_vm"),
                                                (row_for_actions.platform_name.clone(),
                                                 row_for_actions.vm_name.clone().unwrap_or_default(),
                                                 row_for_actions.vm_zone.clone().unwrap_or_default())
                                            ));
                                        }

                                        // 5. Regen
                                        // if ui.add_enabled(row_for_actions.has_vm,
                                        //     MaterialButton::outlined("Regen").small()).on_hover_text("Regenerate VM").clicked() {
                                        //     ui.data_mut(|d| d.insert_temp(
                                        //         egui::Id::new("platform_action_regen_vm"),
                                        //         row_for_actions.platform_name.clone()
                                        //     ));
                                        // }

                                        // 6. Billing
                                        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
                                        if ui.add_enabled(row_for_actions.project_selected,
                                            MaterialButton::outlined("Billing").small()).on_hover_text("Estimated Billing").clicked() {
                                            ui.data_mut(|d| d.insert_temp(
                                                egui::Id::new("platform_action_billing"),
                                                row_for_actions.platform_name.clone()
                                            ));
                                        }

                                        // 7. Delete
                                        if ui.add(MaterialButton::outlined("Delete").small()).on_hover_text("Delete Platform").clicked() {
                                            ui.data_mut(|d| d.insert_temp(
                                                egui::Id::new("platform_action_delete_platform"),
                                                row_for_actions.platform_name.clone()
                                            ));
                                        }
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
            if let Some(platform_name) = ui.data(|d|
                d.get_temp::<String>(egui::Id::new("platform_action_refresh"))) {
                self.loaded = false;

                // Trigger SSH connection test for this platform
                #[cfg(not(target_arch = "wasm32"))]
                self.execute_test_connection(platform_name.clone());

                ui.data_mut(|d| d.remove::<String>(egui::Id::new("platform_action_refresh")));
            }

            #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
            {
                if let Some(platform_name) = ui.data(|d|
                    d.get_temp::<String>(egui::Id::new("platform_action_update_firewall"))) {
                    // Find platform and get project_id
                    if let Ok((app_config, _)) = load_config() {
                        if let Some(platform) = app_config.platforms.iter().find(|p| p.name == platform_name) {
                            if let Some(project_id) = &platform.gcp_selected_project_id {
                                self.update_firewall(platform_name, project_id.clone());
                            }
                        }
                    }
                    ui.data_mut(|d| d.remove::<String>(egui::Id::new("platform_action_update_firewall")));
                }

                if let Some((platform_name, vm_name, vm_zone)) = ui.data(|d|
                    d.get_temp::<(String, String, String)>(egui::Id::new("platform_action_delete_vm"))) {
                    self.show_delete_vm_confirmation(platform_name, vm_name, vm_zone);
                    ui.data_mut(|d| d.remove::<(String, String, String)>(egui::Id::new("platform_action_delete_vm")));
                }

                if let Some(platform_name) = ui.data(|d|
                    d.get_temp::<String>(egui::Id::new("platform_action_regen_vm"))) {
                    // Find platform and get vm_name
                    if let Ok((app_config, _)) = load_config() {
                        if let Some(platform) = app_config.platforms.iter().find(|p| p.name == platform_name) {
                            if let Some(vm) = platform.vms.first() {
                                self.regenerate_vm(platform_name, vm.name.clone());
                            }
                        }
                    }
                    ui.data_mut(|d| d.remove::<String>(egui::Id::new("platform_action_regen_vm")));
                }

                if let Some(platform_name) = ui.data(|d|
                    d.get_temp::<String>(egui::Id::new("platform_action_restart_vm"))) {
                    // Find platform and get vm_name
                    if let Ok((app_config, _)) = load_config() {
                        if let Some(platform) = app_config.platforms.iter().find(|p| p.name == platform_name) {
                            if let Some(vm) = platform.vms.first() {
                                self.restart_vm(platform_name, vm.name.clone());
                            }
                        }
                    }
                    ui.data_mut(|d| d.remove::<String>(egui::Id::new("platform_action_restart_vm")));
                }

                if let Some(platform_name) = ui.data(|d|
                    d.get_temp::<String>(egui::Id::new("platform_action_add_vm"))) {
                    self.show_gcp_wizard(platform_name);
                    ui.data_mut(|d| d.remove::<String>(egui::Id::new("platform_action_add_vm")));
                }

                if let Some(_platform_name) = ui.data(|d|
                    d.get_temp::<String>(egui::Id::new("platform_action_billing"))) {
                    self.show_billing_dialog = true;
                    self.fetch_billing_data();
                    ui.data_mut(|d| d.remove::<String>(egui::Id::new("platform_action_billing")));
                }
            }

            if let Some(platform_name) = ui.data(|d|
                d.get_temp::<String>(egui::Id::new("platform_action_delete_platform"))) {
                self.show_delete_platform_confirmation(platform_name);
                ui.data_mut(|d| d.remove::<String>(egui::Id::new("platform_action_delete_platform")));
            }
        }

        // Add platform dialog
        if self.show_add_dialog {
            self.render_add_dialog(ui.ctx());
        }

        // Delete Platform dialog
        if self.show_delete_platform_dialog {
            self.render_delete_platform_dialog(ui.ctx());
        }

        // Select Project dialog
        #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
        if self.show_select_project_dialog {
            self.render_select_project_dialog(ui.ctx());
        }

        // Delete VM dialog
        if self.show_delete_vm_dialog {
            self.render_delete_vm_dialog(ui.ctx());
        }

        // Billing dialog
        if self.show_billing_dialog {
            self.render_billing_dialog(ui.ctx());
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
                eprintln!("✓ GCP wizard closed, refreshing platform spreadsheet");
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
                        let access_token = if app_config.platforms[idx].gcp_oauth_access_token.is_some() {
                            match self.get_valid_access_token(&mut app_config, idx, &config_path) {
                                Ok(token) => Some(token),
                                Err(e) => {
                                    eprintln!("Failed to get valid access token for platform '{}': {}",
                                        app_config.platforms[idx].name, e);
                                    None
                                }
                            }
                        } else {
                            None
                        };

                        // Borrow platform after get_valid_access_token
                        let platform = &app_config.platforms[idx];

                        // Compute firewall status string (fresh fetch from GCP)
                        let firewall_status_str = compute_firewall_status(
                            access_token.as_deref(),
                            platform.gcp_selected_project_id.as_deref()
                        );

                        // Compute SSH status and readiness flag
                        let ssh_status_str = compute_ssh_status(
                            platform,
                            self.ssh_test_results.get(&platform.name)
                        );

                        // ssh_ready should match ssh_status: only true if actually connected
                        let ssh_ready = if let Some(result) = self.ssh_test_results.get(&platform.name) {
                            matches!(result, Ok(conn_result) if conn_result.success)
                        } else {
                            false
                        };

                        // Load SSH private key from KeePass if VM exists
                        let (ssh_private_key, ssh_public_key, ssh_keyring_domain) = if let Some(vm) = platform.vms.first() {
                            let keyring_domain = vm.ssh_key_name.clone();
                            let (private_key, public_key) = load_ssh_key_from_keyring(&keyring_domain);
                            (private_key, public_key, keyring_domain)
                        } else {
                            (None, None, None)
                        };

                        let row = PlatformRow {
                            platform_name: platform.name.clone(),
                            platform_type: "GCP".to_string(),

                            // Compute state flags
                            gcp_connected: platform.gcp_oauth_access_token.is_some(),
                            project_selected: platform.gcp_selected_project_id.is_some(),
                            vm_created: !platform.vms.is_empty(),
                            firewall_updated: firewall_status_str.starts_with("✓"),
                            ssh_ready,

                            // Extract drawer data
                            email: platform.gcp_connected_email.clone(),
                            total_project_count: fetch_project_count(access_token.as_deref()),
                            selected_project_id: platform.gcp_selected_project_id.clone(),
                            vm_name: platform.vms.first().map(|vm| vm.name.clone()),
                            vm_external_ip: platform.vms.first().and_then(|vm| vm.external_ip.clone()),
                            ssh_private_key,
                            ssh_public_key,
                            ssh_keyring_domain,
                            firewall_status: firewall_status_str,
                            ssh_status: ssh_status_str,

                            // Action button state
                            has_vm: !platform.vms.is_empty(),
                            vm_zone: platform.vms.first().map(|vm| vm.zone.clone()),
                        };

                        self.rows.push(row);

                        // Trigger SSH connection test on first load if VM has external IP
                        #[cfg(not(target_arch = "wasm32"))]
                        {
                            if platform.vms.first().and_then(|vm| vm.external_ip.as_ref()).is_some()
                                && !self.ssh_test_results.contains_key(&platform.name)
                                && !self.ssh_test_promises.contains_key(&platform.name)
                            {
                                self.execute_test_connection(platform.name.clone());
                            }
                        }
                    }

                    self.loaded = true;
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {}", e));
                }
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.load_error = Some("WASM platform not supported".to_string());
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
        use crate::calc::gcp_rest::GcpRestClient;

        // Check if we have a cached summary
        if let Some(cached) = self.platform_summaries.get(&platform.name) {
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
            // Cache the summary
            self.platform_summaries
                .insert(platform.name.clone(), summary.clone());
            Some(summary)
        }
    }

    fn render_add_dialog(&mut self, ctx: &egui::Context) {
        let mut open = self.show_add_dialog;

        egui::Window::new("Add Platform")
            .open(&mut open)
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                ui.label("Configure a new cloud platform:");
                ui.add_space(8.0);

                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.add_platform_name);
                });

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
                                use crate::calc::gcp_rest::GcpRestClient;
                                let client = GcpRestClient::new(oauth_result.access_token.clone());
                                match client.list_projects(None) {
                                    Ok(project_list) => {
                                        self.add_platform_project_list = project_list
                                            .projects
                                            .into_iter()
                                            .filter(|p| p.is_active())
                                            .map(|p| (p.id().to_string(), p.display_name().to_string()))
                                            .collect();
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to fetch projects: {}", e);
                                    }
                                }
                            }
                        }

                        // Show project selection
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
                                    let is_selected = self.add_platform_selected_project == Some(idx);
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
                    }

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(8.0);
                }

                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.show_add_dialog = false;
                        self.add_platform_oauth_result = None;
                        self.add_platform_oauth_promise = None;
                        self.add_platform_connected_email = None;
                        self.add_platform_project_list.clear();
                        self.add_platform_selected_project = None;
                    }

                    let can_add = !self.add_platform_name.is_empty()
                        && (self.add_platform_type != "gcp"
                            || (self.add_platform_connected_email.is_some()
                                && self.add_platform_selected_project.is_some()));

                    ui.add_enabled_ui(can_add, |ui| {
                        if ui.button("Add").clicked() {
                            self.execute_add_platform();
                            self.show_add_dialog = false;
                            self.add_platform_oauth_result = None;
                            self.add_platform_oauth_promise = None;
                            self.add_platform_connected_email = None;
                            self.add_platform_project_list.clear();
                            self.add_platform_selected_project = None;
                        }
                    });

                    if !can_add {
                        if self.add_platform_name.is_empty() {
                            ui.label("⚠ Name required");
                        } else if self.add_platform_type == "gcp"
                            && self.add_platform_connected_email.is_none()
                        {
                            ui.label("⚠ Connect to Google Cloud first");
                        } else if self.add_platform_type == "gcp"
                            && self.add_platform_selected_project.is_none()
                        {
                            ui.label("⚠ Select a project");
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

    fn execute_add_platform(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            match load_config() {
                Ok((mut app_config, config_path)) => {
                    // Check if platform already exists
                    if app_config
                        .platforms
                        .iter()
                        .any(|p| p.name == self.add_platform_name)
                    {
                        self.load_error = Some(format!(
                            "Platform '{}' already exists",
                            self.add_platform_name
                        ));
                        return;
                    }

                    // Create new platform with OAuth info if GCP
                    let (oauth_access, oauth_refresh, oauth_expiry, connected_email, selected_project) =
                        if self.add_platform_type == "gcp" {
                            if let Some(oauth) = &self.add_platform_oauth_result {
                                let project_id = self.add_platform_selected_project
                                    .and_then(|idx| self.add_platform_project_list.get(idx))
                                    .map(|(id, _)| id.clone());
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

                    let platform = CloudPlatformConfig {
                        name: self.add_platform_name.clone(),
                        platform_type: self.add_platform_type.clone(),
                        gcp_oauth_access_token: oauth_access,
                        gcp_oauth_refresh_token: oauth_refresh,
                        gcp_oauth_token_expiry: oauth_expiry,
                        gcp_connected_email: connected_email,
                        gcp_selected_project_id: selected_project,
                        firebase_project_id: None,
                        firebase_api_key: None,
                        supabase_project_ref: None,
                        supabase_api_url: None,
                        supabase_anon_key: None,
                        api_token: None,
                        service_account_json: None,
                        vms: Vec::new(),
                    };

                    // Add to config
                    app_config.platforms.push(platform);

                    // Save config
                    match app_config.save(&config_path) {
                        Ok(_) => {
                            // Record audit event
                            match audit::push_gui(
                                "system",
                                "desktop",
                                "platform add",
                                &self.add_platform_name,
                            ) {
                                Ok(audit_id) => {
                                    eprintln!("✓ Audit record created: ID {}", audit_id);
                                }
                                Err(e) => {
                                    eprintln!("⚠ Failed to record audit event: {}", e);
                                    self.load_error = Some(format!("Audit tracking failed: {}", e));
                                }
                            }

                            eprintln!("✓ Platform added, refreshing spreadsheet");
                            self.loaded = false; // Trigger reload
                            self.load_error = None;
                        }
                        Err(e) => {
                            self.load_error = Some(format!("Failed to save config: {e}"));
                        }
                    }
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {e}"));
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
        let mut wizard = GcpWizard::new(platform_name);

        // Load OAuth from config if exists
        if let Ok((app_config, _)) = load_config() {
            wizard.load_oauth_from_config(&app_config);
        }

        wizard.show();
        self.gcp_wizard = Some(wizard);
    }

    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn restart_vm(&mut self, platform_name: String, vm_name: String) {
        use crate::calc::gcp_rest::GcpRestClient;
        use crate::calc::hosting_gcp;

        // Load config
        let (app_config, config_path) = match load_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Failed to load config: {}", e);
                self.load_error = Some(format!("Failed to load config: {}", e));
                return;
            }
        };

        // Find platform
        let platform = match app_config
            .platforms
            .iter()
            .find(|p| p.name == platform_name)
        {
            Some(p) => p,
            None => {
                eprintln!("Platform not found: {}", platform_name);
                self.load_error = Some(format!("Platform not found: {}", platform_name));
                return;
            }
        };

        // Find VM
        let vm = match platform.vms.iter().find(|v| v.name == vm_name) {
            Some(v) => v,
            None => {
                eprintln!("VM not found: {}", vm_name);
                self.load_error = Some(format!("VM not found: {}", vm_name));
                return;
            }
        };

        // Get access token
        let access_token = match &platform.gcp_oauth_access_token {
            Some(token) => token.clone(),
            None => {
                eprintln!("No access token for platform: {}", platform_name);
                self.load_error = Some("OAuth not connected".to_string());
                return;
            }
        };

        // Create GCP client
        let client = GcpRestClient::new(access_token);

        // Restart VM
        match hosting_gcp::restart_vm(&client, vm) {
            Ok(message) => {
                eprintln!("✓ {}", message);
                self.loaded = false;
                self.load_error = None;
            }
            Err(e) => {
                eprintln!("Failed to restart VM: {}", e);
                self.load_error = Some(format!("Failed to restart VM: {}", e));
            }
        }
    }

    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn regenerate_vm(&mut self, platform_name: String, vm_name: String) {
        use crate::calc::gcp_rest::GcpRestClient;
        use crate::calc::hosting_gcp;

        // Load config
        let (mut app_config, config_path) = match load_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Failed to load config: {}", e);
                self.load_error = Some(format!("Failed to load config: {}", e));
                return;
            }
        };

        // Find platform (mutable)
        let platform = match app_config
            .platforms
            .iter_mut()
            .find(|p| p.name == platform_name)
        {
            Some(p) => p,
            None => {
                eprintln!("Platform not found: {}", platform_name);
                self.load_error = Some(format!("Platform not found: {}", platform_name));
                return;
            }
        };

        // Find VM to get zone
        let zone = match platform.vms.iter().find(|v| v.name == vm_name) {
            Some(v) => v.zone.clone(),
            None => {
                eprintln!("VM not found: {}", vm_name);
                self.load_error = Some(format!("VM not found: {}", vm_name));
                return;
            }
        };

        // Get access token
        let access_token = match &platform.gcp_oauth_access_token {
            Some(token) => token.clone(),
            None => {
                eprintln!("No access token for platform: {}", platform_name);
                self.load_error = Some("OAuth not connected".to_string());
                return;
            }
        };

        // Create GCP client
        let client = GcpRestClient::new(access_token);

        // Regenerate VM
        match hosting_gcp::regenerate_vm(&client, platform, &zone) {
            Ok(message) => {
                eprintln!("✓ {}", message);

                // Save updated config
                if let Err(e) = app_config.save(&config_path) {
                    eprintln!("Failed to save config: {}", e);
                    self.load_error = Some(format!("Failed to save config: {}", e));
                } else {
                    self.loaded = false;
                    self.load_error = None;
                }
            }
            Err(e) => {
                eprintln!("Failed to regenerate VM: {}", e);
                self.load_error = Some(format!("Failed to regenerate VM: {}", e));
            }
        }
    }

    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn update_firewall(&mut self, platform_name: String, project_id: String) {
        use crate::calc::gcp_rest::{GcpRestClient, get_current_ip};

        // Get current IP
        let current_ip = match get_current_ip() {
            Ok(ip) => ip,
            Err(e) => {
                eprintln!("Failed to get current IP: {}", e);
                self.load_error = Some(format!("Failed to get current IP: {}", e));
                return;
            }
        };

        // Load config to get access token
        let (app_config, _) = match load_config() {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Failed to load config: {}", e);
                self.load_error = Some(format!("Failed to load config: {}", e));
                return;
            }
        };

        // Find platform
        let platform = match app_config
            .platforms
            .iter()
            .find(|p| p.name == platform_name)
        {
            Some(p) => p,
            None => {
                eprintln!("Platform not found: {}", platform_name);
                self.load_error = Some(format!("Platform not found: {}", platform_name));
                return;
            }
        };

        // Get access token
        let access_token = match &platform.gcp_oauth_access_token {
            Some(token) => token.clone(),
            None => {
                eprintln!("No access token for platform: {}", platform_name);
                self.load_error = Some("OAuth not connected".to_string());
                return;
            }
        };

        // Create GCP client
        let client = GcpRestClient::new(access_token);

        // Add IP to firewall
        match client.add_ip_to_firewall(&project_id, &current_ip) {
            Ok(()) => {
                eprintln!("✓ Successfully added {} to firewall whitelist", current_ip);
                // Refresh to show updated status
                self.loaded = false;
                self.load_error = None;
            }
            Err(e) => {
                eprintln!("Failed to update firewall: {}", e);
                self.load_error = Some(format!("Failed to update firewall: {}", e));
            }
        }
    }

    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn show_select_project_dialog(&mut self, platform_name: String) {
        use crate::calc::gcp_rest::GcpRestClient;

        self.select_project_platform_name = platform_name.clone();
        self.select_project_list.clear();
        self.select_project_selected = None;

        // Load projects from GCP
        if let Ok((app_config, _)) = load_config() {
            if let Some(platform) = app_config
                .platforms
                .iter()
                .find(|p| p.name == platform_name)
            {
                if let Some(access_token) = &platform.gcp_oauth_access_token {
                    let client = GcpRestClient::new(access_token.clone());
                    match client.list_projects(None) {
                        Ok(list) => {
                            for project in list.projects {
                                self.select_project_list.push((
                                    project.project_id.clone(),
                                    project
                                        .name
                                        .clone()
                                        .unwrap_or_else(|| project.project_id.clone()),
                                ));
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to load projects: {}", e);
                            self.load_error = Some(format!("Failed to load projects: {}", e));
                        }
                    }
                }
            }
        }

        self.show_select_project_dialog = true;
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

    fn render_delete_vm_dialog(&mut self, ctx: &egui::Context) {
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
                                    self.execute_delete_vm(name_clone, zone_clone);
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
    fn execute_delete_vm(&mut self, instance_name: String, zone: String) {
        if let Ok((mut app_config, config_path)) = load_config() {
            // Find platform and VM to get project_id
            let platform_idx = app_config
                .platforms
                .iter()
                .position(|p| p.name == self.delete_vm_platform);

            if let Some(idx) = platform_idx {
                let platform = &app_config.platforms[idx];

                // Find the VM to get its project_id
                let vm = platform.vms.iter().find(|vm| vm.name == instance_name);
                if vm.is_none() {
                    self.load_error = Some(format!("VM '{}' not found in config", instance_name));
                    return;
                }
                let project_id = vm.unwrap().gcp_project_id.clone();

                // Get valid access token (refresh if expired)
                let access_token =
                    match self.get_valid_access_token(&mut app_config, idx, &config_path) {
                        Ok(token) => token,
                        Err(e) => {
                            self.load_error = Some(format!("Failed to get access token: {}", e));
                            return;
                        }
                    };

                // Delete from GCP
                use crate::calc::gcp_rest::GcpRestClient;
                let client = GcpRestClient::new(access_token);

                match client.delete_instance(&project_id, &zone, &instance_name) {
                    Ok(_operation) => {
                        self.load_error = None;

                        // Record audit event
                        match audit::push_gui(
                            "system",
                            "desktop",
                            "vm delete",
                            &format!("{}:{}", project_id, instance_name),
                        ) {
                            Ok(audit_id) => {
                                eprintln!("✓ Audit record created: ID {}", audit_id);
                            }
                            Err(e) => {
                                eprintln!("⚠ Failed to record audit event: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        self.load_error = Some(format!("Failed to delete VM from GCP: {}", e));
                        return;
                    }
                }

                // Remove VM from config after successful deletion
                app_config.platforms[idx]
                    .vms
                    .retain(|vm| vm.name != instance_name);

                // Save config
                if let Err(e) = app_config.save(&config_path) {
                    self.load_error = Some(format!("Failed to save config: {}", e));
                    return;
                }

                eprintln!("✓ VM deleted, refreshing spreadsheet");
                // Refresh the list
                self.loaded = false;
            } else {
                self.load_error = Some(format!("Platform '{}' not found", self.delete_vm_platform));
            }
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
        eprintln!("Access token expired, refreshing...");

        use crate::api::gcp_oauth::{self, OAuthHandler};

        // Use embedded OAuth credentials
        let handler = OAuthHandler::default();
        let oauth_result = gcp_oauth::refresh_access_token(
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

        eprintln!("✓ Access token refreshed");
        Ok(oauth_result.access_token)
    }

    /// Execute SSH connection test for a platform's VM
    #[cfg(not(target_arch = "wasm32"))]
    fn execute_test_connection(&mut self, platform_name: String) {
        // Load config and find the VM for this platform
        let (vm_host, keyring_domain) = match load_config() {
            Ok((app_config, _)) => {
                let platform = app_config
                    .platforms
                    .iter()
                    .find(|p| p.name == platform_name);

                if let Some(platform) = platform {
                    if let Some(vm) = platform.vms.first() {
                        if let Some(external_ip) = &vm.external_ip {
                            // Construct SSH host from VM info
                            let host = format!("root@{}", external_ip);
                            (Some(host), vm.ssh_key_name.clone())
                        } else {
                            self.ssh_test_results.insert(
                                platform_name,
                                Err("VM has no external IP".to_string()),
                            );
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
                self.ssh_test_results.insert(
                    platform_name,
                    Err(format!("Failed to load config: {}", e)),
                );
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

    fn show_delete_platform_confirmation(&mut self, platform_name: String) {
        self.delete_platform_name = platform_name.clone();

        // Count VMs for this platform
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Ok((app_config, _)) = load_config() {
                if let Some(platform) = app_config
                    .platforms
                    .iter()
                    .find(|p| p.name == platform_name)
                {
                    self.delete_platform_vm_count = platform.vms.len();
                }
            }
        }

        self.show_delete_platform_dialog = true;
    }

    fn render_delete_platform_dialog(&mut self, ctx: &egui::Context) {
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

                ui.horizontal(|ui| {
                    if ui.button("No, Cancel").clicked() {
                        self.show_delete_platform_dialog = false;
                    }

                    if ui
                        .add(MaterialButton::filled("Yes, Delete Platform"))
                        .clicked()
                    {
                        self.execute_delete_platform();
                        self.show_delete_platform_dialog = false;
                    }
                });
            });

        if !open {
            self.show_delete_platform_dialog = false;
        }
    }

    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn render_select_project_dialog(&mut self, ctx: &egui::Context) {
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
                        self.execute_select_project(ctx);
                        self.show_select_project_dialog = false;
                    }
                });
            });

        if !open {
            self.show_select_project_dialog = false;
        }
    }

    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    fn execute_select_project(&mut self, ctx: &egui::Context) {
        if let Some(selected_idx) = self.select_project_selected {
            if selected_idx < self.select_project_list.len() {
                let (project_id, _) = &self.select_project_list[selected_idx];
                let platform_name = self.select_project_platform_name.clone();

                // Update config with selected project
                if let Ok((mut app_config, config_path)) = load_config() {
                    if let Some(platform) = app_config
                        .platforms
                        .iter_mut()
                        .find(|p| p.name == platform_name)
                    {
                        platform.gcp_selected_project_id = Some(project_id.clone());

                        // Save config
                        if let Err(e) = app_config.save(&config_path) {
                            eprintln!("Failed to save config: {}", e);
                            self.load_error = Some(format!("Failed to save config: {}", e));
                        } else {
                            eprintln!("✓ Selected project: {}", project_id);
                            // Refresh the table
                            self.loaded = false;
                            self.load_error = None;
                            // Request repaint to ensure UI updates
                            ctx.request_repaint();
                        }
                    }
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn execute_delete_platform(&mut self) {
        if let Ok((mut app_config, config_path)) = load_config() {
            // Find and remove platform
            if let Some(idx) = app_config
                .platforms
                .iter()
                .position(|p| p.name == self.delete_platform_name)
            {
                let platform = app_config.platforms.remove(idx);

                // Save config
                match app_config.save(&config_path) {
                    Ok(_) => {
                        self.load_error = None;

                        // Record audit event
                        match audit::push_gui(
                            "system",
                            "desktop",
                            "platform delete",
                            &format!("{} ({} VMs)", self.delete_platform_name, platform.vms.len()),
                        ) {
                            Ok(audit_id) => {
                                eprintln!("✓ Audit record created: ID {}", audit_id);
                            }
                            Err(e) => {
                                eprintln!("⚠ Failed to record audit event: {}", e);
                            }
                        }

                        eprintln!("✓ Platform deleted, refreshing spreadsheet");
                        // Refresh the list
                        self.loaded = false;
                    }
                    Err(e) => {
                        self.load_error = Some(format!("Failed to save config: {}", e));
                    }
                }
            } else {
                self.load_error = Some(format!(
                    "Platform '{}' not found",
                    self.delete_platform_name
                ));
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn start_add_platform_oauth(&mut self) {
        use crate::api::gcp_oauth::OAuthHandler;
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
            use crate::calc::gcp_rest::GcpRestClient;

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
                    eprintln!("Failed to fetch user info: {}", e);
                    self.add_platform_connected_email = Some("Connected Account".to_string());
                }
            }
        }
    }

    fn fetch_billing_data(&mut self) {
        self.billing_loading = true;
        self.billing_error = None;
        self.billing_data = None;

        // Load config to get GCP platform with OAuth
        let (app_config, _) = match load_config() {
            Ok(config) => config,
            Err(e) => {
                self.billing_error = Some(format!("Failed to load config: {}", e));
                self.billing_loading = false;
                return;
            }
        };

        // Find first GCP platform with OAuth token
        let platform = match app_config
            .platforms
            .iter()
            .find(|p| p.platform_type == "gcp" && p.gcp_oauth_access_token.is_some())
        {
            Some(p) => p,
            None => {
                self.billing_error = Some(
                    "No connected GCP platform found. Please add a GCP platform first.".to_string(),
                );
                self.billing_loading = false;
                return;
            }
        };

        // Get access token
        let access_token = match &platform.gcp_oauth_access_token {
            Some(token) => token.clone(),
            None => {
                self.billing_error = Some("No OAuth token found".to_string());
                self.billing_loading = false;
                return;
            }
        };

        // Get project ID from VMs
        let project_id = if !platform.vms.is_empty() {
            platform.vms[0].gcp_project_id.clone()
        } else {
            self.billing_error =
                Some("No VMs found. Please create a VM to determine the project ID.".to_string());
            self.billing_loading = false;
            return;
        };

        // Create API client
        let client = crate::calc::gcp_rest::GcpRestClient::new(access_token);

        // Auto-discover billing table if not already configured
        if self.billing_dataset.is_empty() || self.billing_table.is_empty() {
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

        // Fetch billing data
        match client.get_current_month_billing(
            &project_id,
            &self.billing_dataset,
            &self.billing_table,
        ) {
            Ok(records) => {
                self.billing_data = Some(records);
                self.billing_loading = false;
            }
            Err(e) => {
                self.billing_error = Some(format!(
                    "Failed to fetch billing data: {}\n\nCurrent settings:\n• Project: {}\n• Dataset: {}\n• Table: {}\n\nPlease verify these settings below.",
                    e, project_id, self.billing_dataset, self.billing_table
                ));
                self.billing_loading = false;
            }
        }
    }

    fn render_billing_dialog(&mut self, ctx: &egui::Context) {
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
                    ui.colored_label(egui::Color32::from_rgb(255, 82, 82), "Error:");
                    ui.add_space(4.0);
                    ui.label(error);
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
                                            ui.label(format!("📅 {} ({})", record.month, record.currency));

                                            // Helper to format currency with thousand separators
                                            let format_currency = |amount: f64, currency: &str| -> String {
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
                                                    format!("{} (credit)", format_currency(record.total_net_cost, &record.currency))
                                                )
                                            } else if record.total_net_cost == 0.0 {
                                                (
                                                    egui::Color32::GRAY,
                                                    format!("{} (no charges)", format_currency(0.0, &record.currency))
                                                )
                                            } else {
                                                (
                                                    egui::Color32::from_rgb(255, 200, 87),
                                                    format_currency(record.total_net_cost, &record.currency)
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
                        self.fetch_billing_data();
                    }

                    if ui.add(MaterialButton::outlined("Close")).clicked() {
                        self.show_billing_dialog = false;
                    }
                });
            });
    }
}
