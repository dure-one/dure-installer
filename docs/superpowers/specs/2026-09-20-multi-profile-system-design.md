# Multi-Profile System Design

**Date:** 2026-09-20  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved

## Overview

This specification defines a multi-profile system for the dure-installer application, enabling users to manage multiple independent configurations, databases, and credentials. Each profile is isolated in its own directory with dedicated config, SSH keys, KeePass database, and SQLite database.

## Goals

1. **Profile Isolation**: Each profile has independent config, database, and credentials
2. **Clean UX**: Profile selector in main UI, blocking login on startup, smooth switching
3. **Data Safety**: Password-protected KeePass databases, no automatic migration
4. **MVVM Integration**: Leverage existing reactive ViewModel architecture
5. **No Breaking Changes**: Old single-profile directory (`~/.config/dure-installer/`) remains untouched

## Requirements Summary

### Functional Requirements

1. **Multi-profile directories**: `~/.config/dure_installer/{profile_name}/`
2. **Per-profile files**: config.yml, id_ed25519, id_ed25519.pub, key.kdbx, dure.db
3. **Startup behavior**: Blank profile selector → blocking "Select or create profile" state
4. **Profile creation**: User provides name + password → auto-generate keys + kdbx
5. **Profile selection**: Show login dialog → verify kdbx password → load all tabs
6. **Profile switching**: Re-authenticate → reload all tab data
7. **Profile deletion**: Confirmation dialog → delete profile directory
8. **No migration**: Start fresh, old `~/.config/dure-installer/` untouched

### Non-Functional Requirements

1. **Security**: All profile operations require password authentication
2. **Performance**: Profile switching < 1 second for typical datasets
3. **Reliability**: Graceful degradation if config corrupted (use defaults)
4. **Testability**: Pure business logic in ProfileManager, UI in separate layer

## Design Decisions

### Decision 1: Centralized ProfileManager + ViewModel Integration

**Chosen Approach**: Approach 1 - Centralized ProfileManager with ViewModel integration

**Rationale**:
- Aligns with existing MVVM architecture
- Reactive state propagation via RefState
- Clean separation: ProfileManager (business logic) + ViewModel (state coordination)
- Testable without egui dependencies

**Alternatives Considered**:
- App-level profile state (simpler but tight coupling, manual reloads)
- Global state (anti-pattern, requires app restart)

### Decision 2: No Automatic Migration

**Chosen Approach**: Leave old `~/.config/dure-installer/` untouched

**Rationale**:
- Cleanest implementation
- No risk of migration bugs destroying user data
- Users can manually recreate profiles if needed

**Alternatives Considered**:
- Auto-migrate to `default` profile (could surprise users)
- Prompt for migration (adds complexity)

### Decision 3: Blocking Startup Dialog

**Chosen Approach**: Show "No profile selected" state until user selects/creates profile

**Rationale**:
- Explicit profile selection required (no ambiguity)
- Prevents operations on non-existent profile
- Clear UX: can't proceed without profile

**Alternatives Considered**:
- Show tabs with empty state (confusing, operations fail silently)
- Auto-select first profile (no explicit user consent)

### Decision 4: Profile Name Validation

**Rules**:
- Allowed characters: `a-z`, `A-Z`, `0-9`, `-`, `_`
- Length: 1-64 characters
- Case-sensitive: "Work" ≠ "work"

**Rationale**: Safe for all filesystems, no escaping needed

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────┐
│                  DureApp (UI)                   │
│  ┌──────────────┐  ┌────────────────────────┐  │
│  │ Profile      │  │ Dialogs:               │  │
│  │ Selector     │  │ - Login                │  │
│  │ ComboBox     │  │ - Create Profile       │  │
│  │              │  │ - Delete Confirmation  │  │
│  └──────┬───────┘  └────────┬───────────────┘  │
│         │                   │                   │
└─────────┼───────────────────┼───────────────────┘
          │                   │
          ▼                   ▼
┌─────────────────────────────────────────────────┐
│              ViewModel (State)                  │
│  ┌──────────────────────────────────────────┐  │
│  │ current_profile: RefState<Option<        │  │
│  │                  ProfileContext>>         │  │
│  ├──────────────────────────────────────────┤  │
│  │ platform_tab: RefState<...>              │  │
│  │ ssh_tab: RefState<...>                   │  │
│  │ ns_tab: RefState<...>                    │  │
│  │ ...                                      │  │
│  └──────────────────────────────────────────┘  │
│                                                 │
│  Methods:                                       │
│  - new_empty() -> Self                          │
│  - load_profile(ProfileContext) -> Result<()>   │
│  - unload_profile()                             │
└─────────────────────┬───────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────┐
│         ProfileManager (Business Logic)         │
│                                                 │
│  Static Methods:                                │
│  - list_profiles() -> Result<Vec<String>>       │
│  - create_profile(name, pwd) -> Result<...>     │
│  - load_profile(name) -> Result<ProfileContext> │
│  - verify_password(profile, pwd) -> Result<()>  │
│  - delete_profile(name) -> Result<()>           │
│  - validate_profile(name) -> Result<bool>       │
└─────────────────────┬───────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────┐
│              Filesystem Layout                  │
│                                                 │
│  ~/.config/dure_installer/                      │
│    ├── personal/                                │
│    │   ├── config.yml                           │
│    │   ├── id_ed25519                           │
│    │   ├── id_ed25519.pub                       │
│    │   ├── key.kdbx                             │
│    │   └── dure.db                              │
│    ├── work/                                    │
│    │   └── ...                                  │
│    └── test-shop/                               │
│        └── ...                                  │
└─────────────────────────────────────────────────┘
```

### Data Model

#### ProfileContext

```rust
/// Profile metadata and paths
#[derive(Debug, Clone)]
pub struct ProfileContext {
    pub name: String,
    pub config_dir: PathBuf,       // ~/.config/dure_installer/{name}/
    pub config_file: PathBuf,      // .../config.yml
    pub db_path: PathBuf,          // .../dure.db
    pub kdbx_path: PathBuf,        // .../key.kdbx
    pub kpkey_path: PathBuf,       // .../id_ed25519
    pub kppubkey_path: PathBuf,    // .../id_ed25519.pub
}
```

#### ProfileManager API

```rust
pub struct ProfileManager;

impl ProfileManager {
    /// List all available profiles (scans ~/.config/dure_installer/)
    pub fn list_profiles() -> Result<Vec<String>>;
    
    /// Create new profile with auto-generated keys
    pub fn create_profile(name: &str, kdbx_password: &str) -> Result<ProfileContext>;
    
    /// Load profile context (does NOT verify password)
    pub fn load_profile(name: &str) -> Result<ProfileContext>;
    
    /// Verify kdbx password for a profile
    pub fn verify_password(profile: &ProfileContext, password: &str) -> Result<()>;
    
    /// Delete profile (removes entire directory)
    pub fn delete_profile(name: &str) -> Result<()>;
    
    /// Check if profile directory exists and is valid
    pub fn validate_profile(name: &str) -> Result<bool>;
    
    /// Validate profile name format
    pub fn validate_name(name: &str) -> bool;
}
```

#### ViewModel Extensions

```rust
pub struct ViewModel {
    // NEW: Current active profile (None = no profile selected)
    pub current_profile: RefState<Option<ProfileContext>>,
    
    // Existing fields (unchanged)
    pub platform_tab: RefState<PlatformTabViewModel>,
    pub ssh_tab: RefState<SshTabViewModel>,
    pub ns_tab: RefState<NsTabViewModel>,
    // ... other tabs
}

impl ViewModel {
    /// Create ViewModel without a profile (for initial app state)
    pub fn new_empty() -> Self;
    
    /// Load ViewModel with a specific profile
    pub fn load_profile(&self, profile: ProfileContext) -> Result<()>;
    
    /// Unload current profile (returns to "no profile" state)
    pub fn unload_profile(&self);
}
```

### File Organization

**New Files**:
- `mobile/src/calc/profile.rs` - ProfileManager and ProfileContext
- `mobile/src/ui_dlg/profile_login.rs` - Login dialog
- `mobile/src/ui_dlg/profile_create.rs` - Create profile dialog
- `mobile/src/ui_dlg/profile_delete.rs` - Delete confirmation dialog

**Modified Files**:
- `mobile/src/lib.rs` - Update `get_app_config_dir()` to support profiles
- `mobile/src/viewmodel/mod.rs` - Add `current_profile` field, `load_profile()`, `unload_profile()`
- `mobile/src/dure.rs` - Add profile selector widget, dialog coordination
- `mobile/src/dure_stt.rs` - Add dialog state fields
- `mobile/src/ui_tabs/*.rs` - Check `current_profile.is_some()` before operations
- `mobile/src/calc/db.rs` - Export `set_db_path()` publicly

## Data Flow

### 1. Application Startup

```
1. App Launch (main.rs / main_android.rs)
   ↓
2. Initialize DureApp with ViewModel::new_empty()
   - current_profile = None
   - All tabs in empty state
   ↓
3. First frame render
   - Profile selector shows "None"
   - Tabs show "No profile selected" message
   ↓
4. User must select or create profile to proceed
```

### 2. Profile Selection

```
User selects existing profile "work"
   ↓
1. Set pending_profile_name = "work"
   Set show_login_dialog = true
   ↓
2. Login dialog appears (modal)
   User enters password
   ↓
3. Verify password:
   profile = ProfileManager::load_profile("work")?
   ProfileManager::verify_password(&profile, password)?
   ↓
4a. SUCCESS:
    viewmodel.load_profile(profile)?
    - Sets db path: calc::db::set_db_path()
    - Loads config: AppConfig::load()
    - Reloads all tabs via RefState
    - Updates current_profile
    Close login dialog
   ↓
4b. FAILURE:
    Show error in login dialog: "Incorrect password"
    Keep dialog open for retry
```

### 3. Profile Creation

```
User clicks "+ Create New Profile"
   ↓
1. Show create profile dialog
   User enters: name="personal", password="secret123"
   ↓
2. Validate inputs:
   - Name not empty, valid chars, <= 64 chars
   - Password not empty
   - Passwords match
   ↓
3. Create profile:
   profile = ProfileManager::create_profile("personal", "secret123")?
   - Creates ~/.config/dure_installer/personal/
   - Generates id_ed25519 + id_ed25519.pub
   - Creates key.kdbx with password
   - Creates empty config.yml
   ↓
4. Auto-login to new profile:
   viewmodel.load_profile(profile)?
   Close create dialog
```

### 4. Profile Switching

```
User has profile "work" loaded, selects "personal"
   ↓
1. Show login dialog for "personal"
   User enters password
   ↓
2. Verify password (same as Profile Selection flow)
   ↓
3. SUCCESS:
   viewmodel.load_profile(profile)?
   - Unloads "work" data from all tabs
   - Loads "personal" data into all tabs
   - All UI components re-render with new data
   ↓
   State after switch:
   - Database: now using ~/.config/dure_installer/personal/dure.db
   - Config: loaded from personal/config.yml
   - Keyring: using personal/key.kdbx
   - All tabs show personal profile data
```

### 5. Profile Deletion

```
User clicks "Delete Profile" for "work" (currently loaded)
   ↓
1. Show delete confirmation dialog
   "Delete profile 'work'? (lists what will be deleted)"
   ↓
2. User confirms
   ↓
3. Check if deleting current profile:
   if current_profile.name == "work":
       viewmodel.unload_profile()  // First unload
   ↓
4. Delete profile:
   ProfileManager::delete_profile("work")?
   - Removes ~/.config/dure_installer/work/ directory
   ↓
5. Return to "No profile selected" state
   Profile selector shows "None"
```

## Error Handling

### Error Types

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ProfileError {
    #[error("Profile '{0}' not found")]
    NotFound(String),
    
    #[error("Profile '{0}' already exists")]
    AlreadyExists(String),
    
    #[error("Invalid profile name: {0}")]
    InvalidName(String),
    
    #[error("Incorrect password")]
    IncorrectPassword,
    
    #[error("Profile directory corrupted: missing {0}")]
    CorruptedProfile(String),
    
    #[error("Failed to create profile: {0}")]
    CreationFailed(String),
    
    #[error("Failed to delete profile: {0}")]
    DeletionFailed(String),
    
    #[error(transparent)]
    Io(#[from] std::io::Error),
    
    #[error(transparent)]
    Keyring(#[from] crate::calc::keyring::KeyringError),
}
```

### Error Recovery Strategy

| Component | Error Scenario | User-Facing Action | Technical Recovery |
|-----------|---------------|-------------------|-------------------|
| **ProfileManager::list_profiles()** | Base dir doesn't exist | Create dir automatically | `fs::create_dir_all()` on first call |
| | Profile dir missing files | Skip profile in list | Log warning, continue with valid profiles |
| | Permission denied | Show error toast | Return empty list, log error |
| **ProfileManager::create_profile()** | Name already exists | Show error in dialog: "Profile 'X' exists" | Return `Err(AlreadyExists)` |
| | Invalid name chars | Show error: "Use only a-z, 0-9, -, _" | Validate before attempting creation |
| | Disk full | Show error: "Disk full" | Return `Err(Io)`, cleanup partial files |
| | Key generation fails | Show error: "Failed to generate keys" | Return error, no partial profile created |
| **ProfileManager::verify_password()** | Incorrect password | Show in dialog: "Incorrect password" | Allow retry, don't lock account |
| | KeePass file corrupted | Show error: "Profile corrupted" | Suggest profile deletion/recovery |
| **ViewModel::load_profile()** | Database migration fails | Show error dialog with details | Unload profile, return to "None" state |
| | Config file invalid YAML | Log warning, use defaults | Continue with `AppConfig::default()` |
| | Database locked | Show retry dialog | Wait + retry 3x, then fail |
| **ProfileManager::delete_profile()** | Profile in use | Unload first automatically | Call `viewmodel.unload_profile()` first |
| | Permission denied | Show error: "Cannot delete" | Return error, leave profile intact |

### Graceful Degradation

**Scenario: Config file missing/corrupted**

The system will continue with default configuration rather than failing:

```rust
let config = match crate::config::AppConfig::load(&profile.config_file) {
    Ok(cfg) => cfg,
    Err(e) => {
        dure_warn!("Failed to load config for profile '{}': {}", profile.name, e);
        dure_warn!("Using default configuration");
        crate::config::AppConfig::default()
    }
};
```

**Critical vs Non-Critical Operations**:
- **Critical (must succeed)**: Database path setting, profile directory creation, key generation
- **Non-Critical (can use defaults)**: Config file parsing, cache initialization

## UI Components

### Profile Selector Widget

**Location**: Top-left of main window in `mobile/src/dure.rs`

**Behavior**:
- Shows current profile name or "None"
- Dropdown lists all available profiles (from `ProfileManager::list_profiles()`)
- Special options: "+ Create New Profile", "🗑 Delete Profile..." (when profile selected)
- Selecting a profile triggers login dialog

### Login Dialog

**Location**: `mobile/src/ui_dlg/profile_login.rs`

**Fields**:
- Profile name (display only)
- Password input (masked)
- Error message area
- Buttons: Cancel, Login

**Behavior**:
- Modal dialog (blocks main UI)
- Enter key submits
- Wrong password shows error, allows retry
- Success → closes dialog, loads profile

### Create Profile Dialog

**Location**: `mobile/src/ui_dlg/profile_create.rs`

**Fields**:
- Profile name input
- Password input (masked)
- Confirm password input (masked)
- Error message area
- Buttons: Cancel, Create

**Validation**:
- Name: not empty, valid chars, <= 64 chars, not already exists
- Password: not empty
- Passwords match

**Behavior**:
- Modal dialog
- Success → auto-login to new profile
- Failure → show error, keep dialog open

### Delete Confirmation Dialog

**Location**: `mobile/src/ui_dlg/profile_delete.rs`

**Content**:
- Warning message
- List of what will be deleted (config, keys, database)
- Buttons: Cancel, Delete

**Behavior**:
- Modal dialog
- Delete → removes profile directory
- If deleting active profile → unload first

## Testing Strategy

### Unit Tests

**Location**: `mobile/src/calc/profile.rs` (inline tests)

**Coverage**:
- Profile creation (success, duplicate, invalid name)
- Password verification (correct, incorrect)
- Profile listing (empty, single, multiple)
- Profile deletion (exists, not exists)
- Profile validation (complete, missing files)
- Name validation (valid, invalid characters, length)

### Integration Tests

**Location**: `tests/profile_integration.rs`

**Scenarios**:
- Full workflow: create → verify → load → unload → delete
- Multi-profile: create 3 → switch between → verify isolation
- Error handling: corrupted profile, wrong password

### Manual Testing Checklist

- [ ] Fresh install: Start app → no profiles → create first profile → verify it works
- [ ] Multi-profile: Create 3 profiles → switch between them → verify data isolation
- [ ] Delete profile: Delete active profile → verify unload → verify directory removed
- [ ] Wrong password: Try to login with wrong password → verify error shown, can retry
- [ ] Corrupted profile: Manually delete key.kdbx → verify profile skipped in list
- [ ] Invalid names: Try creating profiles with spaces, slashes → verify rejection
- [ ] Duplicate names: Try creating profile with existing name → verify rejection
- [ ] Empty password: Try creating profile with empty password → verify allowed/rejected
- [ ] Database isolation: Add data in profile A → switch to profile B → verify not visible
- [ ] Config isolation: Change setting in profile A → switch to profile B → verify independent

## Implementation Plan

Implementation will follow the writing-plans skill workflow:

1. **Phase 1: ProfileManager Core**
   - Create `calc/profile.rs`
   - Implement `ProfileContext` struct
   - Implement `ProfileManager` static methods
   - Add unit tests

2. **Phase 2: ViewModel Integration**
   - Add `current_profile` field to ViewModel
   - Implement `load_profile()` and `unload_profile()`
   - Modify `new_empty()` constructor
   - Update all tab ViewModels with empty state constructors

3. **Phase 3: UI Dialogs**
   - Create `ui_dlg/profile_login.rs`
   - Create `ui_dlg/profile_create.rs`
   - Create `ui_dlg/profile_delete.rs`
   - Reference egui-material3 dialog examples

4. **Phase 4: Main UI Integration**
   - Add profile selector ComboBox to `dure.rs`
   - Wire up dialog state management in `dure_stt.rs`
   - Implement dialog show/hide logic
   - Add startup "no profile" blocking state

5. **Phase 5: Tab Updates**
   - Update all `ui_tabs/*.rs` to check `current_profile.is_some()`
   - Show "No profile selected" when None
   - Test each tab with profile switching

6. **Phase 6: Integration Testing**
   - Write integration tests
   - Manual testing across all workflows
   - Bug fixes and polish

## Open Questions

None - all design decisions have been validated with user.

## Success Criteria

1. ✅ User can create multiple profiles with unique names and passwords
2. ✅ User must authenticate before accessing profile data
3. ✅ Profile switching reloads all tab data correctly
4. ✅ Profile deletion removes all profile files
5. ✅ Old single-profile directory (`~/.config/dure-installer/`) remains untouched
6. ✅ All tabs show "No profile selected" when no profile loaded
7. ✅ Error messages are clear and actionable
8. ✅ 100% unit test coverage for ProfileManager
9. ✅ Integration tests cover all major workflows

## References

- **Existing Patterns**:
  - `mobile/src/calc/keyring.rs` - KeePass password verification pattern
  - `mobile/src/ui_tabs/client.rs` - KeyMgmt sub-tab uses keyring
  - `reference/egui-material3/examples/stories/dialog_window.rs` - Dialog UI pattern
  - `mobile/src/viewmodel/mod.rs` - MVVM with RefState reactive pattern

- **Related Documentation**:
  - `docs/PROJECT_SUMMARY.md` - Architecture overview
  - `CLAUDE.md` - Project context and build instructions
