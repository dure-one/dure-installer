# Dure Project Summary

## What is Dure?

Dure is a **distributed e-commerce platform** that allows small shop owners to host and manage online stores without traditional centralized server infrastructure. It's built with Rust and provides both native (desktop/mobile) and web (WASM) clients.

### Core Value Proposition

- **Zero transaction fees** (unlike Shopify's 2%)
- **Fast setup** (hours, not days)
- **Distributed architecture** (reduces hacking risk)
- **Cross-platform** (Linux, Windows, macOS, Android, WASM)
- **AI-ready** (all operations can be modified by LLM)

## Architecture Overview

### Three Client Types

```
┌─────────────────────────────────────────────────────┐
│                   Dure Platform                     │
├─────────────────────────────────────────────────────┤
│                                                     │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────┐  │
│  │ EGUI Client  │  │ WASM Client  │  │  Guest   │  │
│  │ (Native Apps)│  │ (Web Front)  │  │ Frontend │  │
│  └──────────────┘  └──────────────┘  └──────────┘  │
│         │                  │               │        │
│    ┌────┴─────┬───────────┴───────┬───────┴────┐   │
│    │          │                   │            │   │
│    v          v                   v            v   │
│  Identity   Hosting             Store       Guest  │
│  Mgmt       Mgmt                Mgmt        Browse │
│                                                     │
│  - Keys     - DNS (octodns)     - Products  - Cart │
│  - Attest   - DB (FB/SB)        - Orders    - Pay  │
│  - Auth     - CloudFunctions    - Shipping        │
└─────────────────────────────────────────────────────┘
```

### Component Breakdown

#### 1. **EGUI Client** (Desktop + Android)
Full-featured native application with:
- Identity management (keys, attestation)
- Hosting setup (DNS, database, deployment)
- Store management (products, orders, shipments)
- Analytics (DataFusion + Arrow)
- System tray integration (desktop only)

**Code**: `mobile/src/` (despite the name, handles all platforms)

#### 2. **WASM Client** (Web)
Web-based store front with:
- Product listings
- Shopping cart
- Payment processing
- Basic identity management
- No hosting/admin features

**Target**: `wasm32-unknown-unknown`

#### 3. **Guest Frontend** (WASM)
Minimal customer-facing interface:
- Browse products
- Add to cart
- Checkout and pay
- Minimal identity (guest accounts)

## Directory Structure Explained

### `/mobile/`
**Main application code** (despite the name)

```
mobile/
├── src/              # All Rust source code
├── app/              # Android-specific (Gradle, manifest)
├── assets/           # Images, icons, resources
├── resources/        # i18n translations (Fluent files)
├── tests/            # Integration tests
├── build.sh          # Android build script
├── build.rust-only.sh # Rust lib only (fast iteration)
└── Cargo.toml        # Package dependencies
```

Key files in `src/`:
- `main.rs` - Desktop entry point
- `lib.rs` - Library + Android entry point
- UI modules for different screens
- Business logic for e-commerce operations

### `/docs/`
Documentation directory - **contains mixed content**:
- ✅ Rust coding guidelines (valid)
- ⚠️ Architecture docs from different project (needs updating)
- ⚠️ Testing docs from "beads" project (not relevant)

See [INDEX.md](./INDEX.md) for detailed breakdown.

### `/deploy/`
Deployment configurations for various platforms.

### `/fastlane/`
Mobile CI/CD automation (for Android builds and releases).

### `/snap/`
Snapcraft configuration (Linux snap packages).

### `/scripts/`
Build and utility scripts.

### `/.github/`
GitHub Actions workflows for CI/CD.

## Technology Decisions

### Why Rust?
- **Memory safety** without garbage collection
- **Cross-platform** compilation (native + WASM)
- **Performance** for data processing (DataFusion)
- **Mobile support** via android-activity

### Why egui?
- **Immediate mode** UI - simple, fast
- **Cross-platform** - same code everywhere
- **Native + WASM** - one codebase, all platforms
- **Material3** design via egui-material3

### Why DataFusion?
- **SQL analytics** on local data
- **Parquet files** for efficient storage
- **No server needed** - runs in-app
- **Async processing** with Tokio

### Why Firebase/Supabase?
- **No backend** needed
- **Cloud Functions** for WASM hosting
- **Authentication** built-in
- **Globally distributed**

## Data Flow Example: Order Processing

```
1. Customer (Guest WASM)
   └─> Browse products → Add to cart → Checkout
         │
         v
2. Payment Gateway (Portone/KakaoPay)
   └─> Process payment → Return confirmation
         │
         v
3. Store Owner (EGUI Native)
   └─> Receive order notification
   └─> Process order → Update status
   └─> Arrange shipment → Mark shipped
         │
         v
4. Shared Dure Network
   └─> Sync with other stores (if using Dure shipment)
```

## Development Workflow

### Initial Setup
```bash
# Clone repository
git clone https://github.com/dure-one/dure-installer.git
cd dure

# Build desktop version
cargo build --release

# Run
./target/release/dure
```

### Android Development
```bash
cd mobile

# Full Android build (APK + Rust)
./build.sh

# Fast Rust-only iteration
./build.rust-only.sh

# Test in emulator
./qemu.sh
```

### WASM Development
```bash
# Build WASM (instructions TBD)
cargo build --target wasm32-unknown-unknown
```

## Key Differences from Similar Projects

### vs. Shopify
- **No transaction fees** (Shopify: 2%)
- **Self-hosted** (Shopify: fully managed)
- **Distributed** (Shopify: centralized)
- **Fewer themes** (Shopify: 100+)

### vs. Wix
- **Better inventory** (Wix: basic)
- **More payment options** (Wix: limited)
- **Native mobile app** (Wix: web-based)

### vs. Magento
- **Faster setup** (Magento: weeks)
- **Simpler** (Magento: enterprise complexity)
- **No server needed** (Magento: requires hosting)

## Current Status

### ✅ Implemented
- Cross-platform build system
- EGUI application framework
- i18n support
- DataFusion analytics integration

### 🚧 In Progress
- E-commerce features (products, orders)
- Payment gateway integration
- Store management UI
- Hosting automation

### 📋 Planned
- iOS support (currently Android only)
- WASM deployment workflow
- Complete payment integrations
- Multi-store Dure network
- Mobile POS features

## Common Misconceptions

❌ **"This is a Rust port of beads issue tracker"**
- No, some docs were copied from beads but Dure is an e-commerce platform

❌ **"Mobile folder only contains mobile code"**
- No, it contains ALL application code (desktop + mobile + WASM lib)

❌ **"Requires a backend server"**
- No, uses Firebase/Supabase for cloud features

❌ **"Only works on Android"**
- No, works on Linux, Windows, macOS, Android, and WASM

## For AI Assistants

When asked to work on Dure:

1. **Read CLAUDE.md first** - Complete project context
2. **Then read this file** - Detailed architecture overview
3. **Check docs/INDEX.md** - Know which docs are valid
4. **Ignore beads references** - Different project
5. **Focus on mobile/src/** - Actual codebase
6. **Check Cargo.toml** - Current dependencies
7. **Respect platform conditionals** - Different targets need different deps

## Quick Reference

| Task | Command | Location |
|------|---------|----------|
| Build desktop | `cargo build` | Root |
| Build Android | `cd mobile && ./build.sh` | mobile/ |
| Run tests | `cargo test` | Root |
| Check clippy | `cargo clippy --all-targets` | Root |
| Format code | `cargo fmt` | Root |
| Build docs | `cargo doc --open` | Root |
| Add dependency | Edit `Cargo.toml` | Root or mobile/ |

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/dure-one/dure-installer/issues)
- **Code of Conduct**: See CODE_OF_CONDUCT.md
- **Security**: See SECURITY.md
- **Contributing**: See CREDITS.md
