# GCP Reserved IP Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable users to manage GCP static IP addresses during VM creation and deletion

**Architecture:** Add GCP Compute addresses API, enhance VM delete dialog to detect and optionally release static IPs, enhance VM add wizard to select from available reserved IPs

**Tech Stack:** Rust, egui, egui-material3, GCP Compute API, serde

**Spec:** `docs/superpowers/specs/2026-09-21-gcp-reserved-ip-management-design.md`

## Global Constraints

- Rust nightly required
- Follow existing GCP API patterns in `mobile/src/api/gcp/`
- Use Material3 components (MaterialButton, ComboBox)
- Fetch IP data on-demand (no caching)
- Default to safe: keep static IPs unless user explicitly releases
- All GCP API calls require OAuth token with `compute` scope

---

## File Structure

**New Files:**
- `mobile/src/api/gcp/compute_addresses.rs` - Address management APIs (list, get, delete)

**Modified Files:**
- `mobile/src/api/gcp/mod.rs` - Add compute_addresses module export
- `mobile/src/api/gcp/compute.rs` - Add `nat_ip` field to AccessConfig
- `mobile/src/ui_tabs/platform.rs` - VM delete dialog state and logic
- `mobile/src/ui_dlg/platform_gcp.rs` - VM add wizard IP selection

---

### Task 1: Create GCP Addresses API Module

**Files:**
- Create: `mobile/src/api/gcp/compute_addresses.rs`
- Modify: `mobile/src/api/gcp/mod.rs`

**Interfaces:**
- Consumes: `GcpRestClient` from `mobile/src/api/gcp/mod.rs`, `GCP_COMPUTE_API_BASE` constant
- Produces: `Address` struct, `AddressList` struct, `list_addresses()`, `get_address()`, `delete_address()` methods on `GcpRestClient`

- [ ] **Step 1: Create addresses module file**

```bash
touch mobile/src/api/gcp/compute_addresses.rs
```

- [ ] **Step 2: Write types and imports**

In `mobile/src/api/gcp/compute_addresses.rs`:

```rust
//! GCP Compute Engine Addresses API module
//!
//! Manages external IP addresses (static/reserved).

use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::{GCP_COMPUTE_API_BASE, GcpRestClient, Operation};

/// External IP address (static/reserved)
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub name: String,
    pub address: String,        // IP address (e.g., "35.1.2.3")
    pub status: String,         // "RESERVED", "IN_USE"
    pub address_type: String,   // "EXTERNAL", "INTERNAL"
    #[serde(default)]
    pub users: Vec<String>,     // VM instances using this IP (empty if RESERVED)
    pub region: String,         // e.g., "us-central1"
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressList {
    #[serde(default)]
    pub items: Vec<Address>,
}
```

- [ ] **Step 3: Implement list_addresses method**

In `mobile/src/api/gcp/compute_addresses.rs`:

```rust
impl GcpRestClient {
    /// List all addresses in a region
    pub fn list_addresses(
        &self,
        project_id: &str,
        region: &str,
    ) -> Result<Vec<Address>> {
        let url = format!(
            "{}/projects/{}/regions/{}/addresses",
            GCP_COMPUTE_API_BASE, project_id, region
        );
        
        let response: AddressList = self.get(&url)?;
        Ok(response.items)
    }
}
```

- [ ] **Step 4: Implement get_address method**

In `mobile/src/api/gcp/compute_addresses.rs`:

```rust
impl GcpRestClient {
    // ... existing list_addresses ...

    /// Get a specific address
    pub fn get_address(
        &self,
        project_id: &str,
        region: &str,
        address_name: &str,
    ) -> Result<Address> {
        let url = format!(
            "{}/projects/{}/regions/{}/addresses/{}",
            GCP_COMPUTE_API_BASE, project_id, region, address_name
        );
        
        self.get(&url)
    }
}
```

- [ ] **Step 5: Implement delete_address method**

In `mobile/src/api/gcp/compute_addresses.rs`:

```rust
impl GcpRestClient {
    // ... existing methods ...

    /// Delete (release) a reserved address
    pub fn delete_address(
        &self,
        project_id: &str,
        region: &str,
        address_name: &str,
    ) -> Result<Operation> {
        let url = format!(
            "{}/projects/{}/regions/{}/addresses/{}",
            GCP_COMPUTE_API_BASE, project_id, region, address_name
        );
        
        self.delete(&url)
    }
}
```

- [ ] **Step 6: Export module**

In `mobile/src/api/gcp/mod.rs`, add after existing module declarations:

```rust
pub mod compute_addresses;
pub use compute_addresses::{Address, AddressList};
```

- [ ] **Step 7: Verify compilation**

Run: `cd mobile && cargo check`
Expected: Compiles without errors

- [ ] **Step 8: Commit**

```bash
git add mobile/src/api/gcp/compute_addresses.rs mobile/src/api/gcp/mod.rs
git commit -m "feat(gcp): add addresses API for static IP management

- Add Address and AddressList types
- Implement list_addresses, get_address, delete_address methods
- Export types from gcp module

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 2: Add nat_ip Field to AccessConfig

**Files:**
- Modify: `mobile/src/api/gcp/compute.rs:52-58`

**Interfaces:**
- Consumes: Existing `AccessConfig` struct
- Produces: Updated `AccessConfig` with optional `nat_ip` field for attaching static IPs

- [ ] **Step 1: Add nat_ip field to AccessConfig**

In `mobile/src/api/gcp/compute.rs`, find the `AccessConfig` struct (around line 52-58) and update it:

```rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessConfig {
    #[serde(rename = "type")]
    pub type_: String, // "ONE_TO_ONE_NAT"
    pub name: String, // "External NAT"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nat_ip: Option<String>, // Static IP address to attach
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd mobile && cargo check`
Expected: Compiles without errors

- [ ] **Step 3: Commit**

```bash
git add mobile/src/api/gcp/compute.rs
git commit -m "feat(gcp): add nat_ip field to AccessConfig

Allows attaching static IP addresses to VMs during creation.
Field is optional and skipped if None.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 3: Add State for Static IP Detection in VM Delete Dialog

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs:99-231` (PlatformTab struct and Default impl)

**Interfaces:**
- Consumes: Existing `PlatformTab` struct
- Produces: Four new state fields: `delete_vm_has_static_ip`, `delete_vm_ip_address`, `delete_vm_ip_name`, `delete_vm_release_ip`

- [ ] **Step 1: Add state fields to PlatformTab struct**

In `mobile/src/ui_tabs/platform.rs`, find the `delete_vm_hard_delete` field (around line 168) and add after it:

```rust
    // Delete VM dialog state (existing fields above)
    delete_vm_hard_delete: bool,

    // Static IP detection and release
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_vm_has_static_ip: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_vm_ip_address: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_vm_ip_name: Option<String>,
    #[cfg_attr(feature = "serde", serde(skip))]
    delete_vm_release_ip: bool,
```

- [ ] **Step 2: Initialize fields in Default impl**

In `mobile/src/ui_tabs/platform.rs`, find the `delete_vm_hard_delete: false,` line in Default impl and add after it:

```rust
            delete_vm_hard_delete: false,
            delete_vm_has_static_ip: false,
            delete_vm_ip_address: None,
            delete_vm_ip_name: None,
            delete_vm_release_ip: false,
```

- [ ] **Step 3: Verify compilation**

Run: `cd mobile && cargo check`
Expected: Compiles without errors

- [ ] **Step 4: Commit**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "feat(platform): add state for static IP detection in VM delete

Tracks whether VM has static IP and user choice to release it.
Default is to keep static IPs (release_ip = false).

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 4: Implement Static IP Detection and UI in Delete Dialog

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs:2741-2753` (show_delete_vm_confirmation method)
- Modify: `mobile/src/ui_tabs/platform.rs:2755-2890` (render_delete_vm_dialog method)

**Interfaces:**
- Consumes: `GcpRestClient::list_addresses()`, state fields from Task 3
- Produces: Updated `show_delete_vm_confirmation` that detects static IPs, updated `render_delete_vm_dialog` that shows checkbox and link

- [ ] **Step 1: Add static IP detection to show_delete_vm_confirmation**

In `mobile/src/ui_tabs/platform.rs`, find `show_delete_vm_confirmation` method (around line 2741) and update it:

```rust
fn show_delete_vm_confirmation(
    &mut self,
    platform_name: String,
    vm_name: String,
    zone: String,
) {
    self.delete_vm_platform = platform_name.clone();
    self.delete_vm_list.clear();
    self.delete_vm_list.push((vm_name.clone(), zone.clone(), "".to_string()));
    self.delete_vm_selected = Some(0);
    self.delete_vm_confirming = true;
    self.show_delete_vm_dialog = true;

    // Reset static IP state
    self.delete_vm_has_static_ip = false;
    self.delete_vm_ip_address = None;
    self.delete_vm_ip_name = None;
    self.delete_vm_release_ip = false;

    // Check if VM has static IP (requires access token and config)
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::config::AppConfig;
        
        if let Ok((app_config, _)) = load_config(&None) {
            if let Some(platform) = app_config.platforms.iter()
                .find(|p| p.gcp_selected_project_id.as_ref() == Some(&platform_name))
            {
                if let Some(token) = platform.gcp_oauth_access_token.as_ref() {
                    if let Some(vm) = platform.vms.iter()
                        .find(|v| v.name == vm_name && v.zone == zone)
                    {
                        if let Some(external_ip) = &vm.external_ip {
                            // Extract region from zone (e.g., "us-central1-a" -> "us-central1")
                            let region = zone.rsplitn(2, '-').nth(1)
                                .unwrap_or(&zone)
                                .to_string();

                            // Query addresses to check if IP is static
                            use crate::api::gcp::GcpRestClient;
                            let client = GcpRestClient::new(token.clone());
                            
                            if let Ok(addresses) = client.list_addresses(&platform_name, &region) {
                                // Find address matching this IP
                                if let Some(addr) = addresses.iter()
                                    .find(|a| &a.address == external_ip)
                                {
                                    self.delete_vm_has_static_ip = true;
                                    self.delete_vm_ip_address = Some(addr.address.clone());
                                    self.delete_vm_ip_name = Some(addr.name.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Add static IP UI to render_delete_vm_dialog**

In `mobile/src/ui_tabs/platform.rs`, find the delete confirmation section in `render_delete_vm_dialog` (after the VM list but before the Delete button, around line 2810):

```rust
                    // After displaying VM name/zone, before the Delete button:
                    
                    // Static IP section
                    if self.delete_vm_has_static_ip {
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);
                        
                        ui.label(format!(
                            "Static IP: {}",
                            self.delete_vm_ip_address.as_ref().unwrap()
                        ));
                        ui.checkbox(
                            &mut self.delete_vm_release_ip,
                            "Release static IP address"
                        );
                        ui.label("(Default: Keep IP reserved for future use)");
                        
                        ui.add_space(8.0);
                        if ui.add(MaterialButton::text("Manage IPs in GCP Console").small()).clicked() {
                            let url = format!(
                                "https://console.cloud.google.com/networking/addresses/list?project={}",
                                self.delete_vm_platform
                            );
                            let _ = webbrowser::open(&url);
                        }
                    }

                    ui.add_space(8.0);
```

- [ ] **Step 3: Verify compilation**

Run: `cd mobile && cargo check`
Expected: Compiles without errors

- [ ] **Step 4: Commit**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "feat(platform): detect and display static IP in delete dialog

- Query GCP addresses API to detect if VM has static IP
- Show checkbox to release static IP (default: unchecked/keep)
- Add link to GCP Console IP management
- Extract region from zone for API call

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 5: Implement Static IP Release Logic

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs:2755-2890` (render_delete_vm_dialog method, deletion confirmation section)

**Interfaces:**
- Consumes: `GcpRestClient::delete_address()`, `delete_vm_release_ip` state
- Produces: IP deletion after VM deletion when checkbox is checked

- [ ] **Step 1: Add IP deletion after VM deletion**

In `mobile/src/ui_tabs/platform.rs`, find where the VM deletion is confirmed (in `render_delete_vm_dialog`, around line 2850 where the actual deletion happens):

```rust
// After successful VM deletion (where operation_state is updated):
// Find the section that handles deletion and add IP release logic

// When delete button is clicked and deletion is triggered:
if self.delete_vm_release_ip && self.delete_vm_ip_name.is_some() {
    // Release static IP address
    #[cfg(not(target_arch = "wasm32"))]
    {
        use crate::config::AppConfig;
        
        if let Ok((app_config, _)) = load_config(&None) {
            if let Some(platform) = app_config.platforms.iter()
                .find(|p| p.gcp_selected_project_id.as_ref() == Some(&self.delete_vm_platform))
            {
                if let Some(token) = platform.gcp_oauth_access_token.as_ref() {
                    if let Some((_, zone, _)) = self.delete_vm_list.get(0) {
                        // Extract region from zone
                        let region = zone.rsplitn(2, '-').nth(1)
                            .unwrap_or(zone)
                            .to_string();

                        use crate::api::gcp::GcpRestClient;
                        let client = GcpRestClient::new(token.clone());
                        
                        let addr_name = self.delete_vm_ip_name.as_ref().unwrap();
                        if let Err(e) = client.delete_address(
                            &self.delete_vm_platform,
                            &region,
                            addr_name
                        ) {
                            dure_warn!("Failed to release static IP {}: {}", addr_name, e);
                        } else {
                            dure_info!("Released static IP address: {}", addr_name);
                        }
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cd mobile && cargo check`
Expected: Compiles without errors

- [ ] **Step 3: Test manually**

1. Create a VM with static IP in GCP Console
2. Run app and navigate to Platform tab
3. Click "Del VM"
4. Verify static IP is detected and checkbox shown
5. Check the checkbox and delete VM
6. Verify in GCP Console that static IP is released

- [ ] **Step 4: Commit**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "feat(platform): release static IP when checkbox is checked

After VM deletion, releases static IP if user checked the release
checkbox. Logs success/failure but does not block on result.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 6: Add Reserved IP Selection to VM Add Wizard

**Files:**
- Modify: `mobile/src/ui_dlg/platform_gcp.rs` (GcpWizard struct and VM creation flow)

**Interfaces:**
- Consumes: `GcpRestClient::list_addresses()`, `AccessConfig::nat_ip` field from Task 2
- Produces: Updated GcpWizard with IP selection dropdown, reserved IP filtering, and attachment logic

- [ ] **Step 1: Add IpOption enum**

In `mobile/src/ui_dlg/platform_gcp.rs`, add after the imports (before struct definitions):

```rust
/// External IP option for VM creation
#[derive(Debug, Clone, PartialEq)]
enum IpOption {
    Ephemeral,
    Reserved(usize), // Index into available_reserved_ips
}

impl Default for IpOption {
    fn default() -> Self {
        IpOption::Ephemeral
    }
}
```

- [ ] **Step 2: Add state fields to GcpWizard**

In `mobile/src/ui_dlg/platform_gcp.rs`, find the `GcpWizard` struct and add fields:

```rust
pub struct GcpWizard {
    // ... existing fields ...
    
    // Reserved IP selection
    available_reserved_ips: Vec<crate::api::gcp::Address>,
    selected_ip_option: IpOption,
}
```

- [ ] **Step 3: Initialize fields in Default/new**

In the `Default` or initialization method for `GcpWizard`:

```rust
available_reserved_ips: Vec::new(),
selected_ip_option: IpOption::Ephemeral,
```

- [ ] **Step 4: Fetch reserved IPs when region is selected**

Find where the VM creation form is shown and region/zone are displayed. Add IP fetching logic:

```rust
// After region is selected, fetch available reserved IPs
if let Some(selected_region) = &self.selected_region {
    // Fetch only once per region
    if self.available_reserved_ips.is_empty() || 
       self.last_fetched_region.as_ref() != Some(selected_region) 
    {
        if let Some(token) = &self.access_token {
            use crate::api::gcp::GcpRestClient;
            let client = GcpRestClient::new(token.clone());
            
            if let Ok(addresses) = client.list_addresses(&self.project_id, selected_region) {
                self.available_reserved_ips = addresses
                    .into_iter()
                    .filter(|a| a.status == "RESERVED" && a.address_type == "EXTERNAL")
                    .collect();
                self.last_fetched_region = Some(selected_region.clone());
            }
        }
    }
}
```

- [ ] **Step 5: Add IP selection dropdown to UI**

In the VM creation form UI section (where machine type, name, etc. are shown):

```rust
ui.add_space(8.0);
ui.label("External IP:");
egui::ComboBox::from_label("")
    .selected_text(match self.selected_ip_option {
        IpOption::Ephemeral => "Ephemeral (auto-assigned)".to_string(),
        IpOption::Reserved(idx) => {
            if idx < self.available_reserved_ips.len() {
                let addr = &self.available_reserved_ips[idx];
                format!("Reserved: {} ({})", addr.address, addr.name)
            } else {
                "Ephemeral (auto-assigned)".to_string()
            }
        }
    })
    .show_ui(ui, |ui| {
        ui.selectable_value(
            &mut self.selected_ip_option,
            IpOption::Ephemeral,
            "Ephemeral (auto-assigned)"
        );
        
        for (idx, addr) in self.available_reserved_ips.iter().enumerate() {
            ui.selectable_value(
                &mut self.selected_ip_option,
                IpOption::Reserved(idx),
                format!("Reserved: {} ({})", addr.address, addr.name)
            );
        }
    });

if !self.available_reserved_ips.is_empty() {
    ui.add_space(4.0);
    if ui.add(MaterialButton::text("Manage IPs in GCP Console").small()).clicked() {
        let url = format!(
            "https://console.cloud.google.com/networking/addresses/list?project={}",
            self.project_id
        );
        let _ = webbrowser::open(&url);
    }
}
```

- [ ] **Step 6: Update VM creation to use selected IP**

Find where the `AccessConfig` is created for VM creation and update it:

```rust
let access_configs = match self.selected_ip_option {
    IpOption::Ephemeral => {
        // Existing behavior: ephemeral IP
        Some(vec![crate::api::gcp::compute::AccessConfig {
            type_: "ONE_TO_ONE_NAT".to_string(),
            name: "External NAT".to_string(),
            nat_ip: None,
        }])
    }
    IpOption::Reserved(idx) => {
        // Use reserved IP
        if idx < self.available_reserved_ips.len() {
            let addr = &self.available_reserved_ips[idx];
            Some(vec![crate::api::gcp::compute::AccessConfig {
                type_: "ONE_TO_ONE_NAT".to_string(),
                name: "External NAT".to_string(),
                nat_ip: Some(addr.address.clone()),
            }])
        } else {
            // Fallback to ephemeral if index invalid
            Some(vec![crate::api::gcp::compute::AccessConfig {
                type_: "ONE_TO_ONE_NAT".to_string(),
                name: "External NAT".to_string(),
                nat_ip: None,
            }])
        }
    }
};
```

- [ ] **Step 7: Add last_fetched_region field**

Back in the struct, add:

```rust
pub struct GcpWizard {
    // ... existing fields ...
    available_reserved_ips: Vec<crate::api::gcp::Address>,
    selected_ip_option: IpOption,
    last_fetched_region: Option<String>,
}
```

And in initialization:

```rust
last_fetched_region: None,
```

- [ ] **Step 8: Verify compilation**

Run: `cd mobile && cargo check`
Expected: Compiles without errors

- [ ] **Step 9: Test manually**

1. Reserve an IP in GCP Console
2. Run app and click "Add VM"
3. Select same region as reserved IP
4. Verify dropdown shows reserved IP
5. Select reserved IP and create VM
6. Verify VM gets the reserved IP

- [ ] **Step 10: Commit**

```bash
git add mobile/src/ui_dlg/platform_gcp.rs
git commit -m "feat(platform): add reserved IP selection to VM creation

- Fetch available reserved IPs when region selected
- Show dropdown with ephemeral (default) and reserved IP options
- Attach selected static IP to VM via AccessConfig.nat_ip
- Add GCP Console link for IP management

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Verification

After completing all tasks:

1. **VM Delete with Static IP:**
   - Create VM with static IP in GCP
   - Delete via Dure with checkbox unchecked → IP remains reserved
   - Delete via Dure with checkbox checked → IP is released

2. **VM Delete with Ephemeral IP:**
   - Create VM with ephemeral IP
   - Delete via Dure → no checkbox shown, IP auto-released

3. **VM Create with Reserved IP:**
   - Reserve IP in GCP Console
   - Create VM in Dure selecting reserved IP
   - Verify VM gets the reserved IP

4. **VM Create with Ephemeral IP:**
   - Create VM with "Ephemeral (auto-assigned)"
   - Verify VM gets ephemeral IP (existing behavior)

5. **GCP Console Links:**
   - Click "Manage IPs in GCP Console" buttons
   - Verify correct project ID in URL
   - Verify page opens to addresses list

## Success Criteria

- [ ] All tasks compile without errors
- [ ] Static IP detection works in delete dialog
- [ ] Checkbox defaults to unchecked (keep IP)
- [ ] Static IP released only when checkbox checked
- [ ] Reserved IPs appear in dropdown when available
- [ ] Selected reserved IP attaches to VM correctly
- [ ] Ephemeral IP creation still works (default behavior)
- [ ] GCP Console links open correct pages
- [ ] No regressions in existing VM add/delete flows
