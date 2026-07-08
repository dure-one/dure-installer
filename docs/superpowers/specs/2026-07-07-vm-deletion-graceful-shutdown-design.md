# VM Deletion with Graceful Shutdown Option & Countdown Timer

**Date:** 2026-07-07  
**Status:** Approved  
**Author:** Claude Sonnet 4.5 + nikescar@gmail.com

## Summary

Add a "Skip graceful shutdown" option to VM deletion operations with a countdown timer to show deletion progress. This addresses the issue where VM deletion takes a long time and provides user feedback on operation status.

## Requirements

1. Add "Skip graceful shutdown" checkbox to both Delete Platform and Delete VM dialogs
2. Checkbox defaults to **unchecked** (graceful shutdown ON for safety)
3. When checked, use GCP API `noGracefulShutdown=true` query parameter
4. Show countdown timer on delete button during deletion
5. Countdown durations:
   - **Skip graceful shutdown = checked:** 30 seconds initial
   - **Graceful shutdown = unchecked (default):** 2 minutes initial
6. Auto-extend countdown by 30 seconds if operation not complete
7. Poll GCP operation status when countdown expires
8. Maximum total wait time: 10 minutes (then timeout)
9. Works for both single VM deletion and platform deletion (multiple VMs)

## Architecture Overview

The feature follows the existing layered architecture:

### UI Layer (`mobile/src/ui_tabs/platform.rs`)
- Delete Platform dialog: shows "Skip graceful shutdown" checkbox + delete button
- Delete VM dialog: same checkbox + delete button
- During deletion: button replaced with countdown text "Deleting... 1:45"
- Listens to `PlatformEvent::Progress` events to update countdown

### ViewModel Layer (`mobile/src/viewmodel/platform/`)
- `DeleteOptions` struct adds `skip_graceful_shutdown: bool` field
- `PlatformCommand::DeleteVM` adds `skip_graceful_shutdown: bool` parameter
- Actor sends progress events every 1 second with remaining time
- Manages countdown timer, GCP operation polling, and auto-extension logic

### API Layer (`mobile/src/api/gcp/compute.rs`)
- `delete_instance()` adds optional `no_graceful_shutdown: bool` parameter
- Appends `?noGracefulShutdown=true` query parameter when true
- `get_operation()` method added to poll operation status by operation ID

### Data Flow

UI checkbox state → DeleteOptions → Actor (manages timer + polling) → GCP API (with/without query param) → Progress events → UI countdown display

## Components

### Modified Structs

**`mobile/src/viewmodel/platform/commands.rs`**

```rust
pub struct DeleteOptions {
    pub delete_vms: bool,
    pub delete_project: bool,
    pub skip_graceful_shutdown: bool,  // NEW
}

pub enum PlatformCommand {
    DeleteVM {
        platform_name: String,
        vm_name: String,
        zone: String,
        skip_graceful_shutdown: bool,  // NEW
    },
    DeletePlatform {
        platform_name: String,
        delete_options: DeleteOptions,
    },
    // ... other variants
}
```

### New API Methods

**`mobile/src/api/gcp/compute.rs`**

```rust
impl GcpRestClient {
    /// Delete VM instance with optional graceful shutdown skip
    pub fn delete_instance(
        &self,
        project_id: &str,
        zone: &str,
        instance_name: &str,
        no_graceful_shutdown: bool,  // NEW parameter
    ) -> Result<Operation>;
    
    /// Get operation status by operation ID
    pub fn get_operation(  // NEW method
        &self,
        project_id: &str,
        zone: &str,
        operation_id: &str,
    ) -> Result<Operation>;
}
```

**Implementation Details:**

```rust
pub fn delete_instance(
    &self,
    project_id: &str,
    zone: &str,
    instance_name: &str,
    no_graceful_shutdown: bool,
) -> Result<Operation> {
    let mut url = format!(
        "{}/projects/{}/zones/{}/instances/{}",
        GCP_COMPUTE_API_BASE, project_id, zone, instance_name
    );
    
    if no_graceful_shutdown {
        url.push_str("?noGracefulShutdown=true");
    }
    
    let response = self.delete(&url)?;
    let operation: Operation = response.into_json()?;
    Ok(operation)
}

pub fn get_operation(
    &self,
    project_id: &str,
    zone: &str,
    operation_id: &str,
) -> Result<Operation> {
    let url = format!(
        "{}/projects/{}/zones/{}/operations/{}",
        GCP_COMPUTE_API_BASE, project_id, zone, operation_id
    );
    
    let response = self.get(&url)?;
    let operation: Operation = response.into_json()?;
    Ok(operation)
}
```

### UI State Changes

**`mobile/src/ui_tabs/platform.rs`**

Add to `PlatformTab` struct:

```rust
// Delete Platform dialog
delete_platform_skip_graceful: bool,  // NEW

// Delete VM dialog
delete_vm_skip_graceful: bool,  // NEW
```

### Actor Timer State

**`mobile/src/viewmodel/platform/actor.rs`**

Track active deletion operations:

```rust
struct DeletionTracker {
    operation_id: String,
    project_id: String,
    zone: String,
    vm_name: String,
    start_time: std::time::Instant,
    initial_duration_secs: u64,  // 30 or 120
    countdown_remaining_secs: u64,
}
```

## Data Flow

### Delete Flow (step-by-step)

1. **User Action:** User checks/unchecks "Skip graceful shutdown", clicks Delete

2. **UI → ViewModel:** Send `DeleteVM` or `DeletePlatform` command with `skip_graceful_shutdown` flag

3. **Actor Initialization:**
   - Determine initial countdown: 30s (skip) or 120s (graceful)
   - Call GCP API `delete_instance()` with `no_graceful_shutdown` parameter
   - Receive `Operation` response with `operation_id`
   - Store operation metadata (id, start_time, countdown_remaining)
   - Spawn background task for countdown + polling

4. **Countdown Loop (runs in background task):**
   ```
   every 1 second:
     - countdown_remaining -= 1
     - send Progress event: "Deleting... 1:23"
     - if countdown_remaining == 0:
         check operation status via get_operation()
         if DONE: send success event, exit
         if RUNNING: extend by 30s, continue loop
         if ERROR: send error event, exit
   ```

5. **UI Update:** On each `Progress` event, replace button text with countdown

6. **Completion:**
   - Success: Restore delete button, refresh table
   - Error: Show error message, restore delete button

### Multiple VM Deletion (Delete Platform)

- Process VMs sequentially (one countdown at a time)
- Total progress shows "Deleting VM 2/3... 0:45"
- If one VM fails, continue with next VM (don't abort entire operation)

## Error Handling

### GCP API Failures

- **Initial delete_instance() fails:** Send `PlatformEvent::Error` immediately, no countdown starts
- **get_operation() polling fails:** Retry up to 3 times with 2s backoff, then fail with error event
- **Operation returns ERROR status:** Stop countdown, send error event with GCP error message

### Timeout Protection

- **Max total wait time:** 10 minutes (regardless of auto-extensions)
- After 10 minutes, stop polling and show: "Deletion taking longer than expected. Check GCP Console for status."
- User can manually refresh table to see if VM is gone

### Network Errors

- Transient network failures during polling: retry with exponential backoff (2s, 4s, 8s)
- If all retries fail: show error but suggest manual refresh

### Edge Cases

- **User closes dialog during deletion:** Countdown continues in background (operation already started in GCP)
- **App restart during deletion:** Operation state is lost, but GCP deletion continues server-side
- **Multiple rapid delete clicks:** Disable button after first click (prevent duplicate operations)

### Error Messages

- **GCP errors:** Show exact error from GCP API response
- **Timeout:** "VM deletion exceeded 10 minutes. Please check GCP Console."
- **Network:** "Network error while checking status. Click Refresh to retry."

## Testing Strategy

### Unit Tests

**`mobile/src/viewmodel/platform/commands.rs`**

```rust
#[test]
fn test_delete_options_with_skip_graceful() {
    let opts = DeleteOptions {
        delete_vms: true,
        delete_project: false,
        skip_graceful_shutdown: true,
    };
    assert!(opts.skip_graceful_shutdown);
}

#[test]
fn test_delete_vm_command_with_skip_graceful() {
    let cmd = PlatformCommand::DeleteVM {
        platform_name: "test".to_string(),
        vm_name: "test-vm".to_string(),
        zone: "us-central1-a".to_string(),
        skip_graceful_shutdown: true,
    };
    // Assert command structure
}
```

**`mobile/src/api/gcp/compute.rs`**

```rust
#[test]
fn test_delete_instance_with_no_graceful_shutdown() {
    // Mock: verify URL includes ?noGracefulShutdown=true
}

#[test]
fn test_delete_instance_without_no_graceful_shutdown() {
    // Mock: verify URL does NOT include query parameter
}

#[test]
fn test_get_operation_success() {
    // Mock: operation status DONE
}

#[test]
fn test_get_operation_running() {
    // Mock: operation status RUNNING
}

#[test]
fn test_get_operation_error() {
    // Mock: operation status ERROR
}
```

### Actor Tests (countdown logic)

**`mobile/src/viewmodel/platform/tests.rs`**

```rust
#[test]
fn test_countdown_initial_duration_skip_graceful() {
    // Verify 30s for skip_graceful_shutdown=true
}

#[test]
fn test_countdown_initial_duration_graceful() {
    // Verify 120s for skip_graceful_shutdown=false
}

#[test]
fn test_countdown_auto_extension() {
    // Mock: operation still RUNNING after initial countdown
    // Verify countdown extends by 30s
}

#[test]
fn test_countdown_max_timeout() {
    // Mock: operation never completes
    // Verify stops after 10 minutes with timeout error
}

#[test]
fn test_countdown_progress_events() {
    // Verify Progress events sent every second
    // Verify countdown format "Deleting... 1:23"
}

#[test]
fn test_multiple_vm_deletion_sequential() {
    // Mock: delete platform with 3 VMs
    // Verify VMs deleted sequentially with separate countdowns
    // Verify progress shows "Deleting VM 2/3... 0:45"
}
```

### Integration Test Approach

- Use mock GCP client with controllable operation status
- Test full flow: command → countdown → polling → completion
- Time-based tests use mocked time (not real 2-minute waits)
- Verify event sequences: Progress events → Success/Error event

### Manual Testing Checklist

- [ ] Delete single VM with graceful shutdown (2 min countdown)
- [ ] Delete single VM without graceful shutdown (30s countdown)
- [ ] Delete platform with multiple VMs (sequential countdowns)
- [ ] Verify GCP operation completes before countdown (early completion)
- [ ] Verify auto-extension when operation takes longer
- [ ] Network error during polling (retry behavior)
- [ ] Close dialog during deletion (background continuation)
- [ ] Max timeout at 10 minutes
- [ ] Error from GCP during deletion
- [ ] Checkbox state persists across dialog open/close

## Implementation Notes

### Timer Implementation

Use `smol::Timer` for async countdown:

```rust
async fn countdown_and_poll(&mut self, tracker: DeletionTracker) {
    loop {
        if tracker.countdown_remaining_secs > 0 {
            // Send progress event
            self.send_progress(
                "delete_vm",
                0.5,
                &format!("Deleting... {}:{:02}", 
                    tracker.countdown_remaining_secs / 60,
                    tracker.countdown_remaining_secs % 60)
            ).await;
            
            smol::Timer::after(Duration::from_secs(1)).await;
            tracker.countdown_remaining_secs -= 1;
        } else {
            // Poll operation status
            let operation = self.check_operation(&tracker).await?;
            
            if operation.status == "DONE" {
                // Success
                break;
            } else if operation.status == "RUNNING" {
                // Extend by 30s
                tracker.countdown_remaining_secs = 30;
            } else {
                // Error
                return Err(anyhow::anyhow!("Operation failed: {:?}", operation.error));
            }
        }
        
        // Check max timeout
        if tracker.start_time.elapsed() > Duration::from_secs(600) {
            return Err(anyhow::anyhow!("Operation timeout after 10 minutes"));
        }
    }
}
```

### UI Button State

Current button text logic:

```rust
// Before deletion
if ui.button("Delete").clicked() {
    // Start deletion
}

// During deletion (with countdown)
ui.add_enabled(false, egui::Button::new("Deleting... 1:45"));
```

### Event Handling

Existing `PlatformEvent::Progress` already supports countdown display:

```rust
PlatformEvent::Progress { operation, progress, status } => {
    // Update UI with status text (contains countdown)
}
```

## Open Questions

None - all requirements clarified during design phase.

## Success Criteria

1. ✅ "Skip graceful shutdown" checkbox appears in both delete dialogs
2. ✅ Checkbox defaults to unchecked (graceful shutdown ON)
3. ✅ Countdown shows on delete button during deletion
4. ✅ Correct timings: 30s (skip) vs 2 min (graceful)
5. ✅ Auto-extends by 30s when operation not complete
6. ✅ Times out after 10 minutes with error message
7. ✅ All unit tests pass
8. ✅ Manual testing checklist complete
9. ✅ Works for both single VM and platform deletion
10. ✅ GCP API receives correct query parameter
