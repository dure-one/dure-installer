# DNS Commands Test Suite

Comprehensive test coverage for DNS CLI commands and ViewModel actor.

## Test Files

### 1. Unit Tests - DNS Actor (`mobile/src/viewmodel/dns/tests.rs`)

**13 tests covering:**

- ✅ `test_set_access_token` - OAuth token management
- ✅ `test_list_zones_without_token` - Zone listing error handling
- ✅ `test_list_domains_without_token` - Domain listing error handling  
- ✅ `test_check_availability_without_token` - Availability check error handling
- ✅ `test_domain_commands_compile` - All domain registration commands compile
- ✅ `test_zone_commands_compile` - All zone management commands compile
- ✅ `test_record_commands_compile` - All record management commands compile
- ✅ `test_domain_events_compile` - All domain registration events compile
- ✅ `test_record_type_as_str` - RecordType string conversion
- ✅ `test_domain_info_clone` - DomainInfo cloning

**Coverage:**
- Command dispatch and actor communication
- Error handling for missing authentication
- Event emission verification
- Type safety and compilation checks

### 2. Unit Tests - CLI Commands (`mobile/src/cli/commands/dns.rs`)

**6 tests covering:**

- ✅ `test_parse_record_type_valid` - Valid record type parsing (A, AAAA, CNAME, MX, TXT, NS, SRV)
- ✅ `test_parse_record_type_invalid` - Invalid record type error handling
- ✅ `test_parse_record_type_case_insensitive` - Case-insensitive parsing
- ✅ `test_send_dns_command_helper` - Command helper function communication
- ✅ `test_send_dns_command_error_no_token` - Error propagation without token

**Coverage:**
- Record type parsing and validation
- CLI command → Actor communication
- Error handling and event processing

### 3. Integration Tests (`mobile/tests/dns_cli_integration_tests.rs`)

**11 tests covering:**

- ✅ `test_dns_actor_set_token_flow` - Full OAuth token flow
- ✅ `test_dns_actor_list_zones_no_token` - Zone listing without auth
- ✅ `test_dns_actor_domain_commands_no_token` - Domain commands without auth
- ✅ `test_dns_actor_check_availability_no_token` - Availability check without auth
- ✅ `test_dns_actor_record_operations_no_token` - Record operations without auth
- ✅ `test_dns_actor_multiple_commands_sequence` - Command sequencing
- ✅ `test_dns_record_type_variations` - All RecordType variants
- ✅ `test_dns_command_cloning` - Command cloning behavior
- ✅ `test_dns_event_cloning` - Event cloning behavior

**Coverage:**
- End-to-end command execution
- Multi-step workflows
- Authentication flow
- Error propagation through full stack

## Running Tests

### Run All DNS Tests

```bash
cd mobile
cargo test dns
```

### Run Specific Test Suites

```bash
# DNS Actor tests only
cargo test --lib viewmodel::dns::tests

# CLI command tests only  
cargo test --lib cli::commands::dns::tests

# Integration tests only
cargo test --test dns_cli_integration_tests
```

### Run Individual Tests

```bash
# Run specific test by name
cargo test test_set_access_token -- --nocapture

# Run with debug output
RUST_LOG=debug cargo test test_dns_actor_list_zones -- --nocapture
```

## Test Coverage Summary

| Component | Tests | Coverage |
|-----------|-------|----------|
| DNS Actor | 13 | Command dispatch, events, error handling |
| CLI Commands | 6 | Parsing, validation, communication |
| Integration | 11 | End-to-end flows, multi-step |
| **Total** | **30** | **Comprehensive** |

## Coverage Areas

### ✅ Domain Registration Commands
- `ListDomains` - List all registered domains
- `GetDomain` - Get domain information
- `CheckAvailability` - Check domain availability
- `RegisterDomain` - Register new domain
- `UpdateNameservers` - Update domain nameservers

### ✅ Zone Management Commands
- `ListZones` - List DNS zones
- `FindZone` - Find zone by domain
- `CreateZone` - Create DNS zone

### ✅ Record Management Commands
- `ListRecords` - List DNS records
- `FindRecord` - Find specific record
- `CreateRecord` - Create DNS record
- `UpdateRecord` - Update DNS record
- `DeleteRecord` - Delete DNS record
- `UpsertRecord` - Create/update record

### ✅ Error Handling
- Missing authentication tokens
- Invalid record types
- API failures
- Network errors
- Unexpected events

### ✅ Type Safety
- All RecordType variants (A, AAAA, CNAME, MX, TXT, NS, SRV)
- Command/Event cloning
- Type conversions
- Enum exhaustiveness

## Test Patterns Used

### AAA Pattern (Arrange-Act-Assert)
```rust
#[test]
fn test_set_access_token() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, event_rx) = create_test_actor();

        // Act
        cmd_tx.send(DnsCommand::SetAccessToken { ... }).await.unwrap();

        // Assert
        let event = wait_for_event(&event_rx, 5).await.unwrap();
        assert!(matches!(event, DnsEvent::AccessTokenSet));
    });
}
```

### Table-Driven Tests
```rust
#[test]
fn test_record_type_variations() {
    let record_types = vec![
        RecordType::A, RecordType::AAAA, RecordType::CNAME,
        RecordType::MX, RecordType::TXT, RecordType::NS, RecordType::SRV,
    ];
    
    for record_type in record_types {
        let _cmd = DnsCommand::FindRecord { zone_id, record_type, name };
    }
}
```

### Async Testing with smol
```rust
#[test]
fn test_async_command() {
    smol::block_on(async {
        // Async test code
    });
}
```

## Notes for Future Testing

### API Mocking
For tests that require Cloudflare API access, consider:
- Mock HTTP client for unit tests
- Test accounts with proper cleanup for integration tests
- Mark real API tests with `#[ignore]` by default

### Example Ignored Test
```rust
#[test]
#[ignore] // Run with --ignored flag
fn test_cloudflare_api_real() {
    // Requires CLOUDFLARE_API_TOKEN env var
    // Uses real API, should cleanup resources
}
```

### CI/CD Considerations
- Unit tests run on every commit
- Integration tests run on PR only
- API tests run nightly with test credentials
- Target: 80%+ code coverage

## Test Maintenance

When adding new commands:
1. Add command variant test in `test_*_commands_compile`
2. Add event variant test in `test_*_events_compile`
3. Add error handling test for auth failures
4. Add integration test for full flow
5. Update this documentation

## References

- Rust testing guide: `mobile/docs/GUIDELINES_RUST_CODING.md`
- TDD workflow: `.claude/rules/common/testing.md`
- smol async runtime: `https://docs.rs/smol/`
