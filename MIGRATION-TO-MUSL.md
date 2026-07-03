# Migration to musl-only Builds

## Summary

Successfully migrated the dure project to follow **uad-shizuku's musl-first approach** for maximum Linux compatibility.

## Changes Made

### 1. GitHub Actions Workflow (`release.yml`)

**Before:**
- Used `ubuntu-latest` (GLIBC 2.39)
- Built both GNU and musl variants
- Separate jobs for Linux, Windows, macOS
- Complex matrix with variant/target combinations

**After:**
- Follows uad-shizuku pattern
- Uses Rust **nightly** for all builds
- Unified build job with platform matrix
- **Linux headless: musl-only** (x86_64, aarch64)
- Linux GUI: GNU (requires system libs)
- Native targets for Windows/macOS
- Simplified artifact naming: `dure-desktop-{target}`

### 2. Build Matrix

| Platform | Target | Variant | Static? |
|----------|--------|---------|---------|
| Linux x86_64 Headless | x86_64-unknown-linux-musl | headless | ✅ Yes |
| Linux aarch64 Headless | aarch64-unknown-linux-musl | headless | ✅ Yes |
| Linux x86_64 GUI | x86_64-unknown-linux-gnu | gui | ❌ No (needs GTK) |
| macOS x86_64 | x86_64-apple-darwin | gui | Native |
| macOS aarch64 | aarch64-apple-darwin | gui | Native |
| Windows x86_64 | x86_64-pc-windows-msvc | gui | Native |

### 3. Install Script Updates

**Updated asset detection:**
- Now looks for `dure-desktop-x86_64-unknown-linux-musl` artifacts
- Simplified naming (no more `-headless` suffix)
- Defaults to musl for maximum compatibility
- Works with both stable releases and dev artifacts

### 4. Workspace Configuration

**Already optimal** (matches uad-shizuku):
```toml
[profile.release]
lto = true
codegen-units = 1
opt-level = "z"
debug = 0
strip = "symbols"
panic = "abort"
```

### 5. Documentation

**Created/Updated:**
- ✅ `GLIBC-COMPATIBILITY.md` - Updated to reflect musl-first approach
- ✅ `MIGRATION-TO-MUSL.md` - This document
- ✅ `check-glibc.sh` - Helper script (kept for reference)

## Benefits

### For Users
1. **Zero GLIBC issues** - Works on any Linux distro
2. **Single binary** - No version matrix confusion
3. **No dependencies** - Fully static, portable
4. **Future-proof** - Won't break with system updates

### For Developers
1. **Simpler CI** - One build per architecture
2. **Nightly features** - Access to latest Rust improvements
3. **Consistent builds** - Same result regardless of runner OS version
4. **Less maintenance** - No GLIBC version tracking

## Migration Path

### For Server Deployments

**Old command:**
```bash
DURE_TARGET=musl ./install.sh
```

**New command (same result):**
```bash
./install.sh  # Now defaults to musl headless
```

### For Local Development

**Old:**
```bash
cargo build --release --bin dure-desktop --no-default-features
```

**New:**
```bash
cargo +nightly build --release --bin dure-desktop --no-default-features \
  --target x86_64-unknown-linux-musl
```

## Testing the Changes

### Test Local Build

```bash
# Install nightly and musl target
rustup toolchain install nightly
rustup target add x86_64-unknown-linux-musl --toolchain nightly
sudo apt-get install musl-tools

# Build
cd mobile
cargo +nightly build --release --bin dure-desktop --no-default-features \
  --target x86_64-unknown-linux-musl

# Verify it's static
ldd ../target/x86_64-unknown-linux-musl/release/dure-desktop
# Should output: "not a dynamic executable"

# Test on server
scp ../target/x86_64-unknown-linux-musl/release/dure-desktop user@server:/tmp/
ssh user@server '/tmp/dure-desktop --version'
```

### Test CI Build

1. Push to `linux` branch:
   ```bash
   git push origin HEAD:linux
   ```

2. Check GitHub Actions:
   - Should build 3 Linux targets (x86_64 musl, aarch64 musl, x86_64 gnu GUI)
   - Should build 2 macOS targets (x86_64, aarch64)
   - Should build 1 Windows target (x86_64)

3. Download artifacts and verify:
   ```bash
   # Check headless is static
   ldd dure-desktop  # Should say "not a dynamic executable"
   
   # Check size (should be ~20-30MB for headless)
   ls -lh dure-desktop
   ```

## Next Steps

### Immediate
- [x] Update workflow to use nightly + musl
- [x] Update install script for new artifact names
- [x] Update documentation

### Future Enhancements
1. **Add more architectures** (like uad-shizuku):
   - ✅ aarch64-unknown-linux-musl (added)
   - Consider: armv7, arm (if needed)
   - Consider: i686-unknown-linux-musl (32-bit)

2. **Add FreeBSD support** (if needed):
   - uad-shizuku uses VM-based builds for FreeBSD
   - Would require separate job with `vmactions/freebsd-vm`

3. **Code signing** (optional):
   - Windows: Add certificate signing (see uad-shizuku lines 198-235)
   - macOS: Add notarization

4. **VirusTotal scanning** (optional):
   - Automated malware scanning of releases
   - See uad-shizuku lines 279-395

## References

- **uad-shizuku workflow**: `reference/uad-shizuku/.github/workflows/release.yml`
- **uad-shizuku Cargo.toml**: `reference/uad-shizuku/Cargo.toml`
- **Rust musl docs**: https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-linux-musl.html
- **Static linking RFC**: https://rust-lang.github.io/rfcs/1721-crt-static.html

## Rollback Plan

If issues arise, revert by:
```bash
git revert <commit-hash>
git push origin main
```

The old approach is preserved in git history and can be restored if needed.
