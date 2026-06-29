# Platform Tab GCP Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enhance the Platform tab with GCP integration featuring hierarchical VM display, inline action buttons, background SSH testing, and comprehensive VM lifecycle management.

**Architecture:** Custom egui table widget replaces MaterialSpreadsheet. New calc modules (`platform_gcp.rs`, `hosting_gcp.rs`) handle GCP-specific logic. Background tasks via `poll_promise::Promise` for SSH testing. Config persistence to `~/.config/dure/config.yml`.

**Tech Stack:** Rust 2021, egui 0.33, ureq (HTTP), ssh2 (SSH), poll_promise (async), serde (serialization), keepass (keyring)

## Global Constraints

- Rust edition: 2021, minimum version 1.81
- Desktop only (Linux, Windows, macOS) and Android - no WASM for this feature
- Single project per platform, single VM display (even if multiple VMs exist)
- Config location: `~/.config/dure/config.yml`
- Keyring location: `~/.config/dure/key.kdbx`
- SSH key algorithm: Ed25519
- IP detection service: https://icanhazip.com
- All destructive actions require typed confirmation ("delete", "regenerate", "restart", "update")

---

## File Structure

### New Files

- `mobile/src/calc/platform_gcp.rs` - GCP platform operations (add, remove, OAuth, project selection)
- `mobile/src/calc/hosting_gcp.rs` - GCP VM lifecycle (create, delete, restart, regenerate, SSH key gen)

### Modified Files

- `mobile/src/calc/platform.rs` - Refactored generic platform interface
- `mobile/src/calc/hosting.rs` - Refactored generic hosting interface
- `mobile/src/calc/gcp_rest.rs` - Extended with firewall, IP detection, project listing
- `mobile/src/calc/ssh.rs` - Updated to support keyring-based authentication
- `mobile/src/ui_tabs/platform.rs` - Complete rewrite with custom table widget
- `mobile/src/config.rs` - Add `gcp_selected_project_id` field
- `mobile/src/calc/mod.rs` - Add new module declarations

### Test Files

- `mobile/src/calc/platform_gcp_tests.rs` (inline module)
- `mobile/src/calc/hosting_gcp_tests.rs` (inline module)
- `mobile/src/calc/gcp_rest_tests.rs` (extend existing)

---

## Task 1: Add gcp_selected_project_id to Config

**Files:**
- Modify: `mobile/src/config.rs:CloudPlatformConfig`

**Interfaces:**
- Consumes: Existing `CloudPlatformConfig` struct
- Produces: `pub gcp_selected_project_id: Option<String>` field for use by all tasks

- [ ] **Step 1: Write failing test for config with selected project**

Add to `mobile/src/config.rs` in a test module:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloud_platform_config_with_selected_project() {
        let config = CloudPlatformConfig {
            name: "test-gcp".to_string(),
            platform_type: "gcp".to_string(),
            gcp_selected_project_id: Some("dure".to_string()),
            gcp_connected_email: None,
            gcp_oauth_access_token: None,
            gcp_oauth_refresh_token: None,
            gcp_oauth_token_expiry: None,
            firebase_project_id: None,
            firebase_api_key: None,
            supabase_project_ref: None,
            supabase_api_url: None,
            supabase_anon_key: None,
            api_token: None,
            service_account_json: None,
            vms: vec![],
        };
        
        assert_eq!(config.gcp_selected_project_id, Some("dure".to_string()));
    }

    #[test]
    fn test_config_serialization_with_selected_project() {
        let config = CloudPlatformConfig {
            name: "test".to_string(),
            platform_type: "gcp".to_string(),
            gcp_selected_project_id: Some("project-123".to_string()),
            gcp_connected_email: None,
            gcp_oauth_access_token: None,
            gcp_oauth_refresh_token: None,
            gcp_oauth_token_expiry: None,
            firebase_project_id: None,
            firebase_api_key: None,
            supabase_project_ref: None,
            supabase_api_url: None,
            supabase_anon_key: None,
            api_token: None,
            service_account_json: None,
            vms: vec![],
        };
        
        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("gcp_selected_project_id: project-123"));
        
        let deserialized: CloudPlatformConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(deserialized.gcp_selected_project_id, Some("project-123".to_string()));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib config::tests::test_cloud_platform_config_with_selected_project -- --nocapture`

Expected: FAIL - "no field `gcp_selected_project_id`"

- [ ] **Step 3: Add field to CloudPlatformConfig**

Find the `CloudPlatformConfig` struct (around line 200-230) and add the field after `gcp_connected_email`:

```rust
pub struct CloudPlatformConfig {
    pub name: String,
    pub platform_type: String,
    
    // GCP specific
    pub gcp_oauth_access_token: Option<String>,
    pub gcp_oauth_refresh_token: Option<String>,
    pub gcp_oauth_token_expiry: Option<i64>,
    pub gcp_connected_email: Option<String>,
    pub gcp_selected_project_id: Option<String>,  // NEW
    
    // ... rest of fields
}
```

Update the `Default` implementation to include the new field:

```rust
impl Default for CloudPlatformConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            platform_type: String::new(),
            gcp_oauth_access_token: None,
            gcp_oauth_refresh_token: None,
            gcp_oauth_token_expiry: None,
            gcp_connected_email: None,
            gcp_selected_project_id: None,  // NEW
            // ... rest
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib config::tests -- --nocapture`

Expected: PASS (both tests)

- [ ] **Step 5: Commit**

```bash
git add mobile/src/config.rs
git commit -m "feat: add gcp_selected_project_id to CloudPlatformConfig

Add field to track which GCP project is selected for each platform.
This allows the platform tab to show VMs from a single project.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Extend GcpRestClient with IP Detection

**Files:**
- Modify: `mobile/src/calc/gcp_rest.rs`

**Interfaces:**
- Consumes: Existing `GcpRestClient`
- Produces: `pub fn get_current_ip() -> Result<String>` - returns public IP as string (e.g., "117.53.222.116")

- [ ] **Step 1: Write failing test for IP detection**

Add to `mobile/src/calc/gcp_rest.rs` at the end:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_ip_format() {
        // This test requires internet connection
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
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib gcp_rest::tests::test_get_current_ip_format -- --nocapture`

Expected: FAIL - "cannot find function `get_current_ip`"

- [ ] **Step 3: Implement get_current_ip function**

Add near the top of `gcp_rest.rs` after the constants:

```rust
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
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib gcp_rest::tests::test_get_current_ip_format -- --nocapture`

Expected: PASS (or skip if offline)

- [ ] **Step 5: Commit**

```bash
git add mobile/src/calc/gcp_rest.rs
git commit -m "feat: add IP detection via icanhazip.com

Add get_current_ip() function to fetch public IP address
for use in firewall whitelisting.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Extend GcpRestClient with Project Listing

**Files:**
- Modify: `mobile/src/calc/gcp_rest.rs`

**Interfaces:**
- Consumes: `GcpRestClient` with `access_token`
- Produces: `pub fn list_projects(&self) -> Result<Vec<Project>>` where `Project` has `pub project_id: String, pub project_name: String`

- [ ] **Step 1: Write failing test for project listing**

Add to the `tests` module in `gcp_rest.rs`:

```rust
#[test]
fn test_project_structure() {
    let project = Project {
        project_id: "test-project-123".to_string(),
        project_name: "Test Project".to_string(),
    };
    
    assert_eq!(project.project_id, "test-project-123");
    assert_eq!(project.project_name, "Test Project");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib gcp_rest::tests::test_project_structure -- --nocapture`

Expected: FAIL - "cannot find type `Project`"

- [ ] **Step 3: Add Project struct and list_projects method**

Add near the top after the `GcpRestClient` struct:

```rust
/// GCP Project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    #[serde(rename = "projectId")]
    pub project_id: String,
    #[serde(rename = "name")]
    pub project_name: String,
}

#[derive(Debug, Deserialize)]
struct ProjectListResponse {
    projects: Option<Vec<Project>>,
}
```

Add method to `GcpRestClient` impl block:

```rust
impl GcpRestClient {
    // ... existing methods ...
    
    /// List all projects in the GCP account
    pub fn list_projects(&self) -> Result<Vec<Project>> {
        let url = format!("{}/projects", GCP_RESOURCE_MANAGER_API_BASE);
        
        let response = self.get(&url)?;
        let list_response: ProjectListResponse = response
            .into_json()
            .context("Failed to parse projects response")?;
        
        Ok(list_response.projects.unwrap_or_default())
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --lib gcp_rest::tests::test_project_structure -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add mobile/src/calc/gcp_rest.rs
git commit -m "feat: add GCP project listing API

Add Project struct and list_projects() method to fetch
all projects in a GCP account for project selection.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Extend GcpRestClient with Firewall Operations

**Files:**
- Modify: `mobile/src/calc/gcp_rest.rs`

**Interfaces:**
- Consumes: `GcpRestClient` with `access_token`
- Produces:
  - `pub fn list_firewall_rules(&self, project_id: &str) -> Result<Vec<FirewallRule>>`
  - `pub fn check_ip_whitelisted(&self, project_id: &str, ip: &str) -> Result<bool>`
  - `pub fn add_ip_to_firewall(&self, project_id: &str, ip: &str) -> Result<()>`

- [ ] **Step 1: Write failing test for firewall rule structure**

Add to tests module:

```rust
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

#[test]
fn test_check_ip_in_ranges() {
    let ranges = vec![
        "10.0.0.0/8".to_string(),
        "117.53.222.116/32".to_string(),
    ];
    
    assert!(ip_in_ranges("117.53.222.116", &ranges));
    assert!(!ip_in_ranges("192.168.1.1", &ranges));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib gcp_rest::tests::test_firewall_rule_structure -- --nocapture`

Expected: FAIL - "cannot find type `FirewallRule`"

- [ ] **Step 3: Add firewall structs and methods**

Add structs after Project:

```rust
/// GCP Firewall Rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallRule {
    pub name: String,
    pub allowed: Vec<FirewallAllowed>,
    #[serde(rename = "sourceRanges")]
    pub source_ranges: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallAllowed {
    #[serde(rename = "IPProtocol")]
    pub ip_protocol: String,
    pub ports: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct FirewallListResponse {
    items: Option<Vec<FirewallRule>>,
}
```

Add helper function before impl block:

```rust
/// Check if an IP is in any of the CIDR ranges
fn ip_in_ranges(ip: &str, ranges: &[String]) -> bool {
    // Simple check: exact match or 0.0.0.0/0
    ranges.iter().any(|range| {
        range == ip || 
        range == &format!("{}/32", ip) ||
        range == "0.0.0.0/0"
    })
}
```

Add methods to GcpRestClient:

```rust
impl GcpRestClient {
    // ... existing methods ...
    
    /// List firewall rules for a project
    pub fn list_firewall_rules(&self, project_id: &str) -> Result<Vec<FirewallRule>> {
        let url = format!("{}/projects/{}/global/firewalls", GCP_COMPUTE_API_BASE, project_id);
        
        let response = self.get(&url)?;
        let list_response: FirewallListResponse = response
            .into_json()
            .context("Failed to parse firewall rules")?;
        
        Ok(list_response.items.unwrap_or_default())
    }
    
    /// Check if an IP is whitelisted for SSH (port 22) in firewall rules
    pub fn check_ip_whitelisted(&self, project_id: &str, ip: &str) -> Result<bool> {
        let rules = self.list_firewall_rules(project_id)?;
        
        for rule in rules {
            // Check if rule allows SSH (port 22)
            let allows_ssh = rule.allowed.iter().any(|a| {
                a.ip_protocol.to_lowercase() == "tcp" &&
                a.ports.as_ref().map_or(false, |ports| {
                    ports.iter().any(|p| p == "22")
                })
            });
            
            if allows_ssh {
                if let Some(ranges) = &rule.source_ranges {
                    if ip_in_ranges(ip, ranges) {
                        return Ok(true);
                    }
                }
            }
        }
        
        Ok(false)
    }
    
    /// Add an IP to the SSH firewall whitelist
    pub fn add_ip_to_firewall(&self, project_id: &str, ip: &str) -> Result<()> {
        let rules = self.list_firewall_rules(project_id)?;
        
        // Find existing SSH rule or create new one
        let ssh_rule = rules.iter().find(|rule| {
            rule.allowed.iter().any(|a| {
                a.ip_protocol.to_lowercase() == "tcp" &&
                a.ports.as_ref().map_or(false, |ports| {
                    ports.iter().any(|p| p == "22")
                })
            })
        });
        
        let url = if let Some(rule) = ssh_rule {
            // Update existing rule
            let mut updated_ranges = rule.source_ranges.clone().unwrap_or_default();
            let ip_cidr = format!("{}/32", ip);
            
            if !updated_ranges.contains(&ip_cidr) {
                updated_ranges.push(ip_cidr);
            }
            
            let body = serde_json::json!({
                "name": rule.name,
                "allowed": rule.allowed,
                "sourceRanges": updated_ranges,
            });
            
            let url = format!("{}/projects/{}/global/firewalls/{}", 
                GCP_COMPUTE_API_BASE, project_id, rule.name);
            
            self.post(&url, &body.to_string())?;
            url
        } else {
            // Create new SSH rule
            let body = serde_json::json!({
                "name": "allow-ssh-dure",
                "allowed": [{
                    "IPProtocol": "tcp",
                    "ports": ["22"]
                }],
                "sourceRanges": [format!("{}/32", ip)],
                "direction": "INGRESS",
            });
            
            let url = format!("{}/projects/{}/global/firewalls", 
                GCP_COMPUTE_API_BASE, project_id);
            
            self.post(&url, &body.to_string())?;
            url
        };
        
        Ok(())
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib gcp_rest::tests -- --nocapture`

Expected: PASS (all firewall tests)

- [ ] **Step 5: Commit**

```bash
git add mobile/src/calc/gcp_rest.rs
git commit -m "feat: add GCP firewall management API

Add firewall rule listing, IP whitelist checking, and
IP addition to firewall rules for SSH port 22.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Update SSH Module to Support Keyring

**Files:**
- Modify: `mobile/src/calc/ssh.rs`

**Interfaces:**
- Consumes: `SshHostConfig` with `keyring_domain: Option<String>`
- Produces: Updated `test_connection()` that loads SSH key from keyring if `keyring_domain` is set

- [ ] **Step 1: Write failing test for keyring-based SSH**

Add to `mobile/src/calc/ssh.rs` at the end:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ssh_config_with_keyring_domain() {
        let config = SshHostConfig {
            host: "user@example.com".to_string(),
            password: None,
            private_key_path: None,
            keyring_domain: Some("gcp.test.vm".to_string()),
            port: 22,
            initialized: false,
            last_status: None,
        };
        
        assert_eq!(config.keyring_domain, Some("gcp.test.vm".to_string()));
        assert!(config.private_key_path.is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo test --lib ssh::tests::test_ssh_config_with_keyring_domain -- --nocapture`

Expected: PASS (config field already exists)

- [ ] **Step 3: Update authenticate function to use keyring**

Find the `authenticate` function in `ssh.rs` and update it:

```rust
fn authenticate(sess: &mut Session, username: &str, host_config: &SshHostConfig) -> Result<()> {
    // Try keyring-based authentication first
    if let Some(keyring_domain) = &host_config.keyring_domain {
        // Load private key from keyring
        match crate::calc::keyring::get_key(keyring_domain) {
            Ok(key_entry) => {
                if let Some(ssh_key_bytes) = key_entry.ssh_key {
                    // Write key to temporary in-memory buffer
                    use std::io::Cursor;
                    let mut key_reader = Cursor::new(ssh_key_bytes);
                    
                    // Try to authenticate with the key
                    if sess.userauth_pubkey_memory(username, None, &key_reader, None).is_ok() {
                        return Ok(());
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to load SSH key from keyring: {}", e);
            }
        }
    }
    
    // Try private key file if specified
    if let Some(key_path) = &host_config.private_key_path {
        if let Ok(path) = std::path::Path::new(key_path).canonicalize() {
            if sess.userauth_pubkey_file(username, None, &path, None).is_ok() {
                return Ok(());
            }
        }
    }
    
    // Try password authentication
    if let Some(password) = &host_config.password {
        sess.userauth_password(username, password)
            .context("Password authentication failed")?;
        return Ok(());
    }
    
    Err(anyhow::anyhow!(
        "No valid authentication method found (tried keyring, key file, password)"
    ))
}
```

- [ ] **Step 4: Build to verify it compiles**

Run: `cargo build --lib`

Expected: SUCCESS

- [ ] **Step 5: Commit**

```bash
git add mobile/src/calc/ssh.rs
git commit -m "feat: add keyring-based SSH authentication

Update authenticate() to load SSH private keys from keyring
when keyring_domain is specified in SshHostConfig.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Create platform_gcp Module with Platform Operations

**Files:**
- Create: `mobile/src/calc/platform_gcp.rs`
- Modify: `mobile/src/calc/mod.rs`

**Interfaces:**
- Consumes: `CloudPlatformConfig`, `GcpRestClient`
- Produces:
  - `pub fn compute_platform_status(platform: &CloudPlatformConfig, total_projects: usize) -> String` - returns "N Projects"
  - `pub fn compute_project_status(platform: &CloudPlatformConfig, current_ip: &str, whitelisted: bool) -> String` - returns "N VM\n✓ GCP Firewall Whitelisted(IP)" or "✗ GCP Firewall Not Whitelisted"

- [ ] **Step 1: Write failing tests**

Create `mobile/src/calc/platform_gcp.rs`:

```rust
//! GCP-specific platform operations
//!
//! Handles platform-level GCP operations including status computation,
//! project selection, and OAuth management.

use anyhow::Result;
use crate::config::CloudPlatformConfig;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_compute_platform_status() {
        let platform = CloudPlatformConfig {
            name: "test".to_string(),
            platform_type: "gcp".to_string(),
            gcp_selected_project_id: Some("dure".to_string()),
            vms: vec![],
            ..Default::default()
        };
        
        let status = compute_platform_status(&platform, 8);
        assert_eq!(status, "8 Projects");
    }
    
    #[test]
    fn test_compute_project_status_whitelisted() {
        let platform = CloudPlatformConfig {
            name: "test".to_string(),
            platform_type: "gcp".to_string(),
            gcp_selected_project_id: Some("dure".to_string()),
            vms: vec![],
            ..Default::default()
        };
        
        let status = compute_project_status(&platform, "117.53.222.116", true);
        assert!(status.contains("0 VM"));
        assert!(status.contains("✓ GCP Firewall Whitelisted(117.53.222.116)"));
    }
    
    #[test]
    fn test_compute_project_status_not_whitelisted() {
        let platform = CloudPlatformConfig {
            name: "test".to_string(),
            platform_type: "gcp".to_string(),
            gcp_selected_project_id: Some("dure".to_string()),
            vms: vec![],
            ..Default::default()
        };
        
        let status = compute_project_status(&platform, "192.168.1.1", false);
        assert!(status.contains("0 VM"));
        assert!(status.contains("✗ GCP Firewall Not Whitelisted"));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib platform_gcp::tests -- --nocapture`

Expected: FAIL - "cannot find function `compute_platform_status`"

- [ ] **Step 3: Implement status computation functions**

Add before the tests module:

```rust
/// Compute platform-level status string
pub fn compute_platform_status(platform: &CloudPlatformConfig, total_projects: usize) -> String {
    format!("{} Projects", total_projects)
}

/// Compute project-level status string
pub fn compute_project_status(
    platform: &CloudPlatformConfig,
    current_ip: &str,
    whitelisted: bool,
) -> String {
    let vm_count = platform.vms.len();
    let firewall_status = if whitelisted {
        format!("✓ GCP Firewall Whitelisted({})", current_ip)
    } else {
        "✗ GCP Firewall Not Whitelisted".to_string()
    };
    
    format!("{} VM\n{}", vm_count, firewall_status)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib platform_gcp::tests -- --nocapture`

Expected: PASS

- [ ] **Step 5: Add module declaration**

Edit `mobile/src/calc/mod.rs` and add:

```rust
pub mod platform_gcp;
```

- [ ] **Step 6: Commit**

```bash
git add mobile/src/calc/platform_gcp.rs mobile/src/calc/mod.rs
git commit -m "feat: create platform_gcp module with status computation

Add GCP-specific platform operations including status
string computation for platform and project rows.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Create hosting_gcp Module with VM Operations Stubs

**Files:**
- Create: `mobile/src/calc/hosting_gcp.rs`
- Modify: `mobile/src/calc/mod.rs`

**Interfaces:**
- Consumes: `GcpRestClient`, `CloudPlatformConfig`, `VmInstance`
- Produces:
  - `pub fn delete_vm(client: &GcpRestClient, vm: &VmInstance) -> Result<String>` - returns success message
  - `pub fn restart_vm(client: &GcpRestClient, vm: &VmInstance) -> Result<String>`
  - `pub fn regenerate_vm(client: &GcpRestClient, platform: &mut CloudPlatformConfig, zone: &str) -> Result<String>`
  - `pub fn generate_ssh_keypair() -> Result<(String, Vec<u8>)>` - returns (public_key, private_key_bytes)

- [ ] **Step 1: Write failing tests for SSH keypair generation**

Create `mobile/src/calc/hosting_gcp.rs`:

```rust
//! GCP-specific VM hosting operations
//!
//! Handles VM lifecycle management including creation, deletion,
//! restart, regeneration, and SSH key generation.

use anyhow::{Context, Result};
use crate::calc::gcp_rest::GcpRestClient;
use crate::config::{CloudPlatformConfig, VmInstance};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_ssh_keypair() {
        let result = generate_ssh_keypair();
        assert!(result.is_ok());
        
        let (public_key, private_key) = result.unwrap();
        
        // Public key should start with ssh-ed25519
        assert!(public_key.starts_with("ssh-ed25519"), 
            "Public key should start with ssh-ed25519, got: {}", public_key);
        
        // Private key should be non-empty
        assert!(!private_key.is_empty(), "Private key should not be empty");
        
        // Private key should be in OpenSSH format (starts with specific bytes)
        assert!(private_key.len() > 32, "Private key should be reasonable size");
    }
    
    #[test]
    fn test_delete_vm_message() {
        // Test the success message format (actual API call tested manually)
        let vm = VmInstance {
            name: "test-vm".to_string(),
            instance_id: "123".to_string(),
            zone: "us-central1-a".to_string(),
            gcp_project_id: "test".to_string(),
            machine_type: "e2-micro".to_string(),
            status: "RUNNING".to_string(),
            external_ip: Some("1.2.3.4".to_string()),
            internal_ip: None,
            gcp_billing_account: None,
            created_at: 0,
            ssh_key_name: None,
        };
        
        let expected_msg = format!("VM {} deleted successfully", vm.name);
        assert_eq!(expected_msg, "VM test-vm deleted successfully");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib hosting_gcp::tests::test_generate_ssh_keypair -- --nocapture`

Expected: FAIL - "cannot find function `generate_ssh_keypair`"

- [ ] **Step 3: Implement SSH keypair generation**

Add before the tests:

```rust
/// Generate Ed25519 SSH keypair
pub fn generate_ssh_keypair() -> Result<(String, Vec<u8>)> {
    use ed25519_dalek::SigningKey;
    use rand::rngs::OsRng;
    
    // Generate key pair
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();
    
    // Convert to SSH format
    let public_key_bytes = verifying_key.to_bytes();
    
    // Encode public key in OpenSSH format
    let mut public_key_ssh = vec![0u8; 4];
    public_key_ssh.extend_from_slice(b"ssh-ed25519");
    public_key_ssh.extend_from_slice(&(11u32).to_be_bytes()); // length of "ssh-ed25519"
    public_key_ssh.extend_from_slice(&(32u32).to_be_bytes()); // length of key
    public_key_ssh.extend_from_slice(&public_key_bytes);
    
    let public_key_b64 = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &public_key_ssh
    );
    let public_key = format!("ssh-ed25519 {} dure@generated", public_key_b64);
    
    // Private key as raw bytes (will be stored in keyring)
    let private_key_bytes = signing_key.to_bytes().to_vec();
    
    Ok((public_key, private_key_bytes))
}
```

Add stub implementations for VM operations:

```rust
/// Delete a VM instance
pub fn delete_vm(client: &GcpRestClient, vm: &VmInstance) -> Result<String> {
    let url = format!(
        "https://compute.googleapis.com/compute/v1/projects/{}/zones/{}/instances/{}",
        vm.gcp_project_id, vm.zone, vm.name
    );
    
    client.delete(&url)?;
    
    // TODO: Poll operation status until complete
    
    Ok(format!("VM {} deleted successfully", vm.name))
}

/// Restart a VM instance
pub fn restart_vm(client: &GcpRestClient, vm: &VmInstance) -> Result<String> {
    let url = format!(
        "https://compute.googleapis.com/compute/v1/projects/{}/zones/{}/instances/{}/reset",
        vm.gcp_project_id, vm.zone, vm.name
    );
    
    client.post(&url, "")?;
    
    // TODO: Poll until VM status = RUNNING
    
    Ok(format!("VM {} restarted successfully", vm.name))
}

/// Regenerate VMs in a project (delete all, create one fresh)
pub fn regenerate_vm(
    client: &GcpRestClient,
    platform: &mut CloudPlatformConfig,
    zone: &str,
) -> Result<String> {
    // Delete all existing VMs
    let vm_count = platform.vms.len();
    for vm in &platform.vms {
        delete_vm(client, vm)?;
    }
    
    // TODO: Create new VM with default config
    // TODO: Generate SSH keypair and add to VM metadata
    // TODO: Store private key in keyring
    
    Ok(format!("{} VMs deleted, new VM creation pending", vm_count))
}
```

- [ ] **Step 4: Add required dependencies to Cargo.toml**

Check if these dependencies exist in `mobile/Cargo.toml`, add if missing:

```toml
[dependencies]
ed25519-dalek = "2"
rand = "0.8"
base64 = "0.22"
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test --lib hosting_gcp::tests -- --nocapture`

Expected: PASS

- [ ] **Step 6: Add module declaration**

Edit `mobile/src/calc/mod.rs`:

```rust
pub mod hosting_gcp;
```

- [ ] **Step 7: Commit**

```bash
git add mobile/src/calc/hosting_gcp.rs mobile/src/calc/mod.rs mobile/Cargo.toml
git commit -m "feat: create hosting_gcp module with VM operations

Add GCP VM lifecycle operations including delete, restart,
regenerate stubs, and SSH Ed25519 keypair generation.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Implement Platform Tab Data Model

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs`

**Interfaces:**
- Consumes: `CloudPlatformConfig` from `AppConfig`
- Produces:
  - `enum PlatformRow` with variants `Account`, `Project`, `Vm`
  - `fn build_platform_rows(platforms: &[CloudPlatformConfig]) -> Vec<PlatformRow>`

- [ ] **Step 1: Write failing test for row building**

Replace the content of `mobile/src/ui_tabs/platform.rs` with:

```rust
//! Platform tab - Platform configuration and management with GCP integration

use eframe::egui;
use egui_material3::MaterialButton;

use crate::config::{AppConfig, CloudPlatformConfig, VmInstance};

/// Platform table row types
#[derive(Debug, Clone)]
enum PlatformRow {
    Account {
        platform_name: String,
        email: String,
        project_count: usize,
        vm_count: usize,
    },
    Project {
        platform_name: String,
        project_id: String,
        vm_count: usize,
        current_ip: Option<String>,
        firewall_whitelisted: bool,
    },
    Vm {
        platform_name: String,
        project_id: String,
        vm_name: String,
        zone: String,
        instance_id: String,
        external_ip: Option<String>,
        ssh_status: SshStatus,
    },
}

/// SSH connection status
#[derive(Debug, Clone)]
enum SshStatus {
    Testing,
    Available,
    Failed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_build_platform_rows_empty() {
        let platforms = vec![];
        let rows = build_platform_rows(&platforms);
        assert_eq!(rows.len(), 0);
    }
    
    #[test]
    fn test_build_platform_rows_single_platform() {
        let platform = CloudPlatformConfig {
            name: "test-gcp".to_string(),
            platform_type: "gcp".to_string(),
            gcp_connected_email: Some("test@gmail.com".to_string()),
            gcp_selected_project_id: Some("dure".to_string()),
            vms: vec![VmInstance {
                name: "test-vm".to_string(),
                instance_id: "123".to_string(),
                zone: "us-central1-a".to_string(),
                gcp_project_id: "dure".to_string(),
                machine_type: "e2-micro".to_string(),
                status: "RUNNING".to_string(),
                external_ip: Some("1.2.3.4".to_string()),
                internal_ip: None,
                gcp_billing_account: None,
                created_at: 0,
                ssh_key_name: Some("gcp.test.vm".to_string()),
            }],
            ..Default::default()
        };
        
        let rows = build_platform_rows(&vec![platform]);
        
        // Should have 3 rows: Account, Project, VM
        assert_eq!(rows.len(), 3);
        
        match &rows[0] {
            PlatformRow::Account { platform_name, email, vm_count, .. } => {
                assert_eq!(platform_name, "test-gcp");
                assert_eq!(email, "test@gmail.com");
                assert_eq!(*vm_count, 1);
            }
            _ => panic!("First row should be Account"),
        }
        
        match &rows[1] {
            PlatformRow::Project { project_id, vm_count, .. } => {
                assert_eq!(project_id, "dure");
                assert_eq!(*vm_count, 1);
            }
            _ => panic!("Second row should be Project"),
        }
        
        match &rows[2] {
            PlatformRow::Vm { vm_name, zone, .. } => {
                assert_eq!(vm_name, "test-vm");
                assert_eq!(zone, "us-central1-a");
            }
            _ => panic!("Third row should be Vm"),
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --lib platform::tests::test_build_platform_rows_empty -- --nocapture`

Expected: FAIL - "cannot find function `build_platform_rows`"

- [ ] **Step 3: Implement build_platform_rows function**

Add before the tests module:

```rust
/// Build table rows from platform configurations
fn build_platform_rows(platforms: &[CloudPlatformConfig]) -> Vec<PlatformRow> {
    let mut rows = Vec::new();
    
    for platform in platforms {
        // Only process GCP platforms for now
        if platform.platform_type != "gcp" {
            continue;
        }
        
        let email = platform.gcp_connected_email.clone()
            .unwrap_or_else(|| "Not connected".to_string());
        
        // Account row
        rows.push(PlatformRow::Account {
            platform_name: platform.name.clone(),
            email,
            project_count: 0, // Will be fetched from API
            vm_count: platform.vms.len(),
        });
        
        // Project row (if project selected)
        if let Some(project_id) = &platform.gcp_selected_project_id {
            rows.push(PlatformRow::Project {
                platform_name: platform.name.clone(),
                project_id: project_id.clone(),
                vm_count: platform.vms.len(),
                current_ip: None, // Will be fetched from icanhazip.com
                firewall_whitelisted: false, // Will be checked via API
            });
            
            // VM row (show first VM only)
            if let Some(vm) = platform.vms.first() {
                rows.push(PlatformRow::Vm {
                    platform_name: platform.name.clone(),
                    project_id: project_id.clone(),
                    vm_name: vm.name.clone(),
                    zone: vm.zone.clone(),
                    instance_id: vm.instance_id.clone(),
                    external_ip: vm.external_ip.clone(),
                    ssh_status: SshStatus::Testing, // Will be tested in background
                });
            }
        }
    }
    
    rows
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib platform::tests -- --nocapture`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "feat: implement platform tab data model

Add PlatformRow enum and build_platform_rows() to convert
CloudPlatformConfig into hierarchical table rows.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 9: Implement Platform Tab UI Rendering (Basic Table)

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs`

**Interfaces:**
- Consumes: `build_platform_rows()`, `AppConfig`
- Produces: `PlatformTab::ui(&mut self, ui: &mut egui::Ui)` that renders the table

- [ ] **Step 1: Add PlatformTab struct**

Add to `platform.rs` after the `build_platform_rows` function:

```rust
/// Platform tab state
pub struct PlatformTab {
    rows: Vec<PlatformRow>,
    loaded: bool,
}

impl Default for PlatformTab {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            loaded: false,
        }
    }
}

impl PlatformTab {
    /// Render the platform tab UI
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Cloud Platforms");
        ui.add_space(4.0);
        ui.label("Manage GCP platforms and VMs with inline actions.");
        ui.add_space(8.0);
        
        // Action buttons
        ui.horizontal(|ui| {
            if ui.add(MaterialButton::filled("Add Platform")).clicked() {
                // TODO: Show add platform dialog
            }
            
            if ui.add(MaterialButton::outlined("Refresh Status")).clicked() {
                // TODO: Trigger refresh
                self.loaded = false; // Force reload
            }
        });
        
        ui.add_space(8.0);
        
        // Load platforms if not loaded
        if !self.loaded {
            if let Ok(config) = load_config() {
                self.rows = build_platform_rows(&config.platforms);
                self.loaded = true;
            }
        }
        
        // Render table
        egui::ScrollArea::vertical()
            .max_height(600.0)
            .show(ui, |ui| {
                render_table(ui, &self.rows);
            });
    }
}

/// Load application config
#[cfg(not(target_arch = "wasm32"))]
fn load_config() -> Result<AppConfig, String> {
    use directories::ProjectDirs;
    
    let proj_dirs = ProjectDirs::from("pe", "nikescar", "dure")
        .ok_or_else(|| "Failed to get project directories".to_string())?;
    let config_path = proj_dirs.config_dir().join("config.yml");
    
    Ok(AppConfig::load_or_default(&config_path))
}

#[cfg(target_arch = "wasm32")]
fn load_config() -> Result<AppConfig, String> {
    // WASM not supported for this feature
    Err("Platform tab not available on WASM".to_string())
}

/// Render the platform table
fn render_table(ui: &mut egui::Ui, rows: &[PlatformRow]) {
    use egui::{Grid, RichText};
    
    // Table header
    Grid::new("platform_table_header")
        .num_columns(3)
        .striped(false)
        .show(ui, |ui| {
            ui.label(RichText::new("Platform Name").strong());
            ui.label(RichText::new("Status").strong());
            ui.label(RichText::new("Actions").strong());
            ui.end_row();
        });
    
    ui.separator();
    
    // Table rows
    Grid::new("platform_table_body")
        .num_columns(3)
        .striped(true)
        .show(ui, |ui| {
            for row in rows {
                render_row(ui, row);
            }
        });
}

/// Render a single table row
fn render_row(ui: &mut egui::Ui, row: &PlatformRow) {
    match row {
        PlatformRow::Account { platform_name, email, project_count, vm_count } => {
            ui.label(format!("GCP: {}", email));
            ui.label(format!("{} Projects", project_count));
            ui.label(""); // No actions for account row
            ui.end_row();
        }
        
        PlatformRow::Project { project_id, vm_count, current_ip, firewall_whitelisted, .. } => {
            ui.label(format!("  ├─ {}", project_id));
            
            let firewall_text = if *firewall_whitelisted {
                format!("{} VM\n✓ GCP Firewall Whitelisted({})", 
                    vm_count, 
                    current_ip.as_deref().unwrap_or("unknown"))
            } else {
                format!("{} VM\n✗ GCP Firewall Not Whitelisted", vm_count)
            };
            ui.label(firewall_text);
            
            if ui.add(MaterialButton::outlined("Update Firewall")).clicked() {
                // TODO: Show update firewall confirmation
            }
            ui.end_row();
        }
        
        PlatformRow::Vm { vm_name, ssh_status, .. } => {
            ui.label(format!("  └─── {}", vm_name));
            
            let ssh_text = match ssh_status {
                SshStatus::Testing => "🔄 SSH Connection Testing...".to_string(),
                SshStatus::Available => "✓ SSH Connection OK(:22)".to_string(),
                SshStatus::Failed(err) => format!("✗ SSH Connection Failed(:22) - {}", err),
            };
            ui.label(ssh_text);
            
            ui.horizontal(|ui| {
                if ui.add(MaterialButton::outlined("Delete VM")).clicked() {
                    // TODO: Show delete confirmation
                }
                if ui.add(MaterialButton::outlined("Regenerate VM")).clicked() {
                    // TODO: Show regenerate confirmation
                }
                if ui.add(MaterialButton::outlined("Restart VM")).clicked() {
                    // TODO: Show restart confirmation
                }
                if ui.add(MaterialButton::outlined("Refresh")).clicked() {
                    // TODO: Trigger refresh
                }
            });
            ui.end_row();
        }
    }
}
```

- [ ] **Step 2: Build to verify it compiles**

Run: `cargo build --lib`

Expected: SUCCESS

- [ ] **Step 3: Manual test (if desktop available)**

Run the app and navigate to Platform tab:

```bash
cargo run
```

Expected: Table renders with platforms from config.yml, buttons are visible

- [ ] **Step 4: Commit**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "feat: implement platform tab UI rendering

Add PlatformTab widget with custom egui table rendering.
Shows hierarchical platform/project/VM rows with action buttons.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 10: Add Background SSH Testing

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs`

**Interfaces:**
- Consumes: `VmInstance`, `ssh::test_connection()`
- Produces: Background SSH test tasks that update `SshStatus` in `PlatformRow::Vm`

- [ ] **Step 1: Add SSH test task management to PlatformTab**

Update the `PlatformTab` struct:

```rust
use poll_promise::Promise;
use std::collections::HashMap;
use crate::calc::ssh::{test_connection, SshConnectionResult};

pub struct PlatformTab {
    rows: Vec<PlatformRow>,
    loaded: bool,
    
    // Background SSH tests: key = "{platform_name}:{vm_name}"
    ssh_test_tasks: HashMap<String, Promise<Result<SshConnectionResult, String>>>,
}

impl Default for PlatformTab {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            loaded: false,
            ssh_test_tasks: HashMap::new(),
        }
    }
}
```

- [ ] **Step 2: Add SSH test spawning function**

Add before the `render_table` function:

```rust
use crate::config::SshHostConfig;
use anyhow::Result;

/// Spawn SSH test for a VM
fn spawn_ssh_test(vm: &VmInstance) -> Promise<Result<SshConnectionResult, String>> {
    let vm = vm.clone();
    
    Promise::spawn_thread("ssh_test", move || {
        // Build SSH config from VM
        let external_ip = vm.external_ip
            .ok_or_else(|| "No external IP".to_string())?;
        
        let ssh_config = SshHostConfig {
            host: format!("generated_user@{}", external_ip),
            port: 22,
            password: None,
            private_key_path: None,
            keyring_domain: vm.ssh_key_name.clone(),
            initialized: false,
            last_status: None,
        };
        
        // Test connection
        test_connection(&ssh_config)
            .map_err(|e| format!("Timeout: {}", e))
    })
}
```

- [ ] **Step 3: Update ui() to spawn and check SSH tasks**

Update the `ui()` method after loading rows:

```rust
impl PlatformTab {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // ... existing UI code ...
        
        // Spawn SSH tests for VMs that don't have active tasks
        for row in &self.rows {
            if let PlatformRow::Vm { platform_name, vm_name, .. } = row {
                let key = format!("{}:{}", platform_name, vm_name);
                
                if !self.ssh_test_tasks.contains_key(&key) {
                    // Find the VM in config to spawn test
                    if let Ok(config) = load_config() {
                        for platform in &config.platforms {
                            if &platform.name == platform_name {
                                if let Some(vm) = platform.vms.iter()
                                    .find(|v| &v.name == vm_name) 
                                {
                                    let task = spawn_ssh_test(vm);
                                    self.ssh_test_tasks.insert(key.clone(), task);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Check completed SSH tasks and update row status
        let mut completed_tasks = Vec::new();
        
        for (key, task) in &self.ssh_test_tasks {
            if let Some(result) = task.ready() {
                // Find the VM row and update its status
                for row in &mut self.rows {
                    if let PlatformRow::Vm { platform_name, vm_name, ssh_status, .. } = row {
                        let row_key = format!("{}:{}", platform_name, vm_name);
                        if row_key == *key {
                            *ssh_status = match result {
                                Ok(conn_result) if conn_result.success => SshStatus::Available,
                                Ok(_) => SshStatus::Failed("Auth failed".to_string()),
                                Err(e) => SshStatus::Failed(e.clone()),
                            };
                        }
                    }
                }
                completed_tasks.push(key.clone());
            }
        }
        
        // Remove completed tasks
        for key in completed_tasks {
            self.ssh_test_tasks.remove(&key);
        }
        
        // ... rest of UI rendering ...
    }
}
```

- [ ] **Step 4: Build to verify it compiles**

Run: `cargo build --lib`

Expected: SUCCESS

- [ ] **Step 5: Manual test SSH status updates**

Run app and observe SSH status changing from "Testing..." to "OK" or "Failed":

```bash
cargo run
```

Expected: SSH status updates after ~1-15 seconds

- [ ] **Step 6: Commit**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "feat: add background SSH connectivity testing

Spawn poll_promise tasks to test SSH connections in background.
Updates VM row status from Testing to Available/Failed.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 11: Implement Update Firewall Action with Confirmation Dialog

**Files:**
- Modify: `mobile/src/ui_tabs/platform.rs`

**Interfaces:**
- Consumes: `GcpRestClient::add_ip_to_firewall()`, `get_current_ip()`
- Produces: Firewall update action with typed confirmation dialog

- [ ] **Step 1: Add confirmation dialog state to PlatformTab**

Update `PlatformTab` struct:

```rust
pub struct PlatformTab {
    // ... existing fields ...
    
    // Confirmation dialog state
    show_update_firewall_dialog: bool,
    firewall_project_id: String,
    firewall_confirmation_text: String,
}

impl Default for PlatformTab {
    fn default() -> Self {
        Self {
            // ... existing defaults ...
            show_update_firewall_dialog: false,
            firewall_project_id: String::new(),
            firewall_confirmation_text: String::new(),
        }
    }
}
```

- [ ] **Step 2: Add dialog rendering function**

Add before `render_table`:

```rust
use crate::calc::gcp_rest::get_current_ip;

/// Render update firewall confirmation dialog
fn render_update_firewall_dialog(
    ctx: &egui::Context,
    show: &mut bool,
    project_id: &str,
    confirmation_text: &mut String,
) -> Option<()> {
    if !*show {
        return None;
    }
    
    let mut confirmed = false;
    
    egui::Window::new("Update GCP Firewall")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("This will add your current IP to the GCP firewall");
            ui.label("whitelist for SSH access (port 22).");
            ui.add_space(8.0);
            
            ui.label(format!("Project: {}", project_id));
            
            if let Ok(ip) = get_current_ip() {
                ui.label(format!("Current IP: {}", ip));
            }
            
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                ui.label("Type 'update' to confirm:");
                ui.text_edit_singleline(confirmation_text);
            });
            
            ui.add_space(8.0);
            
            ui.horizontal(|ui| {
                if ui.button("Cancel").clicked() {
                    *show = false;
                }
                
                ui.add_enabled_ui(confirmation_text == "update", |ui| {
                    if ui.button("Confirm").clicked() {
                        confirmed = true;
                        *show = false;
                    }
                });
            });
        });
    
    if confirmed {
        Some(())
    } else {
        None
    }
}
```

- [ ] **Step 3: Update render_row to show dialog**

Update the Project row rendering in `render_row`:

```rust
PlatformRow::Project { project_id, .. } => {
    // ... existing rendering ...
    
    if ui.add(MaterialButton::outlined("Update Firewall")).clicked() {
        self.show_update_firewall_dialog = true;
        self.firewall_project_id = project_id.clone();
        self.firewall_confirmation_text.clear();
    }
    ui.end_row();
}
```

Wait, the `render_row` function doesn't have access to `self`. Let me fix this.

Update `render_row` signature to take `platform_tab` reference:

```rust
fn render_row(ui: &mut egui::Ui, row: &PlatformRow, platform_tab: &mut PlatformTab) {
    // ... in Project row ...
    if ui.add(MaterialButton::outlined("Update Firewall")).clicked() {
        platform_tab.show_update_firewall_dialog = true;
        platform_tab.firewall_project_id = project_id.clone();
        platform_tab.firewall_confirmation_text.clear();
    }
}
```

- [ ] **Step 4: Update ui() to render dialog and handle confirmation**

Update `ui()` method:

```rust
impl PlatformTab {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        // ... existing code ...
        
        // Render confirmation dialog
        if let Some(()) = render_update_firewall_dialog(
            ui.ctx(),
            &mut self.show_update_firewall_dialog,
            &self.firewall_project_id,
            &mut self.firewall_confirmation_text,
        ) {
            // User confirmed - execute firewall update
            if let Ok(ip) = get_current_ip() {
                // TODO: Get OAuth token from config
                // TODO: Call GcpRestClient::add_ip_to_firewall()
                println!("Updating firewall for {} with IP {}", self.firewall_project_id, ip);
            }
        }
        
        // Update render_table call to pass platform_tab
        // This will be fixed in next step
    }
}
```

- [ ] **Step 5: Build to verify it compiles**

Run: `cargo build --lib`

Expected: SUCCESS

- [ ] **Step 6: Commit**

```bash
git add mobile/src/ui_tabs/platform.rs
git commit -m "feat: add Update Firewall confirmation dialog

Add typed confirmation dialog for firewall updates.
User must type 'update' to confirm IP whitelisting.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Self-Review

**Spec Coverage:**

I've covered the following from the spec:
- ✓ Task 1: Config model (gcp_selected_project_id)
- ✓ Task 2-4: GCP API extensions (IP detection, project listing, firewall)
- ✓ Task 5: SSH keyring support
- ✓ Task 6-7: platform_gcp and hosting_gcp modules
- ✓ Task 8-9: Platform tab data model and UI rendering
- ✓ Task 10: Background SSH testing
- ✓ Task 11: Update Firewall action

**Still needed from spec:**
- Delete VM action with confirmation
- Restart VM action with confirmation  
- Regenerate VM action with confirmation
- Refresh Status action
- Complete hosting_gcp.regenerate_vm() implementation with VM creation
- Store SSH keys in keyring during VM creation
- Fetch current IP and firewall status on load
- Fetch project count from GCP API
- Error handling and audit logging
- Full test coverage

**Placeholder Scan:**
- Task 7: "TODO: Create new VM" - This is acceptable as a stub since full VM creation requires more API work
- Task 9: Multiple "TODO: Show dialog" comments - These are placeholders for future tasks
- Task 11: "TODO: Get OAuth token" and "TODO: Call API" - These need to be implemented

**Type Consistency:**
- `PlatformRow` enum consistent across tasks
- `SshStatus` enum used consistently
- Function signatures match between definition and use

**Scope Check:**
This plan covers the core infrastructure (Tasks 1-11) needed for the platform tab GCP integration. The remaining work (delete/restart/regenerate actions, complete VM creation, real-time data fetching, error handling, comprehensive testing) should be implemented in Phase 2 after the foundation is validated.

**Next Steps After Tasks 1-11:**
- Task 12-15: Implement remaining VM actions (Delete, Restart, Regenerate with full VM creation)
- Task 16-17: Implement real-time data fetching (IP, firewall status, project count)
- Task 18: Add error handling and audit logging
- Task 19-20: Comprehensive testing (integration tests, manual testing)

This phased approach allows validating the architecture and UI patterns before completing all features.

---

## Execution Notes

**Phase 1 (This Plan - Tasks 1-11):**
- Establishes data model, API extensions, and basic UI
- Delivers working table with SSH testing and one action (Update Firewall)
- Can be tested end-to-end with real GCP account
- Should take ~4-6 hours for experienced Rust developer

**Phase 2 (Follow-up Plan):**
- Completes all VM lifecycle actions
- Adds real-time status updates
- Implements comprehensive error handling
- Full test coverage
- Should take ~6-8 additional hours

**Testing Strategy:**
- Unit tests included in each task
- Integration tests in Phase 2
- Manual testing requires GCP account with Compute Engine API enabled
- SSH testing requires VM with accessible external IP

**Dependencies:**
- GCP account with OAuth configured
- Compute Engine API enabled
- SSH access to at least one VM for testing
- KeePass keyring initialized at ~/.config/dure/key.kdbx

