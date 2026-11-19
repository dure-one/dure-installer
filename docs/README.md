# Dure Documentation

Welcome to the Dure documentation directory.

## Quick Links

- **[../CLAUDE.md](../CLAUDE.md)** - Complete project guide
- **[ARCH.sudolang.md](./ARCH.sudolang.md)** - Architecture summary (SudoLang format)
- **[INDEX.md](./INDEX.md)** - Complete documentation index
- **[CLI_REFERENCE.md](./CLI_REFERENCE.md)** - CLI commands reference
- **[TROUBLESHOOTING.md](./TROUBLESHOOTING.md)** - Common issues and solutions

## What is Dure?

Dure is a distributed e-commerce platform installer that lets small shop owners deploy and manage mycart backend infrastructure. It automates:

- **GCP VM provisioning** - Compute instances, billing, networking
- **DNS management** - Cloudflare, GCP DNS, DuckDNS, Porkbun
- **SSH operations** - Host configuration, Docker deployment, system hardening
- **Store management** - Products, orders, shipments (GUI only)

## Architecture

Dure uses a layered MVVM architecture:
- **UI Layer**: egui Material3 (Desktop/Android/WASM)
- **ViewModel Layer**: Actor-based with smol async runtime
- **Business Logic (Calc)**: Profile-scoped operations
- **API Layer**: External integrations (GCP, DNS providers)

See **[ARCH.sudolang.md](./ARCH.sudolang.md)** for complete architecture details.

## Usage Modes

### GUI Mode (Default)
```bash
dure
```
Launches egui desktop application with Material3 design.

### CLI Mode
```bash
dure <command> [options]
```
Headless command-line interface. See **[CLI_REFERENCE.md](./CLI_REFERENCE.md)**.

### Platforms
- **Desktop**: Linux (x86_64/aarch64), macOS (x86_64/aarch64), Windows (x86_64)
- **Android**: Native activity via Google Play
- **WASM**: Browser-based (guest shopping only)

## Key Features

### Identity & Security
- Ed25519 key pairs for authentication
- GitHub Sigstore attestation for binary verification
- Platform-specific secure keyring storage
- End-to-end encryption for site-to-site communication

### Platform Management
- GCP VM lifecycle management
- Billing monitoring and cost tracking
- Static IP allocation
- Firewall rule configuration

### DNS Management
- Multi-provider support (Cloudflare, GCP DNS, DuckDNS, Porkbun)
- Record management (A, AAAA, TXT, NS)
- Automatic TXT record updates for bastion hosts
- Cached DNS lookups

### SSH Operations
- Host configuration and key management
- Automated server initialization (swap, nftables, dure server)
- Docker container deployment
- System hardening

## For AI Assistants

When working with this codebase:

1. Read **[../CLAUDE.md](../CLAUDE.md)** first for complete project context
2. Read **[ARCH.sudolang.md](./ARCH.sudolang.md)** for architecture details
3. Check **[INDEX.md](./INDEX.md)** to know which docs are valid
4. Focus on actual code in `mobile/src/` for understanding current implementation
5. Documents marked with **WARNING** may reference beads_rust (different project)

## Technology Stack

- **Language**: Rust nightly (required)
- **UI**: egui 0.33, eframe, egui_material3
- **Async**: smol 2.0 (not tokio)
- **Database**: Diesel 2.3 (SQLite + PostgreSQL)
- **HTTP**: ureq 2.12
- **TLS**: rustls 0.23 (no OpenSSL)
- **SSH**: russh 0.45 (pure Rust)

## Contributing

Help improve documentation:
- Fix errors or unclear explanations
- Add examples and tutorials
- Document missing features
- Update obsolete content

See **[../CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md)** for contribution guidelines.

## Documentation Status

See **[INDEX.md](./INDEX.md)** for complete list of available documentation and status of each document.
