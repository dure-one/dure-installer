# MVVM Migration Guide

## Overview

This guide documents the pattern for migrating UI operations from synchronous blocking calls to async ViewModel-based event-driven architecture.

## Migration Pattern

### 1. Identify the Operation

Find operations that:
- Use `GcpRestClient::new()` or direct `calc::` calls
- Block the UI thread during execution
- Are triggered by user actions (button clicks)

### 2. Check Actor Implementation

Verify the corresponding command exists in:
- `mobile/src/viewmodel/platform/commands.rs` (or ssh/ns)
- Actor implementation in `mobile/src/viewmodel/platform/actor.rs`

### 3. Update Method Signature

**Before:**
```rust
fn operation(&mut self, params: String) {
    // Synchronous implementation
}
```

**After:**
```rust
fn operation(&mut self, params: String, vm: Option<&mut crate::viewmodel::ViewModel>) {
    // ViewModel-based implementation
}
```

### 4. Replace Implementation

**Before:**
```rust
fn update_firewall(&mut self, platform_name: String, project_id: String) {
    use crate::calc::gcp_rest::GcpRestClient;
    
    let current_ip = get_current_ip()?;
    let access_token = load_token()?;
    let client = GcpRestClient::new(access_token);
    
    match client.add_ip_to_firewall(&project_id, &current_ip) {
        Ok(()) => {
            self.loaded = false;
            self.load_error = None;
        }
        Err(e) => {
            self.load_error = Some(format!("Failed: {}", e));
        }
    }
}
```

**After:**
```rust
fn update_firewall(&mut self, platform_name: String, vm: Option<&mut crate::viewmodel::ViewModel>) {
    use crate::calc::gcp_rest::get_current_ip;
    
    if let Some(vm) = vm {
        let current_ip = match get_current_ip() {
            Ok(ip) => ip,
            Err(e) => {
                self.load_error = Some(format!("Failed to get IP: {}", e));
                return;
            }
        };
        
        if let Err(e) = vm.update_firewall(platform_name, current_ip) {
            self.load_error = Some(format!("Failed to start: {}", e));
        }
        // UI will be updated by event processing
    } else {
        self.load_error = Some("ViewModel not available".to_string());
    }
}
```

### 5. Update Call Sites

**Before:**
```rust
self.update_firewall(platform_name, project_id);
```

**After:**
```rust
self.update_firewall(platform_name, vm);
```

### 6. Add Event Processing

In the `ui()` method, add event handlers:

```rust
pub fn ui(&mut self, ui: &mut egui::Ui, vm: Option<&mut crate::viewmodel::ViewModel>) {
    if let Some(vm) = vm {
        let events = vm.poll_events(ui.ctx());
        for event in events {
            use crate::viewmodel::ViewModelEvent;
            use crate::viewmodel::platform::PlatformEvent;
            
            match event {
                ViewModelEvent::Platform(PlatformEvent::FirewallUpdated { whitelisted_ip, .. }) => {
                    eprintln!("✓ Successfully added {} to firewall", whitelisted_ip);
                    self.loaded = false;
                    self.load_error = None;
                }
                ViewModelEvent::Platform(PlatformEvent::Error { operation, error }) => {
                    if operation == "update_firewall" {
                        self.load_error = Some(format!("Failed: {}", error));
                    }
                }
                _ => {}
            }
        }
    }
    
    // Rest of UI rendering...
}
```

### 7. Update Dialog Signatures (if needed)

If the operation is called from a dialog:

```rust
// Before
fn render_dialog(&mut self, ctx: &egui::Context) {
    self.operation(params);
}

// After
fn render_dialog(&mut self, ctx: &egui::Context, vm: Option<&mut crate::viewmodel::ViewModel>) {
    self.operation(params, vm);
}
```

And update the call site:
```rust
// Before
self.render_dialog(ui.ctx());

// After
self.render_dialog(ui.ctx(), vm);
```

## Event Processing Patterns

### Success Event with Data
```rust
ViewModelEvent::Platform(PlatformEvent::BillingFetched { records, .. }) => {
    self.billing_data = Some(records);
    self.billing_loading = false;
}
```

### Success Event without Data
```rust
ViewModelEvent::Platform(PlatformEvent::FirewallUpdated { whitelisted_ip, .. }) => {
    eprintln!("✓ Firewall updated: {}", whitelisted_ip);
    self.loaded = false;
    self.load_error = None;
}
```

### Error Event
```rust
ViewModelEvent::Platform(PlatformEvent::Error { operation, error }) => {
    if operation == "update_firewall" {
        self.load_error = Some(format!("Failed to update firewall: {}", error));
    }
}
```

### Config Update Event
```rust
ViewModelEvent::Platform(PlatformEvent::VMDeleted { platform_name, vm_name }) => {
    if let Ok((mut app_config, config_path)) = load_config() {
        if let Some(platform) = app_config.platforms.iter_mut().find(|p| p.name == platform_name) {
            platform.vms.retain(|vm| vm.name != vm_name);
            app_config.save(&config_path)?;
        }
    }
    self.loaded = false;
}
```

## Common Patterns

### Early Validation
Keep synchronous validation for immediate feedback:
```rust
fn operation(&mut self, params: String, vm: Option<&mut ViewModel>) {
    // Synchronous validation
    let data = match get_current_ip() {
        Ok(ip) => ip,
        Err(e) => {
            self.error = Some(format!("Validation failed: {}", e));
            return;
        }
    };
    
    // Async operation
    if let Some(vm) = vm {
        vm.operation(params, data)?;
    }
}
```

### Loading States
Set loading state before command, clear in event:
```rust
// Before command
self.loading = true;
self.error = None;
vm.operation()?;

// In event processing
ViewModelEvent::Platform(Event::Success { .. }) => {
    self.loading = false;
}
```

### Fallback Handling
Always handle missing ViewModel:
```rust
if let Some(vm) = vm {
    vm.operation()?;
} else {
    self.error = Some("ViewModel not available".to_string());
}
```

## Benefits

1. **Non-blocking UI**: Operations run asynchronously without freezing the interface
2. **Progress reporting**: Built-in progress events from actors
3. **Error handling**: Centralized error processing through events
4. **Separation of concerns**: UI handles display, actors handle I/O
5. **Testability**: Actors can be tested independently
6. **Cross-platform**: Same ViewModel works for Desktop, CLI, WASM

## Migration Checklist

- [ ] Identify operation and verify actor command exists
- [ ] Update method signature to accept ViewModel
- [ ] Replace synchronous calc calls with vm.command()
- [ ] Update all call sites to pass vm
- [ ] Add event processing for success events
- [ ] Add event processing for error events
- [ ] Update any dialogs that call the method
- [ ] Test operation end-to-end
- [ ] Verify error handling works
- [ ] Check progress display (if applicable)

## Examples

See completed migrations:
- `fetch_billing_data` (commit: 581c481)
- `update_firewall` (commit: 16d0f92)
- `restart_vm` (commit: 16d0f92)
- `execute_delete_vm` (commit: 6d04f14)

## Notes

- Audit logging should move to actor/calc layer (currently in UI for delete operations)
- Config updates happen in UI event processing (separation of concerns)
- Some operations require multiple parameters - check ViewModel method signature
- Not all GcpRestClient calls need migration (e.g., display helpers during render)
