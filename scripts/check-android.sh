#!/bin/bash
# Local Android build check - catches target-specific errors before CI

set -e

echo "🤖 Checking Android target build..."
cd mobile && cargo check --target x86_64-linux-android --lib
echo "✅ Android build check passed"
