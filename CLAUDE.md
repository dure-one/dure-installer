# Dure - Distributed E-commerce Platform

## Project Overview

**Dure Installer** enables small shop owners to deploy and manage their own mycart backend instances (distributed e-commerce). Provides infrastructure automation for GCP VMs, DNS, SSH, and store management.

**Separate project (dure-sijang)**: Customer shopping app (out of scope)

### Key Characteristics

- **Language**: Rust nightly (required)
- **UI Framework**: egui + eframe (Material3)
- **Platforms**: Desktop (Linux/macOS/Windows), Android, WASM
- **Architecture**: Layered (UI → ViewModel → Calc/Api) with dual CLI/GUI interface
- **License**: Dual MIT/Apache-2.0

## Architecture

**Directory Structure**: See docs/PROJECT_SUMMARY.md for details

**Design Principles**:
1. Platform abstraction via `#[cfg(...)]`
2. GUI optional (`--no-default-features` for headless)
3. Layered: UI → ViewModel → Calc (business logic) / Api (external services)
4. smol async runtime (no tokio, no OpenSSL)
5. Security first: attestation, encryption, secure key storage

**Key Modules** (`mobile/src/`):
- **Entry points**: `main.rs` (desktop), `main_android.rs`, `main_wasm.rs`, `lib.rs`
- **Business logic** (`calc/`): db, crypt, dns, gcp, platform, ns, site, session, audit
- **External APIs** (`api/`): GCP modules (compute, billing, dns, oauth), DNS providers (Cloudflare, DuckDNS, Porkbun)
- **UI** (`ui_tabs/`, `ui_dlg/`): platform, ssh, ns, site, products, orders, settings (feature-gated `gui`)
- **CLI** (`cli/`): Headless command implementations

### Claude Usage Habits
* Use Inline Execution (Pro plan) or Subagent Driven (Max plan)
* Create feature branches only with superpower plans

## Core Features (CLI + GUI)

1. **Identity**: Private/public keys, attestation (GitHub Sigstore)
2. **Platform**: GCP VM management, billing
3. **DNS**: Domain/TXT record management (Cloudflare, GCP DNS, DuckDNS, Porkbun)
4. **SSH**: Host management, Docker/Ansible, system hardening (Jangbi)
5. **Store**: Products, orders, shipments, promotions
6. **Guest** (WASM only): Shopping cart, payment (Portone/KakaoPay), OAuth login

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

**Core**: egui 0.33, Diesel 2.3 (SQLite/PostgreSQL), smol 2.0 (async), ureq 2.12, rustls 0.23, russh 0.45 (pure Rust, **no OpenSSL**)  
**Platform-specific**: Android (ndk-context, jni), Desktop (tray-icon, dotenvy), WASM (wasm-bindgen, sqlite-wasm-rs)

## Development

**Build profiles**: `dev` (debug), `release` (LTO + size opt), `dev-release` (balanced)  
**Guidelines**: Rust 2021 idioms, clippy::pedantic, minimize unsafe, cross-platform compat  
See docs/GUIDELINES_RUST_CODING.md for details

## Building

**Prerequisites**: Rust nightly (`rustup default nightly`)

**Quick Start**:
```bash
cargo build                        # Debug
cargo build --release              # Release
cargo build --profile dev-release  # Dev-release
cd mobile && ./build.sh            # Android
```

**Headless** (no GUI): `cargo build --no-default-features`

**Production builds**: Linux (musl x86_64/aarch64), macOS (x86_64/aarch64), Windows (x86_64)  
See `.github/workflows/release.yml` and docs/INSTALLING.md for platform-specific build commands

## Testing

```bash
cargo test                                # Unit tests
RUST_LOG=debug cargo test                 # With logging
```

## Distribution

- **GitHub Releases**: Linux (x86_64/aarch64 musl), macOS (x86_64/aarch64), Windows (x86_64) - auto-build on `v*` tags
- **Snap Store**: `sudo snap install dure`
- **Google Play**: `app.dure.installer`

## Known Limitations

1. No iOS support (Android only)
2. WASM deployment not fully automated
3. Linux aarch64 GUI unavailable (headless only)
4. Some docs reference old project name (beads_rust)
5. Requires nightly Rust

## Documentation

**Start here**:
- **[docs/INDEX.md](./docs/INDEX.md)** - Documentation index with status flags
- **[docs/PROJECT_SUMMARY.md](./docs/PROJECT_SUMMARY.md)** - Detailed architecture
- **[docs/QUICK_REFERENCE.md](./docs/QUICK_REFERENCE.md)** - Commands and patterns
- **[docs/INSTALLING.md](./docs/INSTALLING.md)** - Installation guide
- **[docs/TROUBLESHOOTING.md](./docs/TROUBLESHOOTING.md)** - Common issues
- **[docs/GUIDELINES_RUST_CODING.md](./docs/GUIDELINES_RUST_CODING.md)** - Rust standards
- **[docs/GUIDELINES_GIT_COMMITS.md](./docs/GUIDELINES_GIT_COMMITS.md)** - Git conventions

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
