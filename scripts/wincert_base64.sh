#!/usr/bin/env bash

# .github/workflows/release.yml
# WINDOWS_CERT_P12_BASE64: ${{ secrets.WINDOWS_CERT_P12_BASE64 }}
# WINDOWS_CERT_PASSWORD: ${{ secrets.WINDOWS_CERT_PASSWORD }}

# OpenBSD base64 doesn't support -i flag
# Use positional argument for input file
INPUT_FILE="${1:-certificate.pfx}"
OUTPUT_FILE="${2:-}"

if [ ! -f "$INPUT_FILE" ]; then
    echo "Error: Input file '$INPUT_FILE' not found" >&2
    exit 1
fi

if [ -n "$OUTPUT_FILE" ]; then
    base64 "$INPUT_FILE" > "$OUTPUT_FILE"
else
    base64 "$INPUT_FILE"
fi