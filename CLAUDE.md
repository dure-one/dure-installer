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

### Application Architecture

The project follows a layered architecture with platform-specific entry points and shared business logic:

#### Entry Points (`mobile/src/`)
- **`main.rs`** - Desktop launcher (detects CLI/GUI/Tray mode automatically)
- **`main_android.rs`** - Android native-activity entry point
- **`main_wasm.rs`** - WebAssembly entry point for browser deployment
- **`lib.rs`** - Library root with conditional compilation for all platforms

#### Core Business Logic (`mobile/src/calc/`)
Business logic controller layer (API → calc → DB → UI):
- **`db.rs`** - Database operations and query interface
- **`crypt.rs`** - Cryptographic operations (encryption, signing, key management)
- **`dns.rs`** - DNS record management logic
- **`gcp.rs`** / **`gcp_rest.rs`** - Google Cloud Platform integrations
- **`platform.rs`** / **`platform_gcp.rs`** - Cloud platform management
- **`acme.rs`** - DNS provider utilities (internal only, no CLI exposure)
- **`hosting_gcp.rs`** - GCP VM regeneration utilities (internal only, no CLI exposure)
- **`ns.rs`** - Nameserver configuration
- **`site.rs`** - Static site generation
- **`keyring.rs`** - Secure key storage
- **`session.rs`** - User session management
- **`audit.rs`** - Security audit and logging

#### Data Layer (`mobile/src/storage/`)
- **`diesel_schema.rs`** - Database schema definitions (Diesel ORM)
- **`models.rs`** - Data models and structs

#### External Integrations (`mobile/src/api/`)
- **`desktop.rs`** - Desktop-specific API utilities
- **`ehttp_cache.rs`** - HTTP caching layer
- **`gcp/`** - Google Cloud Platform API modules (layered architecture):
  - `mod.rs` - Common GCP client (GcpRestClient) and utilities
  - `compute.rs` - Compute Engine API (VMs, firewalls, regions/zones)
  - `resourcemanager.rs` - Resource Manager API (projects)
  - `billing.rs` - Cloud Billing API (billing accounts, project billing)
  - `bigquery.rs` - BigQuery API (datasets, tables, billing queries)
  - `serviceusage.rs` - Service Usage API (enable/check services)
  - `oauth.rs` - OAuth2 authentication and user info
  - `dns.rs` - Cloud DNS API (managed zones, DNS records)
- **`ns_cloudflare.rs`** - Cloudflare DNS API
- **`ns_gcp.rs`** - GCP Cloud DNS API (re-export of `gcp/dns.rs`)
- Additional DNS providers: DuckDNS (`ns_duckdns.rs`), Porkbun (`ns_porkbun.rs`)

#### User Interface (`mobile/src/` - feature-gated with `gui`)
- **`dure.rs`** - Main eframe application (cross-platform GUI)
- **`dure_stt.rs`** - Application state management
- **`ui_tabs/`** - Tab-based navigation components:
  - `platform.rs` - Cloud platform management UI
  - `ssh.rs` - SSH host management UI
  - `ns.rs` - DNS nameserver UI
  - `site.rs` - Website hosting UI
  - `products.rs` / `orders.rs` - E-commerce management UI
  - `channel.rs` / `dm.rs` / `members.rs` - Messaging UI
  - `client.rs` - Store client UI
  - `email.rs` - Email integration UI
  - `roles.rs` - User role management UI
- **`ui_dlg/`** - Dialog and popup components:
  - `settings.rs` - Settings dialog
  - `about.rs` - About dialog
  - `clipboard_popup.rs` - Clipboard utilities
  - `platform_gcp.rs` - GCP-specific dialogs
- **`tray.rs`** - System tray integration (Windows/Linux/macOS, not OpenBSD)

#### Command-Line Interface (`mobile/src/cli/`)
- **`mod.rs`** - CLI argument parser (clap-based)
- **`commands/`** - Command implementations for headless operation

#### Platform-Specific Modules
- **`android/`** - Android utilities:
  - `activity.rs` - Activity lifecycle management
  - `clipboard.rs` - Android clipboard integration
  - `log.rs` - Android logging (logcat)
  - `inputmethod.rs` - Soft keyboard handling
- **`install.rs`** / **`install_stt.rs`** - Desktop installation/deployment
- **`http_server.rs`** - OAuth callback server (darkhttpd on Unix, winhttpd on Windows)

#### Supporting Modules
- **`config.rs`** - YAML-based configuration management
- **`i18n.rs`** - Fluent-based internationalization (system locale detection)
- **`attestation/`** - Binary verification with GitHub Sigstore
- **`site/`** - Static site generation and asset management
- **`log_capture.rs`** - In-memory log buffer for GUI display
- **`asyncapi_spec.rs`** - API documentation generation

#### Design Principles
1. **Platform Abstraction** - Conditional compilation (`#[cfg(...)]`) isolates platform-specific code
2. **Feature Flags** - GUI optional (`--no-default-features` for headless builds)
3. **Layered Architecture** - Clear separation: UI → ViewModel → Calc/Api
   - UI layer only communicates with ViewModel
   - ViewModel coordinates between Calc (business logic) and Api (external services)
   - Api layer is modular (e.g., `gcp/compute.rs`, `gcp/billing.rs`)
4. **Async Runtime** - smol for async operations (network, I/O)
5. **Security First** - Attestation, encryption, secure key storage
6. **Dual Interface** - All features accessible via CLI and GUI

## Distributed E-Commerce Architecture

Dure provides a **directory-based e-commerce model** where shop owners deploy their own mycart instances and register with the dure.one directory service for discoverability.

### Architecture Overview

```
                    ┌──────────────────┐
                    │  dure.one        │
                    │  Directory       │
                    │  Service         │
                    └────────┬─────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
        ┌─────▼────┐   ┌────▼─────┐  ┌────▼─────┐
        │ mycart A │   │ mycart B │  │ mycart C │
        │ (Owner)  │   │ (Owner)  │  │ (Owner)  │
        └──────────┘   └──────────┘  └──────────┘
```

**Key Components:**

| Component | Purpose | Details |
|-----------|---------|---------|
| **dure.one** | Shop directory service | Central registry for shop discovery |
| **mycart** | Individual shop backend | Owner-controlled e-commerce API |
| **dure-responder** | Auto-assigned identifier | Unique number for each mycart instance |
| **dure-sijang** | Shop discovery flow | Browse dure.one → visit mycart website |

### How It Works

1. **Shop Registration**: mycart owners register their shop on dure.one directory service
2. **Discovery**: Guests browse dure.one to find shops
3. **Shopping**: Each mycart has its own:
   - **dure-responder (auto) number** - Unique identifier
   - **mycart API endpoint** - Independent backend service
   - **Website** - Shop-specific storefront
4. **Data Ownership**: Each shop maintains complete control over products, orders, and customer data

### mycart Deployment Options

**Current Development Status**: 
- ✅ **GCP VM** - In development
- ✅ **Supabase + GCP VM** - In development
- 🔜 **Supabase + BYOS (cloudflared)** - Planned
- 🔜 **Firebase + Firestore** - Planned

#### Deployment Types Feature Matrix

| Feature | Firebase + Firestore | Supabase + GCP VM | Supabase + BYOS (cloudflared) | GCP VM | Others |
|---------|---------------------|-------------------|------------------------------|---------|--------|
| **Backend (API providers)** | GCP Cloud Functions | mycart backend | mycart backend | mycart backend | swagger.json |
| **Auth Base** | Firebase Auth | Go Fiber JWT | Go Fiber JWT | Go Fiber JWT | Admin auth only |
| **Auth Method** | Google login | Email + password | Email + password | Email + password | Admin auth only |
| **Frontend** | Firebase Hosting | nginx | nginx | nginx | Svelte5 SvelteKit2 |
| **CDN** | Firebase Hosting | Cloud CDN | Orange Cloud (CDN) | Cloud CDN | - |
| **Backup/Restore** | Firebase scheduled backup | Supabase backup | Supabase backup | SQLite files | - |
| **Contents Security** | Firebase security rules | JWT permission | JWT permission | JWT permission | - |
| **DDoS Mitigation** | Firebase | fail2jail-rs | fail2jail-rs | fail2jail-rs | XMPP integration |
| **XMPP Service** | - | - | Prosody | Prosody | XMPP integration |
| **Service Healthcheck Provider** | - | - | - | - | - |
| **XMPP Integration** | GCP Cloud Functions | GCP Cloud Functions | Golang | Golang | - |
| **SMS Integration** | GCP Cloud Functions | GCP Cloud Functions | Golang | Golang | - |
| **Kakotalk Integration** | GCP Cloud Functions | GCP Cloud Functions | Golang | Golang | - |
| **Telegram Integration** | GCP Cloud Functions | GCP Cloud Functions | Golang | Golang | - |
| **Jurisdiction** | Seoul | Seoul + Seoul | Seoul | Seoul | - |

**BYOS**: Bring Your Own Server (self-hosted behind Cloudflare Tunnel)

### Deployment Characteristics

**Common Features:**
- **TLS**: Automatic ACME certificates (Let's Encrypt)
- **Scale**: Optimized for small shops (100s-1000s of products)
- **Payment**: Portone, KakaoPay integration
- **Authentication**: OAuth 2.0 (Kakao/Naver/Google) or Email/Password

**GCP VM Deployment:**
- Single VM instance per shop
- SQLite or PostgreSQL backend
- Go Fiber API server
- nginx frontend
- fail2jail-rs for DDoS protection
- Optional Prosody XMPP service

**Supabase + GCP VM:**
- Supabase for auth and database
- GCP VM for additional services
- Automated backup via Supabase
- JWT-based permission system

### Claude Usage Habits
* Use Inline Execution in claude pro plan, Use Subagent Driven Execution in claude max plan
* Create feature branch only with superpower plans

## Core Features

All function exists for both EGUI and CLI. 

### 1. Identity Management
- Private/Public key for personal identity
- Platform(GCP), SSH Host management
- Attestation for WASM/EGUI apps with GitHub Sigstore

### 2. Platform Management (platform)
- GCP management (Add/Del VM,View Billing)

### 3. DNS Management (ns)
- DNS management (Add/Del domain, Add/Del Txt Record)
- Supports Cloudflare, Google Cloud DNS, DuckDNS, Porkbun

### 4. SSH Host Management (ssh)
- Automatically added host from Platform Management
- Docker Management (Add/Del docker host from dockerhub)
- Ansible Management (Add/Del ansible roles from ansiblegalaxy)
- System Hardener (using Jangbi project)
- Automatic key Management

### 5. Store Management (EGUI/CLI)
- Promotions
- Products
- Orders
- Shipments
- Accounts
- Dure (shared listings/shipments with other stores)

### 6. Guest Front (WASM)
- Minimum guest identity for customers
- Product listings
- Shopping cart
- Payment using Portone and Kakaopay
- Login with Kakao/Naver/Google Oauth

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
