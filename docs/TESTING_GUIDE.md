# Testing Guide - Async Runtime Migration

**Branch:** `feat/migrate-to-smol`  
**Migration:** asupersync → smol  
**Status:** Ready for testing

## Prerequisites

### Platform Requirements

**Supported Platforms:**
- ✅ Linux (x86_64, aarch64)
- ✅ macOS (Intel, Apple Silicon)
- ✅ Windows (x86_64)
- ⚠️ OpenBSD (see OpenBSD-specific section)

### System Dependencies

**All Platforms:**
```bash
# Rust toolchain (1.85+)
rustc --version  # Should show 1.85 or newer
```

**OpenBSD Only:**
```bash
# OpenSSL (not LibreSSL)
pkg_add openssl-3.5.7v0
export OPENSSL_LIB_DIR=/usr/local/lib/eopenssl35
export OPENSSL_INCLUDE_DIR=/usr/local/include/eopenssl35
```

## Quick Start

### 1. Clone and Checkout

```bash
cd /path/to/dure
git checkout feat/migrate-to-smol
git pull  # Ensure latest commits
```

### 2. Build

```bash
# Clean build
cargo clean

# Check compilation
cargo check

# Full build
cargo build --release
```

**Expected output:**
- ✅ Clean compilation on Linux/macOS/Windows
- ⚠️ OpenBSD: May have pre-existing diesel/other errors (not migration-related)

### 3. Run Tests

```bash
# Unit tests
cargo test

# Integration tests
cargo test --test '*'

# Specific test
cargo test --test async_runtime_tests
```

## Component Testing

### WebSocket Client

**Test 1: HTTPS GET Request**

```bash
# Start a test server (if you have one), or use public endpoint
cargo run --bin wss-client -- \
  --url https://echo.websocket.org \
  --mode get \
  --path /
```

**Expected:**
- ✅ TLS handshake completes
- ✅ HTTP response received
- ✅ Clean exit

**Test 2: WebSocket Connection**

```bash
# Connect to WebSocket server
cargo run --bin wss-client -- \
  --url wss://echo.websocket.org \
  --mode ws
```

**Expected:**
- ✅ WebSocket upgrade succeeds
- ✅ Can send/receive messages
- ✅ Interactive prompt works
- ✅ Ctrl+D exits cleanly

### WebSocket Server

**Test 3: Start Server**

```bash
# In terminal 1 - Start server
cargo run --bin wss-server -- \
  --domain localhost \
  --addr 0.0.0.0:8443

# Server should start and show:
# ✓ Database opened: ...
# ✓ Static files present (or download)
# ✓ Using certificates from config.yml (or self-signed)
# 🚀 HTTPS/WSS server (TLS enabled)
```

**Test 4: Client Connection to Server**

```bash
# In terminal 2 - Connect client
cargo run --bin wss-client -- \
  --url wss://localhost:8443 \
  --mode ws
```

**Expected:**
- ✅ Server accepts connection
- ✅ Client shows "✓ WebSocket Connected!"
- ✅ Messages echo between client/server
- ✅ Stats printed on server (connections, messages)

### HTTP Endpoints

**Test 5: Static File Serving**

```bash
# With server running:
curl -k https://localhost:8443/
# Should return HTML from static files

curl -k https://localhost:8443/swagger-ui
# Should return Swagger UI HTML
```

**Test 6: API Endpoints**

```bash
# POST webhook test
curl -k -X POST https://localhost:8443/webhook/test \
  -H "Content-Type: application/json" \
  -d '{"test":"data"}'

# Should log webhook on server side
```

## Performance Testing

### Concurrent Connections

```bash
# Terminal 1: Start server with stats
cargo run --release --bin wss-server -- \
  --domain localhost \
  --stats-interval 10

# Terminal 2-N: Start multiple clients
for i in {1..10}; do
  cargo run --release --bin wss-client -- \
    --url wss://localhost:8443 &
done

# Watch server stats output
# Should handle 10+ concurrent connections
```

### Memory Profiling

```bash
# Build with debug symbols
cargo build --profile dev-release

# Run under valgrind (Linux only)
valgrind --tool=massif \
  ./target/dev-release/wss-server \
  --domain localhost

# Check for memory leaks
```

### Load Testing

```bash
# Using websocat (install: cargo install websocat)
for i in {1..100}; do
  echo "Message $i" | websocat -k wss://localhost:8443 &
done

# Server should handle load without crashes
```

## Migration-Specific Checks

### Verify No asupersync References

```bash
# Should return empty
find mobile/src -name "*.rs" | xargs grep "asupersync"
```

✅ **Expected:** No output

### Verify smol Runtime Usage

```bash
# Should find smol::Timer, smol::spawn, smol::block_on
grep -r "smol::" mobile/src/wss | head -5
```

✅ **Expected:** Multiple matches

### Check TLS Implementation

```bash
# Should use async-tls
grep -r "async_tls" mobile/src/wss
```

✅ **Expected:** TlsConnector, TlsAcceptor references

## Regression Testing

### Before/After Comparison

If you have the old `main` branch behavior:

```bash
# Test old version
git checkout main
cargo build --release
cargo run --bin wss-client -- --url wss://echo.websocket.org
# Record behavior

# Test new version  
git checkout feat/migrate-to-smol
cargo build --release
cargo run --bin wss-client -- --url wss://echo.websocket.org
# Compare behavior
```

**Should be identical:**
- Connection success rate
- Message delivery
- Error handling
- Performance characteristics

## OpenBSD-Specific Testing

### Current Status

✅ **Resolved:** muda/tray-icon blocker (excluded on OpenBSD)  
⚠️ **Known:** Pre-existing diesel macro errors (unrelated to migration)

### Build on OpenBSD

```bash
# Set OpenSSL paths (if not in .cargo/config.toml)
export OPENSSL_LIB_DIR=/usr/local/lib/eopenssl35
export OPENSSL_INCLUDE_DIR=/usr/local/include/eopenssl35

# Build (may have non-migration errors)
cargo build

# If diesel errors: these are pre-existing, not migration-related
# Migration itself is complete (no asupersync errors)
```

### Server Testing on OpenBSD

```bash
# If build succeeds:
cargo run --bin wss-server -- --domain localhost

# Test local connection:
cargo run --bin wss-client -- --url wss://localhost:8443
```

## Troubleshooting

### "TLS handshake failed"

**Cause:** Certificate issues  
**Fix:**
```bash
# Use --insecure for testing
cargo run --bin wss-client -- \
  --url wss://localhost:8443 \
  --insecure
```

### "Connection refused"

**Cause:** Server not running or wrong port  
**Fix:**
```bash
# Check server is running
ps aux | grep wss-server

# Check correct port (default 8443)
netstat -an | grep 8443
```

### "No such file or directory" (database)

**Cause:** Database not initialized  
**Fix:**
```bash
# Server creates DB automatically on first run
# Check logs for DB path
# Default: ~/.cache/dure/data/dure.db
```

### Compilation errors on OpenBSD

**diesel macro errors:**
- These are pre-existing code issues
- Not related to async runtime migration
- Migration code itself compiles cleanly

**muda/tray-icon errors:**
- Should be resolved (check you're on latest commit)
- If still present: `git pull origin feat/migrate-to-smol`

## Success Criteria

### Minimum Passing Tests

- [x] `cargo check` completes (or shows only pre-existing errors)
- [x] `cargo test` passes
- [x] WebSocket client connects successfully
- [x] WebSocket server accepts connections
- [x] Messages flow bidirectionally
- [x] TLS handshake works
- [x] No asupersync references in code
- [x] Server handles concurrent connections
- [x] Clean shutdown (Ctrl+C)

### Performance Benchmarks

Compare with baseline (if available):

- Latency: <10ms message round-trip
- Throughput: 1000+ messages/second
- Connections: 100+ concurrent
- Memory: Stable under load (no leaks)

## Reporting Issues

### If Tests Fail

**Collect diagnostics:**
```bash
# Rust version
rustc --version

# Platform
uname -a

# Build output
cargo build 2>&1 | tee build.log

# Runtime logs
RUST_LOG=debug cargo run --bin wss-server 2>&1 | tee server.log
```

**Report:**
1. Platform (OS, architecture)
2. Rust version
3. Failing test command
4. Full error output
5. Attach: build.log, server.log

### Known Non-Issues

**These are NOT migration problems:**
- diesel macro errors on OpenBSD (pre-existing)
- go-webauthn disabled (platform limitation)
- Certificate warnings (self-signed certs)
- GOOGLE_OAUTH warnings (optional feature)

## Next Steps After Testing

### If All Tests Pass

```bash
# Ready to merge
git checkout main
git merge feat/migrate-to-smol
git push origin main
```

### If Tests Fail

1. Document failures in GitHub issue
2. Check if regression (test on `main` branch)
3. Determine if migration-related or pre-existing
4. Fix or defer based on priority

## Additional Resources

- [Migration Completion Report](./MIGRATION_COMPLETE.md)
- [Design Specification](./superpowers/specs/2026-06-22-asupersync-to-smol-migration-design.md)
- [Implementation Plan](./superpowers/plans/2026-06-22-asupersync-to-smol-migration.md)

## Contact

Issues or questions:
- GitHub: https://github.com/nikescar/dure/issues
- Email: nikescar@gmail.com
