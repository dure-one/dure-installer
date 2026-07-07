# GCP Code Refactoring Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Refactor monolithic `gcp_rest.rs` (1,853 lines) into domain-specific modules under `api/gcp/*` with proper layered architecture.

**Architecture:** Split GCP services into focused modules (compute, billing, bigquery, resourcemanager, serviceusage, dns, oauth) with clean boundaries. Enforce UI → ViewModel → Calc/Api layering. Use TDD approach: move tests first, verify they fail, move code, verify they pass.

**Tech Stack:** Rust nightly, ureq (sync HTTP), serde (JSON), anyhow (errors), cargo test

## Global Constraints

- Rust nightly toolchain required
- No functionality changes - pure code movement refactoring
- All existing tests must pass after each task
- Use `ureq` for HTTP (no async migration)
- Follow TDD pattern: move tests first, then code
- Frequent commits after each module migration
- Test files in `mobile/tests/gcp_*.rs` (not inline `#[cfg(test)]`)
- Each module should be under 500 lines
- Preserve all existing function signatures and behavior

---

## Task 1: Setup Directory Structure

**Goal:** Create empty module structure for code migration.

**Files:**
- Create: `mobile/src/api/gcp/mod.rs`
- Create: `mobile/src/api/gcp/compute.rs`
- Create: `mobile/src/api/gcp/resourcemanager.rs`
- Create: `mobile/src/api/gcp/billing.rs`
- Create: `mobile/src/api/gcp/bigquery.rs`
- Create: `mobile/src/api/gcp/serviceusage.rs`
- Create: `mobile/src/api/gcp/dns.rs`
- Create: `mobile/src/api/gcp/oauth.rs`
- Modify: `mobile/src/api/mod.rs`

**Interfaces:**
- Consumes: Nothing
- Produces: Compilable empty module structure

- [ ] **Step 1: Create api/gcp directory and files**

```bash
cd mobile
mkdir -p src/api/gcp
touch src/api/gcp/mod.rs
touch src/api/gcp/compute.rs
touch src/api/gcp/resourcemanager.rs
touch src/api/gcp/billing.rs
touch src/api/gcp/bigquery.rs
touch src/api/gcp/serviceusage.rs
touch src/api/gcp/dns.rs
touch src/api/gcp/oauth.rs
cd ..
```

- [ ] **Step 2: Create api/gcp/mod.rs with module declarations**

File: `mobile/src/api/gcp/mod.rs`

```rust
//! Google Cloud Platform API modules organized by service domain

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

- [ ] **Step 3: Add gcp submodule to api/mod.rs**

Find line in `mobile/src/api/mod.rs` that has `pub mod gcp_oauth;` and add after it:

```rust
pub mod gcp;
```

- [ ] **Step 4: Create test directory structure**

```bash
cd mobile
mkdir -p tests
touch tests/gcp_common_tests.rs
touch tests/gcp_compute_tests.rs
touch tests/gcp_resourcemanager_tests.rs
touch tests/gcp_billing_tests.rs
touch tests/gcp_bigquery_tests.rs
touch tests/gcp_serviceusage_tests.rs
cd ..
```

- [ ] **Step 5: Verify structure compiles**

Run: `cd mobile && cargo check`
Expected: SUCCESS (empty modules compile)

- [ ] **Step 6: Commit structure**

```bash
git add mobile/src/api/gcp/ mobile/src/api/mod.rs mobile/tests/gcp_*
git commit -m "refactor(gcp): setup new api/gcp module structure

- Create api/gcp/ directory with 7 domain modules
- Add empty test files in mobile/tests/
- Prepare for code migration from gcp_rest.rs"
```

---

## Task 2: Create Common Module (api/gcp.rs)

**Goal:** Extract and migrate `GcpRestClient` and shared utilities to `api/gcp.rs`.

**Files:**
- Create: `mobile/src/api/gcp.rs`
- Reference: `mobile/src/calc/gcp_rest.rs` (lines 12-15, 18-44, 47-161)
- Reference: `mobile/src/calc/gcp.rs` (lines 275-340 for Region/MachineType)
- Create: `mobile/tests/gcp_common_tests.rs`

**Interfaces:**
- Consumes: Code from `calc/gcp_rest.rs` and `calc/gcp.rs`
- Produces:
  - `pub struct GcpRestClient`
  - `pub fn get_current_ip() -> Result<String>`
  - `pub fn ip_in_ranges(ip: &str, ranges: &[String]) -> bool`
  - `pub struct MachineType` + `pub fn get_common_machine_types() -> Vec<MachineType>`
  - `pub struct Region` + `pub fn get_common_regions() -> Vec<Region>`
  - `pub const GCP_COMPUTE_API_BASE: &str = "https://compute.googleapis.com/compute/v1"`
  - `pub const GCP_RESOURCE_MANAGER_API_BASE: &str = "https://cloudresourcemanager.googleapis.com/v1"`
  - `pub const GCP_BILLING_API_BASE: &str = "https://cloudbilling.googleapis.com/v1"`
  - `pub const GCP_SERVICE_USAGE_API_BASE: &str = "https://serviceusage.googleapis.com/v1"`

- [ ] **Step 1: Copy existing tests from gcp_rest.rs**

Look at `mobile/src/calc/gcp_rest.rs` lines 1746-1765 and 1797-1803. Copy these tests to `mobile/tests/gcp_common_tests.rs`:

```rust
//! Tests for api/gcp common module

use dure::api::gcp::{get_current_ip, ip_in_ranges, get_common_machine_types};

#[test]
fn test_get_current_ip() {
    let result = get_current_ip();
    
    if let Ok(ip) = result {
        // Should be valid IPv4 format
        assert!(ip.contains('.'));
        assert!(!ip.contains('\n'));
        assert!(!ip.contains(' '));
        
        // Should be parseable as IP
        use std::net::Ipv4Addr;
        let parsed: Result<Ipv4Addr, _> = ip.parse();
        assert!(parsed.is_ok(), "IP should be valid IPv4: {}", ip);
    } else {
        // Allow test to pass if offline
        eprintln!("Skipping IP test (offline): {:?}", result);
    }
}

#[test]
fn test_ip_in_ranges() {
    let ranges = vec!["10.0.0.0/8".to_string(), "117.53.222.116/32".to_string()];
    
    assert!(ip_in_ranges("117.53.222.116", &ranges));
    assert!(!ip_in_ranges("192.168.1.1", &ranges));
}

#[test]
fn test_common_machine_types() {
    let types = get_common_machine_types();
    assert!(!types.is_empty());
    assert!(types.iter().any(|t| t.name == "e2-micro"));
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd mobile && cargo test --test gcp_common_tests`
Expected: FAIL - "no such module `dure::api::gcp`"

- [ ] **Step 3: Create api/gcp.rs with API constants**

Create `mobile/src/api/gcp.rs` with constants from `calc/gcp_rest.rs` lines 12-15:

```rust
//! Google Cloud Platform API common module
//!
//! Provides shared client, utilities, and types for all GCP services.

use anyhow::Result;
use serde::{Deserialize, Serialize};

// ============================================================================
// API Base URLs
// ============================================================================

pub const GCP_COMPUTE_API_BASE: &str = "https://compute.googleapis.com/compute/v1";
pub const GCP_RESOURCE_MANAGER_API_BASE: &str = "https://cloudresourcemanager.googleapis.com/v1";
pub const GCP_BILLING_API_BASE: &str = "https://cloudbilling.googleapis.com/v1";
pub const GCP_SERVICE_USAGE_API_BASE: &str = "https://serviceusage.googleapis.com/v1";
```

- [ ] **Step 4: Add GcpRestClient struct and impl**

Copy from `calc/gcp_rest.rs` lines 46-161 to `api/gcp.rs`:

```rust
// ============================================================================
// GCP REST Client
// ============================================================================

/// GCP REST API client using ureq
pub struct GcpRestClient {
    access_token: String,
}

impl GcpRestClient {
    /// Create new client with access token
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }

    /// Make authenticated GET request with better error handling
    pub(crate) fn get(&self, url: &str) -> Result<ureq::Response> {
        match ureq::get(url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .set("Content-Type", "application/json")
            .call()
        {
            Ok(response) => Ok(response),
            Err(ureq::Error::Status(code, response)) => {
                let body = response.into_string().unwrap_or_default();

                // Check for API not enabled error
                if code == 403
                    && (body.contains("has not been used in project")
                        || body.contains("it is disabled"))
                {
                    let api_name = if body.contains("cloudresourcemanager") {
                        "Cloud Resource Manager API"
                    } else if body.contains("cloudbilling") {
                        "Cloud Billing API"
                    } else if body.contains("compute") {
                        "Compute Engine API"
                    } else {
                        "Required API"
                    };

                    return Err(anyhow::anyhow!(
                        "{} is not enabled. Please enable it in the GCP Console:\n{}",
                        api_name,
                        body
                    ));
                }

                Err(anyhow::anyhow!(
                    "HTTP {} error for {}:\n{}",
                    code,
                    url,
                    if body.len() > 500 {
                        format!("{}...", &body[..500])
                    } else {
                        body
                    }
                ))
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(anyhow::anyhow!("Network error for {}:\n{}", url, transport))
            }
        }
    }

    /// Make authenticated POST request
    /// Returns Response for both success and error statuses (caller must check status)
    pub(crate) fn post(&self, url: &str, body: &str) -> Result<ureq::Response> {
        match ureq::post(url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .set("Content-Type", "application/json")
            .send_string(body)
        {
            Ok(response) => Ok(response),
            Err(ureq::Error::Status(_code, response)) => {
                // Return the response so caller can inspect error
                Ok(response)
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(anyhow::anyhow!("Network error for {}: {}", url, transport))
            }
        }
    }

    /// Make authenticated DELETE request
    pub(crate) fn delete(&self, url: &str) -> Result<ureq::Response> {
        match ureq::delete(url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .set("Content-Type", "application/json")
            .call()
        {
            Ok(response) => Ok(response),
            Err(ureq::Error::Status(code, response)) => {
                let body = response.into_string().unwrap_or_default();
                Err(anyhow::anyhow!("HTTP {} error for {}: {}", code, url, body))
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(anyhow::anyhow!("Network error for {}: {}", url, transport))
            }
        }
    }

    /// Make authenticated PATCH request
    pub(crate) fn patch(&self, url: &str, body: &str) -> Result<ureq::Response> {
        match ureq::request("PATCH", url)
            .set("Authorization", &format!("Bearer {}", self.access_token))
            .set("Content-Type", "application/json")
            .send_string(body)
        {
            Ok(response) => Ok(response),
            Err(ureq::Error::Status(code, response)) => {
                let body = response.into_string().unwrap_or_default();
                Err(anyhow::anyhow!("HTTP {} error for {}: {}", code, url, body))
            }
            Err(ureq::Error::Transport(transport)) => {
                Err(anyhow::anyhow!("Network error for {}: {}", url, transport))
            }
        }
    }
}
```

- [ ] **Step 5: Add utility functions**

Copy from `calc/gcp_rest.rs` lines 17-44:

```rust
// ============================================================================
// Utility Functions
// ============================================================================

/// Get current public IP address from icanhazip.com
pub fn get_current_ip() -> Result<String> {
    let response = ureq::get("https://icanhazip.com")
        .timeout(std::time::Duration::from_secs(10))
        .call()
        .map_err(|e| anyhow::anyhow!("Failed to fetch IP: {}", e))?;

    let ip_text = response
        .into_string()
        .map_err(|e| anyhow::anyhow!("Failed to read IP response: {}", e))?;

    let ip = ip_text.trim().to_string();

    // Validate it looks like an IP
    if !ip.contains('.') || ip.is_empty() {
        return Err(anyhow::anyhow!("Invalid IP format: {}", ip));
    }

    Ok(ip)
}

/// Check if an IP is in any of the CIDR ranges
pub fn ip_in_ranges(ip: &str, ranges: &[String]) -> bool {
    // Simple check: exact match or 0.0.0.0/0
    ranges
        .iter()
        .any(|range| range == ip || range == &format!("{}/32", ip) || range == "0.0.0.0/0")
}
```

- [ ] **Step 6: Add MachineType and get_common_machine_types**

Copy from `calc/gcp.rs` lines 292-340:

```rust
// ============================================================================
// Common Types
// ============================================================================

/// Machine type information for UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineType {
    pub name: String,
    pub description: String,
    pub cpus: i32,
    pub memory_mb: i64,
}

/// Get common machine types for UI selection
pub fn get_common_machine_types() -> Vec<MachineType> {
    vec![
        MachineType {
            name: "e2-micro".to_string(),
            description: "0.25-2 vCPU, 1 GB RAM (free tier eligible)".to_string(),
            cpus: 1,
            memory_mb: 1024,
        },
        MachineType {
            name: "e2-small".to_string(),
            description: "0.5-2 vCPU, 2 GB RAM".to_string(),
            cpus: 1,
            memory_mb: 2048,
        },
        MachineType {
            name: "e2-medium".to_string(),
            description: "1-2 vCPU, 4 GB RAM".to_string(),
            cpus: 1,
            memory_mb: 4096,
        },
        MachineType {
            name: "n1-standard-1".to_string(),
            description: "1 vCPU, 3.75 GB RAM".to_string(),
            cpus: 1,
            memory_mb: 3840,
        },
        MachineType {
            name: "n1-standard-2".to_string(),
            description: "2 vCPU, 7.5 GB RAM".to_string(),
            cpus: 2,
            memory_mb: 7680,
        },
        MachineType {
            name: "n2-standard-2".to_string(),
            description: "2 vCPU, 8 GB RAM".to_string(),
            cpus: 2,
            memory_mb: 8192,
        },
    ]
}

/// GCP Region information for UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Region {
    pub name: String,
    pub location: String,
    pub zones: Vec<String>,
}

/// Get common regions for UI selection  
pub fn get_common_regions() -> Vec<Region> {
    vec![
        Region {
            name: "us-central1".to_string(),
            location: "Iowa, USA".to_string(),
            zones: vec![
                "us-central1-a".to_string(),
                "us-central1-b".to_string(),
                "us-central1-c".to_string(),
                "us-central1-f".to_string(),
            ],
        },
        Region {
            name: "us-east1".to_string(),
            location: "South Carolina, USA".to_string(),
            zones: vec![
                "us-east1-b".to_string(),
                "us-east1-c".to_string(),
                "us-east1-d".to_string(),
            ],
        },
        Region {
            name: "asia-northeast3".to_string(),
            location: "Seoul, South Korea".to_string(),
            zones: vec![
                "asia-northeast3-a".to_string(),
                "asia-northeast3-b".to_string(),
                "asia-northeast3-c".to_string(),
            ],
        },
        Region {
            name: "asia-northeast1".to_string(),
            location: "Tokyo, Japan".to_string(),
            zones: vec![
                "asia-northeast1-a".to_string(),
                "asia-northeast1-b".to_string(),
                "asia-northeast1-c".to_string(),
            ],
        },
        Region {
            name: "europe-west1".to_string(),
            location: "Belgium, Europe".to_string(),
            zones: vec![
                "europe-west1-b".to_string(),
                "europe-west1-c".to_string(),
                "europe-west1-d".to_string(),
            ],
        },
    ]
}
```

- [ ] **Step 7: Add api/gcp module to api/mod.rs**

In `mobile/src/api/mod.rs`, add after the `gcp_oauth` line:

```rust
pub mod gcp;
```

- [ ] **Step 8: Run tests to verify they pass**

Run: `cd mobile && cargo test --test gcp_common_tests`
Expected: All 3 tests PASS

- [ ] **Step 9: Verify module compiles**

Run: `cd mobile && cargo check`
Expected: SUCCESS

- [ ] **Step 10: Commit common module**

```bash
git add mobile/src/api/gcp.rs mobile/src/api/mod.rs mobile/tests/gcp_common_tests.rs
git commit -m "refactor(gcp): migrate common code to api/gcp.rs

- Move GcpRestClient with get/post/delete/patch methods
- Move utilities: get_current_ip, ip_in_ranges
- Move config helpers from calc/gcp.rs: MachineType, Region
- Add API base URL constants
- Add tests in mobile/tests/gcp_common_tests.rs
- All tests pass"
```

---

## Task 3: Migrate Compute Module  

**Goal:** Extract Compute Engine API code from `gcp_rest.rs` to `api/gcp/compute.rs`.

**Files:**
- Modify: `mobile/src/api/gcp/compute.rs`
- Reference: `mobile/src/calc/gcp_rest.rs` (lines 163-1147 - instance, firewall, region/zone operations)
- Create: `mobile/tests/gcp_compute_tests.rs`
- Modify: `mobile/src/api/gcp/mod.rs` (update re-exports)

**Interfaces:**
- Consumes: `GcpRestClient` from `crate::api::gcp`, `GCP_COMPUTE_API_BASE` constant
- Produces:
  - Instance types: `Instance`, `InstanceRequest`, `InstanceList`
  - Firewall types: `FirewallRule`, `FirewallRequest`, `FirewallAllowed`
  - Supporting types: `AttachedDisk`, `NetworkInterface`, `Tags`, `Metadata`, `Operation`
  - Region/Zone types: `RegionList`, `ZoneList`, `Zone`, `RegionInfo`
  - All impl methods on `GcpRestClient` for compute operations

This task involves moving ~980 lines of code. The key is to move types and functions as cohesive units.

- [ ] **Step 1: Copy firewall test from gcp_rest.rs**

Look at `mobile/src/calc/gcp_rest.rs` lines 1781-1795. Copy to `mobile/tests/gcp_compute_tests.rs`:

```rust
//! Tests for api/gcp/compute module

use dure::api::gcp::compute::{FirewallRule, FirewallAllowed};

#[test]
fn test_firewall_rule_structure() {
    let rule = FirewallRule {
        name: "allow-ssh".to_string(),
        allowed: vec![FirewallAllowed {
            ip_protocol: "tcp".to_string(),
            ports: Some(vec!["22".to_string()]),
        }],
        source_ranges: Some(vec!["0.0.0.0/0".to_string()]),
    };

    assert_eq!(rule.name, "allow-ssh");
    assert_eq!(rule.allowed[0].ip_protocol, "tcp");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd mobile && cargo test --test gcp_compute_tests`
Expected: FAIL - "no such module `dure::api::gcp::compute`"

- [ ] **Step 3: Add module header and imports to compute.rs**

File: `mobile/src/api/gcp/compute.rs`

```rust
//! Google Compute Engine API
//!
//! Instance management, firewalls, regions, and zones.

use crate::api::gcp::{GcpRestClient, GCP_COMPUTE_API_BASE};
use anyhow::Result;
use serde::{Deserialize, Serialize};
```

- [ ] **Step 4: Copy instance request/response types**

From `calc/gcp_rest.rs` lines 577-667, copy to `compute.rs`:

```rust
// ============================================================================
// Instance Types
// ============================================================================

/// Instance creation request
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstanceRequest {
    pub name: String,
    pub machine_type: String, // e.g., "zones/us-central1-a/machineTypes/e2-micro"
    pub disks: Vec<AttachedDisk>,
    pub network_interfaces: Vec<NetworkInterface>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Tags>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Metadata>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachedDisk {
    pub boot: bool,
    pub auto_delete: bool,
    pub initialize_params: InitializeParams,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    pub source_image: String,
    pub disk_size_gb: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterface {
    pub network: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_configs: Option<Vec<AccessConfig>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessConfig {
    #[serde(rename = "type")]
    pub type_: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct Tags {
    pub items: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Metadata {
    pub items: Vec<MetadataItem>,
}

#[derive(Debug, Serialize)]
pub struct MetadataItem {
    pub key: String,
    pub value: String,
}

/// Instance response
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub machine_type: String,
    pub zone: String,
    pub status: String,
    #[serde(default)]
    pub network_interfaces: Vec<NetworkInterfaceResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterfaceResponse {
    #[serde(rename = "networkIP", default)]
    pub network_ip: Option<String>,
    #[serde(default)]
    pub access_configs: Vec<AccessConfigResponse>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessConfigResponse {
    #[serde(rename = "natIP")]
    pub nat_ip: Option<String>,
}

/// Instance list response
#[derive(Debug, Deserialize)]
pub struct InstanceList {
    #[serde(default)]
    pub items: Vec<Instance>,
}

impl Instance {
    /// Helper to get external IP
    pub fn external_ip(&self) -> Option<String> {
        self.network_interfaces
            .first()?
            .access_configs
            .first()?
            .nat_ip
            .clone()
    }

    /// Helper to get internal IP
    pub fn internal_ip(&self) -> Option<String> {
        self.network_interfaces.first()?.network_ip.clone()
    }

    /// Create debian e2-micro instance request
    pub fn debian_micro(name: String, zone: String) -> InstanceRequest {
        InstanceRequest {
            name: name.clone(),
            machine_type: format!("zones/{}/machineTypes/e2-micro", zone),
            disks: vec![AttachedDisk {
                boot: true,
                auto_delete: true,
                initialize_params: InitializeParams {
                    source_image: "projects/debian-cloud/global/images/family/debian-11"
                        .to_string(),
                    disk_size_gb: "10".to_string(),
                },
            }],
            network_interfaces: vec![NetworkInterface {
                network: "global/networks/default".to_string(),
                access_configs: Some(vec![AccessConfig {
                    type_: "ONE_TO_ONE_NAT".to_string(),
                    name: "External NAT".to_string(),
                }]),
            }],
            tags: Some(Tags {
                items: vec!["http-server".to_string(), "https-server".to_string()],
            }),
            metadata: None,
        }
    }
}
```

- [ ] **Step 5: Copy firewall types**

From `calc/gcp_rest.rs` lines 800-812, copy to `compute.rs`:

```rust
// ============================================================================
// Firewall Types
// ============================================================================

/// GCP Firewall Rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub name: String,
    pub allowed: Vec<FirewallAllowed>,
    #[serde(rename = "sourceRanges")]
    pub source_ranges: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallAllowed {
    pub ip_protocol: String,
    pub ports: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct FirewallListResponse {
    items: Option<Vec<FirewallRule>>,
}

/// Firewall creation request
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirewallRequest {
    pub name: String,
    pub network: String, // e.g., "global/networks/default"
    pub allowed: Vec<FirewallAllowed>,
    pub source_ranges: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_tags: Option<Vec<String>>,
}
```

- [ ] **Step 6: Copy Operation type**

From `calc/gcp_rest.rs` lines 676-725:

```rust
// ============================================================================
// Operation Types
// ============================================================================

/// Operation response (for async operations)
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    #[serde(default)]
    pub id: Option<String>,
    pub name: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub done: Option<bool>,
    #[serde(default)]
    pub error: Option<OperationError>,
}

#[derive(Debug, Deserialize)]
pub struct OperationError {
    pub errors: Vec<OperationErrorItem>,
}

#[derive(Debug, Deserialize)]
pub struct OperationErrorItem {
    pub code: String,
    pub message: String,
}

impl Operation {
    /// Returns true if the operation is complete
    pub fn is_done(&self) -> bool {
        self.done.unwrap_or(false) || self.status.as_deref() == Some("DONE")
    }

    /// Returns true if the operation has an error
    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    /// Returns human-readable status string
    pub fn status_string(&self) -> String {
        if let Some(error) = &self.error {
            format!("ERROR: {}", error.errors[0].message)
        } else if self.is_done() {
            "DONE".to_string()
        } else {
            self.status.clone().unwrap_or_else(|| "PENDING".to_string())
        }
    }
}
```

- [ ] **Step 7: Copy region/zone types**

From `calc/gcp_rest.rs` lines 727-775:

```rust
// ============================================================================
// Region/Zone Types
// ============================================================================

/// Region list response
#[derive(Debug, Deserialize)]
pub struct RegionList {
    #[serde(default)]
    pub items: Vec<RegionInfo>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegionInfo {
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
}

/// Zone list response
#[derive(Debug, Deserialize)]
pub struct ZoneList {
    #[serde(default)]
    pub items: Vec<Zone>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Zone {
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub region: String, // URL to region
}
```

- [ ] **Step 8: Add GcpRestClient impl for instance operations**

From `calc/gcp_rest.rs` lines 170-289, copy to `compute.rs` (note: change `self` references and imports):

```rust
// ============================================================================
// Instance Operations
// ============================================================================

impl GcpRestClient {
    /// Create a new VM instance
    pub fn create_instance(
        &self,
        project_id: &str,
        zone: &str,
        request: &InstanceRequest,
    ) -> Result<Operation> {
        let url = format!("{}/projects/{}/zones/{}/instances", GCP_COMPUTE_API_BASE, project_id, zone);
        
        let body = serde_json::to_string(request)?;
        let response = self.post(&url, &body)?;
        
        if response.status() != 200 {
            let error_text = response.into_string().unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to create instance: {}", error_text));
        }
        
        let operation: Operation = response.into_json()?;
        Ok(operation)
    }

    /// List VM instances in a zone
    pub fn list_instances(&self, project_id: &str, zone: &str) -> Result<InstanceList> {
        let url = format!(
            "{}/projects/{}/zones/{}/instances",
            GCP_COMPUTE_API_BASE, project_id, zone
        );

        let response = self.get(&url)?;
        let list: InstanceList = response.into_json()?;
        Ok(list)
    }

    /// Get specific VM instance
    pub fn get_instance(
        &self,
        project_id: &str,
        zone: &str,
        instance_name: &str,
    ) -> Result<Instance> {
        let url = format!(
            "{}/projects/{}/zones/{}/instances/{}",
            GCP_COMPUTE_API_BASE, project_id, zone, instance_name
        );

        let response = self.get(&url)?;
        let instance: Instance = response.into_json()?;
        Ok(instance)
    }

    /// Delete VM instance
    pub fn delete_instance(
        &self,
        project_id: &str,
        zone: &str,
        instance_name: &str,
    ) -> Result<Operation> {
        let url = format!(
            "{}/projects/{}/zones/{}/instances/{}",
            GCP_COMPUTE_API_BASE, project_id, zone, instance_name
        );

        let response = self.delete(&url)?;
        let operation: Operation = response.into_json()?;
        Ok(operation)
    }

    /// Reset (restart) VM instance
    pub fn reset_instance(
        &self,
        project_id: &str,
        zone: &str,
        instance_name: &str,
    ) -> Result<Operation> {
        let url = format!(
            "{}/projects/{}/zones/{}/instances/{}/reset",
            GCP_COMPUTE_API_BASE, project_id, zone, instance_name
        );

        let response = self.post(&url, "")?;
        let operation: Operation = response.into_json()?;
        Ok(operation)
    }

    /// Wait for zone operation to complete
    pub fn wait_for_operation(
        &self,
        project_id: &str,
        zone: &str,
        operation: &Operation,
    ) -> Result<Operation> {
        let operation_name = &operation.name;
        let url = format!(
            "{}/projects/{}/zones/{}/operations/{}",
            GCP_COMPUTE_API_BASE, project_id, zone, operation_name
        );

        // Poll with exponential backoff
        let mut wait_time = std::time::Duration::from_secs(1);
        let max_wait = std::time::Duration::from_secs(60);

        loop {
            std::thread::sleep(wait_time);

            let response = self.get(&url)?;
            let op: Operation = response.into_json()?;

            if op.is_done() {
                return Ok(op);
            }

            wait_time = (wait_time * 2).min(max_wait);
        }
    }

    /// Wait for global operation to complete
    pub fn wait_for_global_operation(
        &self,
        project_id: &str,
        operation: &Operation,
    ) -> Result<Operation> {
        let operation_name = &operation.name;
        let url = format!(
            "{}/projects/{}/global/operations/{}",
            GCP_COMPUTE_API_BASE, project_id, operation_name
        );

        // Poll with exponential backoff
        let mut wait_time = std::time::Duration::from_secs(1);
        let max_wait = std::time::Duration::from_secs(60);

        loop {
            std::thread::sleep(wait_time);

            let response = self.get(&url)?;
            let op: Operation = response.into_json()?;

            if op.is_done() {
                return Ok(op);
            }

            wait_time = (wait_time * 2).min(max_wait);
        }
    }
}
```

- [ ] **Step 9: Add region/zone operations**

From `calc/gcp_rest.rs` lines 357-400:

```rust
// ============================================================================
// Region/Zone Operations
// ============================================================================

impl GcpRestClient {
    /// List regions for a project
    pub fn list_regions(&self, project_id: &str) -> Result<RegionList> {
        let url = format!("{}/projects/{}/regions", GCP_COMPUTE_API_BASE, project_id);
        let response = self.get(&url)?;
        let list: RegionList = response.into_json()?;
        Ok(list)
    }

    /// List zones for a project
    pub fn list_zones(&self, project_id: &str) -> Result<ZoneList> {
        let url = format!("{}/projects/{}/zones", GCP_COMPUTE_API_BASE, project_id);
        let response = self.get(&url)?;
        let list: ZoneList = response.into_json()?;
        Ok(list)
    }
}
```

- [ ] **Step 10: Add firewall operations**

From `calc/gcp_rest.rs` lines 1000-1147:

```rust
// ============================================================================
// Firewall Operations
// ============================================================================

impl GcpRestClient {
    /// List firewalls
    pub fn list_firewalls(
        &self,
        project_id: &str,
    ) -> Result<Vec<FirewallRule>> {
        let url = format!("{}/projects/{}/global/firewalls", GCP_COMPUTE_API_BASE, project_id);
        
        let response = self.get(&url)?;
        let list: FirewallListResponse = response.into_json()?;
        Ok(list.items.unwrap_or_default())
    }

    /// Create firewall rule
    pub fn create_firewall(
        &self,
        project_id: &str,
        request: &FirewallRequest,
    ) -> Result<Operation> {
        let url = format!("{}/projects/{}/global/firewalls", GCP_COMPUTE_API_BASE, project_id);
        
        let body = serde_json::to_string(request)?;
        let response = self.post(&url, &body)?;
        
        let operation: Operation = response.into_json()?;
        Ok(operation)
    }

    /// List firewall rules (simplified response)
    pub fn list_firewall_rules(&self, project_id: &str) -> Result<Vec<FirewallRule>> {
        self.list_firewalls(project_id)
    }

    /// Check if IP is whitelisted in any firewall rule
    pub fn check_ip_whitelisted(&self, project_id: &str, ip: &str) -> Result<bool> {
        let rules = self.list_firewall_rules(project_id)?;
        
        for rule in rules {
            if let Some(ranges) = &rule.source_ranges {
                if crate::api::gcp::ip_in_ranges(ip, ranges) {
                    return Ok(true);
                }
            }
        }
        
        Ok(false)
    }

    /// Add IP to firewall rule "allow-ssh-dure"
    pub fn add_ip_to_firewall(&self, project_id: &str, ip: &str) -> Result<()> {
        use crate::api::gcp::get_current_ip;
        
        // Get current firewall rules
        let rules = self.list_firewall_rules(project_id)?;
        
        // Find "allow-ssh-dure" rule
        let rule = rules
            .iter()
            .find(|r| r.name == "allow-ssh-dure")
            .ok_or_else(|| anyhow::anyhow!("Firewall rule 'allow-ssh-dure' not found"))?;

        // Get current source ranges
        let mut source_ranges = rule.source_ranges.clone().unwrap_or_default();

        // Check if IP already in ranges
        if get_current_ip().ok().as_deref() == Some(ip) || source_ranges.contains(&ip.to_string()) {
            return Ok(()); // Already whitelisted
        }

        // Add new IP
        source_ranges.push(ip.to_string());

        // Update firewall rule using PATCH
        let url = format!(
            "{}/projects/{}/global/firewalls/{}",
            GCP_COMPUTE_API_BASE, project_id, "allow-ssh-dure"
        );

        let update_body = serde_json::json!({
            "sourceRanges": source_ranges
        });

        let body = update_body.to_string();
        let response = self.patch(&url, &body)?;
        let response_text = response.into_string().unwrap_or_default();
        eprintln!("DEBUG: Response: {}", response_text);

        Ok(())
    }
}
```

- [ ] **Step 11: Run tests to verify they pass**

Run: `cd mobile && cargo test --test gcp_compute_tests`
Expected: test_firewall_rule_structure PASS

- [ ] **Step 12: Verify module compiles**

Run: `cd mobile && cargo check`
Expected: SUCCESS

- [ ] **Step 13: Commit compute module**

```bash
git add mobile/src/api/gcp/compute.rs mobile/tests/gcp_compute_tests.rs
git commit -m "refactor(gcp): migrate compute module

- Move instance operations and types from gcp_rest.rs
- Move firewall operations and types
- Move region/zone operations
- Add tests for firewall structure
- All tests pass"
```

---

## Task 4-10: Migrate Remaining Modules and Complete Refactor

Due to the size of this refactoring, Tasks 4-10 follow the same TDD pattern as Tasks 2-3:

**Task 4: Migrate Resource Manager** - Move project operations (list, get, create) and Project types
**Task 5: Migrate Billing** - Move billing account operations and types  
**Task 6: Migrate BigQuery** - Move dataset, table, query operations including billing queries
**Task 7: Migrate Service Usage** - Move enable_service and is_service_enabled
**Task 8: Migrate OAuth** - Move content from `api/gcp_oauth.rs` to `api/gcp/oauth.rs`
**Task 9: Migrate DNS** - Move content from `api/ns_gcp.rs` to `api/gcp/dns.rs`, make ns_gcp.rs a re-export
**Task 10: Update All Imports** - Update viewmodel, ui_dlg, ui_tabs, calc, cli to use new paths
**Task 11: Cleanup** - Delete gcp_rest.rs, empty gcp.rs, delete gcp_oauth.rs, verify tests
**Task 12: Update Documentation** - Update CLAUDE.md with new architecture

Each task follows the pattern:
1. Copy tests from old location to `mobile/tests/gcp_X_tests.rs`
2. Run tests to verify failure
3. Copy types and impl blocks to new module
4. Run tests to verify success
5. Verify `cargo check` passes
6. Commit with descriptive message

The implementation strategy for each domain module:
- Identify all types/structs related to that domain in `gcp_rest.rs`
- Identify all `impl GcpRestClient` functions for that domain
- Copy to new module preserving exact signatures
- Update internal imports to use `crate::api::gcp::{GcpRestClient, constants}`
- Add module-specific tests

---

## Execution Notes

**For Subagent-Driven Development:**
- Each task should be assigned to a fresh subagent
- Subagent reviews code between tasks
- Follow commit messages exactly as specified
- Verify `cargo test` passes after each task before proceeding

**For Inline Execution:**
- Complete tasks 1-3 first (structure, common, compute)
- Run `cargo test` checkpoint after task 3
- Complete remaining tasks 4-9 (other modules)
- Run `cargo test` checkpoint after task 9
- Complete tasks 10-12 (imports, cleanup, docs)
- Final `cargo test` verification

**Critical Success Criteria:**
- All existing tests must pass after migration
- No functionality changes
- Each module under 500 lines
- Clean git history with descriptive commits
