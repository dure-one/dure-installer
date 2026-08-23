# Quick Reference Guide

Fast lookup for common tasks and codebase navigation.

## File Locations

| What | Where |
|------|-------|
| **Main app code** | `mobile/src/` |
| **Desktop entry** | `mobile/src/main.rs` |
| **Android entry** | `mobile/src/lib.rs` (android_main) |
| **Dependencies** | `Cargo.toml` (workspace), `mobile/Cargo.toml` (app) |
| **UI code** | `mobile/src/ui/` or `mobile/src/*_view.rs` |
| **i18n translations** | `mobile/resources/` (.ftl files) |
| **Android config** | `mobile/app/` |
| **Build scripts** | `mobile/build*.sh`, `scripts/` |
| **Documentation** | `docs/`, `CLAUDE.md` |

## Build Commands

```bash
# Desktop (from project root)
cargo build                    # Debug
cargo build --release          # Release
cargo build --profile dev-release  # Fast release

# Run desktop
cargo run

# Android (from mobile/ directory)
cd mobile
./build.sh                # Full build (Gradle + Rust)
./build.rust-only.sh      # Rust library only (faster)
./qemu.sh                 # Run in emulator

# WASM (from project root)
cargo build --target wasm32-unknown-unknown

# Testing
cargo test                           # All tests
cargo test --package dure            # Just dure package
RUST_LOG=debug cargo test           # With logging

# Quality checks
cargo clippy --all-targets          # Linting
cargo fmt                           # Format
cargo doc --open                    # Generate docs
```

## Platform CLI Commands

### List Platforms
```bash
dure platform
```

### Show Platform Details
```bash
dure platform my-gcp
```

### Refresh Platform Data
```bash
dure platform my-gcp refresh
```

### Update Firewall (Auto-detect IP)
```bash
dure platform my-gcp firewall
```

### Update Firewall (Explicit IP)
```bash
dure platform my-gcp firewall --ip 203.0.113.42
```

### Add VM (Interactive)
```bash
dure platform my-gcp addvm --vm-name my-vm
```

### Add VM (Full Explicit)
```bash
dure platform my-gcp addvm --vm-name my-vm --zone us-west1-a --machine-type e2-small
```

### Restart VM
```bash
dure platform my-gcp restart
```

### Delete VM
```bash
dure platform my-gcp delvm
```

### Show Billing
```bash
dure platform my-gcp billing
```

### Delete Platform
```bash
dure platform my-gcp delete
```

## Platform-Specific Features

```rust
// Desktop-only code
#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
fn desktop_feature() { }

// Android-only code
#[cfg(target_os = "android")]
fn android_feature() { }

// WASM-only code
#[cfg(target_arch = "wasm32")]
fn wasm_feature() { }

// Desktop or Android (not WASM)
#[cfg(not(target_arch = "wasm32"))]
fn native_feature() { }
```

## Key Dependencies

| Dependency | Purpose | Platforms |
|------------|---------|-----------|
| `egui` | UI framework | All |
| `eframe` | egui application framework | All |
| `egui-material3` | Material Design 3 | All |
| `egui-i18n` | Internationalization | All |
| `serde` / `serde_json` | Serialization | All |
| `datafusion` | SQL analytics | Desktop, Android |
| `arrow` / `parquet` | Data format | Desktop, Android |
| `tokio` | Async runtime | Desktop, Android |
| `tray-icon` | System tray | Desktop only |
| `webbrowser` | Browser integration | Desktop only |
| `android-activity` | Android entry | Android only |
| `wasm-bindgen` | JS interop | WASM only |

## Project Structure

```
dure/
├── CLAUDE.md                  # Complete project guide
├── README.md                  # User-facing readme
├── Cargo.toml                 # Workspace manifest
│
├── mobile/                    # Main application
│   ├── src/                   # All Rust source
│   │   ├── main.rs           # Desktop entry
│   │   ├── lib.rs            # Library + Android entry
│   │   └── ...               # UI and business logic
│   ├── app/                   # Android Gradle project
│   ├── assets/                # Images, icons
│   ├── resources/             # i18n translations
│   ├── tests/                 # Integration tests
│   ├── build.sh              # Android build
│   └── Cargo.toml            # App dependencies
│
├── docs/                      # Documentation
│   ├── INDEX.md              # Doc index with status
│   ├── PROJECT_SUMMARY.md    # Architecture overview
│   ├── QUICK_REFERENCE.md    # This file
│   └── ...                   # Other docs
│
├── deploy/                    # Deployment configs
├── fastlane/                  # Mobile CI/CD
├── scripts/                   # Build utilities
├── snap/                      # Snap packaging
└── .github/                   # GitHub Actions
```

## Common Tasks

### Add a New Dependency

```bash
# For all platforms
cd mobile
cargo add <package>

# For specific platform
cargo add <package> --target <target>
# Examples:
cargo add tray-icon --target x86_64-unknown-linux-gnu
cargo add wasm-bindgen --target wasm32-unknown-unknown
```

### Add i18n Translation

1. Add string to `mobile/resources/<locale>/<domain>.ftl`
2. Use in code:
```rust
use egui_i18n::I18n;

fn render_ui(&mut self, i18n: &I18n, ui: &mut egui::Ui) {
    ui.label(i18n.get("key-name"));
}
```

### Debug Android App

```bash
cd mobile
./build.sh

# View logs
adb logcat | grep RustStdoutStderr
```

### Update Documentation

1. Edit markdown files in `docs/`
2. Update `docs/INDEX.md` if adding new doc
3. Test links work
4. Update status flags (⚠️ or ✅)

### Profile Performance

```bash
# Desktop
cargo build --release
perf record ./target/release/dure
perf report

# Criterion benchmarks (if added)
cargo bench
```

## Code Patterns

### egui UI Pattern

```rust
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("My View");
            ui.label("Content here");

            if ui.button("Click me").clicked() {
                // Handle click
            }
        });
    }
}
```

### Async with Tokio

```rust
use tokio::runtime::Runtime;

// Create runtime
let rt = Runtime::new()?;

// Spawn task
rt.spawn(async {
    // Async work
});

// Block on future
let result = rt.block_on(async_function());
```

### Error Handling

```rust
use anyhow::{Context, Result};

fn do_something() -> Result<()> {
    let data = load_data()
        .context("Failed to load data")?;

    process_data(data)
        .context("Failed to process")?;

    Ok(())
}
```

### DataFusion Query

```rust
use datafusion::prelude::*;

// Create context
let ctx = SessionContext::new();

// Register table
ctx.register_csv("sales", "data/sales.csv", CsvReadOptions::new())?;

// Query
let df = ctx.sql("SELECT * FROM sales WHERE amount > 100").await?;
let results = df.collect().await?;
```

## Configuration

### Build Profiles

| Profile | Command | Use Case |
|---------|---------|----------|
| `dev` | `cargo build` | Development, fast compile |
| `release` | `cargo build --release` | Production, smallest size |
| `dev-release` | `cargo build --profile dev-release` | Testing, balanced |

### Rust Version

- **Minimum**: 1.81
- **Edition**: 2021
- **Recommended**: Latest stable

## Troubleshooting

### Common Issues

| Problem | Solution |
|---------|----------|
| "cannot find -lEGL" | Install mesa dev packages (Linux) |
| "failed to run custom build command" | Check build.rs dependencies |
| Android build fails | Run `./build.sh` from mobile/ dir |
| Missing symbols | Check platform-specific dependencies |
| Cargo.lock conflict | `cargo update` then rebuild |

### Environment Variables

```bash
# Logging
export RUST_LOG=debug           # All debug logs
export RUST_LOG=dure=trace      # Just dure crate
export RUST_LOG=error           # Errors only

# Android
export ANDROID_HOME=~/Android/Sdk
export NDK_HOME=$ANDROID_HOME/ndk/<version>

# Build
export RUSTFLAGS="-C target-cpu=native"  # Optimize for local CPU
```

## Resources

### Documentation
- [CLAUDE.md](../CLAUDE.md) - Complete project guide
- [PROJECT_SUMMARY.md](./PROJECT_SUMMARY.md) - Architecture overview
- [QUICK_REFERENCE.md](./QUICK_REFERENCE.md) - This file
- [INDEX.md](./INDEX.md) - All docs

### External
- [egui docs](https://docs.rs/egui)
- [eframe examples](https://github.com/emilk/egui/tree/master/examples)
- [Rust book](https://doc.rust-lang.org/book/)
- [Tokio tutorial](https://tokio.rs/tokio/tutorial)
- [DataFusion guide](https://datafusion.apache.org/)

### Tools
- [rustup](https://rustup.rs/) - Rust installer
- [cargo](https://doc.rust-lang.org/cargo/) - Package manager
- [clippy](https://github.com/rust-lang/rust-clippy) - Linter
- [rustfmt](https://github.com/rust-lang/rustfmt) - Formatter

## Git Workflow

```bash
# Feature branch
git checkout -b feature/my-feature
# ... make changes ...
cargo test
cargo clippy
git add .
git commit -m "feat: add my feature"
git push origin feature/my-feature
# Create PR

# Commit format (see GUIDELINES_GIT_COMMITS.md)
# feat: new feature
# fix: bug fix
# docs: documentation
# style: formatting
# refactor: code restructure
# test: add tests
# chore: maintenance
```

## Help

- **Issues**: https://github.com/dure-one/dure-installer/issues
- **Discussions**: GitHub Discussions (if enabled)
- **Code of Conduct**: [CODE_OF_CONDUCT.md](../CODE_OF_CONDUCT.md)
- **Security**: [SECURITY.md](../SECURITY.md)
