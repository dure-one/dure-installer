# MVVM Migration Blockers

This document lists operations that cannot be easily migrated to the ViewModel pattern and explains why.

## Category 1: Interface Mismatches

### SSH: Add Host
**Current Implementation:**
- UI stores: `host: "username@hostname"`, `port: number`, `password` or `private_key_path`
- Single text field for user@host

**ViewModel Interface:**
```rust
fn add_ssh_host(&self, name: String, host: String, port: u16, user: String, ssh_key_path: String)
```

**Blocker:** ViewModel expects separate `name` (identifier), `host` (hostname), and `user` fields.

**Solutions:**
1. **UI Refactor (Recommended)**: Split UI into separate fields:
   - Name/Label field
   - Username field
   - Hostname field
   - Port field (already separate)
   - Authentication fields (already separate)

2. **ViewModel Redesign**: Change command to match current UI (less ideal)

3. **Temporary Mapping**: Parse "user@host" in UI before calling ViewModel

### NS: Delete Record
**Current Implementation:**
- UI uses: `name`, `record_type`, `value` to identify record
- Searches for matching record in config

**ViewModel Interface:**
```rust
DeleteRecord {
    provider_name: String,
    domain: String,
    record_id: String,  // ← Expects ID, not name/type/value
}
```

**Blocker:** ViewModel expects `record_id` but UI identifies records by name/type/value.

**Solutions:**
1. **Add ID to records**: Modify NsConfig to include record IDs
2. **Change command**: Accept name/type/value instead of record_id
3. **Map in UI**: Look up record_id from name/type/value before calling

## Category 2: Missing Actor Implementation

### Platform: OAuth Flow
**Current Implementation:**
- Uses `poll_promise::Promise` for OAuth
- Complex multi-step flow:
  1. Start OAuth → get auth URL
  2. User completes in browser
  3. Poll for completion
  4. Fetch user info
  5. Fetch project list

**ViewModel Commands Needed:**
```rust
StartOAuth { platform_name: String }  // Not implemented in actor
CompleteOAuth { platform_name: String, auth_code: String }  // Not implemented
```

**Blocker:** PlatformActor doesn't implement OAuth commands.

**Solutions:**
1. **Implement OAuth commands in PlatformActor**
2. **Keep poll_promise for OAuth** (complex, browser interaction)
3. **Hybrid approach**: Use poll_promise for OAuth, ViewModel for everything else

### Platform: List Projects
**Current Implementation:**
- Synchronously calls `client.list_projects()` in multiple places
- Used during OAuth flow and platform setup

**ViewModel Status:**
- Command exists: `ListProjects { platform_name: String }`
- Actor method: **NOT IMPLEMENTED** (stub only)

**Blocker:** Actor's list_projects method not implemented.

**Solution:** Implement in actor:
```rust
async fn list_projects(&mut self, platform_name: String) -> anyhow::Result<()> {
    self.send_progress("list_projects", 0.5, "Fetching projects...").await;
    
    let platform = runtime::unblock({
        let platform_name = platform_name.clone();
        move || crate::calc::db::load_platform(&platform_name)
    }).await?;
    
    let projects = runtime::unblock({
        let token = platform.access_token.clone();
        move || crate::calc::gcp_rest::list_projects(&token)
    }).await?;
    
    self.send_event(PlatformEvent::ProjectsListed {
        platform_name,
        projects: /* convert to (id, name) tuples */
    }).await;
    
    Ok(())
}
```

## Category 3: Render-Time Operations

### Platform: Helper Functions
These functions are called during UI rendering, not from user actions:

**compute_firewall_status()**
- Line 248
- Synchronously checks if current IP is whitelisted
- Called while building table rows

**fetch_project_count()**
- Line 599
- Synchronously fetches project count
- Called while building table rows

**Blocker:** These run during `load_rows()` which is synchronous and called during rendering.

**Why It's Hard:**
- Rendering must be synchronous in egui
- Can't await async operations during render
- Would need to:
  1. Trigger async fetch on first render
  2. Show "Loading..." during fetch
  3. Update UI when event arrives
  4. This breaks current table building approach

**Solutions:**
1. **Cache and refresh**: Fetch data asynchronously on Refresh button, cache results
2. **Remove live fetching**: Only show data from config, not live GCP
3. **Background polling**: Periodically refresh in background (complex)

### Platform: GCP Wizard
**Current Implementation:**
- Complex multi-step dialog (`ui_dlg/platform_gcp.rs`, 78KB)
- Manages VM creation flow with many sub-steps
- Tightly integrated with egui dialog state

**Blocker:** Massive refactor required, complex state machine.

**Solution:** Keep as-is initially, migrate later if needed.

## Category 4: Configuration Management

### Where Config is Updated
Many operations update config files directly:
- SSH add/delete host
- Platform add/delete
- NS provider/domain/record management

**Current Pattern:**
```rust
fn operation(&mut self) {
    let (mut config, path) = load_config()?;
    config.modify();
    config.save(&path)?;
    self.loaded = false;
}
```

**ViewModel Pattern:**
```rust
// UI triggers
vm.operation(params)?;

// Actor executes (doesn't touch config)
async fn operation() {
    api_call().await?;
    send_event(Success).await;
}

// UI handles event (updates config)
match event {
    Success => {
        let (mut config, path) = load_config()?;
        config.modify();
        config.save(&path)?;
    }
}
```

**Decision:** Config management stays in UI (separation of concerns).

## Summary of Blockers

| Category | Operations | Effort to Unblock |
|----------|-----------|-------------------|
| Interface Mismatch | SSH Add Host, NS Delete Record | Medium (UI refactor) |
| Missing Actor | OAuth, List Projects | Medium (implement in actor) |
| Render-Time | Firewall status, Project count | High (architecture change) |
| Complex Flows | GCP Wizard, OAuth flow | Very High (major refactor) |

## Recommended Approach

1. **Quick Wins (Already Done)**:
   - ✅ Platform: Billing, Firewall Update, VM Restart, VM Delete
   - ✅ SSH: Host Delete

2. **Medium Effort (Next Priority)**:
   - Implement missing actor commands (List Projects)
   - Refactor SSH Add Host UI to match ViewModel interface
   - Fix NS Delete Record interface mismatch

3. **Long Term (Future Work)**:
   - OAuth flow migration (complex)
   - GCP Wizard refactor (very complex)
   - Render-time operations (architectural change)

## Migration Statistics

**Total Operations Identified:** ~30
**Migrated:** 7 (23%)
**Blocked by Interface:** 2
**Blocked by Missing Actor:** 2
**Blocked by Architecture:** 4-5
**Remaining Straightforward:** 14-19

**Realistic Next Target:** 10-12 operations (40% total coverage)
