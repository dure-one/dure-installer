# GCP Platform Management UI Improvements

**Date:** 2026-07-07  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved

## Overview

This specification covers four improvements to the GCP platform management UI in the Dure application:

1. **Swap Memory Configuration** - Add user input for swap size in VM creation wizard
2. **Billing Without VMs** - Enable monthly billing dialog to work without requiring VMs
3. **VM Creation Row Refresh** - Auto-refresh platform table row after VM creation completes
4. **Delete via API** - Add checkboxes to actually delete VMs and projects from GCP, not just local config

## Architecture Overview

### Layer Responsibilities

**UI Layer** (`mobile/src/ui_dlg/platform_gcp.rs`, `mobile/src/ui_tabs/platform.rs`)
- Render input fields and dialogs
- Collect user input (swap size, delete options)
- Emit commands to ViewModel
- Handle events to update UI state

**ViewModel Layer** (`mobile/src/viewmodel/platform/`)
- `commands.rs`: Define command variants with new parameters
- `events.rs`: Event definitions (VMCreated already exists)
- `actor.rs`: Implement business logic and orchestrate GCP API calls
- Handle errors and emit appropriate events

**API Layer** (`mobile/src/api/gcp/`)
- GCP REST API calls
- Existing: `delete_instance()` in compute.rs
- New: `delete_project()` in resourcemanager.rs

### Design Principles
- Commands flow down (UI → ViewModel)
- Events flow up (ViewModel → UI)
- All GCP API calls happen in actor layer
- UI layer is stateless for ViewModel operations

## Feature 1: Swap Memory Configuration

### Current Behavior
VM startup script automatically determines swap size based on available memory and disk space (lines 1963-1978 in platform_gcp.rs).

### New Behavior
Add optional user input - if empty, use automatic logic; if specified, use that value.

### Component Changes

#### Wizard State (`mobile/src/ui_dlg/platform_gcp.rs`)

Add field to `GcpWizard`:
```rust
pub struct GcpWizard {
    // ... existing fields ...
    
    /// Optional user-specified swap size in GB (empty = automatic)
    swap_size_gb: String,
}
```

Initialize in `new()` and `with_platform_context()`:
```rust
swap_size_gb: String::new(),
```

#### UI Changes (`render_configure_server()`, after disk size input ~line 875)

```rust
ui.add_space(8.0);

// Swap size input (optional)
ui.horizontal(|ui| {
    ui.label("Swap Size (GB):");
    ui.add(egui::TextEdit::singleline(&mut self.swap_size_gb)
        .desired_width(80.0)
        .hint_text("Auto"));
    ui.colored_label(egui::Color32::GRAY, "Leave empty for automatic (0-8GB)");
});

// Validation
if !self.swap_size_gb.is_empty() {
    if let Err(e) = validate_swap_size(&self.swap_size_gb) {
        ui.colored_label(egui::Color32::from_rgb(245, 101, 101), format!("⚠ {}", e));
    }
}
```

Add validation function:
```rust
fn validate_swap_size(input: &str) -> Result<u32, String> {
    if input.is_empty() {
        return Ok(0); // 0 means auto
    }
    
    let size = input.parse::<u32>()
        .map_err(|_| "Must be a number".to_string())?;
    
    if size > 32 {
        return Err("Maximum 32 GB".to_string());
    }
    
    Ok(size)
}
```

Update "Create Server" button condition:
```rust
let can_create = !self.instance_name.is_empty()
    && self.validate_instance_name(&self.instance_name)
    && !self.selected_region.is_empty()
    && !self.selected_zone.is_empty()
    && !self.selected_machine_type.is_empty()
    && validate_disk_size(&self.disk_size_gb).is_ok()
    && (self.swap_size_gb.is_empty() || validate_swap_size(&self.swap_size_gb).is_ok())
    && self.image_promise.is_none();
```

#### Command Changes (`mobile/src/viewmodel/platform/commands.rs`)

Update `CreateVM` command:
```rust
CreateVM {
    platform_name: String,
    project_id: String,
    zone: String,
    machine_type: String,
    instance_name: String,
    image: String,
    disk_size_gb: u32,
    swap_size_gb: Option<u32>,  // New: None = auto, Some(n) = user specified
},
```

#### Wizard Integration

In `start_server_creation()`, parse swap size:
```rust
let swap_size_gb = if self.swap_size_gb.is_empty() {
    None
} else {
    Some(validate_swap_size(&self.swap_size_gb).unwrap()) // Already validated
};

// ... emit CreateVM command with swap_size_gb
```

#### Actor Logic (`mobile/src/viewmodel/platform/actor.rs`)

In `handle_create_vm()`, modify startup script generation:

```rust
fn build_startup_script(swap_size_gb: Option<u32>) -> String {
    format!(r#"#!/bin/bash
# ... existing script ...

# Swap configuration
if [ -n "{}" ]; then
    # User specified swap size
    SWAP_SIZE_GB={}
    
    if [ $SWAP_SIZE_GB -gt 0 ]; then
        echo "Creating user-specified ${{SWAP_SIZE_GB}}GB swap..."
        # ... swap creation logic ...
    fi
else
    # Automatic swap detection (existing logic)
    TOTAL_MEM_KB=$(grep MemTotal /proc/meminfo | awk '{{print $2}}')
    # ... existing auto-detection logic from lines 1963-1978 ...
fi
"#, 
    if swap_size_gb.is_some() { "set" } else { "" },
    swap_size_gb.unwrap_or(0))
}
```

### Data Flow

```
User enters "4" in swap input
  → Wizard validates (numeric, 0-32 range)
  → "Create Server" clicked
  → Wizard emits CreateVM { swap_size_gb: Some(4), ... }
  → Actor receives command
  → Actor generates startup script with: SWAP_SIZE_GB=4
  → Actor calls GCP Compute API to create instance
  → Actor emits VMCreated event
  → Platform UI refreshes table
```

### Error Handling

- **Invalid input** (non-numeric): Show validation error in wizard, disable "Create Server" button
- **Value out of range**: Show "Maximum 32 GB" validation error
- **Empty input**: No error - passes `None`, uses automatic logic

## Feature 2: Billing Without VMs

### Current Behavior
Billing dialog requires VMs to exist to extract `project_id` from `platform.vms[0].gcp_project_id` (line 2572-2581 in platform.rs).

### New Behavior
Pass `project_id` from the platform table row to the billing dialog, similar to how `platform_name` is passed.

### Component Changes

#### Platform UI Changes (`mobile/src/ui_tabs/platform.rs`)

Modify billing button rendering (~line 1095):

In the operations column, disable the billing button if no project_id is available:
```rust
let billing_button = MaterialButton::outlined("Billing").small();
ui.add_enabled_ui(row.selected_project_id.is_some(), |ui| {
    if ui.add(billing_button)
        .on_hover_text("Estimated Billing")
        .clicked()
    {
        // Store both platform name and project ID
        ui.data_mut(|d| {
            d.insert_temp(egui::Id::new("platform_action_billing_name"), row.platform_name.clone());
            d.insert_temp(egui::Id::new("platform_action_billing_project"), 
                row.selected_project_id.clone().unwrap()); // Safe: checked by is_some()
        });
    }
});
```

Update billing action handler (~line 1228):
```rust
if let Some(platform_name) =
    ui.data(|d| d.get_temp::<String>(egui::Id::new("platform_action_billing_name")))
{
    let project_id = ui.data(|d| d.get_temp::<String>(egui::Id::new("platform_action_billing_project")))
        .expect("project_id must exist if billing button was clicked"); // Guaranteed by button enabled state
    
    self.show_billing_dialog = true;
    self.fetch_billing_data_with_project(vm.as_deref_mut(), project_id);
    ui.data_mut(|d| {
        d.remove::<String>(egui::Id::new("platform_action_billing_name"));
        d.remove::<String>(egui::Id::new("platform_action_billing_project"));
    });
}
```

#### Command Changes (`mobile/src/viewmodel/platform/commands.rs`)

Add new command variant:
```rust
pub enum PlatformCommand {
    // ... existing commands ...
    
    FetchBilling {
        platform_name: String,
        project_id: String,  // Passed from UI row, not extracted from VMs
    },
}
```

#### Platform Tab Method

Rename and modify `fetch_billing_data()` to `fetch_billing_data_with_project()`:
```rust
fn fetch_billing_data_with_project(&mut self, vm: Option<&mut crate::viewmodel::ViewModel>, project_id: String) {
    if let Some(vm) = vm {
        self.billing_loading = true;
        self.billing_error = None;
        self.billing_data = None;

        // Emit command with project_id
        if let Err(e) = vm.fetch_billing(platform_name, project_id) {
            self.billing_error = Some(format!("Failed to start billing fetch: {}", e));
            self.billing_loading = false;
        }
    }
}
```

#### Actor Logic (`mobile/src/viewmodel/platform/actor.rs`)

Add `handle_fetch_billing()` method:
```rust
async fn handle_fetch_billing(&self, platform_name: String, project_id: String) {
    // Load config to get access token (by platform_name)
    let (mut app_config, config_path) = match load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            self.send_error("fetch_billing", format!("Failed to load config: {}", e)).await;
            return;
        }
    };

    // Find platform by name to get access token
    let platform_idx = match app_config.platforms.iter().enumerate()
        .find(|(_, p)| p.name == platform_name && p.platform_type == "gcp")
        .map(|(idx, _)| idx)
    {
        Some(idx) => idx,
        None => {
            self.send_error("fetch_billing", format!("Platform '{}' not found", platform_name)).await;
            return;
        }
    };

    // Get valid access token (with refresh if needed)
    let access_token = match self.get_valid_access_token(&mut app_config, platform_idx, &config_path).await {
        Ok(token) => token,
        Err(e) => {
            self.send_error("fetch_billing", format!("OAuth error: {}", e)).await;
            return;
        }
    };

    // Use provided project_id (not from VMs)
    let client = GcpRestClient::new(access_token);

    // Auto-discover billing table
    let (dataset, table) = match client.discover_billing_table(&project_id) {
        Ok(result) => result,
        Err(e) => {
            // Fall back to defaults
            let dataset = "billing_export".to_string();
            let table = format!("gcp_billing_export_v1_{}", project_id.replace('-', "_"));
            self.send_progress("fetch_billing", 0.3, &format!("Using default names (discovery failed: {})", e)).await;
            (dataset, table)
        }
    };

    // Fetch billing records
    match client.fetch_billing_records(&project_id, &dataset, &table) {
        Ok(records) => {
            self.send_event(PlatformEvent::BillingFetched {
                platform_name,
                records,
            }).await;
        }
        Err(e) => {
            self.send_error("fetch_billing", format!("BigQuery error: {}", e)).await;
        }
    }
}
```

### Data Flow

```
User clicks "Billing" button on row with project_id="my-project-123"
  → Platform UI stores to egui temp data: (platform_name, project_id)
  → Platform UI emits FetchBilling { platform_name, project_id: "my-project-123" }
  → Actor loads access token from config (by platform_name)
  → Actor refreshes token if expired
  → Actor calls BigQuery API with project_id
  → Actor emits BillingFetched { records } event
  → Platform UI displays billing dialog with data
```

### Error Handling

- **No project_id in row**: Billing button is disabled (grayed out, not clickable)
- **Access token expired**: Actor refreshes token automatically
- **Token refresh fails**: Emit Error event → UI shows "OAuth token invalid, please reconnect"
- **BigQuery API fails**: Emit Error event → UI shows billing_error message

## Feature 3: VM Creation Row Refresh

### Current Behavior
Platform table row is not refreshed after VM creation completes. The wizard closure detection (lines 1287-1291) triggers a refresh, but only when the wizard is closed, not when VM creation finishes.

### New Behavior
Handle `VMCreated` event to immediately refresh the platform table row.

### Component Changes

#### Platform UI Event Handler (`mobile/src/ui_tabs/platform.rs`, ~line 730)

Add event handler case in the existing match statement:

```rust
match event {
    // ... existing handlers ...
    
    ViewModelEvent::Platform(PlatformEvent::VMCreated { platform_name, vm_name, external_ip }) => {
        eprintln!("✓ VM '{}' created successfully with IP {}", vm_name, external_ip);
        // Refresh to show updated VM details
        self.loaded = false;
        self.load_error = None;
    }
    
    // ... other handlers ...
}
```

### Data Flow

```
VM creation completes in actor
  → Actor saves VM to config
  → Actor emits VMCreated { platform_name, vm_name, external_ip }
  → Platform UI event loop receives VMCreated
  → Sets self.loaded = false
  → Next UI frame calls load_rows()
  → Table refreshed with new VM data (row shows VM name, IP, SSH status)
```

### Error Handling

No new error cases - reuses existing event handling error patterns.

## Feature 4: Delete via API

### Current Behavior
Delete platform dialog only removes platform from local config. Warning message states "VMs will be removed from config but NOT deleted from GCP" (line 2339 in platform.rs).

### New Behavior
Add two independent checkboxes to let users choose:
1. Delete VMs from GCP
2. Delete Project from GCP

Attempt selected deletions via API and show detailed error messages if failures occur.

### Component Changes

#### Platform UI State (`mobile/src/ui_tabs/platform.rs`)

Add fields to `PlatformTab`:
```rust
pub struct PlatformTab {
    // ... existing fields ...
    
    /// Delete platform dialog - checkbox states
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_platform_delete_vms: bool,
    
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_platform_delete_project: bool,
}
```

Initialize in `default()`:
```rust
delete_platform_delete_vms: false,
delete_platform_delete_project: false,
```

Reset in `show_delete_platform_confirmation()`:
```rust
self.delete_platform_delete_vms = false;
self.delete_platform_delete_project = false;
```

#### Dialog UI Changes (`render_delete_platform_dialog()`, ~line 2307)

Add checkboxes before the action buttons:
```rust
ui.add_space(12.0);
ui.separator();
ui.add_space(8.0);

ui.label("Additional Actions:");
ui.add_space(4.0);

ui.checkbox(&mut self.delete_platform_delete_vms, "Delete VMs from GCP");
if self.delete_platform_vm_count > 0 {
    ui.colored_label(
        egui::Color32::from_rgb(255, 152, 0),
        format!("  ⚠ Will delete {} VM(s) from Google Cloud", self.delete_platform_vm_count)
    );
} else {
    ui.colored_label(egui::Color32::GRAY, "  (No VMs to delete)");
}

ui.add_space(4.0);
ui.checkbox(&mut self.delete_platform_delete_project, "Delete Project from GCP");
ui.colored_label(
    egui::Color32::from_rgb(245, 101, 101),
    "  ⚠ WARNING: This will permanently delete the entire GCP project!"
);

ui.add_space(12.0);
```

Update delete button handler:
```rust
if ui.add(MaterialButton::filled("Yes, Delete Platform")).clicked() {
    self.execute_delete_platform(vm.as_deref_mut());
    self.show_delete_platform_dialog = false;
}
```

#### Command Changes (`mobile/src/viewmodel/platform/commands.rs`)

Add struct for delete options:
```rust
#[derive(Debug, Clone)]
pub struct DeleteOptions {
    pub delete_vms_from_gcp: bool,
    pub delete_project_from_gcp: bool,
}
```

Update command:
```rust
pub enum PlatformCommand {
    // ... existing commands ...
    
    DeletePlatform {
        platform_name: String,
        options: DeleteOptions,
    },
}
```

#### Platform Tab Method (`execute_delete_platform()`)

Update to emit command with options:
```rust
fn execute_delete_platform(&mut self, vm: Option<&mut crate::viewmodel::ViewModel>) {
    if let Some(vm) = vm {
        let options = DeleteOptions {
            delete_vms_from_gcp: self.delete_platform_delete_vms,
            delete_project_from_gcp: self.delete_platform_delete_project,
        };
        
        match vm.delete_platform(self.delete_platform_name.clone(), options) {
            Ok(_) => {
                eprintln!("✓ Platform delete command sent");
            }
            Err(e) => {
                self.load_error = Some(format!("Failed to delete platform: {}", e));
            }
        }
    }
}
```

#### Actor Logic (`mobile/src/viewmodel/platform/actor.rs`)

Update `handle_delete_platform()`:
```rust
async fn handle_delete_platform(&self, platform_name: String, options: DeleteOptions) {
    // Load config to get platform details
    let (mut app_config, config_path) = match load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            self.send_error("delete_platform", format!("Failed to load config: {}", e)).await;
            return;
        }
    };

    // Find platform
    let platform_idx = match app_config.platforms.iter().enumerate()
        .find(|(_, p)| p.name == platform_name)
        .map(|(idx, _)| idx)
    {
        Some(idx) => idx,
        None => {
            self.send_error("delete_platform", format!("Platform '{}' not found", platform_name)).await;
            return;
        }
    };

    let platform = &app_config.platforms[platform_idx];
    let vm_count = platform.vms.len();
    let project_id = platform.gcp_selected_project_id.clone();

    let mut errors = Vec::new();
    let mut vms_deleted = 0;
    let mut project_deleted = false;

    // Delete VMs from GCP if requested
    if options.delete_vms_from_gcp && !platform.vms.is_empty() {
        // Get access token
        let access_token = match self.get_valid_access_token(&mut app_config, platform_idx, &config_path).await {
            Ok(token) => token,
            Err(e) => {
                errors.push(format!("Cannot delete VMs: OAuth error: {}", e));
                access_token = None; // Skip VM deletion
            }
        };

        if let Some(token) = access_token {
            let client = GcpRestClient::new(token);
            
            for vm in &platform.vms {
                if let Some(project_id) = &vm.gcp_project_id {
                    self.send_progress("delete_platform", 
                        (vms_deleted as f32) / (platform.vms.len() as f32), 
                        &format!("Deleting VM '{}'...", vm.name)
                    ).await;

                    match client.delete_instance(project_id, &vm.zone, &vm.name) {
                        Ok(_) => {
                            eprintln!("✓ Deleted VM '{}' from GCP", vm.name);
                            vms_deleted += 1;
                        }
                        Err(e) => {
                            errors.push(format!("VM '{}': {}", vm.name, e));
                        }
                    }
                }
            }
        }
    }

    // Delete project from GCP if requested
    if options.delete_project_from_gcp {
        if let Some(project_id) = project_id {
            // Get access token (refresh if needed after VM deletions)
            let access_token = match self.get_valid_access_token(&mut app_config, platform_idx, &config_path).await {
                Ok(token) => token,
                Err(e) => {
                    errors.push(format!("Cannot delete project: OAuth error: {}", e));
                    None
                }
            };

            if let Some(token) = access_token {
                let client = GcpRestClient::new(token);
                
                self.send_progress("delete_platform", 0.9, "Deleting project...").await;
                
                match client.delete_project(&project_id) {
                    Ok(_) => {
                        eprintln!("✓ Deleted project '{}' from GCP", project_id);
                        project_deleted = true;
                    }
                    Err(e) => {
                        errors.push(format!("Project deletion failed: {}", e));
                    }
                }
            }
        } else {
            errors.push("Cannot delete project: no project ID in config".to_string());
        }
    }

    // Remove from config (always happens)
    app_config.platforms.remove(platform_idx);
    
    if let Err(e) = app_config.save(&config_path) {
        self.send_error("delete_platform", format!("Failed to save config: {}", e)).await;
        return;
    }

    // Emit success event with details
    if errors.is_empty() {
        self.send_event(PlatformEvent::PlatformDeleted {
            platform_name,
            vm_count,
        }).await;
    } else {
        // Partial success - send error with summary
        let summary = format!(
            "Deleted from config. GCP operations: {} of {} VMs deleted{}, {}. Errors: {}",
            vms_deleted,
            vm_count,
            if options.delete_vms_from_gcp { "" } else { " (not requested)" },
            if project_deleted { "project deleted" } else if options.delete_project_from_gcp { "project deletion failed" } else { "project not deleted (not requested)" },
            errors.join("; ")
        );
        
        self.send_error("delete_platform", summary).await;
        
        // Still emit PlatformDeleted since it's removed from config
        self.send_event(PlatformEvent::PlatformDeleted {
            platform_name,
            vm_count,
        }).await;
    }
}
```

#### GCP API Changes (`mobile/src/api/gcp/resourcemanager.rs`)

Add delete_project method to `GcpRestClient`:
```rust
impl GcpRestClient {
    // ... existing methods ...
    
    /// Delete a GCP project
    /// 
    /// This marks the project for deletion. The project will be deleted after a 30-day waiting period.
    /// https://cloud.google.com/resource-manager/reference/rest/v1/projects/delete
    pub fn delete_project(&self, project_id: &str) -> anyhow::Result<()> {
        let url = format!(
            "https://cloudresourcemanager.googleapis.com/v1/projects/{}",
            project_id
        );

        let response = ureq::delete(&url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .call()?;

        if response.status() == 200 {
            Ok(())
        } else {
            let error_body = response.into_string()?;
            Err(anyhow::anyhow!("Failed to delete project: {}", error_body))
        }
    }
}
```

### Data Flow

**Happy Path (both checkboxes selected):**
```
User opens delete dialog for platform with 2 VMs
  → User checks "Delete VMs from GCP"
  → User checks "Delete Project from GCP"
  → User clicks "Yes, Delete Platform"
  → Platform UI emits DeletePlatform { 
      platform_name, 
      options: { delete_vms_from_gcp: true, delete_project_from_gcp: true }
    }
  → Actor loads config (VMs: [vm1, vm2], project_id: "proj-123")
  → Actor calls client.delete_instance("proj-123", "vm1", "zone-a")
  → Success → Actor emits Progress { "Deleting VM 'vm2'..." }
  → Actor calls client.delete_instance("proj-123", "vm2", "zone-b")
  → Success → Actor emits Progress { "Deleting project..." }
  → Actor calls client.delete_project("proj-123")
  → Success → Actor removes platform from config, saves
  → Actor emits PlatformDeleted { platform_name, vm_count: 2 }
  → Platform UI refreshes table
```

### Error Handling

**Strategy:** Best-effort deletion - attempt all selected operations even if some fail.

- **VM deletion fails**: Continue with remaining VMs, collect errors
- **Project deletion fails** (has other resources like Cloud Storage): Include GCP error message in summary
- **No access token**: Skip GCP deletions, emit error, still remove from config
- **Partial success**: Emit Error event with detailed summary, still emit PlatformDeleted

**Error Message Format:**
```
Deleted from config. GCP operations: 2 of 3 VMs deleted, project deletion failed. 
Errors: VM 'server-3': Instance not found; Project deletion failed: Project has active Cloud Storage buckets
```

**Why Remove from Config Even on Errors:**
User initiated the delete action. Even if GCP API calls fail, the local config should be cleaned up. User can manually check GCP Console if needed.

## Testing Strategy

### Unit Tests

**Wizard Tests** (`mobile/src/ui_dlg/platform_gcp.rs`)
- `test_swap_validation_valid()`: Test valid inputs (0, 4, 16, 32)
- `test_swap_validation_invalid()`: Test invalid inputs ("", "abc", "-1", "99")
- `test_swap_none_when_empty()`: Verify `swap_size_gb: None` when field is empty
- `test_swap_some_when_specified()`: Verify `swap_size_gb: Some(8)` when "8" entered

**ViewModel Tests** (`mobile/src/viewmodel/platform/tests.rs`)
- `test_create_vm_with_swap()`: Verify startup script contains `SWAP_SIZE_GB=4` when Some(4) passed
- `test_create_vm_auto_swap()`: Verify startup script contains auto-detection logic when None passed
- `test_fetch_billing_with_project_id()`: Verify BigQuery called with provided project_id (not extracted from VMs)
- `test_delete_platform_vms_only()`: Verify only delete_instance called when delete_vms=true, delete_project=false
- `test_delete_platform_project_only()`: Verify only delete_project called when delete_vms=false, delete_project=true
- `test_delete_platform_both()`: Verify both APIs called in sequence
- `test_delete_platform_partial_failure()`: Verify error event includes details when VM deletion fails but continues
- `test_vm_created_event_emitted()`: Verify VMCreated event emitted after successful VM creation

**GCP API Tests** (`mobile/tests/gcp_common_tests.rs`)
- `test_delete_project_api_request()`: Mock GCP API, verify DELETE request format to Resource Manager API
- Reuse existing OAuth mock patterns for token refresh scenarios

### Integration Tests

**Platform Tab Tests** (`mobile/src/ui_tabs/platform.rs`)
- `test_vm_created_event_refreshes_table()`: Verify `self.loaded = false` triggered on VMCreated event
- `test_billing_button_passes_project_id()`: Verify FetchBilling command contains correct project_id from row

### Manual Testing Checklist

**Swap Memory:**
- [ ] Create VM with empty swap field → SSH to VM, verify auto-detection logic ran (check `/var/log/syslog`)
- [ ] Create VM with swap=4 → SSH to VM, verify 4GB swap file exists (`swapon --show`)
- [ ] Try invalid inputs (abc, -1, 99) → verify validation errors shown, Create button disabled
- [ ] Create VM with swap=0 → verify no swap created

**Billing Dialog:**
- [ ] Create platform with project but no VMs → click Billing → verify dialog loads with data
- [ ] Verify BigQuery query uses correct project_id (check browser console or logs)
- [ ] Test with platform that has no project_id → verify billing button is disabled or shows error

**VM Refresh:**
- [ ] Create VM → verify row updates from "Add VM" button active to showing VM details (name, IP)
- [ ] Verify external IP and SSH status appear in row after creation completes
- [ ] Verify row updates before closing wizard (not waiting for wizard closure)

**Deletion:**
- [ ] Delete platform with no checkboxes → verify only config updated, GCP resources untouched
- [ ] Delete platform with delete_vms=true → verify VMs deleted from GCP Console
- [ ] Delete platform with delete_project=true → verify project marked for deletion in GCP Console
- [ ] Delete platform with both checkboxes → verify both operations attempted
- [ ] Test error case: delete project with non-VM resources (e.g., Cloud Storage bucket) → verify error message shown with GCP details
- [ ] Test error case: delete VM that doesn't exist → verify other VMs still deleted, error message shows which failed

## Implementation Notes

### File Changes Summary

**New Files:**
- None (all changes in existing files)

**Modified Files:**
1. `mobile/src/ui_dlg/platform_gcp.rs` - Add swap input UI and validation
2. `mobile/src/ui_tabs/platform.rs` - Add VMCreated handler, update billing button, update delete dialog
3. `mobile/src/viewmodel/platform/commands.rs` - Add swap_size_gb to CreateVM, add FetchBilling command, update DeletePlatform
4. `mobile/src/viewmodel/platform/actor.rs` - Update handlers for all four features
5. `mobile/src/api/gcp/resourcemanager.rs` - Add delete_project() method

### Dependencies

No new dependencies required. All features use existing crates:
- `ureq` for GCP REST API calls
- `egui` for UI components
- `serde` for command/event serialization

### Backward Compatibility

**Config Format:**
- No changes to config schema
- `swap_size_gb: None` in CreateVM command is backward compatible (default behavior)

**API Changes:**
- New command variants are additive (existing code unaffected)
- New event handler is additive (existing handlers unchanged)

### Security Considerations

**Delete Operations:**
- User must explicitly check checkboxes to delete from GCP (opt-in)
- Warning messages clearly indicate permanent deletion
- OAuth token required for GCP API calls (existing auth flow)

**Billing:**
- Project ID comes from authenticated user's config (no privilege escalation)
- BigQuery access controlled by existing OAuth scopes

### Performance Considerations

**Delete Operations:**
- Sequential VM deletions (could be parallelized in future, but simpler error handling with sequential)
- Each VM deletion typically takes 30-60 seconds
- Progress events emitted for user feedback

**Billing:**
- No longer requires VMs, so faster initial load
- BigQuery query performance unchanged

### Future Enhancements

Potential improvements not included in this spec:
- Parallel VM deletion with `futures::join_all()`
- Swap size suggestions based on selected machine type
- Billing date range selector
- Dry-run mode for deletions (show what would be deleted)
- Undo for platform deletion (restore from trash)

## Acceptance Criteria

1. **Swap Memory**
   - [ ] Swap input field appears in VM creation wizard after disk size
   - [ ] Empty input uses automatic swap detection (existing behavior)
   - [ ] User-specified swap size (0-32GB) is applied to VM
   - [ ] Invalid inputs show validation errors and disable Create button
   - [ ] Created VMs have correct swap configuration (verified via SSH)

2. **Billing Without VMs**
   - [ ] Billing button works on platforms with project but no VMs
   - [ ] Billing dialog loads data using project_id from platform config
   - [ ] Billing button is disabled if no project_id is available

3. **VM Creation Refresh**
   - [ ] Platform table row refreshes immediately when VM creation completes
   - [ ] Row shows VM name, external IP, and SSH status after refresh
   - [ ] Refresh happens before wizard is closed

4. **Delete via API**
   - [ ] Delete dialog shows two checkboxes: "Delete VMs from GCP" and "Delete Project from GCP"
   - [ ] Checkboxes default to unchecked (opt-in)
   - [ ] Checking "Delete VMs" actually deletes VMs from GCP
   - [ ] Checking "Delete Project" actually marks project for deletion in GCP
   - [ ] Partial failures show detailed error messages
   - [ ] Platform always removed from config regardless of GCP API results
   - [ ] Success/error messages clearly indicate what succeeded and what failed

## References

- Existing VM creation flow: `mobile/src/ui_dlg/platform_gcp.rs`
- ViewModel architecture: `mobile/src/viewmodel/platform/`
- GCP API client: `mobile/src/api/gcp/`
- Platform management UI: `mobile/src/ui_tabs/platform.rs`
- GCP Resource Manager API: https://cloud.google.com/resource-manager/reference/rest/v1/projects/delete
- GCP Compute Engine API: https://cloud.google.com/compute/docs/reference/rest/v1/instances/delete
