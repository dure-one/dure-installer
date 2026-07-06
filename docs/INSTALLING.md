# Installation Guide

Complete installation instructions for `dure`, a distributed e-commerce platform supporting Desktop, Android, and WASM.

---

## Table of Contents

- [Requirements](#requirements)
- [Quick Install](#quick-install)
- [Installation Methods](#installation-methods)
  - [Build from Source](#build-from-source)
  - [Platform-Specific Builds](#platform-specific-builds)
- [Platform-Specific Builds](#platform-specific-builds)
  - [Desktop (Linux, Windows, macOS)](#desktop-linux-windows-macos)
  - [Android](#android)
  - [WASM](#wasm)
- [Configuration](#configuration)
- [Verifying Installation](#verifying-installation)
- [Troubleshooting](#troubleshooting)

---

## Requirements

### Minimum Requirements

- **Rust**: 2021 edition (min version 1.81)
- **Platform-specific**:
  - Desktop: egui 0.33, eframe 0.33
  - Android: NDK, Android SDK (API level 21+)
  - WASM: wasm-bindgen, wasm32-unknown-unknown target

### Supported Platforms

| Platform | Architecture | Status | Features |
|----------|--------------|--------|----------|
| Linux | x86_64, aarch64 | ✅ Supported | Full EGUI client (X11, Wayland) |
| Windows | x86_64 | ✅ Supported | Full EGUI client |
| macOS | x86_64, aarch64 | ✅ Supported | Full EGUI client |
| Android | arm64-v8a, armeabi-v7a | ✅ Supported | EGUI client via native-activity |
| WASM | wasm32 | ✅ Supported | Guest & Store Front only |

---

## Quick Install (Recommended)

Install dure with a single command:

### Stable Release (Recommended)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://dure.one/install.sh | sh
```

### Development Build (Latest)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://dure.one/install.sh | DURE_CHANNEL=dev sh
```

### Options

- **Quiet mode:** Add `DURE_QUIET=1` to suppress output
- **Custom channel:** Set `DURE_CHANNEL=dev` for development builds

The installer will:
1. Detect your platform (Linux x86_64 only)
2. Download and verify the binary with SHA256
3. Install to `~/.local/bin/dure`
4. Add to your PATH automatically

**Note:** Currently only supports Linux x86_64. arm64/aarch64 support coming soon.

---

## Installation Methods

### Build from Source

For development or customization:

```bash
# Clone the repository
git clone https://github.com/nikescar/dure.git
cd dure

# Build for desktop (debug)
cargo build

# Build for desktop (release)
cargo build --release

# Build for desktop (dev-release - faster iteration)
cargo build --profile dev-release

# The binary is at ./target/release/dure
./target/release/dure --version
```

**Build Profiles:**

- **dev**: Fast incremental builds, full debug info, no optimizations
- **release**: Full LTO, size optimization (opt-level = "z"), stripped symbols
- **dev-release**: Balanced for testing (opt-level = 2, no LTO, faster builds)

---

## Platform-Specific Builds

### Desktop (Linux, Windows, macOS)

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Dev-release (faster iteration)
cargo build --profile dev-release

# Run directly
cargo run
```

**Platform-specific dependencies:**
- Linux: X11 or Wayland support
- Windows: No additional requirements
- macOS: No additional requirements

### Android

Building for Android requires additional setup:

```bash
cd mobile

# Full Android build
./build.sh

# Rust library only
./build.rust-only.sh
```

**Requirements:**
- Android NDK
- Android SDK (API level 21+)
- Java/Kotlin build tools

**Supported ABIs:**
- arm64-v8a (primary)
- armeabi-v7a

See `mobile/README.md` for detailed Android build instructions.

### WASM

Building for WASM:

```bash
# Install wasm-bindgen-cli
cargo install wasm-bindgen-cli

# Build WASM target
cargo build --target wasm32-unknown-unknown --release

# Generate bindings
wasm-bindgen --target web --out-dir ./output target/wasm32-unknown-unknown/release/dure.wasm
```

**Note:** WASM builds support Guest Front and Store Front only. Hosting management features are not available in WASM.

---

## Configuration

After installation, configure Dure for your environment.

### Initialize Configuration

Dure uses layered configuration:

1. CLI flags (highest priority)
2. Environment variables
3. Project config (`.dure/config.yaml`)
4. User config (`~/.config/dure/config.yaml`)
5. Defaults (lowest priority)

### Example Configuration

Dure uses layered configuration:

1. CLI flags (highest priority)
2. Environment variables
3. Project config (`.dure/config.yaml`)
4. User config (`~/.config/dure/config.yaml`)
5. Defaults (lowest priority)

Example `.dure/config.yaml`:

```yaml
device:
  name: "mylaptop"

# DNS configuration
dns:
  domain_registar: ""          # ""/Cloudflare/Porkbun
  domain_registar_token: ""
  name: "www.example.com"
  dns_provider: "CLOUDFLARE_DNS"  # DUCKDNS/CLOUDFLARE_DNS/PORKBUN/GCP_CLOUDDNS
  dns_provider_token: ""

# Web hosting configuration
web:
  web_provider: "GCE"          # GCE/VPS/CLOUDFLARE_PAGES/FIREBASE_HOSTING
  web_provider_token: ""
  db_provider: "GCE"           # GCE/GCP_CLOUDSQL/SUPABASE
  db_provider_token: ""

# Messaging service configuration
messaging:
  msg_provider: "GCE"          # GCE/GCP_CLOUDMESSAGING/SUPABASE_REALTIME
  msg_provider_token: ""
```

---

## Verifying Installation

```bash
# Check version
dure --version

# Run in GUI mode (default)
dure

# Run in server mode
dure --serv

# Test basic functionality
dure --help
```

---

## Troubleshooting

### Common Build Issues

#### "error: linker `cc` not found"

Install build tools:

```bash
# Ubuntu/Debian
sudo apt install build-essential pkg-config

# Fedora
sudo dnf install gcc pkg-config

# macOS
xcode-select --install
```

#### Missing Platform Dependencies

**Linux (X11/Wayland):**
```bash
# Ubuntu/Debian
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
  libxkbcommon-dev

# Fedora
sudo dnf install libxcb-devel libxkbcommon-devel
```

**macOS:**
```bash
# No additional dependencies needed
# Ensure Xcode Command Line Tools are installed
xcode-select --install
```

**Windows:**
```bash
# No additional dependencies needed
# Ensure Visual Studio Build Tools are installed
```

#### Android Build Fails

Ensure proper Android setup:

```bash
# Set environment variables
export ANDROID_HOME=~/Android/Sdk
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/<version>

# Verify NDK installation
ls $ANDROID_NDK_HOME

# Install required targets
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
```

#### WASM Build Fails

Install required tools:

```bash
# Add wasm target
rustup target add wasm32-unknown-unknown

# Install wasm-bindgen-cli
cargo install wasm-bindgen-cli

# Verify installation
wasm-bindgen --version
```

#### "Rust version too old"

Dure requires Rust 1.81 or newer:

```bash
# Update Rust
rustup update stable

# Switch to stable
rustup default stable

# Check version
rustc --version
```

#### Compilation Errors with Dependencies

```bash
# Clean build cache
cargo clean

# Update dependencies
cargo update

# Rebuild
cargo build --release
```

#### Permission Denied

When installing to system directories:

```bash
# Option 1: Use sudo for system installation
sudo cp target/release/dure /usr/local/bin/

# Option 2: Install to user directory (recommended)
mkdir -p ~/.local/bin
cp target/release/dure ~/.local/bin/

# Add to PATH if needed
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Getting Help

- **Documentation**: [docs/INDEX.md](./INDEX.md)
- **Troubleshooting**: [TROUBLESHOOTING.md](./TROUBLESHOOTING.md)
- **Issues**: [GitHub Issues](https://github.com/nikescar/dure/issues)

---

## Related Documentation

- [README.md](../README.md) - Project overview
- [CLAUDE.md](../CLAUDE.md) - Complete project guide
- [CLI_REFERENCE.md](./CLI_REFERENCE.md) - Complete command reference
- [ARCHITECTURE.md](./ARCHITECTURE.md) - Technical architecture
- [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) - Common issues and solutions

---

*Last updated: 2026-04-02*
