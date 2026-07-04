# Dure - Distributed E-commerce Platform

## Project Overview

**Dure** is a distributed e-commerce client and hosting solution built with Rust and egui. It enables small shop owners to run e-commerce operations without traditional centralized server infrastructure.

### Key Characteristics

- **Language**: Rust (nightly toolchain required)
- **UI Framework**: egui + eframe (Material3 design)
- **Architecture**: Multi-platform (Desktop, Mobile, WASM)
- **Purpose**: Distributed e-commerce for small shop owners
- **License**: Dual MIT/Apache-2.0

## Project Structure

```
dure/
├── mobile/              # Main application code
│   ├── src/            # Rust source code
│   ├── app/            # Android app configuration
│   ├── assets/         # Application assets
│   └── Cargo.toml      # Package manifest
├── docs/               # Documentation (see docs/INDEX.md for complete index)
├── deploy/             # Deployment configurations
├── fastlane/           # Mobile CI/CD automation
├── snap/               # Snap package configuration
├── scripts/            # Build and utility scripts
└── Cargo.toml          # Workspace manifest
```

## Core Features

All function exists for both EGUI and CLI. 

### 1. Identity Management
- Private/Public key for personal identity
- Platform(GCP), SSH Host management
- Attestation for WASM/EGUI apps with GitHub Sigstore

### 2. Platform Management 
- GCP management (Add/Del VM,View Billing)

### 3. DNS Management(Cloudflare, Google Cloud DNS, DuckDNS, Porkbun)
- DNS management (Add/Del domain, Add/Del Txt Record)

### 4. SSH Host Management
- Automatically added host from Platform Management
- Docker Management (Add/Del docker host from dockerhub)
- Port Management (Port open/close management with nft)
- Ansible Management (Add/Del ansible roles from ansiblegalaxy)
- System Hardener (using Jangbi project)
- Dure WSS Service Management (Add/Del dure install)

### 5. Hosting Management
- DNS management (octodns)
- ACME License Management (lego)
- Dure Chat Server Hosting (dure)

### 6. Store Management (EGUI/CLI, WSS Client)
- Promotions
- Products
- Orders
- Shipments
- Accounts
- Dure (shared listings/shipments with other stores)

### 7. Guest Front (WASM, WSS Client)
- Minimum guest identity for customers
- Product browsing and cart functionality
- Product listings
- Shopping cart
- Payment integration (Portone, KakaoPay)

## Platform Support

| Platform | Status | Features | Distribution |
|----------|--------|----------|--------------|
| **Linux x86_64** | ✅ Supported | Full EGUI client / Headless CLI | GitHub Releases, Snap Store |
| **Linux aarch64** | ✅ Supported | Headless CLI | GitHub Releases, Snap Store |
| **macOS (Intel/Apple Silicon)** | ✅ Supported | Full EGUI client | GitHub Releases |
| **Windows x86_64** | ✅ Supported | Full EGUI client | GitHub Releases |
| **Android** | ✅ Supported | EGUI client via native-activity | Google Play Store |
| **WASM** | ✅ Supported | Guest & Store Front only | Web deployment |

## Technology Stack

### Core Dependencies
- **UI**: egui 0.33, eframe 0.33, egui-material3
- **i18n**: egui-i18n with Fluent
- **Database**: Diesel 2.3, diesel_migrations 2.3 (SQLite with encryption via libsqlite3-hotbundle + optional PostgreSQL)
- **Async**: smol 2.0 (multi-threaded runtime)
- **HTTP**: ureq 2.12 (with rustls/ring TLS backend), ehttp
- **TLS/Crypto**: rustls 0.23 (ring backend), futures-rustls 0.26 - **no OpenSSL dependency**
- **SSH**: russh 0.45 (pure Rust, no OpenSSL)
- **Serialization**: serde, serde_json, bincode

### Platform-Specific
- **Android**: ndk-context, jni, android-activity, diesel (SQLite)
- **Desktop**: tray-icon, webbrowser, trash, diesel (SQLite/PostgreSQL with dotenvy)
- **WASM**: wasm-bindgen, web-sys, js-sys, diesel (SQLite with sqlite-wasm-rs, sqlite-wasm-vfs)

## Build Profiles

### Development (`dev`)
- Fast incremental builds
- Full debug info
- No optimizations
- 256 codegen units

### Release (`release`)
- Full LTO
- Size optimization (`opt-level = "z"`)
- Single codegen unit
- Symbols stripped
- Panic = abort

### Dev-Release (`dev-release`)
- Balanced profile for testing
- No LTO (faster builds)
- opt-level = 2
- 16 codegen units
- Incremental compilation enabled

## Development Guidelines

### Code Style
- Follow Rust 2021 idioms
- Use clippy::pedantic
- Maintain cross-platform compatibility
- Document public APIs with examples

### Safety Requirements
- Minimize unsafe code
- Document all invariants
- Test on all target platforms
- Handle errors gracefully

### Performance Considerations
- UI rendering is performance-critical
- Use async for I/O operations
- Leverage DataFusion for analytics queries
- Profile before optimizing

## Building

### Prerequisites

**Rust Toolchain**: This project requires **Rust nightly**. Install with:
```bash
rustup toolchain install nightly
rustup default nightly
```

#### Platform-Specific Requirements

##### Linux (for musl builds)
```bash
# For x86_64 musl builds
sudo apt-get update
sudo apt-get install -y musl-tools musl-dev

# For aarch64 musl cross-compilation
# Download and install aarch64-linux-musl toolchain
wget https://github.com/troglobit/misc/releases/download/11-20211120/aarch64-linux-musl-cross.tgz
tar -xzf aarch64-linux-musl-cross.tgz -C "$HOME"
export PATH="$HOME/aarch64-linux-musl-cross/bin:$PATH"
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER="$HOME/aarch64-linux-musl-cross/bin/aarch64-linux-musl-gcc"
```

##### Linux GUI Dependencies
For GUI builds on Linux, install GTK and related dependencies:
```bash
sudo apt-get install -y pkg-config libgtk-3-dev \
  libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev \
  libglib2.0-dev libgdk-pixbuf-2.0-dev libwayland-dev \
  libcairo2-dev libpango1.0-dev golang-go
```

##### Windows
**Note**: The CI workflow installs OpenSSL via vcpkg, but this may not be necessary since the project uses rustls. If you encounter build issues:
```bash
vcpkg install openssl:x64-windows-static-md
vcpkg integrate install
```

### Quick Build Commands

#### Desktop - Local Development
```bash
# Debug build (default toolchain)
cargo build

# Release build (default toolchain)
cargo build --release

# Dev-release (faster iteration)
cargo build --profile dev-release
```

#### Desktop - Production Builds (CI/CD Targets)

**Linux x86_64 (musl) - Headless**
```bash
rustup target add x86_64-unknown-linux-musl
cd mobile
RUSTFLAGS="-C link-arg=-lm" \
  cargo +nightly build --release --bin dure-desktop \
  --no-default-features --target x86_64-unknown-linux-musl
```

**Linux x86_64 (musl) - GUI**
```bash
rustup target add x86_64-unknown-linux-musl
cd mobile
RUSTFLAGS="-C target-feature=-crt-static -C link-arg=-lm" \
  cargo +nightly build --release --bin dure-desktop \
  --target x86_64-unknown-linux-musl
```

**Linux aarch64 (musl) - Headless**
```bash
rustup target add aarch64-unknown-linux-musl
cd mobile
RUSTFLAGS="-C link-arg=-lm -C link-arg=-lpthread -C link-arg=-ldl -C link-arg=-lc" \
  cargo +nightly build --release --bin dure-desktop \
  --no-default-features --target aarch64-unknown-linux-musl
```

**macOS x86_64**
```bash
rustup target add x86_64-apple-darwin
cd mobile
cargo +nightly build --release --bin dure-desktop \
  --target x86_64-apple-darwin
```

**macOS aarch64 (Apple Silicon)**
```bash
rustup target add aarch64-apple-darwin
cd mobile
cargo +nightly build --release --bin dure-desktop \
  --target aarch64-apple-darwin
```

**Windows x86_64**
```bash
rustup target add x86_64-pc-windows-msvc
cd mobile
cargo +nightly build --release --bin dure-desktop \
  --target x86_64-pc-windows-msvc
```

### Android
```bash
cd mobile
./build.sh  # Full Android build
./build.rust-only.sh  # Rust library only
```

### WASM
```bash
# Build instructions TBD
# Target: wasm32-unknown-unknown
```

### CI/CD Build Matrix

The project uses GitHub Actions for automated builds. See `.github/workflows/release.yml` for the complete build matrix:

| Platform | Target | Build Type | Notes |
|----------|--------|------------|-------|
| **Linux x86_64** | `x86_64-unknown-linux-musl` | Headless | Static musl, no GUI |
| **Linux x86_64** | `x86_64-unknown-linux-musl` | GUI | Dynamic CRT for GTK |
| **Linux aarch64** | `aarch64-unknown-linux-musl` | Headless | Cross-compiled with musl-cross |
| **macOS x86_64** | `x86_64-apple-darwin` | GUI | Intel Macs |
| **macOS aarch64** | `aarch64-apple-darwin` | GUI | Apple Silicon |
| **Windows x86_64** | `x86_64-pc-windows-msvc` | GUI | MSVC toolchain |

**Build Features:**
- All builds use **Rust nightly** toolchain
- Linux builds prefer **musl** for static linking and portability
- GUI builds disable `crt-static` on musl to allow dynamic GTK linking
- aarch64 builds include explicit libc/pthread/dl linking for SQLite compatibility

## Documentation

See [`docs/`](./docs/) directory for detailed documentation:

- **[docs/INDEX.md](./docs/INDEX.md)** - Complete documentation index with status flags
- **[docs/PROJECT_SUMMARY.md](./docs/PROJECT_SUMMARY.md)** - Detailed architecture overview
- **[docs/QUICK_REFERENCE.md](./docs/QUICK_REFERENCE.md)** - Commands, patterns, and common tasks
- **[docs/INSTALLING.md](./docs/INSTALLING.md)** - Installation instructions
- **[docs/TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md)** - Common issues and solutions
- **[docs/GUIDELINES_RUST_CODING.md](./docs/GUIDELINES_RUST_CODING.md)** - Rust coding standards
- **[docs/GUIDELINES_GIT_COMMITS.md](./docs/GUIDELINES_GIT_COMMITS.md)** - Git commit conventions

## Distribution & Releases

### GitHub Releases
Tagged releases (`v*`) trigger automated builds for all platforms:
- Linux x86_64 (musl): Headless and GUI binaries
- Linux aarch64 (musl): Headless binary
- macOS x86_64 and aarch64: GUI binaries
- Windows x86_64: GUI binary

All artifacts include SHA256 checksums for verification.

### Snap Store
Linux builds (x86_64, aarch64, armv7) are automatically published to the Snap Store:
```bash
sudo snap install dure
```

### Google Play Store
Android builds are automatically published to Google Play Store after successful desktop builds.

### Package Names
- **Android**: `pe.nikescar.dure`
- **Snap**: `dure`

## Configuration

- **Location**: Project uses egui persistence for settings
- **Internationalization**: Fluent-based i18n with system locale detection
- **Logging**: env_logger (desktop), android_logger (Android)

## Testing

```bash
# Run unit tests
cargo test

# Run with logging
RUST_LOG=debug cargo test

# Platform-specific tests
cargo test --target x86_64-unknown-linux-gnu
```

## Known Limitations

1. **No iOS support yet** - Android only for mobile platforms
2. **WASM deployment** - Build process not fully automated in CI/CD
3. **Linux aarch64 GUI** - Only headless builds available (no GUI dependencies in CI)
4. **Documentation** - Some docs still reference old project name (beads_rust)
5. **Payment integration** - Portone and KakaoPay integration in progress
6. **Requires nightly** - Project depends on nightly Rust features

## Comparison with Traditional E-commerce

| Feature | Dure | Shopify | Wix | Magento |
|---------|------|---------|-----|---------|
| **Hosting** | Distributed | Managed | Managed | Self/Cloud |
| **Transaction Fees** | 0% | 2% | 0% | 0% |
| **Setup Time** | Hours | 1-2 days | Hours | Weeks |
| **Payment Options** | Portone, KakaoPay | Many | Limited | Many |
| **Inventory Mgmt** | Excellent | Good | Basic | Excellent |

## For AI Assistants

When working with this codebase:

1. **Start with this file (CLAUDE.md)** - Provides complete project context
2. **Read [docs/INDEX.md](./docs/INDEX.md)** - Know which docs are valid vs. need review
3. **Check [docs/PROJECT_SUMMARY.md](./docs/PROJECT_SUMMARY.md)** - Deep architecture details
4. **Use [docs/QUICK_REFERENCE.md](./docs/QUICK_REFERENCE.md)** - Fast lookups for commands and patterns
5. **Ignore docs marked with ⚠️** - These reference a different project (beads_rust)
6. **Focus on `mobile/src/`** - This is where the actual application code lives (despite the directory name)
7. **Check `.github/workflows/`** - For build configurations and CI/CD processes
8. **Remember**: This project requires **Rust nightly** and prefers **musl** builds on Linux

## Contributing

See CODE_OF_CONDUCT.md and CREDITS.md for contribution guidelines.

## Security

See SECURITY.md for reporting security issues.

## License

Dual-licensed under MIT OR Apache-2.0. See LICENSE-MIT and LICENSE-Apache-2.0.
