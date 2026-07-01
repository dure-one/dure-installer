# winhttpd-sys: Windows HTTP Server FFI Crate

**Date:** 2026-06-30  
**Status:** Approved  
**Author:** Claude + User

## Overview

Create `winhttpd-sys` crate to provide Windows HTTP server functionality, mirroring the existing `darkhttpd-sys` crate for Unix-like systems. This enables cross-platform static file serving with identical APIs, specifically for OAuth callback server functionality.

## Background

- **Current state**: `darkhttpd-sys` provides static file serving on Unix-like systems (Linux, macOS, BSD)
- **Problem**: Windows builds fail due to Unix-specific headers (`sys/time.h`) in darkhttpd
- **Solution**: Create Windows-specific `winhttpd-sys` using winhttpd (MSVC-compatible darkhttpd port)
- **Use case**: Localhost HTTP server for OAuth callback handling in desktop application

## Goals

1. Provide identical API to `darkhttpd-sys` for cross-platform compatibility
2. Enable Windows builds to serve static files
3. Maintain clean platform separation
4. Follow Rust ecosystem conventions for platform-specific `-sys` crates
5. Comprehensive test coverage using TDD approach

## Non-Goals

- Supporting Unix-specific features (chroot, daemon, uid/gid, pidfile)
- Dynamic routing or server-side processing
- Production web server features
- MinGW or Cygwin compatibility (MSVC only)

## Architecture

### Crate Structure

```
winhttpd-sys/
├── Cargo.toml           # Crate manifest
├── build.rs             # Compiles winhttpd C source
├── src/
│   ├── lib.rs          # Public API and safe wrapper
│   └── ffi.rs          # Raw FFI bindings (unsafe)
├── winhttpd_lib.c      # Vendored winhttpd source (reorganized)
├── winhttpd_lib.h      # C header file
└── tests/
    ├── integration.rs   # Server lifecycle tests
    └── serve_files.rs   # File serving tests
```

### Layered Design

**Layer 1: FFI Bindings** (`ffi.rs`)
- Raw `unsafe` extern "C" bindings to winhttpd
- C-compatible types (`c_int`, `c_char`)
- Internal module (not public)

**Layer 2: Safe Wrapper** (`lib.rs`)
- Idiomatic Rust API
- State management (initialized, running)
- Error handling
- RAII cleanup (Drop implementation)

## Public API

### Main Types

```rust
pub struct WinHttpd {
    initialized: bool,
    running: bool,
}

impl WinHttpd {
    /// Create a new server instance
    pub fn new() -> Self;
    
    /// Start serving static files from a directory on the specified port
    pub fn serve(&mut self, wwwroot: &str, port: u16) -> Result<(), WinHttpdError>;
    
    /// Stop the server
    pub fn stop(&mut self) -> Result<(), WinHttpdError>;
}

impl Drop for WinHttpd {
    // Automatically stop server on cleanup
}
```

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum WinHttpdError {
    #[error("Failed to convert string to C string: {0}")]
    StringConversion(#[from] std::ffi::NulError),
    
    #[error("Initialization failed with code: {0}")]
    InitializationFailed(i32),
    
    #[error("Server is already initialized")]
    AlreadyInitialized,
    
    #[error("Server is not initialized")]
    NotInitialized,
}
```

**Design decisions:**
- Struct named `WinHttpd` (reflects underlying library)
- API matches `DarkHttpd` exactly for cross-platform compatibility
- Same error variants as `darkhttpd-sys`
- State tracked in struct fields for safety
- Drop ensures cleanup even on panic

## FFI Layer

### C Function Bindings

```rust
// ffi.rs
use std::os::raw::{c_char, c_int};

extern "C" {
    pub fn winhttpd_init() -> c_int;
    pub fn winhttpd_serve(wwwroot: *const c_char, port: c_int) -> c_int;
    pub fn winhttpd_stop() -> c_int;
    pub fn winhttpd_cleanup() -> c_int;
}
```

**Safety considerations:**
- All FFI functions marked `unsafe`
- Null pointer validation before FFI calls
- C string conversion with error handling
- Return code checking for all operations

**Note:** Exact function names will be determined after examining vendored winhttpd source. May require thin C wrapper layer (similar to `darkhttpd_lib.c`).

## Build Process

### Build Script

```rust
// build.rs
use std::env;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let winhttpd_c = manifest_dir.join("winhttpd_lib.c");
    
    println!("cargo:rerun-if-changed={}", winhttpd_c.display());
    
    // Compile winhttpd C source for MSVC
    cc::Build::new()
        .file(&winhttpd_c)
        .warnings(false)  // Suppress C code warnings
        .opt_level(2)
        .compile("winhttpd");
    
    println!("cargo:rustc-link-lib=static=winhttpd");
    
    // Link Windows socket library (required by winhttpd)
    println!("cargo:rustc-link-lib=ws2_32");
}
```

### Dependencies

```toml
[build-dependencies]
cc = "1.0"

[dependencies]
libc = "0.2"
thiserror = "2.0"

[dev-dependencies]
tempfile = "3.0"  # For test fixtures
ureq = "2.0"      # For HTTP requests in tests
```

**Build requirements:**
- MSVC compiler (default on Windows)
- `cc` crate handles MSVC compilation automatically
- Static linking for simple distribution
- `ws2_32.lib` for Windows sockets

## Source Integration

### Vendoring winhttpd

1. Download winhttpd source from https://github.com/FmasterofU/winhttpd
2. Extract necessary C files (winhttpd.c and dependencies)
3. Reorganize into `winhttpd_lib.c` and `winhttpd_lib.h`
4. Adapt as needed for clean FFI interface

**Key notes:**
- winhttpd is based on darkhttpd 1.12
- MSVC-specific (not MinGW/Cygwin)
- Missing Unix features: chroot, daemon, uid, gid, pidfile (not needed for our use case)
- See diff at winhttpd repo for changes from darkhttpd

**Reorganization approach:**
- Break original source path structure as needed
- Organize for clean FFI interface
- Similar to how `darkhttpd_lib.c` is structured
- Keep it simple - single C file if possible

## Testing Strategy (TDD)

### Unit Tests

Located in `src/lib.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_server() {
        let server = WinHttpd::new();
        assert!(!server.initialized);
        assert!(!server.running);
    }
    
    #[test]
    fn test_error_formatting() {
        let err = WinHttpdError::AlreadyInitialized;
        assert_eq!(err.to_string(), "Server is already initialized");
    }
    
    #[test]
    fn test_invalid_state_transitions() {
        // Test serving before init, double-stop, etc.
    }
}
```

### Integration Tests

**tests/integration.rs** - Server lifecycle:
```rust
#[test]
fn test_server_lifecycle() {
    let mut server = WinHttpd::new();
    assert!(server.serve("./test_www", 8080).is_ok());
    assert!(server.stop().is_ok());
}

#[test]
fn test_serve_invalid_path() {
    let mut server = WinHttpd::new();
    assert!(server.serve("/nonexistent/path", 8080).is_err());
}

#[test]
fn test_multiple_servers_different_ports() {
    // Test running multiple instances
}
```

**tests/serve_files.rs** - File serving:
```rust
use tempfile::TempDir;
use std::fs::write;

#[test]
fn test_serves_html_file() {
    // Create temp directory with test.html
    let tmp = TempDir::new().unwrap();
    let html_path = tmp.path().join("test.html");
    write(&html_path, "<h1>Test</h1>").unwrap();
    
    // Start server on random free port
    let port = find_free_port();
    let mut server = WinHttpd::new();
    server.serve(tmp.path().to_str().unwrap(), port).unwrap();
    
    // HTTP GET the file
    let response = ureq::get(&format!("http://localhost:{}/test.html", port))
        .call()
        .unwrap();
    
    // Verify content
    assert_eq!(response.status(), 200);
    assert!(response.into_string().unwrap().contains("<h1>Test</h1>"));
    
    // Cleanup
    server.stop().unwrap();
}

#[test]
fn test_serves_index_html() {
    // Test default index.html behavior
}

#[test]
fn test_404_for_missing_file() {
    // Verify 404 response
}
```

### Test Fixtures

```
tests/
├── fixtures/
│   └── test_www/
│       ├── index.html
│       ├── test.html
│       └── subdir/
│           └── nested.html
└── helpers.rs  # find_free_port(), etc.
```

### TDD Workflow

1. **Write failing test** - Define expected behavior
2. **Implement minimal FFI** - Get test compiling
3. **Make test pass** - Implement functionality
4. **Refactor** - Clean up while keeping tests green
5. **Repeat** - For each feature

**Test execution order:**
1. Unit tests (error types, state management)
2. FFI binding tests (calling C functions)
3. Integration tests (server lifecycle)
4. File serving tests (end-to-end)

## Integration with Mobile Crate

### Workspace Changes

Add to `Cargo.toml`:
```toml
[workspace]
members = [
    "mobile",
    "crates/darkhttpd-sys",
    "crates/winhttpd-sys",  # Add this
    "crates/go-webauthn-client",
]
```

### Mobile Crate Dependencies

Update `mobile/Cargo.toml`:
```toml
# Unix-like systems (Linux, macOS, BSD) - use darkhttpd
[target.'cfg(not(target_os = "windows"))'.dependencies]
darkhttpd-sys = { path = "../crates/darkhttpd-sys" }

# Windows - use winhttpd
[target.'cfg(target_os = "windows")'.dependencies]
winhttpd-sys = { path = "../crates/winhttpd-sys" }
```

### Usage in Mobile Code

```rust
// In mobile/src/calc/platform.rs or appropriate module

// Platform-specific imports with type aliasing
#[cfg(not(target_os = "windows"))]
use darkhttpd_sys::DarkHttpd as HttpServer;

#[cfg(target_os = "windows")]
use winhttpd_sys::WinHttpd as HttpServer;

// Use HttpServer transparently - no conditional compilation needed
fn start_oauth_callback_server(wwwroot: &str, port: u16) -> Result<()> {
    let mut server = HttpServer::new();
    server.serve(wwwroot, port)?;
    Ok(())
}

fn stop_oauth_callback_server(server: &mut HttpServer) -> Result<()> {
    server.stop()?;
    Ok(())
}
```

**Benefits:**
- Zero runtime overhead (compile-time selection)
- Same API on all platforms
- No conditional compilation in business logic
- Easy to test each platform independently
- Clear separation of platform concerns

## Error Handling

### Error Flow

1. **C level**: Return codes (0 = success, negative = error)
2. **FFI level**: Check return codes, convert to Result
3. **Wrapper level**: Convert to `WinHttpdError` variants
4. **Application level**: Handle errors appropriately

### Example Error Conversion

```rust
impl WinHttpd {
    pub fn serve(&mut self, wwwroot: &str, port: u16) -> Result<(), WinHttpdError> {
        if self.running {
            return Err(WinHttpdError::AlreadyInitialized);
        }
        
        // Convert Rust string to C string
        let c_wwwroot = CString::new(wwwroot)?;  // StringConversion error
        
        // Call FFI - unsafe block
        let result = unsafe {
            ffi::winhttpd_serve(c_wwwroot.as_ptr(), port as c_int)
        };
        
        // Check return code
        if result < 0 {
            return Err(WinHttpdError::InitializationFailed(result));
        }
        
        self.running = true;
        Ok(())
    }
}
```

## Implementation Plan

### Phase 1: Setup (TDD)
1. Create crate structure
2. Add to workspace
3. Write failing unit tests for error types
4. Implement error types
5. Verify tests pass

### Phase 2: FFI Layer (TDD)
1. Vendor winhttpd source
2. Write failing FFI binding tests
3. Create build.rs
4. Define FFI functions
5. Verify compilation and basic FFI calls

### Phase 3: Safe Wrapper (TDD)
1. Write failing tests for WinHttpd struct
2. Implement new(), serve(), stop()
3. Add state management
4. Implement Drop
5. Verify all tests pass

### Phase 4: Integration Tests (TDD)
1. Write failing integration tests
2. Implement test fixtures
3. Add helper functions (find_free_port)
4. Verify end-to-end functionality
5. Test actual file serving

### Phase 5: Mobile Integration
1. Update workspace Cargo.toml
2. Update mobile/Cargo.toml dependencies
3. Add platform-specific imports
4. Update OAuth callback code
5. Test on Windows

### Phase 6: Documentation
1. Add crate-level docs
2. Add API documentation
3. Add usage examples
4. Update CLAUDE.md if needed

## Success Criteria

- [ ] winhttpd-sys compiles on Windows (MSVC)
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Can serve static HTML files on localhost
- [ ] API matches darkhttpd-sys exactly
- [ ] Mobile crate compiles on Windows
- [ ] Mobile crate works on Windows (OAuth flow)
- [ ] No regression on Unix-like platforms
- [ ] Comprehensive documentation

## Risks and Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| winhttpd source incompatible with FFI | High | Review source early, create thin C wrapper if needed |
| API doesn't match darkhttpd-sys | Medium | Write cross-platform tests, verify identical behavior |
| Windows socket library issues | Medium | Test early, link ws2_32 correctly |
| Integration tests flaky (port conflicts) | Low | Use random free ports, retry logic |
| Build script complexity | Low | Follow darkhttpd-sys pattern closely |

## Future Enhancements

**Not in scope for initial implementation:**

- HTTPS support (TLS)
- Custom error pages
- Request logging
- Response headers customization
- Directory listing
- Content type detection beyond basics
- Performance optimizations
- Unix feature parity (daemon mode, etc.)

These can be added later if needed, but are not required for the OAuth callback use case.

## References

- winhttpd source: https://github.com/FmasterofU/winhttpd
- darkhttpd-sys crate: `crates/darkhttpd-sys`
- Rust FFI guide: https://doc.rust-lang.org/nomicon/ffi.html
- cc crate docs: https://docs.rs/cc/latest/cc/

## Appendix: winhttpd Notes

From winhttpd README:
- Based on darkhttpd 1.12 (newest)
- MSVC-compatible port of darkhttpd
- Not compatible with MinGW or Cygwin
- Missing options: `--chroot`, `--daemon`, `--uid`, `--gid`, `--pidfile`
- See full diff from darkhttpd at repository

These limitations are acceptable for our use case (localhost OAuth callbacks).
