# Testing go-webauthn-cli

## Overview

The go-webauthn-cli has two types of tests:
1. **Unit tests** - Test individual components (errors, metrics, rate limiting)
2. **Integration tests** - Test end-to-end functionality via JSON-RPC

## Quick Start

```bash
# Run all tests
./test-unit.sh          # Go unit tests
./test-simple.sh        # Integration tests (no dependencies)
./test-endpoints.sh     # Integration tests (requires jq)
```

## Unit Tests

Tests for individual components using Go's testing framework.

**Location:** `cmd/*_test.go`

**Coverage:**
- ✅ `errors_test.go` - Error code system, DetailedError types
- ✅ `metrics_test.go` - Request tracking, latency calculation, NaN handling
- ✅ `ratelimit_test.go` - Token bucket algorithm, per-user/method limits

**Run:**
```bash
./test-unit.sh
```

**Results:**
```
=== RUN   TestDetailedErrorCreation
--- PASS: TestDetailedErrorCreation (0.00s)
=== RUN   TestMetricsRecordRequest
--- PASS: TestMetricsRecordRequest (0.00s)
=== RUN   TestRateLimiterAllow
--- PASS: TestRateLimiterAllow (0.00s)
...
PASS
ok  	dure/webauthn/cmd	0.162s
```

## Integration Tests

Test the CLI via its JSON-RPC interface over stdin/stdout.

### Simple Tests (No Dependencies)

**File:** `test-simple.sh`

Tests all endpoints without requiring external tools:

```bash
./test-simple.sh
```

**What it tests:**
- ✅ Health endpoint
- ✅ Version endpoint  
- ✅ Metrics endpoint
- ✅ ED25519 key generation
- ✅ Error handling
- ✅ Metrics tracking
- ✅ Debug mode logging

**Sample output:**
```
🧪 Testing go-webauthn-cli
=========================

1️⃣  Health endpoint:
{"id":"1","result":{"status":"ok","uptime_seconds":0.00017279,"version":"2.0.0"}}
✅ Success

2️⃣  Version endpoint:
{"id":"2","result":{"version":"2.0.0","go_version":"go1.26.4","build_date":"2026-06-26"}}
✅ Success
...
```

### Advanced Tests (Requires jq)

**File:** `test-endpoints.sh`

Pretty-printed output using jq for better readability:

```bash
./test-endpoints.sh
```

Includes rate limiting test (sends 60 requests to trigger limits).

## Manual Testing

### Basic Request

```bash
echo '{"id":"1","method":"health"}' | ./bin/go-webauthn-cli
```

**Output:**
```json
{"id":"1","result":{"status":"ok","uptime_seconds":0.000172,"version":"2.0.0"}}
```

### Debug Mode

```bash
echo '{"id":"1","method":"health"}' | ./bin/go-webauthn-cli --debug
```

**Output:**
```
[2026-06-26T21:50:01.471-06:00] INFO: Debug mode enabled
[2026-06-26T21:50:01.471-06:00] INFO: go-webauthn JSON-RPC server starting
[2026-06-26T21:50:01.471-06:00] DEBUG: Request received | method=health request_id=1
[2026-06-26T21:50:01.471-06:00] INFO: Request completed | method=health request_id=1 duration_ms=0 success=true
{"id":"1","result":{"status":"ok","uptime_seconds":0.000111,"version":"2.0.0"}}
```

### Multiple Requests

```bash
(
  echo '{"id":"1","method":"health"}'
  echo '{"id":"2","method":"ed25519.generateKey"}'
  echo '{"id":"3","method":"metrics"}'
) | ./bin/go-webauthn-cli
```

## Available Endpoints

### System Endpoints

| Method | Description | Parameters |
|--------|-------------|------------|
| `health` | Health check | None |
| `version` | Version info | None |
| `metrics` | Operational metrics | None |

### Crypto Endpoints

| Method | Description | Parameters |
|--------|-------------|------------|
| `ed25519.generateKey` | Generate ED25519 keypair | None |
| `ed25519.sign` | Sign data | `public_key`, `private_key`, `data` |
| `ed25519.verify` | Verify signature | `public_key`, `data`, `signature` |

### WebAuthn Endpoints

| Method | Description | Parameters |
|--------|-------------|------------|
| `webauthn.signup.begin` | Start registration | `rp_display_name`, `rp_id`, `rp_origins`, `username`, `display_name`, `scenario` |
| `webauthn.signup.finish` | Complete registration | `username`, `response` |
| `webauthn.signin.begin` | Start authentication | `username`, `scenario` |
| `webauthn.signin.finish` | Complete authentication | `username`, `response` |
| `webauthn.passkey.begin` | Start passkey login | `rp_display_name`, `rp_id`, `rp_origins` |
| `webauthn.passkey.finish` | Complete passkey login | `response` |

## Test Coverage

| Component | Unit Tests | Integration Tests |
|-----------|------------|-------------------|
| **Infrastructure** | | |
| Errors | ✅ 5 tests | ✅ Error handling |
| Logging | ⚠️ Manual | ✅ Debug mode |
| Metrics | ✅ 5 tests | ✅ Tracking |
| Rate Limiting | ✅ 6 tests | ✅ Limits |
| **Endpoints** | | |
| Health | - | ✅ Tested |
| Version | - | ✅ Tested |
| Metrics | - | ✅ Tested |
| ED25519 | - | ✅ Key generation |
| **Total** | **16 tests** | **7 scenarios** |

## Continuous Integration

To add CI testing, create `.github/workflows/test.yml`:

```yaml
name: Test

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-go@v4
        with:
          go-version: '1.25'
      
      - name: Run unit tests
        run: ./test-unit.sh
      
      - name: Run integration tests
        run: ./test-simple.sh
```

## Writing New Tests

### Unit Test Template

```go
package main

import "testing"

func TestFeatureName(t *testing.T) {
    // Arrange
    input := "test"
    
    // Act
    result := FunctionUnderTest(input)
    
    // Assert
    if result != expected {
        t.Errorf("Expected %v, got %v", expected, result)
    }
}
```

### Integration Test Template

```bash
echo '{"id":"test","method":"your.method","params":{...}}' \
  | ./bin/go-webauthn-cli 2>/dev/null
```

## Troubleshooting

**Problem:** `jq: command not found`
- **Solution:** Use `test-simple.sh` instead of `test-endpoints.sh`

**Problem:** `cannot find main module`
- **Solution:** Use `test-unit.sh` which sets up a temporary module

**Problem:** Tests fail with rate limiting errors
- **Solution:** Each test creates fresh rate limiter instances or unique usernames

**Problem:** Build fails before tests
- **Solution:** Run `./build-cli.sh` manually first
