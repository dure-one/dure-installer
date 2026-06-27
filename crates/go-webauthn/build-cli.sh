#!/usr/bin/env bash
# Build go-webauthn-cli executable

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo "Building go-webauthn-cli..."

# Create bin directory
mkdir -p bin

# Build from go directory (where go.mod is)
cd go
go build -o ../bin/go-webauthn-cli ../cmd/main.go

echo "✅ Built successfully: $SCRIPT_DIR/bin/go-webauthn-cli"

# Make it executable
chmod +x ../bin/go-webauthn-cli

# Test it
echo ""
echo "Testing CLI..."
echo '{"id":"test","method":"ed25519.generateKey"}' | ../bin/go-webauthn-cli 2>/dev/null | head -1 | grep -q "public_key" && echo "✅ CLI test passed!" || echo "❌ CLI test failed"
