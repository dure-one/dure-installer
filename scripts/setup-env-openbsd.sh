#!/usr/bin/env bash
# Setup script for OpenBSD development environment
#
# This script sets environment variables needed for building on OpenBSD.
# Source this file in your shell or add to ~/.bashrc:
#
#   source /path/to/dure/scripts/setup-env-openbsd.sh
#
# Or use direnv with .envrc (recommended) instead.

if [[ "$(uname -s)" == "OpenBSD" ]]; then
    export OPENSSL_LIB_DIR="/usr/local/lib/eopenssl35"
    export OPENSSL_INCLUDE_DIR="/usr/local/include/eopenssl35"
    echo "✓ OpenBSD environment configured (OpenSSL 3.5.7)"
else
    echo "⚠ Not on OpenBSD - no environment changes needed"
fi
