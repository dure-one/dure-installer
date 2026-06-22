# Asupersync to Smol Runtime Migration Design

**Date:** 2026-06-22  
**Status:** Approved  
**Migration Type:** Big Bang (complete replacement with TDD)

## Context

### Problem Statement

The Dure e-commerce platform currently uses `asupersync` as its async runtime, which has three critical issues blocking development:

1. **Malformed test fixtures** - The asupersync dependency contains a malformed `Cargo.toml` in its test fixtures that prevents cargo from parsing the dependency tree
2. **OpenBSD incompatibility** - The `go-webauthn` crate (essential for WebAuthn support) depends on `mem-ring` → `monoio`, which doesn't support OpenBSD. This blocks development on the primary development platform.
3. **Build complexity** - `webauthn-rs` pulls in `webauthn-attestation-ca` which requires OpenSSL, adding native dependency management overhead

### Why This Change

- **Unblock development** - Fix the cargo parsing error preventing builds
- **Platform support** - Enable development on OpenBSD (primary dev environment)
- **Ecosystem alignment** - `smol` is a mature, well-maintained async runtime with better cross-platform support
- **Simplicity** - Smaller, cleaner runtime with less magic than asupersync
- **Dependencies** - Keep essential `go-webauthn` working while migrating runtime

### Success Criteria

**Primary:** All existing tests pass without modification after migration

**Secondary:**
- WebSocket client/server roundtrip works on all platforms
- TLS handshake succeeds with self-signed and ACME certificates  
- File I/O operations work correctly
- No performance regression >20%

---

## Architecture Overview

### Current State

```
dure (mobile/)
├── asupersync (runtime)
│   ├── Cx (runtime context)
│   ├── io::{AsyncReadExt, AsyncWriteExt}
│   ├── net::{TcpListener, TcpStream}
│   ├── tls::{TlsConnector, TlsAcceptor}
│   └── fs, time utilities
├── async-tungstenite::asupersync::AsupersyncAdapter
├── go-webauthn → mem-ring → monoio (BROKEN on OpenBSD)
└── webauthn-rs → webauthn-attestation-ca (needs OpenSSL)
```

### Target State

```
dure (mobile/)
├── smol (runtime)
│   ├── Executor
│   ├── block_on() for main
├── async-io (networking)
│   ├── Async<TcpListener>
│   ├── Async<TcpStream>
├── async-tls (TLS layer, rustls-based)
│   ├── TlsConnector
│   ├── TlsAcceptor
├── async-tungstenite (WebSocket, async-std feature)
├── go-webauthn → mem-ring → monoio (PATCHED for OpenBSD)
└── webauthn-rs (OpenSSL found via pkg-config)
```

### Migration Principles

1. **Zero behavioral changes** - Drop-in replacement, same functionality
2. **TDD approach** - Write tests before migration, verify after
3. **Big bang execution** - Replace all asupersync usage at once
4. **Foundation-first** - Fix monoio before migrating runtime
5. **Platform support** - Linux, macOS, Windows, OpenBSD, Android

---

## Phase 1: Monoio OpenBSD Patch

### Problem Analysis

The `monoio` crate has three categories of OpenBSD incompatibilities:

1. **I/O syscalls** - Uses Linux-specific `pread64`/`pwrite64`
   - OpenBSD uses `pread`/`pwrite` (no 64 suffix)
   - Location: `src/driver/op/read.rs`, `src/driver/op/write.rs`

2. **File metadata** - Uses Linux `statx` syscall
   - OpenBSD doesn't have `statx`, needs `fstat`/`stat` fallback
   - Location: `src/driver/op/statx.rs`, `src/fs/metadata/`

3. **Trait completeness** - Missing `legacy_call` implementations
   - Required for `OpAble` trait when `statx` unavailable
   - Location: `src/driver/op/statx.rs`

4. **TCP keepalive** - API incompatibilities with newer socket2 crate
   - `with_interval()` → `with_time()`
   - `with_retries()` method removed
   - Location: `src/net/tcp/stream.rs`

### Patch Implementation

#### 1. Fork and Branch Setup

```bash
# Fork monoio to your account
gh repo fork bytedance/monoio nikescar/monoio

# Clone and create patch branch
git clone git@github.com:nikescar/monoio.git
cd monoio
git checkout -b openbsd-support
```

#### 2. Apply Fixes

**File: `src/driver/op/read.rs`**
```rust
// Replace pread64 with pread on OpenBSD
#[cfg(target_os = "openbsd")]
return syscall_u32!(pread(
    self.fd,
    self.buf.write_ptr(),
    self.buf.bytes_total().min(u32::MAX as usize),
    self.offset as i64,
));

#[cfg(not(target_os = "openbsd"))]
return syscall_u32!(pread64(
    self.fd,
    self.buf.write_ptr(),
    self.buf.bytes_total().min(u32::MAX as usize),
    self.offset,
));
```

**File: `src/driver/op/write.rs`**
```rust
// Replace pwrite64 with pwrite on OpenBSD
#[cfg(target_os = "openbsd")]
return syscall_u32!(pwrite(
    self.fd,
    self.buf.read_ptr(),
    self.buf.bytes_init().min(u32::MAX as usize),
    self.offset as i64,
));

#[cfg(not(target_os = "openbsd"))]
return syscall_u32!(pwrite64(
    self.fd,
    self.buf.read_ptr(),
    self.buf.bytes_init().min(u32::MAX as usize),
    self.offset,
));
```

**File: `src/driver/op/statx.rs`**
```rust
impl OpAble for FdStatx {
    // ... existing methods ...
    
    fn legacy_call(&mut self) -> io::Result<u32> {
        #[cfg(target_os = "openbsd")]
        {
            use std::mem::MaybeUninit;
            let mut stat: MaybeUninit<libc::stat> = MaybeUninit::uninit();
            let ret = unsafe { libc::fstat(self.fd, stat.as_mut_ptr()) };
            if ret < 0 {
                return Err(io::Error::last_os_error());
            }
            let stat = unsafe { stat.assume_init() };
            // Convert libc::stat to statx format
            // ... conversion logic ...
            Ok(0)
        }
        #[cfg(not(target_os = "openbsd"))]
        {
            // Linux fallback
            Ok(0)
        }
    }
}

impl OpAble for PathStatx {
    // ... existing methods ...
    
    fn legacy_call(&mut self) -> io::Result<u32> {
        #[cfg(target_os = "openbsd")]
        {
            use std::ffi::CString;
            use std::mem::MaybeUninit;
            let path = CString::new(self.path.to_str().unwrap())?;
            let mut stat: MaybeUninit<libc::stat> = MaybeUninit::uninit();
            let ret = unsafe { libc::stat(path.as_ptr(), stat.as_mut_ptr()) };
            if ret < 0 {
                return Err(io::Error::last_os_error());
            }
            // Convert to statx format
            Ok(0)
        }
        #[cfg(not(target_os = "openbsd"))]
        {
            Ok(0)
        }
    }
}
```

**File: `src/fs/metadata/unix.rs`**
```rust
// Fix FileAttr field access - use accessor methods instead of direct field access
impl FileAttr {
    pub fn size(&self) -> u64 {
        self.stat.st_size as u64  // Keep this
    }
    
    pub fn perm(&self) -> FilePermissions {
        FilePermissions {
            mode: self.stat.st_mode as mode_t,  // Keep this
        }
    }
}
```

**File: `src/fs/metadata/mod.rs`**
```rust
// Fix missing 'op' variable - ensure statx operation is assigned
pub async fn metadata<P: AsRef<Path>>(path: P) -> io::Result<Metadata> {
    let op = Op::statx(path.as_ref())?;  // Add this line
    op.statx_result().await.map(FileAttr::from).map(Metadata)
}
```

**File: `src/net/tcp/stream.rs`**
```rust
// Fix TcpKeepalive API changes
let mut keepalive = TcpKeepalive::new().with_time(Duration::from_secs(time));
// Remove: .with_interval() and .with_retries() - no longer supported
```

#### 3. Cargo Patch Configuration

**File: `Cargo.toml` (workspace root)**
```toml
[patch.crates-io]
monoio = { git = "https://github.com/nikescar/monoio", branch = "openbsd-support" }
```

### Testing the Patch

```bash
# 1. Test monoio builds on OpenBSD
cd ~/.cargo/git/checkouts/monoio-*/
cargo build --all-features

# 2. Run monoio test suite
cargo test

# 3. Verify go-webauthn compiles
cd /home/wj/work/dure
cargo build -p go-webauthn

# 4. Verify full workspace builds
cargo build
```

### Success Criteria

- [ ] `cargo build` succeeds on OpenBSD without errors
- [ ] No monoio compilation warnings or errors
- [ ] `go-webauthn` crate compiles successfully
- [ ] Basic monoio functionality tests pass

---

## Phase 2: Smol Migration

### Dependency Changes

**Remove from `mobile/Cargo.toml` (desktop target):**

```toml
asupersync = { git = "https://github.com/Dicklesworthstone/asupersync", features = ["tls", "tls-native-roots"] }
async-tungstenite = { git = "https://github.com/nikescar/async-tungstenite", features = ["asupersync-runtime"] }
```

**Add to `mobile/Cargo.toml` (desktop target):**

```toml
[target.'cfg(not(any(target_os = "android", target_arch = "wasm32")))'.dependencies]
# Async runtime
smol = "2.0"
async-io = "2.3"
async-fs = "2.1"
futures-lite = "2.3"

# TLS (rustls-based)
async-tls = "0.13"
rustls = { version = "0.23", features = ["ring"] }
rustls-pemfile = "2"

# WebSocket (async-std compatible with smol)
async-tungstenite = { version = "0.28", features = ["async-std-runtime"] }
```

### API Migration Mapping

| asupersync | smol equivalent | Notes |
|------------|----------------|-------|
| `asupersync::Cx` | `smol::Executor` | Runtime context, usually not needed explicitly |
| `asupersync::net::TcpListener` | `async_io::Async<std::net::TcpListener>` | Wrap std types |
| `asupersync::net::TcpStream` | `async_io::Async<std::net::TcpStream>` | Wrap std types |
| `asupersync::tls::TlsConnector` | `async_tls::TlsConnector` | rustls-based |
| `asupersync::tls::TlsAcceptor` | `async_tls::TlsAcceptor` | rustls-based |
| `asupersync::io::AsyncReadExt` | `futures_lite::io::AsyncReadExt` | Same trait |
| `asupersync::io::AsyncWriteExt` | `futures_lite::io::AsyncWriteExt` | Same trait |
| `asupersync::time::sleep(cx, dur)` | `async_io::Timer::after(dur).await` | No context needed |
| `asupersync::fs` | `async_fs` | File I/O crate |
| `asupersync::runtime::RuntimeBuilder` | `smol::block_on()` | Simpler API |

### Code Pattern Changes

#### 1. Runtime Execution (main.rs, server binaries)

**Before:**
```rust
use asupersync::runtime::RuntimeBuilder;

fn main() {
    let rt = RuntimeBuilder::new().build().unwrap();
    rt.block_on(async_main());
}
```

**After:**
```rust
fn main() {
    smol::block_on(async_main());
}
```

#### 2. TCP Connection (client.rs)

**Before:**
```rust
use asupersync::{Cx, net::TcpStream, tls::TlsConnector};

async fn connect(cx: &Cx, addr: &str) -> io::Result<TlsStream<TcpStream>> {
    let stream = TcpStream::connect(addr).await?;
    let tls = TlsConnector::new(config);
    tls.connect(domain, stream).await
}
```

**After:**
```rust
use async_io::Async;
use async_tls::TlsConnector;
use std::net::TcpStream;

async fn connect(addr: &str) -> io::Result<async_tls::client::TlsStream<Async<TcpStream>>> {
    let stream = Async::<TcpStream>::connect(addr).await?;
    let connector = TlsConnector::new();
    connector.connect(domain, stream).await
}
```

#### 3. TCP Listener (server/mod.rs)

**Before:**
```rust
use asupersync::{Cx, net::TcpListener, tls::TlsAcceptor};

async fn serve(cx: &Cx, acceptor: TlsAcceptor) -> io::Result<()> {
    let listener = TcpListener::bind("0.0.0.0:443").await?;
    loop {
        let (stream, addr) = listener.accept().await?;
        let tls_stream = acceptor.accept(stream).await?;
        // handle connection
    }
}
```

**After:**
```rust
use async_io::Async;
use async_tls::TlsAcceptor;
use std::net::TcpListener;

async fn serve(acceptor: TlsAcceptor) -> io::Result<()> {
    let listener = Async::<TcpListener>::bind("0.0.0.0:443")?;
    loop {
        let (stream, addr) = listener.accept().await?;
        let tls_stream = acceptor.accept(stream).await?;
        // handle connection
    }
}
```

#### 4. WebSocket Integration (client.rs, server/ws.rs)

**Before:**
```rust
use async_tungstenite::asupersync::AsupersyncAdapter;
use asupersync::tls::TlsStream;

type WsStream = WebSocketStream<AsupersyncAdapter<TlsStream<TcpStream>>>;

async fn upgrade(stream: TlsStream<TcpStream>) -> Result<WsStream> {
    let ws = AsupersyncAdapter::new(stream);
    client_async(url, ws).await
}
```

**After:**
```rust
use async_tungstenite::{client_async, WebSocketStream};
use async_tls::client::TlsStream;

type WsStream = WebSocketStream<TlsStream<Async<TcpStream>>>;

async fn upgrade(stream: TlsStream<Async<TcpStream>>) -> Result<WsStream> {
    client_async(url, stream).await
}
```

#### 5. Async I/O Traits (https.rs, ws.rs)

**Before:**
```rust
use asupersync::io::{AsyncReadExt, AsyncWriteExt};

pub async fn read_request<S: AsyncReadExt + Unpin>(stream: &mut S) -> io::Result<Vec<u8>> {
    let mut buf = vec![0u8; 1024];
    stream.read_exact(&mut buf).await?;
    Ok(buf)
}
```

**After:**
```rust
use futures_lite::io::{AsyncReadExt, AsyncWriteExt};

pub async fn read_request<S: AsyncReadExt + Unpin>(stream: &mut S) -> io::Result<Vec<u8>> {
    let mut buf = vec![0u8; 1024];
    stream.read_exact(&mut buf).await?;
    Ok(buf)
}
```

#### 6. TLS Configuration (tls.rs)

**Before:**
```rust
use asupersync::tls::{TlsAcceptor, CertificateChain, PrivateKey};

fn create_acceptor(cert_path: &Path, key_path: &Path) -> io::Result<TlsAcceptor> {
    let cert = CertificateChain::from_pem_file(cert_path)?;
    let key = PrivateKey::from_pem_file(key_path)?;
    TlsAcceptor::new(cert, key)
}
```

**After:**
```rust
use async_tls::TlsAcceptor;
use rustls::{ServerConfig, pki_types::CertificateDer};
use rustls_pemfile::{certs, private_key};

fn create_acceptor(cert_path: &Path, key_path: &Path) -> io::Result<TlsAcceptor> {
    let cert_file = std::fs::File::open(cert_path)?;
    let key_file = std::fs::File::open(key_path)?;
    
    let certs: Vec<CertificateDer> = certs(&mut BufReader::new(cert_file))
        .collect::<Result<_, _>>()?;
    let key = private_key(&mut BufReader::new(key_file))?
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "No private key"))?;
    
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    
    Ok(TlsAcceptor::from(Arc::new(config)))
}
```

#### 7. File I/O (http_get.rs)

**Before:**
```rust
use asupersync::fs;

async fn read_file(path: &Path) -> io::Result<Vec<u8>> {
    fs::read(path).await
}
```

**After:**
```rust
use async_fs;

async fn read_file(path: &Path) -> io::Result<Vec<u8>> {
    async_fs::read(path).await
}
```

#### 8. Timers and Sleep (client.rs, server/mod.rs)

**Before:**
```rust
use asupersync::{Cx, time::sleep};

async fn periodic_ping(cx: &Cx) {
    loop {
        sleep(cx.now(), Duration::from_secs(30)).await;
        // send ping
    }
}
```

**After:**
```rust
use async_io::Timer;

async fn periodic_ping() {
    loop {
        Timer::after(Duration::from_secs(30)).await;
        // send ping
    }
}
```

### Files to Modify

| File | Changes Required |
|------|------------------|
| `mobile/Cargo.toml` | Update dependencies |
| `mobile/src/wss/client.rs` | Runtime, TLS, WebSocket |
| `mobile/src/wss/server/mod.rs` | Runtime, listener, acceptor |
| `mobile/src/wss/server/https.rs` | Async I/O traits |
| `mobile/src/wss/server/ws.rs` | WebSocket, async I/O |
| `mobile/src/wss/server/http_get.rs` | File I/O |
| `mobile/src/wss/server/http_post.rs` | Async I/O traits |
| `mobile/src/wss/server/tls.rs` | TLS configuration |
| `mobile/src/main.rs` (if exists) | Runtime execution |

### Migration Checklist

- [ ] Update `Cargo.toml` dependencies
- [ ] Replace runtime execution (`RuntimeBuilder` → `smol::block_on`)
- [ ] Replace network types (`TcpListener`, `TcpStream` → `Async<T>`)
- [ ] Replace TLS types (`TlsConnector`, `TlsAcceptor` → `async_tls`)
- [ ] Update WebSocket types (remove `AsupersyncAdapter`)
- [ ] Replace async I/O traits (`asupersync::io` → `futures_lite::io`)
- [ ] Replace file I/O (`asupersync::fs` → `async_fs`)
- [ ] Replace timers (`sleep(cx, dur)` → `Timer::after(dur)`)
- [ ] Remove `Cx` parameter from all async functions
- [ ] Update type aliases and documentation

---

## Testing Strategy (TDD)

### Unit Test Development

**Phase 1: Write Tests Before Migration**

Create test module structure:

```
mobile/src/wss/tests/
├── mod.rs              # Test module setup
├── async_io.rs         # TCP, TLS, I/O tests
├── websocket.rs        # WebSocket protocol tests
├── tls.rs              # Certificate handling tests
└── integration.rs      # Full stack tests
```

**1. Async I/O Tests** (`mobile/src/wss/tests/async_io.rs`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tcp_connect_loopback() {
        smol::block_on(async {
            // Connect to localhost
            let stream = Async::<TcpStream>::connect("127.0.0.1:80").await;
            assert!(stream.is_ok() || stream.err().unwrap().kind() == io::ErrorKind::ConnectionRefused);
        });
    }
    
    #[test]
    fn test_tls_handshake_self_signed() {
        smol::block_on(async {
            // Test TLS with self-signed certificate
            // ...
        });
    }
    
    #[test]
    fn test_async_read_write() {
        smol::block_on(async {
            // Test reading/writing data through stream
            // ...
        });
    }
}
```

**2. WebSocket Tests** (`mobile/src/wss/tests/websocket.rs`):

```rust
#[test]
fn test_ws_handshake_calculation() {
    let key = "dGhlIHNhbXBsZSBub25jZQ==";
    let accept = calculate_websocket_accept(key);
    assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
}

#[test]
fn test_ws_message_echo() {
    smol::block_on(async {
        // Start server, connect client, echo message
        // ...
    });
}

#[test]
fn test_ws_ping_pong() {
    smol::block_on(async {
        // Test ping/pong frame handling
        // ...
    });
}
```

**3. TLS Certificate Tests** (`mobile/src/wss/tests/tls.rs`):

```rust
#[test]
fn test_load_certificate_pem() {
    let cert_path = Path::new("test-data/cert.pem");
    let key_path = Path::new("test-data/key.pem");
    let acceptor = create_acceptor(cert_path, key_path);
    assert!(acceptor.is_ok());
}

#[test]
fn test_generate_self_signed() {
    let (cert, key) = generate_self_signed_cert("localhost");
    assert!(Path::new(&cert).exists());
    assert!(Path::new(&key).exists());
}
```

**4. Integration Tests** (`mobile/src/wss/tests/integration.rs`):

```rust
#[test]
fn test_client_server_roundtrip() {
    smol::block_on(async {
        // 1. Start HTTPS/WSS server in background
        let server_handle = smol::spawn(async {
            // Start server on port 8443
        });
        
        // 2. Connect client
        let mut client = connect_client("wss://localhost:8443").await.unwrap();
        
        // 3. Send message
        client.send(Message::Text("Hello".into())).await.unwrap();
        
        // 4. Receive echo
        let msg = client.next().await.unwrap().unwrap();
        assert_eq!(msg, Message::Text("Hello".into()));
        
        // 5. Clean shutdown
        client.close(None).await.unwrap();
        server_handle.cancel().await;
    });
}

#[test]
fn test_concurrent_connections() {
    smol::block_on(async {
        // Spawn 10 concurrent client connections
        let tasks: Vec<_> = (0..10).map(|i| {
            smol::spawn(async move {
                let client = connect_client("wss://localhost:8443").await.unwrap();
                // ...
            })
        }).collect();
        
        futures::future::join_all(tasks).await;
    });
}
```

### Test Execution Flow

**Step 1: Establish Baseline (Before Migration)**

```bash
# Write all tests against current asupersync code
# All tests should pass (GREEN)
cargo test --lib --bins
```

**Step 2: Migration (Tests will fail - RED)**

```bash
# Replace asupersync with smol
# Tests will fail during replacement
cargo test  # Expected to fail during migration
```

**Step 3: Fix Implementation (Back to GREEN)**

```bash
# Fix code until all tests pass
cargo test  # All tests pass = migration successful
```

**Step 4: Final Validation**

```bash
# Run full test suite
cargo test --all-features --all-targets

# Check clippy
cargo clippy --all-targets -- -D warnings

# Check formatting
cargo fmt --check

# Build all platforms (if possible)
cargo build --target x86_64-unknown-linux-gnu
cargo build --target x86_64-apple-darwin
cargo build --target x86_64-pc-windows-msvc
cargo build --target x86_64-unknown-openbsd  # Primary development platform
```

### Test Coverage Matrix

| Platform | Async Runtime | TLS | WebSocket | File I/O | Status |
|----------|--------------|-----|-----------|----------|--------|
| Linux    | smol         | async-tls | ✓ | async-fs | Primary |
| macOS    | smol         | async-tls | ✓ | async-fs | Primary |
| OpenBSD  | smol         | async-tls | ✓ | async-fs | Primary (dev) |
| Windows  | smol         | async-tls | ✓ | async-fs | Secondary |
| Android  | N/A          | N/A | N/A | N/A | Desktop-only |
| WASM     | N/A          | N/A | N/A | N/A | Desktop-only |

### Success Criteria Verification

**Primary Criterion:**

```bash
cargo test --lib --bins
# All tests pass without modification
```

**Secondary Criteria:**

```bash
# WebSocket client/server works
cargo run --bin wss-server -- --domain localhost &
cargo run --bin wss-client -- --url wss://localhost:8443 --mode ws
# ✓ Connection successful, messages exchanged

# TLS handshake works
cargo test test_tls_handshake
# ✓ TLS handshake with self-signed cert

# File I/O works
cargo test test_file_operations
# ✓ Read/write files asynchronously

# No performance regression
cargo bench
# ✓ Within 20% of baseline
```

---

## Error Handling & Rollback

### Potential Issues & Detection

**1. Monoio Patch Failures**

- **Symptom**: Build errors persist after patching, monoio tests fail
- **Detection**: `cargo build` fails on OpenBSD, `go-webauthn` doesn't compile
- **Root Causes**:
  - Incomplete syscall mapping
  - Incorrect `stat` to `statx` conversion
  - Missing platform-specific features
- **Fix**:
  - Review Linux monoio behavior, ensure OpenBSD matches
  - Add debug logging to syscalls
  - Compare with FreeBSD implementation (similar BSD)
- **Rollback**: `git revert` patch commit, disable `go-webauthn` temporarily

**2. Runtime Behavioral Differences**

- **Symptom**: Tests pass but WebSocket connections hang or fail
- **Detection**: Integration tests timeout, manual testing shows connection issues
- **Root Causes**:
  - Different executor polling behavior
  - Future waker implementation differences
  - Task scheduling differences
- **Fix**:
  - Add debug logging to async functions
  - Use `RUST_LOG=debug` to trace execution
  - Profile with `tokio-console` (if compatible) or print debugging
- **Rollback**: Full git revert to asupersync (clean history)

**3. TLS Handshake Issues**

- **Symptom**: Certificate validation fails, "bad certificate" errors
- **Detection**: TLS connection refused, handshake timeouts
- **Root Causes**:
  - Incorrect rustls configuration
  - Certificate format mismatch (DER vs PEM)
  - Missing certificate chain
- **Fix**:
  - Verify certificate loading with `rustls-pemfile`
  - Check certificate chain completeness
  - Compare with working asupersync TLS config
- **Rollback**: Restore asupersync TLS configuration only

**4. Performance Regression**

- **Symptom**: Slower WebSocket throughput, higher latency
- **Detection**: Benchmarks show >20% regression
- **Root Causes**:
  - Inefficient buffer management
  - Excessive context switching
  - Lock contention in smol executor
- **Fix**:
  - Profile with `cargo flamegraph`
  - Optimize hot paths identified by profiler
  - Adjust executor configuration
- **Rollback**: Not critical for success criteria (tests pass is enough)

**5. Platform-Specific Issues**

- **Symptom**: Works on Linux, fails on OpenBSD/macOS/Windows
- **Detection**: CI fails on specific platforms, local testing shows differences
- **Root Causes**:
  - Platform-specific async-io behavior
  - Different epoll/kqueue/IOCP implementations
  - File path or socket handling differences
- **Fix**:
  - Add platform-specific conditional compilation
  - Test on all platforms before merging
  - Use platform-agnostic APIs
- **Rollback**: Platform-specific revert, keep working platforms

### Rollback Strategy

**Git Workflow:**

```bash
# Create feature branch for migration
git checkout -b feat/migrate-to-smol

# Phase 1: Monoio patch (checkpoint 1)
git add Cargo.toml
git commit -m "feat: patch monoio for OpenBSD support"
git tag checkpoint-monoio-patch

# Phase 2: Dependency changes (checkpoint 2)
git add mobile/Cargo.toml
git commit -m "feat: update dependencies (asupersync → smol)"
git tag checkpoint-deps

# Phase 3: Code migration (checkpoint 3)
git add mobile/src/wss/
git commit -m "feat: migrate async runtime to smol"
git tag checkpoint-runtime

# Phase 4: Tests pass (checkpoint 4)
git commit -m "feat: all tests passing with smol"
git tag checkpoint-complete

# If rollback needed at any point:
git reset --hard checkpoint-deps  # Go back to specific checkpoint
# or
git revert HEAD~3..HEAD  # Revert last 3 commits
# or
git checkout main  # Abandon feature branch entirely
```

**Incremental Commits:**

1. **Monoio patch** - Can rollback just this if it fails
2. **Cargo.toml changes** - Can rollback dependencies only
3. **TLS migration** - Isolated, can revert just TLS
4. **WebSocket migration** - Isolated, can revert just WS
5. **Runtime execution** - Final integration
6. **Cleanup** - Remove dead code

**Validation Checkpoints:**

After each commit:

```bash
# Compilation check
cargo check --all-targets

# Test suite
cargo test

# Clippy warnings
cargo clippy --all-targets

# If any fail, fix before next commit
```

**Emergency Rollback Plan:**

If critical production issues found after merge:

1. **Keep asupersync branch alive for 1 week** after merge
2. **Monitor production deployments** for connection failures, panics
3. **Fast revert**: `git revert -m 1 <merge-commit>` reverts entire merge
4. **Deploy old branch** immediately if critical issue found
5. **Debug offline**, fix in new branch, re-merge when stable

### Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|-----------|
| Monoio patch fails | Medium | High | Test thoroughly on OpenBSD, compare with Linux behavior |
| Smol behavioral differences | Low | Medium | Comprehensive integration tests, manual testing |
| TLS handshake breaks | Low | High | Test with multiple certificate types, verify rustls config |
| Performance regression | Medium | Low | Benchmark before/after, profile if needed |
| Platform-specific failures | Medium | Medium | CI on all platforms, test locally on macOS/Windows/OpenBSD |
| Production downtime | Low | Critical | Keep rollback branch, monitor deployments, fast revert plan |

**Overall Risk Level: Low-Medium**

- Mature dependencies (smol is battle-tested)
- Comprehensive test suite
- Clear rollback plan
- Incremental git commits

### Post-Migration Monitoring

**Watch for these issues in production:**

1. **Memory leaks** - Long-running connections accumulating memory
   - Monitor: `htop`, `ps aux | grep dure`
   - Fix: Profile with `valgrind` or Rust memory profilers

2. **Connection timeouts** - WebSocket connections timing out unexpectedly
   - Monitor: Server logs, error rates
   - Fix: Adjust keepalive settings, check executor task scheduling

3. **Unexpected panics** - Async code panicking in edge cases
   - Monitor: Application logs, crash reports
   - Fix: Add error handling, graceful degradation

4. **Platform-specific bugs** - Issues only on OpenBSD or macOS
   - Monitor: Platform-specific error logs
   - Fix: Platform-specific patches, conditional compilation

**Monitoring Commands:**

```bash
# Memory usage over time
watch -n 5 'ps aux | grep wss-server | grep -v grep'

# Connection count
netstat -an | grep :443 | wc -l

# Application logs
tail -f /var/log/dure/wss-server.log

# Error rate (if logging to file)
grep ERROR /var/log/dure/wss-server.log | wc -l
```

---

## Implementation Timeline

### Phase 1: Monoio OpenBSD Patch (1-2 days)

**Day 1:**
- [ ] Fork monoio repository
- [ ] Create `openbsd-support` branch
- [ ] Apply syscall fixes (`pread`/`pwrite`)
- [ ] Apply `statx` fallback implementation
- [ ] Fix `TcpKeepalive` API usage
- [ ] Test compilation on OpenBSD

**Day 2:**
- [ ] Run monoio test suite
- [ ] Verify `go-webauthn` builds
- [ ] Test basic I/O operations
- [ ] Commit and push patch branch
- [ ] Update `Cargo.toml` with patch directive

### Phase 2: Smol Migration (1-2 days)

**Day 3:**
- [ ] Write unit tests for async I/O (TDD)
- [ ] Write WebSocket protocol tests
- [ ] Write TLS certificate tests
- [ ] Verify all tests pass with asupersync (baseline)
- [ ] Update `Cargo.toml` dependencies
- [ ] Replace runtime execution code

**Day 4:**
- [ ] Migrate TLS configuration
- [ ] Migrate WebSocket client/server
- [ ] Migrate file I/O operations
- [ ] Fix compilation errors
- [ ] Run test suite until all pass
- [ ] Manual testing on all platforms

### Phase 3: Validation (0.5-1 day)

**Day 5:**
- [ ] Full test suite on Linux
- [ ] Full test suite on macOS
- [ ] Full test suite on OpenBSD
- [ ] Full test suite on Windows (if possible)
- [ ] Performance benchmarks
- [ ] Code review and cleanup
- [ ] Merge to main

**Total Estimated Time: 3-5 days**

---

## Dependencies Reference

### Current Dependencies (to remove)

```toml
asupersync = { git = "https://github.com/Dicklesworthstone/asupersync", features = ["tls", "tls-native-roots"] }
async-tungstenite = { git = "https://github.com/nikescar/async-tungstenite", features = ["asupersync-runtime"] }
```

### New Dependencies (to add)

```toml
# Async runtime
smol = "2.0"
async-io = "2.3"
async-fs = "2.1"
futures-lite = "2.3"

# TLS (rustls-based)
async-tls = "0.13"
rustls = { version = "0.23", features = ["ring"] }
rustls-pemfile = "2"

# WebSocket (async-std compatible)
async-tungstenite = { version = "0.28", features = ["async-std-runtime"] }
```

### Keep Unchanged

```toml
# These remain the same
futures = "0.3"
rustls-native-certs = "0.8"
url = "2.5"

# go-webauthn (now works with patched monoio)
go-webauthn = { path = "../crates/go-webauthn" }

# webauthn-rs (OpenSSL via pkg-config)
webauthn-rs = { version = "0.5.0", features = ["danger-allow-state-serialisation"] }
```

---

## Appendix: Quick Reference

### Smol Runtime Cheat Sheet

```rust
// Blocking execution
smol::block_on(async { /* ... */ })

// Spawn task
let handle = smol::spawn(async { /* ... */ });

// Wait for task
handle.await;

// Spawn blocking task
smol::unblock(|| { /* blocking work */ }).await;

// Sleep
use async_io::Timer;
Timer::after(Duration::from_secs(1)).await;

// TCP listener
use async_io::Async;
let listener = Async::<TcpListener>::bind("0.0.0.0:8080")?;
let (stream, addr) = listener.accept().await?;

// TCP stream
let stream = Async::<TcpStream>::connect("example.com:80").await?;

// TLS connector
use async_tls::TlsConnector;
let connector = TlsConnector::new();
let tls_stream = connector.connect("example.com", stream).await?;

// TLS acceptor
use async_tls::TlsAcceptor;
let acceptor = TlsAcceptor::from(Arc::new(server_config));
let tls_stream = acceptor.accept(stream).await?;

// WebSocket client
use async_tungstenite::client_async;
let (ws, _) = client_async(url, tls_stream).await?;

// File I/O
use async_fs;
let contents = async_fs::read("file.txt").await?;
async_fs::write("file.txt", b"data").await?;
```

### Migration Command Reference

```bash
# Build on OpenBSD
cargo build

# Run tests
cargo test --lib --bins

# Run specific test
cargo test test_ws_handshake

# Run with debug logging
RUST_LOG=debug cargo test

# Build for release
cargo build --release

# Run WebSocket server
cargo run --bin wss-server -- --domain localhost

# Run WebSocket client
cargo run --bin wss-client -- --url wss://localhost:8443 --mode ws

# Check clippy
cargo clippy --all-targets

# Format code
cargo fmt

# Update dependencies
cargo update
```

---

**End of Design Document**
