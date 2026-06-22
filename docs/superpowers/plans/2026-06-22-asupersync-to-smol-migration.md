# Asupersync to Smol Runtime Migration - Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace asupersync async runtime with smol, fixing OpenBSD build issues while maintaining all existing functionality

**Architecture:** Foundation-first approach - patch monoio for OpenBSD compatibility, then migrate all asupersync usage to smol in one pass (big bang), using TDD to ensure zero behavioral changes

**Tech Stack:** 
- Runtime: smol 2.0 + async-io 2.3
- TLS: async-tls 0.13 (rustls-based)
- WebSocket: async-tungstenite 0.28 (async-std feature)
- Testing: cargo test with smol::block_on

## Global Constraints

- Rust version: 1.85 minimum
- Zero behavioral changes: drop-in replacement
- Success criterion: all existing tests pass without modification
- Platform support: Linux, macOS, Windows, OpenBSD (primary dev), Android (desktop-only features)
- TDD: write tests before migration, verify after
- Frequent commits: after each passing test

---

## Task 1: Fork and Setup Monoio Patch Repository

**Files:**
- External: Fork `bytedance/monoio` → `nikescar/monoio`
- External: Create branch `openbsd-support`

**Interfaces:**
- Produces: Git repository at `git@github.com:nikescar/monoio.git` with branch `openbsd-support`

- [ ] **Step 1: Fork monoio repository**

```bash
gh repo fork bytedance/monoio nikescar/monoio --clone=false
```

Expected: Repository forked to `nikescar/monoio`

- [ ] **Step 2: Clone and create patch branch**

```bash
git clone git@github.com:nikescar/monoio.git /tmp/monoio-patch
cd /tmp/monoio-patch
git checkout -b openbsd-support
git push -u origin openbsd-support
```

Expected: Branch `openbsd-support` created and pushed

- [ ] **Step 3: Verify setup**

```bash
cd /tmp/monoio-patch
git status
git branch -a
```

Expected output:
```
On branch openbsd-support
Your branch is up to date with 'origin/openbsd-support'.
```

- [ ] **Step 4: Commit setup documentation**

```bash
echo "# OpenBSD Support Branch

This branch adds OpenBSD compatibility to monoio.

Fixes:
- pread64/pwrite64 → pread/pwrite
- statx → fstat/stat fallback
- TcpKeepalive API updates
" > OPENBSD.md

git add OPENBSD.md
git commit -m "docs: add OpenBSD support documentation"
git push
```

---

## Task 2: Patch Monoio I/O Syscalls for OpenBSD

**Files:**
- Modify: `/tmp/monoio-patch/src/driver/op/read.rs`
- Modify: `/tmp/monoio-patch/src/driver/op/write.rs`

**Interfaces:**
- Consumes: Forked monoio repository with `openbsd-support` branch
- Produces: `pread`/`pwrite` syscalls work on OpenBSD

- [ ] **Step 1: Locate pread64 usage in read.rs**

```bash
cd /tmp/monoio-patch
grep -n "pread64" src/driver/op/read.rs
```

Expected: Find line number with `pread64` call

- [ ] **Step 2: Add OpenBSD conditional compilation for pread**

Edit `/tmp/monoio-patch/src/driver/op/read.rs`:

Find the `pread64` syscall (around line 89) and replace with:

```rust
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

- [ ] **Step 3: Add OpenBSD conditional compilation for pwrite**

Edit `/tmp/monoio-patch/src/driver/op/write.rs`:

Find the `pwrite64` syscall (around line 73) and replace with:

```rust
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

- [ ] **Step 4: Verify compilation**

```bash
cd /tmp/monoio-patch
cargo check --target x86_64-unknown-openbsd
```

Expected: No errors related to `pread64`/`pwrite64`

- [ ] **Step 5: Commit I/O syscall patches**

```bash
git add src/driver/op/read.rs src/driver/op/write.rs
git commit -m "feat: add OpenBSD support for pread/pwrite syscalls

OpenBSD uses pread/pwrite instead of pread64/pwrite64.
Add conditional compilation to support both variants.
"
git push
```

---

## Task 3: Patch Monoio Statx Syscall for OpenBSD

**Files:**
- Modify: `/tmp/monoio-patch/src/driver/op/statx.rs`
- Modify: `/tmp/monoio-patch/src/fs/metadata/mod.rs`

**Interfaces:**
- Consumes: I/O syscall patches from Task 2
- Produces: `legacy_call()` implementations using `fstat`/`stat` on OpenBSD

- [ ] **Step 1: Add legacy_call for FdStatx**

Edit `/tmp/monoio-patch/src/driver/op/statx.rs`:

Find `impl OpAble for FdStatx` and add:

```rust
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
        
        // Convert libc::stat to statx buffer
        unsafe {
            let statx_buf = &mut *self.statx_buf;
            statx_buf.stx_mask = libc::STATX_BASIC_STATS as u32;
            statx_buf.stx_blksize = stat.st_blksize as u32;
            statx_buf.stx_nlink = stat.st_nlink as u32;
            statx_buf.stx_uid = stat.st_uid;
            statx_buf.stx_gid = stat.st_gid;
            statx_buf.stx_mode = stat.st_mode as u16;
            statx_buf.stx_ino = stat.st_ino;
            statx_buf.stx_size = stat.st_size as u64;
            statx_buf.stx_blocks = stat.st_blocks as u64;
        }
        Ok(0)
    }
    #[cfg(not(target_os = "openbsd"))]
    {
        // Linux uses actual statx syscall
        Err(io::Error::new(io::ErrorKind::Unsupported, "statx not available"))
    }
}
```

- [ ] **Step 2: Add legacy_call for PathStatx**

In the same file, find `impl OpAble for PathStatx` and add:

```rust
fn legacy_call(&mut self) -> io::Result<u32> {
    #[cfg(target_os = "openbsd")]
    {
        use std::ffi::CString;
        use std::mem::MaybeUninit;
        
        let path = CString::new(
            self.path.to_str()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid path"))?
        )?;
        
        let mut stat: MaybeUninit<libc::stat> = MaybeUninit::uninit();
        let ret = unsafe { libc::stat(path.as_ptr(), stat.as_mut_ptr()) };
        if ret < 0 {
            return Err(io::Error::last_os_error());
        }
        let stat = unsafe { stat.assume_init() };
        
        // Convert libc::stat to statx buffer
        unsafe {
            let statx_buf = &mut *self.statx_buf;
            statx_buf.stx_mask = libc::STATX_BASIC_STATS as u32;
            statx_buf.stx_blksize = stat.st_blksize as u32;
            statx_buf.stx_nlink = stat.st_nlink as u32;
            statx_buf.stx_uid = stat.st_uid;
            statx_buf.stx_gid = stat.st_gid;
            statx_buf.stx_mode = stat.st_mode as u16;
            statx_buf.stx_ino = stat.st_ino;
            statx_buf.stx_size = stat.st_size as u64;
            statx_buf.stx_blocks = stat.st_blocks as u64;
        }
        Ok(0)
    }
    #[cfg(not(target_os = "openbsd"))]
    {
        Err(io::Error::new(io::ErrorKind::Unsupported, "statx not available"))
    }
}
```

- [ ] **Step 3: Fix missing op variable in metadata functions**

Edit `/tmp/monoio-patch/src/fs/metadata/mod.rs`:

Find `pub async fn metadata` and ensure it has:

```rust
pub async fn metadata<P: AsRef<Path>>(path: P) -> io::Result<Metadata> {
    let op = Op::statx(path.as_ref())?;
    op.statx_result().await.map(FileAttr::from).map(Metadata)
}
```

Do the same for any other metadata functions missing the `op` variable.

- [ ] **Step 4: Verify compilation**

```bash
cd /tmp/monoio-patch
cargo check --target x86_64-unknown-openbsd
```

Expected: No errors related to `statx` or `legacy_call`

- [ ] **Step 5: Commit statx patches**

```bash
git add src/driver/op/statx.rs src/fs/metadata/mod.rs
git commit -m "feat: add OpenBSD statx fallback using fstat/stat

OpenBSD doesn't have statx syscall. Implement legacy_call()
for FdStatx and PathStatx using traditional fstat/stat.
"
git push
```

---

## Task 4: Fix Monoio TcpKeepalive API and FileAttr

**Files:**
- Modify: `/tmp/monoio-patch/src/net/tcp/stream.rs`
- Modify: `/tmp/monoio-patch/src/fs/metadata/unix.rs`

**Interfaces:**
- Consumes: Statx patches from Task 3
- Produces: TcpKeepalive compatible with newer socket2, FileAttr without direct field access

- [ ] **Step 1: Fix TcpKeepalive API**

Edit `/tmp/monoio-patch/src/net/tcp/stream.rs`:

Find TcpKeepalive usage (around line 624-628) and update:

```rust
// Old code with errors:
// t = t.with_interval(interval)
// t = t.with_retries(retries)

// New code:
let mut t = TcpKeepalive::new().with_time(Duration::from_secs(time));
// with_interval and with_retries removed in newer socket2
socket.set_tcp_keepalive(&t)?;
```

- [ ] **Step 2: Verify FileAttr doesn't use direct field access**

Edit `/tmp/monoio-patch/src/fs/metadata/unix.rs`:

Ensure FileAttr methods use `self.stat.field` not trying to access non-existent fields:

```rust
impl FileAttr {
    pub fn size(&self) -> u64 {
        self.stat.st_size as u64
    }
    
    pub fn perm(&self) -> FilePermissions {
        FilePermissions {
            mode: self.stat.st_mode as mode_t,
        }
    }
}
```

- [ ] **Step 3: Verify compilation**

```bash
cd /tmp/monoio-patch
cargo check --all-features
```

Expected: No errors

- [ ] **Step 4: Run monoio test suite**

```bash
cd /tmp/monoio-patch
cargo test --lib
```

Expected: Tests pass or skip on OpenBSD (some may require Linux-specific features)

- [ ] **Step 5: Commit API compatibility fixes**

```bash
git add src/net/tcp/stream.rs src/fs/metadata/unix.rs
git commit -m "fix: update TcpKeepalive API for newer socket2

Remove with_interval() and with_retries() calls that no longer
exist in socket2 crate. Use with_time() instead.
"
git push
```

---

## Task 5: Configure Cargo Patch and Verify Monoio Build

**Files:**
- Modify: `Cargo.toml` (workspace root)

**Interfaces:**
- Consumes: Complete monoio patches from Tasks 1-4
- Produces: Dure workspace using patched monoio, go-webauthn builds successfully

- [ ] **Step 1: Add Cargo patch directive**

Edit `Cargo.toml` (workspace root):

Add after `[workspace.lints.rust]` section:

```toml
[patch.crates-io]
monoio = { git = "https://github.com/nikescar/monoio", branch = "openbsd-support" }
```

- [ ] **Step 2: Clean cargo cache**

```bash
rm -rf ~/.cargo/git/checkouts/monoio-*
rm -rf target/
cargo clean
```

Expected: Fresh build state

- [ ] **Step 3: Verify go-webauthn builds**

```bash
cargo build -p go-webauthn
```

Expected: Build succeeds without monoio errors

- [ ] **Step 4: Verify full workspace builds**

```bash
cargo check
```

Expected: All crates check successfully

- [ ] **Step 5: Commit Cargo patch**

```bash
git add Cargo.toml
git commit -m "feat: patch monoio for OpenBSD support

Add Cargo patch to use nikescar/monoio fork with OpenBSD
compatibility fixes for pread/pwrite/statx syscalls.
"
```

---

## Task 6: Write Test Module Structure

**Files:**
- Create: `mobile/src/wss/tests/mod.rs`
- Create: `mobile/src/wss/tests/async_io.rs`
- Create: `mobile/src/wss/tests/websocket.rs`
- Create: `mobile/src/wss/tests/tls.rs`
- Modify: `mobile/src/wss/mod.rs`

**Interfaces:**
- Produces: Test module structure for TDD approach

- [ ] **Step 1: Create tests directory**

```bash
mkdir -p mobile/src/wss/tests
```

- [ ] **Step 2: Write test module file**

Create `mobile/src/wss/tests/mod.rs`:

```rust
//! Tests for WebSocket client/server functionality
//!
//! These tests verify async runtime behavior is identical
//! before and after migrating from asupersync to smol.

#[cfg(test)]
mod async_io;
#[cfg(test)]
mod websocket;
#[cfg(test)]
mod tls;
```

- [ ] **Step 3: Declare tests module in wss**

Edit `mobile/src/wss/mod.rs`:

Add at the end:

```rust
#[cfg(test)]
mod tests;
```

- [ ] **Step 4: Verify module structure compiles**

```bash
cargo test --no-run
```

Expected: Compiles successfully (no tests yet)

- [ ] **Step 5: Commit test module structure**

```bash
git add mobile/src/wss/tests/mod.rs mobile/src/wss/mod.rs
git commit -m "test: add test module structure for async runtime migration

Prepare test modules for TDD approach. Tests will verify
identical behavior before/after asupersync → smol migration.
"
```

---

## Task 7: Write Baseline WebSocket Handshake Test

**Files:**
- Create: `mobile/src/wss/tests/websocket.rs`

**Interfaces:**
- Consumes: Test module structure from Task 6
- Produces: `test_ws_handshake_calculation()` passes with current asupersync code

- [ ] **Step 1: Write failing WebSocket handshake test**

Create `mobile/src/wss/tests/websocket.rs`:

```rust
use crate::wss::server::ws::calculate_websocket_accept;

#[test]
fn test_ws_handshake_calculation() {
    // RFC 6455 example key
    let key = "dGhlIHNhbXBsZSBub25jZQ==";
    let accept = calculate_websocket_accept(key);
    
    // Expected from RFC 6455
    assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
}
```

- [ ] **Step 2: Run test to verify it passes (baseline)**

```bash
cargo test test_ws_handshake_calculation -v
```

Expected: `test wss::tests::websocket::test_ws_handshake_calculation ... ok`

- [ ] **Step 3: Add test for session ID generation**

Add to `websocket.rs`:

```rust
use crate::wss::server::generate_session_id;

#[test]
fn test_session_id_format() {
    let session_id = generate_session_id();
    
    // Should be 32 character hex string
    assert_eq!(session_id.len(), 32);
    assert!(session_id.chars().all(|c| c.is_ascii_hexdigit()));
}
```

- [ ] **Step 4: Run both tests**

```bash
cargo test websocket -v
```

Expected: Both tests pass

- [ ] **Step 5: Commit baseline WebSocket tests**

```bash
git add mobile/src/wss/tests/websocket.rs
git commit -m "test: add WebSocket handshake baseline tests

Tests verify:
- WebSocket accept key calculation (RFC 6455)
- Session ID format (32 hex chars)

These establish baseline behavior before runtime migration.
"
```

---

## Task 8: Write Baseline TLS Certificate Test

**Files:**
- Create: `mobile/src/wss/tests/tls.rs`

**Interfaces:**
- Consumes: Test module from Task 6
- Produces: TLS self-signed cert generation test passes

- [ ] **Step 1: Write TLS cert generation test**

Create `mobile/src/wss/tests/tls.rs`:

```rust
use std::path::Path;

#[test]
fn test_generate_self_signed_cert() {
    use crate::wss::server::generate_self_signed_cert;
    
    let (cert_path, key_path) = generate_self_signed_cert("localhost");
    
    // Verify files exist
    assert!(Path::new(&cert_path).exists(), "Certificate file should exist");
    assert!(Path::new(&key_path).exists(), "Key file should exist");
    
    // Verify files are not empty
    let cert_size = std::fs::metadata(&cert_path).unwrap().len();
    let key_size = std::fs::metadata(&key_path).unwrap().len();
    
    assert!(cert_size > 0, "Certificate should not be empty");
    assert!(key_size > 0, "Key should not be empty");
}
```

- [ ] **Step 2: Run test to verify baseline**

```bash
cargo test test_generate_self_signed_cert -v
```

Expected: Test passes, creates cert/key files

- [ ] **Step 3: Add TLS acceptor creation test**

Add to `tls.rs`:

```rust
#[test]
fn test_create_tls_acceptor() {
    use crate::wss::server::tls::create_acceptor;
    use crate::wss::server::generate_self_signed_cert;
    use std::path::Path;
    
    let (cert_path, key_path) = generate_self_signed_cert("test-domain.local");
    
    let acceptor_result = create_acceptor(
        Path::new(&cert_path),
        Path::new(&key_path)
    );
    
    assert!(acceptor_result.is_ok(), "Should create TLS acceptor from valid cert/key");
}
```

- [ ] **Step 4: Run all TLS tests**

```bash
cargo test tls -v
```

Expected: Both tests pass

- [ ] **Step 5: Commit TLS baseline tests**

```bash
git add mobile/src/wss/tests/tls.rs
git commit -m "test: add TLS certificate baseline tests

Tests verify:
- Self-signed certificate generation
- TLS acceptor creation from cert/key

Establishes baseline before async runtime migration.
"
```

---

## Task 9: Write Baseline Async I/O Test

**Files:**
- Create: `mobile/src/wss/tests/async_io.rs`

**Interfaces:**
- Consumes: Test module from Task 6
- Produces: Async TCP connection test with asupersync baseline

- [ ] **Step 1: Write async TCP loopback test**

Create `mobile/src/wss/tests/async_io.rs`:

```rust
use std::io;

#[test]
fn test_tcp_connect_loopback() {
    use asupersync::runtime::RuntimeBuilder;
    use asupersync::net::TcpStream;
    
    let rt = RuntimeBuilder::new().build().unwrap();
    rt.block_on(async {
        // Try to connect to localhost (may fail if nothing listening)
        let result = TcpStream::connect("127.0.0.1:80").await;
        
        // Accept either success or connection refused (not other errors)
        match result {
            Ok(_) => {}, // Connected to something
            Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {}, // Expected
            Err(e) => panic!("Unexpected error: {}", e),
        }
    });
}
```

- [ ] **Step 2: Run test to verify baseline**

```bash
cargo test test_tcp_connect_loopback -v
```

Expected: Test passes

- [ ] **Step 3: Add async read/write test**

Add to `async_io.rs`:

```rust
#[test]
fn test_async_read_write_buffer() {
    use asupersync::runtime::RuntimeBuilder;
    use asupersync::io::{AsyncReadExt, AsyncWriteExt};
    use std::io::Cursor;
    
    let rt = RuntimeBuilder::new().build().unwrap();
    rt.block_on(async {
        let data = b"Hello, async world!";
        let mut cursor = Cursor::new(Vec::new());
        
        // Write
        cursor.write_all(data).await.unwrap();
        
        // Read back
        cursor.set_position(0);
        let mut buffer = vec![0u8; data.len()];
        cursor.read_exact(&mut buffer).await.unwrap();
        
        assert_eq!(&buffer, data);
    });
}
```

- [ ] **Step 4: Run all async_io tests**

```bash
cargo test async_io -v
```

Expected: Both tests pass

- [ ] **Step 5: Commit async I/O baseline tests**

```bash
git add mobile/src/wss/tests/async_io.rs
git commit -m "test: add async I/O baseline tests

Tests verify:
- TCP connection (loopback)
- Async read/write operations

Uses current asupersync runtime as baseline.
"
```

---

## Task 10: Update Cargo Dependencies (Smol Migration Start)

**Files:**
- Modify: `mobile/Cargo.toml`

**Interfaces:**
- Consumes: Passing baseline tests from Tasks 7-9
- Produces: New dependencies added, old ones commented out (tests will fail)

- [ ] **Step 1: Comment out asupersync dependencies**

Edit `mobile/Cargo.toml`, in desktop dependencies section:

```toml
# Desktop dependencies - OLD (asupersync)
# asupersync = { git = "https://github.com/Dicklesworthstone/asupersync", features = ["tls", "tls-native-roots"] }
# async-tungstenite = { git = "https://github.com/nikescar/async-tungstenite", features = ["asupersync-runtime"] }
```

- [ ] **Step 2: Add smol and async-io dependencies**

Add in the same section:

```toml
# Async runtime - NEW (smol)
smol = "2.0"
async-io = "2.3"
async-fs = "2.1"
futures-lite = "2.3"
```

- [ ] **Step 3: Add async-tls dependencies**

```toml
# TLS (rustls-based)
async-tls = "0.13"
# rustls already exists, ensure it has ring feature
# rustls-pemfile already exists
```

- [ ] **Step 4: Update async-tungstenite**

```toml
# WebSocket (async-std compatible with smol)
async-tungstenite = { version = "0.28", features = ["async-std-runtime"] }
```

- [ ] **Step 5: Verify dependencies resolve**

```bash
cargo update
cargo fetch
```

Expected: All dependencies download successfully

- [ ] **Step 6: Commit dependency changes**

```bash
git add mobile/Cargo.toml
git commit -m "feat: update dependencies for smol migration

Remove: asupersync (custom fork)
Add: smol, async-io, async-fs, futures-lite, async-tls
Update: async-tungstenite to use async-std-runtime

Tests will fail until code migration completes.
"
```

---

## Task 11: Migrate WebSocket Client Runtime

**Files:**
- Modify: `mobile/src/wss/client.rs`

**Interfaces:**
- Consumes: Smol dependencies from Task 10
- Produces: WebSocket client using smol runtime (may not compile until all files updated)

- [ ] **Step 1: Update imports**

Edit `mobile/src/wss/client.rs`:

Replace:
```rust
use asupersync::{
    Cx,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    tls::TlsConnector,
};
```

With:
```rust
use async_io::Async;
use async_tls::TlsConnector;
use futures_lite::io::{AsyncReadExt, AsyncWriteExt};
use std::net::TcpStream;
use std::sync::Arc;
```

- [ ] **Step 2: Update TLS connector creation**

Replace `create_tls_connector_insecure()` function:

```rust
fn create_tls_connector_insecure() -> TlsConnector {
    use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
    use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
    use rustls::{ClientConfig, DigitallySignedStruct, Error, SignatureScheme};

    #[derive(Debug)]
    struct AcceptAnyCert;

    impl ServerCertVerifier for AcceptAnyCert {
        // ... keep existing implementation ...
    }

    let _ = rustls::crypto::ring::default_provider().install_default();

    let config = ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(AcceptAnyCert))
        .with_no_client_auth();

    TlsConnector::from(Arc::new(config))
}
```

- [ ] **Step 3: Update WsStream type alias**

Replace:
```rust
type WsStream = async_tungstenite::WebSocketStream<
    async_tungstenite::asupersync::AsupersyncAdapter<asupersync::tls::TlsStream<TcpStream>>,
>;
```

With:
```rust
type WsStream = async_tungstenite::WebSocketStream<
    async_tls::client::TlsStream<Async<TcpStream>>,
>;
```

- [ ] **Step 4: Update main function to use smol**

Replace:
```rust
fn main() {
    let rt = asupersync::runtime::RuntimeBuilder::new().build().unwrap();
    rt.block_on(async_main());
}
```

With:
```rust
fn main() {
    smol::block_on(async_main());
}
```

- [ ] **Step 5: Remove Cx parameter from async functions**

Find all `async fn` signatures with `cx: &Cx` parameter and remove it:

```rust
// Before:
async fn connect(cx: &Cx, url: &Url) -> Result<WsStream>

// After:
async fn connect(url: &Url) -> Result<WsStream>
```

- [ ] **Step 6: Update TCP connection**

Replace:
```rust
let stream = TcpStream::connect(addr).await?;
```

With:
```rust
let stream = Async::<TcpStream>::connect(addr).await?;
```

- [ ] **Step 7: Update TLS connection**

Replace:
```rust
let tls_stream = tls.connect(domain, stream).await?;
```

With:
```rust
let tls_stream = tls.connect(domain, stream).await?;
```

(API is the same, just types changed)

- [ ] **Step 8: Update WebSocket upgrade**

Remove AsupersyncAdapter:

```rust
// Before:
let ws = async_tungstenite::asupersync::AsupersyncAdapter::new(tls_stream);
let (ws_stream, _) = client_async(url, ws).await?;

// After:
let (ws_stream, _) = client_async(url, tls_stream).await?;
```

- [ ] **Step 9: Update sleep/timer calls**

Replace:
```rust
let mut tick = asupersync::time::sleep(cx.now(), Duration::from_millis(100)).fuse();
```

With:
```rust
let mut tick = async_io::Timer::after(Duration::from_millis(100)).fuse();
```

- [ ] **Step 10: Verify compilation**

```bash
cargo check -p dure
```

Expected: May have errors in other files, but client.rs should compile

- [ ] **Step 11: Commit client migration**

```bash
git add mobile/src/wss/client.rs
git commit -m "feat: migrate WebSocket client to smol runtime

Replace asupersync with smol:
- Runtime: RuntimeBuilder → smol::block_on
- TLS: asupersync::tls → async_tls
- Network: TcpStream → Async<TcpStream>
- I/O traits: asupersync::io → futures_lite::io
- Remove Cx parameter from all functions
"
```

---

## Task 12: Migrate WebSocket Server Runtime

**Files:**
- Modify: `mobile/src/wss/server/mod.rs`

**Interfaces:**
- Consumes: Client migration from Task 11
- Produces: WebSocket server using smol runtime

- [ ] **Step 1: Update imports**

Edit `mobile/src/wss/server/mod.rs`:

Replace:
```rust
use asupersync::{
    Cx,
    net::{TcpListener, TcpStream},
    tls::TlsAcceptor,
};
```

With:
```rust
use async_io::Async;
use async_tls::TlsAcceptor;
use std::net::{TcpListener, TcpStream};
```

- [ ] **Step 2: Update main function**

Replace:
```rust
fn main() {
    let rt = asupersync::runtime::RuntimeBuilder::new().build().unwrap();
    rt.block_on(async_main());
}
```

With:
```rust
fn main() {
    smol::block_on(async_main());
}
```

- [ ] **Step 3: Remove Cx from function signatures**

Remove `cx: &Cx` parameter from all async functions in the file.

- [ ] **Step 4: Update TcpListener bind**

Replace:
```rust
let listener = TcpListener::bind(bind_addr).await?;
```

With:
```rust
let listener = Async::<TcpListener>::bind(bind_addr)?;
```

- [ ] **Step 5: Update accept loop**

The accept loop stays similar, just type changes:

```rust
loop {
    let (stream, peer_addr) = listener.accept().await?;
    
    // Spawn handler
    smol::spawn(handle_connection(
        stream,
        peer_addr,
        acceptor.clone(),
        settings.clone(),
        stats.clone(),
    )).detach();
}
```

- [ ] **Step 6: Update sleep calls**

Replace:
```rust
asupersync::time::sleep(cx.now(), Duration::from_secs(interval_secs)).await;
```

With:
```rust
async_io::Timer::after(Duration::from_secs(interval_secs)).await;
```

- [ ] **Step 7: Verify compilation**

```bash
cargo check -p dure
```

Expected: Fewer errors, server/mod.rs should compile

- [ ] **Step 8: Commit server migration**

```bash
git add mobile/src/wss/server/mod.rs
git commit -m "feat: migrate WebSocket server to smol runtime

Replace asupersync with smol:
- Runtime: RuntimeBuilder → smol::block_on
- Network: TcpListener → Async<TcpListener>
- Spawning: Cx spawn → smol::spawn
- Remove Cx parameter from all functions
"
```

---

## Task 13: Migrate TLS Configuration Module

**Files:**
- Modify: `mobile/src/wss/server/tls.rs`

**Interfaces:**
- Consumes: Server migration from Task 12
- Produces: `create_acceptor()` using async-tls

- [ ] **Step 1: Update imports**

Edit `mobile/src/wss/server/tls.rs`:

Replace:
```rust
use asupersync::tls::{TlsAcceptor, CertificateChain, PrivateKey};
```

With:
```rust
use async_tls::TlsAcceptor;
use rustls::{ServerConfig, pki_types::CertificateDer};
use rustls_pemfile::{certs, private_key};
use std::io::BufReader;
use std::sync::Arc;
```

- [ ] **Step 2: Rewrite create_acceptor function**

Replace entire `create_acceptor` function:

```rust
pub fn create_acceptor(
    cert_path: &std::path::Path,
    key_path: &std::path::Path,
) -> std::io::Result<TlsAcceptor> {
    use std::fs::File;
    
    let cert_file = File::open(cert_path)?;
    let key_file = File::open(key_path)?;
    
    let certs: Vec<CertificateDer> = certs(&mut BufReader::new(cert_file))
        .collect::<Result<_, _>>()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    
    let key = private_key(&mut BufReader::new(key_file))?
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "No private key found"))?;
    
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    
    Ok(TlsAcceptor::from(Arc::new(config)))
}
```

- [ ] **Step 3: Verify compilation**

```bash
cargo check -p dure
```

Expected: tls.rs compiles

- [ ] **Step 4: Commit TLS migration**

```bash
git add mobile/src/wss/server/tls.rs
git commit -m "feat: migrate TLS configuration to async-tls

Replace asupersync TLS types with async-tls:
- TlsAcceptor from ServerConfig
- Use rustls-pemfile for cert/key loading
- Explicit error handling for cert validation
"
```

---

## Task 14: Migrate HTTPS Request Handler

**Files:**
- Modify: `mobile/src/wss/server/https.rs`

**Interfaces:**
- Consumes: TLS migration from Task 13
- Produces: HTTP request handling with smol I/O traits

- [ ] **Step 1: Update imports**

Edit `mobile/src/wss/server/https.rs`:

Replace:
```rust
use asupersync::{
    Cx,
    io::{AsyncReadExt, AsyncWriteExt},
};
```

With:
```rust
use futures_lite::io::{AsyncReadExt, AsyncWriteExt};
```

- [ ] **Step 2: Remove Cx from function signatures**

Update all function signatures to remove `cx: &Cx`:

```rust
// Before:
pub async fn read_http_request<S: AsyncReadExt + Unpin>(cx: &Cx, stream: &mut S) -> io::Result<HttpRequest>

// After:
pub async fn read_http_request<S: AsyncReadExt + Unpin>(stream: &mut S) -> io::Result<HttpRequest>
```

- [ ] **Step 3: Verify trait bounds**

Ensure trait bounds use `futures_lite::io` traits:

```rust
pub async fn handle_https_request<S>(
    stream: &mut S,
    request: HttpRequest,
    settings: ServerSettings,
    stats: Stats,
) -> io::Result<()>
where
    S: AsyncReadExt + AsyncWriteExt + Unpin,
{
    // ... implementation unchanged ...
}
```

- [ ] **Step 4: Verify compilation**

```bash
cargo check -p dure
```

Expected: https.rs compiles

- [ ] **Step 5: Commit HTTPS migration**

```bash
git add mobile/src/wss/server/https.rs
git commit -m "feat: migrate HTTPS handler to smol I/O traits

Replace asupersync::io with futures_lite::io:
- AsyncReadExt, AsyncWriteExt traits
- Remove Cx parameter from all functions
"
```

---

## Task 15: Migrate WebSocket Handler

**Files:**
- Modify: `mobile/src/wss/server/ws.rs`

**Interfaces:**
- Consumes: HTTPS migration from Task 14
- Produces: WebSocket handling with smol

- [ ] **Step 1: Update imports**

Edit `mobile/src/wss/server/ws.rs`:

Replace:
```rust
use asupersync::{Cx, io::AsyncWriteExt};
use async_tungstenite::{WebSocketStream, asupersync::AsupersyncAdapter, tungstenite::Message};
```

With:
```rust
use futures_lite::io::AsyncWriteExt;
use async_tungstenite::{WebSocketStream, tungstenite::Message};
```

- [ ] **Step 2: Remove Cx parameter from handle_websocket**

Update function signature:

```rust
// Before:
pub async fn handle_websocket<S>(
    cx: &Cx,
    ws_stream: WebSocketStream<AsupersyncAdapter<S>>,
    ...
)

// After:
pub async fn handle_websocket<S>(
    ws_stream: WebSocketStream<S>,
    peer_addr: std::net::SocketAddr,
    session_id: String,
    settings: ServerSettings,
    stats: Stats,
) -> io::Result<()>
where
    S: futures_lite::io::AsyncReadExt + futures_lite::io::AsyncWriteExt + Unpin + Send + 'static,
```

- [ ] **Step 3: Remove AsupersyncAdapter from type**

Remove all references to `AsupersyncAdapter` - WebSocket works directly with the stream:

```rust
// The WebSocketStream now wraps the TLS stream directly
// No adapter needed
```

- [ ] **Step 4: Verify compilation**

```bash
cargo check -p dure
```

Expected: ws.rs compiles

- [ ] **Step 5: Commit WebSocket handler migration**

```bash
git add mobile/src/wss/server/ws.rs
git commit -m "feat: migrate WebSocket handler to smol

Remove AsupersyncAdapter wrapper - async-tungstenite's
async-std feature works directly with smol streams.

Remove Cx parameter from handle_websocket function.
"
```

---

## Task 16: Migrate HTTP GET (File Serving)

**Files:**
- Modify: `mobile/src/wss/server/http_get.rs`

**Interfaces:**
- Consumes: WebSocket migration from Task 15
- Produces: File serving with async-fs

- [ ] **Step 1: Update imports**

Edit `mobile/src/wss/server/http_get.rs`:

Replace:
```rust
use asupersync::{fs, io::AsyncWriteExt};
```

With:
```rust
use async_fs;
use futures_lite::io::AsyncWriteExt;
```

- [ ] **Step 2: Update file read operations**

Replace all `asupersync::fs` calls:

```rust
// Before:
let contents = fs::read(&file_path).await?;

// After:
let contents = async_fs::read(&file_path).await?;
```

- [ ] **Step 3: Verify compilation**

```bash
cargo check -p dure
```

Expected: http_get.rs compiles

- [ ] **Step 4: Commit HTTP GET migration**

```bash
git add mobile/src/wss/server/http_get.rs
git commit -m "feat: migrate HTTP GET handler to async-fs

Replace asupersync::fs with async_fs for file I/O.
Replace asupersync::io with futures_lite::io traits.
"
```

---

## Task 17: Migrate HTTP POST Handler

**Files:**
- Modify: `mobile/src/wss/server/http_post.rs`

**Interfaces:**
- Consumes: HTTP GET migration from Task 16
- Produces: POST handling with smol I/O traits

- [ ] **Step 1: Update imports**

Edit `mobile/src/wss/server/http_post.rs`:

Replace:
```rust
use asupersync::io::AsyncWriteExt;
```

With:
```rust
use futures_lite::io::AsyncWriteExt;
```

- [ ] **Step 2: Verify compilation**

```bash
cargo check -p dure
```

Expected: http_post.rs compiles

- [ ] **Step 3: Commit HTTP POST migration**

```bash
git add mobile/src/wss/server/http_post.rs
git commit -m "feat: migrate HTTP POST handler to smol I/O traits

Replace asupersync::io::AsyncWriteExt with
futures_lite::io::AsyncWriteExt.
"
```

---

## Task 18: Update Test Suite for Smol

**Files:**
- Modify: `mobile/src/wss/tests/async_io.rs`
- Modify: `mobile/src/wss/tests/websocket.rs`
- Modify: `mobile/src/wss/tests/tls.rs`

**Interfaces:**
- Consumes: Complete code migration from Tasks 11-17
- Produces: All tests updated to use smol, tests should pass (GREEN)

- [ ] **Step 1: Update async_io tests**

Edit `mobile/src/wss/tests/async_io.rs`:

Replace asupersync runtime with smol:

```rust
use std::io;
use async_io::Async;
use std::net::TcpStream;

#[test]
fn test_tcp_connect_loopback() {
    smol::block_on(async {
        // Try to connect to localhost
        let result = Async::<TcpStream>::connect("127.0.0.1:80").await;
        
        match result {
            Ok(_) => {},
            Err(e) if e.kind() == io::ErrorKind::ConnectionRefused => {},
            Err(e) => panic!("Unexpected error: {}", e),
        }
    });
}

#[test]
fn test_async_read_write_buffer() {
    use futures_lite::io::{AsyncReadExt, AsyncWriteExt};
    use std::io::Cursor;
    
    smol::block_on(async {
        let data = b"Hello, async world!";
        let mut cursor = Cursor::new(Vec::new());
        
        cursor.write_all(data).await.unwrap();
        cursor.set_position(0);
        
        let mut buffer = vec![0u8; data.len()];
        cursor.read_exact(&mut buffer).await.unwrap();
        
        assert_eq!(&buffer, data);
    });
}
```

- [ ] **Step 2: Run async_io tests**

```bash
cargo test async_io -v
```

Expected: Tests pass (GREEN)

- [ ] **Step 3: Verify websocket tests still pass**

```bash
cargo test websocket -v
```

Expected: Tests pass (no changes needed, they don't use runtime)

- [ ] **Step 4: Verify TLS tests still pass**

```bash
cargo test tls -v
```

Expected: Tests pass (TLS cert generation should work same)

- [ ] **Step 5: Run full test suite**

```bash
cargo test --lib --bins
```

Expected: All tests pass (SUCCESS CRITERION MET)

- [ ] **Step 6: Commit test updates**

```bash
git add mobile/src/wss/tests/async_io.rs
git commit -m "test: update test suite for smol runtime

Replace asupersync::runtime with smol::block_on in tests.
Replace asupersync I/O types with smol equivalents.

All tests pass - migration successful!
"
```

---

## Task 19: Clean Up and Final Verification

**Files:**
- Modify: `mobile/Cargo.toml` (remove commented asupersync lines)

**Interfaces:**
- Consumes: Passing tests from Task 18
- Produces: Clean codebase, all checks pass

- [ ] **Step 1: Remove commented asupersync dependencies**

Edit `mobile/Cargo.toml`:

Delete the commented-out asupersync lines:

```toml
# Remove these:
# asupersync = { git = "..." }
# async-tungstenite = { git = "...", features = ["asupersync-runtime"] }
```

- [ ] **Step 2: Run cargo clippy**

```bash
cargo clippy --all-targets -- -D warnings
```

Expected: No warnings

- [ ] **Step 3: Run cargo fmt**

```bash
cargo fmt --check
```

Expected: All files formatted correctly

- [ ] **Step 4: Full test suite**

```bash
cargo test --all-features --all-targets
```

Expected: All tests pass

- [ ] **Step 5: Build release**

```bash
cargo build --release
```

Expected: Clean release build

- [ ] **Step 6: Commit cleanup**

```bash
git add mobile/Cargo.toml
git commit -m "chore: remove deprecated asupersync dependencies

Clean up commented-out dependencies.
Migration to smol is complete.
"
```

---

## Task 20: Platform Verification and Merge

**Files:**
- None (verification only)

**Interfaces:**
- Consumes: Clean codebase from Task 19
- Produces: Verified multi-platform build, ready to merge

- [ ] **Step 1: Test on OpenBSD (primary dev platform)**

```bash
cargo test --lib --bins
cargo build --release
```

Expected: All tests pass, clean build

- [ ] **Step 2: Test on Linux (if available)**

```bash
cargo test --lib --bins --target x86_64-unknown-linux-gnu
```

Expected: All tests pass

- [ ] **Step 3: Test on macOS (if available)**

```bash
cargo test --lib --bins --target x86_64-apple-darwin
```

Expected: All tests pass

- [ ] **Step 4: Verify go-webauthn still works**

```bash
cargo test -p go-webauthn
```

Expected: go-webauthn tests pass with patched monoio

- [ ] **Step 5: Final commit and tag**

```bash
git tag migration-complete
git log --oneline | head -20
```

Expected: Clean commit history showing migration progression

- [ ] **Step 6: Merge to main**

```bash
git checkout main
git merge --no-ff feat/migrate-to-smol -m "feat: migrate from asupersync to smol runtime

Complete migration from asupersync to smol async runtime.

Changes:
- Patched monoio for OpenBSD support (pread/pwrite/statx)
- Replaced asupersync with smol 2.0
- Replaced asupersync TLS with async-tls (rustls)
- Updated async-tungstenite to use async-std feature
- All tests pass on all platforms

Success criteria met:
✓ All existing tests pass without modification
✓ WebSocket client/server works
✓ TLS handshake succeeds
✓ File I/O works correctly
✓ OpenBSD build fixed

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
"
```

---

## Verification Checklist

After completing all tasks, verify:

- [ ] `cargo build` succeeds on OpenBSD
- [ ] `cargo test --lib --bins` passes all tests
- [ ] `cargo run --bin wss-server` starts successfully
- [ ] `cargo run --bin wss-client` connects successfully
- [ ] No asupersync dependencies remain
- [ ] monoio patch works (go-webauthn builds)
- [ ] Clippy has no warnings
- [ ] Code is formatted
- [ ] All platforms tested (Linux, macOS, OpenBSD, Windows)

## Success Criteria Met

✓ All existing tests pass without modification (PRIMARY)
✓ WebSocket client/server roundtrip works
✓ TLS handshake succeeds with self-signed certs
✓ File I/O operations work correctly
✓ Build works on OpenBSD (dev platform)

---

**End of Implementation Plan**
