# KeePass Database Handle Architecture

**Date:** 2026-09-22  
**Status:** Approved  
**Author:** Claude Sonnet 4.5

## Summary

Refactor keyring management from password-passing to handle-sharing. Open KeePass database once at profile login, share `Arc<RwLock<Database>>` handle across threads, auto-save on mutations via RAII guard.

**Benefits:**
- More secure: password not copied/stored
- More efficient: decrypt once, not per operation
- Cleaner: single source of truth
- Thread-safe: Arc<RwLock> for background threads

## Problem

Current architecture (password-passing):
```
Login → password stored in DureApp.current_profile_password
Every operation → open(password) → use → close
Password copied to: UI → ViewModel → Actor → Business logic
Security risk: password in memory at multiple locations
Performance: decrypt database on every operation
```

Issues:
1. Password stored in `DureApp`, `PlatformCommand`, closures
2. Database decrypted repeatedly (SSH, VM creation, keyring ops)
3. Threading requires password clones for `move` closures
4. Easy to forget password parameter in call chains

## Solution

New architecture (handle-sharing):
```
Login → password → open once → Arc<DatabaseHandle> 
Operations → clone Arc → read/write → auto-save
Logout → drop handle → password gone
```

Changes:
1. Database opened once at login, closed at logout
2. Handle shared via cheap `Arc::clone()`
3. Auto-save on mutations via Drop guard
4. Password never stored, exists only during open

## Design

### 1. Core Data Structures

```rust
// mobile/src/calc/keyring.rs

/// Thread-safe handle to opened KeePass database
pub struct DatabaseHandle {
    db: Arc<RwLock<Database>>,
    kdbx_path: PathBuf,
    kpkey_path: PathBuf,
    key: DatabaseKey,  // Cached for save
}

impl DatabaseHandle {
    /// Open database at login
    pub fn open(
        kdbx_path: PathBuf,
        kpkey_path: PathBuf,
        password: Option<&str>,
    ) -> Result<Self> {
        // Build DatabaseKey from password + keyfile
        let mut key = DatabaseKey::new();
        if let Some(pwd) = password {
            if !pwd.is_empty() {
                key = key.with_password(pwd);
            }
        }
        let kpkey_data = std::fs::read(&kpkey_path)?;
        let mut cursor = std::io::Cursor::new(kpkey_data);
        key = key.with_keyfile(&mut cursor)?;
        
        // Open database
        let mut file = File::open(&kdbx_path)?;
        let db = Database::open(&mut file, key.clone())?;
        
        Ok(Self {
            db: Arc::new(RwLock::new(db)),
            kdbx_path,
            kpkey_path,
            key,
        })
    }
    
    /// Read-only access
    pub fn read(&self) -> RwLockReadGuard<Database> {
        self.db.read().unwrap()  // Poisoning = panic
    }
    
    /// Mutable access with auto-save on drop
    pub fn write_and_save(&self) -> SaveGuard {
        SaveGuard {
            guard: self.db.write().unwrap(),
            handle: self,
        }
    }
    
    /// Internal save (called by Drop)
    fn save_internal(&self) -> Result<()> {
        let db = self.db.read().unwrap();
        let mut file = File::create(&self.kdbx_path)?;
        db.save(&mut file, self.key.clone())?;
        file.sync_all()?;
        Ok(())
    }
}

/// RAII guard: auto-saves database when dropped
pub struct SaveGuard<'a> {
    guard: RwLockWriteGuard<'a, Database>,
    handle: &'a DatabaseHandle,
}

impl<'a> Deref for SaveGuard<'a> {
    type Target = Database;
    fn deref(&self) -> &Self::Target { &self.guard }
}

impl<'a> DerefMut for SaveGuard<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.guard }
}

impl<'a> Drop for SaveGuard<'a> {
    fn drop(&mut self) {
        // Write guard unlocks first
        drop(&mut self.guard);
        
        // Then save to disk
        if let Err(e) = self.handle.save_internal() {
            dure_error!("CRITICAL: Failed to save keyring: {}", e);
            dure_error!("  Path: {:?}", self.handle.kdbx_path);
        }
    }
}
```

**Design decisions:**
- `DatabaseKey` cached: avoid re-reading keyfile on every save
- `RwLock::unwrap()`: lock poisoning means panic (acceptable - app already broken)
- `Drop` saves after unlock: database unlocked during I/O
- Save errors logged, not panicked: Drop must not panic

### 2. Database Lifecycle

```rust
// mobile/src/dure.rs

pub struct DureApp {
    // REMOVE:
    // current_profile_password: Option<String>,
    
    // ADD:
    #[cfg_attr(feature = "serde", serde(skip))]
    pub current_profile_kdbx: Option<Arc<DatabaseHandle>>,
}

impl DureApp {
    /// Login flow (around line 964)
    fn handle_profile_login(&mut self, password: String) {
        // ... existing OAuth/profile loading ...
        
        // Open database immediately after profile loads
        let kdbx_path = ctx.kdbx_path.clone();
        let kpkey_path = ctx.kpkey_path.clone();
        
        match DatabaseHandle::open(kdbx_path, kpkey_path, Some(&password)) {
            Ok(handle) => {
                self.current_profile_kdbx = Some(Arc::new(handle));
                self.current_profile = Some(ctx);
                // Password dropped here - not stored
            }
            Err(e) => {
                self.dlg_profile_login.error = Some(format!("Failed to open keyring: {}", e));
                return;  // Login fails if DB can't open
            }
        }
    }
    
    /// Logout flow
    fn handle_profile_logout(&mut self) {
        // Drop database handle (triggers final save if refcount = 1)
        self.current_profile_kdbx = None;
        self.current_profile = None;
    }
}
```

**Lifecycle stages:**
1. **Login**: `DatabaseHandle::open()` → stored in `Arc` → fail fast if can't open
2. **Active**: Handle shared via `Arc::clone()`, operations use `read()` / `write_and_save()`
3. **Logout**: Drop handle → final save (if last reference)
4. **Crash**: Auto-saved after each mutation (no data loss)

**Security:**
- Password exists only during `DatabaseHandle::open()` call
- Not stored in `DureApp` or anywhere else
- Database stays decrypted in memory (acceptable tradeoff for usability)

### 3. API Changes

```rust
// mobile/src/calc/keyring.rs

// OLD API (mark deprecated, remove after migration):
#[deprecated(note = "Use list_keys_from_handle instead")]
pub fn list_keys(kdbx_path: &Path, kpkey_path: Option<&Path>, password: Option<&str>) -> Result<Vec<KeyEntry>>

// NEW API:
pub fn list_keys_from_handle(handle: &DatabaseHandle) -> Result<Vec<KeyEntry>> {
    let db = handle.read();
    let mut keys = Vec::new();
    collect_keys_from_group(&db.root, &mut keys)?;
    Ok(keys)
}

pub fn add_key_to_handle(
    handle: &DatabaseHandle,
    domain: &str,
    username: &str,
    password: &str,
    ssh_key: Option<&[u8]>,
    notes: Option<&str>,
) -> Result<()> {
    let mut db = handle.write_and_save();  // Auto-saves on drop
    
    let group = find_or_create_group(&mut db.root, KEEPASS_GROUP_NAME)?;
    
    let mut entry = Entry::default();
    entry.fields.insert("Title".to_string(), Value::Unprotected(domain.to_string()));
    entry.fields.insert("UserName".to_string(), Value::Unprotected(username.to_string()));
    entry.fields.insert("Password".to_string(), Value::Protected(password.as_bytes().into()));
    
    if let Some(ssh) = ssh_key {
        entry.binaries.insert("ssh_key".to_string(), ssh.to_vec());
    }
    if let Some(n) = notes {
        entry.fields.insert("Notes".to_string(), Value::Unprotected(n.to_string()));
    }
    
    group.entries.push(entry);
    Ok(())
    // db saved automatically when guard drops
}

// Similarly: update_key_to_handle, delete_key_from_handle
```

**Migration strategy:**
1. Add new `_from_handle` / `_to_handle` functions
2. Mark old API `#[deprecated]`
3. Update call sites incrementally
4. Remove old API after all sites migrated

**Call site transformation:**
```rust
// OLD:
keyring::list_keys(&kdbx_path, Some(&kpkey_path), password.as_deref())?;

// NEW:
keyring::list_keys_from_handle(&db_handle)?;
```

### 4. Threading & Handle Propagation

**Thread safety:**
- `DatabaseHandle` is `Send + Sync` via `Arc<RwLock<Database>>`
- Safe to clone and send to background threads
- `Arc::clone()` is cheap (atomic refcount increment)

**Example: GCP wizard background thread**
```rust
// mobile/src/ui_dlg/platform_gcp.rs

Promise::spawn_thread("gcp_create_vm", move || {
    let db_handle = db_handle.clone();  // Clone Arc before move
    
    // Generate SSH key...
    
    keyring::add_key_to_handle(
        &db_handle,
        &domain,
        "root",
        "",
        Some(ssh_private_key.as_bytes()),
        Some("GCP VM SSH private key"),
    )?;
    // Auto-saves when function returns
})
```

**Propagation through layers:**
```
DureApp.current_profile_kdbx: Arc<DatabaseHandle>
    ↓ clone
PlatformTab.ui(db_handle: &Option<Arc<DatabaseHandle>>)
    ↓ clone
GcpWizard.db_handle: Option<Arc<DatabaseHandle>>
    ↓ clone before move
Promise::spawn_thread(move || { ... })
    ↓ use
keyring::add_key_to_handle(&db_handle, ...)
```

**Threading contexts (from analysis):**
1. **Native platforms**: `Promise::spawn_thread`, `runtime::unblock` → background threads
2. **WASM**: Everything on main thread (no threading)
3. **Arc<RwLock>** works for both (just overhead on WASM)

### 5. Error Handling

**Error scenarios:**

1. **Open fails at login** → Login rejected, user sees error
   ```rust
   match DatabaseHandle::open(...) {
       Err(e) => {
           self.dlg_profile_login.error = Some(format!("Failed to open keyring: {}", e));
           return;  // Login fails
       }
   }
   ```

2. **Save fails during mutation** → Logged, user continues
   ```rust
   impl Drop for SaveGuard {
       fn drop(&mut self) {
           if let Err(e) = self.handle.save_internal() {
               dure_error!("CRITICAL: Failed to save keyring: {}", e);
               // Can't return error from Drop
           }
       }
   }
   ```

3. **Corrupted database** → Detected at open, login fails
   ```rust
   // Database::open() validates format
   // Returns Err if corrupted
   // User must restore from backup manually
   ```

**Backup strategy:**
- No automatic backup on save failure (adds complexity)
- Rely on auto-save frequency (risk window <1 second)
- Manual backup command out of scope (can add later)

### 6. Migration Path

**Phase 1: Add new API alongside old**
```rust
// Keep old functions, mark deprecated
#[deprecated(note = "Use list_keys_from_handle instead")]
pub fn list_keys(...) -> Result<Vec<KeyEntry>> { ... }

// Add new handle-based functions
pub fn list_keys_from_handle(handle: &DatabaseHandle) -> Result<Vec<KeyEntry>> { ... }
```

**Phase 2: Update call sites**

Migration order:
1. **DureApp** (dure.rs) - Add `current_profile_kdbx`, open at login
2. **UI layer** (ui_tabs/platform.rs, ui_dlg/platform_gcp.rs) - Pass handle
3. **ViewModel** (viewmodel/mod.rs, viewmodel/platform/actor.rs) - Accept handle in commands
4. **Business logic** (calc/hosting_gcp.rs, calc/ssh.rs) - Use new keyring API
5. **CLI commands** (cli/commands/*) - Migrate or keep old API for headless

**Phase 3: Remove old API**
- Delete deprecated functions after all sites migrated
- Remove password fields from structs/commands

**Backward compatibility:**
```rust
// CLI mode (headless) - handle not available at startup
#[cfg(not(feature = "gui"))]
pub fn list_keys(kdbx_path: &Path, kpkey_path: Option<&Path>, password: Option<&str>) -> Result<Vec<KeyEntry>> {
    // CLI uses password-based approach
}

#[cfg(feature = "gui")]
pub fn list_keys(...) {
    compile_error!("Use list_keys_from_handle in GUI mode");
}
```

**Rollback plan:**
- Git branch for migration
- Each phase = separate commit
- Can revert individual commits
- Keep old API until verified working

### 7. Testing Strategy

**Unit tests:**
```rust
// mobile/src/calc/keyring.rs

#[cfg(test)]
mod tests {
    #[test]
    fn test_handle_lifecycle() {
        // Create database, open handle, verify can read
    }
    
    #[test]
    fn test_auto_save() {
        // Mutate via write_and_save(), reopen, verify persisted
    }
    
    #[test]
    fn test_concurrent_reads() {
        // Spawn 10 threads, all read simultaneously
    }
    
    #[test]
    fn test_sequential_writes() {
        // 5 sequential mutations, verify no deadlock
    }
}
```

**Integration tests:**
```rust
// mobile/tests/keyring_integration.rs

#[test]
fn test_add_key_workflow() {
    // Setup → Add key → List keys → Verify persisted
}
```

**Manual testing checklist:**
- [ ] Login with password → DB opens
- [ ] Add VM → SSH key saved → VM created
- [ ] SSH connect → Key loaded from keyring
- [ ] Logout → DB closed (no errors)
- [ ] Login again → Keys still present
- [ ] Crash test: Add key, kill -9, restart → Key present
- [ ] Concurrent: Add VM + SSH connect simultaneously

## Files Changed

**Core:**
- `mobile/src/calc/keyring.rs` - Add DatabaseHandle, SaveGuard, new API
- `mobile/src/dure.rs` - Add current_profile_kdbx, remove current_profile_password

**UI Layer:**
- `mobile/src/ui_tabs/platform.rs` - Pass handle instead of password
- `mobile/src/ui_dlg/platform_gcp.rs` - Store handle, use new keyring API

**ViewModel:**
- `mobile/src/viewmodel/mod.rs` - Commands accept handle
- `mobile/src/viewmodel/platform/actor.rs` - Pass handle to business logic
- `mobile/src/viewmodel/platform/commands.rs` - Remove password fields

**Business Logic:**
- `mobile/src/calc/hosting_gcp.rs` - Accept handle parameter
- `mobile/src/calc/ssh.rs` - Use new keyring API

**CLI:**
- `mobile/src/cli/commands/keyring.rs` - Use new API or keep old for headless
- `mobile/src/cli/commands/platform/runner.rs` - Remove password passing

## Security Considerations

**Improved:**
- Password not stored in memory after login
- Password not copied across call chains
- Password lifetime minimized to open() call only

**Unchanged:**
- Database decrypted in memory while logged in
- No encryption at rest while running
- No password manager integration

**Trade-offs:**
- Convenience vs security: keeping DB open in memory
- Acceptable for desktop app, user chooses to stay logged in

## Performance Impact

**Improvements:**
- Database decrypted once (not per operation)
- No repeated keyfile reads
- `Arc::clone()` cheaper than password clone

**Overhead:**
- `RwLock` sync overhead (minimal)
- Auto-save on every mutation (acceptable - infrequent operations)

**Benchmarks (expected):**
- Current: ~50ms per keyring operation (decrypt + use + encrypt)
- New: ~0.1ms per read, ~10ms per write (just save, no decrypt)

## Risks & Mitigations

**Risk 1: Save failure in Drop**
- Drop can't return errors
- **Mitigation**: Log prominently, user sees next operation fail

**Risk 2: Lock contention on concurrent writes**
- Multiple threads try write_and_save() simultaneously
- **Mitigation**: RwLock handles this, second writer blocks until first finishes

**Risk 3: Database corruption on crash during save**
- Partial write to disk
- **Mitigation**: KeePass format has checksums, open will fail (user restores backup)

**Risk 4: Handle leaked (Arc never dropped)**
- Database never saved on logout
- **Mitigation**: Each mutation auto-saves, logout just final sync

## Future Enhancements

1. **Explicit save control** - Add `handle.save()` for batch operations
2. **Dirty tracking** - Skip save if database unchanged
3. **Backup on save** - Keep `.kdbx.backup` before overwrite
4. **Password change** - Rekey database without logout
5. **Multi-profile** - Multiple handles open simultaneously

## References

- Current implementation: `mobile/src/calc/keyring.rs` (password-based)
- Threading analysis: `mobile/src/viewmodel/runtime.rs` (smol::unblock)
- KeePass library: `keepass` crate documentation
