# Dure Installer

**Mycart & Prosody installer and manager for small shop owners**

Automate deployment, configuration, and management of your e-commerce backend infrastructure on GCP with integrated DNS, SSH, and security management.

---

## 🌐 Dure Ecosystem

| Project | Purpose |
|---------|---------|
| **[dure-installer](https://github.com/dure-one/dure-installer)** | Mycart & Prosody installer & manager (this project) |
| **[dure-responder](https://github.com/dure-one/dure-responder)** | Auto-responder for customer requests via workflow automation |
| **[dure.one](https://dure.one)** | Business directory listing service |
| **[dure-sijang](https://github.com/dure-one/dure-sijang)** | Customer shopping client for Dure ecosystem |

---

## ✨ What It Does

**dure-installer** automates infrastructure setup and management for shop owners:

### 🚀 Core Capabilities

1. **Automatic Server Deployment**
   - Deploy VMs to GCP Compute Engine
   - Configure firewall rules automatically
   - Manage IP addresses and billing

2. **Application Installation**
   - Install mycart backend (REST API + admin panel)
   - Install Prosody XMPP server
   - Configure nginx, fail2ban-rs, nft firewall

3. **Domain & DNS Management**
   - Register domains via Cloudflare
   - Configure DNS records (A, AAAA, TXT)
   - Support for Cloudflare, GCP DNS, DuckDNS, Porkbun

4. **Security Hardening**
   - GCE firewall rule management
   - nft (nftables) configuration
   - fail2ban-rs intrusion prevention
   - SSH key management with keyring integration

5. **Store Operations**
   - View store statistics and health
   - Manage crontab (jobs, intervals, enable/disable)
   - Fetch logs (duremycart, xmppproxy, fail2ban-rs)
   - Monitor services remotely

---

## 📚 Documentation

**Start here:**
- **[CLAUDE.md](./CLAUDE.md)** - Complete project guide for developers and AI assistants
- **[docs/ARCH.sudolang.md](./docs/ARCH.sudolang.md)** - Architecture summary (layers, actors, data flow)
- **[docs/INDEX.md](./docs/INDEX.md)** - Complete documentation index with status flags
- **[docs/QUICK_REFERENCE.md](./docs/QUICK_REFERENCE.md)** - Commands, patterns, common tasks

---

## 🏗️ Architecture

**Tech Stack:**
- **Language**: Rust nightly (required)
- **UI Framework**: egui + eframe (Material3 theme)
- **Async Runtime**: smol 2.0 (no tokio, no OpenSSL)
- **Database**: Diesel ORM (SQLite, optional PostgreSQL)
- **SSH**: russh 0.45 (pure Rust)
- **HTTP**: ureq 2.12
- **TLS**: rustls 0.23

**Platforms:**
- ✅ **Linux**: x86_64 (GUI + CLI), aarch64 (CLI only)
- ✅ **macOS**: x86_64 & Apple Silicon
- ✅ **Windows**: x86_64
- ✅ **Android**: Native EGUI app
- ✅ **WASM**: Browser-based (guest features only)

**MVVM Architecture:**
- **View**: egui UI (Desktop/Android), WASM (Browser), CLI (Headless)
- **ViewModel**: Actor-based async coordinator (smol runtime)
- **Model**: Business logic (calc layer), Database (Diesel), External APIs

**Actors** (implemented):
- ✅ **Platform**: GCP VM management + billing
- ✅ **SSH**: Host/container management + operations drawer
- ✅ **Logs**: Unified operation logging for all tabs
- 📋 **Mycart** (planned): Backend API + SSH log fetching
- 📋 **DNS** (planned): Cloudflare DNS management

---

## 🚦 Quick Start

### Prerequisites

- Rust nightly: `rustup default nightly`
- Target platform toolchain (for cross-compilation)

### Installation

#### From GitHub Releases (Recommended)

Download prebuilt binaries for Linux (x86_64/aarch64 musl), macOS (x86_64/aarch64), or Windows (x86_64):

```bash
# Linux x86_64 example
wget https://github.com/dure-one/dure-installer/releases/latest/download/dure-linux-x86_64
chmod +x dure-linux-x86_64
sudo mv dure-linux-x86_64 /usr/local/bin/dure
```

#### From Snap Store

```bash
sudo snap install dure
```

#### From Source

```bash
git clone https://github.com/dure-one/dure-installer.git
cd dure-installer
cargo build --release
```

For headless servers (no GUI):

```bash
cargo build --release --no-default-features
```

### First Run

```bash
# GUI mode (default)
dure

# CLI mode
dure profile create mystore mypassword
dure profile login mystore mypassword
dure platform status
```

---

## 💻 CLI Usage

### Profile Management

Profiles scope all operations (DB, files, credentials):

```bash
# Create profile
dure profile create <name> <password>

# Login to profile
dure profile login <name> <password>

# List profiles
dure profile list

# Delete profile
dure profile delete <name>
```

### Platform Operations (GCP)

```bash
# List VMs
dure platform status

# Add GCP platform configuration
dure platform add <name>

# Initialize platform (OAuth, project setup)
dure platform init <name>

# Delete platform
dure platform del <name>
```

### DNS Management

#### Domain Registration (Cloudflare)

```bash
# Check availability
dure dns check-availability --account-id <id> example.com

# Register domain
dure dns register-domain --account-id <id> example.com --years 1

# List domains
dure dns list-domains --account-id <id>
```

#### Zone & Record Management

```bash
# List zones
dure dns list-zones

# Create zone
dure dns create-zone example.com

# Add A record
dure dns create-record example.com A www 1.2.3.4

# Add TXT record (SPF, DKIM, etc.)
dure dns create-record example.com TXT @ "v=spf1 include:_spf.example.com ~all"

# Update record
dure dns update-record example.com <record-id> --content 5.6.7.8

# Delete record
dure dns delete-record example.com <record-id>
```

#### Supported DNS Providers

- ✅ **Cloudflare** (full API support)
- ✅ **GCP Cloud DNS**
- ✅ **DuckDNS**
- ✅ **Porkbun**

### SSH Management

```bash
# List SSH hosts
dure ssh status

# Add host (password auth)
dure ssh add <host> --password <password>

# Add host (key auth)
dure ssh add <host> --key-file ~/.ssh/id_ed25519

# Delete host
dure ssh del <host>

# Test connection
dure ssh init <host>
```

### Mycart Operations

Manage mycart backend installations:

```bash
# Add mycart site
dure mycart add <domain> <admin-email> <admin-password> \
  --platform <platform-id> \
  --ssh-host <ssh-host>

# List sites
dure mycart list

# View statistics
dure mycart stats <domain>

# View logs (requires SSH)
dure mycart logs <domain>

# Manage crontab
dure mycart crontab <domain>
```

### Key Management (Keyring)

Secure credential storage using OS keychain/keyring:

```bash
# List stored keys
dure key status

# Add key
dure key add <service> <username> <password>

# Delete key
dure key del <service>

# Export keychain (KeePass format)
dure key save export keychain.kdbx

# Import keychain
dure key load import keychain.kdbx
```

### Audit Logs

```bash
# View audit trail (last 50 actions)
dure audit status

# Clear audit logs
dure audit clear
```

### DNS Client (Lookup)

```bash
# Lookup A records
dure dns a example.com

# Lookup AAAA records
dure dns aaaa example.com

# Lookup TXT records
dure dns txt example.com
```

---

## 🖥️ GUI Usage

Launch GUI mode:

```bash
dure
# or
dure --gui
```

**Features:**
- Material3 theme with responsive design
- Profile management with login/logout
- Platform tab: GCP VM management, billing viewer
- SSH tab: Host management, container operations
- DNS tab: Domain registration, zone/record management
- Mycart tab: Store health, stats, logs, crontab
- Settings tab: Theme, language, API keys
- System tray icon (desktop only)

**Drawer UI Pattern:**
- Operations tab: Real-time operation logs (platform, SSH, mycart)
- Logs tab: Historical operation records with filtering

---

## 📋 Configuration

### Config File Locations

1. **CLI flags** (highest priority)
2. **Environment variables**
3. **Project config**: `.dure/config.yaml`
4. **User config**: `~/.config/dure/config.yaml`
5. **Defaults** (lowest priority)

### Example Configuration

```yaml
# Platform identification
platform:
  name: "mylaptop"

# Domain configuration
domain:
  name: "example.com"
  dns_provider: "cloudflare"  # cloudflare | duckdns | gcloud | porkbun
  
  cert:
    email: "admin@example.com"
    cert_path: "/etc/letsencrypt/live/example.com/fullchain.pem"
    key_path: "/etc/letsencrypt/live/example.com/privkey.pem"
  
  cloudflare:
    api_token: "your_api_token_here"
  
  porkbun:
    api_key: "pk_xxx"
    secret_api_key: "sk_xxx"

# Server configuration
server:
  web_provider: "GCE"  # GCE | VPS | CLOUDFLARE_PAGES | FIREBASE_HOSTING
  db_provider: "GCE"   # GCE | GCP_CLOUDSQL | SUPABASE
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `DURE_DB` | Override database path |
| `RUST_LOG` | Logging level (debug, info, warn, error) |
| `CLOUDFLARE_API_TOKEN` | Cloudflare API token |
| `GCP_PROJECT_ID` | GCP project ID |

---

## 🔧 Development

### Build Profiles

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Dev-release (balanced)
cargo build --profile dev-release

# Android build
cd mobile && ./build.sh
```

### Testing

```bash
# Run all tests
cargo test

# Run with logging
RUST_LOG=debug cargo test

# Run specific test
cargo test test_resolve_ssh_host
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy -- -D warnings

# Check compilation
cargo check
```

---

## 🤝 Contributing

See [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md) and [CREDITS.md](./CREDITS.md).

---

## 🔒 Security

Report security issues via [SECURITY.md](./SECURITY.md).

---

## 📄 License

Dual-licensed under **MIT** OR **Apache-2.0**.

See [LICENSE-MIT](./LICENSE-MIT) and [LICENSE-Apache-2.0](./LICENSE-Apache-2.0).

---

## 🌟 Star History

[![Star History Chart](https://api.star-history.com/svg?repos=dure-one/dure-installer&type=Date)](https://star-history.com/#dure-one/dure-installer&Date)

---

## 🙏 Acknowledgments

Built with:
- [egui](https://github.com/emilk/egui) - Immediate mode GUI framework
- [smol](https://github.com/smol-rs/smol) - Async runtime
- [russh](https://github.com/warp-tech/russh) - Pure Rust SSH
- [diesel](https://diesel.rs/) - Safe, extensible ORM
- [ureq](https://github.com/algesten/ureq) - Minimal HTTP client

---

**Made with ❤️ for small shop owners**
