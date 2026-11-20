# Dure Installer — SudoLang Project Specification

> Distributed e-commerce platform installer for small shop owners
> Pure Rust · Cross-platform · Material3 UI · smol async

## Project Interface

```sudo
Dure {
  purpose: "Automate GCP VM deployment, DNS, SSH, and mycart backend management"
  language: Rust nightly
  platforms: [Linux, macOS, Windows, Android, WASM]
  architecture: Layered MVVM
  
  constraint: Pure Rust (no tokio, no OpenSSL)
  constraint: Profile-scoped operations
  constraint: GUI optional via features
  constraint: Security-first (attestation, encryption, keyring)
}
```

## Architecture Layers

```sudo
interface Architecture {
  // Uni-directional data flow
  Entry → UI/CLI → ViewModel → Calc → Api
  
  constraint: Platform abstraction via #[cfg(...)]
  constraint: Actors communicate via smol channels
  constraint: ProfileContext propagation for profile-aware ops
}

Layers {
  Entry {
    main.rs               // Desktop eframe
    main_android.rs       // Android native-activity
    main_wasm.rs          // WASM
    lib.rs                // Library exports
  }

  UI {
    dure.rs               // DureApp root
    ui_tabs/              // Material3 tabs (Platform, SSH, DNS, Mycart)
    ui_dlg/               // Dialogs (ProfileLogin, Settings, DbDebugger)
    constraint: feature-gated "gui"
  }

  ViewModel {
    // Actor-based MVVM coordinator
    actor_pattern {
      command_channel → Actor → event_channel
      smol::spawn async processing
      unified event receiver for UI/CLI
    }
    
    actors: [platform, ssh, dns, logs, mycart]
    constraint: Optional egui::Context for GUI mode
  }

  Calc {
    // Business logic (pure Rust)
    modules: [db, profile, crypt, keyring, gcp, dns, ssh, mycart, acme]
    constraint: Profile-scoped DB operations (Diesel)
    constraint: Platform-specific keyring (Linux/macOS/Windows/Android)
  }

  Api {
    // External integrations
    gcp: [compute, billing, dns, oauth, resourcemanager]
    dns: [cloudflare, duckdns, porkbun, gcp]
    mycart: [admin_api, stats, logs, crontab]
  }
}
```

## Core Components

### Profile Management

```sudo
interface ProfileContext {
  // Profile-scoped operations and authentication
  config_file: PathBuf
  db_connection: SqliteConnection
  keyring: Option<DatabaseHandle>
  
  constraint: All operations scoped to active profile
  constraint: SQLite per profile (desktop/Android)
  constraint: PostgreSQL support (future)
}
```

### ViewModel Actor Pattern

```sudo
interface Actor {
  // Generic actor implementation
  command_rx: Receiver<Command>
  event_tx: Sender<ViewModelEvent>
  
  fn run() {
    loop {
      command = command_rx.recv().await
      result = handle_command(command).await
      event_tx.send(ViewModelEvent).await
    }
  }
}

// Example: Mycart Actor
MycartActor {
  commands: [LoadStats, LoadLogs, ListCrontab, EnableCronjob]
  events: [StatsLoaded, LogsLoaded, CrontabLoaded, Error]
  
  constraint: JWT token from config
  constraint: SSH host resolution via resolve_ssh_host_for_site()
}
```

### SSH Host Resolution

```sudo
interface SshHostResolution {
  // Smart resolution with fallback chain
  
  fn resolve_ssh_host_for_site(config, site) {
    // 1. Explicit binding (site.ssh_host)
    if site.ssh_host exists {
      return find_ssh_host_matching(site.ssh_host)
    }
    
    // 2. Fallback: domain/IP matching
    site_host = site.site_address.split(':').first
    return find_ssh_host_by_domain_or_ip(site_host)
  }
  
  constraint: Prefer explicit bindings over fallback
  constraint: Support both domain and IP matching
}
```

### Mycart Integration

```sudo
interface MycartIntegration {
  // Backend API + SSH log access
  
  api_endpoints {
    POST /api/sign/in           // Login, get JWT
    GET /api/_/settings/crontab // List cron jobs
    PATCH /api/_/settings/crontab/:id // Update job
  }
  
  ssh_logs {
    /srv/data/logs/mycart-stdout.log
    /srv/data/logs/mycart-stderr.log
    /srv/data/logs/xmpp-proxy-stdout.log
    /srv/data/logs/fail2ban-rs-stdout.log
  }
  
  // Flexible JSON parsing (array or map)
  fn parse_crontab_response(json) {
    match json.result {
      Array => deserialize directly
      Map => collect values into array
      _ => error
    }
  }
  
  constraint: Handle both API response formats
  constraint: JWT stored in config.yml
  constraint: SSH via resolve_ssh_host_for_site()
}
```

## Recent Fixes (2026-11-21)

```sudo
Fix {
  issue: "SSH host resolution failures"
  root_cause: "Naive substring matching instead of proper resolution"
  
  before {
    let ssh_host = config.ssh_hosts.iter()
      .find(|h| h.host.contains(&site));
  }
  
  after {
    let site_config = config.mycart_sites.iter()
      .find(|s| s.site_address == site);
    let ssh_host = site_config
      .and_then(|s| mycart::resolve_ssh_host_for_site(&config, s));
  }
  
  impact: CheckDureMycart, CheckXmppProxy, CheckFail2BanRS now resolve correctly
}

Fix {
  issue: "Crontab JSON parsing failure"
  root_cause: "API returns map format, code expected array"
  error: "invalid type: map, expected a sequence"
  
  solution: "Flexible parsing supporting both array and map formats"
  
  implementation {
    match result {
      Array => serde_json::from_value(result)
      Map => map.values().collect()
      _ => error
    }
  }
  
  constraint: Backward compatible with both API versions
}
```

## Technology Stack

```sudo
Stack {
  core {
    egui: "0.33"
    diesel: "2.3" // SQLite, PostgreSQL
    smol: "2.0"
    ureq: "2.12"
    rustls: "0.23"
    russh: "0.45"
  }
  
  platform_specific {
    android: [ndk-context, jni]
    desktop: [tray-icon, dotenvy]
    wasm: [wasm-bindgen, sqlite-wasm-rs]
  }
  
  constraint: No tokio (use smol)
  constraint: No OpenSSL (use rustls)
}
```

## Build Profiles

```sudo
Profiles {
  dev {
    opt_level: 0
    debug: true
  }
  
  dev-release {
    opt_level: 2
    debug: true
    // Balanced: fast compile + reasonable performance
  }
  
  release {
    opt_level: 3
    lto: true
    codegen_units: 1
    // Full optimization for production
  }
}
```

## Commands

```sudo
// Build commands
/build [profile] - Build for specified profile (dev, dev-release, release)
/check - Fast compilation check
/test - Run test suite (cargo nextest)
/clean - Clean build artifacts

// Development
/fmt - Format code (rustfmt)
/clippy - Run linter
/audit - Security audit (cargo audit)

// Platform-specific
/build-android - Build Android APK
/build-wasm - Build WASM bundle
```

## Testing Strategy

```sudo
interface Testing {
  unit_tests {
    location: "Same file as implementation (#[cfg(test)])"
    framework: cargo nextest
    coverage_min: 80%
  }
  
  integration_tests {
    location: "tests/"
    scope: "Cross-layer integration"
  }
  
  examples {
    ssh_resolution: "calc/mycart_tests.rs"
    constraint: "Test both explicit and fallback resolution"
  }
}
```

## Deployment

```sudo
Distribution {
  github_releases {
    linux_x86_64: "musl static binary"
    linux_aarch64: "musl static binary (headless only)"
    macos: "Universal binary (x86_64 + aarch64)"
    windows: "MSVC binary"
  }
  
  snap_store {
    package: "dure"
    command: "sudo snap install dure"
  }
  
  google_play {
    package: "app.dure.installer"
    build: "./build.sh" // in mobile/
  }
  
  constraint: Auto-build on `v*` tags
  constraint: GitHub Actions workflows
}
```

## Security Model

```sudo
Security {
  attestation {
    provider: GitHub Sigstore
    scope: "Identity verification"
  }
  
  encryption {
    passwords: "base64 (temporary), upgrade to keyring pending"
    oauth_tokens: "YAML config (temporary), upgrade to keyring pending"
    // ponytail: keyring when security review done
  }
  
  keyring {
    linux: "Secret Service API"
    macos: "Keychain"
    windows: "Credential Manager"
    android: "EncryptedSharedPreferences"
  }
  
  ssh {
    library: russh
    keys: "Stored in keyring with domain namespace"
    constraint: "No password storage, keys only"
  }
}
```

## Style Constraints

```sudo
Lint {
  style {
    * Rust 2021 idioms
    * clippy::pedantic enabled
    * Minimize unsafe code
    * Document all public APIs
    * Platform-specific code isolated with #[cfg(...)]
  }
  
  architecture {
    * Layers respect boundaries (no skip)
    * Async via smol (not tokio)
    * Errors via anyhow (apps) / thiserror (libs)
    * No .unwrap() in production
  }
  
  naming {
    * snake_case: variables, functions, modules
    * CamelCase: types, traits, enum variants
    * SCREAMING_SNAKE_CASE: constants
  }
}
```

## Documentation

```sudo
Docs {
  entry: "CLAUDE.md" // Project overview for AI
  architecture: "docs/ARCH.sudolang.md"
  index: "docs/INDEX.md" // All docs with status flags
  troubleshooting: "docs/TROUBLESHOOTING.md"
  
  constraint: Documentation is code (SudoLang)
  constraint: Flag outdated docs with WARNING
}
```

## For AI Assistants

```sudo
AIContext {
  start_here: "CLAUDE.md, docs/ARCH.sudolang.md"
  valid_docs: "Check docs/INDEX.md status flags"
  focus: "mobile/src/ (despite name)"
  build: ".github/workflows/ (CI/CD configs)"
  
  constraints {
    * Rust nightly required
    * Prefer musl on Linux
    * No tokio, no OpenSSL
    * Profile-scoped operations
  }
}
```
