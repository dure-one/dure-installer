# Remote Installation Script Design

**Date:** 2026-07-01  
**Status:** Approved  
**Approach:** Script-Only with Manual Checksums (Approach 1)

## Overview

A rustup-style remote installation script (`install.sh`) that enables one-command installation of dure on Linux servers. The script supports two modes: stable (GitHub releases) and dev (GitHub Actions artifacts), with SHA256 checksum verification for security.

### Goals

- Enable installation via `curl https://dure.one/install.sh | sh`
- Support both stable releases and development builds
- Verify downloads with SHA256 checksums
- Automatically setup PATH configuration
- Provide clear, helpful error messages
- Follow industry-standard patterns (rustup, Node.js installers)

### Non-Goals

- Multi-platform support (Windows, macOS) - Linux x86_64 only for now
- Automatic updates or version management
- Uninstallation script (separate concern)
- GUI installation wizard

## Requirements Summary

### Functional Requirements

1. **Installation Modes:**
   - Stable mode (default): Install from GitHub releases
   - Dev mode (`DURE_CHANNEL=dev`): Install from GitHub Actions artifacts

2. **Platform Support:**
   - Linux x86_64 only
   - Detect architecture, fail gracefully on arm64/aarch64
   - Auto-detect shell type (bash/zsh/sh) for PATH setup

3. **Security:**
   - SHA256 checksum verification for all downloads
   - Stable mode: Download `.sha256` checksum file from release assets
   - Dev mode: Download `.sha256` checksum file from artifacts

4. **Installation:**
   - Download binary to temp directory
   - Verify checksum
   - Install to `~/.local/bin/dure` (renamed from `dure-desktop`)
   - Add `~/.local/bin` to PATH if not present
   - Overwrite existing installation after user confirmation

5. **User Experience:**
   - Verbose output by default (show progress)
   - Quiet mode via `DURE_QUIET=1` environment variable
   - Clear error messages with actionable guidance
   - ANSI color support where available

### Technical Requirements

1. **Dependencies:**
   - Minimal POSIX shell compatibility (bash, dash, zsh)
   - Required commands: `uname`, `mktemp`, `chmod`, `mkdir`, `rm`, `sha256sum`
   - Downloader: `curl` (preferred) or `wget` (fallback)

2. **GitHub API:**
   - Embedded GitHub token for API authentication
   - Stable mode: `GET /repos/nikescar/dure/releases/latest`
   - Dev mode: `GET /repos/nikescar/dure/actions/artifacts`

3. **Deployment:**
   - Single file: `/home/wj/work/dure/install.sh`
   - Served at: `https://dure.one/install.sh`
   - Included in docs deployment via `_config.yml`

## Architecture

### Component Overview

```
install.sh (single file, ~500-700 lines)
├── Configuration
│   ├── GITHUB_TOKEN (embedded)
│   ├── REPO_OWNER="nikescar"
│   └── REPO_NAME="dure"
│
├── Helper Functions
│   ├── say()               # Info messages (respects DURE_QUIET)
│   ├── warn()              # Warning messages (always shown)
│   ├── err()               # Error messages (always shown)
│   ├── need_cmd()          # Check required commands
│   ├── check_cmd()         # Check optional commands
│   ├── get_architecture()  # Platform detection
│   ├── downloader()        # curl/wget wrapper
│   ├── verify_checksum()   # SHA256 verification
│   └── setup_path()        # Shell config modification
│
└── main()                  # Entry point
    ├── Dependency checks
    ├── Platform detection
    ├── Mode selection
    ├── Download
    ├── Verification
    ├── Installation
    └── PATH setup
```

### Data Flow

#### Stable Mode

```
1. main()
2. ├── get_architecture() → x86_64-unknown-linux-gnu
3. ├── Query GitHub Releases API
4. │   GET https://api.github.com/repos/nikescar/dure/releases/latest
5. │   └── Parse: download URLs for binary + .sha256
6. ├── downloader(binary_url, /tmp/dure-XXXXX/dure-desktop-linux)
7. ├── downloader(checksum_url, /tmp/dure-XXXXX/dure-desktop-linux.sha256)
8. ├── verify_checksum()
9. │   └── sha256sum -c dure-desktop-linux.sha256
10. ├── Prompt for overwrite if exists
11. ├── cp dure-desktop-linux ~/.local/bin/dure
12. ├── chmod +x ~/.local/bin/dure
13. ├── setup_path() → Modify ~/.bashrc (or detected shell config)
14. └── Cleanup temp directory
```

#### Dev Mode

```
1. main()
2. ├── get_architecture() → x86_64-unknown-linux-gnu
3. ├── Query GitHub Artifacts API
4. │   GET https://api.github.com/repos/nikescar/dure/actions/artifacts
5. │   └── Filter: name="dure-desktop-linux", sort by created_at, pick latest
6. ├── downloader(artifact.archive_download_url, /tmp/dure-XXXXX/artifact.zip)
7. ├── unzip artifact.zip → dure-desktop
8. ├── downloader(checksum_url, /tmp/dure-XXXXX/dure-desktop-linux.sha256)
9. ├── verify_checksum()
10. │   └── sha256sum -c dure-desktop-linux.sha256
11. ├── Prompt for overwrite if exists
12. ├── cp dure-desktop ~/.local/bin/dure
13. ├── chmod +x ~/.local/bin/dure
14. ├── setup_path() → Modify detected shell config
15. └── Cleanup temp directory
```

## Detailed Design

### 1. Platform Detection

Detect OS and architecture before proceeding:

```bash
get_architecture() {
    local _os
    local _arch
    
    _os=$(uname -s)
    _arch=$(uname -m)
    
    # Validate OS
    if [ "$_os" != "Linux" ]; then
        err "This installer only supports Linux. Detected: $_os"
        exit 1
    fi
    
    # Validate architecture
    case "$_arch" in
        x86_64|amd64)
            echo "x86_64-unknown-linux-gnu"
            ;;
        aarch64|arm64)
            err "Only x86_64 is currently supported. Detected: $_arch"
            err "arm64/aarch64 support is planned for future releases."
            exit 1
            ;;
        *)
            err "Unsupported architecture: $_arch"
            exit 1
            ;;
    esac
}
```

**Decision Points:**
- Exit immediately on unsupported platforms (fail-fast)
- Provide clear error messages with future roadmap hints
- Return normalized triple format for consistency

### 2. Mode Selection

Determine installation mode from environment:

```bash
CHANNEL="${DURE_CHANNEL:-stable}"

case "$CHANNEL" in
    stable)
        install_from_release
        ;;
    dev)
        install_from_artifacts
        ;;
    *)
        err "Invalid DURE_CHANNEL: $CHANNEL"
        err "Valid options: stable, dev"
        exit 1
        ;;
esac
```

**Decision Points:**
- Default to stable (production-ready)
- Explicit mode selection (no auto-fallback)
- Clear error for invalid channel names

### 3. Download Manager

Unified downloader supporting curl/wget:

```bash
downloader() {
    local _url="$1"
    local _dest="$2"
    local _auth_header=""
    
    # Add auth header if token is set
    if [ -n "$GITHUB_TOKEN" ]; then
        _auth_header="Authorization: Bearer $GITHUB_TOKEN"
    fi
    
    if check_cmd curl; then
        if [ -n "$_auth_header" ]; then
            curl --proto '=https' --tlsv1.2 -fsSL \
                 -H "$_auth_header" \
                 -o "$_dest" "$_url"
        else
            curl --proto '=https' --tlsv1.2 -fsSL \
                 -o "$_dest" "$_url"
        fi
    elif check_cmd wget; then
        if [ -n "$_auth_header" ]; then
            wget --https-only --quiet \
                 --header="$_auth_header" \
                 -O "$_dest" "$_url"
        else
            wget --https-only --quiet \
                 -O "$_dest" "$_url"
        fi
    else
        err "Neither curl nor wget found. Please install one and try again."
        exit 1
    fi
    
    # Check download succeeded
    if [ ! -f "$_dest" ] || [ ! -s "$_dest" ]; then
        err "Download failed: $_url"
        exit 1
    fi
}
```

**Decision Points:**
- Prefer curl (more common, better error handling)
- Require HTTPS with TLS 1.2+
- Silent by default (use progress bars in verbose mode)
- Validate downloaded files exist and are non-empty

### 4. SHA256 Verification

Verify downloads against checksums:

```bash
verify_checksum() {
    local _binary="$1"
    local _checksum_file="$2"
    
    need_cmd sha256sum
    
    # Verify checksum file format
    if ! grep -qE '^[a-f0-9]{64} ' "$_checksum_file"; then
        err "Invalid checksum file format"
        exit 1
    fi
    
    # Change to binary directory for relative path matching
    local _binary_dir
    _binary_dir=$(dirname "$_binary")
    local _binary_name
    _binary_name=$(basename "$_binary")
    
    cd "$_binary_dir" || exit 1
    
    # Verify checksum
    if sha256sum -c "$_checksum_file" >/dev/null 2>&1; then
        say "SHA256 verification passed"
    else
        local _expected
        local _actual
        _expected=$(awk '{print $1}' "$_checksum_file")
        _actual=$(sha256sum "$_binary_name" | awk '{print $1}')
        
        err "SHA256 verification failed"
        err "Expected: $_expected"
        err "Got:      $_actual"
        err "The download may be corrupted. Try again."
        exit 1
    fi
}
```

**Decision Points:**
- Use standard `sha256sum -c` format
- Validate checksum file format before verification
- Provide detailed error output on mismatch
- Exit immediately on verification failure (security-critical)

### 5. Installation

Install verified binary to user directory:

```bash
install_binary() {
    local _src="$1"
    local _dest="$HOME/.local/bin/dure"
    
    # Check for existing installation
    if [ -f "$_dest" ]; then
        warn "dure is already installed at $_dest"
        
        # Prompt for overwrite (skip if non-interactive)
        if [ -t 0 ]; then
            printf "Overwrite? (y/N) "
            read -r response
            case "$response" in
                [yY]|[yY][eE][sS])
                    say "Overwriting existing installation..."
                    ;;
                *)
                    say "Installation cancelled"
                    exit 0
                    ;;
            esac
        else
            # Non-interactive: overwrite silently
            say "Overwriting existing installation (non-interactive mode)..."
        fi
    fi
    
    # Create directory if needed
    mkdir -p "$HOME/.local/bin" || {
        err "Failed to create directory: $HOME/.local/bin"
        exit 1
    }
    
    # Copy and set permissions
    cp "$_src" "$_dest" || {
        err "Failed to install binary to $_dest"
        exit 1
    }
    
    chmod +x "$_dest" || {
        err "Failed to set executable permissions on $_dest"
        exit 1
    }
    
    say "Installed to $_dest"
}
```

**Decision Points:**
- Install to user directory (`~/.local/bin`, no sudo required)
- Prompt for overwrite in interactive mode
- Silent overwrite in non-interactive mode (for automation)
- Create parent directory if missing
- Validate each step with error handling

### 6. PATH Setup

Auto-detect shell and modify appropriate config file:

```bash
setup_path() {
    local _bin_dir="$HOME/.local/bin"
    
    # Check if already in PATH
    case ":$PATH:" in
        *:"$_bin_dir":*)
            say "$_bin_dir is already in PATH"
            return 0
            ;;
    esac
    
    # Detect shell config file
    local _shell_config
    if [ -n "$BASH_VERSION" ]; then
        _shell_config="$HOME/.bashrc"
    elif [ -n "$ZSH_VERSION" ]; then
        _shell_config="$HOME/.zshrc"
    else
        # Default to .profile for POSIX shells
        _shell_config="$HOME/.profile"
    fi
    
    # Check if config file exists
    if [ ! -f "$_shell_config" ]; then
        touch "$_shell_config" || {
            warn "Could not create $_shell_config"
            warn "Manually add to PATH: export PATH=\"\$HOME/.local/bin:\$PATH\""
            return 1
        }
    fi
    
    # Check if PATH export already exists
    if grep -q "^export PATH=.*\.local/bin" "$_shell_config"; then
        say "PATH already configured in $_shell_config"
        return 0
    fi
    
    # Append PATH export
    {
        echo ""
        echo "# Added by dure installer"
        echo "export PATH=\"\$HOME/.local/bin:\$PATH\""
    } >> "$_shell_config" || {
        warn "Could not modify $_shell_config"
        warn "Manually add to PATH: export PATH=\"\$HOME/.local/bin:\$PATH\""
        return 1
    }
    
    say "Added to PATH in $_shell_config"
    say "Run 'source $_shell_config' or restart your shell to use dure"
}
```

**Decision Points:**
- Detect shell at runtime (bash/zsh/other)
- Graceful fallback to `.profile` for unknown shells
- Check for existing PATH configuration (idempotent)
- Non-fatal warnings if PATH setup fails (installation succeeds)
- Provide manual instructions on failure

## Error Handling

### Dependency Failures

```bash
# Check at startup
need_cmd uname
need_cmd mktemp
need_cmd chmod
need_cmd mkdir
need_cmd rm
need_cmd sha256sum

# Downloader check
if ! check_cmd curl && ! check_cmd wget; then
    err "Neither curl nor wget found. Please install one and try again."
    exit 1
fi
```

Exit codes:
- `0`: Success
- `1`: General error (missing deps, unsupported platform, download failure, etc.)

### Network Failures

All network operations wrapped with error handling:

```bash
# Example: Release API call
if ! response=$(curl -fsSL ...); then
    err "Failed to fetch release metadata from GitHub"
    err "Check your network connection and try again."
    exit 1
fi

# Example: No releases found
if [ "$response" = "[]" ] || echo "$response" | grep -q '"message":"Not Found"'; then
    err "No stable releases available yet"
    err "Try: DURE_CHANNEL=dev $0"
    exit 1
fi
```

### Cleanup

Ensure temp directory is always cleaned up:

```bash
main() {
    local _temp_dir
    _temp_dir=$(mktemp -d -t dure-install.XXXXXXXXXX)
    
    # Cleanup on exit (success or failure)
    trap "rm -rf '$_temp_dir'" EXIT
    
    # ... rest of main logic
}
```

## Output & Verbosity

### Output Functions

```bash
# ANSI color codes (if terminal supports it)
if [ -t 2 ] && [ "${TERM:-dumb}" != "dumb" ]; then
    _ansi_bold='\033[1m'
    _ansi_reset='\033[0m'
    _ansi_yellow='\033[33m'
    _ansi_red='\033[31m'
else
    _ansi_bold=''
    _ansi_reset=''
    _ansi_yellow=''
    _ansi_red=''
fi

say() {
    if [ "${DURE_QUIET:-no}" = "no" ]; then
        printf "${_ansi_bold}info:${_ansi_reset} %s\n" "$1" >&2
    fi
}

warn() {
    printf "${_ansi_bold}${_ansi_yellow}warn:${_ansi_reset} %s\n" "$1" >&2
}

err() {
    printf "${_ansi_bold}${_ansi_red}error:${_ansi_reset} %s\n" "$1" >&2
}
```

### Example Output

**Verbose (default):**
```
info: Detected platform: x86_64-unknown-linux-gnu
info: Running in stable mode
info: Fetching latest release from GitHub...
info: Downloading dure binary...
info: Downloading checksum file...
info: Verifying SHA256 checksum...
info: SHA256 verification passed
warn: dure is already installed at /home/user/.local/bin/dure
Overwrite? (y/N) y
info: Overwriting existing installation...
info: Installed to /home/user/.local/bin/dure
info: Added to PATH in /home/user/.bashrc
info: Run 'source /home/user/.bashrc' or restart your shell to use dure
info: Installation complete!
```

**Quiet (`DURE_QUIET=1`):**
```
(Only warnings and errors shown)
```

## Deployment

### File Locations

```
Repository:
  /home/wj/work/dure/install.sh

Docs site:
  https://dure.one/install.sh
```

### Configuration Changes

**`_config.yml` modification:**

```yaml
include:
  - .well-known
  - index.md
  - fastlane
  - README.md
  - install.sh          # ← Add this line
```

### Workflow

```
1. Create install.sh in repository root
2. Modify _config.yml to include install.sh
3. Push to main branch
4. GitHub Actions (.github/workflows/vite.docs.yml) triggers
5. Theme builds and deploys to Cloudflare Pages
6. install.sh available at https://dure.one/install.sh
```

### Usage Examples

```bash
# Standard installation (stable mode)
curl --proto '=https' --tlsv1.2 -sSf https://dure.one/install.sh | sh

# Dev mode installation
curl --proto '=https' --tlsv1.2 -sSf https://dure.one/install.sh | DURE_CHANNEL=dev sh

# Quiet mode
curl --proto '=https' --tlsv1.2 -sSf https://dure.one/install.sh | DURE_QUIET=1 sh

# Combined flags
curl --proto '=https' --tlsv1.2 -sSf https://dure.one/install.sh | DURE_CHANNEL=dev DURE_QUIET=1 sh
```

## Checksum Management

### Manual Process (Approach 1)

Since this design uses manual checksum generation, the release process requires these additional steps:

**For Stable Releases (GitHub Releases):**

```bash
# After building the binary
cd target/release

# Generate checksum
sha256sum dure-desktop > dure-desktop-linux.sha256

# Upload both files to GitHub Release:
# 1. Rename dure-desktop → dure-desktop-linux
# 2. Upload dure-desktop-linux (binary)
# 3. Upload dure-desktop-linux.sha256 (checksum file)
```

**For Dev Artifacts (GitHub Actions):**

The release workflow (`.github/workflows/release.yml`) needs modification to generate and upload checksum files:

```yaml
- name: Generate checksum
  run: |
    sha256sum target/release/dure-desktop > dure-desktop-linux.sha256

- name: Upload artifact
  uses: actions/upload-artifact@v4
  with:
    name: dure-desktop-linux
    path: |
      target/release/dure-desktop
      dure-desktop-linux.sha256
```

### Future Automation

This manual process can be automated in the future by:
1. Adding checksum generation to the release workflow
2. Auto-uploading checksums as release assets
3. Moving to Approach 3 (automated checksums) for better maintainability

## Security Considerations

### Embedded GitHub Token

**Risk:** The GitHub token is embedded in the public script.

**Mitigation:**
- Use a read-only token with minimal scopes
- Token only grants access to public repositories
- Can be rotated if compromised
- Future: move to tokenless public API for releases, use token only for artifacts

### SHA256 Verification

**Risk:** Man-in-the-middle attack during download.

**Mitigation:**
- HTTPS-only downloads (TLS 1.2+)
- SHA256 checksum verification before installation
- Fail immediately on checksum mismatch
- Clear error messages on verification failure

### Installation Directory

**Risk:** Malicious binary overwriting system binaries.

**Mitigation:**
- Install to user directory (`~/.local/bin`), not system directory
- No sudo required
- User confirmation before overwriting existing installation

## Testing Strategy

### Manual Testing Scenarios

1. **Fresh installation (stable mode):**
   - Clean system without dure
   - Run: `curl https://dure.one/install.sh | sh`
   - Verify: binary installed, PATH configured, can run `dure --version`

2. **Fresh installation (dev mode):**
   - Clean system without dure
   - Run: `curl https://dure.one/install.sh | DURE_CHANNEL=dev sh`
   - Verify: latest artifact installed

3. **Overwrite installation:**
   - System with existing dure
   - Run installer
   - Verify: overwrite prompt shown, installation succeeds

4. **No releases (stable mode):**
   - Before any releases published
   - Run: `curl https://dure.one/install.sh | sh`
   - Verify: clear error message, suggests dev mode

5. **Unsupported architecture:**
   - Run on arm64 Linux
   - Verify: clear error message, mentions future support

6. **Missing dependencies:**
   - System without curl/wget
   - Verify: clear error message, asks to install downloader

7. **Network failure:**
   - Disconnect network mid-installation
   - Verify: clear error message, cleanup happens

8. **Checksum mismatch:**
   - Corrupt downloaded binary
   - Verify: verification fails with clear error, installation aborted

### Automated Testing (Future)

Consider adding:
- Shell script linter (shellcheck)
- Unit tests for helper functions (bats framework)
- Integration tests in Docker containers
- CI workflow to validate install.sh syntax

## Documentation Updates

After implementation, update these docs:

1. **`docs/INSTALLING.md`:**
   - Add "Quick Install" section at the top
   - Show one-line install command
   - Explain stable vs dev modes
   - Link to troubleshooting

2. **`README.md`:**
   - Add installation section
   - Show the curl | sh command

3. **`CLAUDE.md`:**
   - Update installation instructions
   - Reference install.sh

4. **Create `docs/INSTALL_SCRIPT.md`:**
   - Detailed guide for install.sh
   - Environment variables reference
   - Troubleshooting common issues
   - Manual installation fallback

## Success Criteria

The installation script is successful when:

- ✅ Users can install dure with a single command
- ✅ Both stable and dev modes work correctly
- ✅ SHA256 verification prevents corrupted downloads
- ✅ PATH is automatically configured
- ✅ Clear error messages guide users through problems
- ✅ Installation completes in under 30 seconds on typical connections
- ✅ Script is maintainable and well-documented
- ✅ Zero security vulnerabilities (HTTPS, verification, user directory)

## Future Enhancements

Not in initial scope, but consider for future iterations:

1. **Automated checksums** - Move to Approach 3 with workflow-generated checksums
2. **arm64 support** - Add aarch64 builds and detection
3. **Version selection** - Allow installing specific versions (`DURE_VERSION=v1.2.3`)
4. **Uninstall script** - Provide `uninstall.sh` companion
5. **Update command** - Built-in `dure self-update` command
6. **Multi-platform** - Extend to macOS, Windows (PowerShell script)
7. **Offline mode** - Support local binary installation
8. **Rollback** - Keep previous version as `.old` for easy rollback

## References

- **rustup installer:** https://sh.rustup.rs (primary inspiration)
- **GitHub Releases API:** https://docs.github.com/en/rest/releases/releases
- **GitHub Artifacts API:** https://docs.github.com/en/rest/actions/artifacts
- **POSIX shell scripting:** IEEE Std 1003.1
- **Dure repository:** https://github.com/nikescar/dure
