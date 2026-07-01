# winhttpd-sys

Rust FFI bindings for [winhttpd](https://github.com/FmasterofU/winhttpd), a Windows port of darkhttpd static file server.

## Features

- ✅ Safe Rust API for serving static files on Windows
- ✅ Identical API to `darkhttpd-sys` for cross-platform code
- ✅ Minimal dependencies
- ✅ RAII cleanup (automatic stop on drop)
- ✅ Comprehensive test coverage

## Platform Support

**Windows only** - requires MSVC compiler.

For Unix-like systems (Linux, macOS, BSD), use `darkhttpd-sys` instead.

## Usage

Add to your `Cargo.toml`:

```toml
[target.'cfg(target_os = "windows")'.dependencies]
winhttpd-sys = { path = "../crates/winhttpd-sys" }
```

Basic usage:

```rust
use winhttpd_sys::WinHttpd;

let mut server = WinHttpd::new();
server.serve("./www", 8080)?;
// Server runs...
server.stop()?;
```

## Cross-Platform

For cross-platform code:

```rust
#[cfg(not(target_os = "windows"))]
use darkhttpd_sys::DarkHttpd as HttpServer;

#[cfg(target_os = "windows")]
use winhttpd_sys::WinHttpd as HttpServer;

let mut server = HttpServer::new();
```

## Build Requirements

- Windows OS
- MSVC compiler
- `cc` crate (automatic)
- `ws2_32.lib` (Windows sockets - automatic)

## License

Dual-licensed under MIT or Apache-2.0.

Based on winhttpd which is based on darkhttpd 1.12.
