# Dure - Distributed E-commerce Platform

## Project Overview

**Dure** is a distributed e-commerce client and hosting solution built with Rust and egui. It enables small shop owners to run e-commerce operations without traditional centralized server infrastructure.

### Key Characteristics

- **Language**: Rust (2021 edition, min version 1.81)
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

### 1. Identity Management
- Private/Public key for personal identity
- Firebase/Supabase identity integration
- Attestation for WASM/EGUI apps with GitHub Sigstore

### 2. Guest Front (WASM)
- Minimum guest identity for customers
- Product browsing and cart functionality

### 3. Store Front (WASM)
- Product listings
- Shopping cart
- Payment integration (Portone, KakaoPay)

### 4. Hosting Management (EGUI)
- DNS management (octodns)
- Database setup (Firebase, Supabase)
- Site deployment (Firebase/Supabase Cloud Functions)

### 5. Store Management (EGUI)
- Promotions
- Products
- Orders
- Shipments
- Accounts
- Dure (shared listings/shipments with other stores)

## Platform Support

| Platform | Status | Features |
|----------|--------|----------|
| **Linux** | ✅ Supported | Full EGUI client (X11, Wayland) |
| **Windows** | ✅ Supported | Full EGUI client |
| **macOS** | ✅ Supported | Full EGUI client |
| **Android** | ✅ Supported | EGUI client via native-activity |
| **WASM** | ✅ Supported | Guest & Store Front only |

## Technology Stack

### Core Dependencies
- **UI**: egui 0.33, eframe 0.33, egui-material3
- **i18n**: egui-i18n with Fluent
- **Database**: Diesel 2.2, diesel_migrations 2.2 (SQLite + optional PostgreSQL)
- **Async**: tokio (multi-threaded runtime)
- **HTTP**: ureq, ehttp
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

#### OpenBSD
OpenBSD requires OpenSSL 3.5+ (uses eopenssl35 instead of LibreSSL):

**Option 1: Using direnv (recommended)**
```bash
# Install direnv
pkg_add direnv

# Add to ~/.bashrc
eval "$(direnv hook bash)"

# Allow direnv in this project
cd /path/to/dure
direnv allow
```

**Option 2: Manual environment variables**
```bash
# Add to ~/.bashrc or ~/.profile
export OPENSSL_LIB_DIR="/usr/local/lib/eopenssl35"
export OPENSSL_INCLUDE_DIR="/usr/local/include/eopenssl35"
```

### Desktop
```bash
# Debug build
cargo build

# Release build
cargo build --release

# Dev-release (faster iteration)
cargo build --profile dev-release
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

## Documentation

See [`docs/`](./docs/) directory for detailed documentation:

- **[docs/INDEX.md](./docs/INDEX.md)** - Complete documentation index with status flags
- **[docs/PROJECT_SUMMARY.md](./docs/PROJECT_SUMMARY.md)** - Detailed architecture overview
- **[docs/QUICK_REFERENCE.md](./docs/QUICK_REFERENCE.md)** - Commands, patterns, and common tasks
- **[docs/INSTALLING.md](./docs/INSTALLING.md)** - Installation instructions
- **[docs/TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md)** - Common issues and solutions
- **[docs/GUIDELINES_RUST_CODING.md](./docs/GUIDELINES_RUST_CODING.md)** - Rust coding standards
- **[docs/GUIDELINES_GIT_COMMITS.md](./docs/GUIDELINES_GIT_COMMITS.md)** - Git commit conventions

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

1. No iOS support yet (Android only for mobile)
2. WASM deployment workflow not fully documented
3. Some documentation needs updating to match Dure (not beads)
4. Payment gateway integration in progress

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

## Contributing

See CODE_OF_CONDUCT.md and CREDITS.md for contribution guidelines.

## Security

See SECURITY.md for reporting security issues.

## License

Dual-licensed under MIT OR Apache-2.0. See LICENSE-MIT and LICENSE-Apache-2.0.
