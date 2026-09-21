# GCP Reserved IP Address Management

**Date:** 2026-09-21  
**Status:** Design  
**Author:** Claude Sonnet 4.5

## Overview

Add GCP reserved (static) IP address management to VM lifecycle (add/delete) flows, allowing users to:
- Release static IPs when deleting VMs (with confirmation)
- Attach existing reserved IPs when creating VMs
- View and manage IPs via GCP Console links

## Requirements

### VM Delete Flow
1. When deleting a VM, check if it has a static external IP
2. If static IP exists:
   - Show checkbox: "Release static IP address"
   - Default: **unchecked** (keep IP)
   - If ephemeral IP: auto-release (no checkbox)
3. Add link button: "Manage IPs in GCP Console"
4. When user confirms deletion:
   - Delete VM
   - If checkbox checked: delete associated static IP address

### VM Add Flow
1. When creating a VM, fetch available reserved IPs in the region
2. If reserved IPs exist:
   - Show dropdown: "External IP" with options:
     - "Ephemeral (auto-assigned)" - **default**
     - Each reserved IP: "Reserved: {ip_address} ({name})"
3. Add link button: "Manage IPs in GCP Console"
4. When user creates VM:
   - If "Ephemeral" selected: create VM with ephemeral IP (existing behavior)
   - If reserved IP selected: attach that IP to the new VM

### GCP Console Links
- VM Delete: `https://console.cloud.google.com/networking/addresses/list?project={project_id}`
- VM Add: `https://console.cloud.google.com/networking/addresses/list?project={project_id}`

## Architecture

### 1. New GCP API Module: `compute_addresses.rs`

**Location:** `mobile/src/api/gcp/compute_addresses.rs`

**GCP Compute API Endpoints:**
- List addresses: `GET /projects/{project}/regions/{region}/addresses`
- Delete address: `DELETE /projects/{project}/regions/{region}/addresses/{address}`
- Get address: `GET /projects/{project}/regions/{region}/addresses/{address}`

**Types:**

```rust
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

**API Functions:**

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

### 2. VM Delete Dialog Enhancement

**File:** `mobile/src/ui_tabs/platform.rs`

**State additions to `PlatformTab`:**

```rust
// Delete VM dialog state
delete_vm_has_static_ip: bool,
delete_vm_ip_address: Option<String>,
delete_vm_ip_name: Option<String>,
delete_vm_release_ip: bool,  // Checkbox state (default: false)
```

**Flow:**

```rust
// When delete confirmation dialog opens (show_delete_vm_confirmation):
// 1. Check if VM has external IP
let external_ip = vm.external_ip;

// 2. If external IP exists, check if it's static
if let Some(ip) = external_ip {
    // Query all addresses in the VM's region
    let addresses = client.list_addresses(project_id, region)?;
    
    // Find address matching this IP
    if let Some(addr) = addresses.iter().find(|a| a.address == ip) {
        // Static IP found
        self.delete_vm_has_static_ip = true;
        self.delete_vm_ip_address = Some(addr.address.clone());
        self.delete_vm_ip_name = Some(addr.name.clone());
        self.delete_vm_release_ip = false; // Default: keep
    } else {
        // Ephemeral IP (not in address list)
        self.delete_vm_has_static_ip = false;
    }
}

// In render_delete_vm_dialog:
if self.delete_vm_has_static_ip {
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);
    
    ui.label(format!("Static IP: {}", 
                     self.delete_vm_ip_address.as_ref().unwrap()));
    ui.checkbox(&mut self.delete_vm_release_ip, "Release static IP address");
    ui.label("(Default: Keep IP reserved for future use)");
    
    ui.add_space(8.0);
    if ui.add(MaterialButton::text("Manage IPs in GCP Console").small()).clicked() {
        let url = format!(
            "https://console.cloud.google.com/networking/addresses/list?project={}",
            project_id
        );
        let _ = webbrowser::open(&url);
    }
}

// When user confirms deletion:
if self.delete_vm_release_ip && self.delete_vm_ip_name.is_some() {
    // Delete address after VM deletion succeeds
    let addr_name = self.delete_vm_ip_name.as_ref().unwrap();
    let _op = client.delete_address(project_id, region, addr_name)?;
    // Note: Could wait for operation to complete, but not critical
}
```

### 3. VM Add Dialog Enhancement

**File:** `mobile/src/ui_dlg/platform_gcp.rs` (GCP Wizard)

**State additions to `GcpWizard`:**

```rust
// Available reserved IPs in selected region
available_reserved_ips: Vec<Address>,
selected_ip_option: IpOption, // Enum: Ephemeral or Reserved(index)

enum IpOption {
    Ephemeral,
    Reserved(usize), // Index into available_reserved_ips
}
```

**Flow:**

```rust
// When VM creation step is shown and region is selected:
// Fetch available reserved IPs
let addresses = client.list_addresses(project_id, region)?;
self.available_reserved_ips = addresses
    .into_iter()
    .filter(|a| a.status == "RESERVED" && a.address_type == "EXTERNAL")
    .collect();

// In VM creation UI:
ui.label("External IP:");
egui::ComboBox::from_label("")
    .selected_text(match self.selected_ip_option {
        IpOption::Ephemeral => "Ephemeral (auto-assigned)".to_string(),
        IpOption::Reserved(idx) => {
            let addr = &self.available_reserved_ips[idx];
            format!("Reserved: {} ({})", addr.address, addr.name)
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
            project_id
        );
        let _ = webbrowser::open(&url);
    }
}

// When creating VM:
let access_configs = match self.selected_ip_option {
    IpOption::Ephemeral => {
        // Existing behavior: ephemeral IP
        Some(vec![AccessConfig {
            type_: "ONE_TO_ONE_NAT".to_string(),
            name: "External NAT".to_string(),
        }])
    }
    IpOption::Reserved(idx) => {
        // Use reserved IP
        let addr = &self.available_reserved_ips[idx];
        Some(vec![AccessConfig {
            type_: "ONE_TO_ONE_NAT".to_string(),
            name: "External NAT".to_string(),
            nat_ip: Some(addr.address.clone()), // NEW field
        }])
    }
};
```

**AccessConfig update:**

```rust
// In mobile/src/api/gcp/compute.rs
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessConfig {
    #[serde(rename = "type")]
    pub type_: String, // "ONE_TO_ONE_NAT"
    pub name: String, // "External NAT"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nat_ip: Option<String>, // NEW: Static IP address to attach
}
```

## Data Flow

### VM Delete with Static IP Release

```
User clicks Del VM
    ↓
Show delete confirmation dialog
    ↓
Query GCP addresses API for VM's region
    ↓
Find address matching VM's external IP
    ↓
If found (static):
    Show checkbox "Release static IP" (default: unchecked)
    Show "Manage IPs" link button
    ↓
User confirms deletion + checkbox state
    ↓
Delete VM via compute API
    ↓
If release_ip checked:
    Delete address via addresses API
```

### VM Add with Reserved IP Selection

```
User clicks Add VM
    ↓
VM creation wizard opens
    ↓
User selects region
    ↓
Fetch reserved IPs in region via addresses API
    ↓
Filter: status == "RESERVED" && type == "EXTERNAL"
    ↓
Show dropdown:
    - Ephemeral (default)
    - Reserved: {ip} ({name}) for each reserved IP
Show "Manage IPs" link button
    ↓
User selects IP option + fills VM details
    ↓
User confirms VM creation
    ↓
Create VM with:
    If Ephemeral: standard AccessConfig
    If Reserved: AccessConfig with nat_ip = selected IP
```

## Files to Modify

1. **NEW:** `mobile/src/api/gcp/compute_addresses.rs` - Address management APIs
2. **MODIFY:** `mobile/src/api/gcp/mod.rs` - Add `pub mod compute_addresses;`
3. **MODIFY:** `mobile/src/api/gcp/compute.rs` - Add `nat_ip` field to `AccessConfig`
4. **MODIFY:** `mobile/src/ui_tabs/platform.rs` - VM delete dialog enhancement
5. **MODIFY:** `mobile/src/ui_dlg/platform_gcp.rs` - VM add wizard enhancement

## Testing

### VM Delete
1. Create VM with static IP
2. Click Del VM
3. Verify checkbox appears: "Release static IP"
4. Verify default is unchecked
5. Verify "Manage IPs" link opens correct GCP console page
6. Test deletion with checkbox checked → IP should be released
7. Test deletion with checkbox unchecked → IP should remain reserved

### VM Add
1. Reserve an IP in GCP console
2. Click Add VM in Dure
3. Select same region as reserved IP
4. Verify dropdown shows reserved IP option
5. Select reserved IP
6. Create VM
7. Verify VM gets the reserved IP attached

### Edge Cases
1. VM with ephemeral IP → no checkbox shown on delete
2. Region with no reserved IPs → only "Ephemeral" option shown
3. Reserved IP in different region → not shown in dropdown
4. Multiple reserved IPs → all shown in dropdown

## Security Considerations

- GCP API calls require OAuth token with `compute` scope (already required)
- No new permissions needed
- IP deletion is non-destructive (can reserve again)
- Link buttons use HTTPS

## Performance

- Address list API call: ~200-500ms per region
- Minimal impact: only called when dialogs open
- No background polling needed (fetch on-demand)

## Future Enhancements

- Show IP cost estimate in dropdown
- Auto-suggest IP names when reserving new IPs
- Bulk IP management view
- Support for IPv6 addresses
- Support for internal (private) addresses

## Why This Approach

1. **Fetch on-demand:** No state caching needed, always fresh data
2. **Default to safe:** Keep static IPs by default (avoid accidental deletion)
3. **Ephemeral auto-release:** No user action needed for temporary IPs
4. **Dropdown in wizard:** Natural place to select IP during VM creation
5. **GCP Console links:** Escape hatch for advanced management
