# Android Platform Compatibility Design

**Date**: 2026-09-12  
**Branch**: `feature/mobile-desktop-compat`  
**Status**: Design Approved  
**Approach**: Minimal Platform Gates (Approach 1)

## Summary

Enable Android builds by removing incorrect platform gates from platform-independent modules (HTTP API clients, config data structures, business logic) while keeping gates only on truly platform-specific code (X11 desktop detection, CLI, installation tools, tray). This unblocks Android compilation without requiring immediate UI parity with desktop.

## Problem Statement

The current codebase incorrectly gates many platform-independent modules behind `#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]`, preventing Android builds from accessing functionality that would work on any platform. Specifically:

- **Config module** (`config.rs`) - Just data structures and YAML serialization, but gated for desktop-only
- **Business logic** (`calc/ns.rs`, `calc/docker.rs`, `calc/ansible.rs`, `calc/acme.rs`) - Network-based operations using HTTP APIs, incorrectly gated
- **API clients** (`api/gcp/*`, `api/ns_cloudflare.rs`, `api/ns_porkbun.rs`) - Pure HTTP clients, incorrectly gated
- **Android modules** (`android/*`) - Exist but not exported from `android/mod.rs`

Build errors show:
- "unresolved import `crate::config`" on Android (line 132 of `lib.rs`)
- "unresolved import `crate::calc::ns`" on Android
- "unresolved import `crate::api::gcp`" on Android

## Goals

### Immediate Goals (This Design)
1. ✅ Android builds compile successfully (`cargo ndk build`)
2. ✅ Platform-independent modules work on all platforms
3. ✅ Desktop-specific code remains properly gated
4. ✅ Android app launches without crashes

### Deferred Goals (Future Work)
- Responsive UI with 1400px breakpoint for mobile/desktop views
- Full UI parity for desktop-specific tabs (platform, SSH, NS wizards)
- Android-native UI components for advanced features

## Architecture

### Platform Gate Strategy

```
Platform-Independent (No Gates)
├── config.rs                    # Data structures, YAML serialization
├── api/
│   ├── gcp/*                   # HTTP APIs (OAuth, Compute, DNS, etc.)
│   ├── ns_cloudflare.rs        # HTTP API client
│   ├── ns_porkbun.rs           # HTTP API client
│   ├── ns_duckdns.rs           # HTTP API client
│   └── ns_gcp.rs               # Re-export of gcp/dns
├── calc/
│   ├── ns.rs                   # DNS business logic
│   ├── docker.rs               # Docker management logic
│   ├── ansible.rs              # Ansible management logic
│   ├── acme.rs                 # Certificate management logic
│   └── keyring.rs              # Keyring abstraction
└── storage/                    # Database models

Desktop-Only (Keep Gates)
├── api/desktop.rs              # X11 desktop environment detection
├── cli/                        # Command-line interface
├── install/                    # Desktop installation wizards
├── tray/                       # System tray integration
├── config_migration.rs         # Filesystem-based config migration
└── ui_tabs/
    ├── platform.rs             # Has GcpWizard (desktop dialog)
    ├── ssh.rs                  # Has desktop-specific wizards
    └── ns.rs                   # Has desktop-specific wizards

Android-Only (Properly Gated)
└── android/
    ├── activity.rs             # Android Activity lifecycle
    ├── clipboard.rs            # JNI clipboard integration
    ├── inputmethod.rs          # Soft keyboard handling
    └── screensize.rs           # Android screen metrics
```

### Current State vs Desired State

| Module | Current State | Desired State | Reason |
|--------|--------------|---------------|---------|
| `config` | Desktop-only gate | No gate | Just data structures |
| `calc/ns` | Desktop-only gate | No gate | HTTP API logic |
| `calc/docker` | Desktop-only gate | No gate | HTTP API logic |
| `calc/acme` | Desktop-only gate | No gate | HTTP API logic |
| `api/gcp/*` | Desktop-only gate | No gate | HTTP APIs |
| `api/ns_*` | Desktop-only gate | No gate | HTTP APIs |
| `api/desktop` | Desktop-only gate | Keep gate | X11 detection (desktop-specific) |
| `android/mod.rs` | Empty (no exports) | Export all submodules | Enable Android features |
| `ui_tabs/platform` | No gate | Desktop-only gate | Has GcpWizard dialog |
| `ui_tabs/ssh` | No gate | Desktop-only gate | Has desktop wizards |
| `ui_tabs/ns` | No gate | Desktop-only gate | Has desktop wizards |

## Detailed Changes

### 1. Core Library (`mobile/src/lib.rs`)

**Remove config gate:**
```rust
// BEFORE (lines 28-29)
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod config;

// AFTER
pub mod config;
```

**Keep config_migration gate** (uses filesystem operations):
```rust
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod config_migration;
```

**Fix Android Config initialization:**
```rust
// BEFORE (line 132)
let app_config = config::AppConfig::default();

// AFTER
let app_config = config::AppConfig::default_android();
```

**Update Config struct:**
```rust
impl Config {
    #[cfg(target_os = "android")]
    pub fn new() -> Result<Self> {
        let config_dir = PathBuf::from("/data/data/pe.nikescar.dure/files");
        let cache_dir = PathBuf::from("/data/data/pe.nikescar.dure/cache");
        let tmp_dir = cache_dir.join("tmp");
        let data_dir = cache_dir.join("data");

        // Create directories
        for dir in [&config_dir, &cache_dir, &tmp_dir, &data_dir] {
            fs::create_dir_all(dir)
                .context(format!("Failed to create directory: {:?}", dir))?;
        }

        let app_config = config::AppConfig::default_android();

        Ok(Config {
            config_dir,
            cache_dir,
            tmp_dir,
            data_dir,
            app_config,
        })
    }
}
```

### 2. Config Module (`mobile/src/config.rs`)

**Add Android defaults** (no filesystem config.yml access):
```rust
impl AppConfig {
    /// Android-specific defaults (no filesystem access for config.yml)
    #[cfg(target_os = "android")]
    pub fn default_android() -> Self {
        Self {
            platforms: vec![],
            ssh_hosts: vec![],
            cert: CertConfig::default(),
            cloudflare: CloudflareConfig::default(),
            porkbun: PorkbunConfig::default(),
            gcp_dns: GcpDnsConfig::default(),
            duckdns: DuckDnsConfig::default(),
        }
    }
    
    /// Desktop: Load from config.yml or use defaults
    #[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
    pub fn load_or_default(path: &Path) -> Self {
        // ... existing implementation ...
    }
}
```

### 3. Android Module (`mobile/src/android/mod.rs`)

**Export all Android submodules:**
```rust
// Currently EMPTY - add these exports:

#[cfg(target_os = "android")]
pub mod activity;

#[cfg(target_os = "android")]
pub mod clipboard;

#[cfg(target_os = "android")]
pub mod contexttheme;

#[cfg(target_os = "android")]
pub mod inputmethod;

#[cfg(target_os = "android")]
pub mod log;

#[cfg(target_os = "android")]
pub mod packagemanager;

#[cfg(target_os = "android")]
pub mod screensize;
```

### 4. Calc Modules (Remove Desktop-Only Gates)

**`mobile/src/calc/ns.rs`** - Remove gate from acme imports:
```rust
// BEFORE (lines 16-19)
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use crate::calc::acme::{
    DnsProvider, DnsProviderType, set_a_record, set_aaaa_record, set_txt_record,
};

// AFTER
use crate::calc::acme::{
    DnsProvider, DnsProviderType, set_a_record, set_aaaa_record, set_txt_record,
};
```

**`mobile/src/calc/docker.rs`** - Already platform-independent (uses `config`, `ssh`)  
**`mobile/src/calc/ansible.rs`** - Already platform-independent  
**`mobile/src/calc/acme.rs`** - Already platform-independent (HTTP APIs)

### 5. API Modules

**`mobile/src/api/mod.rs`** - Keep desktop.rs gated, remove gates from HTTP APIs:
```rust
// Desktop-only (X11 detection)
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod desktop;

// Platform-independent HTTP APIs (remove any existing gates)
pub mod ehttp_cache;
pub mod gcp;
pub mod ns_cloudflare;
pub mod ns_duckdns;
pub mod ns_gcp;
pub mod ns_porkbun;
```

**All `api/gcp/*` modules** - No gates needed (pure HTTP APIs)

### 6. UI Tabs (Conditionally Compile Desktop Wizards)

**`mobile/src/ui_tabs/mod.rs`** - Gate desktop-specific tabs:
```rust
// Android-compatible tabs (keep as-is)
pub mod client;
pub mod orders;
pub mod products;
pub mod channel;
pub mod dm;
pub mod members;
pub mod email;
pub mod roles;
pub mod site;

// Desktop-only tabs (have GcpWizard and desktop-specific dialogs)
#[cfg(not(target_os = "android"))]
pub mod platform;

#[cfg(not(target_os = "android"))]
pub mod ssh;

#[cfg(not(target_os = "android"))]
pub mod ns;
```

**Future work**: Implement Android-appropriate UI for these features with 1400px responsive breakpoint.

## Data Flow

### Desktop Flow (Unchanged)
```
User Input (CLI/GUI)
    ↓
Desktop-specific UI/CLI
    ↓
Business Logic (calc/*)
    ↓
HTTP API Clients (api/*)
    ↓
External Services (GCP, Cloudflare, etc.)
```

### Android Flow (New)
```
User Input (GUI only)
    ↓
Android-compatible UI tabs
    ↓
Business Logic (calc/*) ← NOW ACCESSIBLE
    ↓
HTTP API Clients (api/*) ← NOW ACCESSIBLE
    ↓
External Services (GCP, Cloudflare, etc.)
```

### Blocked on Android (Deferred)
```
Desktop-specific UI tabs (platform, ssh, ns with wizards)
Desktop environment detection (api/desktop)
CLI interface (headless not needed on Android)
System tray (not applicable to Android)
```

## Error Handling

### Platform-Specific Error Messages

When desktop-only features are accessed on Android (defensive programming):

```rust
// Example: If desktop API is somehow called on Android
#[cfg(target_os = "android")]
pub fn get_desktop_environment() -> Result<String> {
    Err(anyhow::anyhow!(
        "Desktop environment detection not available on Android"
    ))
}
```

### Config Loading Errors

```rust
impl Config {
    #[cfg(target_os = "android")]
    pub fn new() -> Result<Self> {
        // Fail fast if directory creation fails
        for dir in [&config_dir, &cache_dir, &tmp_dir, &data_dir] {
            fs::create_dir_all(dir)
                .with_context(|| format!(
                    "Failed to create Android directory: {:?}", dir
                ))?;
        }
        
        // Log successful initialization
        dure_info!("Android config initialized: {:?}", config_dir);
        
        Ok(Config { /* ... */ })
    }
}
```

### Build Errors

If compilation fails due to missing modules:
- **Symptom**: "unresolved import `crate::module_name`"
- **Solution**: Check cfg gates, ensure module is exported in parent `mod.rs`
- **Rollback**: Temporarily gate the problematic module, fix incrementally

## Testing Strategy

### Phase 1: Rust Library Build
```bash
cd mobile
cargo ndk -t arm64-v8a build --release --lib --features gui
```

**Success Criteria:**
- ✅ No "unresolved import" errors
- ✅ All platform-independent modules compile for Android
- ✅ Desktop-only modules excluded from build

### Phase 2: Full Android Build
```bash
cd mobile
./build.sh
```

**Success Criteria:**
- ✅ Rust compilation succeeds
- ✅ Gradle assembles APK successfully
- ✅ `app-debug.apk` or `app-release.apk` created

### Phase 3: Runtime Validation
```bash
# Install APK
adb install app/build/outputs/apk/debug/app-debug.apk

# Launch app
adb shell am start -n pe.nikescar.dure/.MainActivity

# Monitor logs
adb logcat -s Dure:V
```

**Success Criteria:**
- ✅ App launches without crashes
- ✅ Database initializes (Android paths)
- ✅ i18n loads successfully
- ✅ Available UI tabs render (client, orders, products)
- ✅ No JNI symbol resolution errors

### Validation Checklist

**Compile-time:**
- [ ] `config` module imports successfully on Android
- [ ] `calc/ns`, `calc/docker`, `calc/ansible` compile for Android
- [ ] All `api/gcp/*` modules compile for Android
- [ ] `api/ns_cloudflare`, `api/ns_porkbun` compile for Android
- [ ] `android/*` modules exported and accessible
- [ ] Desktop modules excluded: `api/desktop`, `cli`, `install`, `tray`
- [ ] Desktop UI tabs excluded: `platform`, `ssh`, `ns`

**Runtime:**
- [ ] App launches without crashes
- [ ] Database path set correctly (`/data/data/pe.nikescar.dure/files/dure.db`)
- [ ] Config initialized with Android defaults
- [ ] Material3 theme loads
- [ ] Korean fonts render correctly
- [ ] Available tabs accessible (client, orders, products, etc.)

## Rollback Plan

If Android build fails after changes:

1. **Identify failure module** - Check compiler error for which module failed
2. **Isolate the change** - Use `git diff` to see what changed
3. **Temporary gate** - Re-add desktop-only gate to problematic module
4. **Incremental fix** - Fix one module at a time, test each change
5. **Git bisect** - If needed, use `git bisect` to find exact breaking commit

## Future Work

### Phase 2: Responsive UI (1400px Breakpoint)
- Implement responsive layout system
- Desktop view: 1400px+ (multi-column, wizards, advanced controls)
- Mobile view: <1400px (single-column, simplified UI)
- Adapt desktop wizards (GcpWizard, etc.) for mobile

### Phase 3: Android-Native UI Components
- Replace desktop wizards with Android-appropriate UI
- Implement `ui_tabs/platform.rs` for Android (without GcpWizard)
- Implement `ui_tabs/ssh.rs` for Android
- Implement `ui_tabs/ns.rs` for Android

### Phase 4: Platform Abstraction (Optional)
- If adding more platforms (iOS, etc.), consider trait-based abstraction
- `PlatformServices` trait with Desktop/Android/iOS implementations
- Only if needed - don't over-engineer

## Files Modified

### Core
- `mobile/src/lib.rs` - Remove config gate, fix Android Config initialization, add android mod export
- `mobile/src/config.rs` - Add `default_android()` method

### Android
- `mobile/src/android/mod.rs` - Export all submodules

### Calc
- `mobile/src/calc/ns.rs` - Remove desktop-only gate from acme imports
- (No changes needed: `calc/docker.rs`, `calc/ansible.rs`, `calc/acme.rs` - already platform-independent)

### API
- `mobile/src/api/mod.rs` - Ensure desktop.rs stays gated, HTTP APIs ungated
- (No changes needed: all `api/gcp/*`, `api/ns_*` - already platform-independent)

### UI
- `mobile/src/ui_tabs/mod.rs` - Gate platform/ssh/ns tabs for desktop-only

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Android build still fails | Low | High | Incremental testing, rollback plan |
| Runtime crashes on Android | Medium | High | Extensive logging, logcat monitoring |
| Desktop build breaks | Low | High | Test desktop builds in parallel |
| Missing Android-specific code paths | Medium | Medium | Defensive error handling, graceful degradation |
| Performance issues on Android | Low | Medium | Profile after build succeeds |

## Success Metrics

**Immediate (This Design):**
- ✅ `./build.sh` completes without errors
- ✅ Android APK installs and launches
- ✅ No crashes in first 5 minutes of usage
- ✅ Available UI tabs render correctly

**Future (Deferred):**
- Responsive UI works at 1400px breakpoint
- Full feature parity for platform/SSH/NS tabs
- Android users can manage GCP, DNS, Docker, SSH from mobile app

## Conclusion

This design removes incorrect platform gates from platform-independent modules, enabling Android builds to access HTTP API clients, config structures, and business logic that work on any platform. Desktop-specific code (X11 detection, CLI, tray) remains properly gated. Desktop-specific UI tabs (platform, SSH, NS wizards) are temporarily gated for Android pending responsive UI implementation.

**Approach**: Minimal Platform Gates (simplest, fastest path to working Android builds)  
**Next Step**: Create implementation plan with detailed task breakdown
