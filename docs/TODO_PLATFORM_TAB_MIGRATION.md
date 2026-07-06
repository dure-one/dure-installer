# Platform Tab Migration TODO

## Status
**Signature updated** - Platform tab now accepts `Option<&mut ViewModel>` parameter.
**Pattern established** - Example code in comments shows where ViewModel would be used.
**Backward compatible** - Existing poll-promise code still functional when vm is None.

## Remaining Work

### 1. Remove Poll-Promise State Fields

From `PlatformTab` struct (lines 62+), remove:

```rust
// Line 82-83: OAuth promise
add_platform_oauth_promise: Option<poll_promise::Promise<...>>,

// Line 93: Init progress (use vm.active_operations() instead)
init_in_progress: bool,

// Line 161: SSH test promises (use ViewModel events instead)
ssh_test_promises: HashMap<String, poll_promise::Promise<...>>,
```

### 2. Replace Poll-Promise Code Blocks

#### Example 1: OAuth Flow (lines ~800-900)
**Current:**
```rust
if let Some(promise) = &self.add_platform_oauth_promise {
    if let Some(result) = promise.ready() {
        // Handle OAuth result
    }
}
```

**Replace with:**
```rust
if let Some(vm) = vm {
    // OAuth events already processed in apply_event()
    // Just check vm.recent_errors() for OAuth-related errors
}
```

#### Example 2: SSH Test (lines 715-728)
**Current:**
```rust
for (platform_name, promise) in &self.ssh_test_promises {
    if let Some(result) = promise.ready() {
        completed.push(platform_name.clone());
    }
}
```

**Replace with:**
```rust
// Call vm.test_connection() when button clicked
// Process SshEvent::ConnectionTested in event handler
```

### 3. Replace Direct calc:: Calls

Search for all `calc::` calls in platform.rs and replace:

**List VMs:**
```rust
// OLD
poll_promise::Promise::spawn_async(async {
    calc::gcp_rest::list_vms(&project_id).await
})

// NEW
vm.list_vms(platform_name.clone())?;
```

**Create VM:**
```rust
// OLD
poll_promise::Promise::spawn_async(async {
    calc::gcp_rest::create_vm(&project_id, &vm_name, &zone, &machine_type).await
})

// NEW
vm.create_vm(platform_name, vm_name, zone, machine_type)?;
```

**Delete VM:**
```rust
// OLD
poll_promise::Promise::spawn_async(async {
    calc::gcp_rest::delete_vm(&project_id, &vm_name, &zone).await
})

// NEW
vm.delete_vm(platform_name, vm_name, zone)?;
```

### 4. Add Event Processing

At the top of `ui()` method, uncomment and implement:

```rust
if let Some(vm) = vm {
    // Show active operations with progress bars
    for (op_name, progress) in vm.active_operations() {
        ui.horizontal(|ui| {
            ui.add(egui::ProgressBar::new(progress.progress)
                .text(format!("{}: {}", op_name, progress.status)));
        });
    }
    
    // Show recent errors
    if let Some(error) = vm.recent_errors().iter()
        .filter(|e| e.actor == "platform")
        .rev()
        .next() 
    {
        ui.colored_label(
            egui::Color32::RED,
            format!("⚠ {}: {}", error.operation, error.error)
        );
    }
}
```

### 5. Update GCP Wizard Integration

The GCP wizard (lines ~100) may need updates to use ViewModel for OAuth flow.

### 6. Testing Checklist

After migration, test:
- [ ] Add platform via OAuth
- [ ] List VMs
- [ ] Create VM (check progress display)
- [ ] Delete VM (check progress display)
- [ ] Restart VM
- [ ] Update firewall rules
- [ ] Fetch billing data
- [ ] Delete platform
- [ ] Error handling (invalid credentials, API errors)
- [ ] Progress display during long operations

### 7. Cleanup

After migration complete:
- Remove unused imports: `poll_promise`
- Remove SSH promise helper methods
- Update tests if any

## Migration Strategy

**Recommended approach: Incremental**

1. Start with one operation (e.g., List VMs)
2. Replace poll-promise with ViewModel for that operation
3. Test thoroughly
4. Move to next operation
5. Repeat until all operations migrated

**Don't:** Try to migrate everything at once - too risky

## Files to Modify

- `mobile/src/ui_tabs/platform.rs` - Main migration work
- `mobile/src/dure.rs` - Already updated to pass ViewModel ✅

## Estimated Effort

- **Lines to modify**: ~100-200 (out of 2693 total)
- **Time**: 2-4 hours with testing
- **Risk**: Medium (UI changes require manual testing)

## Notes

- Signature change is backward compatible (vm is Option)
- Existing functionality preserved during migration
- Can be done operation-by-operation
- Example pattern in comments at top of ui() method
