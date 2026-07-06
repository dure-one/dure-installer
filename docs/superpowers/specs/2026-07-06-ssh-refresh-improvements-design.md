# SSH Tab Refresh Improvements Design Spec

**Date:** 2026-07-06  
**Status:** Approved  
**Goal:** Fix SSH tab refresh behavior - auto-refresh on first load, prevent continuous firing, and show visual feedback

## Problem Statement

The SSH tab refresh functionality has three critical issues:

1. **No auto-refresh on first load** - Host statuses (Linux, Docker, Ansible, Dure-WSS) are never queried when the tab is first opened, leaving status indicators blank
2. **Continuous refresh firing** - Refresh button temp data isn't cleared after reading, causing status queries to fire every frame (hundreds of times)
3. **No visual feedback** - Button provides no indication that a refresh is in progress

## Architecture

### State Management

Add per-row refresh state tracking to `SshRowData`:

```rust
struct SshRowData {
    // ... existing fields ...
    
    /// Is this row currently refreshing?
    refreshing: bool,
    
    /// Number of pending status responses (0-4)
    /// Decremented as each status event arrives
    refresh_pending_count: u8,
}
```

### Refresh Lifecycle

```
┌─────────────────┐
│ Button Clicked  │
└────────┬────────┘
         │
         ▼
┌─────────────────────────────┐
│ Clear temp data             │
│ Set refreshing = true       │
│ Set pending_count = 4       │
└────────┬────────────────────┘
         │
         ▼
┌─────────────────────────────┐
│ Send 4 status queries:      │
│ - get_linux_status()        │
│ - get_docker_status()       │
│ - get_ansible_status()      │
│ - get_dure_wss_status()     │
└────────┬────────────────────┘
         │
         ▼
┌─────────────────────────────┐
│ Each status event received: │
│ - Update row data           │
│ - pending_count -= 1        │
└────────┬────────────────────┘
         │
         ▼
┌─────────────────────────────┐
│ pending_count == 0?         │
│ → Set refreshing = false    │
└─────────────────────────────┘
```

## Component Changes

### 1. SshRowData Structure

**File:** `mobile/src/ui_tabs/ssh.rs`

Add two fields to `SshRowData` struct (around line 38):
```rust
refreshing: bool,
refresh_pending_count: u8,
```

Initialize in `load_rows()` method:
```rust
self.rows.push(SshRowData {
    // ... existing fields ...
    refreshing: false,
    refresh_pending_count: 0,
});
```

Update `Default` implementation if applicable.

### 2. Auto-Refresh on First Load

**File:** `mobile/src/ui_tabs/ssh.rs`  
**Location:** `ui()` method, after `load_rows()` (around line 1422-1426)

**Current code:**
```rust
if !self.loaded {
    self.load_rows();
    self.loaded = true;
}
```

**Change to:**
```rust
if !self.loaded {
    self.load_rows();
    self.loaded = true;
    
    // Auto-refresh all rows on first load
    if let Some(ref mut vm) = vm {
        for row in &mut self.rows {
            row.refreshing = true;
            row.refresh_pending_count = 4;
            let _ = vm.get_linux_status(row.host.clone());
            let _ = vm.get_docker_status(row.host.clone());
            let _ = vm.get_ansible_status(row.host.clone());
            let _ = vm.get_dure_wss_status(row.host.clone());
        }
    }
}
```

### 3. Refresh Button Handler

**File:** `mobile/src/ui_tabs/ssh.rs`  
**Location:** `process_action_triggers()` method (around line 610-617)

**Current code:**
```rust
if let Some(host) =
    ui.data(|d| d.get_temp::<String>(egui::Id::new(format!("ssh_refresh_{}", idx))))
{
    let _ = vm.get_linux_status(host.clone());
    let _ = vm.get_docker_status(host.clone());
    let _ = vm.get_ansible_status(host.clone());
    let _ = vm.get_dure_wss_status(host);
}
```

**Change to:**
```rust
let refresh_id = egui::Id::new(format!("ssh_refresh_{}", idx));
if let Some(host) = ui.data(|d| d.get_temp::<String>(refresh_id)) {
    // Clear temp data immediately to prevent continuous firing
    ui.data_mut(|d| d.remove::<String>(refresh_id));
    
    // Only start refresh if not already refreshing
    if let Some(row) = self.rows.get_mut(idx) {
        if !row.refreshing {
            row.refreshing = true;
            row.refresh_pending_count = 4;
            
            let _ = vm.get_linux_status(host.clone());
            let _ = vm.get_docker_status(host.clone());
            let _ = vm.get_ansible_status(host.clone());
            let _ = vm.get_dure_wss_status(host);
        }
    }
}
```

### 4. Button Visual Feedback

**File:** `mobile/src/ui_tabs/ssh.rs`  
**Location:** `render_table()` method, operations column (around line 2071)

**Current code:**
```rust
if ui.add(MaterialButton::text("⟳ Refresh")).clicked() {
    ui.data_mut(|d| {
        d.insert_temp(
            egui::Id::new(format!("ssh_refresh_{}", idx)),
            row.host.clone(),
        )
    });
}
```

**Change to:**
```rust
if row.refreshing {
    ui.add_enabled(false, MaterialButton::text("⟳ Refreshing..."));
} else {
    if ui.add(MaterialButton::text("⟳ Refresh")).clicked() {
        ui.data_mut(|d| {
            d.insert_temp(
                egui::Id::new(format!("ssh_refresh_{}", idx)),
                row.host.clone(),
            )
        });
    }
}
```

### 5. Event Handlers

**File:** `mobile/src/ui_tabs/ssh.rs`  
**Location:** `handle_event()` method

Add a helper method to `SshTab`:
```rust
/// Decrement refresh counter and clear refreshing flag when complete
fn decrement_refresh_counter(&mut self, host: &str) {
    if let Some(row) = self.rows.iter_mut().find(|r| r.host == host) {
        if row.refreshing && row.refresh_pending_count > 0 {
            row.refresh_pending_count -= 1;
            if row.refresh_pending_count == 0 {
                row.refreshing = false;
            }
        }
    }
}
```

Update event handlers to call this method:

**LinuxStatusRetrieved:**
```rust
ViewModelEvent::Ssh(SshEvent::LinuxStatusRetrieved { name, .. }) => {
    // ... existing status update code ...
    self.decrement_refresh_counter(&name);
}
```

**DockerStatusRetrieved:**
```rust
ViewModelEvent::Ssh(SshEvent::DockerStatusRetrieved { name, .. }) => {
    // ... existing status update code ...
    self.decrement_refresh_counter(&name);
}
```

**AnsibleStatusRetrieved:**
```rust
ViewModelEvent::Ssh(SshEvent::AnsibleStatusRetrieved { name, .. }) => {
    // ... existing status update code ...
    self.decrement_refresh_counter(&name);
}
```

**DureWssStatusRetrieved:**
```rust
ViewModelEvent::Ssh(SshEvent::DureWssStatusRetrieved { name, .. }) => {
    // ... existing status update code ...
    self.decrement_refresh_counter(&name);
}
```

## Error Handling

If any status check fails (ServiceError or Error event), still decrement the refresh counter to prevent the button from getting stuck:

```rust
ViewModelEvent::Ssh(SshEvent::ServiceError { name, .. }) => {
    // ... existing error handling ...
    self.decrement_refresh_counter(&name);
}

ViewModelEvent::Ssh(SshEvent::Error { operation, .. }) => {
    // Detect which host this error is for (parse operation string)
    // and decrement counter if it's a status operation
    if operation.contains("get_") && operation.contains("_status") {
        // Extract host from operation context if available
        // self.decrement_refresh_counter(&host);
    }
}
```

**Note:** Error events may not always include host information. In practice, most status operations succeed or timeout gracefully, so stuck refresh states should be rare. Consider adding a timeout mechanism (e.g., auto-clear after 30 seconds) if this becomes an issue.

## Edge Cases

1. **User closes tab while refreshing** - State is not persisted (marked with `#[serde(skip)]`), so reopening the tab will start fresh
2. **Multiple rapid clicks** - Button is disabled while `refreshing == true`, preventing duplicate requests
3. **Partial refresh failure** - Counter still decrements to prevent stuck state
4. **Tab switch during refresh** - Background refresh continues, button updates when user returns

## Testing

### Manual Verification

1. **Auto-refresh on load:**
   - Open SSH tab for the first time
   - Verify all refresh buttons show "⟳ Refreshing..."
   - Wait 2-3 seconds for status queries to complete
   - Verify buttons return to "⟳ Refresh" state
   - Verify status indicators update (Linux, Docker, Ansible, Dure-WSS)

2. **Manual refresh:**
   - Click refresh button on one row
   - Verify only that button changes to "⟳ Refreshing..."
   - Verify other rows remain in normal state
   - Verify button re-enables after ~2-3 seconds

3. **Prevent spam:**
   - Click refresh button rapidly multiple times
   - Verify only one set of status queries is sent
   - Verify button stays disabled during refresh
   - Check logs: should see only 4 status commands (not hundreds)

4. **Error handling:**
   - Disconnect network or stop SSH host
   - Click refresh button
   - Verify button eventually re-enables even if queries fail
   - Verify error is displayed in UI

### Expected Log Output

**First load (single host):**
```
🔍 SSH Actor: Received command: GetLinuxStatus { name: "root@136.66.81.199" }
🔍 SSH Actor: Received command: GetDockerStatus { name: "root@136.66.81.199" }
🔍 SSH Actor: Received command: GetAnsibleStatus { name: "root@136.66.81.199" }
🔍 SSH Actor: Received command: GetDureWssStatus { name: "root@136.66.81.199" }
[... status responses ...]
```

**Single refresh click:**
```
🔍 SSH Actor: Received command: GetLinuxStatus { name: "root@136.66.81.199" }
🔍 SSH Actor: Received command: GetDockerStatus { name: "root@136.66.81.199" }
🔍 SSH Actor: Received command: GetAnsibleStatus { name: "root@136.66.81.199" }
🔍 SSH Actor: Received command: GetDureWssStatus { name: "root@136.66.81.199" }
[... status responses ...]
```

**Should NOT see:** Repeated commands flooding the log every frame.

## Implementation Notes

- All changes are contained within `mobile/src/ui_tabs/ssh.rs`
- No changes required to ViewModel or actor code
- No database or config changes
- No new dependencies
- State is ephemeral (not persisted to config file)

## Success Criteria

1. ✅ SSH tab auto-refreshes all rows on first display
2. ✅ Refresh button shows "Refreshing..." state during operation
3. ✅ Refresh button click triggers exactly 4 status queries (no continuous firing)
4. ✅ Button re-enables after all status responses received or timeout
5. ✅ Spam-clicking refresh button is safely ignored
