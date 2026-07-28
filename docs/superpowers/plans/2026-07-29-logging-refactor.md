# Logging Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Standardize logging across the Dure codebase with tab-separated format and automatic module prefixes.

**Architecture:** Create custom logging macros (`dure_info!`, `dure_debug!`, etc.) that automatically inject module prefixes and format logs consistently across desktop, Android, and WASM platforms. Migrate all existing `log::*!` calls to use the new macros.

**Tech Stack:** Rust log crate (0.4), env_logger, android_logger, web_sys::console (WASM)

## Global Constraints

- Rust nightly toolchain required
- Log format: `timestamp\t[LEVEL]\t[Module]\tmessage`
- Debug logs only appear when `RUST_LOG=debug` is set
- Must work on desktop (env_logger), Android (android_logger), and WASM (console.log)
- Module prefixes follow naming convention: `[GCP::OAuth]`, `[UI::SSH]`, `[DB]`, etc.
- All macros exported via `#[macro_export]` for global availability
- DRY: Do not repeat module prefix manually - macros handle it automatically
- YAGNI: Only implement the four core macros (info, debug, warn, error)
- TDD: Test each layer before migrating the next
- Frequent commits after each major step

---

### Task 1: Create Logging Infrastructure

**Files:**
- Create: `mobile/src/logging.rs`
- Modify: `mobile/src/lib.rs:1-150` (add module declaration)
- Modify: `mobile/src/main.rs:1-100` (update logger init)
- Modify: `mobile/src/main_android.rs:1-50` (update logger init)

**Interfaces:**
- Consumes: N/A (foundational task)
- Produces:
  - `pub fn module_name(path: &str) -> String` - transforms module paths
  - `dure_info!(format, args...)` - info-level logging macro
  - `dure_debug!(format, args...)` - debug-level logging macro
  - `dure_warn!(format, args...)` - warning-level logging macro
  - `dure_error!(format, args...)` - error-level logging macro

- [ ] **Step 1: Create mobile/src/logging.rs with module_name function**

```rust
//! Dure logging infrastructure with automatic module prefixes
//!
//! Provides custom logging macros that automatically inject module context
//! and format logs consistently across all platforms.

/// Transform Rust module path into clean component name
///
/// Examples:
/// - "dure::api::gcp::oauth" -> "GCP::OAuth"
/// - "dure::calc::db" -> "DB"
/// - "dure::ui_tabs::ssh" -> "UI::SSH"
/// - "dure::viewmodel::platform::actor" -> "Platform::Actor"
pub fn module_name(path: &str) -> String {
    // Strip "dure::" prefix if present
    let path = path.strip_prefix("dure::").unwrap_or(path);
    
    // Handle special cases first
    if path.starts_with("api::gcp::") {
        // api::gcp::oauth -> GCP::OAuth
        let component = path.strip_prefix("api::gcp::").unwrap();
        return format!("GCP::{}", capitalize_component(component));
    }
    
    if path.starts_with("ui_tabs::") {
        // ui_tabs::ssh -> UI::SSH
        let component = path.strip_prefix("ui_tabs::").unwrap();
        return format!("UI::{}", component.to_uppercase());
    }
    
    if path.starts_with("viewmodel::") {
        // viewmodel::platform::actor -> Platform::Actor
        let rest = path.strip_prefix("viewmodel::").unwrap();
        let parts: Vec<&str> = rest.split("::").collect();
        if parts.len() >= 2 {
            return format!("{}::{}", capitalize_component(parts[0]), capitalize_component(parts[1]));
        }
        return capitalize_component(rest);
    }
    
    if path.starts_with("wss::") {
        // wss::server -> WSS::Server
        let component = path.strip_prefix("wss::").unwrap();
        return format!("WSS::{}", capitalize_component(component));
    }
    
    // Handle calc:: (business logic)
    if path.starts_with("calc::") {
        // calc::db -> DB
        let component = path.strip_prefix("calc::").unwrap();
        return component.to_uppercase();
    }
    
    // Handle api:: (non-GCP)
    if path.starts_with("api::") {
        // api::ehttp_cache -> API::EhttpCache
        let component = path.strip_prefix("api::").unwrap();
        return format!("API::{}", capitalize_component(component));
    }
    
    // Handle android::
    if path.starts_with("android::") {
        return "Android".to_string();
    }
    
    // Handle attestation::
    if path.starts_with("attestation::") {
        return "Attestation".to_string();
    }
    
    // Handle site::
    if path.starts_with("site::") {
        return "Site".to_string();
    }
    
    // Fallback: use last 2 components or capitalize single component
    let parts: Vec<&str> = path.split("::").collect();
    if parts.len() >= 2 {
        format!("{}::{}", 
                capitalize_component(parts[parts.len() - 2]),
                capitalize_component(parts[parts.len() - 1]))
    } else if parts.len() == 1 {
        capitalize_component(parts[0])
    } else {
        "Unknown".to_string()
    }
}

/// Capitalize a component name (handle acronyms and snake_case)
fn capitalize_component(s: &str) -> String {
    // Handle known acronyms
    match s {
        "gcp" => "GCP".to_string(),
        "wss" => "WSS".to_string(),
        "ssh" => "SSH".to_string(),
        "db" => "DB".to_string(),
        "dns" => "DNS".to_string(),
        "api" => "API".to_string(),
        "oauth" => "OAuth".to_string(),
        "vm" => "VM".to_string(),
        "ui" => "UI".to_string(),
        _ => {
            // Handle snake_case: ehttp_cache -> EhttpCache
            s.split('_')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().chain(chars).collect(),
                    }
                })
                .collect::<Vec<_>>()
                .join("")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_name_gcp() {
        assert_eq!(module_name("dure::api::gcp::oauth"), "GCP::OAuth");
        assert_eq!(module_name("dure::api::gcp::compute"), "GCP::Compute");
    }

    #[test]
    fn test_module_name_calc() {
        assert_eq!(module_name("dure::calc::db"), "DB");
        assert_eq!(module_name("dure::calc::ssh"), "SSH");
    }

    #[test]
    fn test_module_name_ui() {
        assert_eq!(module_name("dure::ui_tabs::ssh"), "UI::SSH");
        assert_eq!(module_name("dure::ui_tabs::platform"), "UI::PLATFORM");
    }

    #[test]
    fn test_module_name_viewmodel() {
        assert_eq!(module_name("dure::viewmodel::platform::actor"), "Platform::Actor");
    }

    #[test]
    fn test_module_name_wss() {
        assert_eq!(module_name("dure::wss::server"), "WSS::Server");
        assert_eq!(module_name("dure::wss::client"), "WSS::Client");
    }

    #[test]
    fn test_module_name_android() {
        assert_eq!(module_name("dure::android::activity"), "Android");
    }
}
```

- [ ] **Step 2: Run tests to verify module_name function**

Run: `cd mobile && cargo test logging::tests --lib`
Expected: All tests PASS

- [ ] **Step 3: Add logging macros to mobile/src/logging.rs**

Append to `mobile/src/logging.rs`:

```rust
// Non-WASM platforms: use standard log crate
#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! dure_info {
    ($($arg:tt)*) => {
        log::info!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*))
    };
}

#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! dure_debug {
    ($($arg:tt)*) => {
        log::debug!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*))
    };
}

#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! dure_warn {
    ($($arg:tt)*) => {
        log::warn!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*))
    };
}

#[cfg(not(target_family = "wasm"))]
#[macro_export]
macro_rules! dure_error {
    ($($arg:tt)*) => {
        log::error!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*))
    };
}

// WASM platform: use web_sys::console
#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! dure_info {
    ($($arg:tt)*) => {
        web_sys::console::log_1(
            &format!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*)).into()
        )
    };
}

#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! dure_debug {
    ($($arg:tt)*) => {
        // In WASM, debug logs still go to console (user controls via browser devtools)
        web_sys::console::debug_1(
            &format!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*)).into()
        )
    };
}

#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! dure_warn {
    ($($arg:tt)*) => {
        web_sys::console::warn_1(
            &format!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*)).into()
        )
    };
}

#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! dure_error {
    ($($arg:tt)*) => {
        web_sys::console::error_1(
            &format!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*)).into()
        )
    };
}
```

- [ ] **Step 4: Add logging module to mobile/src/lib.rs**

Find the module declarations in `mobile/src/lib.rs` and add:

```rust
pub mod logging;
```

After existing module declarations (e.g., after `pub mod config;`).

- [ ] **Step 5: Update desktop logger initialization in mobile/src/main.rs**

Find the logger initialization code in `main.rs` and replace it with:

```rust
fn init_logger() {
    use env_logger::fmt::Color;
    
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format(|buf, record| {
            use std::io::Write;
            
            let timestamp = buf.timestamp();
            let level = record.level();
            
            // Color coding by level
            let mut level_style = buf.style();
            match level {
                log::Level::Error => level_style.set_color(Color::Red).set_bold(true),
                log::Level::Warn => level_style.set_color(Color::Yellow).set_bold(true),
                log::Level::Info => level_style.set_color(Color::Green),
                log::Level::Debug => level_style.set_color(Color::Blue),
                log::Level::Trace => level_style.set_color(Color::Magenta),
            };
            
            writeln!(
                buf,
                "{}\t{}\t{}",
                timestamp,
                level_style.value(format!("[{}]", level)),
                record.args()
            )
        })
        .init();
}
```

If `init_logger()` doesn't exist, add it before `main()` and call it at the start of `main()`:

```rust
fn main() {
    init_logger();
    // ... rest of main
}
```

- [ ] **Step 6: Update Android logger initialization in mobile/src/main_android.rs**

Find the logger initialization code in `main_android.rs` and ensure it looks like:

```rust
fn init_logger() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("Dure")
    );
}
```

Call it at the start of the Android activity/main function if not already called.

- [ ] **Step 7: Test macro compilation**

Run: `cd mobile && cargo build --lib`
Expected: Build succeeds with no errors

- [ ] **Step 8: Create manual test file to verify macros work**

Create `mobile/src/logging_test.rs`:

```rust
//! Manual test for logging macros

pub fn test_logging() {
    dure_info!("Test info message");
    dure_debug!("Test debug message with arg: {}", 42);
    dure_warn!("Test warning");
    dure_error!("Test error: {}", "something went wrong");
}
```

Add to `mobile/src/lib.rs`:
```rust
#[cfg(test)]
pub mod logging_test;
```

- [ ] **Step 9: Run manual test to see output**

Run: `cd mobile && RUST_LOG=debug cargo test logging_test::test_logging -- --nocapture`
Expected: See colored, tab-separated output with module prefix like `[LoggingTest]`

- [ ] **Step 10: Commit logging infrastructure**

```bash
git add mobile/src/logging.rs mobile/src/lib.rs mobile/src/main.rs mobile/src/main_android.rs mobile/src/logging_test.rs
git commit -m "feat: add logging infrastructure with auto module prefixes

- Add logging.rs with module_name() transformation function
- Create dure_info!, dure_debug!, dure_warn!, dure_error! macros
- Add WASM support via web_sys::console
- Update logger init for tab-separated format with colors
- Add unit tests for module name transformations

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 2: Migrate Business Logic and API Layer

**Files:**
- Modify: `mobile/src/calc/*.rs` (~15 files)
- Modify: `mobile/src/api/gcp/*.rs` (~8 files)
- Modify: `mobile/src/api/*.rs` (~3 files)

**Interfaces:**
- Consumes: `dure_info!`, `dure_debug!`, `dure_warn!`, `dure_error!` from Task 1
- Produces: N/A (migration task)

- [ ] **Step 1: Migrate mobile/src/calc/db.rs**

Find and replace in `mobile/src/calc/db.rs`:
- Remove the `console_log!` macro definition (lines 36-43)
- Replace `console_log!` usage with `dure_info!`
- Replace `log::info!` with `dure_info!`
- Remove `use log::{...}` if present

Example changes:
```rust
// OLD
console_log!("Connection established");

// NEW
dure_info!("Connection established");
```

- [ ] **Step 2: Migrate mobile/src/calc/lego.rs**

Replace all:
- `log::info!` → `dure_info!`
- `log::debug!` → `dure_debug!`
- `log::warn!` → `dure_warn!`
- `log::error!` → `dure_error!`

Remove: `use log::{info, debug, warn, error};` if present

- [ ] **Step 3: Migrate remaining calc/ files**

Apply same replacements to:
- `mobile/src/calc/crypt.rs`
- `mobile/src/calc/dns.rs`
- `mobile/src/calc/gcp.rs`
- `mobile/src/calc/gcp_rest.rs`
- `mobile/src/calc/platform.rs`
- `mobile/src/calc/platform_gcp.rs`
- `mobile/src/calc/hosting.rs`
- `mobile/src/calc/hosting_gcp.rs`
- `mobile/src/calc/nft.rs`
- `mobile/src/calc/ns.rs`
- `mobile/src/calc/site.rs`
- `mobile/src/calc/keyring.rs`
- `mobile/src/calc/session.rs`
- `mobile/src/calc/audit.rs`

For each file:
- `log::info!` → `dure_info!`
- `log::debug!` → `dure_debug!`
- `log::warn!` → `dure_warn!`
- `log::error!` → `dure_error!`
- Remove `use log::*;` imports

- [ ] **Step 4: Migrate mobile/src/api/gcp/oauth.rs**

Replace in `mobile/src/api/gcp/oauth.rs`:
- Line 419: `log::debug!(` → `dure_debug!(`
- Line 408: `log::error!(` → `dure_error!(`
- Line 412: `log::error!(` → `dure_error!(`
- Line 442: `log::error!(` → `dure_error!(`
- Line 457: `log::error!(` → `dure_error!(`

- [ ] **Step 5: Migrate mobile/src/api/gcp/compute.rs**

Replace in `mobile/src/api/gcp/compute.rs`:
- Line 796: `log::info!(` → `dure_info!(`
- Line 801: `log::warn!(` → `dure_warn!(`
- Line 809: `log::info!(` → `dure_info!(`
- Line 814: `log::warn!(` → `dure_warn!(`
- Line 824: `log::info!(` → `dure_info!(`
- Line 829: `log::info!(` → `dure_info!(`
- Line 857: `log::info!(` → `dure_info!(`
- Line 858: `log::info!(` → `dure_info!(`

- [ ] **Step 6: Migrate remaining api/gcp/ files**

Apply same replacements to:
- `mobile/src/api/gcp/billing.rs`
- `mobile/src/api/gcp/bigquery.rs`
- `mobile/src/api/gcp/dns.rs`
- `mobile/src/api/gcp/resourcemanager.rs`
- `mobile/src/api/gcp/serviceusage.rs`

- [ ] **Step 7: Migrate mobile/src/api/ehttp_cache.rs**

Replace:
- Line 4: `use log::{debug, info, warn};` → (remove this line)
- Replace `debug!(` → `dure_debug!(`
- Replace `info!(` → `dure_info!(`
- Replace `warn!(` → `dure_warn!(`

- [ ] **Step 8: Migrate other api/ files**

Apply same replacements to:
- `mobile/src/api/desktop.rs`
- `mobile/src/api/ns_cloudflare.rs`
- `mobile/src/api/ns_gcp.rs`
- `mobile/src/api/ns_duckdns.rs`
- `mobile/src/api/ns_porkbun.rs`

- [ ] **Step 9: Test that calc and api modules compile**

Run: `cd mobile && cargo build --lib`
Expected: Build succeeds

- [ ] **Step 10: Commit business logic and API layer migration**

```bash
git add mobile/src/calc/ mobile/src/api/
git commit -m "refactor: migrate calc and api layers to dure logging macros

- Replace log::info! with dure_info! in calc/ (~15 files)
- Replace log::debug! with dure_debug! in api/gcp/ (~8 files)
- Remove console_log! macro from calc/db.rs
- Remove unused log crate imports

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 3: Migrate Application Layer

**Files:**
- Modify: `mobile/src/viewmodel/**/*.rs` (~10 files)
- Modify: `mobile/src/ui_tabs/*.rs` (visible tabs only)
- Modify: `mobile/src/android/*.rs` (~5 files)
- Modify: `mobile/src/wss/**/*.rs`
- Modify: `mobile/src/attestation/*.rs` (~2 files)
- Modify: `mobile/src/*.rs` (root level files)

**Interfaces:**
- Consumes: `dure_info!`, `dure_debug!`, `dure_warn!`, `dure_error!` from Task 1
- Produces: N/A (migration task)

- [ ] **Step 1: Migrate mobile/src/viewmodel/platform/actor.rs**

Replace in `mobile/src/viewmodel/platform/actor.rs`:
- Line 24: `log::info!("PlatformActor started");` → `dure_info!("Starting actor loop");`
- Line 30: `log::error!("PlatformActor command failed: {}", e);` → `dure_error!("Command failed: {}", e);`
- Line 34: `log::info!("PlatformActor: channel closed, shutting down");` → `dure_info!("Channel closed, shutting down");`

Note: Improved messages to follow design spec (simple descriptive style).

- [ ] **Step 2: Migrate other viewmodel/ files**

Apply same replacements to all files in:
- `mobile/src/viewmodel/`
- `mobile/src/viewmodel/platform/`
- `mobile/src/viewmodel/*/` (any subdirectories)

For each file:
- `log::info!` → `dure_info!`
- `log::debug!` → `dure_debug!`
- `log::warn!` → `dure_warn!`
- `log::error!` → `dure_error!`
- Remove `use log::*;`

- [ ] **Step 3: Migrate mobile/src/ui_tabs/ssh.rs**

Replace all log macro calls in `mobile/src/ui_tabs/ssh.rs`:
- `log::info!` → `dure_info!`
- `log::debug!` → `dure_debug!`
- `log::warn!` → `dure_warn!`
- `log::error!` → `dure_error!`

- [ ] **Step 4: Migrate other ui_tabs/ files (visible tabs only)**

Apply same replacements to:
- `mobile/src/ui_tabs/platform.rs`
- `mobile/src/ui_tabs/ns.rs`
- `mobile/src/ui_tabs/site.rs`
- `mobile/src/ui_tabs/roles.rs`

Skip hidden tabs (products, orders, channel, dm, members, client, email) as per CLAUDE.md.

- [ ] **Step 5: Migrate mobile/src/android/ files**

Apply replacements to:
- `mobile/src/android/activity.rs` (lines 14, 76, 123, 134, 216, 260, 265, 270, 275)
- `mobile/src/android/clipboard.rs` (line 479)
- `mobile/src/android/contexttheme.rs` (lines 122, 131, 135, 139)
- `mobile/src/android/inputmethod.rs` (lines 151, 153, 319, 321, 422)
- `mobile/src/android/screensize.rs` (lines 77, 88)

All: `log::debug!` → `dure_debug!`, `log::info!` → `dure_info!`, `log::warn!` → `dure_warn!`, `log::error!` → `dure_error!`

- [ ] **Step 6: Migrate mobile/src/wss/ files**

Apply same replacements to all files in:
- `mobile/src/wss/server/`
- `mobile/src/wss/client.rs`

- [ ] **Step 7: Migrate mobile/src/attestation/ files**

Replace in:
- `mobile/src/attestation/download.rs` (lines 58, 96, 104, 125, 146, 172)
- `mobile/src/attestation/verify.rs` (lines 22, 23, 29, 35)

- [ ] **Step 8: Migrate root level files**

Apply replacements to:
- `mobile/src/lib.rs` (lines 114, 126, 127, 145, 180)
- `mobile/src/main.rs`
- `mobile/src/main_android.rs`
- `mobile/src/main_wasm.rs`
- `mobile/src/dure.rs`
- `mobile/src/install.rs`
- `mobile/src/i18n.rs`

- [ ] **Step 9: Test full build**

Run: `cd mobile && cargo build`
Expected: Build succeeds with no errors

- [ ] **Step 10: Test with RUST_LOG=info (info messages only)**

Run: `cd mobile && RUST_LOG=info cargo run --bin dure-desktop`
Expected: Only info/warn/error messages appear, no debug messages

- [ ] **Step 11: Test with RUST_LOG=debug (all messages)**

Run: `cd mobile && RUST_LOG=debug cargo run --bin dure-desktop`
Expected: All log levels appear (debug, info, warn, error)

- [ ] **Step 12: Verify tab-separated format**

Check terminal output from Step 11.
Expected format: `2026-07-29 HH:MM:SS	[LEVEL]	[Module]	Message`

- [ ] **Step 13: Commit application layer migration**

```bash
git add mobile/src/viewmodel/ mobile/src/ui_tabs/ mobile/src/android/ mobile/src/wss/ mobile/src/attestation/ mobile/src/lib.rs mobile/src/main.rs mobile/src/main_android.rs mobile/src/dure.rs mobile/src/install.rs mobile/src/i18n.rs
git commit -m "refactor: migrate application layer to dure logging macros

- Migrate viewmodel/ (~10 files)
- Migrate ui_tabs/ (visible tabs)
- Migrate android/ (~5 files)
- Migrate wss/ (WebSocket server/client)
- Migrate attestation/ (2 files)
- Migrate root level files (lib, main, dure, install, i18n)
- All logs now use consistent tab-separated format

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 4: Documentation and Final Cleanup

**Files:**
- Modify: `docs/GUIDELINES_RUST_CODING.md` (add logging standards section)
- Delete: `mobile/src/logging_test.rs` (temporary test file)
- Modify: `mobile/src/lib.rs` (remove logging_test module)

**Interfaces:**
- Consumes: All migrated code from Tasks 1-3
- Produces: Complete, documented logging system

- [ ] **Step 1: Verify no remaining log:: calls**

Run: `grep -r "log::" mobile/src --include="*.rs" | grep -v "mobile/src/logging.rs"`
Expected: Only imports of `use log::Level;` or similar, no `log::info!(` etc.

If any `log::*!(` calls remain, replace them with `dure_*!(`.

- [ ] **Step 2: Remove unused log imports**

Search for and remove lines like:
- `use log::{info, debug, warn, error};`
- `use log::*;`

Keep only:
- `use log::Level;` (if needed for level checks)
- The log crate dependency in Cargo.toml

Run: `cd mobile && cargo build`
Expected: No "unused import" warnings

- [ ] **Step 3: Delete temporary test file**

Remove `mobile/src/logging_test.rs`:
```bash
rm mobile/src/logging_test.rs
```

Remove from `mobile/src/lib.rs`:
```rust
#[cfg(test)]
pub mod logging_test;
```

- [ ] **Step 4: Add logging standards to docs/GUIDELINES_RUST_CODING.md**

Check if `docs/GUIDELINES_RUST_CODING.md` exists:
```bash
ls docs/GUIDELINES_RUST_CODING.md
```

If it exists, add this section at the end. If it doesn't exist, create it with this content:

```markdown
# Rust Coding Guidelines

## Logging Standards

Use Dure logging macros with descriptive messages. The macros automatically add module context and format logs consistently.

### Good Examples

✅ **Info messages** (always visible):
```rust
dure_info!("Starting OAuth flow");
dure_info!("Created VM: {}", vm_name);
dure_info!("Actor loop shutting down");
```

✅ **Debug messages** (only when RUST_LOG=debug):
```rust
dure_debug!("Fetching images from project: {}", project_id);
dure_debug!("Refreshing access token for client: {}", client_id_prefix);
dure_debug!("Health check passed for host: {}", hostname);
```

✅ **Warning/Error messages**:
```rust
dure_warn!("No attestations found for binary: {}", binary_path);
dure_error!("Failed to connect to VM: {}", error);
```

### Bad Examples

❌ **Wrong macro** (bypasses module prefix):
```rust
log::info!("Starting OAuth flow");  // DON'T USE log:: directly
```

❌ **Too terse** (unclear what's happening):
```rust
dure_info!("OAuth");
dure_debug!("Fetch");
```

❌ **Too verbose** (belongs in debug, not info):
```rust
dure_info!("Parsed Docker image: {}:{}, with {} env vars and {} port mappings", 
           image, tag, env_count, port_count);
// Better as dure_debug!
```

### Guidelines

- **info**: Important events users/operators should see
- **debug**: Detailed troubleshooting information (only shows when `RUST_LOG=debug`)
- **warn**: Recoverable issues or unexpected conditions
- **error**: Failures that prevent operations from completing

### Log Format

Logs are automatically formatted as tab-separated columns:
```
2026-07-29 14:23:10	[INFO]	[Module::Component]	Message here
```

The module prefix is injected automatically - you don't need to add it to your messages.
```

- [ ] **Step 5: Test desktop build with full logging**

Run: `cd mobile && RUST_LOG=debug cargo run --bin dure-desktop -- --version`
Expected: 
- Application runs successfully
- Logs show tab-separated format with module prefixes
- Debug messages appear
- Colors display correctly (if terminal supports it)

- [ ] **Step 6: Test Android build (if possible)**

If Android development environment is set up:
```bash
cd mobile
./build.sh
```

Expected: Build succeeds

If you can run on device/emulator:
```bash
adb logcat | grep Dure
```

Expected: Module prefixes appear in logcat output

If Android env not available, skip this step.

- [ ] **Step 7: Test WASM build (if possible)**

If WASM build is configured:
```bash
cd mobile
cargo build --target wasm32-unknown-unknown
```

Expected: Build succeeds

If build not configured, skip this step.

- [ ] **Step 8: Run integration test - verify all log levels work**

Create temporary test:
```bash
cat > /tmp/test_logging.sh << 'EOF'
#!/bin/bash
cd mobile

echo "=== Test 1: Default (info level) ==="
RUST_LOG=info cargo run --bin dure-desktop -- --version 2>&1 | head -20

echo ""
echo "=== Test 2: Debug level ==="
RUST_LOG=debug cargo run --bin dure-desktop -- --version 2>&1 | head -20

echo ""
echo "=== Test 3: Warn level only ==="
RUST_LOG=warn cargo run --bin dure-desktop -- --version 2>&1 | head -20
EOF

chmod +x /tmp/test_logging.sh
/tmp/test_logging.sh
```

Expected:
- Test 1: info/warn/error messages appear
- Test 2: debug/info/warn/error messages appear
- Test 3: only warn/error messages appear
- All logs show tab-separated format with module prefixes

- [ ] **Step 9: Commit documentation and cleanup**

```bash
git add docs/GUIDELINES_RUST_CODING.md mobile/src/lib.rs
git status  # verify logging_test.rs is deleted
git commit -m "docs: add logging standards to coding guidelines

- Document dure_info!, dure_debug!, dure_warn!, dure_error! usage
- Add good/bad examples
- Explain log levels and RUST_LOG environment variable
- Remove temporary test file
- Clean up unused log imports

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

- [ ] **Step 10: Final verification checklist**

Verify all success criteria:
- [ ] All ~50 files migrated to new macros
- [ ] No remaining `log::info!` calls (except in logging.rs itself)
- [ ] `console_log!` macro removed from calc/db.rs
- [ ] Logger initialization updated in all entry points
- [ ] Desktop app runs with RUST_LOG=info
- [ ] Desktop app runs with RUST_LOG=debug
- [ ] Tab alignment looks correct in terminal
- [ ] Colors display correctly (if terminal supports it)
- [ ] Documentation added to GUIDELINES_RUST_CODING.md

- [ ] **Step 11: Create summary commit**

If all checks pass, create a summary:

```bash
git log --oneline --since="1 day ago"
```

Expected: 4 commits for this refactoring:
1. feat: add logging infrastructure
2. refactor: migrate calc and api layers
3. refactor: migrate application layer
4. docs: add logging standards

- [ ] **Step 12: Tag completion**

Optional: Tag this milestone:
```bash
git tag -a logging-refactor-v1 -m "Complete logging refactor with auto module prefixes"
```

## Plan Self-Review

**Spec coverage check:**
- ✅ Logging module created with macros (Task 1)
- ✅ Module name transformation implemented (Task 1)
- ✅ WASM compatibility via web_sys (Task 1)
- ✅ Logger initialization updated (Task 1)
- ✅ Business logic migrated (Task 2)
- ✅ Application layer migrated (Task 3)
- ✅ console_log! macro removed (Task 2, Step 1)
- ✅ Documentation added (Task 4)
- ✅ Testing on all platforms (Task 4, Steps 5-7)

**Placeholder scan:**
- ✅ No TBD/TODO markers
- ✅ All code examples complete
- ✅ All commands specify exact paths
- ✅ All test expectations specified

**Type consistency:**
- ✅ macro names consistent: `dure_info!`, `dure_debug!`, `dure_warn!`, `dure_error!`
- ✅ function signature: `pub fn module_name(path: &str) -> String`
- ✅ No type conflicts between tasks

**Scope check:**
- ✅ Focused on logging refactor only
- ✅ No feature additions beyond spec
- ✅ ~50 files is manageable for this plan
- ✅ Each task is independently testable
