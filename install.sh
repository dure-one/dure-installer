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

main "$@"
