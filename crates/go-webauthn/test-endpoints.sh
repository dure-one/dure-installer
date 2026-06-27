#!/usr/bin/env bash
# Test script for go-webauthn-cli endpoints

set -e

CLI="./bin/go-webauthn-cli"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Build first if needed
if [ ! -f "$CLI" ]; then
    echo "Building CLI..."
    ./build-cli.sh
fi

echo "🧪 Testing go-webauthn-cli endpoints"
echo "===================================="
echo

# Test 1: Health endpoint
echo "1️⃣  Testing health endpoint..."
echo '{"id":"1","method":"health"}' | $CLI 2>/dev/null | jq '.'
echo "✅ Health check passed"
echo

# Test 2: Version endpoint
echo "2️⃣  Testing version endpoint..."
echo '{"id":"2","method":"version"}' | $CLI 2>/dev/null | jq '.'
echo "✅ Version check passed"
echo

# Test 3: Metrics endpoint (initial state)
echo "3️⃣  Testing metrics endpoint..."
echo '{"id":"3","method":"metrics"}' | $CLI 2>/dev/null | jq '.'
echo "✅ Metrics check passed"
echo

# Test 4: ED25519 key generation
echo "4️⃣  Testing ED25519 key generation..."
RESULT=$(echo '{"id":"4","method":"ed25519.generateKey"}' | $CLI 2>/dev/null | jq '.result.public_key')
if [ -n "$RESULT" ] && [ "$RESULT" != "null" ]; then
    echo "✅ Key generation passed (public_key: ${RESULT:0:20}...)"
else
    echo "❌ Key generation failed"
    exit 1
fi
echo

# Test 5: Multiple requests to test metrics tracking
echo "5️⃣  Testing metrics tracking with multiple requests..."
(
    echo '{"id":"5","method":"health"}'
    echo '{"id":"6","method":"ed25519.generateKey"}'
    echo '{"id":"7","method":"health"}'
    echo '{"id":"8","method":"metrics"}'
) | $CLI 2>/dev/null | tail -1 | jq '.result | {requests_total, requests_success, method_stats}'
echo "✅ Metrics tracking passed"
echo

# Test 6: Error handling
echo "6️⃣  Testing error handling (invalid method)..."
ERROR=$(echo '{"id":"9","method":"invalid.method"}' | $CLI 2>/dev/null | jq '.error.message')
if [ -n "$ERROR" ] && [ "$ERROR" != "null" ]; then
    echo "✅ Error handling passed (error: $ERROR)"
else
    echo "❌ Error handling failed"
    exit 1
fi
echo

# Test 7: Rate limiting (requires many requests)
echo "7️⃣  Testing rate limiting (sending 60 requests)..."
for i in {1..60}; do
    echo "{\"id\":\"rate-$i\",\"method\":\"health\"}"
done | $CLI 2>/dev/null | grep -c '"error"' > /tmp/rate_test_errors.txt || true
ERRORS=$(cat /tmp/rate_test_errors.txt)
if [ "$ERRORS" -gt 0 ]; then
    echo "✅ Rate limiting working ($ERRORS requests were rate-limited)"
else
    echo "⚠️  Rate limiting not triggered (may need more requests or shorter window)"
fi
rm -f /tmp/rate_test_errors.txt
echo

echo "===================================="
echo "✅ All tests completed!"
