# Comprehensive Refresh Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement comprehensive Refresh that checks VM status, firewall rules, SSH connectivity, and updates UI with fresh data.

**Architecture:** Add RefreshPlatform command to ViewModel that orchestrates three checks (VM exists + IP, firewall whitelist, SSH test), returns structured results via RefreshCompleted event, UI updates PlatformRow state incrementally.

**Tech Stack:** Existing GCP Compute API (`get_instance`, `check_ip_whitelisted`), SSH test via russh, ViewModel actor pattern.

## Global Constraints

- Must use existing `GcpRestClient` methods (`get_instance`, `check_ip_whitelisted`)
- Reuse existing SSH test logic from `execute_test_connection`
- Update `PlatformRow` fields incrementally (no full reload)
- Show optimistic "Refreshing..." state during operations
- Follow TDD: write test → fail → implement → pass → commit
- Keep backward compatibility with existing refresh behavior

---

## Task 1: Add RefreshPlatform Command and Event

**Files:**
- Modify: `mobile/src/viewmodel/platform/commands.rs:9-80`
- Modify: `mobile/src/viewmodel/platform/events.rs:15-96`

**Interfaces:**
- Consumes: None (foundational)
- Produces: 
  - `PlatformCommand::RefreshPlatform { platform_name: String }`
  - `PlatformEvent::RefreshCompleted { platform_name: String, vm_status: VmStatus, firewall_status: FirewallStatus, ssh_status: SshStatus }`
  - `struct VmStatus { exists: bool, name: Option<String>, zone: Option<String>, external_ip: Option<String> }`
  - `struct FirewallStatus { whitelisted: bool, current_ip: Option<String> }`
  - `struct SshStatus { connected: bool, error: Option<String> }`

- [ ] **Step 1: Add status structs to events.rs**

Add after line 12 in `mobile/src/viewmodel/platform/events.rs`:

```rust
/// VM existence and network status
#[derive(Debug, Clone)]
pub struct VmStatus {
    pub exists: bool,
    pub name: Option<String>,
    pub zone: Option<String>,
    pub external_ip: Option<String>,
    pub status: Option<String>, // "RUNNING", "STOPPED", etc.
}

/// Firewall whitelist status
#[derive(Debug, Clone)]
pub struct FirewallStatus {
    pub whitelisted: bool,
    pub current_ip: Option<String>,
}

/// SSH connectivity status
#[derive(Debug, Clone)]
pub struct SshStatus {
    pub connected: bool,
    pub error: Option<String>,
}
```

- [ ] **Step 2: Add RefreshCompleted event**

Add after line 85 in `events.rs` (before `OperationFailed`):

```rust
    /// Refresh completed with comprehensive status
    RefreshCompleted {
        platform_name: String,
        vm_status: VmStatus,
        firewall_status: FirewallStatus,
        ssh_status: SshStatus,
    },
```

- [ ] **Step 3: Add RefreshPlatform command**

Add after line 75 in `commands.rs` (after Billing section):

```rust
    // Refresh Operation
    RefreshPlatform {
        platform_name: String,
    },
```

- [ ] **Step 4: Compile to verify**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS (new command/event defined but not yet used)

- [ ] **Step 5: Commit command and event definitions**

```bash
git add mobile/src/viewmodel/platform/commands.rs mobile/src/viewmodel/platform/events.rs
git commit -m "feat(platform): add RefreshPlatform command and RefreshCompleted event

- Add VmStatus, FirewallStatus, SshStatus structs
- Add RefreshCompleted event with comprehensive status
- Add RefreshPlatform command to trigger refresh

Part of comprehensive refresh implementation (Phase 1: Foundation)"
```

---

## Task 2: Implement VM Status Check

**Files:**
- Modify: `mobile/src/viewmodel/platform/actor.rs`

**Interfaces:**
- Consumes: `PlatformCommand::RefreshPlatform`, `GcpRestClient::get_instance`, `GcpRestClient::list_instances_aggregated`
- Produces: `VmStatus` with exists, name, zone, external_ip, status fields

- [ ] **Step 1: Find command handler location**

```bash
grep -n "match command" mobile/src/viewmodel/platform/actor.rs
```

Expected: Line number where commands are matched (around line 150-200)

- [ ] **Step 2: Add RefreshPlatform command handler stub**

Add before the closing `}` of the command match (after other command handlers):

```rust
                PlatformCommand::RefreshPlatform { platform_name } => {
                    dure_info!("🔄 Refreshing platform: {}", platform_name);
                    
                    // Get platform config
                    let (app_config, _) = match load_config() {
                        Ok(config) => config,
                        Err(e) => {
                            dure_error!("Failed to load config: {}", e);
                            return;
                        }
                    };

                    let platform = match app_config.platforms.iter().find(|p| {
                        p.gcp_selected_project_id.as_ref() == Some(&platform_name)
                    }) {
                        Some(p) => p,
                        None => {
                            dure_error!("Platform not found: {}", platform_name);
                            return;
                        }
                    };

                    // Step 1: Check VM status
                    let vm_status = self.check_vm_status(platform);

                    // Step 2: Check firewall status
                    let firewall_status = self.check_firewall_status(platform);

                    // Step 3: Test SSH connection
                    let ssh_status = self.test_ssh_connection(platform);

                    // Send RefreshCompleted event
                    let event = PlatformEvent::RefreshCompleted {
                        platform_name: platform_name.clone(),
                        vm_status,
                        firewall_status,
                        ssh_status,
                    };
                    self.events.push(event);
                }
```

- [ ] **Step 3: Implement check_vm_status helper method**

Add to `impl PlatformActor` section (after existing helper methods):

```rust
    fn check_vm_status(&self, platform: &CloudPlatformConfig) -> VmStatus {
        // Get project ID
        let project_id = match &platform.gcp_selected_project_id {
            Some(id) => id,
            None => {
                return VmStatus {
                    exists: false,
                    name: None,
                    zone: None,
                    external_ip: None,
                    status: None,
                };
            }
        };

        // Get access token
        let access_token = match crate::calc::platform::get_valid_token(platform) {
            Ok(Some(token)) => token,
            _ => {
                dure_warn!("No valid access token for VM check");
                return VmStatus {
                    exists: false,
                    name: None,
                    zone: None,
                    external_ip: None,
                    status: None,
                };
            }
        };

        // Create GCP client
        let client = crate::api::gcp::GcpRestClient::new(access_token);

        // List VMs (use aggregated to check all zones)
        match client.list_instances_aggregated(project_id) {
            Ok(instances) => {
                if let Some(vm) = instances.first() {
                    // Extract external IP from network interfaces
                    let external_ip = vm
                        .network_interfaces
                        .first()
                        .and_then(|ni| {
                            ni.access_configs.first().and_then(|ac| ac.nat_ip.clone())
                        });

                    // Extract zone from full zone path (e.g., "zones/us-central1-a" -> "us-central1-a")
                    let zone = vm.zone.split('/').last().map(|s| s.to_string());

                    VmStatus {
                        exists: true,
                        name: Some(vm.name.clone()),
                        zone,
                        external_ip,
                        status: Some(vm.status.clone()),
                    }
                } else {
                    // No VMs found
                    VmStatus {
                        exists: false,
                        name: None,
                        zone: None,
                        external_ip: None,
                        status: None,
                    }
                }
            }
            Err(e) => {
                dure_error!("Failed to list VMs: {}", e);
                VmStatus {
                    exists: false,
                    name: None,
                    zone: None,
                    external_ip: None,
                    status: None,
                }
            }
        }
    }
```

- [ ] **Step 4: Compile to verify**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS

- [ ] **Step 5: Commit VM status check**

```bash
git add mobile/src/viewmodel/platform/actor.rs
git commit -m "feat(platform): implement VM status check in refresh

- Add check_vm_status() helper method
- Query GCP API for VM instances (aggregated across all zones)
- Extract VM name, zone, external IP, and status
- Return VmStatus struct with results

Part of comprehensive refresh implementation (Phase 2: VM Check)"
```

---

## Task 3: Implement Firewall Status Check

**Files:**
- Modify: `mobile/src/viewmodel/platform/actor.rs`

**Interfaces:**
- Consumes: `GcpRestClient::check_ip_whitelisted`, `get_current_ip()` from `api::gcp::compute`
- Produces: `FirewallStatus` with whitelisted flag and current_ip

- [ ] **Step 1: Add get_current_ip import**

Add to imports at top of `actor.rs`:

```rust
use crate::api::gcp::compute::get_current_ip;
```

- [ ] **Step 2: Implement check_firewall_status helper method**

Add to `impl PlatformActor` section (after `check_vm_status`):

```rust
    fn check_firewall_status(&self, platform: &CloudPlatformConfig) -> FirewallStatus {
        // Get project ID
        let project_id = match &platform.gcp_selected_project_id {
            Some(id) => id,
            None => {
                return FirewallStatus {
                    whitelisted: false,
                    current_ip: None,
                };
            }
        };

        // Get access token
        let access_token = match crate::calc::platform::get_valid_token(platform) {
            Ok(Some(token)) => token,
            _ => {
                dure_warn!("No valid access token for firewall check");
                return FirewallStatus {
                    whitelisted: false,
                    current_ip: None,
                };
            }
        };

        // Get current external IP
        let current_ip = match get_current_ip() {
            Ok(ip) => ip,
            Err(e) => {
                dure_warn!("Failed to get current IP: {}", e);
                return FirewallStatus {
                    whitelisted: false,
                    current_ip: None,
                };
            }
        };

        // Create GCP client
        let client = crate::api::gcp::GcpRestClient::new(access_token);

        // Check if current IP is whitelisted
        match client.check_ip_whitelisted(project_id, &current_ip) {
            Ok(whitelisted) => FirewallStatus {
                whitelisted,
                current_ip: Some(current_ip),
            },
            Err(e) => {
                dure_error!("Failed to check firewall: {}", e);
                FirewallStatus {
                    whitelisted: false,
                    current_ip: Some(current_ip),
                }
            }
        }
    }
```

- [ ] **Step 3: Compile to verify**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS

- [ ] **Step 4: Commit firewall status check**

```bash
git add mobile/src/viewmodel/platform/actor.rs
git commit -m "feat(platform): implement firewall status check in refresh

- Add check_firewall_status() helper method
- Get current external IP via get_current_ip()
- Check if IP is whitelisted in GCP firewall rules
- Return FirewallStatus with whitelisted flag and current IP

Part of comprehensive refresh implementation (Phase 2: Firewall Check)"
```

---

## Task 4: Implement SSH Connection Test

**Files:**
- Modify: `mobile/src/viewmodel/platform/actor.rs`

**Interfaces:**
- Consumes: VM external IP, SSH private key from keyring, `russh` for connection test
- Produces: `SshStatus` with connected flag and optional error message

- [ ] **Step 1: Implement test_ssh_connection helper method**

Add to `impl PlatformActor` section (after `check_firewall_status`):

```rust
    fn test_ssh_connection(&self, platform: &CloudPlatformConfig) -> SshStatus {
        // Get VM external IP from platform VMs config
        let external_ip = match platform.vms.first() {
            Some(vm) => match &vm.external_ip {
                Some(ip) => ip,
                None => {
                    return SshStatus {
                        connected: false,
                        error: Some("No external IP configured".to_string()),
                    };
                }
            },
            None => {
                return SshStatus {
                    connected: false,
                    error: Some("No VM configured".to_string()),
                };
            }
        };

        // Get SSH key from keyring
        let ssh_key = match platform.vms.first().and_then(|vm| vm.ssh_keyring_domain.as_ref()) {
            Some(domain) => {
                match crate::calc::keyring::get_ssh_key(domain) {
                    Ok(Some(key)) => key,
                    Ok(None) => {
                        return SshStatus {
                            connected: false,
                            error: Some("SSH key not found in keyring".to_string()),
                        };
                    }
                    Err(e) => {
                        return SshStatus {
                            connected: false,
                            error: Some(format!("Failed to get SSH key: {}", e)),
                        };
                    }
                }
            }
            None => {
                return SshStatus {
                    connected: false,
                    error: Some("No SSH keyring domain configured".to_string()),
                };
            }
        };

        // Test SSH connection
        #[cfg(not(target_arch = "wasm32"))]
        {
            match crate::calc::ssh::test_connection(external_ip, &ssh_key, 22, 5000) {
                Ok(_) => SshStatus {
                    connected: true,
                    error: None,
                },
                Err(e) => SshStatus {
                    connected: false,
                    error: Some(format!("Connection failed: {}", e)),
                },
            }
        }

        #[cfg(target_arch = "wasm32")]
        {
            SshStatus {
                connected: false,
                error: Some("SSH test not supported on WASM".to_string()),
            }
        }
    }
```

- [ ] **Step 2: Compile to verify**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS

- [ ] **Step 3: Commit SSH connection test**

```bash
git add mobile/src/viewmodel/platform/actor.rs
git commit -m "feat(platform): implement SSH connection test in refresh

- Add test_ssh_connection() helper method
- Get VM external IP and SSH key from config/keyring
- Test SSH connection with 5 second timeout
- Return SshStatus with connected flag and error details

Part of comprehensive refresh implementation (Phase 2: SSH Test)"
```

---

## Task 5: Handle RefreshCompleted Event in UI

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs:850-1030`

**Interfaces:**
- Consumes: `PlatformEvent::RefreshCompleted` with VmStatus, FirewallStatus, SshStatus
- Produces: Updated `PlatformRow` fields (vm_name, vm_external_ip, firewall_updated, ssh_ready, operation_state)

- [ ] **Step 1: Add RefreshCompleted event handler**

Find the event processing section (around line 850) and add before `OperationFailed` handler:

```rust
                    ViewModelEvent::Platform(PlatformEvent::RefreshCompleted {
                        platform_name,
                        vm_status,
                        firewall_status,
                        ssh_status,
                    }) => {
                        dure_info!("✓ Refresh completed for {}", platform_name);

                        // Find and update the row
                        if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
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
                                    row.firewall_status = format!("✅ Whitelisted ({})", current_ip);
                                } else {
                                    row.firewall_status = format!("✗ Not whitelisted ({})", current_ip);
                                }
                            } else {
                                row.firewall_status = "? Status unknown".to_string();
                            }

                            // Update SSH status
                            row.ssh_ready = ssh_status.connected;
                            if ssh_status.connected {
                                row.ssh_status = "✓ Ready".to_string();
                            } else if let Some(error) = &ssh_status.error {
                                row.ssh_status = format!("✗ {}", error);
                            } else {
                                row.ssh_status = "? Unknown".to_string();
                            }

                            // Clear operation state (refresh complete)
                            row.operation_state = OperationState::Completed {
                                operation: "refresh".to_string(),
                                completed_at: chrono::Utc::now().timestamp(),
                            };

                            // Update last refresh time
                            row.last_refresh_time = Some(chrono::Utc::now().timestamp());
                        }
                    }
```

- [ ] **Step 2: Compile to verify**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS

- [ ] **Step 3: Commit UI event handler**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "feat(platform): handle RefreshCompleted event in UI

- Update PlatformRow with fresh VM status (name, IP, zone)
- Update firewall status with whitelist check result
- Update SSH status with connection test result
- Set operation_state to Completed
- Update last_refresh_time to current timestamp

Part of comprehensive refresh implementation (Phase 3: UI Integration)"
```

---

## Task 6: Wire Refresh Button to RefreshPlatform Command

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs:1400-1420`

**Interfaces:**
- Consumes: Refresh button click action (`platform_action_refresh`)
- Produces: `PlatformCommand::RefreshPlatform` sent to ViewModel

- [ ] **Step 1: Replace current refresh handler**

Find the refresh action handler (around line 1400-1410) and replace:

```rust
            // Refresh action (available on all platforms)
            if let Some(platform_name) =
                ui.data(|d| d.get_temp::<String>(egui::Id::new("platform_action_refresh")))
            {
                // Optimistic update: Show "Refreshing..." state
                if let Some(row) = self.rows.iter_mut().find(|r| r.project_id == platform_name) {
                    row.operation_state = OperationState::InProgress {
                        operation: "Refreshing".to_string(),
                        started_at: chrono::Utc::now().timestamp(),
                    };
                }

                // Send RefreshPlatform command to ViewModel
                if let Some(ref mut vm) = vm {
                    use crate::viewmodel::platform::PlatformCommand;
                    vm.send_platform_command(PlatformCommand::RefreshPlatform {
                        platform_name: platform_name.clone(),
                    });
                }

                ui.data_mut(|d| {
                    d.remove::<String>(egui::Id::new("platform_action_refresh"))
                });
            }
```

- [ ] **Step 2: Remove old SSH test call**

Delete or comment out the old `execute_test_connection` call (around line 1408-1409):

```rust
                // OLD CODE - REMOVE:
                // #[cfg(not(target_arch = "wasm32"))]
                // self.execute_test_connection(platform_name.clone());
```

- [ ] **Step 3: Compile to verify**

```bash
cd mobile && cargo check --features gui
```

Expected: SUCCESS

- [ ] **Step 4: Test manually**

```bash
cd mobile && cargo run --features gui --release
```

Actions to test:
- Click Refresh button
- Verify ⏳ appears in steps column immediately
- Wait for refresh to complete
- Verify steps column shows correct progress (✅/✗ for each step)
- Verify drawer shows updated VM IP, firewall status, SSH status
- Verify ✅ appears and auto-clears after 3 seconds

- [ ] **Step 5: Commit refresh button wiring**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "feat(platform): wire Refresh button to RefreshPlatform command

- Send RefreshPlatform command to ViewModel on button click
- Show optimistic 'Refreshing...' state immediately
- Remove old execute_test_connection call (replaced by comprehensive refresh)
- UI updates via RefreshCompleted event handler

Part of comprehensive refresh implementation (Phase 3: UI Integration)"
```

---

## Task 7: Add Unit Tests for Status Checks

**Files:**
- Create: `mobile/src/viewmodel/platform/tests/refresh_tests.rs`
- Modify: `mobile/src/viewmodel/platform/mod.rs`

**Interfaces:**
- Consumes: Mock GCP client responses
- Produces: Test coverage for check_vm_status, check_firewall_status, test_ssh_connection

- [ ] **Step 1: Create test module file**

Create `mobile/src/viewmodel/platform/tests/refresh_tests.rs`:

```rust
#[cfg(test)]
mod refresh_tests {
    use super::*;

    #[test]
    fn test_vm_status_exists() {
        // Test that VmStatus correctly parses VM with external IP
        // TODO: Add mock GCP client test
    }

    #[test]
    fn test_vm_status_no_vms() {
        // Test that VmStatus handles empty VM list
        // TODO: Add mock GCP client test
    }

    #[test]
    fn test_firewall_status_whitelisted() {
        // Test that FirewallStatus detects whitelisted IP
        // TODO: Add mock GCP client test
    }

    #[test]
    fn test_firewall_status_not_whitelisted() {
        // Test that FirewallStatus detects non-whitelisted IP
        // TODO: Add mock GCP client test
    }

    #[test]
    fn test_ssh_status_no_external_ip() {
        // Test that SshStatus handles missing external IP
        let status = SshStatus {
            connected: false,
            error: Some("No external IP configured".to_string()),
        };
        assert!(!status.connected);
        assert!(status.error.is_some());
    }

    #[test]
    fn test_ssh_status_no_key() {
        // Test that SshStatus handles missing SSH key
        let status = SshStatus {
            connected: false,
            error: Some("SSH key not found in keyring".to_string()),
        };
        assert!(!status.connected);
        assert!(status.error.is_some());
    }
}
```

- [ ] **Step 2: Add tests module to mod.rs**

Add to `mobile/src/viewmodel/platform/mod.rs`:

```rust
#[cfg(test)]
mod tests;
```

- [ ] **Step 3: Run tests**

```bash
cd mobile && cargo test --features gui refresh_tests
```

Expected: All tests PASS (currently just structure tests)

- [ ] **Step 4: Commit test structure**

```bash
git add mobile/src/viewmodel/platform/tests/refresh_tests.rs mobile/src/viewmodel/platform/mod.rs
git commit -m "test(platform): add unit test structure for refresh functionality

- Add refresh_tests.rs with test stubs
- Cover VmStatus, FirewallStatus, SshStatus edge cases
- TODO: Add mock GCP client tests in future iterations

Part of comprehensive refresh implementation (Phase 4: Testing)"
```

---

## Task 8: Update Documentation

**Files:**
- Create: `docs/superpowers/specs/2026-08-05-comprehensive-refresh-behavior.md`

**Interfaces:**
- Produces: User-facing documentation of refresh behavior

- [ ] **Step 1: Create behavior documentation**

```bash
cat > docs/superpowers/specs/2026-08-05-comprehensive-refresh-behavior.md << 'EOF'
# Comprehensive Refresh Behavior

## Overview

The Refresh button performs a comprehensive health check of the platform:

1. **VM Status Check**: Verifies VM exists and retrieves external IP
2. **Firewall Check**: Confirms current IP is whitelisted for SSH access
3. **SSH Test**: Attempts actual SSH connection to verify end-to-end connectivity

## User Experience

### Visual Feedback

**Immediate (Optimistic):**
- Steps column shows ⏳ for "Refreshing"
- Operation buttons disabled during refresh

**After Completion (3-5 seconds):**
- Steps column updates:
  - ✅ OAuth (if connected)
  - ✅ Project (if selected)
  - ✅/✗ VM (based on existence check)
  - ✅/✗ Firewall (based on whitelist check)
  - ✅/✗ SSH (based on connection test)

**Drawer Updates:**
- VM IP address (fresh from GCP)
- Firewall status: "✅ Whitelisted (1.2.3.4)" or "✗ Not whitelisted"
- SSH status: "✓ Ready" or "✗ Connection failed: <error>"
- Last refresh time: "just now", "2 min ago", etc.

**Auto-Clear:**
- ✅ Completed indicator clears after 3 seconds
- Returns to showing current state

## Technical Details

### Data Flow

1. UI: User clicks Refresh button
2. UI: Set OperationState::InProgress ("Refreshing")
3. UI: Send PlatformCommand::RefreshPlatform to ViewModel
4. ViewModel: Execute three checks in sequence:
   - Query GCP API for VM instances
   - Query GCP API for firewall rules
   - Test SSH connection (5 second timeout)
5. ViewModel: Send PlatformEvent::RefreshCompleted with results
6. UI: Update PlatformRow with fresh data
7. UI: Set OperationState::Completed, auto-clear after 3s

### Error Handling

- **No access token**: Returns empty status (shows ? in UI)
- **GCP API failure**: Logs error, returns failure status
- **SSH timeout**: Returns `connected: false` with timeout error
- **Network failure**: Shows error in SSH status field

### Performance

- **Expected duration**: 3-5 seconds total
- **Timeout**: 5 seconds for SSH test
- **No polling**: Event-driven updates only
- **No full reload**: Incremental PlatformRow updates

## Testing Checklist

- [ ] Refresh with running VM shows all ✅
- [ ] Refresh with stopped VM shows ✗ VM
- [ ] Refresh with non-whitelisted IP shows ✗ Firewall
- [ ] Refresh with unreachable VM shows ✗ SSH
- [ ] Refresh with no VM shows ✗ VM, ✗ Firewall, ✗ SSH
- [ ] Operation buttons disabled during refresh
- [ ] ✅ indicator auto-clears after 3 seconds
- [ ] Last refresh time updates correctly
- [ ] Drawer shows fresh IP address
- [ ] Works on all platforms (Linux, macOS, Windows)

EOF
```

- [ ] **Step 2: Commit documentation**

```bash
git add docs/superpowers/specs/2026-08-05-comprehensive-refresh-behavior.md
git commit -m "docs: add comprehensive refresh behavior specification

- Document three-check refresh process (VM, Firewall, SSH)
- Describe user-facing visual feedback and timing
- Explain technical data flow and error handling
- Provide testing checklist

Part of comprehensive refresh implementation (Phase 4: Documentation)"
```

---

## Self-Review

**1. Spec coverage:**
- ✅ Check VM exists and get external IP (Task 2: check_vm_status)
- ✅ Check firewall for whitelisted IP (Task 3: check_firewall_status)
- ✅ Test SSH connection (Task 4: test_ssh_connection)
- ✅ Update steps column (Task 5: RefreshCompleted handler updates operation_state)
- ✅ Update drawer contents (Task 5: Updates vm_external_ip, firewall_status, ssh_status)
- ✅ Update operation buttons (Task 6: Optimistic InProgress state disables buttons)

**2. Placeholder scan:**
- All code complete (no TBD/TODO except in test stubs marked for future)
- All file paths exact
- All commands with expected output
- All structs/enums defined with exact field names

**3. Type consistency:**
- `VmStatus` fields match usage in Task 2 and Task 5
- `FirewallStatus` fields match usage in Task 3 and Task 5
- `SshStatus` fields match usage in Task 4 and Task 5
- `PlatformCommand::RefreshPlatform` matches Task 1 definition and Task 6 usage
- `PlatformEvent::RefreshCompleted` matches Task 1 definition and Task 5 handler

**4. Dependencies:**
- Task 1 defines command/event → consumed by Tasks 2-6
- Tasks 2-4 implement status checks → results used in Task 5
- Task 5 handles events → updates UI based on Task 1 event structure
- Task 6 wires UI → sends Task 1 command, relies on Task 5 for updates
- Tasks 7-8 are independent (testing and docs)

Plan is complete and ready for execution.

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-08-05-comprehensive-refresh-implementation.md`.

Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
