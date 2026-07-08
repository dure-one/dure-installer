# GCP Platform/Project Management Refactor Design

**Date**: 2026-07-07  
**Status**: Approved  
**Author**: Claude Code  

## Overview

This design refactors the GCP platform management system to establish a 1:1 mapping between platforms and GCP projects, add status caching, support project creation via API, and improve the OAuth user experience.

## Goals

1. Remove the `platform_name` input field and use GCP project ID as the platform identifier
2. Cache VM status, firewall status, and external IP in config; refresh only on user action
3. Add GCP project creation via API in the Add Platform dialog
4. Display OAuth URL in UI (multiline textbox) for manual copy/paste
5. Fix incorrect project counter display with real cached value
6. Fix "Start Over" button in VM wizard to go to ConfigureServer (not ConnectAccount)

## Non-Goals

- Multi-project support per platform (future enhancement)
- Automatic background status refresh (intentionally manual)
- OAuth credential storage changes (keep current keyring approach)

## Design

### 1. Config Schema Changes

**File**: `mobile/src/config.rs`

**Remove from `CloudPlatformConfig`:**
```rust
pub name: String,  // DELETE THIS FIELD
```

**Add to `CloudPlatformConfig`:**
```rust
// Status cache fields
pub cached_total_project_count: Option<usize>,
pub cached_vm_status: Option<String>,          // "RUNNING", "STOPPED", "TERMINATED"
pub cached_firewall_status: Option<String>,    // "✓ Whitelisted (IP)" or "✗ Not whitelisted"
pub cached_vm_external_ip: Option<String>,     // Last known external IP
pub last_status_refresh: Option<i64>,          // Unix timestamp of last refresh
```

**Rationale:**
- `gcp_selected_project_id` becomes the platform identifier (must be `Some` for valid GCP platforms)
- Platform name shown in UI = `gcp_selected_project_id` value
- Cache fields are `Option` so they're omitted from YAML when `None`
- `last_status_refresh` enables staleness indicators in UI ("Last updated: 5 minutes ago")

### 2. Data Migration Strategy

**Approach**: Clean break with automatic migration on first load.

**Migration flow:**

1. **Create legacy config struct** for deserialization:
   ```rust
   #[derive(Deserialize)]
   struct CloudPlatformConfigV1 {
       pub name: String,
       pub gcp_selected_project_id: Option<String>,
       // ... other fields same as current
   }
   ```

2. **Detect and migrate** in `load_config()`:
   ```rust
   // Try to load as V1 (with 'name' field)
   let legacy_config: Result<AppConfigV1> = serde_yaml::from_str(&contents);
   
   if legacy_config.is_ok() {
       // Migration needed
       backup_config(&config_path)?; // → config.yml.backup
       let migrated = migrate_v1_to_v2(legacy_config.unwrap())?;
       migrated.save(&config_path)?;
       show_migration_notice(migrated.platforms.len());
   } else {
       // Load as V2 (current format)
       // ...
   }
   ```

3. **Migration rules**:
   - If `gcp_selected_project_id.is_some()`: Valid platform ✅
   - If `gcp_selected_project_id.is_none()`: Invalid platform, skip with warning ⚠️
   - After migration: Auto-save config.yml immediately
   - Show brief UI notification: "Migrated X platforms to new format"

4. **Failure handling**:
   - Backup `config.yml` → `config.yml.backup` before first write
   - If migration fails: Restore from backup, show error, exit gracefully
   - If only some platforms migrate: Partial success, show list of failed platforms

### 3. Add Platform Dialog UI Changes

**File**: `mobile/src/ui_tabs/platform.rs`

**New state fields:**
```rust
// Add to PlatformTab struct
add_platform_oauth_url: Option<String>,
add_platform_creating_project: bool,
add_platform_new_project_id: String,
add_platform_new_project_name: String,
add_platform_create_project_error: Option<String>,
```

**Flow:**

#### Step 1: OAuth Authentication

Display:
```
┌─────────────────────────────────────┐
│ Connect to Google Cloud             │
│                                     │
│ [Connect with Google] button        │
│                                     │
│ Opening browser for authorization...│
│ If browser doesn't open, copy URL:  │
│ ┌─────────────────────────────────┐ │
│ │ https://accounts.google.com/... │ │
│ │ (multiline TextEdit, 3 rows,    │ │
│ │  monospace font, read-only)     │ │
│ └─────────────────────────────────┘ │
│ [📋 Copy URL]                       │
└─────────────────────────────────────┘
```

Implementation:
```rust
ui.label("Opening browser for authorization...");
ui.label("If browser doesn't open, copy this URL:");

let mut oauth_url_display = oauth_url.clone();
ui.add(
    egui::TextEdit::multiline(&mut oauth_url_display)
        .desired_rows(3)
        .desired_width(f32::INFINITY)
        .font(egui::TextStyle::Monospace)
        .interactive(false)  // Read-only
);

if ui.button("📋 Copy URL").clicked() {
    ui.output_mut(|o| o.copied_text = oauth_url.clone());
}
```

#### Step 2: Project Selection/Creation

After OAuth succeeds:
1. Fetch project list: `client.list_projects(None)`
2. Show ComboBox with existing projects:
   - Display: `project.display_name()`
   - Store: `project.project_id`
3. Add special entry at bottom: **"➕ Create New Project..."**

If "Create New Project" selected, show form:
```
┌─────────────────────────────────────────┐
│ Create New GCP Project                  │
│                                         │
│ Project ID: [________________]          │
│ (6-30 chars, lowercase, numbers, -)    │
│                                         │
│ Display Name: [________________]        │
│ (optional, defaults to Project ID)     │
│                                         │
│ [Create Project] [Cancel]               │
└─────────────────────────────────────────┘
```

**Project ID validation** (live, inline):
- Regex: `^[a-z][a-z0-9-]{4,28}[a-z0-9]$`
- Length: 6-30 characters
- Must start with letter, end with letter or number
- Only lowercase letters, numbers, hyphens

**Project creation flow**:
```rust
let client = GcpRestClient::new(access_token);
match client.create_project(&project_id, &display_name) {
    Ok(operation) => {
        // Operation is async, poll for completion or just proceed
        // Project should be usable immediately for most operations
        self.add_platform_selected_project = Some(project_id);
    }
    Err(e) => {
        self.add_platform_create_project_error = Some(e.to_string());
    }
}
```

#### Step 3: Finalize

```rust
// Create new platform config
let new_platform = CloudPlatformConfig {
    platform_type: "gcp".to_string(),
    gcp_oauth_access_token: Some(oauth_result.access_token),
    gcp_oauth_refresh_token: Some(oauth_result.refresh_token),
    gcp_oauth_token_expiry: Some(oauth_result.expires_at as i64),
    gcp_connected_email: Some(email),
    gcp_selected_project_id: Some(selected_project_id),  // This is the platform identifier
    vms: vec![],
    cached_total_project_count: None,  // Will be fetched on first refresh
    cached_vm_status: None,
    cached_firewall_status: None,
    cached_vm_external_ip: None,
    last_status_refresh: None,
    ..Default::default()
};

// Add to config and save
config.platforms.push(new_platform);
config.save(&config_path)?;

// Refresh platform table
self.loaded = false;
```

### 4. Platform Table UI Changes

**File**: `mobile/src/ui_tabs/platform.rs`

#### Changes to `PlatformRow` struct

**Remove:**
```rust
platform_name: String,  // DELETE
```

**Add:**
```rust
project_id: String,              // GCP project ID (platform identifier)
project_display_name: String,    // Display name (may differ from ID)
last_refresh_time: Option<i64>,  // For staleness indicator
```

#### Table columns

| Column | Current | New |
|--------|---------|-----|
| Name | `platform_name` | `project_display_name` (or `project_id` if no display name) |
| Type | "GCP" | "GCP" (unchanged) |
| Steps | Connection progress | Connection progress (unchanged) |
| Actions | Buttons | **Add "🔄 Refresh" button** |

#### Refresh button

**Icon**: 🔄 (MaterialButton with refresh icon)  
**Tooltip**: 
- "Refresh status (Last updated: X min ago)" if refreshed before
- "Refresh status (Never refreshed)" if never refreshed

**On click**:
1. Show loading spinner
2. Spawn background promise:
   - Fetch VM status via `client.get_instance()`
   - Fetch firewall status via `client.check_ip_whitelisted()`
   - Fetch external IP from VM instance
   - Fetch project count via `client.list_projects()`
3. Update cache fields in config
4. Save config.yml
5. Update table row
6. Set `last_status_refresh = now()`

**Implementation:**
```rust
fn execute_refresh(&mut self, project_id: String) {
    let promise = Promise::spawn_thread("refresh_status", move || {
        let (mut config, config_path) = load_config()?;
        
        let platform = config.platforms.iter_mut()
            .find(|p| p.gcp_selected_project_id.as_ref() == Some(&project_id))
            .ok_or("Platform not found")?;
        
        let token = get_valid_access_token(platform)?;
        let client = GcpRestClient::new(token);
        
        // Fetch status
        if let Some(vm) = platform.vms.first() {
            match client.get_instance(&project_id, &vm.zone, &vm.name) {
                Ok(instance) => {
                    platform.cached_vm_status = Some(instance.status);
                    platform.cached_vm_external_ip = instance.network_interfaces
                        .first()
                        .and_then(|ni| ni.access_configs.first())
                        .and_then(|ac| ac.nat_ip.clone());
                }
                Err(e) => log::warn!("Failed to fetch VM status: {}", e),
            }
        }
        
        // Fetch firewall status
        if let Ok(current_ip) = get_current_ip() {
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
                Err(e) => log::warn!("Failed to check firewall: {}", e),
            }
        }
        
        // Fetch project count
        match client.list_projects(None) {
            Ok(list) => {
                platform.cached_total_project_count = Some(list.projects.len());
            }
            Err(e) => log::warn!("Failed to fetch project count: {}", e),
        }
        
        platform.last_status_refresh = Some(chrono::Utc::now().timestamp());
        
        config.save(&config_path)?;
        Ok(())
    });
    
    self.refresh_promises.insert(project_id, promise);
}
```

#### Drawer content changes

**OLD:**
```rust
ui.label(format!("{} ({} projects total)", email, row.total_project_count));
ui.label(format!("  └─ Project: {}", project_id));
```

**NEW:**
```rust
ui.label(format!("{} ({} projects in account)", email, row.total_project_count));
ui.label(format!("  └─ Project: {} ({})", project_display_name, project_id));

// Add staleness indicator
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
    ui.colored_label(egui::Color32::YELLOW, "        • Status never refreshed");
}
```

#### Fix project counter (line 647, 1411)

**OLD:**
```rust
total_project_count: 0, // Will be fetched in background (BUT NEVER WAS!)
```

**NEW:**
```rust
total_project_count: platform.cached_total_project_count.unwrap_or(0),
```

### 5. VM Creation Wizard Changes

**File**: `mobile/src/ui_dlg/platform_gcp.rs`

#### Remove obsolete states

Delete:
- `WizardState::ConnectAccount` (OAuth already done at platform level)
- Account selection UI (lines 400-487)
- `selected_platform_email` field
- `available_platforms` field

#### Update workflow

**OLD:**
```
ConnectAccount → SelectProject → ConfigureServer → CreatingServer → Complete
```

**NEW:**
```
ConfigureServer → CreatingServer → Complete
```

(SelectProject kept for future enhancement but skipped initially)

#### Constructor changes

**OLD:**
```rust
pub fn new(platform_name: String) -> Self {
    Self {
        state: WizardState::ConnectAccount,
        platform_name,
        // ...
    }
}
```

**NEW:**
```rust
pub fn new(platform: &CloudPlatformConfig) -> Self {
    let oauth_result = if let (Some(token), Some(refresh), Some(expiry)) = (
        &platform.gcp_oauth_access_token,
        &platform.gcp_oauth_refresh_token,
        platform.gcp_oauth_token_expiry,
    ) {
        Some(OAuthResult {
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
        // ...
    }
}
```

#### Fix "Start Over" button (line 1348)

**OLD:**
```rust
if ui.button("← Start Over").clicked() {
    self.state = WizardState::ConnectAccount;  // ❌ Goes to deleted state!
    self.progress_log.clear();
}
```

**NEW:**
```rust
if ui.button("← Start Over").clicked() {
    self.state = WizardState::ConfigureServer;  // ✅ Back to server config
    self.progress_log.clear();
    
    // Reset server configuration fields
    self.selected_region = String::new();
    self.selected_zone = String::new();
    self.selected_machine_type = String::new();
    self.instance_name = String::new();
}
```

#### Token refresh handling

In `render_configure_server()`, check token expiry:
```rust
// Check if OAuth token is expired
if let Some(oauth) = &self.oauth_result {
    let now = chrono::Utc::now().timestamp() as u64;
    if now >= oauth.expires_at.saturating_sub(60) {
        // Token expired, refresh it
        match self.refresh_token_sync(&oauth.refresh_token) {
            Ok(new_oauth) => {
                self.oauth_result = Some(new_oauth);
                // Also update platform config in background
            }
            Err(e) => {
                // Show error, require re-authentication
                self.state = WizardState::Error(format!(
                    "OAuth token expired and refresh failed: {}. Please close and re-authenticate from Platform tab.",
                    e
                ));
            }
        }
    }
}
```

### 6. Status Caching and Refresh

#### Load behavior

**On initial tab load** (`PlatformTab::load_data`):
```rust
// Build row from cached data (NO API calls)
let row = PlatformRow {
    project_id: platform.gcp_selected_project_id.clone().unwrap_or_default(),
    project_display_name: platform.gcp_selected_project_id.clone().unwrap_or_default(),
    // ... other fields from platform struct
    
    // Use cached status
    total_project_count: platform.cached_total_project_count.unwrap_or(0),
    firewall_status: platform.cached_firewall_status.clone()
        .unwrap_or_else(|| "? Not checked".to_string()),
    ssh_status: if platform.cached_vm_status.as_deref() == Some("RUNNING") {
        "? Not tested".to_string()
    } else {
        platform.cached_vm_status.clone().unwrap_or_else(|| "? Unknown".to_string())
    },
    last_refresh_time: platform.last_status_refresh,
};
```

**No background refresh** - User must click 🔄 button explicitly.

#### Refresh promise management

Add to `PlatformTab`:
```rust
#[cfg_attr(feature = "serde", serde(skip))]
refresh_promises: std::collections::HashMap<String, Promise<Result<(), String>>>,
```

Poll promises in `ui()`:
```rust
// Check refresh promises
let mut completed_refreshes = Vec::new();
for (project_id, promise) in &self.refresh_promises {
    if let Some(result) = promise.ready() {
        match result {
            Ok(_) => {
                // Reload data to show fresh status
                self.loaded = false;
            }
            Err(e) => {
                // Show error toast
                log::error!("Refresh failed for {}: {}", project_id, e);
            }
        }
        completed_refreshes.push(project_id.clone());
    }
}
for project_id in completed_refreshes {
    self.refresh_promises.remove(&project_id);
}
```

### 7. Error Handling

#### Migration errors

| Error | Handling |
|-------|----------|
| Config parse failure | Show error dialog with backup path, offer to reset config to defaults |
| Partial migration (some platforms invalid) | Skip invalid platforms, show warning list, save valid ones |
| Save failure after migration | Restore from `config.yml.backup`, show error dialog, exit with instructions |

#### API errors during refresh

| Error | Handling |
|-------|----------|
| OAuth token expired | Auto-refresh using refresh token, retry once |
| OAuth refresh fails | Clear OAuth tokens, show "Re-authenticate required" badge in row |
| API quota exceeded | Show error tooltip, don't update cache, keep stale data |
| Network timeout | Show error tooltip, keep stale data, suggest retry |
| Permission denied | Show error explaining which API/permission is missing |

#### Project creation errors

| Error | Handling |
|-------|----------|
| Project ID already exists | Inline error: "Project ID already in use, choose another" |
| Invalid project ID format | Live validation with regex, prevent submission |
| Quota exceeded | Error: "Project creation quota reached. Delete unused projects or wait 24 hours" |
| Permission denied | Error: "No permission to create projects. Use existing project instead" |

#### Error display patterns

- **Inline errors**: Form validation (project ID format)
- **Toast notifications**: Transient errors (network timeout)
- **Modal dialogs**: Critical errors (config corruption)
- **Status badges**: Persistent errors (expired OAuth in table row)

#### Graceful degradation

- If cache read fails → Fall back to live API calls
- If API calls fail → Show stale data with warning
- If no data at all → Show "No data" placeholder with explanation

## Implementation Notes

### Files to modify

1. **`mobile/src/config.rs`**
   - Remove `name` field from `CloudPlatformConfig`
   - Add cache fields: `cached_total_project_count`, `cached_vm_status`, `cached_firewall_status`, `cached_vm_external_ip`, `last_status_refresh`
   - Add migration logic for V1 → V2 config

2. **`mobile/src/ui_tabs/platform.rs`**
   - Remove `platform_name` from `PlatformRow`
   - Add `project_id`, `project_display_name`, `last_refresh_time` to `PlatformRow`
   - Add Add Platform dialog OAuth URL display
   - Add project creation UI
   - Add Refresh button and promise handling
   - Fix project counter (line 1411)
   - Update drawer content rendering

3. **`mobile/src/ui_dlg/platform_gcp.rs`**
   - Remove `ConnectAccount` state
   - Remove account selection UI (lines 400-487)
   - Update constructor to accept `&CloudPlatformConfig`
   - Fix "Start Over" button (line 1348)
   - Add token refresh logic

4. **`mobile/src/api/gcp/oauth.rs`**
   - No changes needed (create_project already exists in resourcemanager.rs)

5. **`mobile/src/api/gcp/resourcemanager.rs`**
   - Verify `create_project` method exists and works (already implemented at line 132)

### Testing checklist

- [ ] Migration from old config (with `name` field) works correctly
- [ ] Migration creates backup before modifying config
- [ ] Invalid platforms (no `gcp_selected_project_id`) are skipped with warning
- [ ] Add Platform dialog shows OAuth URL in multiline textbox
- [ ] Copy URL button works
- [ ] Project creation validates ID format in real-time
- [ ] Project creation handles API errors gracefully
- [ ] Platform table shows cached status on load (no API calls)
- [ ] Refresh button fetches and caches fresh status
- [ ] Refresh button shows staleness indicator
- [ ] Project counter shows real cached value (not 0)
- [ ] VM wizard starts at ConfigureServer (not ConnectAccount)
- [ ] "Start Over" button goes to ConfigureServer
- [ ] Token refresh works when expired
- [ ] All error cases show appropriate messages

## Risks and Mitigations

| Risk | Mitigation |
|------|------------|
| Migration corrupts existing configs | Create backup before migration, restore on failure |
| Users confused by platform = project change | Show migration notice explaining the change |
| Stale cached data misleads users | Prominent staleness indicator, Refresh button always visible |
| Project creation quota limits users | Clear error message, suggest using existing project |
| OAuth token expires mid-operation | Auto-refresh with fallback to re-auth prompt |

## Future Enhancements

1. **Multi-project platforms**: Allow one platform to manage multiple projects
2. **Auto-refresh on interval**: Background refresh every N minutes (user configurable)
3. **Batch refresh**: "Refresh All" button for all platforms at once
4. **Status history**: Track status changes over time, show graph
5. **Project templates**: Pre-configure common project settings (region, billing account)

## Alternatives Considered

### Alternative 1: Keep platform_name, add project_id as separate field
**Rejected because**: Adds confusion (two names), doesn't solve the core UX issue

### Alternative 2: Don't cache status, always fetch fresh
**Rejected because**: Too slow, wastes API quota, poor offline UX

### Alternative 3: Incremental migration (keep backward compatibility)
**Rejected because**: User chose clean break approach for simplicity

## Success Criteria

- [ ] Users can create platforms without entering platform name
- [ ] Platform table shows project ID/name instead of arbitrary platform name
- [ ] VM status loads instantly from cache on tab open
- [ ] Users can create new GCP projects directly from Add Platform dialog
- [ ] OAuth URL is visible in UI for manual copy/paste
- [ ] Project counter shows accurate cached value
- [ ] "Start Over" button works without errors
- [ ] Existing configs migrate automatically without data loss

## Appendix: Config Schema Comparison

### Old Config (V1)
```yaml
platforms:
  - name: "my-platform-123"        # User-provided, arbitrary
    platform_type: "gcp"
    gcp_oauth_access_token: "..."
    gcp_oauth_refresh_token: "..."
    gcp_oauth_token_expiry: 1234567890
    gcp_connected_email: "user@example.com"
    gcp_selected_project_id: "my-gcp-project"
    vms:
      - name: "vm-1"
        zone: "us-central1-a"
        status: "RUNNING"
        # ...
```

### New Config (V2)
```yaml
platforms:
  - platform_type: "gcp"
    # 'name' field removed - use gcp_selected_project_id as identifier
    gcp_oauth_access_token: "..."
    gcp_oauth_refresh_token: "..."
    gcp_oauth_token_expiry: 1234567890
    gcp_connected_email: "user@example.com"
    gcp_selected_project_id: "my-gcp-project"  # Now used as platform ID
    
    # NEW: Cached status fields
    cached_total_project_count: 5
    cached_vm_status: "RUNNING"
    cached_firewall_status: "✓ Whitelisted (203.0.113.45)"
    cached_vm_external_ip: "203.0.113.45"
    last_status_refresh: 1720346400  # Unix timestamp
    
    vms:
      - name: "vm-1"
        zone: "us-central1-a"
        status: "RUNNING"
        # ...
```

## References

- [GCP Cloud Resource Manager API](https://cloud.google.com/resource-manager/reference/rest)
- [GCP Compute Engine API](https://cloud.google.com/compute/docs/reference/rest/v1)
- [OAuth 2.0 for Mobile & Desktop Apps](https://developers.google.com/identity/protocols/oauth2/native-app)
