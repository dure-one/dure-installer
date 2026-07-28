# Logging Refactor Design

**Date:** 2026-07-29  
**Author:** Woojae Park (nikescar@naver.com)  
**Status:** Design Approved

## Overview

Standardize logging across the Dure codebase with consistent, tab-separated format and automatic module prefixes. This design replaces ad-hoc logging patterns with a unified approach that works across desktop, Android, and WASM platforms.

## Goals

1. **Consistency:** All log messages follow the same format with module context
2. **Clarity:** Tab-separated columns for easy reading and parsing
3. **Debug Control:** Debug messages only appear when `RUST_LOG=debug` is set
4. **Platform Support:** Works seamlessly on desktop, Android, and WASM
5. **Maintainability:** Automatic module prefixes prevent inconsistencies

## Current State

**Problems identified:**
- Mixed usage of `log::debug!`, `log::info!`, `log::warn!`, `log::error!`
- Inconsistent message formatting (some descriptive, some terse)
- Custom `console_log!` macro in `calc/db.rs` for WASM compatibility
- Debug logs may appear regardless of RUST_LOG setting
- No module context in most log messages

**Example current logs:**
```
PlatformActor started
PlatformActor command failed: connection timeout
Fetching images from project: my-project
```

## Desired State

**Standardized format:**
```
2026-07-29 14:23:10	[INFO]	[Platform::Actor]	Starting actor loop
2026-07-29 14:23:11	[DEBUG]	[GCP::OAuth]	Refreshing access token for client: 1234567890...
2026-07-29 14:23:12	[INFO]	[GCP::Compute]	Created VM: my-vm
```

**Tab-separated columns:**
1. Timestamp
2. Log level
3. Module/component
4. Message

## Architecture

### Core Components

1. **New logging module** (`mobile/src/logging.rs`)
   - Custom macros: `dure_info!`, `dure_debug!`, `dure_warn!`, `dure_error!`
   - Automatic module path injection using `module_path!()`
   - WASM compatibility layer

2. **env_logger configuration** (in `main.rs`, `main_android.rs`)
   - Tab-separated format: `timestamp\t[LEVEL]\t[module]\tmessage`
   - Respect `RUST_LOG` environment variable (info by default, debug when set)

3. **Module name mapping**
   - Transform Rust module paths into clean component names
   - `dure::viewmodel::platform::actor` → `[Platform::Actor]`
   - `dure::api::gcp::oauth` → `[GCP::OAuth]`

4. **Migration approach**
   - Project-wide find-replace of `log::info!` → `dure_info!`, etc.
   - Update imports from `log` to `crate::logging`
   - Remove custom `console_log!` macro in favor of unified approach

## Module Naming Conventions

### Mapping Strategy

The macro will automatically transform Rust module paths into clean, hierarchical component names:

| Rust Module Path | Component Name | Category |
|-----------------|----------------|----------|
| `dure::viewmodel::platform::actor` | `[Platform::Actor]` | Actor pattern files |
| `dure::api::gcp::oauth` | `[GCP::OAuth]` | GCP services |
| `dure::api::gcp::compute` | `[GCP::Compute]` | GCP services |
| `dure::calc::db` | `[DB]` | Core business logic |
| `dure::calc::ssh` | `[SSH]` | Core business logic |
| `dure::ui_tabs::ssh` | `[UI::SSH]` | UI components |
| `dure::ui_tabs::platform` | `[UI::Platform]` | UI components |
| `dure::wss::server` | `[WSS::Server]` | WebSocket server |
| `dure::wss::client` | `[WSS::Client]` | WebSocket client |
| `dure::android::activity` | `[Android]` | Platform-specific |
| `dure::attestation::verify` | `[Attestation]` | Security components |

### Naming Rules

1. Strip `dure::` prefix (implied)
2. Top-level modules become single brackets: `calc::db` → `[DB]`
3. Nested modules use `::` separator: `api::gcp::oauth` → `[GCP::OAuth]`
4. UI modules get `UI::` prefix: `ui_tabs::ssh` → `[UI::SSH]`
5. Capitalize appropriately: `gcp` → `GCP`, `wss` → `WSS`

### Usage Example

Developers simply write:
```rust
dure_info!("Starting actor loop");
```

The macro automatically becomes:
```
[Platform::Actor]\tStarting actor loop
```

## Implementation Details

### Macro Definitions

Create four primary macros in `mobile/src/logging.rs`:

```rust
/// Info-level logging with automatic module prefix
#[macro_export]
macro_rules! dure_info {
    ($($arg:tt)*) => {
        log::info!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*))
    };
}

/// Debug-level logging with automatic module prefix (only shows when RUST_LOG=debug)
#[macro_export]
macro_rules! dure_debug {
    ($($arg:tt)*) => {
        log::debug!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*))
    };
}

/// Warning-level logging with automatic module prefix
#[macro_export]
macro_rules! dure_warn {
    ($($arg:tt)*) => {
        log::warn!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*))
    };
}

/// Error-level logging with automatic module prefix
#[macro_export]
macro_rules! dure_error {
    ($($arg:tt)*) => {
        log::error!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*))
    };
}
```

### Module Name Transformer

```rust
/// Transform Rust module path into clean component name
///
/// Examples:
/// - "dure::api::gcp::oauth" -> "GCP::OAuth"
/// - "dure::calc::db" -> "DB"
/// - "dure::ui_tabs::ssh" -> "UI::SSH"
pub fn module_name(path: &str) -> String {
    // Implementation will handle:
    // 1. Strip "dure::" prefix
    // 2. Map known patterns (api::gcp -> GCP, calc -> component name)
    // 3. Handle ui_tabs specially (-> UI::)
    // 4. Capitalize acronyms (gcp -> GCP, wss -> WSS)
    // 5. Fallback to last 2 components if unknown
    
    // Detailed implementation will be in the implementation plan
}
```

### WASM Compatibility

For WASM builds, override macros to use `console.log`:

```rust
#[cfg(target_family = "wasm")]
#[macro_export]
macro_rules! dure_info {
    ($($arg:tt)*) => {
        web_sys::console::log_1(
            &format!("[{}]\t{}", $crate::logging::module_name(module_path!()), format!($($arg)*)).into()
        )
    };
}

// Similar overrides for dure_debug!, dure_warn!, dure_error!
```

This replaces the current `console_log!` macro in `calc/db.rs`.

## Logger Initialization

### Desktop (`main.rs`)

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

### Android (`main_android.rs`)

```rust
fn init_logger() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Debug)
            .with_tag("Dure")
    );
}
```

Android logcat adds its own timestamp, so we keep the format simple.

### WASM (`main_wasm.rs`)

No logger initialization needed - `web_sys::console::log` handles it directly.

### Behavior

- **Default:** `RUST_LOG=info` (shows info, warn, error)
- **Debug mode:** `RUST_LOG=debug` (shows all including debug)
- **Custom:** Users can override with `RUST_LOG=warn` (only warnings and errors)

## Migration Strategy

### Phase 1: Add New Logging Module

1. Create `mobile/src/logging.rs` with:
   - Macro definitions (`dure_info!`, `dure_debug!`, etc.)
   - `module_name()` function
   - WASM-specific overrides

2. Add to `mobile/src/lib.rs`:
   ```rust
   pub mod logging;
   ```

3. Update logger initialization in:
   - `mobile/src/main.rs`
   - `mobile/src/main_android.rs`
   - `mobile/src/main_wasm.rs` (if needed)

### Phase 2: Migrate Existing Log Calls

**Project-wide replacements:**
```
log::info!     → dure_info!
log::debug!    → dure_debug!
log::warn!     → dure_warn!
log::error!    → dure_error!
```

**Update imports:**
```rust
// OLD
use log::{info, debug, warn, error};

// NEW
// No import needed - macros are globally available via #[macro_export]
// Or explicitly:
use crate::{dure_info, dure_debug, dure_warn, dure_error};
```

### Phase 3: Clean Up Legacy Code

1. Remove `console_log!` macro from `calc/db.rs` (lines 36-43)
2. Replace its usage with `dure_info!`
3. Remove unused `use log::{...}` imports

### Files Affected (Estimated ~50 files)

- `mobile/src/android/*.rs` (5 files)
- `mobile/src/api/gcp/*.rs` (8 files)
- `mobile/src/api/*.rs` (3 files)
- `mobile/src/calc/*.rs` (15 files)
- `mobile/src/viewmodel/**/*.rs` (10 files)
- `mobile/src/ui_tabs/*.rs` (visible tabs only)
- `mobile/src/wss/**/*.rs` (WebSocket server/client)
- `mobile/src/attestation/*.rs` (2 files)
- `mobile/src/*.rs` (main files: main.rs, lib.rs, dure.rs, etc.)

## Additional Considerations

### Performance

- **Minimal overhead:** Module name transformation happens once per log call
- **Lazy evaluation:** `format!()` only evaluated when log level is enabled
- **No runtime complexity:** Simple string manipulation, no regex

### Edge Cases

1. **Very long module paths:**
   - Max component name length: ~20 chars
   - Example: `dure::viewmodel::platform::actor::commands` → `[Platform::Actor]`
   - Truncate at meaningful boundary

2. **Test modules:**
   - `dure::calc::db::tests` → `[DB::Tests]`
   - Keep test context visible in logs

3. **Generic/anonymous modules:**
   - Fallback to last 2 path components if transformation unclear
   - `dure::unknown::module` → `[Unknown::Module]`

### Documentation

Add to `docs/GUIDELINES_RUST_CODING.md`:

```markdown
## Logging Standards

Use Dure logging macros with descriptive messages:

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

❌ Wrong macro (bypasses module prefix):
```rust
log::info!("Starting OAuth flow");  // DON'T USE log:: directly
```

❌ Too terse (unclear what's happening):
```rust
dure_info!("OAuth");
dure_debug!("Fetch");
```

❌ Too verbose (belongs in debug, not info):
```rust
dure_info!("Parsed Docker image: {}:{}, with {} env vars and {} port mappings", 
           image, tag, env_count, port_count);
// Better as dure_debug!
```

### Guidelines

- **info**: Important events users/operators should see
- **debug**: Detailed troubleshooting information
- **warn**: Recoverable issues or unexpected conditions
- **error**: Failures that prevent operations from completing
```

### Future Extensibility

The macro design allows easy future additions:

1. **Structured fields:**
   ```rust
   dure_info!("User logged in", user_id = 123, session_id = "abc");
   ```

2. **Trace level:**
   ```rust
   dure_trace!("Entering function with args: {:?}", args);
   ```

3. **Custom formatters per module:**
   ```rust
   #[cfg(feature = "json-logging")]
   // Output JSON format instead of tab-separated
   ```

4. **Conditional compilation:**
   ```rust
   #[cfg(feature = "verbose-logging")]
   // Include additional context automatically
   ```

## Testing Plan

### Test Scenarios

1. **Desktop info logging:**
   - Run app with default `RUST_LOG` (or `RUST_LOG=info`)
   - Verify info messages appear with correct format
   - Verify debug messages do NOT appear

2. **Desktop debug logging:**
   - Run app with `RUST_LOG=debug`
   - Verify both info and debug messages appear
   - Verify tab-separated format is correct

3. **Android logging:**
   - Run app on Android device
   - Check logcat output: `adb logcat | grep Dure`
   - Verify module prefixes appear correctly

4. **WASM logging:**
   - Build WASM version
   - Open browser console
   - Verify `console.log` shows module prefixes
   - Verify tab-separated format

5. **Color output:**
   - Verify color coding in terminal:
     - ERROR: Red + bold
     - WARN: Yellow + bold
     - INFO: Green
     - DEBUG: Blue

### Validation Checklist

- [ ] All ~50 files migrated to new macros
- [ ] No remaining `log::info!` calls (except in logging.rs)
- [ ] `console_log!` macro removed from calc/db.rs
- [ ] Logger initialization updated in all entry points
- [ ] Desktop app runs with RUST_LOG=info
- [ ] Desktop app runs with RUST_LOG=debug
- [ ] Android app logs correctly to logcat
- [ ] WASM app logs correctly to browser console
- [ ] Tab alignment looks correct in terminal
- [ ] Colors display correctly (if terminal supports it)

## Success Criteria

1. ✅ **Consistency:** All log messages follow tab-separated format with module prefix
2. ✅ **Debug control:** Debug messages only appear when `RUST_LOG=debug` is set
3. ✅ **Platform support:** Works on desktop, Android, and WASM without platform-specific code in call sites
4. ✅ **Maintainability:** Developers can't forget module prefix (automatic)
5. ✅ **Readability:** Logs are easy to scan, filter, and parse

## Non-Goals

- **Structured logging framework:** Not replacing with `tracing` crate (too large a change)
- **Log aggregation:** Not implementing centralized log collection (future consideration)
- **Log rotation:** Not handling log file management (OS/deployment concern)
- **Performance profiling:** Not adding timing/profiling data to logs (separate tool)

## References

- Current logging patterns: `mobile/src/viewmodel/platform/actor.rs:24,30,34`
- Current console_log macro: `mobile/src/calc/db.rs:36-43`
- Rust log crate: https://docs.rs/log/
- env_logger crate: https://docs.rs/env_logger/
- Android logger: https://docs.rs/android_logger/
