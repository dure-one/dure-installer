# Build Architecture

## Strategy: musl-only for Linux (all variants)

### Linux - musl for everything
**Both headless and GUI use musl**

```
x86_64-unknown-linux-musl   → Headless (static libc, no dependencies)
x86_64-unknown-linux-musl   → GUI (static libc, links GTK/X11 dynamically)
aarch64-unknown-linux-musl  → Headless (ARM64 servers)
```

**Why musl for GUI too?**
- ✅ musl = static libc (no GLIBC version issues)
- ✅ GTK/X11/Wayland link dynamically (like normal GUI apps)
- ✅ Works on any Linux distro
- ✅ Simpler: no need for separate gnu builds

### Desktop OSes (User Machines)
**Native targets with full GUI**

```
macOS:   x86_64-apple-darwin, aarch64-apple-darwin
Windows: x86_64-pc-windows-msvc
```

## Benefits

### For Users
1. **Linux**: Download one binary, works on any distro
2. **No GLIBC issues**: Fully static, zero dependencies
3. **Simple**: No need to choose variants or targets

### For Developers
1. **Cleaner CI**: 5 targets instead of 7
2. **Faster builds**: No redundant Linux GUI
3. **Less confusion**: One Linux binary to rule them all

## Build Matrix

```yaml
matrix:
  platform:
    # Linux - musl only (both headless and GUI)
    - Linux x86_64 Headless: x86_64-unknown-linux-musl --no-default-features
    - Linux x86_64 GUI:      x86_64-unknown-linux-musl (with GTK)
    - Linux aarch64 Headless: aarch64-unknown-linux-musl --no-default-features
    
    # Desktop OSes - native
    - macOS x86_64:   x86_64-apple-darwin
    - macOS aarch64:  aarch64-apple-darwin
    - Windows x86_64: x86_64-pc-windows-msvc
```

## Installation

### Linux (any distro)
```bash
curl -sSL https://raw.githubusercontent.com/nikescar/dure/main/install.sh | sh
```

### macOS / Windows
Download from GitHub Releases:
- macOS: `dure-desktop` (native binary)
- Windows: `dure-desktop.exe`

## Building Locally

### Linux (musl)
```bash
rustup toolchain install nightly
rustup target add x86_64-unknown-linux-musl --toolchain nightly
sudo apt-get install musl-tools

cd mobile
cargo +nightly build --release --bin dure-desktop \
  --no-default-features --target x86_64-unknown-linux-musl
```

### macOS / Windows (native)
```bash
rustup toolchain install nightly

cd mobile
cargo +nightly build --release --bin dure-desktop
```

## Design Philosophy

**Follow uad-shizuku approach:**
1. **musl for ALL Linux builds** (headless AND GUI)
2. Static libc = no GLIBC version issues
3. GUI still links GTK dynamically (normal behavior)
4. Use nightly Rust (latest features)

**Result:**
- ✅ Works on any Linux distro (Ubuntu 16.04 to latest)
- ✅ No "GLIBC_X.XX not found" errors
- ✅ Simpler: no gnu/musl choice needed
- ✅ Clean, maintainable CI

**Key insight:** musl ≠ fully static. It means:
- libc is statically linked (no GLIBC dependency)
- Other libs (GTK, X11) still link dynamically
- Best of both worlds!
