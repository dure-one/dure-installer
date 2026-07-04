# dure Troubleshooting Guide

Common issues and solutions when using `dure`, a distributed e-commerce platform.

---

## Table of Contents

- [Quick Diagnostics](#quick-diagnostics)
- [Platform-Specific Issues](#platform-specific-issues)
- [GUI/Application Issues](#guiapplication-issues)
- [Hosting Configuration Issues](#hosting-configuration-issues)
- [Identity & Authentication Issues](#identity--authentication-issues)
- [Store Management Issues](#store-management-issues)
- [Messaging & Communication Issues](#messaging--communication-issues)
- [Database & Storage Problems](#database--storage-problems)
- [Configuration Issues](#configuration-issues)
- [Build & Compilation Issues](#build--compilation-issues)
- [Performance Issues](#performance-issues)
- [Network & Connectivity Issues](#network--connectivity-issues)
- [Recovery Procedures](#recovery-procedures)

---

## Quick Diagnostics

Run these commands to diagnose common problems:

```bash
# Check version
dure --version

# Test basic functionality
dure --help

# Run with verbose logging
dure -v

# Check configuration
# (Commands TBD based on final CLI design)
```

---

## Platform-Specific Issues

### Desktop Issues

#### GUI Not Launching

**Symptoms:** Application fails to start or crashes immediately on desktop.

**Common Causes:**
- Missing graphics drivers
- Incompatible egui version
- Display server issues (X11/Wayland on Linux)

**Solutions:**

```bash
# Linux: Check display server
echo $XDG_SESSION_TYPE  # Should show 'x11' or 'wayland'

# Try forcing X11 (if using Wayland)
env XDG_SESSION_TYPE=x11 dure

# Check for graphics driver issues
glxinfo | grep "OpenGL"  # Should show your GPU

# Update graphics drivers (Ubuntu/Debian)
sudo apt install mesa-utils libgl1-mesa-dri

# macOS: No additional dependencies needed
# Windows: Ensure graphics drivers are up to date
```

#### Window Not Rendering Correctly

**Solutions:**
```bash
# Try software rendering (slower but more compatible)
env LIBGL_ALWAYS_SOFTWARE=1 dure

# Check egui compatibility
# Update to latest version if available
cargo update -p egui -p eframe
cargo build --release
```

### Android Issues

#### App Crashes on Launch

**Common Causes:**
- Missing native libraries
- Incompatible ABI
- Insufficient permissions

**Solutions:**

```bash
# Rebuild with correct ABIs
cd mobile
./build.sh

# Check logcat for errors
adb logcat | grep dure

# Verify APK includes all necessary libraries
unzip -l app/build/outputs/apk/debug/app-debug.apk | grep '\.so$'
```

#### APK Installation Fails

**Solutions:**
```bash
# Uninstall old version first
adb uninstall com.dure.app

# Install fresh
adb install -r app/build/outputs/apk/debug/app-debug.apk

# Check for signature mismatches
adb install app/build/outputs/apk/debug/app-debug.apk
```

### WASM Issues

#### WASM Module Fails to Load

**Common Causes:**
- Incorrect MIME type
- Missing wasm-bindgen glue
- Browser compatibility

**Solutions:**

```bash
# Ensure proper build
cargo build --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir ./output \
  target/wasm32-unknown-unknown/release/dure.wasm

# Test in different browsers
# Chrome/Edge: Usually works
# Firefox: Check about:config -> javascript.options.wasm enabled
# Safari: Ensure recent version (14+)
```

#### WASM Guest Front Not Displaying

**Check browser console:**
```javascript
// Should show no errors
// Check for WebAssembly support
console.log(typeof WebAssembly);  // Should be "object"
```

---

## GUI/Application Issues

### Application Freezes or Hangs

**Possible Causes:**
- Long-running database operations
- Network requests blocking UI thread
- Heavy egui rendering

**Solutions:**

```bash
# Enable verbose logging to identify bottleneck
RUST_LOG=debug dure

# Check system resources
top  # or htop on Linux
# Activity Monitor on macOS
# Task Manager on Windows

# Reduce UI complexity (if available as option)
# Check for pending network operations
```

### Text Input Not Working

**Solutions:**
- Check keyboard layout and input method
- Try clicking in a different text field
- Restart application
- Check egui input handling in logs

### Colors/Theme Issues

**Solutions:**
- egui-material3 theme should be consistent
- Check system theme settings
- Verify egui version compatibility

---

## Hosting Configuration Issues

### DNS Configuration Fails

**Symptoms:** Unable to configure DNS settings, domain not resolving.

**Common Causes:**
- Invalid API tokens
- Incorrect DNS provider selection
- DNS propagation delay

**Solutions:**

```bash
# Verify DNS provider tokens are set correctly
# Check .dure/config.yaml

# Test DNS propagation
dig yourdomain.com
nslookup yourdomain.com

# Wait for propagation (can take up to 48 hours)
```

**Supported DNS Providers:**
- DuckDNS
- Cloudflare DNS
- Porkbun
- GCP Cloud DNS

### Web Hosting Setup Fails

**Symptoms:** Cannot deploy web front-end, hosting provider errors.

**Supported Web Providers:**
- GCE (Google Compute Engine)
- VPS (Generic VPS via SSH)
- Cloudflare Pages
- Firebase Hosting

**Solutions:**

```bash
# Verify API keys/tokens
# Check provider account status and quotas
# Ensure billing is enabled (for paid services)
# Check provider-specific logs
```

### Database Provider Issues

**Supported DB Providers:**
- GCE (with custom DB setup)
- GCP Cloud SQL
- Supabase

**Common Issues:**

```bash
# Connection refused
# - Check firewall rules
# - Verify IP whitelisting
# - Ensure database is running

# Authentication failed
# - Verify credentials in config
# - Check password/token expiration
# - Regenerate API keys if needed
```

---

## Identity & Authentication Issues

### Private/Public Key Generation Fails

**Symptoms:** Cannot generate identity keys.

**Solutions:**

```bash
# Ensure sufficient entropy
# On Linux, check /dev/urandom availability
ls -l /dev/urandom

# Check permissions on key storage directory
mkdir -p ~/.local/share/dure/keys
chmod 700 ~/.local/share/dure/keys

# Verify OpenSSL/crypto libraries installed
```

### Firebase/Supabase Authentication Fails

**Common Issues:**
- Invalid API keys
- Network connectivity problems
- Account permissions

**Solutions:**

```bash
# Verify API keys in configuration
# Check network connectivity
curl -I https://firebase.google.com
curl -I https://supabase.com

# Ensure account has proper permissions
# Check Firebase/Supabase console for project status
```

### GitHub Sigstore Attestation Issues

**Symptoms:** Cannot verify binary attestations.

**Solutions:**

```bash
# Ensure gh CLI is installed
gh version

# Verify sigstore tools available
# Check attestation verification command
```

---

## Store Management Issues

### Product Creation Fails

**Common Causes:**
- Invalid product data
- Image upload issues
- Database connection problems

**Solutions:**

```bash
# Verify image format and size
# Supported formats: PNG, JPEG, WebP
# Maximum size: (TBD)

# Check database connection
# Verify storage quota
```

### Order Processing Issues

**Order Status Flow:**
- Pending → Processing → Shipping → Completed

**Common Problems:**

```bash
# Orders stuck in pending
# - Check payment gateway status
# - Verify order data completeness

# Cannot update order status
# - Check permissions
# - Verify database connectivity
```

### Payment Integration Issues

**Supported Payment Gateways:**
- Portone
- KakaoPay

**Common Problems:**

```bash
# Payment authorization fails
# - Verify API keys
# - Check merchant account status
# - Ensure SSL/TLS configured correctly

# Callback/webhook not received
# - Check firewall rules
# - Verify callback URL configuration
# - Check webhook signature validation
```

---

## Messaging & Communication Issues

### WebSocket Connection Fails

**Symptoms:** Cannot send/receive messages in real-time.

**Common Causes:**
- Firewall blocking WebSocket connections
- Server not running
- Invalid authentication

**Solutions:**

```bash
# Check WebSocket server status
# Verify port is not blocked
netstat -tulpn | grep <port>

# Test WebSocket connection
wscat -c ws://yourserver.com:<port>

# Check firewall rules
# Ensure port is allowed for WebSocket connections
```

### Messages Not Synchronizing

**Solutions:**

```bash
# Check network connectivity
# Verify server is accessible
# Check message queue status
# Ensure database is not locked
```

### Group/Channel Issues

**Common Problems:**
- Cannot create channels
- Members not seeing messages
- Permission issues

**Solutions:**

```bash
# Verify channel permissions
# Check member roles
# Ensure proper authentication
```

---

## Database & Storage Problems

### Database Corruption

**Symptoms:** Application crashes, data inconsistency, query failures.

**Diagnosis:**

```bash
# For SQLite databases
sqlite3 ~/.local/share/dure/dure.db "PRAGMA integrity_check;"

# Check database file size and permissions
ls -lh ~/.local/share/dure/dure.db
```

**Recovery:**

```bash
# Backup current database
cp ~/.local/share/dure/dure.db ~/.local/share/dure/dure.db.corrupt

# Try repair
sqlite3 ~/.local/share/dure/dure.db "VACUUM;"
sqlite3 ~/.local/share/dure/dure.db "REINDEX;"

# If repair fails, restore from backup (if available)
```

### Database Lock Issues

**Symptoms:** "Database is locked" errors.

**Solutions:**

```bash
# Check for other running instances
ps aux | grep dure

# Remove stale lock files (only if dure is not running)
rm ~/.local/share/dure/*.db-wal
rm ~/.local/share/dure/*.db-shm

# Increase timeout in configuration (if available)
```

### Storage Quota Exceeded

**Solutions:**

```bash
# Check disk space
df -h ~/.local/share/dure

# Clean up old data
# Remove cached images
# Archive old orders/messages

# Check database size
du -h ~/.local/share/dure/dure.db
```

---

## Configuration Issues

### Invalid Configuration File

**Symptoms:** Application fails to start, configuration errors.

**Solutions:**

```bash
# Validate YAML syntax
# Check .dure/config.yaml

# Use Python to validate
python3 -c "import yaml; yaml.safe_load(open('.dure/config.yaml'))"

# Reset to defaults
mv .dure/config.yaml .dure/config.yaml.backup
# Dure will create new config on next run
```

### Configuration Not Taking Effect

**Config Precedence (highest to lowest):**
1. CLI flags
2. Environment variables
3. Project config (`.dure/config.yaml`)
4. User config (`~/.config/dure/config.yaml`)
5. Defaults

**Diagnosis:**

```bash
# Check which config is being used
# (Command TBD)

# Verify environment variables
env | grep DURE

# Override with CLI flags for testing
dure --config-path /path/to/config.yaml
```

### Missing API Tokens

**Solutions:**

```bash
# Set environment variables
export DURE_DNS_TOKEN="your_token"
export DURE_WEB_TOKEN="your_token"
export DURE_DB_TOKEN="your_token"

# Or add to configuration file
# Edit .dure/config.yaml
```

---

## Build & Compilation Issues

### Rust Version Too Old

**Symptoms:** Compilation fails with "feature not supported" errors.

**Solutions:**

```bash
# Update Rust
rustup update stable
rustup default stable

# Verify version (must be 1.81+)
rustc --version
```

### Missing Dependencies

**Linux:**
```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config \
  libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev

# Fedora
sudo dnf install gcc pkg-config \
  libxcb-devel libxkbcommon-devel
```

**macOS:**
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

**Windows:**
- Install Visual Studio Build Tools
- Ensure Rust targets are installed

### Compilation Errors

**Common Issues:**

```bash
# Clean build cache
cargo clean

# Update dependencies
cargo update

# Check for conflicting versions
cargo tree | grep -i <dependency>

# Rebuild
cargo build --release
```

### Linker Errors

**Solutions:**

```bash
# Install linker
# Ubuntu/Debian
sudo apt install lld

# Configure Rust to use lld
# Add to .cargo/config.toml:
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

---

## Performance Issues

### Slow Application Startup

**Common Causes:**
- Large database
- Many plugins/extensions
- Slow disk I/O

**Solutions:**

```bash
# Check database size
du -h ~/.local/share/dure/dure.db

# Vacuum database
sqlite3 ~/.local/share/dure/dure.db "VACUUM;"

# Use SSD if available
# Reduce auto-load data on startup
```

### UI Lag/Stutter

**Solutions:**

```bash
# Check system resources
top  # or htop

# Reduce UI complexity
# Disable animations (if option available)
# Lower refresh rate

# Check graphics driver
glxinfo | head -20  # Linux
```

### Slow Network Operations

**Solutions:**

```bash
# Check network latency
ping yourserver.com

# Test connection speed
curl -o /dev/null http://yourserver.com/testfile

# Enable connection pooling (if available)
# Use CDN for static assets
```

### High Memory Usage

**Diagnosis:**

```bash
# Monitor memory usage
top -p $(pgrep dure)

# Check for memory leaks
valgrind --leak-check=full dure
```

**Solutions:**
- Reduce cache size in configuration
- Close unused tabs/windows
- Restart application periodically

---

## Network & Connectivity Issues

### Cannot Connect to Hosting Provider

**Solutions:**

```bash
# Check internet connectivity
ping 8.8.8.8

# Test DNS resolution
nslookup yourdomain.com

# Check firewall rules
sudo iptables -L  # Linux
# Windows Firewall settings

# Verify proxy settings (if applicable)
echo $HTTP_PROXY
echo $HTTPS_PROXY
```

### SSL/TLS Certificate Errors

**Solutions:**

```bash
# Update CA certificates
# Ubuntu/Debian
sudo apt update && sudo apt install ca-certificates

# macOS
# System should update automatically

# Windows
# Check Windows Update

# Verify certificate chain
openssl s_client -connect yourserver.com:443 -showcerts
```

### Timeout Errors

**Solutions:**

```bash
# Increase timeout in configuration
# (if option available)

# Check server status
curl -I https://yourserver.com

# Use verbose mode to identify bottleneck
RUST_LOG=debug dure -v
```

---

## Recovery Procedures

### Complete Application Reset

**Warning:** This will delete all local data.

```bash
# Backup current data
mkdir -p ~/dure-backup
cp -r ~/.local/share/dure ~/dure-backup/
cp -r .dure ~/dure-backup/

# Remove all dure data
rm -rf ~/.local/share/dure
rm -rf ~/.config/dure
rm -rf .dure

# Restart dure
dure
```

### Restore from Backup

```bash
# Stop dure if running
pkill dure

# Restore data
cp -r ~/dure-backup/dure ~/.local/share/
cp -r ~/dure-backup/.dure ./

# Restart dure
dure
```

### Database Recovery

```bash
# Export data (if possible)
# (Command TBD)

# Create new database
mv ~/.local/share/dure/dure.db ~/.local/share/dure/dure.db.old
# Dure will create new database on next run

# Import data
# (Command TBD)
```

### Configuration Reset

```bash
# Backup current config
cp .dure/config.yaml .dure/config.yaml.backup

# Remove config
rm .dure/config.yaml

# Dure will create default config on next run
dure
```

---

## Debug Logging

Enable detailed logging for troubleshooting:

```bash
# Basic verbose mode
dure -v

# Debug level
RUST_LOG=debug dure

# Trace level (very detailed)
RUST_LOG=trace dure 2>dure-trace.log

# Module-specific logging
RUST_LOG=dure::storage=debug dure
RUST_LOG=dure::network=trace dure

# Combined with output redirect
RUST_LOG=debug dure 2>debug.log 1>output.log
```

### Log Locations

- **Linux**: `~/.local/share/dure/logs/`
- **Windows**: `%APPDATA%\dure\logs\`
- **macOS**: `~/Library/Application Support/dure/logs/`
- **Android**: Use `adb logcat | grep dure`

---

## Getting Help

If you're still stuck:

1. **Check documentation:**
   - [README.md](../README.md) - Project overview
   - [CLAUDE.md](../CLAUDE.md) - Complete project guide
   - [CLI_REFERENCE.md](./CLI_REFERENCE.md) - Command reference
   - [ARCHITECTURE.md](./ARCHITECTURE.md) - Technical architecture

2. **Enable debug logging:**
   ```bash
   RUST_LOG=debug dure 2>debug.log
   ```

3. **Check system requirements:**
   ```bash
   dure --version
   rustc --version  # Should be 1.81+
   ```

4. **Report issues:**
   - GitHub Issues: https://github.com/nikescar/dure/issues
   - Include: dure version, OS, relevant logs

---

## See Also

- [INSTALLING.md](./INSTALLING.md) - Installation instructions
- [CLI_REFERENCE.md](./CLI_REFERENCE.md) - Complete command reference
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Technical architecture
- [INDEX.md](./INDEX.md) - Documentation index

---

*Last updated: 2026-04-03*
