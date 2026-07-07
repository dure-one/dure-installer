# GCP Code Refactoring Design

**Date:** 2026-07-07  
**Status:** Approved  
**Author:** Claude (with user collaboration)

## Overview

Refactor GCP-related code from monolithic `gcp_rest.rs` (1,853 lines) into a layered architecture with domain-specific modules under `api/gcp/*`. This improves maintainability, testability, and enforces proper separation of concerns.

## Goals

1. **Domain Separation**: Split GCP services into focused modules (compute, billing, bigquery, etc.)
2. **Layered Architecture**: Enforce UI → ViewModel → Calc/Api boundaries
3. **Maintainability**: Smaller, focused files are easier to understand and modify
4. **Testability**: Isolated modules with clear boundaries are easier to test
5. **Consistency**: All GCP APIs under unified `api/gcp/*` structure

## Non-Goals

- Changing functionality or API behavior
- Switching from `ureq` to async HTTP client
- Adding new GCP service integrations (can be added later in new modules)

## Architecture

### Layered Structure

```
┌─────────────────────────────────────────┐
│ UI Layer (ui_tabs/*, ui_dlg/*)         │
│ - Only imports from viewmodel           │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│ ViewModel Layer (viewmodel/*/actor.rs)  │
│ - Imports from calc (common operations) │
│ - Imports from api (direct operations)  │
└──────┬──────────────────┬───────────────┘
       │                  │
       │                  │
┌──────▼─────────┐  ┌────▼───────────────┐
│ Calc Layer     │  │ Api Layer          │
│ (calc/gcp.rs)  │  │ (api/gcp/*)        │
│ - Common utils │  │ - Direct GCP calls │
│ - Empty init   │  │ - Domain-specific  │
└────────────────┘  └────────────────────┘
```

### Key Principles

- **UI → ViewModel only**: No direct calc/api imports in UI code
- **ViewModel → Calc/Api**: Can use both, choosing based on need
- **Calc → Api**: Common operations may wrap api calls
- **Api independence**: Each domain module is self-contained

## File Structure

### New Structure

```
mobile/src/
├── api/
│   ├── gcp.rs                 # Common client, utilities, types
│   ├── gcp/
│   │   ├── mod.rs             # Re-exports all submodules
│   │   ├── compute.rs         # Compute Engine API
│   │   ├── resourcemanager.rs # Resource Manager API
│   │   ├── billing.rs         # Cloud Billing API
│   │   ├── bigquery.rs        # BigQuery API (includes billing queries)
│   │   ├── serviceusage.rs    # Service Usage API
│   │   ├── dns.rs             # Cloud DNS API
│   │   └── oauth.rs           # OAuth & user info
│   └── ns_gcp.rs              # Re-export from gcp/dns.rs (compatibility)
├── calc/
│   └── gcp.rs                 # Empty initially, add commons as needed
└── viewmodel/
    └── platform/
        └── actor.rs           # Uses api::gcp::* directly

mobile/tests/
├── gcp_common_tests.rs        # Tests for gcp.rs utilities
├── gcp_compute_tests.rs       # Tests for compute module
├── gcp_billing_tests.rs       # Tests for billing module
├── gcp_bigquery_tests.rs      # Tests for bigquery module
├── gcp_resourcemanager_tests.rs
├── gcp_serviceusage_tests.rs
└── ...
```

### Files to Delete

- `mobile/src/calc/gcp_rest.rs` (split into domain modules)
- Content of `mobile/src/calc/gcp.rs` (empty initially)
- `mobile/src/api/gcp_oauth.rs` (move to `api/gcp/oauth.rs`)
- Inline tests in `gcp_rest.rs` (move to `mobile/tests/`)

## Module Responsibilities

### `api/gcp.rs` (Common Module)

**Purpose:** Shared client, utilities, and common types used across all GCP domains.

**Exports:**
- `GcpRestClient` - Core HTTP client with OAuth token
  - `new(access_token: String) -> Self`
  - `get(&self, url: &str) -> Result<Response>`
  - `post(&self, url: &str, body: &str) -> Result<Response>`
  - `delete(&self, url: &str) -> Result<Response>`
  - `patch(&self, url: &str, body: &str) -> Result<Response>`

- Utility functions:
  - `get_current_ip() -> Result<String>`
  - `ip_in_ranges(ip: &str, ranges: &[String]) -> bool`

- Configuration helpers:
  - `get_common_machine_types() -> Vec<MachineType>`
  - `get_common_regions() -> Vec<Region>`

- Common types:
  - `MachineType { name, description, cpus, memory_mb }`
  - `Region { name, location, zones }`
  - `Operation` (used by multiple services)

- API base URL constants:
  - `GCP_COMPUTE_API_BASE`
  - `GCP_RESOURCE_MANAGER_API_BASE`
  - `GCP_BILLING_API_BASE`
  - `GCP_SERVICE_USAGE_API_BASE`

### `api/gcp/compute.rs`

**Purpose:** Google Compute Engine API operations.

**Exports:**
- Instance operations:
  - `create_instance(&self, project_id, zone, request) -> Result<Operation>`
  - `delete_instance(&self, project_id, zone, name) -> Result<Operation>`
  - `list_instances(&self, project_id, zone) -> Result<InstanceList>`
  - `get_instance(&self, project_id, zone, name) -> Result<Instance>`
  - `reset_instance(&self, project_id, zone, name) -> Result<Operation>`

- Firewall operations:
  - `list_firewalls(&self, project_id) -> Result<Vec<Firewall>>`
  - `create_firewall(&self, project_id, request) -> Result<Operation>`
  - `list_firewall_rules(&self, project_id) -> Result<Vec<FirewallRule>>`
  - `check_ip_whitelisted(&self, project_id, ip) -> Result<bool>`
  - `add_ip_to_firewall(&self, project_id, ip) -> Result<()>`

- Region/zone operations:
  - `list_regions(&self, project_id) -> Result<RegionList>`
  - `list_zones(&self, project_id) -> Result<ZoneList>`

- Operation polling:
  - `wait_for_operation(&self, project_id, zone, operation) -> Result<Operation>`
  - `wait_for_global_operation(&self, project_id, operation) -> Result<Operation>`

**Types:**
- `Instance { id, name, machine_type, zone, status, network_interfaces }`
  - `impl Instance { external_ip(), internal_ip() }`
- `InstanceRequest { name, machine_type, disks, network_interfaces, tags, metadata }`
- `InstanceList { items: Vec<Instance> }`
- `FirewallRule { name, allowed, source_ranges }`
- `FirewallRequest { name, network, allowed, source_ranges, target_tags }`
- `AttachedDisk`, `NetworkInterface`, `Tags`, `Metadata`, etc.
- `RegionList`, `ZoneList`

### `api/gcp/resourcemanager.rs`

**Purpose:** Google Cloud Resource Manager API operations.

**Exports:**
- Project operations:
  - `list_projects(&self, filter: Option<&str>) -> Result<ProjectList>`
  - `get_project(&self, project_id) -> Result<Project>`
  - `create_project(&self, project_id, display_name) -> Result<Operation>`

**Types:**
- `Project { name, project_id, display_name, state, labels }`
  - `impl Project { id(), display_name(), state(), is_active() }`
- `ProjectList { projects: Vec<Project>, next_page_token }`

### `api/gcp/billing.rs`

**Purpose:** Google Cloud Billing API operations.

**Exports:**
- Billing account operations:
  - `list_billing_accounts(&self) -> Result<BillingAccountList>`

- Project billing operations:
  - `get_project_billing_info(&self, project_id) -> Result<ProjectBillingInfo>`
  - `enable_project_billing(&self, project_id, billing_account_id) -> Result<ProjectBillingInfo>`

**Types:**
- `BillingAccount { name, display_name, open, master_billing_account }`
  - `impl BillingAccount { id() }`
- `BillingAccountList { billing_accounts, next_page_token }`
- `ProjectBillingInfo { name, project_id, billing_account_name, billing_enabled }`

### `api/gcp/bigquery.rs`

**Purpose:** Google BigQuery API operations, including billing data queries.

**Exports:**
- Dataset operations:
  - `list_bigquery_datasets(&self, project_id) -> Result<Vec<String>>`

- Table operations:
  - `list_bigquery_tables(&self, project_id, dataset_id) -> Result<Vec<String>>`

- Query operations:
  - `query_bigquery(&self, project_id, query) -> Result<BigQueryResponse>`
  - `discover_billing_table(&self, project_id) -> Result<(String, String)>`

- Billing-specific queries:
  - `get_current_month_billing(&self, project_id, dataset, table) -> Result<Vec<BillingRecord>>`
  - `get_billing_by_service(&self, project_id, dataset, table, start_date, end_date) -> Result<Vec<BillingRecord>>`

**Types:**
- `BigQueryResponse { rows, total_rows, schema }`
- `BillingRecord { service_description, cost, currency, usage_start_time, usage_end_time }`

### `api/gcp/serviceusage.rs`

**Purpose:** Google Service Usage API operations.

**Exports:**
- Service management:
  - `enable_service(&self, project_id, service) -> Result<()>`
  - `is_service_enabled(&self, project_id, service) -> Result<bool>`

**Types:** None (simple operations returning primitives)

### `api/gcp/dns.rs`

**Purpose:** Google Cloud DNS API operations (migrated from `api/ns_gcp.rs`).

**Exports:**
- DNS zone operations
- DNS record management
- `GcpDnsClient` and related types

**Note:** Content moved from existing `api/ns_gcp.rs`. Original file becomes re-export for compatibility.

### `api/gcp/oauth.rs`

**Purpose:** OAuth authentication and user info (migrated from `api/gcp_oauth.rs`).

**Exports:**
- OAuth flow handling:
  - `OAuthHandler` struct and methods
  - `start_oauth_flow()`, `exchange_code()`, etc.

- User info:
  - `get_user_info(&self) -> Result<UserInfo>`

**Types:**
- `OAuthHandler`
- `OAuthResult { access_token, refresh_token, expires_in }`
- `UserInfo { email, verified_email, name, given_name, family_name, picture }`

### `api/gcp/mod.rs`

**Purpose:** Module coordinator and convenient re-exports.

```rust
pub mod compute;
pub mod resourcemanager;
pub mod billing;
pub mod bigquery;
pub mod serviceusage;
pub mod dns;
pub mod oauth;

// Re-export commonly used types for convenience
pub use compute::{Instance, InstanceRequest, FirewallRule};
pub use resourcemanager::Project;
pub use billing::BillingAccount;
pub use bigquery::BigQueryResponse;
pub use oauth::OAuthHandler;
```

### `api/ns_gcp.rs` (Compatibility Re-export)

**Purpose:** Maintain backward compatibility for existing NS tab code.

```rust
// Re-export everything from gcp/dns for compatibility
pub use crate::api::gcp::dns::*;
```

### `calc/gcp.rs`

**Purpose:** Common GCP business logic and utilities (empty initially).

**Initial State:** Empty or minimal placeholder.

**Future Use:** Add common wrapper functions that:
- Combine multiple API calls
- Add business logic/validation
- Provide higher-level abstractions for ViewModels

**Example future content:**
```rust
use crate::api::gcp::{self, GcpRestClient};
use crate::api::gcp::compute;

/// Allow current IP in firewall (combines two operations)
pub async fn allow_current_ip(client: &GcpRestClient, project_id: &str) -> Result<()> {
    let ip = gcp::get_current_ip()?;
    compute::add_ip_to_firewall(client, project_id, &ip)?;
    Ok(())
}
```

## Data Flow Examples

### Example 1: Create VM Instance (Direct API call)

```
┌─────────────────────────────────────────────┐
│ UI (ui_tabs/platform.rs)                    │
│ - User clicks "Create VM"                   │
│ - Sends ViewModelCommand::CreateVM          │
└──────────────┬──────────────────────────────┘
               │
┌──────────────▼──────────────────────────────┐
│ ViewModel (viewmodel/platform/actor.rs)     │
│ - Receives PlatformCommand::CreateVM        │
│ - Calls api::gcp::compute::create_instance()│
│ - Emits PlatformEvent with result           │
└──────────────┬──────────────────────────────┘
               │
┌──────────────▼──────────────────────────────┐
│ Api (api/gcp/compute.rs)                    │
│ - create_instance(&self, project, zone, ..) │
│ - Uses GcpRestClient.post()                 │
│ - Returns Result<Operation>                 │
└─────────────────────────────────────────────┘
```

### Example 2: Update Firewall (With optional Calc abstraction)

```
┌─────────────────────────────────────────────┐
│ UI (ui_tabs/platform.rs)                    │
│ - User clicks "Allow My IP"                 │
│ - Sends command to ViewModel                │
└──────────────┬──────────────────────────────┘
               │
┌──────────────▼──────────────────────────────┐
│ ViewModel (viewmodel/platform/actor.rs)     │
│ Option A: Direct API calls                  │
│   - api::gcp::get_current_ip()              │
│   - api::gcp::compute::add_ip_to_firewall() │
│                                             │
│ Option B: Calc abstraction (if exists)      │
│   - calc::gcp::allow_current_ip()           │
└──────────────┬──────────────────────────────┘
               │
┌──────────────▼──────────────────────────────┐
│ Calc (calc/gcp.rs) [Optional]               │
│ - Combines multiple api calls               │
│ - Adds business logic/validation            │
└──────────────┬──────────────────────────────┘
               │
┌──────────────▼──────────────────────────────┐
│ Api (api/gcp.rs + api/gcp/compute.rs)       │
│ - get_current_ip() -> IP address            │
│ - add_ip_to_firewall() -> updates rule      │
└─────────────────────────────────────────────┘
```

## Type Organization

### API Request/Response Types → Domain Modules
- `InstanceRequest`, `AttachedDisk`, etc. → `api/gcp/compute.rs`
- `Project`, `ProjectList` → `api/gcp/resourcemanager.rs`
- `BillingAccount`, `ProjectBillingInfo` → `api/gcp/billing.rs`

### Shared Types → `api/gcp.rs`
- `GcpRestClient` (core client)
- `MachineType`, `Region` (configuration helpers)
- `Operation` (used by multiple services)

### Domain Structs with Methods → Domain Modules
- `impl Instance` with `external_ip()`, `internal_ip()` → `compute.rs`
- `impl Project` with `id()`, `is_active()` → `resourcemanager.rs`
- `impl BillingAccount` with `id()` → `billing.rs`

### Deprecated Types → Remove
- `calc/gcp.rs` stub types (unused `GcpClient`, `InstanceConfig`)
- Duplicate `calc/gcp.rs::Instance` type

### Re-exports in `api/gcp/mod.rs`
```rust
// Commonly used types exported at module root
pub use compute::{Instance, InstanceRequest, FirewallRule};
pub use resourcemanager::Project;
pub use billing::BillingAccount;
pub use bigquery::BigQueryResponse;
```

## Import Patterns

### After Refactoring

```rust
// In viewmodel or calc code
use crate::api::gcp::GcpRestClient;
use crate::api::gcp::compute::{Instance, InstanceRequest};
use crate::api::gcp::billing::BillingAccount;

// Or use re-exports for common types
use crate::api::gcp::{GcpRestClient, Instance, Project};
```

### Before Refactoring (old pattern)

```rust
use crate::calc::gcp_rest::GcpRestClient;
use crate::calc::gcp_rest::{Instance, InstanceRequest};
use crate::api::gcp_oauth::{OAuthHandler, OAuthResult};
```

## Testing Strategy

### Test Organization

Tests reside in `mobile/tests/` directory (not inline `#[cfg(test)]` modules).

```
mobile/tests/
├── gcp_common_tests.rs       # Tests for gcp.rs utilities
├── gcp_compute_tests.rs      # Tests for compute module
├── gcp_billing_tests.rs      # Tests for billing module
├── gcp_bigquery_tests.rs     # Tests for bigquery module
├── gcp_resourcemanager_tests.rs
├── gcp_serviceusage_tests.rs
└── ...
```

### Test Structure Example

```rust
// mobile/tests/gcp_compute_tests.rs
use dure::api::gcp::compute::*;
use dure::api::gcp::GcpRestClient;

#[test]
fn test_instance_external_ip() {
    // Test Instance::external_ip() helper method
    let instance = Instance {
        id: "123".to_string(),
        name: "test-vm".to_string(),
        machine_type: "e2-micro".to_string(),
        zone: "us-central1-a".to_string(),
        status: "RUNNING".to_string(),
        network_interfaces: vec![/* ... */],
    };
    
    assert_eq!(instance.external_ip(), Some("1.2.3.4".to_string()));
}

#[test]
fn test_instance_request_construction() {
    // Verify InstanceRequest builds correct structure
}
```

### Existing Tests Migration

Current tests in `calc/gcp_rest.rs` (lines 1720-1803):
- `test_get_current_ip()` → `mobile/tests/gcp_common_tests.rs`
- `test_project_structure()` → `mobile/tests/gcp_resourcemanager_tests.rs`
- `test_firewall_rule_structure()` → `mobile/tests/gcp_compute_tests.rs`
- `test_check_ip_in_ranges()` → `mobile/tests/gcp_common_tests.rs`

### TDD Workflow

1. **Extract test from `gcp_rest.rs`**
   - Copy test to appropriate `mobile/tests/gcp_X_tests.rs`
   - Update imports to new module paths
   - Run `cargo test` (should fail - code not migrated yet)

2. **Move code to new module**
   - Extract function/type from `gcp_rest.rs`
   - Place in appropriate domain module
   - Run `cargo test` (should pass)

3. **Repeat for each function/type**

4. **Final verification**
   - All tests pass in new modules
   - `cargo check` passes
   - Delete old files

### Test Coverage

- ✅ Type serialization/deserialization
- ✅ Helper methods (external_ip, is_active, etc.)
- ✅ URL construction
- ✅ Error handling paths
- ❌ Actual HTTP calls (would require credentials/live API)

## Migration Plan

### Phase 1: Setup New Structure

1. Create directory `mobile/src/api/gcp/`
2. Create empty module files:
   - `api/gcp.rs`
   - `api/gcp/mod.rs`
   - `api/gcp/compute.rs`
   - `api/gcp/resourcemanager.rs`
   - `api/gcp/billing.rs`
   - `api/gcp/bigquery.rs`
   - `api/gcp/serviceusage.rs`
   - `api/gcp/dns.rs`
   - `api/gcp/oauth.rs`
3. Create test files in `mobile/tests/`:
   - `gcp_common_tests.rs`
   - `gcp_compute_tests.rs`
   - `gcp_resourcemanager_tests.rs`
   - `gcp_billing_tests.rs`
   - `gcp_bigquery_tests.rs`
   - `gcp_serviceusage_tests.rs`

**Commit:** "refactor(gcp): setup new api/gcp module structure"

### Phase 2: Migrate Common Code (`api/gcp.rs`)

1. Move `GcpRestClient` struct and impl from `gcp_rest.rs`
2. Move utility functions: `get_current_ip()`, `ip_in_ranges()`
3. Move from `calc/gcp.rs`: `MachineType`, `Region`, `get_common_machine_types()`
4. Move API base URL constants
5. Move existing tests to `tests/gcp_common_tests.rs`
6. Run `cargo test` - verify tests pass

**Commit:** "refactor(gcp): migrate common code to api/gcp.rs"

### Phase 3: Migrate Domain Modules (one by one)

For each domain (compute, resourcemanager, billing, bigquery, serviceusage):

1. Move types from `gcp_rest.rs` to domain module
2. Move functions that use those types
3. Update internal `use` statements within the module
4. Move tests to `tests/gcp_X_tests.rs`
5. Run `cargo test` after each domain
6. Commit each domain separately

**Commits:**
- "refactor(gcp): migrate compute module"
- "refactor(gcp): migrate resourcemanager module"
- "refactor(gcp): migrate billing module"
- "refactor(gcp): migrate bigquery module"
- "refactor(gcp): migrate serviceusage module"

### Phase 4: Migrate OAuth and DNS

1. Move `api/gcp_oauth.rs` content → `api/gcp/oauth.rs`
2. Move `api/ns_gcp.rs` content → `api/gcp/dns.rs`
3. Update `api/ns_gcp.rs` to re-export: `pub use crate::api::gcp::dns::*;`
4. Optionally keep `api/gcp_oauth.rs` as re-export temporarily

**Commit:** "refactor(gcp): migrate oauth and dns to api/gcp"

### Phase 5: Update Imports

Update all files that import from old locations:

1. **`viewmodel/platform/actor.rs`:**
   - `use crate::calc::gcp_rest::GcpRestClient;` → `use crate::api::gcp::GcpRestClient;`
   - `use crate::calc::gcp_rest::*;` → `use crate::api::gcp::compute::*;`

2. **`ui_dlg/platform_gcp.rs`:**
   - `use crate::api::gcp_oauth::*;` → `use crate::api::gcp::oauth::*;`
   - `use crate::calc::gcp::*;` → `use crate::api::gcp::*;`
   - `use crate::calc::gcp_rest::*;` → `use crate::api::gcp::compute::*;`

3. **`ui_tabs/platform.rs`:**
   - Update imports to new paths

4. **`ui_tabs/ns.rs`:**
   - Already uses `api::ns_gcp`, should continue working via re-export

5. **`calc/hosting_gcp.rs`:**
   - `use crate::calc::gcp_rest::*;` → `use crate::api::gcp::compute::*;`

6. **`cli/commands/platform/tests.rs`:**
   - Update test imports

**Commit:** "refactor(gcp): update all imports to new module structure"

### Phase 6: Cleanup

1. Verify `cargo check` passes
2. Verify `cargo test` passes
3. Delete `mobile/src/calc/gcp_rest.rs`
4. Empty `mobile/src/calc/gcp.rs` (keep file for future utilities)
5. Delete `mobile/src/api/gcp_oauth.rs` (if using re-export strategy)
6. Run final `cargo test`

**Commit:** "refactor(gcp): remove old files after migration"

### Phase 7: Documentation

1. Update `CLAUDE.md` with new architecture section
2. Document layer responsibilities
3. Document import patterns
4. Update project structure diagram

**Commit:** "docs: update CLAUDE.md with new GCP architecture"

## Rollback Strategy

- Each phase commits separately
- If issues found, `git revert` the problematic commit
- All existing tests must pass before proceeding to next phase
- Can pause migration at any phase boundary

## Success Criteria

### Must Have
- ✅ All existing tests pass
- ✅ `cargo check` passes with no warnings
- ✅ All GCP code organized under `api/gcp/*`
- ✅ Layer boundaries enforced (UI → ViewModel → Calc/Api)
- ✅ No duplicate code between old and new modules

### Nice to Have
- ✅ Tests organized in `mobile/tests/gcp_*_tests.rs`
- ✅ Common types re-exported in `api/gcp/mod.rs`
- ✅ CLAUDE.md updated with architecture documentation
- ✅ Each module under 500 lines

## Risks and Mitigations

### Risk: Breaking existing functionality
**Mitigation:** TDD approach - move tests first, ensure they pass after each migration step

### Risk: Import errors across many files
**Mitigation:** Update imports in bulk during Phase 5, commit separately for easy rollback

### Risk: Type conflicts or circular dependencies
**Mitigation:** Carefully organize types in domain modules, use re-exports sparingly

### Risk: Losing git history for moved code
**Mitigation:** Use descriptive commit messages referencing old file locations

## Future Enhancements

After this refactoring, future improvements become easier:

1. **Add new GCP services** - Create new module in `api/gcp/`
2. **Add common utilities** - Add to `calc/gcp.rs` as needed
3. **Mock testing** - Easier to mock individual domain modules
4. **Async migration** - Can migrate modules one at a time if needed
5. **Parallel development** - Multiple developers can work on different GCP domains

## References

- Current implementation: `mobile/src/calc/gcp_rest.rs` (1,853 lines)
- Current abstraction: `mobile/src/calc/gcp.rs` (366 lines, mostly stubs)
- OAuth implementation: `mobile/src/api/gcp_oauth.rs` (465 lines)
- DNS implementation: `mobile/src/api/ns_gcp.rs` (used by NS tab)
- GCP REST API documentation: https://cloud.google.com/compute/docs/reference/rest/v1

---

**End of Design Document**
