# GCP Platform/Project Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor GCP platform management to use project ID as identifier, add status caching, support project creation, and improve OAuth UX.

**Architecture:** Clean break migration from platform_name to project_id-based identification, with cached status stored in config and manual refresh via UI button.

**Tech Stack:** Rust, egui, serde_yaml, poll-promise, GCP REST APIs

## Global Constraints

- Rust nightly toolchain required
- No OpenSSL dependency (use rustls/ring)
- All UI changes in `#[cfg(feature = "gui")]` blocks
- YAML config format preserved (serde serialization)
- Backward compatibility via one-time automatic migration
- TDD: Write test first, verify fail, implement, verify pass, commit

---

### Task 1: Update Config Schema

**Files:**
- Modify: `mobile/src/config.rs:27-56` (CloudPlatformConfig struct)
- Test: Manual verification (config is serialization-focused, will test in Task 2)

**Interfaces:**
- Consumes: Nothing (first task)
- Produces: `CloudPlatformConfig` without `name` field, with cache fields (`cached_total_project_count: Option<usize>`, `cached_vm_status: Option<String>`, `cached_firewall_status: Option<String>`, `cached_vm_external_ip: Option<String>`, `last_status_refresh: Option<i64>`)

- [ ] **Step 1: Remove name field from CloudPlatformConfig**

Edit `mobile/src/config.rs`, find `CloudPlatformConfig` struct (around line 28), remove:
```rust
pub name: String,
```

- [ ] **Step 2: Add cache fields to CloudPlatformConfig**

In same struct, after `vms: Vec<VmInstance>`, add:
```rust
// Status cache fields (optional to avoid cluttering YAML when empty)
#[serde(default, skip_serializing_if = "Option::is_none")]
pub cached_total_project_count: Option<usize>,

#[serde(default, skip_serializing_if = "Option::is_none")]
pub cached_vm_status: Option<String>,

#[serde(default, skip_serializing_if = "Option::is_none")]
pub cached_firewall_status: Option<String>,

#[serde(default, skip_serializing_if = "Option::is_none")]
pub cached_vm_external_ip: Option<String>,

#[serde(default, skip_serializing_if = "Option::is_none")]
pub last_status_refresh: Option<i64>,
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check --bin dure-desktop`
Expected: Compilation errors in files referencing `platform.name` - expected, will fix in later tasks

- [ ] **Step 4: Commit schema changes**

```bash
git add mobile/src/config.rs
git commit -m "refactor(config): remove platform.name, add status cache fields

- Remove name field from CloudPlatformConfig
- Add cached_total_project_count, cached_vm_status, cached_firewall_status
- Add cached_vm_external_ip, last_status_refresh
- All cache fields optional to avoid YAML clutter

Breaking change: platforms now identified by gcp_selected_project_id

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 2: Add Config Migration Logic

**Files:**
- Modify: `mobile/src/config.rs:1-100` (add V1 structs and migration)
- Create: `mobile/src/config_migration.rs` (migration logic module)
- Modify: `mobile/src/lib.rs` (add config_migration module)

**Interfaces:**
- Consumes: `CloudPlatformConfig` from Task 1 (without `name`, with cache fields)
- Produces: `migrate_config_v1_to_v2(v1: AppConfigV1) -> Result<AppConfig, String>`, `CloudPlatformConfigV1` struct with `name` field

- [ ] **Step 1: Create config_migration module**

Create `mobile/src/config_migration.rs`:
```rust
//! Configuration migration from V1 (with platform.name) to V2 (project_id-based)

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::config::{AppConfig, CloudPlatformConfig, VmInstance};

/// Legacy CloudPlatformConfig with 'name' field (V1)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudPlatformConfigV1 {
    pub name: String,
    pub platform_type: String,
    
    // GCP specific
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gcp_oauth_access_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gcp_oauth_refresh_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gcp_oauth_token_expiry: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gcp_connected_email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gcp_selected_project_id: Option<String>,
    
    // Firebase specific
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firebase_project_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub firebase_api_key: Option<String>,
    
    // Supabase specific
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supabase_project_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supabase_api_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supabase_anon_key: Option<String>,
    
    // Common fields
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_token: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_account_json: Option<String>,
    
    // VM instances (for GCP)
    #[serde(default)]
    pub vms: Vec<VmInstance>,
}

/// Legacy AppConfig with V1 platforms
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfigV1 {
    #[serde(default)]
    pub platforms: Vec<CloudPlatformConfigV1>,
}

/// Migrate V1 platform to V2
fn migrate_platform_v1_to_v2(v1: CloudPlatformConfigV1) -> Option<CloudPlatformConfig> {
    // Only migrate platforms with valid gcp_selected_project_id
    if v1.platform_type == "gcp" && v1.gcp_selected_project_id.is_none() {
        eprintln!("⚠ Skipping platform '{}': no gcp_selected_project_id", v1.name);
        return None;
    }
    
    Some(CloudPlatformConfig {
        platform_type: v1.platform_type,
        gcp_oauth_access_token: v1.gcp_oauth_access_token,
        gcp_oauth_refresh_token: v1.gcp_oauth_refresh_token,
        gcp_oauth_token_expiry: v1.gcp_oauth_token_expiry,
        gcp_connected_email: v1.gcp_connected_email,
        gcp_selected_project_id: v1.gcp_selected_project_id,
        firebase_project_id: v1.firebase_project_id,
        firebase_api_key: v1.firebase_api_key,
        supabase_project_ref: v1.supabase_project_ref,
        supabase_api_url: v1.supabase_api_url,
        supabase_anon_key: v1.supabase_anon_key,
        api_token: v1.api_token,
        service_account_json: v1.service_account_json,
        vms: v1.vms,
        // New cache fields - start empty
        cached_total_project_count: None,
        cached_vm_status: None,
        cached_firewall_status: None,
        cached_vm_external_ip: None,
        last_status_refresh: None,
    })
}

/// Migrate entire config from V1 to V2
pub fn migrate_config_v1_to_v2(v1: AppConfigV1) -> Result<AppConfig, String> {
    let mut migrated_platforms = Vec::new();
    let mut skipped_count = 0;
    
    for platform in v1.platforms {
        if let Some(migrated) = migrate_platform_v1_to_v2(platform) {
            migrated_platforms.push(migrated);
        } else {
            skipped_count += 1;
        }
    }
    
    if skipped_count > 0 {
        eprintln!("⚠ Migration: Skipped {} invalid platform(s)", skipped_count);
    }
    
    Ok(AppConfig {
        platforms: migrated_platforms,
    })
}

/// Create backup of config file
pub fn backup_config(config_path: &Path) -> Result<(), String> {
    let backup_path = config_path.with_extension("yml.backup");
    std::fs::copy(config_path, &backup_path)
        .map_err(|e| format!("Failed to create backup: {}", e))?;
    eprintln!("✓ Created backup: {}", backup_path.display());
    Ok(())
}

/// Restore config from backup
pub fn restore_from_backup(config_path: &Path) -> Result<(), String> {
    let backup_path = config_path.with_extension("yml.backup");
    std::fs::copy(&backup_path, config_path)
        .map_err(|e| format!("Failed to restore backup: {}", e))?;
    eprintln!("✓ Restored from backup: {}", backup_path.display());
    Ok(())
}
```

- [ ] **Step 2: Add module to lib.rs**

Edit `mobile/src/lib.rs`, add after `pub mod config;`:
```rust
pub mod config_migration;
```

- [ ] **Step 3: Verify compilation**

Run: `cargo check --bin dure-desktop`
Expected: SUCCESS (migration module compiles)

- [ ] **Step 4: Commit migration module**

```bash
git add mobile/src/config_migration.rs mobile/src/lib.rs
git commit -m "feat(config): add V1 to V2 migration logic

- Add CloudPlatformConfigV1 with 'name' field
- Add migrate_config_v1_to_v2 function
- Add backup_config and restore_from_backup helpers
- Skip platforms without gcp_selected_project_id

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 3: Integrate Migration into Config Load

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs:226-240` (load_config function)

**Interfaces:**
- Consumes: `migrate_config_v1_to_v2` from Task 2, `CloudPlatformConfig` from Task 1
- Produces: Updated `load_config() -> Result<(AppConfig, PathBuf), String>` that auto-migrates V1 configs

- [ ] **Step 1: Update load_config function**

Edit `mobile/src/ui_tabs/platform.rs`, find `load_config()` function (around line 235), replace entire function:
```rust
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
                    eprintln!("✓ Detected V1 config, migrating to V2...");
                    
                    // Create backup before migration
                    backup_config(&config_path)?;
                    
                    // Migrate
                    let v2_config = migrate_config_v1_to_v2(v1_config)?;
                    
                    // Save migrated config immediately
                    v2_config.save(&config_path)
                        .map_err(|e| format!("Failed to save migrated config: {}", e))?;
                    
                    eprintln!("✓ Migrated {} platform(s) to V2 format", v2_config.platforms.len());
                    
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
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check --bin dure-desktop`
Expected: SUCCESS

- [ ] **Step 3: Test migration with sample V1 config**

Create test config at `/tmp/test_config_v1.yml`:
```yaml
platforms:
  - name: test-platform
    platform_type: gcp
    gcp_selected_project_id: my-gcp-project
    gcp_connected_email: test@example.com
    vms: []
```

Create test in `mobile/src/config_migration.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_migrate_v1_to_v2() {
        let v1_platform = CloudPlatformConfigV1 {
            name: "test-platform".to_string(),
            platform_type: "gcp".to_string(),
            gcp_selected_project_id: Some("my-gcp-project".to_string()),
            gcp_connected_email: Some("test@example.com".to_string()),
            gcp_oauth_access_token: None,
            gcp_oauth_refresh_token: None,
            gcp_oauth_token_expiry: None,
            firebase_project_id: None,
            firebase_api_key: None,
            supabase_project_ref: None,
            supabase_api_url: None,
            supabase_anon_key: None,
            api_token: None,
            service_account_json: None,
            vms: vec![],
        };
        
        let v1_config = AppConfigV1 {
            platforms: vec![v1_platform],
        };
        
        let v2_config = migrate_config_v1_to_v2(v1_config).unwrap();
        
        assert_eq!(v2_config.platforms.len(), 1);
        assert_eq!(v2_config.platforms[0].platform_type, "gcp");
        assert_eq!(v2_config.platforms[0].gcp_selected_project_id, Some("my-gcp-project".to_string()));
        assert_eq!(v2_config.platforms[0].gcp_connected_email, Some("test@example.com".to_string()));
        assert_eq!(v2_config.platforms[0].cached_total_project_count, None);
    }
    
    #[test]
    fn test_skip_invalid_platform() {
        let v1_platform = CloudPlatformConfigV1 {
            name: "invalid-platform".to_string(),
            platform_type: "gcp".to_string(),
            gcp_selected_project_id: None, // Invalid - no project ID
            gcp_connected_email: None,
            gcp_oauth_access_token: None,
            gcp_oauth_refresh_token: None,
            gcp_oauth_token_expiry: None,
            firebase_project_id: None,
            firebase_api_key: None,
            supabase_project_ref: None,
            supabase_api_url: None,
            supabase_anon_key: None,
            api_token: None,
            service_account_json: None,
            vms: vec![],
        };
        
        let v1_config = AppConfigV1 {
            platforms: vec![v1_platform],
        };
        
        let v2_config = migrate_config_v1_to_v2(v1_config).unwrap();
        
        assert_eq!(v2_config.platforms.len(), 0); // Should skip invalid platform
    }
}
```

- [ ] **Step 4: Run migration tests**

Run: `cargo test --lib config_migration`
Expected: PASS (2 tests)

- [ ] **Step 5: Commit migration integration**

```bash
git add mobile/src/ui_tabs/platform.rs mobile/src/config_migration.rs
git commit -m "feat(platform): integrate config V1 to V2 migration on load

- Auto-detect V1 config and migrate to V2
- Create backup before migration
- Skip invalid platforms (no gcp_selected_project_id)
- Add migration tests

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 4: Update PlatformRow and Table Rendering

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs:13-42` (PlatformRow struct)
- Modify: `mobile/src/ui_tabs/platform.rs:640-700` (drawer content rendering)
- Modify: `mobile/src/ui_tabs/platform.rs:1400-1450` (row population from config)

**Interfaces:**
- Consumes: `CloudPlatformConfig` from Task 1 (with cache fields, without `name`)
- Produces: `PlatformRow` with `project_id: String`, `project_display_name: String`, `last_refresh_time: Option<i64>` instead of `platform_name`

- [ ] **Step 1: Update PlatformRow struct**

Edit `mobile/src/ui_tabs/platform.rs`, find `PlatformRow` struct (around line 14), replace:
```rust
/// Platform row data for data table
#[derive(Clone, Debug)]
struct PlatformRow {
    // Identity (CHANGED: project_id replaces platform_name)
    project_id: String,              // GCP project ID (platform identifier)
    project_display_name: String,    // Display name (may differ from ID)
    platform_type: String,           // "GCP"

    // Connection state flags (for Steps column)
    gcp_connected: bool,    // Has OAuth access token
    project_selected: bool, // Has gcp_selected_project_id
    vm_created: bool,       // vms.len() > 0
    firewall_updated: bool, // Current IP is whitelisted
    ssh_ready: bool,        // VM has external_ip.is_some()

    // Drawer content data
    email: Option<String>,      // Connected Google account
    total_project_count: usize, // Cached from config (not 0!)
    selected_project_id: Option<String>,
    vm_name: Option<String>,            // First VM name
    vm_external_ip: Option<String>,     // First VM external IP (from cache or VM)
    ssh_private_key: Option<String>,    // SSH private key from KeePass
    ssh_public_key: Option<String>,     // Derived SSH public key for verification
    ssh_keyring_domain: Option<String>, // Keyring domain for SSH key
    firewall_status: String,            // Cached from config
    ssh_status: String,                 // "✓ Ready" or "? No external IP"

    // NEW: Status cache metadata
    last_refresh_time: Option<i64>,     // For staleness indicator

    // Action button state
    has_vm: bool,            // Enable/disable VM operation buttons
    vm_zone: Option<String>, // For VM operations (delete, restart, regen)
}
```

- [ ] **Step 2: Update row population from config**

Edit `mobile/src/ui_tabs/platform.rs`, find where rows are populated from platforms (around line 1400), replace the row creation:
```rust
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

    // Action button state
    has_vm: !platform.vms.is_empty(),
    vm_zone: platform.vms.first().map(|vm| vm.zone.clone()),
};
```

- [ ] **Step 3: Update drawer content rendering**

Edit `mobile/src/ui_tabs/platform.rs`, find `render_drawer_content` function (around line 641), update the project display:
```rust
fn render_drawer_content(ui: &mut egui::Ui, row: &PlatformRow) {
    ui.add_space(8.0);

    // Level 1: Email + project count (from cache!)
    if let Some(email) = &row.email {
        ui.label(format!(
            "{} ({} projects in account)",
            email, row.total_project_count
        ));
    } else {
        ui.label("Not connected");
    }

    // Level 2: Selected project (show both display name and ID)
    if let Some(project_id) = &row.selected_project_id {
        ui.label(format!("  └─ Project: {} ({})", row.project_display_name, project_id));

        // NEW: Show staleness indicator
        if let Some(last_refresh) = row.last_refresh_time {
            let elapsed = chrono::Utc::now().timestamp() - last_refresh;
            let time_str = if elapsed < 60 {
                "just now".to_string()
            } else if elapsed < 3600 {
                format!("{} min ago", elapsed / 60)
            } else if elapsed < 86400 {
                format!("{} hours ago", elapsed / 3600)
            } else {
                format!("{} days ago", elapsed / 86400)
            };
            ui.label(format!("        • Last refreshed: {}", time_str));
        } else {
            ui.colored_label(egui::Color32::from_rgb(255, 193, 7), "        • Status never refreshed");
        }

        // Level 3: VM details
        if let Some(vm_name) = &row.vm_name {
            // ... rest of VM rendering unchanged
```

- [ ] **Step 4: Update PlatformAction enum to use project_id**

Find `PlatformAction` enum (around line 45), update variants that used `platform_name`:
```rust
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
```

- [ ] **Step 5: Verify compilation**

Run: `cargo check --bin dure-desktop --features gui`
Expected: Compilation errors in action handlers - will fix next

- [ ] **Step 6: Commit PlatformRow refactor**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "refactor(platform): use project_id instead of platform_name in UI

- Replace platform_name with project_id, project_display_name in PlatformRow
- Add last_refresh_time for staleness indicator
- Use cached_total_project_count instead of hardcoded 0
- Use cached_firewall_status and cached_vm_external_ip
- Show staleness in drawer ('X min ago' or 'Never refreshed')
- Add Refresh action to PlatformAction enum

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 5: Add Refresh Button and Status Update Logic

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs:100-200` (add refresh_promises field to PlatformTab)
- Modify: `mobile/src/ui_tabs/platform.rs:800-900` (add refresh button to action buttons)
- Modify: `mobile/src/ui_tabs/platform.rs:1500-1700` (add execute_refresh method)

**Interfaces:**
- Consumes: `PlatformRow.project_id`, `CloudPlatformConfig` with cache fields
- Produces: `execute_refresh(&mut self, project_id: String)` method, refresh promises polling in `ui()` method

- [ ] **Step 1: Add refresh_promises field to PlatformTab**

Edit `mobile/src/ui_tabs/platform.rs`, find `PlatformTab` struct (around line 62), add field:
```rust
#[cfg_attr(feature = "serde", serde(skip))]
refresh_promises: std::collections::HashMap<String, poll_promise::Promise<Result<(), String>>>,
```

In `impl Default for PlatformTab` (around line 175), initialize:
```rust
refresh_promises: std::collections::HashMap::new(),
```

- [ ] **Step 2: Add execute_refresh method**

Add method to `impl PlatformTab` (after `load_data` method):
```rust
/// Execute status refresh for a platform
#[cfg(not(target_arch = "wasm32"))]
fn execute_refresh(&mut self, project_id: String) {
    use crate::api::gcp::{GcpRestClient, get_current_ip};
    
    let promise = poll_promise::Promise::spawn_thread("refresh_status", move || {
        // Load config
        let (mut config, config_path) = load_config()
            .map_err(|e| format!("Failed to load config: {}", e))?;
        
        // Find platform
        let platform = config.platforms.iter_mut()
            .find(|p| p.gcp_selected_project_id.as_ref() == Some(&project_id))
            .ok_or_else(|| format!("Platform {} not found", project_id))?;
        
        // Get valid access token
        let token = get_valid_access_token(platform)?;
        let client = GcpRestClient::new(token);
        
        // Fetch VM status if VM exists
        if let Some(vm) = platform.vms.first() {
            match client.get_instance(&project_id, &vm.zone, &vm.name) {
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
                    eprintln!("Failed to fetch VM status: {}", e);
                }
            }
        }
        
        // Fetch firewall status
        match get_current_ip() {
            Ok(current_ip) => {
                match client.check_ip_whitelisted(&project_id, &current_ip) {
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
                        eprintln!("Failed to check firewall: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to get current IP: {}", e);
            }
        }
        
        // Fetch total project count
        match client.list_projects(None) {
            Ok(list) => {
                platform.cached_total_project_count = Some(list.projects.len());
            }
            Err(e) => {
                eprintln!("Failed to fetch project count: {}", e);
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

/// Get valid access token, refreshing if expired
fn get_valid_access_token(platform: &mut crate::config::CloudPlatformConfig) -> Result<String, String> {
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
```

- [ ] **Step 3: Poll refresh promises in ui() method**

In `PlatformTab::ui()` method, before rendering the table, add:
```rust
// Poll refresh promises
let mut completed_refreshes = Vec::new();
for (project_id, promise) in &self.refresh_promises {
    if let Some(result) = promise.ready() {
        match result {
            Ok(_) => {
                // Reload data to show fresh status
                self.loaded = false;
            }
            Err(e) => {
                eprintln!("Refresh failed for {}: {}", project_id, e);
                // TODO: Show error toast
            }
        }
        completed_refreshes.push(project_id.clone());
    }
}
for project_id in completed_refreshes {
    self.refresh_promises.remove(&project_id);
}
```

- [ ] **Step 4: Add refresh button to action buttons**

Find where action buttons are rendered in the table (search for "UpdateFirewall"), add refresh button:
```rust
// Refresh button (always available)
if ui.add(
    egui::Button::new("🔄")
        .small()
        .frame(true)
).on_hover_text(
    if let Some(last_refresh) = row.last_refresh_time {
        let elapsed = chrono::Utc::now().timestamp() - last_refresh;
        if elapsed < 60 {
            "Refresh status (updated just now)".to_string()
        } else if elapsed < 3600 {
            format!("Refresh status (updated {} min ago)", elapsed / 60)
        } else {
            format!("Refresh status (updated {} hours ago)", elapsed / 3600)
        }
    } else {
        "Refresh status (never refreshed)".to_string()
    }
).clicked() {
    return Some(PlatformAction::Refresh(row.project_id.clone()));
}
```

- [ ] **Step 5: Handle Refresh action**

Find where `PlatformAction` is handled (search for `match action`), add:
```rust
PlatformAction::Refresh(project_id) => {
    self.execute_refresh(project_id);
}
```

- [ ] **Step 6: Verify compilation**

Run: `cargo check --bin dure-desktop --features gui`
Expected: SUCCESS

- [ ] **Step 7: Commit refresh functionality**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "feat(platform): add manual status refresh button

- Add refresh_promises HashMap to track async refresh operations
- Add execute_refresh method to fetch fresh status from GCP API
- Poll refresh promises in ui() and reload on completion
- Add 🔄 refresh button with staleness tooltip
- Auto-refresh OAuth token if expired before API calls

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 6: Fix Remaining platform_name References

**Files:**
- Modify: All files referencing `platform.name` (found via grep)

**Interfaces:**
- Consumes: `CloudPlatformConfig.gcp_selected_project_id` as identifier
- Produces: Compilation success for all targets

- [ ] **Step 1: Find all platform.name references**

Run: `grep -r "platform\.name\|platform_name" mobile/src --include="*.rs" | grep -v "project_id\|//"`
Expected: List of files still referencing platform.name

- [ ] **Step 2: Fix references in action handlers**

For each `PlatformAction` handler that used `platform_name`, update to use `project_id`.

Example in DeleteVM handler:
```rust
// OLD:
PlatformAction::DeleteVM { platform_name, vm_name, vm_zone } => {
    // Find platform by name
    let platform = config.platforms.iter()
        .find(|p| p.name == platform_name)
        // ...

// NEW:
PlatformAction::DeleteVM { project_id, vm_name, vm_zone } => {
    // Find platform by project_id
    let platform = config.platforms.iter()
        .find(|p| p.gcp_selected_project_id.as_ref() == Some(&project_id))
        // ...
```

Apply same pattern to:
- `UpdateFirewall(project_id)`
- `SelectProject(project_id)`
- `RegenVM(project_id)`
- `RestartVM(project_id)`
- `DeletePlatform(project_id)`

- [ ] **Step 3: Fix references in GcpWizard**

Edit `mobile/src/ui_dlg/platform_gcp.rs`, find constructor (around line 150), update to accept platform config:
```rust
// OLD signature:
pub fn new(platform_name: String) -> Self {

// NEW signature:
pub fn new(platform: &crate::config::CloudPlatformConfig) -> Self {
    let oauth_result = if let (Some(token), Some(refresh), Some(expiry)) = (
        &platform.gcp_oauth_access_token,
        &platform.gcp_oauth_refresh_token,
        platform.gcp_oauth_token_expiry,
    ) {
        Some(crate::api::gcp::oauth::OAuthResult {
            access_token: token.clone(),
            refresh_token: refresh.clone(),
            expires_at: expiry as u64,
        })
    } else {
        None
    };
    
    Self {
        state: WizardState::ConfigureServer,  // Skip to config
        platform_name: platform.gcp_selected_project_id.clone()
            .unwrap_or_else(|| "unknown".to_string()),
        oauth_result,
        selected_project_id: platform.gcp_selected_project_id.clone()
            .unwrap_or_default(),
        // ... rest of fields
    }
}
```

- [ ] **Step 4: Update wizard initialization calls**

Find where `GcpWizard::new()` is called, update to pass platform config instead of name:
```rust
// OLD:
self.gcp_wizard = Some(GcpWizard::new(platform_name));

// NEW:
self.gcp_wizard = Some(GcpWizard::new(&platform));
```

- [ ] **Step 5: Verify full compilation**

Run: `cargo check --bin dure-desktop --features gui`
Expected: SUCCESS (no more platform.name references)

- [ ] **Step 6: Commit reference fixes**

```bash
git add mobile/src/ui_tabs/platform.rs mobile/src/ui_dlg/platform_gcp.rs
git commit -m "refactor: replace all platform.name refs with project_id

- Update all PlatformAction handlers to use project_id
- Update GcpWizard::new to accept platform config reference
- Update wizard initialization to pass platform instead of name
- All platform lookups now use gcp_selected_project_id

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 7: Add OAuth URL Display in Add Platform Dialog

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs:74-90` (add OAuth URL field to PlatformTab)
- Modify: `mobile/src/ui_tabs/platform.rs:900-1200` (render OAuth URL in add platform dialog)

**Interfaces:**
- Consumes: OAuth flow from `mobile/src/api/gcp/oauth.rs`
- Produces: Multiline textbox showing OAuth URL with copy button

- [ ] **Step 1: Add OAuth URL field to PlatformTab**

Edit `mobile/src/ui_tabs/platform.rs`, in `PlatformTab` struct, add:
```rust
// Add dialog state
#[cfg_attr(feature = "serde", serde(skip))]
add_platform_oauth_url: Option<String>,
```

In `impl Default for PlatformTab`, initialize:
```rust
add_platform_oauth_url: None,
```

- [ ] **Step 2: Capture OAuth URL from handler**

Modify OAuth flow initiation. Find where `OAuthHandler` is created and `run_oauth_flow` is called.

Before calling `run_oauth_flow`, build and store the URL:
```rust
let handler = crate::api::gcp::oauth::OAuthHandler::default();

// Build OAuth URL for display
let redirect_uri = "http://localhost:8080/oauth/callback"; // Will be dynamic in actual handler
let state = uuid::Uuid::new_v4().to_string();
let oauth_url = handler.build_auth_url_public(&redirect_uri, &state)
    .unwrap_or_default();

self.add_platform_oauth_url = Some(oauth_url.clone());
```

Note: Need to expose `build_auth_url` as public method in oauth.rs:
```rust
// In mobile/src/api/gcp/oauth.rs
impl OAuthHandler {
    // Make public
    pub fn build_auth_url_public(&self, redirect_uri: &str, state: &str) -> Result<String> {
        self.build_auth_url(redirect_uri, state)
    }
}
```

- [ ] **Step 3: Render OAuth URL in dialog**

Find the Add Platform dialog rendering (search for "add_platform_oauth_promise"), add after OAuth button:
```rust
if let Some(oauth_url) = &self.add_platform_oauth_url {
    ui.add_space(8.0);
    ui.label("Opening browser for authorization...");
    ui.label("If browser doesn't open, copy this URL:");
    ui.add_space(4.0);
    
    let mut url_display = oauth_url.clone();
    ui.add(
        egui::TextEdit::multiline(&mut url_display)
            .desired_rows(3)
            .desired_width(ui.available_width() - 16.0)
            .font(egui::TextStyle::Monospace)
            .interactive(false)  // Read-only
    );
    
    ui.add_space(4.0);
    if ui.button("📋 Copy URL").clicked() {
        ui.output_mut(|o| o.copied_text = oauth_url.clone());
    }
}
```

- [ ] **Step 4: Clear OAuth URL on dialog close**

Find where add platform dialog is closed/reset, add:
```rust
self.add_platform_oauth_url = None;
```

- [ ] **Step 5: Make build_auth_url public in oauth.rs**

Edit `mobile/src/api/gcp/oauth.rs`, find `build_auth_url` method (around line 137), change visibility:
```rust
// OLD:
fn build_auth_url(&self, redirect_uri: &str, state: &str) -> Result<String> {

// NEW:
pub fn build_auth_url(&self, redirect_uri: &str, state: &str) -> Result<String> {
```

- [ ] **Step 6: Verify compilation**

Run: `cargo check --bin dure-desktop --features gui`
Expected: SUCCESS

- [ ] **Step 7: Commit OAuth URL display**

```bash
git add mobile/src/ui_tabs/platform.rs mobile/src/api/gcp/oauth.rs
git commit -m "feat(platform): show OAuth URL in Add Platform dialog

- Add add_platform_oauth_url field to PlatformTab
- Build and display OAuth URL in multiline textbox
- Add copy button for manual paste into browser
- Make build_auth_url public in OAuthHandler

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 8: Add Project Creation UI in Add Platform Dialog

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs:74-90` (add project creation fields)
- Modify: `mobile/src/ui_tabs/platform.rs:1000-1300` (render project creation form)

**Interfaces:**
- Consumes: `GcpRestClient.create_project()` from `mobile/src/api/gcp/resourcemanager.rs:132`
- Produces: Project creation form in Add Platform dialog after OAuth success

- [ ] **Step 1: Add project creation fields to PlatformTab**

Edit `mobile/src/ui_tabs/platform.rs`, in `PlatformTab` struct, add:
```rust
#[cfg_attr(feature = "serde", serde(skip))]
add_platform_creating_project: bool,

#[cfg_attr(feature = "serde", serde(skip))]
add_platform_new_project_id: String,

#[cfg_attr(feature = "serde", serde(skip))]
add_platform_new_project_name: String,

#[cfg_attr(feature = "serde", serde(skip))]
add_platform_create_project_error: Option<String>,

#[cfg_attr(feature = "serde", serde(skip))]
add_platform_create_project_promise: Option<poll_promise::Promise<Result<String, String>>>,
```

In `impl Default for PlatformTab`, initialize:
```rust
add_platform_creating_project: false,
add_platform_new_project_id: String::new(),
add_platform_new_project_name: String::new(),
add_platform_create_project_error: None,
add_platform_create_project_promise: None,
```

- [ ] **Step 2: Add project creation UI after project selection**

Find where project selection ComboBox is rendered, add "Create New Project" option and form:
```rust
// After existing project ComboBox
egui::ComboBox::from_label("Select Project")
    .selected_text(
        self.add_platform_selected_project
            .and_then(|idx| self.add_platform_project_list.get(idx))
            .map(|(id, name)| format!("{} ({})", name, id))
            .unwrap_or_else(|| "Select a project...".to_string())
    )
    .show_ui(ui, |ui| {
        for (idx, (project_id, display_name)) in self.add_platform_project_list.iter().enumerate() {
            let label = format!("{} ({})", display_name, project_id);
            if ui.selectable_label(
                self.add_platform_selected_project == Some(idx),
                label
            ).clicked() {
                self.add_platform_selected_project = Some(idx);
                self.add_platform_creating_project = false;
            }
        }
        
        // Add separator and "Create New" option
        ui.separator();
        if ui.selectable_label(
            self.add_platform_creating_project,
            "➕ Create New Project..."
        ).clicked() {
            self.add_platform_creating_project = true;
            self.add_platform_selected_project = None;
        }
    });

ui.add_space(8.0);

// Show project creation form if selected
if self.add_platform_creating_project {
    ui.separator();
    ui.heading("Create New GCP Project");
    ui.add_space(8.0);
    
    ui.label("Project ID:");
    ui.add_space(4.0);
    
    let project_id_response = ui.add(
        egui::TextEdit::singleline(&mut self.add_platform_new_project_id)
            .hint_text("my-project-123")
            .desired_width(300.0)
    );
    
    // Validate project ID format
    let project_id_valid = if self.add_platform_new_project_id.is_empty() {
        false
    } else {
        let regex = regex::Regex::new(r"^[a-z][a-z0-9-]{4,28}[a-z0-9]$").unwrap();
        regex.is_match(&self.add_platform_new_project_id)
    };
    
    if !project_id_valid && !self.add_platform_new_project_id.is_empty() {
        ui.colored_label(
            egui::Color32::RED,
            "⚠ Invalid: 6-30 chars, lowercase letters, numbers, hyphens only"
        );
    } else {
        ui.label("6-30 characters, lowercase letters, numbers, hyphens");
    }
    
    ui.add_space(8.0);
    ui.label("Display Name (optional):");
    ui.add_space(4.0);
    
    ui.add(
        egui::TextEdit::singleline(&mut self.add_platform_new_project_name)
            .hint_text("My Project")
            .desired_width(300.0)
    );
    
    ui.add_space(8.0);
    
    // Show error if any
    if let Some(error) = &self.add_platform_create_project_error {
        ui.colored_label(egui::Color32::RED, format!("❌ {}", error));
        ui.add_space(4.0);
    }
    
    // Create button
    ui.horizontal(|ui| {
        if ui.add_enabled(
            project_id_valid && self.add_platform_create_project_promise.is_none(),
            egui::Button::new("Create Project")
        ).clicked() {
            self.execute_create_project();
        }
        
        if ui.button("Cancel").clicked() {
            self.add_platform_creating_project = false;
            self.add_platform_new_project_id.clear();
            self.add_platform_new_project_name.clear();
            self.add_platform_create_project_error = None;
        }
    });
    
    // Show progress if creating
    if self.add_platform_create_project_promise.is_some() {
        ui.add_space(4.0);
        ui.spinner();
        ui.label("Creating project...");
    }
}
```

- [ ] **Step 3: Add execute_create_project method**

Add method to `impl PlatformTab`:
```rust
#[cfg(not(target_arch = "wasm32"))]
fn execute_create_project(&mut self) {
    use crate::api::gcp::GcpRestClient;
    
    let project_id = self.add_platform_new_project_id.clone();
    let display_name = if self.add_platform_new_project_name.is_empty() {
        project_id.clone()
    } else {
        self.add_platform_new_project_name.clone()
    };
    
    let access_token = self.add_platform_oauth_result.as_ref()
        .map(|r| r.access_token.clone())
        .unwrap_or_default();
    
    let promise = poll_promise::Promise::spawn_thread("create_project", move || {
        let client = GcpRestClient::new(access_token);
        
        match client.create_project(&project_id, &display_name) {
            Ok(_operation) => {
                // Project created successfully
                Ok(project_id)
            }
            Err(e) => {
                Err(format!("Failed to create project: {}", e))
            }
        }
    });
    
    self.add_platform_create_project_promise = Some(promise);
}

#[cfg(target_arch = "wasm32")]
fn execute_create_project(&mut self) {
    // WASM not supported
}
```

- [ ] **Step 4: Poll create project promise**

In `ui()` method, poll promise:
```rust
// Poll create project promise
if let Some(promise) = &self.add_platform_create_project_promise {
    if let Some(result) = promise.ready() {
        match result {
            Ok(project_id) => {
                // Add created project to list and select it
                self.add_platform_project_list.push((
                    project_id.clone(),
                    self.add_platform_new_project_name.clone()
                ));
                self.add_platform_selected_project = Some(self.add_platform_project_list.len() - 1);
                self.add_platform_creating_project = false;
                self.add_platform_new_project_id.clear();
                self.add_platform_new_project_name.clear();
                self.add_platform_create_project_error = None;
            }
            Err(e) => {
                self.add_platform_create_project_error = Some(e.clone());
            }
        }
        self.add_platform_create_project_promise = None;
    }
}
```

- [ ] **Step 5: Add regex dependency**

Edit `mobile/Cargo.toml`, add to dependencies:
```toml
regex = "1.10"
```

- [ ] **Step 6: Verify compilation**

Run: `cargo check --bin dure-desktop --features gui`
Expected: SUCCESS

- [ ] **Step 7: Commit project creation UI**

```bash
git add mobile/src/ui_tabs/platform.rs mobile/Cargo.toml
git commit -m "feat(platform): add GCP project creation in Add Platform dialog

- Add 'Create New Project' option in project selection dropdown
- Add project creation form with validation (6-30 chars, lowercase, etc)
- Add execute_create_project method calling GCP API
- Poll creation promise and add to project list on success
- Add regex dependency for project ID validation

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 9: Fix VM Wizard "Start Over" Button

**Files:**
- Modify: `mobile/src/ui_dlg/platform_gcp.rs:1347-1360` (Start Over button handler)

**Interfaces:**
- Consumes: `WizardState::ConfigureServer`
- Produces: Fixed "Start Over" behavior that resets to server configuration

- [ ] **Step 1: Fix Start Over button**

Edit `mobile/src/ui_dlg/platform_gcp.rs`, find the "Start Over" button (around line 1348), replace:
```rust
// OLD:
if ui.button("← Start Over").clicked() {
    self.state = WizardState::ConnectAccount;  // ❌ Deleted state!
    self.progress_log.clear();
}

// NEW:
if ui.button("← Start Over").clicked() {
    self.state = WizardState::ConfigureServer;  // ✅ Back to server config
    self.progress_log.clear();
    
    // Reset server configuration fields
    self.selected_region = String::new();
    self.selected_zone = String::new();
    self.selected_machine_type = String::new();
    self.instance_name = String::new();
    self.available_regions = Vec::new();
    self.available_machine_types = Vec::new();
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo check --bin dure-desktop --features gui`
Expected: SUCCESS

- [ ] **Step 3: Commit Start Over fix**

```bash
git add mobile/src/ui_dlg/platform_gcp.rs
git commit -m "fix(wizard): change Start Over to go to ConfigureServer

- Change Start Over from ConnectAccount to ConfigureServer
- ConnectAccount state removed (OAuth done at platform level)
- Reset all server configuration fields on Start Over

Fixes #6 from design spec

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 10: Integration Testing and Final Cleanup

**Files:**
- Test: Manual testing of all features
- Modify: Any final cleanup needed

**Interfaces:**
- Consumes: All previous tasks
- Produces: Fully working refactored platform management

- [ ] **Step 1: Test config migration**

Create test V1 config at `~/.config/dure/config.yml.test`:
```yaml
platforms:
  - name: test-platform-old
    platform_type: gcp
    gcp_selected_project_id: my-test-project
    gcp_connected_email: test@example.com
    vms: []
```

Rename to `config.yml`, run app, verify:
- [ ] Backup created at `config.yml.backup`
- [ ] Config migrated to V2 (no `name` field)
- [ ] Platform shows in table with project ID

- [ ] **Step 2: Test OAuth URL display**

1. Click "Add Platform"
2. Click "Connect with Google"
3. Verify OAuth URL appears in multiline textbox
4. Verify "Copy URL" button works
5. Verify browser opens automatically

- [ ] **Step 3: Test project creation**

1. In Add Platform dialog, after OAuth
2. Select "Create New Project"
3. Enter invalid project ID → verify error shown
4. Enter valid project ID (e.g., `test-proj-123`)
5. Click "Create Project"
6. Verify project appears in dropdown after creation
7. Complete platform creation

- [ ] **Step 4: Test status refresh**

1. Add platform with VM
2. Wait for cache to age (or manually edit config to set old timestamp)
3. Click 🔄 Refresh button
4. Verify:
   - [ ] Spinner shows during refresh
   - [ ] Status updates after refresh
   - [ ] "Last refreshed: X min ago" updates

- [ ] **Step 5: Test VM wizard Start Over**

1. Open VM creation wizard
2. Configure server settings
3. Trigger an error
4. Click "Start Over"
5. Verify goes to ConfigureServer (not error/crash)
6. Verify fields are reset

- [ ] **Step 6: Clean up debug prints**

Search for debug `eprintln!` added during development:
```bash
grep -n "eprintln!" mobile/src/ui_tabs/platform.rs mobile/src/config_migration.rs
```

Convert to `log::info!` or `log::debug!` where appropriate.

- [ ] **Step 7: Final compilation check**

Run all build targets:
```bash
cargo check --bin dure-desktop --features gui
cargo check --bin dure-desktop --no-default-features
cargo test --lib
```

Expected: All SUCCESS

- [ ] **Step 8: Update documentation**

Edit `CLAUDE.md` or relevant docs to mention:
- Platforms now identified by GCP project ID
- Manual status refresh required (no auto-refresh)
- Config migration happens automatically on first load

- [ ] **Step 9: Final commit**

```bash
git add -A
git commit -m "test: verify all platform refactor features working

- Tested config V1 to V2 migration
- Tested OAuth URL display and copy
- Tested project creation with validation
- Tested manual status refresh
- Tested VM wizard Start Over fix
- All integration tests passing

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Spec Coverage Review

Checking each requirement from `docs/superpowers/specs/2026-07-07-gcp-platform-project-refactor-design.md`:

1. ✅ **Remove platform_name field** - Task 1, 4, 6
2. ✅ **Cache VM status, firewall, external IP** - Task 1, 5
3. ✅ **Project creation via API** - Task 8
4. ✅ **OAuth URL in UI** - Task 7
5. ✅ **Fix project counter** - Task 4 (line 1411 fix)
6. ✅ **Fix Start Over button** - Task 9
7. ✅ **Config migration** - Task 2, 3
8. ✅ **Manual refresh button** - Task 5
9. ✅ **Staleness indicator** - Task 4

All spec requirements covered.

## Execution Strategy

**Recommended: Subagent-Driven Development**
- Each task executed by fresh subagent
- Review between tasks
- Fast iteration on failures

**Alternative: Inline Execution**
- Execute tasks sequentially in this session
- Checkpoints for review

Choose execution approach to proceed.
