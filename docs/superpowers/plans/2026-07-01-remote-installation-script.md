# Remote Installation Script Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Create a rustup-style installation script enabling one-command dure installation via `curl https://dure.one/install.sh | sh`

**Architecture:** Single-file POSIX shell script with platform detection, GitHub API integration for releases/artifacts, SHA256 verification, automatic installation to `~/.local/bin`, and PATH setup with shell auto-detection.

**Tech Stack:** POSIX shell (bash/dash/zsh compatible), curl/wget, sha256sum, GitHub REST API

## Global Constraints

- POSIX shell compatible (no bash-only features except in detection)
- Linux x86_64 only (detect and fail on other platforms)
- HTTPS-only with TLS 1.2+ for all downloads
- SHA256 verification required before installation
- Install to `~/.local/bin` (no sudo required)
- Verbose output by default, controllable via `DURE_QUIET=1`
- Embedded GitHub token: `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`
- Repository: `nikescar/dure`

---

### Task 1: Create Script Foundation and Platform Detection

**Files:**
- Create: `install.sh`
- Test: Manual verification on Linux x86_64 and arm64

**Interfaces:**
- Consumes: Nothing (entry point)
- Produces:
  - `get_architecture() -> string` (returns "x86_64-unknown-linux-gnu" or exits)
  - `say(message: string) -> void` (info output, respects DURE_QUIET)
  - `warn(message: string) -> void` (warning output, always shown)
  - `err(message: string) -> void` (error output, always shown)
  - `need_cmd(command: string) -> void` (exits if command not found)
  - `check_cmd(command: string) -> boolean` (returns 0 if found, 1 otherwise)

- [ ] **Step 1: Create install.sh with shebang and basic structure**

```bash
#!/bin/sh
# shellcheck shell=dash
# shellcheck disable=SC2039  # local is non-POSIX

set -u

# Configuration
GITHUB_TOKEN="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"
REPO_OWNER="nikescar"
REPO_NAME="dure"
CHANNEL="${DURE_CHANNEL:-stable}"
QUIET="${DURE_QUIET:-no}"

main() {
    say "Dure installer"
    say "Detected channel: $CHANNEL"
}

main "$@"
```

- [ ] **Step 2: Add ANSI color detection and output functions**

```bash
# ANSI color detection (add after configuration, before main)
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
    if [ "$QUIET" = "no" ]; then
        printf "${_ansi_bold}info:${_ansi_reset} %s\n" "$1" >&2
    fi
}

warn() {
    printf "${_ansi_bold}${_ansi_yellow}warn:${_ansi_reset} %s\n" "$1" >&2
}

err() {
    printf "${_ansi_bold}${_ansi_red}error:${_ansi_reset} %s\n" "$1" >&2
}

need_cmd() {
    if ! check_cmd "$1"; then
        err "need '$1' (command not found)"
        exit 1
    fi
}

check_cmd() {
    command -v "$1" > /dev/null 2>&1
}
```

- [ ] **Step 3: Test output functions**

Run: `sh install.sh`
Expected: See colored "info: Dure installer" and "info: Detected channel: stable"

Run: `DURE_QUIET=1 sh install.sh`
Expected: No output (quiet mode)

- [ ] **Step 4: Add platform detection function**

```bash
# Add after output functions, before main
get_architecture() {
    local _os
    local _arch
    
    need_cmd uname
    
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

- [ ] **Step 5: Update main() to call platform detection**

```bash
main() {
    local _arch
    
    say "Dure installer"
    
    # Detect platform
    _arch=$(get_architecture)
    say "Detected platform: $_arch"
    
    say "Channel: $CHANNEL"
}
```

- [ ] **Step 6: Test platform detection on x86_64**

Run: `sh install.sh`
Expected: See "info: Detected platform: x86_64-unknown-linux-gnu"

- [ ] **Step 7: Test platform detection failure (simulate arm64)**

Edit install.sh temporarily to force arm64:
```bash
# In get_architecture(), change:
_arch=$(uname -m)
# To:
_arch="aarch64"
```

Run: `sh install.sh`
Expected: "error: Only x86_64 is currently supported. Detected: aarch64"

Revert the change after testing.

- [ ] **Step 8: Commit foundation**

```bash
git add install.sh
git commit -m "feat(install): add script foundation and platform detection

- POSIX shell compatible structure
- Output functions with ANSI color support
- Platform detection (Linux x86_64 only)
- Verbose/quiet mode support via DURE_QUIET

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 2: Implement Download Manager

**Files:**
- Modify: `install.sh`
- Test: Manual verification with GitHub API

**Interfaces:**
- Consumes:
  - `say(message: string)`
  - `err(message: string)`
  - `need_cmd(command: string)`
  - `check_cmd(command: string)`
  - Global: `GITHUB_TOKEN`, `REPO_OWNER`, `REPO_NAME`
- Produces:
  - `downloader(url: string, dest: string) -> void` (downloads file or exits)
  - `get_release_metadata() -> string` (returns JSON or exits)
  - `get_artifact_metadata() -> string` (returns JSON or exits)

- [ ] **Step 1: Add dependency checks to main()**

```bash
# In main(), after platform detection, add:
main() {
    local _arch
    
    say "Dure installer"
    
    # Check dependencies
    need_cmd uname
    need_cmd mktemp
    need_cmd chmod
    need_cmd mkdir
    need_cmd rm
    need_cmd sha256sum
    
    # Check downloader
    if ! check_cmd curl && ! check_cmd wget; then
        err "Neither curl nor wget found. Please install one and try again."
        exit 1
    fi
    
    # Detect platform
    _arch=$(get_architecture)
    say "Detected platform: $_arch"
    
    say "Channel: $CHANNEL"
}
```

- [ ] **Step 2: Test dependency checks**

Run: `sh install.sh`
Expected: Script runs normally (all deps present)

Run on system without sha256sum (or temporarily rename it):
```bash
sudo mv /usr/bin/sha256sum /usr/bin/sha256sum.bak
sh install.sh
sudo mv /usr/bin/sha256sum.bak /usr/bin/sha256sum
```
Expected: "error: need 'sha256sum' (command not found)"

- [ ] **Step 3: Add downloader function**

```bash
# Add after get_architecture(), before main()
downloader() {
    local _url="$1"
    local _dest="$2"
    local _status
    
    say "Downloading: $_url"
    
    if check_cmd curl; then
        if [ -n "$GITHUB_TOKEN" ]; then
            curl --proto '=https' --tlsv1.2 -fsSL \
                 -H "Accept: application/vnd.github+json" \
                 -H "Authorization: Bearer $GITHUB_TOKEN" \
                 -H "X-GitHub-Api-Version: 2026-03-10" \
                 -o "$_dest" "$_url"
            _status=$?
        else
            curl --proto '=https' --tlsv1.2 -fsSL \
                 -o "$_dest" "$_url"
            _status=$?
        fi
    elif check_cmd wget; then
        if [ -n "$GITHUB_TOKEN" ]; then
            wget --https-only --quiet \
                 --header="Accept: application/vnd.github+json" \
                 --header="Authorization: Bearer $GITHUB_TOKEN" \
                 --header="X-GitHub-Api-Version: 2026-03-10" \
                 -O "$_dest" "$_url"
            _status=$?
        else
            wget --https-only --quiet \
                 -O "$_dest" "$_url"
            _status=$?
        fi
    fi
    
    if [ $_status -ne 0 ]; then
        err "Download failed: $_url"
        exit 1
    fi
    
    # Check file exists and is non-empty
    if [ ! -f "$_dest" ] || [ ! -s "$_dest" ]; then
        err "Downloaded file is empty or missing: $_dest"
        exit 1
    fi
    
    say "Downloaded to: $_dest"
}
```

- [ ] **Step 4: Add GitHub API metadata functions**

```bash
# Add after downloader(), before main()
get_release_metadata() {
    local _url="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"
    local _temp_file
    local _response
    
    _temp_file=$(mktemp)
    
    downloader "$_url" "$_temp_file"
    _response=$(cat "$_temp_file")
    rm "$_temp_file"
    
    # Check if release exists
    if echo "$_response" | grep -q '"message":"Not Found"'; then
        err "No stable releases available yet"
        err "Try: DURE_CHANNEL=dev $0"
        exit 1
    fi
    
    echo "$_response"
}

get_artifact_metadata() {
    local _url="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/actions/artifacts"
    local _temp_file
    local _response
    
    _temp_file=$(mktemp)
    
    downloader "$_url" "$_temp_file"
    _response=$(cat "$_temp_file")
    rm "$_temp_file"
    
    echo "$_response"
}
```

- [ ] **Step 5: Test release metadata fetching**

Temporarily add to main():
```bash
# In main(), after platform detection:
if [ "$CHANNEL" = "stable" ]; then
    _metadata=$(get_release_metadata)
    say "Release metadata fetched (length: ${#_metadata})"
fi
```

Run: `sh install.sh`
Expected: "error: No stable releases available yet" (since no releases exist)

- [ ] **Step 6: Test artifact metadata fetching**

Temporarily add to main():
```bash
# In main(), after platform detection:
if [ "$CHANNEL" = "dev" ]; then
    _metadata=$(get_artifact_metadata)
    say "Artifact metadata fetched (length: ${#_metadata})"
fi
```

Run: `DURE_CHANNEL=dev sh install.sh`
Expected: "info: Artifact metadata fetched (length: NNNN)" (where NNNN > 0)

Remove temporary test code from main().

- [ ] **Step 7: Commit download manager**

```bash
git add install.sh
git commit -m "feat(install): add download manager and GitHub API integration

- downloader() function with curl/wget support
- GitHub API metadata fetching for releases/artifacts
- Dependency checks for required commands
- TLS 1.2+ enforcement for all downloads

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 3: Implement SHA256 Verification

**Files:**
- Modify: `install.sh`
- Test: Manual verification with real/fake checksums

**Interfaces:**
- Consumes:
  - `say(message: string)`
  - `err(message: string)`
  - `need_cmd(command: string)`
- Produces:
  - `verify_checksum(binary_path: string, checksum_file: string) -> void` (verifies or exits)

- [ ] **Step 1: Add verify_checksum function**

```bash
# Add after get_artifact_metadata(), before main()
verify_checksum() {
    local _binary="$1"
    local _checksum_file="$2"
    local _binary_dir
    local _binary_name
    local _expected
    local _actual
    
    need_cmd sha256sum
    
    say "Verifying SHA256 checksum..."
    
    # Verify checksum file format
    if ! grep -qE '^[a-f0-9]{64} ' "$_checksum_file"; then
        err "Invalid checksum file format"
        exit 1
    fi
    
    # Change to binary directory for relative path matching
    _binary_dir=$(dirname "$_binary")
    _binary_name=$(basename "$_binary")
    
    cd "$_binary_dir" || exit 1
    
    # Verify checksum
    if sha256sum -c "$_checksum_file" >/dev/null 2>&1; then
        say "SHA256 verification passed"
        cd - >/dev/null || exit 1
    else
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

- [ ] **Step 2: Create test checksum file for verification**

```bash
# Create temp directory for testing
mkdir -p /tmp/dure-checksum-test
cd /tmp/dure-checksum-test

# Create a test binary file
echo "test binary content" > test-binary

# Generate valid checksum
sha256sum test-binary > test-binary.sha256

# Show the checksum
cat test-binary.sha256
```

- [ ] **Step 3: Test verify_checksum with valid checksum**

Temporarily add to install.sh after verify_checksum function:
```bash
# Test code (remove after testing)
if [ "${TEST_CHECKSUM:-no}" = "yes" ]; then
    verify_checksum "/tmp/dure-checksum-test/test-binary" "/tmp/dure-checksum-test/test-binary.sha256"
    say "Checksum verification test passed!"
    exit 0
fi
```

Run: `TEST_CHECKSUM=yes sh install.sh`
Expected: "info: SHA256 verification passed" and "info: Checksum verification test passed!"

- [ ] **Step 4: Test verify_checksum with invalid checksum**

```bash
# Corrupt the checksum file
echo "0000000000000000000000000000000000000000000000000000000000000000  test-binary" > /tmp/dure-checksum-test/test-binary.sha256
```

Run: `TEST_CHECKSUM=yes sh install.sh`
Expected: "error: SHA256 verification failed" with expected/actual hashes shown

- [ ] **Step 5: Test verify_checksum with invalid format**

```bash
# Invalid checksum format
echo "invalid-checksum-format" > /tmp/dure-checksum-test/test-binary.sha256
```

Run: `TEST_CHECKSUM=yes sh install.sh`
Expected: "error: Invalid checksum file format"

- [ ] **Step 6: Clean up test code and files**

Remove TEST_CHECKSUM test code from install.sh.

```bash
rm -rf /tmp/dure-checksum-test
```

- [ ] **Step 7: Commit SHA256 verification**

```bash
git add install.sh
git commit -m "feat(install): add SHA256 checksum verification

- verify_checksum() function with format validation
- Detailed error messages on verification failure
- Support for sha256sum -c standard format

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 4: Implement Installation Logic

**Files:**
- Modify: `install.sh`
- Test: Manual verification with installation to ~/.local/bin

**Interfaces:**
- Consumes:
  - `say(message: string)`
  - `warn(message: string)`
  - `err(message: string)`
- Produces:
  - `install_binary(source_path: string, dest_name: string) -> void` (installs binary or exits)

- [ ] **Step 1: Add install_binary function**

```bash
# Add after verify_checksum(), before main()
install_binary() {
    local _src="$1"
    local _dest_name="${2:-dure}"
    local _dest="$HOME/.local/bin/$_dest_name"
    local _response
    
    # Check for existing installation
    if [ -f "$_dest" ]; then
        warn "$_dest_name is already installed at $_dest"
        
        # Prompt for overwrite (skip if non-interactive)
        if [ -t 0 ]; then
            printf "Overwrite? (y/N) " >&2
            read -r _response
            case "$_response" in
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
    if ! mkdir -p "$HOME/.local/bin"; then
        err "Failed to create directory: $HOME/.local/bin"
        exit 1
    fi
    
    # Copy and set permissions
    if ! cp "$_src" "$_dest"; then
        err "Failed to install binary to $_dest"
        exit 1
    fi
    
    if ! chmod +x "$_dest"; then
        err "Failed to set executable permissions on $_dest"
        exit 1
    fi
    
    say "Installed to $_dest"
}
```

- [ ] **Step 2: Create test binary for installation**

```bash
# Create a test binary
mkdir -p /tmp/dure-install-test
echo '#!/bin/sh' > /tmp/dure-install-test/dure-desktop
echo 'echo "Test dure binary"' >> /tmp/dure-install-test/dure-desktop
chmod +x /tmp/dure-install-test/dure-desktop
```

- [ ] **Step 3: Test fresh installation**

Temporarily add to install.sh after install_binary function:
```bash
# Test code (remove after testing)
if [ "${TEST_INSTALL:-no}" = "yes" ]; then
    # Remove existing if present
    rm -f "$HOME/.local/bin/dure"
    
    install_binary "/tmp/dure-install-test/dure-desktop" "dure"
    
    # Verify installation
    if [ -x "$HOME/.local/bin/dure" ]; then
        say "Installation test passed!"
        "$HOME/.local/bin/dure"
    else
        err "Installation test failed!"
    fi
    exit 0
fi
```

Run: `TEST_INSTALL=yes sh install.sh`
Expected: 
- "info: Installed to /home/user/.local/bin/dure"
- "info: Installation test passed!"
- "Test dure binary"

- [ ] **Step 4: Test overwrite prompt (interactive)**

Run: `TEST_INSTALL=yes sh install.sh`
Expected: 
- "warn: dure is already installed at /home/user/.local/bin/dure"
- "Overwrite? (y/N) " prompt

Type: `n`
Expected: "info: Installation cancelled"

Run again: `TEST_INSTALL=yes sh install.sh`
Type: `y`
Expected: "info: Overwriting existing installation..."

- [ ] **Step 5: Test non-interactive overwrite**

Run: `echo "y" | TEST_INSTALL=yes sh install.sh`
Expected: "info: Overwriting existing installation (non-interactive mode)..."

- [ ] **Step 6: Clean up test code and files**

Remove TEST_INSTALL test code from install.sh.

```bash
rm -rf /tmp/dure-install-test
rm -f "$HOME/.local/bin/dure"
```

- [ ] **Step 7: Commit installation logic**

```bash
git add install.sh
git commit -m "feat(install): add binary installation logic

- install_binary() with overwrite confirmation
- Interactive and non-interactive mode support
- Installation to ~/.local/bin (no sudo required)
- Permission setting and error handling

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 5: Implement PATH Setup

**Files:**
- Modify: `install.sh`
- Test: Manual verification with shell config detection

**Interfaces:**
- Consumes:
  - `say(message: string)`
  - `warn(message: string)`
  - Global: `BASH_VERSION`, `ZSH_VERSION`, `HOME`, `PATH`
- Produces:
  - `setup_path() -> void` (modifies shell config or warns)

- [ ] **Step 1: Add setup_path function**

```bash
# Add after install_binary(), before main()
setup_path() {
    local _bin_dir="$HOME/.local/bin"
    local _shell_config
    local _shell_name
    
    # Check if already in PATH
    case ":$PATH:" in
        *:"$_bin_dir":*)
            say "$_bin_dir is already in PATH"
            return 0
            ;;
    esac
    
    # Detect shell config file
    if [ -n "${BASH_VERSION:-}" ]; then
        _shell_config="$HOME/.bashrc"
        _shell_name="bash"
    elif [ -n "${ZSH_VERSION:-}" ]; then
        _shell_config="$HOME/.zshrc"
        _shell_name="zsh"
    else
        # Default to .profile for POSIX shells
        _shell_config="$HOME/.profile"
        _shell_name="sh"
    fi
    
    say "Detected shell: $_shell_name"
    
    # Check if config file exists, create if not
    if [ ! -f "$_shell_config" ]; then
        if ! touch "$_shell_config"; then
            warn "Could not create $_shell_config"
            warn "Manually add to PATH: export PATH=\"\$HOME/.local/bin:\$PATH\""
            return 1
        fi
    fi
    
    # Check if PATH export already exists
    if grep -q "^export PATH=.*\.local/bin" "$_shell_config"; then
        say "PATH already configured in $_shell_config"
        return 0
    fi
    
    # Append PATH export
    if ! {
        echo ""
        echo "# Added by dure installer"
        echo "export PATH=\"\$HOME/.local/bin:\$PATH\""
    } >> "$_shell_config"; then
        warn "Could not modify $_shell_config"
        warn "Manually add to PATH: export PATH=\"\$HOME/.local/bin:\$PATH\""
        return 1
    fi
    
    say "Added to PATH in $_shell_config"
    say "Run 'source $_shell_config' or restart your shell to use dure"
}
```

- [ ] **Step 2: Test PATH already in PATH**

Temporarily add to install.sh after setup_path function:
```bash
# Test code (remove after testing)
if [ "${TEST_PATH:-no}" = "yes" ]; then
    # Temporarily add to PATH for testing
    export PATH="$HOME/.local/bin:$PATH"
    setup_path
    exit 0
fi
```

Run: `TEST_PATH=yes sh install.sh`
Expected: "info: /home/user/.local/bin is already in PATH"

- [ ] **Step 3: Test shell detection and config modification (bash)**

Remove temporary PATH from previous test.

```bash
# Backup current shell config
cp ~/.bashrc ~/.bashrc.backup 2>/dev/null || true

# Remove dure installer lines if they exist
sed -i '/# Added by dure installer/,+1d' ~/.bashrc 2>/dev/null || true
```

Run in bash: `bash install.sh` (with TEST_PATH=yes still in code)
Expected:
- "info: Detected shell: bash"
- "info: Added to PATH in /home/user/.bashrc"

Verify: `tail -3 ~/.bashrc`
Expected to see:
```
# Added by dure installer
export PATH="$HOME/.local/bin:$PATH"
```

- [ ] **Step 4: Test idempotency (running setup_path twice)**

Run in bash again: `bash install.sh` (with TEST_PATH=yes)
Expected: "info: PATH already configured in /home/user/.bashrc"

Verify: `grep -c "Added by dure installer" ~/.bashrc`
Expected: `1` (only one occurrence)

- [ ] **Step 5: Test with missing config file**

```bash
# Temporarily rename bashrc
mv ~/.bashrc ~/.bashrc.tmp 2>/dev/null || true
```

Run in bash: `bash install.sh` (with TEST_PATH=yes)
Expected:
- File created
- "info: Added to PATH in /home/user/.bashrc"

- [ ] **Step 6: Restore shell config and clean up test code**

```bash
# Restore original bashrc
rm ~/.bashrc
mv ~/.bashrc.backup ~/.bashrc 2>/dev/null || mv ~/.bashrc.tmp ~/.bashrc 2>/dev/null || true
```

Remove TEST_PATH test code from install.sh.

- [ ] **Step 7: Commit PATH setup**

```bash
git add install.sh
git commit -m "feat(install): add PATH configuration with shell detection

- setup_path() with bash/zsh/sh detection
- Automatic shell config modification
- Idempotent (checks for existing PATH configuration)
- Non-fatal warnings on failure

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 6: Integrate Installation Modes (Stable and Dev)

**Files:**
- Modify: `install.sh`
- Test: End-to-end installation in both modes

**Interfaces:**
- Consumes: All previously defined functions
- Produces: Complete installation flow

- [ ] **Step 1: Add stable mode installation flow**

```bash
# Add after setup_path(), before main()
install_from_release() {
    local _metadata
    local _binary_url
    local _checksum_url
    local _temp_dir
    local _binary_path
    local _checksum_path
    
    say "Running in stable mode"
    say "Fetching latest release from GitHub..."
    
    # Get release metadata
    _metadata=$(get_release_metadata)
    
    # Parse binary and checksum URLs from release assets
    # Expected asset names: "dure-desktop-linux" and "dure-desktop-linux.sha256"
    _binary_url=$(echo "$_metadata" | grep -o '"browser_download_url"[[:space:]]*:[[:space:]]*"[^"]*dure-desktop-linux"' | grep -o 'https://[^"]*' | grep -v '\.sha256$' | head -1)
    _checksum_url=$(echo "$_metadata" | grep -o '"browser_download_url"[[:space:]]*:[[:space:]]*"[^"]*dure-desktop-linux\.sha256"' | grep -o 'https://[^"]*' | head -1)
    
    if [ -z "$_binary_url" ]; then
        err "Could not find dure-desktop-linux in release assets"
        err "This release may be incomplete."
        exit 1
    fi
    
    if [ -z "$_checksum_url" ]; then
        err "Could not find dure-desktop-linux.sha256 in release assets"
        err "This release may be incomplete."
        exit 1
    fi
    
    # Create temp directory
    _temp_dir=$(mktemp -d -t dure-install.XXXXXXXXXX)
    
    # Setup cleanup trap
    trap "rm -rf '$_temp_dir'" EXIT
    
    _binary_path="$_temp_dir/dure-desktop-linux"
    _checksum_path="$_temp_dir/dure-desktop-linux.sha256"
    
    # Download binary and checksum
    downloader "$_binary_url" "$_binary_path"
    downloader "$_checksum_url" "$_checksum_path"
    
    # Verify checksum
    verify_checksum "$_binary_path" "$_checksum_path"
    
    # Install binary
    install_binary "$_binary_path" "dure"
    
    # Setup PATH
    setup_path
    
    say "Installation complete!"
    say "Run 'dure --version' to verify installation"
}
```

- [ ] **Step 2: Add dev mode installation flow**

```bash
# Add after install_from_release(), before main()
install_from_artifacts() {
    local _metadata
    local _artifact_url
    local _artifact_digest
    local _checksum_url
    local _temp_dir
    local _binary_path
    local _checksum_path
    local _artifact_zip
    
    say "Running in dev mode"
    say "Fetching latest artifacts from GitHub..."
    
    # Get artifact metadata
    _metadata=$(get_artifact_metadata)
    
    # Filter for dure-desktop-linux artifact (latest)
    # Parse JSON manually (no jq dependency)
    _artifact_url=$(echo "$_metadata" | grep -B5 '"name"[[:space:]]*:[[:space:]]*"dure-desktop-linux"' | grep '"archive_download_url"' | head -1 | grep -o 'https://[^"]*')
    _artifact_digest=$(echo "$_metadata" | grep -B5 '"name"[[:space:]]*:[[:space:]]*"dure-desktop-linux"' | grep '"digest"' | head -1 | grep -o 'sha256:[a-f0-9]*' | cut -d: -f2)
    
    if [ -z "$_artifact_url" ]; then
        err "Could not find dure-desktop-linux artifact"
        err "No development builds available."
        exit 1
    fi
    
    if [ -z "$_artifact_digest" ]; then
        warn "Could not find digest for artifact, will download checksum file"
    fi
    
    # Create temp directory
    _temp_dir=$(mktemp -d -t dure-install.XXXXXXXXXX)
    
    # Setup cleanup trap
    trap "rm -rf '$_temp_dir'" EXIT
    
    _artifact_zip="$_temp_dir/artifact.zip"
    _binary_path="$_temp_dir/dure-desktop"
    _checksum_path="$_temp_dir/dure-desktop-linux.sha256"
    
    # Download artifact (it's a zip file)
    need_cmd unzip
    downloader "$_artifact_url" "$_artifact_zip"
    
    # Extract artifact
    say "Extracting artifact..."
    if ! unzip -q "$_artifact_zip" -d "$_temp_dir"; then
        err "Failed to extract artifact"
        exit 1
    fi
    
    # The artifact should contain dure-desktop and dure-desktop-linux.sha256
    if [ ! -f "$_binary_path" ]; then
        err "Binary not found in artifact: dure-desktop"
        exit 1
    fi
    
    if [ ! -f "$_checksum_path" ]; then
        # If checksum file not in artifact, create one from digest
        if [ -n "$_artifact_digest" ]; then
            say "Using digest from API"
            echo "$_artifact_digest  dure-desktop" > "$_checksum_path"
        else
            err "Checksum file not found and no digest available"
            exit 1
        fi
    fi
    
    # Verify checksum
    verify_checksum "$_binary_path" "$_checksum_path"
    
    # Install binary
    install_binary "$_binary_path" "dure"
    
    # Setup PATH
    setup_path
    
    say "Installation complete!"
    say "Run 'dure --version' to verify installation"
}
```

- [ ] **Step 3: Update main() to call installation flows**

```bash
# Replace existing main() function
main() {
    local _arch
    
    say "Dure installer"
    
    # Check dependencies
    need_cmd uname
    need_cmd mktemp
    need_cmd chmod
    need_cmd mkdir
    need_cmd rm
    need_cmd sha256sum
    
    # Check downloader
    if ! check_cmd curl && ! check_cmd wget; then
        err "Neither curl nor wget found. Please install one and try again."
        exit 1
    fi
    
    # Detect platform
    _arch=$(get_architecture)
    say "Detected platform: $_arch"
    
    say "Channel: $CHANNEL"
    
    # Run installation based on channel
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
}
```

- [ ] **Step 4: Test dev mode installation (full end-to-end)**

```bash
# Remove existing dure if present
rm -f ~/.local/bin/dure

# Remove PATH modification from shell config
sed -i '/# Added by dure installer/,+1d' ~/.bashrc 2>/dev/null || true
```

Run: `DURE_CHANNEL=dev sh install.sh`

Expected output flow:
1. "info: Dure installer"
2. "info: Detected platform: x86_64-unknown-linux-gnu"
3. "info: Channel: dev"
4. "info: Running in dev mode"
5. "info: Fetching latest artifacts from GitHub..."
6. Download and extraction messages
7. "info: SHA256 verification passed"
8. "info: Installed to /home/user/.local/bin/dure"
9. "info: Added to PATH in /home/user/.bashrc"
10. "info: Installation complete!"

Verify: `ls -la ~/.local/bin/dure`
Expected: File exists and is executable

Verify: `tail -3 ~/.bashrc`
Expected: PATH export present

- [ ] **Step 5: Test stable mode (should fail since no releases)**

Run: `sh install.sh`
Expected: "error: No stable releases available yet"

- [ ] **Step 6: Test overwrite behavior**

Run dev mode again: `DURE_CHANNEL=dev sh install.sh`
Type `n` when prompted
Expected: "info: Installation cancelled"

Run again: `DURE_CHANNEL=dev sh install.sh`
Type `y` when prompted
Expected: Installation completes successfully

- [ ] **Step 7: Make install.sh executable**

```bash
chmod +x install.sh
```

Test running directly: `DURE_CHANNEL=dev ./install.sh`
Expected: Works the same as `sh install.sh`

- [ ] **Step 8: Commit integrated installation flows**

```bash
git add install.sh
git commit -m "feat(install): integrate stable and dev mode installation flows

- install_from_release() for stable mode (GitHub releases)
- install_from_artifacts() for dev mode (GitHub Actions artifacts)  
- Complete end-to-end installation workflow
- Proper cleanup with trap on EXIT
- Artifact extraction support (unzip)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 7: Add Deployment Configuration

**Files:**
- Modify: `_config.yml`
- Modify: `.github/workflows/release.yml`
- Test: Deploy to docs site and test remote installation

**Interfaces:**
- Consumes: Completed `install.sh`
- Produces: Deployed script at https://dure.one/install.sh, automated checksum generation

- [ ] **Step 1: Add install.sh to _config.yml**

```yaml
# In _config.yml, find the 'include:' section and add install.sh
# File: _config.yml
# Location: around line 40-46

include:
  - .well-known
  - index.md
  - fastlane
  - README.md
  - install.sh          # ← Add this line
```

- [ ] **Step 2: Verify _config.yml syntax**

Run: `grep -A6 "^include:" _config.yml`
Expected: See install.sh in the list

- [ ] **Step 3: Commit _config.yml changes**

```bash
git add _config.yml
git commit -m "feat(docs): add install.sh to deployment configuration

Include install.sh in docs site deployment for remote installation
support via curl https://dure.one/install.sh | sh

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

- [ ] **Step 4: Add checksum generation to release workflow**

```yaml
# In .github/workflows/release.yml
# Add after the "Build release binary" step in build-linux job
# Location: after line 36

      - name: Generate checksum
        run: |
          cd mobile
          sha256sum target/release/dure-desktop > dure-desktop-linux.sha256

      - name: Upload checksum artifact
        uses: actions/upload-artifact@v4
        with:
          name: dure-desktop-linux-checksum
          path: mobile/dure-desktop-linux.sha256
```

- [ ] **Step 5: Update create-release job to include checksums**

```yaml
# In .github/workflows/release.yml
# Modify the create-release job's files section
# Location: around line 112

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            dure-desktop-linux/dure-desktop
            dure-desktop-linux-checksum/dure-desktop-linux.sha256
            dure-desktop-windows/dure-desktop.exe
            dure-desktop-macos/dure-desktop
          generate_release_notes: true
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

- [ ] **Step 6: Update artifact uploads to include checksums (for dev mode)**

```yaml
# In .github/workflows/release.yml
# Modify build-linux job's upload artifact step
# Location: around line 38-42

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: dure-desktop-linux
          path: |
            mobile/target/release/dure-desktop
            mobile/dure-desktop-linux.sha256
```

- [ ] **Step 7: Commit release workflow changes**

```bash
git add .github/workflows/release.yml
git commit -m "feat(ci): add checksum generation to release workflow

- Generate SHA256 checksums for all builds
- Upload checksums as release assets (stable mode)
- Include checksums in artifacts (dev mode)
- Supports install.sh verification requirements

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

- [ ] **Step 8: Push all changes and trigger deployment**

```bash
git push origin main
```

Expected: 
- GitHub Actions workflow triggers
- vite.docs.yml deploys to Cloudflare Pages
- install.sh available at https://dure.one/install.sh

- [ ] **Step 9: Wait for deployment and test remote installation**

Wait for GitHub Actions to complete (check https://github.com/nikescar/dure/actions)

Once deployed, test remote installation:
```bash
# Remove existing installation
rm -f ~/.local/bin/dure
sed -i '/# Added by dure installer/,+1d' ~/.bashrc 2>/dev/null || true

# Test dev mode installation
curl --proto '=https' --tlsv1.2 -sSf https://dure.one/install.sh | DURE_CHANNEL=dev sh
```

Expected:
- Script downloads and runs
- Binary installed to ~/.local/bin/dure
- PATH configured in ~/.bashrc

- [ ] **Step 10: Test quiet mode**

```bash
# Remove installation
rm -f ~/.local/bin/dure
sed -i '/# Added by dure installer/,+1d' ~/.bashrc 2>/dev/null || true

# Test quiet mode
curl --proto '=https' --tlsv1.2 -sSf https://dure.one/install.sh | DURE_CHANNEL=dev DURE_QUIET=1 sh
```

Expected: Minimal output, only errors/warnings

- [ ] **Step 11: Verify installed binary works**

```bash
# Source bashrc to get PATH
source ~/.bashrc

# Test binary
dure --version
```

Expected: Dure version output (or error if binary doesn't support --version yet)

- [ ] **Step 12: Update documentation**

Create a commit with documentation updates:

```bash
# Edit docs/INSTALLING.md - add Quick Install section at the top
```

Add this section after "# Installation Guide":

```markdown
## Quick Install (Recommended)

Install dure with a single command:

### Stable Release (Recommended)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://dure.one/install.sh | sh
```

### Development Build (Latest)
```bash
curl --proto '=https' --tlsv1.2 -sSf https://dure.one/install.sh | DURE_CHANNEL=dev sh
```

### Options

- **Quiet mode:** Add `DURE_QUIET=1` to suppress output
- **Custom channel:** Set `DURE_CHANNEL=dev` for development builds

The installer will:
1. Detect your platform (Linux x86_64 only)
2. Download and verify the binary with SHA256
3. Install to `~/.local/bin/dure`
4. Add to your PATH automatically

**Note:** Currently only supports Linux x86_64. arm64/aarch64 support coming soon.

---
```

- [ ] **Step 13: Commit documentation updates**

```bash
git add docs/INSTALLING.md
git commit -m "docs: add quick install section to installation guide

Show one-line remote installation command for both stable and dev
modes. Explain options and requirements.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

- [ ] **Step 14: Push documentation updates**

```bash
git push origin main
```

- [ ] **Step 15: Final verification**

After docs deployment completes, verify the documentation:

1. Visit https://dure.one/
2. Navigate to installation docs
3. Verify Quick Install section is visible
4. Test copy-paste of installation command

Expected: All documentation is live and accurate

---

## Completion Checklist

- [ ] install.sh script fully functional
- [ ] Stable mode (GitHub releases) works correctly
- [ ] Dev mode (GitHub Actions artifacts) works correctly  
- [ ] SHA256 verification prevents corrupted downloads
- [ ] Platform detection (Linux x86_64, fail on others)
- [ ] PATH automatically configured for bash/zsh/sh
- [ ] Output functions with verbose/quiet modes
- [ ] Overwrite confirmation for existing installations
- [ ] Deployed to https://dure.one/install.sh
- [ ] Release workflow generates checksums automatically
- [ ] Documentation updated with installation instructions
- [ ] End-to-end testing completed successfully

## Success Criteria

The implementation is successful when:

- ✅ Users can install dure with: `curl https://dure.one/install.sh | sh`
- ✅ Both stable and dev modes work correctly
- ✅ SHA256 verification catches corrupted downloads
- ✅ Installation completes in under 30 seconds
- ✅ PATH is configured automatically
- ✅ Clear error messages guide users through problems
- ✅ Script is POSIX-compatible and works across shells
- ✅ Zero security vulnerabilities (HTTPS, verification, user directory)
