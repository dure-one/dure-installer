# GLIBC Compatibility Guide

## Solution: musl-only builds

As of the latest release, **all headless Linux builds use musl** for maximum compatibility. The binary works on **any Linux distribution** with no GLIBC dependencies.

```bash
# Install (defaults to musl headless build)
./install.sh

# For development builds
DURE_CHANNEL=dev ./install.sh
```

## Build Strategy

Following the **uad-shizuku** approach:
- **Linux Headless**: musl-only (fully static, works everywhere)
- **Linux GUI**: GNU target (requires system libraries anyway)
- **All platforms**: Rust nightly toolchain

### Why musl?

**Pros:**
- ✅ Works on any Linux distribution (no GLIBC version issues)
- ✅ Fully static binary - no runtime dependencies
- ✅ Single binary for all Linux distros
- ✅ Maximum compatibility

**Trade-offs:**
- Slightly larger binary size (~10-15%)
- Linux only (Windows/macOS use native targets)

## Verify Binary Type

```bash
# Check if binary is static (musl)
ldd /path/to/dure-desktop
# Output: "not a dynamic executable" = fully static ✅

# Or check file type
file /path/to/dure-desktop
# Output should mention "statically linked"
```

## Platform Build Matrix

| Platform | Target | Rust | Features | Notes |
|----------|--------|------|----------|-------|
| **Linux x86_64 Headless** | x86_64-unknown-linux-musl | nightly | headless | Static libc ✅ |
| **Linux x86_64 GUI** | x86_64-unknown-linux-musl | nightly | full | Static libc + GTK ✅ |
| **Linux aarch64 Headless** | aarch64-unknown-linux-musl | nightly | headless | ARM64 servers ✅ |
| macOS x86_64 | x86_64-apple-darwin | nightly | full | Intel Macs |
| macOS aarch64 | aarch64-apple-darwin | nightly | full | Apple Silicon |
| Windows x86_64 | x86_64-pc-windows-msvc | nightly | full | Windows 10+ |

**All Linux builds use musl:**
- ✅ Headless: Fully static (no dependencies)
- ✅ GUI: Static libc + dynamic GTK/X11 (normal GUI app)
- ✅ Works on any distro (no GLIBC issues)

### Install Script

```bash
# Headless (default) - for servers
./install.sh

# GUI - for desktop Linux
DURE_VARIANT=gui ./install.sh

# Development builds
DURE_CHANNEL=dev ./install.sh
DURE_VARIANT=gui DURE_CHANNEL=dev ./install.sh
```

**Both variants use musl** - works on any Linux distro!

## Building Locally

### Headless (musl - recommended)

```bash
# Install Rust nightly and musl toolchain
rustup toolchain install nightly
rustup target add x86_64-unknown-linux-musl --toolchain nightly

# On Ubuntu/Debian
sudo apt-get install musl-tools

# Build
cd mobile
cargo +nightly build --release --bin dure-desktop --no-default-features --target x86_64-unknown-linux-musl

# Binary location
ls -lh ../target/x86_64-unknown-linux-musl/release/dure-desktop

# Verify it's static
ldd ../target/x86_64-unknown-linux-musl/release/dure-desktop
# Should output: "not a dynamic executable"
```

### GUI (GNU)

```bash
# Install dependencies (Ubuntu/Debian)
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev librsvg2-dev golang-go

# Build
cd mobile
cargo +nightly build --release --bin dure-desktop --target x86_64-unknown-linux-gnu

# Binary location
ls -lh ../target/x86_64-unknown-linux-gnu/release/dure-desktop
```

### ARM64 (aarch64-musl)

```bash
# Add ARM64 target
rustup target add aarch64-unknown-linux-musl --toolchain nightly

# May need cross-compilation setup
sudo apt-get install gcc-aarch64-linux-gnu

# Build
cd mobile
cargo +nightly build --release --bin dure-desktop --no-default-features --target aarch64-unknown-linux-musl
```

## Troubleshooting

### "GLIBC_X.XX not found" error

If you still get GLIBC errors:
```bash
# You might have downloaded the wrong variant
# The headless build should be musl (fully static)

# Check binary type
file /path/to/dure-desktop
# Should show: "statically linked" for musl builds

# If it shows "dynamically linked", you have the GUI version
# Download the headless build instead
```

### musl build fails with linker errors

```bash
# Ensure musl-tools is installed
sudo apt-get install musl-tools

# Verify target is installed
rustup target list --installed | grep musl

# If missing, add it
rustup target add x86_64-unknown-linux-musl --toolchain nightly
```

### Binary too large

musl binaries are ~10-15% larger than GNU due to static linking:
```bash
# This is expected and ensures compatibility
# The binary includes libc and has no dependencies

# To reduce size (already done in CI):
# - opt-level = "z" (size optimization)
# - strip = "symbols" (remove debug symbols)
# - lto = true (link-time optimization)
```

## References

- [Rust musl cross-compilation](https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-linux-musl.html)
- [GLIBC version history](https://sourceware.org/glibc/wiki/Glibc%20Timeline)
- [Static linking in Rust](https://rust-lang.github.io/rfcs/1721-crt-static.html)
