#!/bin/bash
# Demo script to show new logging format

cd "$(dirname "$0")"

echo "Building and testing new logging format..."
echo ""

cd mobile
cargo run --bin dure-desktop -- --help 2>&1 | head -15

echo ""
echo "Expected format: 20260729T201132 [INFO] [DureDeskt] Dure v0.0.1 starting..."
echo "Note: Module names truncated to 10 chars, compact ISO dates, single spaces"
