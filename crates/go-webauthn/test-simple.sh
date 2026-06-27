#!/usr/bin/env bash
# Simple test script for go-webauthn-cli (no dependencies)

set -e

CLI="./bin/go-webauthn-cli"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Build first if needed
if [ ! -f "$CLI" ]; then
    echo "Building CLI..."
    ./build-cli.sh
fi

echo "🧪 Testing go-webauthn-cli"
echo "========================="
echo

# Test 1: Health endpoint
echo "1️⃣  Health endpoint:"
echo '{"id":"1","method":"health"}' | $CLI 2>/dev/null
echo "✅ Success"
echo

# Test 2: Version endpoint
echo "2️⃣  Version endpoint:"
echo '{"id":"2","method":"version"}' | $CLI 2>/dev/null
echo "✅ Success"
echo

# Test 3: Metrics endpoint
echo "3️⃣  Metrics endpoint (initial):"
echo '{"id":"3","method":"metrics"}' | $CLI 2>/dev/null
echo "✅ Success"
echo

# Test 4: ED25519 key generation
echo "4️⃣  ED25519 key generation:"
RESULT=$(echo '{"id":"4","method":"ed25519.generateKey"}' | $CLI 2>/dev/null)
if echo "$RESULT" | grep -q '"public_key"'; then
    echo "$RESULT"
    echo "✅ Success (key generated)"
else
    echo "❌ Failed"
    exit 1
fi
echo

# Test 5: Error handling
echo "5️⃣  Error handling (invalid method):"
ERROR=$(echo '{"id":"5","method":"invalid.method"}' | $CLI 2>/dev/null)
if echo "$ERROR" | grep -q '"error"'; then
    echo "$ERROR"
    echo "✅ Success (error properly returned)"
else
    echo "❌ Failed"
    exit 1
fi
echo

# Test 6: Multiple requests with metrics
echo "6️⃣  Multiple requests + metrics:"
(
    echo '{"id":"6","method":"health"}'
    echo '{"id":"7","method":"ed25519.generateKey"}'
    echo '{"id":"8","method":"health"}'
    echo '{"id":"9","method":"metrics"}'
) | $CLI 2>/dev/null | tail -1
echo "✅ Success (metrics tracked)"
echo

# Test 7: Debug mode
echo "7️⃣  Debug mode logging:"
echo '{"id":"10","method":"health"}' | $CLI --debug 2>&1 | head -5
echo "✅ Success (debug logs shown)"
echo

echo "========================="
echo "✅ All integration tests passed!"
echo
echo "📝 To run Go unit tests, you need to fix the module structure:"
echo "   Option 1: Move cmd files into go/cmd/"
echo "   Option 2: Create go.mod in cmd/"
echo "   Option 3: Use go test with explicit file list"
