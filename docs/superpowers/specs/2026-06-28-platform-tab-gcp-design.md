# Platform Tab GCP Integration Design

**Date:** 2026-06-28  
**Author:** Claude Sonnet 4.5  
**Status:** Approved  

## Overview

This design describes the enhancement of the Platform tab (`mobile/src/ui_tabs/platform.rs`) to provide comprehensive GCP integration with a custom egui table widget. The new implementation will replace the existing MaterialSpreadsheet with a hierarchical table showing platforms, projects, and VMs with inline action buttons for VM management, firewall configuration, and SSH connectivity testing.

## Goals

1. **Hierarchical Display:** Show platform accounts → projects → VMs in a flat table with visual indentation
2. **Action Integration:** Provide inline buttons for VM operations (Delete, Regenerate, Restart) and firewall management
3. **Status Visibility:** Display real-time status including project counts, VM counts, firewall whitelisting, and SSH connectivity
4. **Config Persistence:** Store all platform and VM data in `~/.config/dure/config.yml`
5. **SSH Key Management:** Integrate with KeePass keyring at `~/.config/dure/key.kdbx` for SSH private key storage

## Non-Goals

- Multi-platform selection (only one project per platform configuration)
- Multi-VM display (show only primary VM even if multiple exist)
- Auto-refresh on timer (manual refresh + auto-refresh after actions only)
- iOS support (Android and Desktop only)

---

## Architecture Overview

### Layered Architecture

**UI Layer** (`mobile/src/ui_tabs/platform.rs`)
- Custom `PlatformTable` widget renders hierarchical table using egui primitives
- Handles user interactions (button clicks, row selection, dialog management)
- Manages UI state (loading indicators, dialogs, background task tracking)

**Business Logic Layer**
- `mobile/src/calc/platform_gcp.rs` - GCP-specific platform operations (add, remove, configure)
- `mobile/src/calc/hosting_gcp.rs` - GCP VM lifecycle management (create, delete, restart, regenerate)
- `mobile/src/calc/platform.rs` - Generic platform interface (refactored to remove GCP-specific code)
- `mobile/src/calc/hosting.rs` - Generic hosting abstraction (refactored to remove unnecessary code)

**API Layer** (`mobile/src/calc/gcp_rest.rs`)
- GCP Compute Engine REST API client (existing, extended)
- Firewall rule management
- IP detection via icanhazip.com

**SSH Layer** (`mobile/src/calc/ssh.rs`)
- Existing `test_connection()` function
- Wrapped in background tasks via `poll_promise::Promise`

### Data Flow

1. **On Load:** Read `~/.config/dure/config.yml` → build table rows from cached platform/VM data
2. **On Refresh:** Fetch fresh data from GCP API → update `AppConfig.platforms[].vms` → save to config → rebuild table
3. **After Actions:** Execute action (delete/restart/regenerate) → fetch fresh data → update config → rebuild table
4. **SSH Tests:** Run in background → update SSH status (transient, not saved to config)

---

## Data Model & State Management

### Configuration Structures

**Updated CloudPlatformConfig:**
```rust
pub struct CloudPlatformConfig {
    pub name: String,
    pub platform_type: String, // "gcp", "firebase", "supabase"
    pub gcp_connected_email: Option<String>,
    pub gcp_oauth_access_token: Option<String>,
    pub gcp_oauth_refresh_token: Option<String>,
    pub gcp_selected_project_id: Option<String>,  // NEW: selected project
    pub vms: Vec<VmInstance>,  // All VMs in selected project
    // ... other fields
}
```

**Existing VmInstance:**
```rust
pub struct VmInstance {
    pub name: String,
    pub instance_id: String,
    pub zone: String,
    pub gcp_project_id: String,
    pub machine_type: String,
    pub status: String,
    pub external_ip: Option<String>,
    pub internal_ip: Option<String>,
    pub gcp_billing_account: Option<String>,
    pub created_at: i64,
    pub ssh_key_name: Option<String>,  // Keyring domain for SSH key
}
```

### Table Row Types

```rust
enum PlatformRow {
    Account {
        platform_name: String,
        email: String,
        project_count: usize,       // Total projects in GCP account
        vm_count: usize,            // VMs in selected project
    },
    Project {
        platform_name: String,
        project_id: String,
        vm_count: usize,            // Total VMs in project
        current_ip: Option<String>,
        firewall_status: FirewallStatus, // Whitelisted, NotWhitelisted, Unknown
    },
    Vm {
        platform_name: String,
        project_id: String,
        vm_name: String,
        zone: String,
        instance_id: String,
        external_ip: Option<String>,
        ssh_status: SshStatus, // Available, Unavailable, Testing, Error(String)
    },
}
```

### PlatformTab State

```rust
struct PlatformTab {
    // Table data
    rows: Vec<PlatformRow>,  // Flattened hierarchy built from config
    
    // Background tasks (transient)
    ssh_test_tasks: HashMap<String, Promise<SshConnectionResult>>,
    refresh_task: Option<Promise<Result<()>>>,
    
    // Action in progress
    current_action: Option<ActionInProgress>,
    action_task: Option<Promise<Result<String>>>,
    
    // Dialogs
    confirmation_dialog: Option<ConfirmationDialog>,
    
    // Manual refresh
    last_refresh: Option<Instant>,
}
```

### Config Update Points

- **Refresh Status:** Fetch GCP VM list → update `platform.vms` → save config
- **Delete VM:** Remove VM from `platform.vms` → save config
- **Regenerate VM:** Clear `platform.vms`, add new VM → save config
- **Restart VM:** Fetch updated VM status → update `platform.vms` → save config
- **Update Firewall:** Fetch firewall rules → update firewall status (not persisted)

---

## UI Components & Layout

### Table Structure

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ Platform Name               │ Status                          │ Actions      │
├──────────────────────────────────────────────────────────────────────────────┤
│ GCP: nikescar@gmail.com     │ 8 Projects                      │              │
│  ├─ dure                    │ 1 VM                            │ [Update      │
│  │                          │ ✓ GCP Firewall Whitelisted      │  Firewall]   │
│  │                          │   (117.53.222.116)              │              │
│  └─── dure-vm               │ ✓ SSH Connection OK(:22)        │ [Delete VM]  │
│                             │                                 │ [Regenerate] │
│                             │                                 │ [Restart VM] │
│                             │                                 │ [Refresh]    │
└──────────────────────────────────────────────────────────────────────────────┘
```

### Row Rendering

**Indentation Levels:**
- **Level 0 (Platform):** No indent, shows platform name and email
- **Level 1 (Project):** 1x indent with `├─` connector, shows project name
- **Level 2 (VM):** 2x indent with `└───` connector, shows VM name

**Status Column Details:**

| Row Type | Status Display |
|----------|----------------|
| **Platform** | `"{count} Projects"` |
| **Project** | `"{count} VM\n{✓\|✗} GCP Firewall Whitelisted({ip})"` |
| **VM** | `"{✓\|✗\|🔄} SSH Connection {OK\|Failed\|Testing...}(:22)"` |

**Visual Elements:**
- **Status Badges:** Colored text (green ✓, red ✗, yellow 🔄)
- **Action Buttons:** Material3 filled/outlined buttons
- **Icons:** Unicode symbols for tree structure
- **Striped Rows:** Alternating background colors for readability

### Action Button Layout

**Platform Row:** No actions (summary only)

**Project Row:** 
- `[Update Firewall]` button - adds current IP to GCP firewall whitelist

**VM Row:**
- `[Delete VM]` button (destructive, red)
- `[Regenerate VM]` button (destructive, red)
- `[Restart VM]` button (warning, yellow)
- `[Refresh Status]` button (primary, blue)

---

## GCP API Integration

### API Client

Extend existing `GcpRestClient` in `mobile/src/calc/gcp_rest.rs`

**Authentication:**
- OAuth access token from `CloudPlatformConfig.gcp_oauth_access_token`
- Auto-refresh using `gcp_oauth_refresh_token` if expired
- All requests include `Authorization: Bearer {token}` header

### API Operations

**1. List Projects (for platform status count)**
```
GET https://cloudresourcemanager.googleapis.com/v1/projects
→ Count total projects for status display
```

**2. List VMs in Selected Project**
```
GET https://compute.googleapis.com/compute/v1/projects/{project_id}/aggregated/instances
→ Extract VM details (name, zone, IPs, status)
→ Save to config.yml platform.vms
```

**3. Get Current Public IP**
```
GET https://icanhazip.com
→ Returns plain text IP (e.g., "117.53.222.116\n")
→ Strip whitespace, cache for session
```

**4. List Firewall Rules**
```
GET https://compute.googleapis.com/compute/v1/projects/{project_id}/global/firewalls
→ Check if current IP in sourceRanges for SSH (port 22)
→ Return: Whitelisted (true/false)
```

**5. Update Firewall (Add Current IP)**
```
POST https://compute.googleapis.com/compute/v1/projects/{project_id}/global/firewalls/{rule_name}
Body: {
  "allowed": [{"IPProtocol": "tcp", "ports": ["22"]}],
  "sourceRanges": ["existing_ips...", "117.53.222.116/32"]
}
→ Add current IP to existing SSH firewall rule
→ Create new rule if none exists
```

**6. Delete VM**
```
DELETE https://compute.googleapis.com/compute/v1/projects/{project_id}/zones/{zone}/instances/{name}
→ Returns long-running operation
→ Poll until complete
```

**7. Restart VM**
```
POST https://compute.googleapis.com/compute/v1/projects/{project_id}/zones/{zone}/instances/{name}/reset
→ Poll until VM status = "RUNNING"
```

**8. Regenerate VM**
```
1. DELETE all VMs in platform.vms
2. POST create new VM:
   - Machine type: e2-micro
   - Boot disk: Debian 11
   - Network: default
   - Firewall: allow SSH from all IPs initially
→ Poll until VM is RUNNING
→ Generate SSH keypair
→ Add public key to VM metadata
→ Store private key in keyring
→ Update config with new VM
```

---

## SSH Testing & Background Tasks

### Code Refactoring (Prerequisite)

Before implementing platform tab enhancements:

1. **Create `mobile/src/calc/hosting_gcp.rs`:**
   - Move GCP VM creation code from `ui_dlg/platform_gcp.rs`
   - Functions: `create_vm()`, `delete_vm()`, `restart_vm()`, `regenerate_vm()`
   - SSH keypair generation and keyring storage

2. **Create `mobile/src/calc/platform_gcp.rs`:**
   - Move platform add/remove code from `calc/platform.rs`
   - GCP-specific platform initialization
   - Project selection and OAuth flow

3. **Refactor `calc/platform.rs`:**
   - Remove GCP-specific logic (move to `platform_gcp.rs`)
   - Keep only generic platform interface

4. **Refactor `calc/hosting.rs`:**
   - Remove unnecessary code
   - Keep only hosting abstraction layer

### SSH Key Management

**Keyring Location:** `~/.config/dure/key.kdbx` (single KeePass file)

**SSH Key Storage Flow:**
1. During VM creation, generate Ed25519 keypair
2. Add public key to VM metadata (GCP SSH keys)
3. Store private key in keyring:
   ```rust
   keyring::add_key(
       domain: "gcp.{platform_name}.{vm_name}",
       username: "generated_user",
       password: "", // Empty password
       ssh_key: Some(private_key_bytes),
   )
   ```
4. Save keyring domain to VM config:
   ```rust
   VmInstance {
       ssh_key_name: Some("gcp.platform_name.vm_name"),
       // ...
   }
   ```

**SSH Testing with Keyring:**
```rust
fn test_ssh_connection(vm: &VmInstance) -> Result<SshConnectionResult> {
    // Load private key from keyring
    let keyring_domain = vm.ssh_key_name
        .ok_or("No SSH key configured")?;
    
    let key_entry = keyring::get_key(&keyring_domain)?;
    let private_key_bytes = key_entry.ssh_key
        .ok_or("No SSH key in keyring entry")?;
    
    // Build SSH config
    let ssh_config = SshHostConfig {
        host: format!("user@{}", vm.external_ip.ok_or("No IP")?),
        port: 22,
        keyring_domain: Some(keyring_domain),
        password: None,
        private_key_path: None,
    };
    
    // Call existing test_connection
    ssh::test_connection(&ssh_config)
}
```

### Background Task Management

**Task Lifecycle:**

1. **On Tab Load / Refresh:**
   ```rust
   for platform in platforms {
       if let Some(vm) = platform.vms.first() {
           let key = format!("{}:{}", platform.name, vm.name);
           
           let task = Promise::spawn_thread("ssh_test", {
               let vm = vm.clone();
               move || test_ssh_connection(&vm)
           });
           
           ssh_test_tasks.insert(key, task);
       }
   }
   ```

2. **Each Frame:**
   ```rust
   for (key, task) in ssh_test_tasks.iter() {
       if let Some(result) = task.ready() {
           // Update VM row SSH status
           match result {
               Ok(conn) if conn.success => status = "✓ SSH Connection OK(:22)",
               _ => status = "✗ SSH Connection Failed(:22)",
           }
       } else {
           status = "🔄 SSH Connection Testing..."
       }
   }
   
   // Remove completed tasks
   ssh_test_tasks.retain(|_, task| task.ready().is_none());
   ```

**Status States:**
- **Not Started:** Should not occur (tasks spawn on load)
- **Testing:** `"🔄 SSH Connection Testing..."` (yellow spinner)
- **Success:** `"✓ SSH Connection OK(:22)"` (green checkmark)
- **Failed:** `"✗ SSH Connection Failed(:22)"` (red X)

---

## Action Handlers & Confirmation Dialogs

### 1. Update Firewall (Project-level)

**Confirmation Dialog:**
```
┌─────────────────────────────────────────────────────┐
│ Update GCP Firewall                                 │
├─────────────────────────────────────────────────────┤
│ This will add your current IP to the GCP firewall  │
│ whitelist for SSH access (port 22).                │
│                                                     │
│ Project: dure                                       │
│ Current IP: 117.53.222.116                          │
│                                                     │
│ Type 'update' to confirm: [____________]            │
│                                                     │
│           [Cancel]  [Confirm]                       │
└─────────────────────────────────────────────────────┘
```

**Execution:**
1. Fetch current IP from icanhazip.com
2. Get firewall rules for project
3. Add current IP to SSH rule's `sourceRanges`
4. Poll operation until complete
5. Refresh platform data
6. Show success: "Firewall updated: {ip} whitelisted"

### 2. Delete VM (VM-level)

**Confirmation Dialog:**
```
┌─────────────────────────────────────────────────────┐
│ ⚠️  Delete Virtual Machine                          │
├─────────────────────────────────────────────────────┤
│ This will PERMANENTLY DELETE the VM and all data.  │
│                                                     │
│ Platform: GCP (nikescar@gmail.com)                  │
│ Project: dure                                       │
│ VM: dure-vm                                         │
│ Zone: us-central1-a                                 │
│ External IP: 117.53.222.116 (will be released)     │
│                                                     │
│ ⚠️  This action CANNOT be undone!                   │
│                                                     │
│ Type 'delete' to confirm: [____________]            │
│                                                     │
│           [Cancel]  [Delete]                        │
└─────────────────────────────────────────────────────┘
```

**Execution:**
1. Call GCP DELETE instance API
2. Poll operation status
3. Remove VM from `platform.vms`
4. Remove SSH key from keyring (cleanup)
5. Save config
6. Refresh table
7. Show success: "VM {name} deleted"

### 3. Regenerate VM (VM-level)

**Confirmation Dialog:**
```
┌─────────────────────────────────────────────────────┐
│ ⚠️  Regenerate Virtual Machine                      │
├─────────────────────────────────────────────────────┤
│ This will DELETE ALL VMs in this project and       │
│ create ONE fresh VM with default settings.         │
│                                                     │
│ Platform: GCP (nikescar@gmail.com)                  │
│ Project: dure                                       │
│ VMs to delete: 2                                    │
│   - dure-vm (us-central1-a)                         │
│   - dure-vm-2 (us-west1-b)                          │
│                                                     │
│ New VM will be created with:                        │
│   - Machine type: e2-micro                          │
│   - Region: us-central1                             │
│   - Boot disk: Debian 11                            │
│   - SSH key: auto-generated                         │
│                                                     │
│ ⚠️  ALL DATA ON EXISTING VMs WILL BE LOST!          │
│                                                     │
│ Type 'regenerate' to confirm: [____________]        │
│                                                     │
│           [Cancel]  [Regenerate]                    │
└─────────────────────────────────────────────────────┘
```

**Execution:**
1. Delete all VMs in `platform.vms`
2. Wait for all deletions
3. Generate new SSH keypair
4. Create new VM with defaults
5. Add SSH public key to VM metadata
6. Store private key in keyring: `"gcp.{platform}.{vm}"`
7. Poll VM creation until RUNNING
8. Update `platform.vms` with new VM
9. Save config
10. Refresh table
11. Show success: "VM regenerated: {name}"

### 4. Restart VM (VM-level)

**Confirmation Dialog:**
```
┌─────────────────────────────────────────────────────┐
│ Restart Virtual Machine                             │
├─────────────────────────────────────────────────────┤
│ This will restart the VM. Services will be          │
│ temporarily unavailable.                            │
│                                                     │
│ VM: dure-vm                                         │
│ Project: dure                                       │
│                                                     │
│ Type 'restart' to confirm: [____________]           │
│                                                     │
│           [Cancel]  [Restart]                       │
└─────────────────────────────────────────────────────┘
```

**Execution:**
1. Call GCP POST reset API
2. Poll until VM status = "RUNNING"
3. Refresh platform data
4. Show success: "VM {name} restarted"

### 5. Refresh Status (VM-level)

**No confirmation dialog** (read-only operation)

**Execution:**
1. Fetch fresh VM data from GCP API
2. Update `platform.vms`
3. Save config
4. Trigger background SSH test
5. Update table display
6. Show success: "Status refreshed"

### Progress Indicators

During action execution:
```
┌─────────────────────────────────────────────────────┐
│ Deleting VM...                                      │
│                                                     │
│ [████████░░░░░░░░] 60%                              │
│                                                     │
│ Waiting for operation to complete...                │
└─────────────────────────────────────────────────────┘
```

---

## Error Handling

### Error Categories

#### 1. GCP API Errors

| Error | Cause | User Message | Recovery |
|-------|-------|--------------|----------|
| 401 Unauthorized | Expired token | "Authentication expired. Reconnect GCP account." | Show "Reconnect" button |
| 403 Forbidden | API not enabled | "API not enabled: {name}. Enable in Console: {link}" | Show console link |
| 404 Not Found | Resource deleted | "Resource not found. May have been deleted." | Remove from config |
| 429 Rate Limit | Too many requests | "Rate limit exceeded. Wait 30s and retry." | Exponential backoff |
| 500 Server Error | GCP internal | "GCP server error. Retry in a few minutes." | Manual retry |
| Network Error | Timeout/offline | "Network error. Check connection." | Retry button |

#### 2. SSH Connection Errors

| Error | Cause | Display | Recovery |
|-------|-------|---------|----------|
| Timeout | Firewall/VM down | "✗ SSH Failed(:22) - Timeout" | Suggest "Update Firewall" |
| Auth Failed | Wrong key | "✗ SSH Failed(:22) - Auth Failed" | Suggest "Regenerate VM" |
| Refused | SSH not running | "✗ SSH Failed(:22) - Refused" | Info about SSH service |
| Unreachable | No IP/network | "✗ SSH Failed(:22) - Unreachable" | Check external IP |

#### 3. Keyring Errors

| Error | Cause | Message | Recovery |
|-------|-------|---------|----------|
| Not Found | Missing key.kdbx | "Keyring not found. Initialize?" | Create keyring |
| Key Missing | No SSH entry | "SSH key not found. Regenerate VM?" | Offer regenerate |
| Locked | Wrong keyfile | "Cannot unlock keyring. Check id_ed25519" | Show path |

#### 4. Config Errors

| Error | Cause | Message | Recovery |
|-------|-------|---------|----------|
| Not Writable | Permissions | "Cannot save: {path} not writable" | Show path |
| Corrupt | Invalid YAML | "Config corrupt. Backup saved." | Reset defaults |

### Error Display

**In Table:**
```
✗ SSH Connection Failed(:22) - Timeout
✗ GCP API Error: 403 Forbidden
```

**Error Dialog:**
```
┌─────────────────────────────────────────────────────┐
│ ⚠️  Action Failed                                    │
├─────────────────────────────────────────────────────┤
│ Failed to delete VM: dure-vm                        │
│                                                     │
│ Error: GCP API returned 403 Forbidden              │
│                                                     │
│ Possible causes:                                    │
│ • Compute Engine API not enabled                   │
│ • Insufficient IAM permissions                     │
│                                                     │
│ Enable API: [link]                                  │
│                                                     │
│           [Copy Error]  [Close]                     │
└─────────────────────────────────────────────────────┘
```

### Audit Trail

Log all actions:
```rust
audit::push_cli("platform", "gcp", "delete_vm", "VM: dure-vm - Success");
audit::push_cli("platform", "gcp", "update_firewall", "IP 117.53.2.116 whitelisted");
audit::push_cli("platform", "gcp", "regenerate_vm", "2 VMs deleted, new-vm created");
```

### Retry Strategy

- **Transient (429, 500, network):** Auto-retry with exponential backoff (1s, 2s, 4s, max 3 attempts)
- **Auth (401):** No retry, require user reconnect
- **Permanent (403, 404):** No retry, show error
- **SSH timeout:** Single attempt (15s timeout), no auto-retry

---

## Testing Strategy

### 1. Unit Tests

**Status Computation:**
```rust
#[test]
fn test_compute_platform_status() {
    let platform = test_platform_with_vms(2);
    let status = compute_platform_status(&platform, 8);
    assert_eq!(status, "8 Projects, 2 VM");
}

#[test]
fn test_firewall_whitelisted() {
    let rules = vec![test_firewall_rule()];
    let status = check_firewall_whitelisted(&rules, "117.53.222.116");
    assert_eq!(status, FirewallStatus::Whitelisted);
}
```

**Row Building:**
```rust
#[test]
fn test_build_table_rows() {
    let platforms = vec![test_platform()];
    let rows = build_platform_rows(&platforms);
    
    assert_eq!(rows.len(), 3); // Account, Project, VM
    assert!(matches!(rows[0], PlatformRow::Account { .. }));
    assert!(matches!(rows[1], PlatformRow::Project { .. }));
    assert!(matches!(rows[2], PlatformRow::Vm { .. }));
}
```

**SSH Key Management:**
```rust
#[test]
fn test_generate_ssh_keypair() {
    let (pub_key, priv_key) = generate_ssh_keypair().unwrap();
    assert!(pub_key.starts_with("ssh-ed25519"));
    assert!(priv_key.len() > 0);
}

#[test]
fn test_store_ssh_key_in_keyring() {
    let domain = "gcp.test.vm";
    store_ssh_key(domain, vec![1, 2, 3]).unwrap();
    
    let entry = keyring::get_key(domain).unwrap();
    assert!(entry.ssh_key.is_some());
}
```

### 2. Integration Tests

**GCP API Mocking:**
```rust
#[test]
fn test_delete_vm_integration() {
    let mock = mockito::mock("DELETE", "/compute/v1/projects/test/zones/z/instances/vm")
        .with_status(200)
        .with_body(r#"{"status": "DONE"}"#)
        .create();
    
    let client = GcpRestClient::new_mock("token");
    let result = delete_vm(&client, "test", "z", "vm");
    
    assert!(result.is_ok());
    mock.assert();
}
```

**Config Persistence:**
```rust
#[test]
fn test_vm_update_saves_config() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.yml");
    
    let mut config = AppConfig::default();
    config.platforms.push(platform_with_vm());
    config.save(&path).unwrap();
    
    let loaded = AppConfig::load_or_default(&path);
    assert_eq!(loaded.platforms[0].vms.len(), 1);
}
```

### 3. Manual Testing Checklist

**Platform Tab UI:**
- [ ] Table displays with proper indentation
- [ ] Status column shows correct counts/labels
- [ ] Action buttons in correct rows
- [ ] Material3 styling and hover states
- [ ] SSH status updates (spinner → result)

**Add Platform:**
- [ ] "Add Platform" opens dialog
- [ ] OAuth flow completes
- [ ] Project list loads from GCP
- [ ] Platform appears in table

**Actions:**
- [ ] Update Firewall: confirmation → update → status change
- [ ] Delete VM: warning → delete → removed from table
- [ ] Regenerate VM: warning → delete all → create new
- [ ] Restart VM: confirmation → restart → status refresh
- [ ] Refresh Status: no confirmation → data updates

**Error Handling:**
- [ ] Expired OAuth shows "Reconnect"
- [ ] Network error shows retry
- [ ] SSH timeout shows error message
- [ ] Missing keyring shows init prompt

**SSH Testing:**
- [ ] Background test starts on load
- [ ] Shows "Testing..." during test
- [ ] Updates to "OK(:22)" or "Failed(:22)"
- [ ] Timeout after 15s

**Config Persistence:**
- [ ] VM changes saved to config.yml
- [ ] SSH keys saved to key.kdbx
- [ ] Survives app restart

---

## Implementation Plan Summary

### Phase 1: Code Refactoring
1. Create `calc/hosting_gcp.rs` - VM operations
2. Create `calc/platform_gcp.rs` - Platform operations
3. Refactor `calc/platform.rs` - Remove GCP-specific code
4. Refactor `calc/hosting.rs` - Clean up unnecessary code

### Phase 2: Data Model
1. Add `gcp_selected_project_id` to `CloudPlatformConfig`
2. Update config save/load logic
3. Implement row building logic

### Phase 3: UI Implementation
1. Build custom table widget with egui
2. Implement row rendering with indentation
3. Add action buttons to rows
4. Implement status badges and icons

### Phase 4: GCP API Integration
1. Extend `GcpRestClient` with new methods
2. Implement IP detection (icanhazip.com)
3. Add firewall management APIs
4. Implement VM lifecycle APIs

### Phase 5: SSH & Background Tasks
1. Update `ssh::test_connection` to support keyring
2. Implement background task spawning
3. Add SSH status tracking
4. Implement task cleanup

### Phase 6: Actions & Dialogs
1. Implement confirmation dialogs
2. Add action handlers
3. Implement progress indicators
4. Add auto-refresh after actions

### Phase 7: Error Handling
1. Add API error handling
2. Implement SSH error handling
3. Add keyring error handling
4. Implement audit logging

### Phase 8: Testing
1. Write unit tests
2. Add integration tests with mocking
3. Manual testing checklist
4. Bug fixes and polish

---

## Success Criteria

1. **Functional:**
   - [ ] Table displays platform → project → VM hierarchy
   - [ ] All actions execute successfully (Delete, Regenerate, Restart, Update Firewall, Refresh)
   - [ ] SSH status updates automatically in background
   - [ ] All data persists to config.yml
   - [ ] SSH keys stored in keyring

2. **UX:**
   - [ ] Clear visual hierarchy with indentation
   - [ ] Intuitive action button placement
   - [ ] Helpful error messages with recovery options
   - [ ] Responsive UI with loading indicators

3. **Reliability:**
   - [ ] No data loss on errors
   - [ ] Graceful handling of network failures
   - [ ] OAuth token refresh works
   - [ ] Config saves atomic (no corruption)

4. **Performance:**
   - [ ] Table renders smoothly with 10+ platforms
   - [ ] Background SSH tests don't block UI
   - [ ] API calls complete within reasonable time (< 5s)

---

## Future Enhancements (Out of Scope)

- Support for multiple projects per platform
- Support for displaying multiple VMs per project
- Auto-refresh on timer
- VM creation wizard from platform tab
- Bulk operations (delete all VMs, restart all)
- Cost tracking and billing integration
- Firebase and Supabase platform integrations
- Export platform configuration
- Platform templates for quick setup
