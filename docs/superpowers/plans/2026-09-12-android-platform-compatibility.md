# Android Platform Compatibility Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable Android builds by removing incorrect platform gates from platform-independent modules

**Architecture:** Remove cfg gates from HTTP API clients, config data structures, and business logic modules. Keep gates only on truly platform-specific code (X11 desktop detection, CLI, tray). Gate desktop-specific UI tabs (platform, SSH, NS) for Android.

**Tech Stack:** Rust, cargo-ndk, Android NDK, egui, eframe

**Spec:** `docs/superpowers/specs/2026-09-12-android-platform-compatibility-design.md`

## Global Constraints

- Rust nightly toolchain required
- Branch: `feature/mobile-desktop-compat`
- Build target: `arm64-v8a` (Android 64-bit)
- Desktop builds must continue working
- Follow TDD: verify failure before implementing fix
- Commit after each working task

---

## File Structure

### Files to Modify
- `mobile/src/lib.rs` - Remove config gate (line 28-29), fix Android Config::new() (line 132)
- `mobile/src/config.rs` - Add AppConfig::default_android() method
- `mobile/src/android/mod.rs` - Export all submodules (currently empty)
- `mobile/src/calc/ns.rs` - Remove platform gate from acme imports (line 16-19)
- `mobile/src/api/mod.rs` - Verify desktop.rs stays gated, HTTP APIs ungated
- `mobile/src/ui_tabs/mod.rs` - Add cfg gates to platform, ssh, ns modules

### Verification Points
- After Task 1-5: `cargo check --target aarch64-linux-android --lib --features gui`
- After Task 6: `cargo ndk -t arm64-v8a build --release --lib --features gui`
- After Task 7: `./build.sh` (full Android build)
- After Task 8: Runtime test on device/emulator

---

### Task 1: Remove Config Platform Gate

**Files:**
- Modify: `mobile/src/lib.rs:28-29` (remove config gate)
- Modify: `mobile/src/lib.rs:132` (fix Android Config initialization)
- Modify: `mobile/src/config.rs` (add default_android method)

**Interfaces:**
- Consumes: None (first task)
- Produces: `config::AppConfig::default_android()` method for Android builds

- [ ] **Step 1: Verify current build failure**

Run: `cargo check --target aarch64-linux-android --lib --features gui`
Expected: FAIL with "unresolved import `crate::config`" on line 132

```bash
cd mobile
cargo check --target aarch64-linux-android --lib --features gui 2>&1 | grep -A5 "unresolved import.*config"
```

- [ ] **Step 2: Remove config platform gate in lib.rs**

Edit `mobile/src/lib.rs` lines 28-29:

```rust
// BEFORE
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod config;

// AFTER
pub mod config;
```

- [ ] **Step 3: Add default_android method to config.rs**

Add to `mobile/src/config.rs` in the `impl AppConfig` block:

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
    
    // Existing load_or_default stays below (desktop-only)
}
```

- [ ] **Step 4: Fix Android Config::new() to use default_android**

Edit `mobile/src/lib.rs` line 132:

```rust
// BEFORE
let app_config = config::AppConfig::default();

// AFTER  
let app_config = config::AppConfig::default_android();
```

- [ ] **Step 5: Verify build now succeeds for config**

Run: `cargo check --target aarch64-linux-android --lib --features gui`
Expected: PASS (or new errors from other ungated modules, not config)

```bash
cd mobile
cargo check --target aarch64-linux-android --lib --features gui 2>&1 | grep -v "unresolved import.*config" | head -20
```

- [ ] **Step 6: Commit**

```bash
git add mobile/src/lib.rs mobile/src/config.rs
git commit -m "fix: remove config platform gate for Android

- Remove cfg gate from config module in lib.rs
- Add AppConfig::default_android() for Android builds
- Fix Config::new() on Android to use default_android()

This allows Android builds to import config module (data structures).
Desktop config loading (config.yml) stays platform-specific."
```

---

### Task 2: Export Android Modules

**Files:**
- Modify: `mobile/src/android/mod.rs` (export all submodules)

**Interfaces:**
- Consumes: None
- Produces: `crate::android::*` modules accessible on Android

- [ ] **Step 1: Verify android mod is empty**

```bash
cat mobile/src/android/mod.rs
# Expected: Empty file or no meaningful exports
```

- [ ] **Step 2: Export all Android submodules**

Replace contents of `mobile/src/android/mod.rs`:

```rust
//! Android platform integration modules
//!
//! Provides JNI bindings and Android-specific utilities.

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

- [ ] **Step 3: Verify modules are exported**

```bash
cd mobile
cargo check --target aarch64-linux-android --lib --features gui 2>&1 | grep -E "(android::|clipboard|activity)" || echo "No Android module errors - good!"
```

- [ ] **Step 4: Commit**

```bash
git add mobile/src/android/mod.rs
git commit -m "feat: export Android submodules

Export all Android-specific modules:
- activity (lifecycle management)
- clipboard (JNI clipboard integration)
- contexttheme (theme context)
- inputmethod (soft keyboard)
- log (Android logging)
- packagemanager (package info)
- screensize (screen metrics)

These were implemented but not exported from mod.rs."
```

---

### Task 3: Remove Calc Module Platform Gates

**Files:**
- Modify: `mobile/src/calc/ns.rs:16-19` (remove platform gate from acme imports)

**Interfaces:**
- Consumes: `crate::calc::acme` module (platform-independent HTTP APIs)
- Produces: `calc::ns` module accessible on Android

- [ ] **Step 1: Verify current import gate**

```bash
grep -A4 "cfg.*android" mobile/src/calc/ns.rs | grep -A3 "use crate::calc::acme"
# Expected: Shows cfg gate before acme imports
```

- [ ] **Step 2: Remove platform gate from acme imports**

Edit `mobile/src/calc/ns.rs` lines 16-19:

```rust
// BEFORE
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
use crate::calc::acme::{
    DnsProvider, DnsProviderType, set_a_record, set_aaaa_record, set_txt_record,
};

// AFTER
use crate::calc::acme::{
    DnsProvider, DnsProviderType, set_a_record, set_aaaa_record, set_txt_record,
};
```

- [ ] **Step 3: Verify calc/ns compiles for Android**

```bash
cd mobile
cargo check --target aarch64-linux-android --lib --features gui 2>&1 | grep -E "(calc::ns|calc::acme)" || echo "No calc module errors - good!"
```

- [ ] **Step 4: Verify calc/docker and calc/ansible don't have gates**

```bash
grep "cfg.*android" mobile/src/calc/docker.rs mobile/src/calc/ansible.rs mobile/src/calc/acme.rs
# Expected: No output (these modules don't have incorrect gates)
```

- [ ] **Step 5: Commit**

```bash
git add mobile/src/calc/ns.rs
git commit -m "fix: remove platform gate from calc/ns acme imports

The acme module (certificate management) uses HTTP APIs and works
on all platforms. Remove incorrect Android gate from imports.

calc/docker, calc/ansible, calc/acme already platform-independent."
```

---

### Task 4: Verify API Module Gates

**Files:**
- Modify: `mobile/src/api/mod.rs` (verify desktop.rs stays gated, HTTP APIs ungated)

**Interfaces:**
- Consumes: HTTP API modules (gcp, ns_cloudflare, ns_porkbun, etc.)
- Produces: API modules accessible on all platforms (except desktop.rs)

- [ ] **Step 1: Check current api/mod.rs gates**

```bash
grep -E "pub mod (desktop|gcp|ns_)" mobile/src/api/mod.rs
# Expected: desktop should have cfg gate, others should not
```

- [ ] **Step 2: Verify desktop.rs gate is correct**

Check that `api/desktop.rs` has the desktop-only gate:

```bash
grep -B1 "pub mod desktop" mobile/src/api/mod.rs
# Expected: Should show cfg gate for desktop
```

If desktop.rs is NOT gated, add the gate:

```rust
// Desktop-only (X11 desktop environment detection)
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub mod desktop;
```

- [ ] **Step 3: Verify HTTP API modules are ungated**

Check that these modules have NO cfg gates:

```bash
grep -B1 "pub mod (gcp|ns_cloudflare|ns_porkbun|ns_duckdns|ns_gcp|ehttp_cache)" mobile/src/api/mod.rs
# Expected: No cfg gates before these modules
```

If any HTTP API module has a gate, remove it. The correct structure:

```rust
// Platform-independent HTTP APIs (no gates)
pub mod ehttp_cache;
pub mod gcp;
pub mod ns_cloudflare;
pub mod ns_duckdns;
pub mod ns_gcp;
pub mod ns_porkbun;
```

- [ ] **Step 4: Verify API modules compile for Android**

```bash
cd mobile
cargo check --target aarch64-linux-android --lib --features gui 2>&1 | grep -E "api::(gcp|ns_)" || echo "No API module errors - good!"
```

- [ ] **Step 5: Commit (if changes made)**

```bash
git add mobile/src/api/mod.rs
git commit -m "fix: verify API module platform gates

- Keep desktop.rs gated (X11 detection, desktop-specific)
- Ensure HTTP API modules ungated (gcp, ns_cloudflare, etc.)

HTTP API clients work on all platforms via ureq/ehttp."
```

---

### Task 5: Gate Desktop-Specific UI Tabs

**Files:**
- Modify: `mobile/src/ui_tabs/mod.rs` (add desktop-only gates to platform, ssh, ns)

**Interfaces:**
- Consumes: UI tab modules
- Produces: Desktop-only tabs (platform, ssh, ns) excluded from Android builds

- [ ] **Step 1: Check current ui_tabs/mod.rs**

```bash
grep -E "pub mod (platform|ssh|ns)" mobile/src/ui_tabs/mod.rs
# Expected: These modules likely have no cfg gates currently
```

- [ ] **Step 2: Add desktop-only gates to platform/ssh/ns tabs**

Edit `mobile/src/ui_tabs/mod.rs`, find the lines with `pub mod platform`, `pub mod ssh`, `pub mod ns` and add cfg gates:

```rust
// Android-compatible tabs (keep as-is, no changes)
pub mod client;
pub mod orders;
pub mod products;
pub mod channel;
pub mod dm;
pub mod members;
pub mod email;
pub mod roles;
pub mod site;

// Desktop-only tabs (have GcpWizard and desktop-specific wizards)
#[cfg(not(target_os = "android"))]
pub mod platform;

#[cfg(not(target_os = "android"))]
pub mod ssh;

#[cfg(not(target_os = "android"))]
pub mod ns;
```

- [ ] **Step 3: Verify gated tabs don't compile for Android**

```bash
cd mobile
cargo check --target aarch64-linux-android --lib --features gui 2>&1 | grep -E "ui_tabs::(platform|ssh|ns)" && echo "ERROR: Desktop tabs should be excluded!" || echo "Desktop tabs correctly excluded"
```

- [ ] **Step 4: Verify Android-compatible tabs still compile**

```bash
cd mobile
cargo check --target aarch64-linux-android --lib --features gui 2>&1 | grep -E "ui_tabs::(client|orders|products)" || echo "Android tabs compile correctly"
```

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_tabs/mod.rs
git commit -m "feat: gate desktop-specific UI tabs for Android

Gate platform, ssh, ns tabs as desktop-only:
- platform.rs has GcpWizard (desktop dialog)
- ssh.rs has desktop-specific wizards
- ns.rs has desktop-specific wizards

Android-compatible tabs remain accessible:
- client, orders, products, channel, dm, members, email, roles, site

Future work: Responsive UI with 1400px breakpoint for mobile views."
```

---

### Task 6: Rust Library Build Verification

**Files:**
- None (verification task)

**Interfaces:**
- Consumes: All changes from Tasks 1-5
- Produces: Verified Android Rust library build succeeds

- [ ] **Step 1: Clean build artifacts**

```bash
cd mobile
cargo clean
```

- [ ] **Step 2: Run cargo ndk build for arm64-v8a**

```bash
cd mobile
cargo ndk -t arm64-v8a build --release --lib --features gui
```

Expected: BUILD SUCCEEDS with no errors

- [ ] **Step 3: Verify library output exists**

```bash
ls -lh mobile/target/aarch64-linux-android/release/libdure.so
# Expected: File exists (Android shared library)
```

- [ ] **Step 4: Check for remaining platform gate errors**

```bash
cd mobile
cargo ndk -t arm64-v8a build --release --lib --features gui 2>&1 | grep -i "unresolved import" && echo "ERROR: Still have unresolved imports!" || echo "✓ No unresolved imports"
```

- [ ] **Step 5: Verify desktop build still works**

```bash
cd mobile
cargo build --release --bin dure-desktop --features gui
# Expected: Desktop build succeeds
```

- [ ] **Step 6: Document verification results**

Create verification log:

```bash
echo "# Android Platform Compatibility Build Verification" > /tmp/build-verification.log
echo "Date: $(date)" >> /tmp/build-verification.log
echo "" >> /tmp/build-verification.log
echo "## Android Build (arm64-v8a)" >> /tmp/build-verification.log
cargo ndk -t arm64-v8a build --release --lib --features gui 2>&1 | tail -20 >> /tmp/build-verification.log
echo "" >> /tmp/build-verification.log
echo "## Desktop Build" >> /tmp/build-verification.log
cargo build --release --bin dure-desktop --features gui 2>&1 | tail -10 >> /tmp/build-verification.log

cat /tmp/build-verification.log
```

- [ ] **Step 7: Commit verification checkpoint**

```bash
git add -A
git commit -m "chore: verify Android Rust library build succeeds

Verification results:
- ✓ cargo ndk arm64-v8a build succeeds
- ✓ libdure.so generated
- ✓ No unresolved import errors
- ✓ Desktop build still works

Ready for full Android build (Gradle APK assembly)."
```

---

### Task 7: Full Android Build Test

**Files:**
- None (verification task using build.sh)

**Interfaces:**
- Consumes: Working Rust library from Task 6
- Produces: Verified full Android APK build succeeds

- [ ] **Step 1: Clean previous build artifacts**

```bash
cd mobile
./gradlew clean 2>&1 | tail -5
cargo clean
```

- [ ] **Step 2: Run full build script**

```bash
cd mobile
./build.sh 2>&1 | tee /tmp/full-build.log
```

Expected: Build completes successfully

Monitor for:
- ✓ Rust compilation for arm64-v8a
- ✓ Gradle assembleDebug or assembleRelease
- ✓ APK creation

- [ ] **Step 3: Verify APK output exists**

```bash
find mobile/app/build/outputs/apk -name "*.apk" -type f
# Expected: app-debug.apk or app-release.apk exists
```

- [ ] **Step 4: Check APK size is reasonable**

```bash
ls -lh mobile/app/build/outputs/apk/debug/app-debug.apk
# Expected: File size ~15-50 MB (reasonable for Android app with egui)
```

- [ ] **Step 5: Verify libc++_shared.so was copied**

```bash
ls -lh mobile/app/src/main/jniLibs/arm64-v8a/libc++_shared.so
# Expected: File exists (NDK C++ runtime for DuckDB)
```

- [ ] **Step 6: Check build log for errors**

```bash
grep -i "error" /tmp/full-build.log | grep -v "0 errors" | head -20
# Expected: No actual error lines (only "0 errors" success messages)
```

- [ ] **Step 7: Commit build success checkpoint**

```bash
git add -A
git commit -m "chore: verify full Android build succeeds

Full build.sh verification:
- ✓ Rust library compiled for arm64-v8a
- ✓ Gradle assembled APK successfully
- ✓ APK output: $(find mobile/app/build/outputs/apk -name '*.apk' | head -1)
- ✓ libc++_shared.so copied to jniLibs
- ✓ No build errors

Ready for runtime validation on device/emulator."
```

---

### Task 8: Runtime Validation

**Files:**
- None (runtime testing task)

**Interfaces:**
- Consumes: Built APK from Task 7
- Produces: Verified Android app launches and runs without crashes

- [ ] **Step 1: Check device/emulator connection**

```bash
adb devices
# Expected: At least one device/emulator listed
```

If no devices:
```bash
# Option 1: Connect physical device via USB (enable USB debugging)
# Option 2: Start emulator
# emulator -avd <avd-name>
```

- [ ] **Step 2: Install APK on device/emulator**

```bash
adb install -r mobile/app/build/outputs/apk/debug/app-debug.apk
# Expected: Success message
```

- [ ] **Step 3: Launch app**

```bash
adb shell am start -n pe.nikescar.dure/.MainActivity
# Expected: App launches
```

- [ ] **Step 4: Monitor logcat for crashes**

```bash
adb logcat -c  # Clear old logs
adb logcat -s Dure:V -s AndroidRuntime:E | head -100
# Monitor for:
# - ✓ "Dure v<version> starting on Android"
# - ✓ "Database path set to: /data/data/pe.nikescar.dure/files/dure.db"
# - ✓ "DureApp initialized"
# - ✗ No "FATAL EXCEPTION" or crash logs
```

Wait 30 seconds and check app is still running.

- [ ] **Step 5: Verify database initialization**

```bash
adb shell ls -lh /data/data/pe.nikescar.dure/files/dure.db
# Expected: Database file exists
```

- [ ] **Step 6: Verify app directories created**

```bash
adb shell ls -lh /data/data/pe.nikescar.dure/
# Expected: files/, cache/ directories exist
```

- [ ] **Step 7: Check UI tabs are accessible**

Manually test on device/emulator:
- [ ] Open app, verify main screen loads
- [ ] Navigate to Client tab - should work
- [ ] Navigate to Products tab - should work
- [ ] Navigate to Orders tab - should work
- [ ] Verify Platform/SSH/NS tabs are NOT visible (desktop-only)

- [ ] **Step 8: Check for JNI symbol errors**

```bash
adb logcat -d | grep -i "UnsatisfiedLinkError\|JNI\|symbol"
# Expected: No symbol resolution errors
```

- [ ] **Step 9: Document runtime validation results**

```bash
cat > /tmp/runtime-validation.md << 'EOF'
# Android Runtime Validation Results

Date: $(date)

## Device Info
$(adb shell getprop ro.product.model)
Android Version: $(adb shell getprop ro.build.version.release)

## Installation
- ✓ APK installed successfully
- ✓ App launches without crashes

## Database
- ✓ Database created at /data/data/pe.nikescar.dure/files/dure.db
- ✓ Config directories initialized

## UI Verification
- ✓ Main screen loads
- ✓ Android-compatible tabs work (Client, Products, Orders)
- ✓ Desktop-only tabs excluded (Platform, SSH, NS)

## Logs
No crashes or JNI errors in first 60 seconds of runtime.

## Next Steps
- Responsive UI implementation (1400px breakpoint)
- Android-native UI for platform/SSH/NS features
EOF

cat /tmp/runtime-validation.md
```

- [ ] **Step 10: Final commit**

```bash
git add -A
git commit -m "chore: verify Android runtime validation passes

Runtime verification results:
- ✓ App installs on device/emulator
- ✓ App launches without crashes
- ✓ Database initializes correctly
- ✓ Android-compatible UI tabs work
- ✓ Desktop-only tabs properly excluded
- ✓ No JNI symbol errors
- ✓ Stable for 60+ seconds

ANDROID PLATFORM COMPATIBILITY COMPLETE

All immediate goals achieved:
1. Android builds compile successfully
2. Platform-independent modules work on Android
3. Desktop-specific code properly gated
4. Android app launches without crashes

Deferred to future work:
- Responsive UI (1400px breakpoint)
- Full UI parity for platform/SSH/NS tabs"
```

---

## Spec Coverage Self-Review

Checking spec requirements against tasks:

**Spec Section 1: Core Library Changes** ✅
- Task 1 covers: Remove config gate, fix Android Config::new()

**Spec Section 2: Config Module** ✅
- Task 1 covers: Add AppConfig::default_android()

**Spec Section 3: Android Module** ✅
- Task 2 covers: Export all Android submodules

**Spec Section 4: Calc Modules** ✅
- Task 3 covers: Remove gates from calc/ns

**Spec Section 5: API Modules** ✅
- Task 4 covers: Verify desktop.rs gated, HTTP APIs ungated

**Spec Section 6: UI Tabs** ✅
- Task 5 covers: Gate desktop-specific tabs (platform, ssh, ns)

**Spec Testing Strategy** ✅
- Task 6 covers: Phase 1 (Rust library build)
- Task 7 covers: Phase 2 (Full Android build)
- Task 8 covers: Phase 3 (Runtime validation)

**No gaps found.** All spec requirements covered by tasks.

---

## Type Consistency Check

Function signatures used across tasks:

- `config::AppConfig::default_android()` → Defined in Task 1, used in Task 1 ✅
- Android module exports → Defined in Task 2, accessible after Task 2 ✅
- `calc::acme` imports → Used in Task 3, already exist ✅
- API module structure → Verified in Task 4 ✅
- UI tab gates → Applied in Task 5 ✅

All types and signatures consistent across tasks.
