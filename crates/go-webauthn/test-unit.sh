#!/usr/bin/env bash
# Run Go unit tests for cmd package

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR/cmd"

echo "🧪 Running Go unit tests"
echo "======================="
echo

# Create temporary go.mod for testing
cat > go.mod.tmp << 'EOF'
module dure/webauthn/cmd

go 1.25.0

require (
	github.com/go-webauthn/webauthn v0.16.4
	github.com/google/uuid v1.6.0
)
EOF

# Run tests with temporary module
mv go.mod.tmp go.mod
trap "rm -f go.mod go.sum" EXIT

echo "Setting up dependencies..."
go mod tidy

echo "Running tests..."
go test -v .

echo
echo "✅ Unit tests completed!"
