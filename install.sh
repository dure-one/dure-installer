#!/bin/sh
# shellcheck shell=dash
# shellcheck disable=SC2039  # local is non-POSIX

set -u

# Configuration
# Allow users to override with their own token via environment variable
GITHUB_TOKEN="${GITHUB_TOKEN:-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}"
REPO_OWNER="nikescar"
REPO_NAME="dure"
CHANNEL="${DURE_CHANNEL:-stable}"
VARIANT="${DURE_VARIANT:-headless}"  # headless or gui
QUIET="${DURE_QUIET:-no}"

# ANSI color detection
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
                 -H "X-GitHub-Api-Version: 2022-11-28" \
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
                 --header="X-GitHub-Api-Version: 2022-11-28" \
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

get_release_metadata() {
    local _url="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/releases/latest"
    local _temp_file
    local _response
    local _status

    _temp_file=$(mktemp)

    # Public releases don't need authentication, so try without token first
    say "Downloading: $_url"
    if check_cmd curl; then
        curl --proto '=https' --tlsv1.2 -sSL \
             -H "Accept: application/vnd.github+json" \
             -o "$_temp_file" "$_url" 2>/dev/null
        _status=$?
    elif check_cmd wget; then
        wget --https-only --quiet \
             --header="Accept: application/vnd.github+json" \
             -O "$_temp_file" "$_url" 2>/dev/null
        _status=$?
    fi

    # Check for HTTP errors (404, etc.)
    if [ $_status -ne 0 ] || [ ! -s "$_temp_file" ]; then
        rm "$_temp_file"
        err "No stable releases available yet"
        err "Try: DURE_CHANNEL=dev $0"
        exit 1
    fi

    _response=$(cat "$_temp_file")
    rm "$_temp_file"

    # Check if release exists
    if echo "$_response" | grep -q '"message":"Not Found"'; then
        err "No stable releases available yet"
        err "Try: DURE_CHANNEL=dev $0"
        exit 1
    fi

    say "Release metadata retrieved"
    echo "$_response"
}

get_artifact_metadata() {
    local _artifact_name="$1"
    local _url="https://api.github.com/repos/$REPO_OWNER/$REPO_NAME/actions/artifacts"
    local _temp_file
    local _response
    local _status

    if [ -z "$GITHUB_TOKEN" ]; then
        err "GitHub token required for dev mode"
        err "Set GITHUB_TOKEN environment variable with a valid token"
        exit 1
    fi

    # Filter by artifact name and get only the latest (per_page=1)
    if [ -n "$_artifact_name" ]; then
        _url="${_url}?name=${_artifact_name}&per_page=1"
    fi

    _temp_file=$(mktemp)

    # Artifacts endpoint requires authentication
    say "Downloading: $_url"
    if check_cmd curl; then
        curl --proto '=https' --tlsv1.2 -fsSL \
             -H "Accept: application/vnd.github+json" \
             -H "Authorization: Bearer $GITHUB_TOKEN" \
             -H "X-GitHub-Api-Version: 2022-11-28" \
             -o "$_temp_file" "$_url"
        _status=$?
    elif check_cmd wget; then
        wget --https-only --quiet \
             --header="Accept: application/vnd.github+json" \
             --header="Authorization: Bearer $GITHUB_TOKEN" \
             --header="X-GitHub-Api-Version: 2022-11-28" \
             -O "$_temp_file" "$_url"
        _status=$?
    fi

    if [ $_status -ne 0 ]; then
        err "Download failed: $_url"
        err "This may indicate an invalid or expired GitHub token"
        err "Try setting GITHUB_TOKEN environment variable with a valid token"
        rm "$_temp_file"
        exit 1
    fi

    _response=$(cat "$_temp_file")
    rm "$_temp_file"

    say "Downloaded to: $_temp_file"
    echo "$_response"
}

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

install_from_release() {
    local _metadata
    local _binary_url
    local _checksum_url
    local _temp_dir
    local _binary_path
    local _checksum_path
    local _asset_name
    local _binary_filename

    say "Running in stable mode"
    say "Variant: $VARIANT"
    say "Fetching latest release from GitHub..."

    # Get release metadata
    _metadata=$(get_release_metadata)

    # Look for x86_64-unknown-linux-musl binary (both headless and GUI use musl)
    _binary_filename="dure-desktop"

    if [ "$VARIANT" = "headless" ]; then
        # Look for headless variant (has -headless in artifact name)
        _binary_url=$(echo "$_metadata" | grep -o '"browser_download_url"[[:space:]]*:[[:space:]]*"[^"]*dure-desktop"' | grep -o 'https://[^"]*' | grep -v '\.sha256$' | grep 'x86_64-unknown-linux-musl-headless' | head -1)
        _checksum_url=$(echo "$_metadata" | grep -o '"browser_download_url"[[:space:]]*:[[:space:]]*"[^"]*dure-desktop\.sha256"' | grep -o 'https://[^"]*' | grep 'x86_64-unknown-linux-musl-headless' | head -1)
    else
        # Look for GUI variant (no -headless suffix)
        _binary_url=$(echo "$_metadata" | grep -o '"browser_download_url"[[:space:]]*:[[:space:]]*"[^"]*dure-desktop"' | grep -o 'https://[^"]*' | grep -v '\.sha256$' | grep 'x86_64-unknown-linux-musl' | grep -v 'headless' | head -1)
        _checksum_url=$(echo "$_metadata" | grep -o '"browser_download_url"[[:space:]]*:[[:space:]]*"[^"]*dure-desktop\.sha256"' | grep -o 'https://[^"]*' | grep 'x86_64-unknown-linux-musl' | grep -v 'headless' | head -1)
    fi

    if [ -z "$_binary_url" ]; then
        err "Could not find $VARIANT variant in release assets"
        if [ "$VARIANT" = "gui" ]; then
            err "Try headless variant: DURE_VARIANT=headless $0"
        fi
        exit 1
    fi

    if [ -z "$_checksum_url" ]; then
        err "Could not find checksum file in release assets"
        err "This release may be incomplete."
        exit 1
    fi

    # Create temp directory
    _temp_dir=$(mktemp -d -t dure-install.XXXXXXXXXX)

    # Setup cleanup trap
    trap "rm -rf '$_temp_dir'" EXIT

    _binary_path="$_temp_dir/$_binary_filename"
    _checksum_path="$_temp_dir/$_asset_name.sha256"

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
    if [ "$VARIANT" = "headless" ]; then
        say "Installed musl headless binary (no GUI, no dependencies)"
    else
        say "Installed musl GUI binary (with GTK support)"
    fi
    say "Works on any Linux distribution (no GLIBC version issues)"
    say "Run 'dure --version' to verify installation"
}

install_from_artifacts() {
    local _metadata
    local _artifact_url
    local _artifact_digest
    local _artifact_id
    local _checksum_url
    local _temp_dir
    local _binary_path
    local _checksum_path
    local _artifact_zip
    local _artifact_name
    local _binary_filename
    local _checksum_filename

    say "Running in dev mode"
    say "Variant: $VARIANT"

    # Linux uses musl for both headless and GUI
    if [ "$VARIANT" = "headless" ]; then
        _artifact_name="dure-desktop-x86_64-unknown-linux-musl-headless"
    else
        _artifact_name="dure-desktop-x86_64-unknown-linux-musl"
    fi
    _binary_filename="dure-desktop"
    _checksum_filename="dure-desktop.sha256"

    say "Fetching latest $_artifact_name artifact from GitHub..."

    # Get artifact metadata (filtered by name, returns only latest)
    _metadata=$(get_artifact_metadata "$_artifact_name")

    # Check if we got any artifacts
    if ! echo "$_metadata" | grep -q '"total_count"[[:space:]]*:[[:space:]]*[1-9]'; then
        err "Could not find $_artifact_name artifact"
        if [ "$VARIANT" = "gui" ]; then
            err "Try headless variant: DURE_VARIANT=headless DURE_CHANNEL=dev $0"
        fi
        exit 1
    fi

    # Parse the first (and only) artifact from the filtered results
    _artifact_url=$(echo "$_metadata" | grep '"archive_download_url"' | head -1 | sed 's/.*"archive_download_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/')
    _artifact_digest=$(echo "$_metadata" | grep '"digest"' | head -1 | sed 's/.*"digest"[[:space:]]*:[[:space:]]*"sha256:\([a-f0-9]*\)".*/\1/')
    _artifact_id=$(echo "$_metadata" | grep '"id"' | head -1 | sed 's/.*"id"[[:space:]]*:[[:space:]]*\([0-9]*\).*/\1/')

    if [ -z "$_artifact_url" ]; then
        err "Could not find artifact download URL"
        err "No development builds available."
        exit 1
    fi

    # Show which artifact was selected
    if [ -n "$_artifact_id" ]; then
        say "Selected artifact ID: $_artifact_id"
    fi

    if [ -z "$_artifact_digest" ]; then
        warn "Could not find digest for artifact, will download checksum file"
    fi

    # Create temp directory
    _temp_dir=$(mktemp -d -t dure-install.XXXXXXXXXX)

    # Setup cleanup trap
    trap "rm -rf '$_temp_dir'" EXIT

    _artifact_zip="$_temp_dir/artifact.zip"
    _binary_path="$_temp_dir/$_binary_filename"
    _checksum_path="$_temp_dir/$_checksum_filename"

    # Download artifact (it's a zip file)
    need_cmd unzip
    downloader "$_artifact_url" "$_artifact_zip"

    # Extract artifact
    say "Extracting artifact..."
    if ! unzip -q "$_artifact_zip" -d "$_temp_dir"; then
        err "Failed to extract artifact"
        exit 1
    fi

    # The artifact should contain binary and checksum
    if [ ! -f "$_binary_path" ]; then
        err "Binary not found in artifact: $_binary_filename"
        exit 1
    fi

    if [ ! -f "$_checksum_path" ]; then
        # If checksum file not in artifact, create one from digest
        if [ -n "$_artifact_digest" ]; then
            say "Using digest from API"
            echo "$_artifact_digest  $_binary_filename" > "$_checksum_path"
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
    if [ "$VARIANT" = "headless" ]; then
        say "Installed musl headless binary (no GUI, no dependencies)"
    else
        say "Installed musl GUI binary (with GTK support)"
    fi
    say "Works on any Linux distribution (no GLIBC version issues)"
    say "Run 'dure --version' to verify installation"
}

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

    # Validate variant
    case "$VARIANT" in
        headless|gui)
            say "Variant: $VARIANT"
            ;;
        *)
            err "Invalid DURE_VARIANT: $VARIANT"
            err "Valid options: headless (default), gui"
            exit 1
            ;;
    esac

    say "Target: x86_64-unknown-linux-musl (all variants use musl)"

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

main "$@"
