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

main() {
    local _arch

    say "Dure installer"

    # Detect platform
    _arch=$(get_architecture)
    say "Detected platform: $_arch"

    say "Channel: $CHANNEL"
}

main "$@"
