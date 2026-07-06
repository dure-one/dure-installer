# Platform CLI Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reimplement platform CLI commands to match GUI functionality using ViewModel/MVVM pattern with TDD approach.

**Architecture:** CLI commands create a ViewModel instance, send PlatformCommand messages to PlatformActor, poll for PlatformEvent responses, and display formatted output. Uses smol::block_on() to bridge sync CLI with async ViewModel.

**Tech Stack:** Rust, clap (CLI), smol (async runtime), existing ViewModel/PlatformActor infrastructure

## Global Constraints

- Rust nightly toolchain required
- Use existing ViewModel/PlatformActor infrastructure (no direct calc layer calls)
- Timeout for operations: 60 seconds
- Progress polling interval: 100ms
- Default VM zone: `us-central1-a`
- Default VM machine type: `e2-micro`
- Smart IP detection: `https://api.ipify.org`
- All tests must pass before each commit
- TDD: Write test → Run (fail) → Implement → Run (pass) → Commit

---

## File Structure

```
mobile/src/cli/
├── mod.rs                        # Update PlatformCommands enum, add command routing
├── commands/
│   ├── mod.rs                    # Re-export platform module  
│   └── platform/
│       ├── mod.rs                # Main command router, re-exports
│       ├── runner.rs             # PlatformCliRunner (ViewModel wrapper)
│       ├── list.rs               # List and show commands
│       ├── vm.rs                 # VM operations (add, delete, restart)
│       ├── firewall.rs           # Firewall command
│       ├── billing.rs            # Billing command
│       ├── helpers.rs            # Format and validation helpers
│       └── tests.rs              # Test suite with fixtures
```

**Decomposition Strategy:**
- `helpers.rs` - Pure functions for formatting and validation (no I/O)
- `runner.rs` - ViewModel lifecycle management (create, send, poll, cleanup)
- `list.rs`, `vm.rs`, `firewall.rs`, `billing.rs` - Command implementations using helpers + runner
- `mod.rs` - Command routing and CLI integration
- `tests.rs` - All test fixtures, mocks, and test cases

---

### Task 1: Setup Infrastructure and Test Fixtures

**Files:**
- Create: `mobile/src/cli/commands/platform/mod.rs`
- Create: `mobile/src/cli/commands/platform/tests.rs`
- Modify: `mobile/src/cli/commands/mod.rs`

**Interfaces:**
- Produces: `mock_platform_connected() -> CloudPlatformConfig`
- Produces: `mock_platform_no_vm() -> CloudPlatformConfig`
- Produces: `mock_platform_disconnected() -> CloudPlatformConfig`
- Produces: `MockPlatformRunner` for testing

- [ ] **Step 1: Create platform module skeleton**

Create `mobile/src/cli/commands/platform/mod.rs`:

```rust
//! Platform command implementation with ViewModel integration

pub mod runner;
pub mod list;
pub mod vm;
pub mod firewall;
pub mod billing;
pub mod helpers;

#[cfg(test)]
mod tests;

use anyhow::Result;

/// Execute platform commands
pub fn execute_platform_command(cmd: crate::cli::PlatformCommands) -> Result<()> {
    todo!("Router implementation in Task 2")
}
```

- [ ] **Step 2: Add re-export to commands/mod.rs**

Modify `mobile/src/cli/commands/mod.rs`:

```rust
pub mod platform;
```

- [ ] **Step 3: Create test fixtures file**

Create `mobile/src/cli/commands/platform/tests.rs`:

```rust
#![cfg(test)]

use crate::config::CloudPlatformConfig;
use crate::viewmodel::platform::{PlatformCommand, PlatformEvent};
use anyhow::{anyhow, Result};
use std::collections::VecDeque;

/// Mock platform with full OAuth and VM
pub fn mock_platform_connected() -> CloudPlatformConfig {
    CloudPlatformConfig {
        name: "test-gcp".to_string(),
        platform_type: "gcp".to_string(),
        gcp_oauth_access_token: Some("mock_token".to_string()),
        gcp_oauth_refresh_token: Some("mock_refresh".to_string()),
        gcp_oauth_token_expiry: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        gcp_connected_email: Some("test@example.com".to_string()),
        gcp_selected_project_id: Some("test-project-123".to_string()),
        vms: vec![
            crate::config::VmConfig {
                name: "test-vm".to_string(),
                zone: "us-central1-a".to_string(),
                machine_type: "e2-micro".to_string(),
                external_ip: Some("203.0.113.42".to_string()),
                ..Default::default()
            }
        ],
        ..Default::default()
    }
}

/// Mock platform with OAuth but no VMs
pub fn mock_platform_no_vm() -> CloudPlatformConfig {
    let mut platform = mock_platform_connected();
    platform.vms.clear();
    platform
}

/// Mock platform with no OAuth connection
pub fn mock_platform_disconnected() -> CloudPlatformConfig {
    CloudPlatformConfig {
        name: "test-gcp".to_string(),
        platform_type: "gcp".to_string(),
        gcp_oauth_access_token: None,
        ..Default::default()
    }
}

/// Mock ViewModel runner for testing
pub struct MockPlatformRunner {
    pub responses: VecDeque<PlatformEvent>,
}

impl MockPlatformRunner {
    pub fn new() -> Self {
        Self {
            responses: VecDeque::new(),
        }
    }
    
    pub fn expect_response(&mut self, response: PlatformEvent) {
        self.responses.push_back(response);
    }
    
    pub async fn execute_command(&mut self, _cmd: PlatformCommand) -> Result<PlatformEvent> {
        self.responses.pop_front()
            .ok_or_else(|| anyhow!("No more mock responses"))
    }
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo check --all-targets`
Expected: Success (no errors)

- [ ] **Step 5: Commit**

```bash
git add mobile/src/cli/commands/platform/
git commit -m "feat(cli): setup platform command infrastructure with test fixtures

- Create platform module skeleton
- Add test fixtures for connected/disconnected/no-vm platforms
- Add MockPlatformRunner for testing
- Re-export platform module from commands

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 2: Helpers - Format and Validation Functions (TDD)

**Files:**
- Create: `mobile/src/cli/commands/platform/helpers.rs`
- Modify: `mobile/src/cli/commands/platform/tests.rs`

**Interfaces:**
- Consumes: `CloudPlatformConfig` from config module
- Produces: `format_steps(platform: &CloudPlatformConfig) -> String`
- Produces: `format_drawer_content(platform: &CloudPlatformConfig) -> String`
- Produces: `validate_platform_ready(platform: &CloudPlatformConfig, operation: &str) -> Result<()>`

- [ ] **Step 1: Write test for format_steps**

Add to `mobile/src/cli/commands/platform/tests.rs`:

```rust
#[cfg(test)]
mod helpers_tests {
    use super::*;
    use crate::cli::commands::platform::helpers::*;

    #[test]
    fn test_format_steps_all_complete() {
        let platform = mock_platform_connected();
        let steps = format_steps(&platform);
        
        assert!(steps.contains("✓"));
        assert!(steps.contains("→"));
        assert!(steps.contains("GCP Connected"));
        assert!(steps.contains("Project Created"));
        assert!(steps.contains("VM Created"));
    }

    #[test]
    fn test_format_steps_no_vm() {
        let platform = mock_platform_no_vm();
        let steps = format_steps(&platform);
        
        assert!(steps.contains("✓ GCP Connected"));
        assert!(steps.contains("✓ Project Created"));
        assert!(steps.contains("✗ VM Created"));
    }

    #[test]
    fn test_format_steps_disconnected() {
        let platform = mock_platform_disconnected();
        let steps = format_steps(&platform);
        
        assert!(steps.contains("✗ GCP Connected"));
        assert!(steps.contains("✗ Project Created"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::helpers_tests::test_format_steps`
Expected: FAIL with "cannot find function `format_steps`"

- [ ] **Step 3: Create helpers.rs with format_steps**

Create `mobile/src/cli/commands/platform/helpers.rs`:

```rust
//! Helper functions for formatting and validation

use crate::config::CloudPlatformConfig;
use anyhow::{anyhow, Result};

/// Format connection progress steps with status indicators
pub fn format_steps(platform: &CloudPlatformConfig) -> String {
    let gcp = if platform.gcp_oauth_access_token.is_some() { "✓" } else { "✗" };
    let proj = if platform.gcp_selected_project_id.is_some() { "✓" } else { "✗" };
    let vm = if !platform.vms.is_empty() { "✓" } else { "✗" };
    
    // Firewall check simplified for now (would need GCP API call)
    let firewall = "?";
    
    // SSH ready if VM has external IP
    let ssh = if platform.vms.first().and_then(|v| v.external_ip.as_ref()).is_some() {
        "✓"
    } else {
        "✗"
    };

    format!(
        "{} GCP Connected → {} Project Created → {} VM Created → {} Firewall Rules Updated → {} SSH Connected",
        gcp, proj, vm, firewall, ssh
    )
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::helpers_tests::test_format_steps`
Expected: All 3 tests PASS

- [ ] **Step 5: Write test for format_drawer_content**

Add to `mobile/src/cli/commands/platform/tests.rs` in `helpers_tests` mod:

```rust
#[test]
fn test_format_drawer_content_connected() {
    let platform = mock_platform_connected();
    let content = format_drawer_content(&platform);
    
    assert!(content.contains("test@example.com"));
    assert!(content.contains("test-project-123"));
    assert!(content.contains("test-vm"));
    assert!(content.contains("203.0.113.42"));
}

#[test]
fn test_format_drawer_content_no_vm() {
    let platform = mock_platform_no_vm();
    let content = format_drawer_content(&platform);
    
    assert!(content.contains("test@example.com"));
    assert!(content.contains("test-project-123"));
    assert!(content.contains("No VM created"));
}
```

- [ ] **Step 6: Run test to verify it fails**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::helpers_tests::test_format_drawer_content`
Expected: FAIL with "cannot find function `format_drawer_content`"

- [ ] **Step 7: Implement format_drawer_content**

Add to `mobile/src/cli/commands/platform/helpers.rs`:

```rust
/// Format drawer content showing platform hierarchy
pub fn format_drawer_content(platform: &CloudPlatformConfig) -> String {
    let mut output = String::new();
    
    // Level 1: Email
    if let Some(email) = &platform.gcp_connected_email {
        output.push_str(&format!("{}\n", email));
    } else {
        output.push_str("Not connected\n");
    }
    
    // Level 2: Selected project
    if let Some(project_id) = &platform.gcp_selected_project_id {
        output.push_str(&format!("  └─ Project: {} (selected)\n", project_id));
        
        // Level 3: VM details
        if let Some(vm) = platform.vms.first() {
            let vm_display = if let Some(external_ip) = &vm.external_ip {
                format!("     └─ VM: {} ({})\n", vm.name, external_ip)
            } else {
                format!("     └─ VM: {} (no external IP)\n", vm.name)
            };
            output.push_str(&vm_display);
        } else {
            output.push_str("     └─ No VM created\n");
        }
    } else {
        output.push_str("  └─ No project selected\n");
    }
    
    output
}
```

- [ ] **Step 8: Run tests to verify they pass**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::helpers_tests::test_format_drawer_content`
Expected: All 2 tests PASS

- [ ] **Step 9: Write tests for validate_platform_ready**

Add to `mobile/src/cli/commands/platform/tests.rs` in `helpers_tests` mod:

```rust
#[test]
fn test_validate_platform_not_connected() {
    let platform = mock_platform_disconnected();
    let result = validate_platform_ready(&platform, "addvm");
    
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not connected"));
    assert!(err.contains("dure platform init"));
}

#[test]
fn test_validate_platform_no_project() {
    let mut platform = mock_platform_connected();
    platform.gcp_selected_project_id = None;
    
    let result = validate_platform_ready(&platform, "addvm");
    
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("No project selected"));
}

#[test]
fn test_validate_platform_ready_success() {
    let platform = mock_platform_connected();
    let result = validate_platform_ready(&platform, "addvm");
    
    assert!(result.is_ok());
}

#[test]
fn test_validate_platform_list_no_validation() {
    let platform = mock_platform_disconnected();
    let result = validate_platform_ready(&platform, "list");
    
    // List command doesn't require connection
    assert!(result.is_ok());
}
```

- [ ] **Step 10: Run test to verify it fails**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::helpers_tests::test_validate_platform`
Expected: FAIL with "cannot find function `validate_platform_ready`"

- [ ] **Step 11: Implement validate_platform_ready**

Add to `mobile/src/cli/commands/platform/helpers.rs`:

```rust
/// Validate platform is ready for the requested operation
pub fn validate_platform_ready(
    platform: &CloudPlatformConfig,
    operation: &str,
) -> Result<()> {
    // List/show commands don't require validation
    if ["list", "show", "delete"].contains(&operation) {
        return Ok(());
    }
    
    // Check OAuth
    if platform.gcp_oauth_access_token.is_none() {
        return Err(anyhow!(
            "Platform '{}' is not connected\n\
             Run 'dure platform init {}' to authenticate",
            platform.name, platform.name
        ));
    }
    
    // Check token expiry
    if let Some(expiry) = platform.gcp_oauth_token_expiry {
        if expiry < chrono::Utc::now() {
            return Err(anyhow!(
                "OAuth token expired\n\
                 Run 'dure platform init {}' to reconnect",
                platform.name
            ));
        }
    }
    
    // Check project for VM/firewall/billing operations
    let project_required = ["addvm", "firewall", "restart", "delvm", "billing"];
    if project_required.contains(&operation) {
        if platform.gcp_selected_project_id.is_none() {
            return Err(anyhow!(
                "No project selected for platform '{}'\n\
                 Run 'dure platform init {}' to select a project",
                platform.name, platform.name
            ));
        }
    }
    
    Ok(())
}
```

- [ ] **Step 12: Run tests to verify they pass**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::helpers_tests`
Expected: All 9 tests PASS

- [ ] **Step 13: Commit**

```bash
git add mobile/src/cli/commands/platform/helpers.rs mobile/src/cli/commands/platform/tests.rs
git commit -m "feat(cli): implement platform helpers with TDD

- Add format_steps for connection progress display
- Add format_drawer_content for detailed info
- Add validate_platform_ready for pre-flight checks
- All tests passing (9 tests)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 3: PlatformCliRunner - ViewModel Wrapper (TDD)

**Files:**
- Create: `mobile/src/cli/commands/platform/runner.rs`
- Modify: `mobile/src/cli/commands/platform/tests.rs`

**Interfaces:**
- Consumes: `PlatformCommand`, `PlatformEvent` from viewmodel
- Produces: `PlatformCliRunner::new() -> Self`
- Produces: `PlatformCliRunner::execute_command(cmd: PlatformCommand) -> Result<PlatformEvent>`

- [ ] **Step 1: Write test for runner creation**

Add to `mobile/src/cli/commands/platform/tests.rs`:

```rust
#[cfg(test)]
mod runner_tests {
    use super::*;
    use crate::cli::commands::platform::runner::*;

    #[test]
    fn test_runner_creation() {
        let runner = PlatformCliRunner::new();
        // Just verify it compiles and creates successfully
        drop(runner);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::runner_tests::test_runner_creation`
Expected: FAIL with "cannot find `PlatformCliRunner`"

- [ ] **Step 3: Create runner.rs skeleton**

Create `mobile/src/cli/commands/platform/runner.rs`:

```rust
//! PlatformCliRunner - ViewModel wrapper for CLI commands

use crate::viewmodel::{ViewModel, ViewModelEvent};
use crate::viewmodel::platform::{PlatformCommand, PlatformEvent};
use anyhow::{anyhow, Result};
use std::time::{Duration, Instant};

/// CLI-specific ViewModel runner
pub struct PlatformCliRunner {
    vm: ViewModel,
}

impl PlatformCliRunner {
    /// Create a new runner with ViewModel
    pub fn new() -> Self {
        Self {
            vm: ViewModel::new(),
        }
    }
    
    /// Execute a platform command and wait for result
    pub async fn execute_command(&mut self, cmd: PlatformCommand) -> Result<PlatformEvent> {
        todo!("Implementation in next step")
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::runner_tests::test_runner_creation`
Expected: PASS

- [ ] **Step 5: Write test for execute_command with timeout**

Add to `mobile/src/cli/commands/platform/tests.rs` in `runner_tests` mod:

```rust
#[smol::test]
async fn test_execute_command_success() {
    let mut runner = MockPlatformRunner::new();
    runner.expect_response(PlatformEvent::FirewallUpdated {
        platform_name: "test-gcp".to_string(),
        whitelisted_ip: "203.0.113.42".to_string(),
    });
    
    let result = runner.execute_command(
        PlatformCommand::UpdateFirewall {
            platform_name: "test-gcp".to_string(),
            allow_ip: "203.0.113.42".to_string(),
        }
    ).await;
    
    assert!(result.is_ok());
    if let Ok(PlatformEvent::FirewallUpdated { whitelisted_ip, .. }) = result {
        assert_eq!(whitelisted_ip, "203.0.113.42");
    } else {
        panic!("Expected FirewallUpdated event");
    }
}

#[smol::test]
async fn test_execute_command_error() {
    let mut runner = MockPlatformRunner::new();
    runner.expect_response(PlatformEvent::Error {
        operation: "update_firewall".to_string(),
        error: "Permission denied".to_string(),
    });
    
    let result = runner.execute_command(
        PlatformCommand::UpdateFirewall {
            platform_name: "test-gcp".to_string(),
            allow_ip: "203.0.113.42".to_string(),
        }
    ).await;
    
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Permission denied"));
}
```

- [ ] **Step 6: Run test to verify it fails**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::runner_tests::test_execute_command`
Expected: FAIL with "not yet implemented"

- [ ] **Step 7: Implement execute_command**

Update `mobile/src/cli/commands/platform/runner.rs`:

```rust
impl PlatformCliRunner {
    // ... existing new() ...
    
    /// Execute a platform command and wait for result
    pub async fn execute_command(&mut self, cmd: PlatformCommand) -> Result<PlatformEvent> {
        // Send command to ViewModel
        self.vm.platform_send(cmd)?;
        
        // Poll for result event with timeout
        let timeout = Duration::from_secs(60);
        let start = Instant::now();
        
        loop {
            if start.elapsed() > timeout {
                return Err(anyhow!("Operation timed out after 60 seconds"));
            }
            
            // Check for events
            let events = self.vm.poll_events(&egui::Context::default());
            for event in events {
                if let ViewModelEvent::Platform(platform_event) = event {
                    match platform_event {
                        PlatformEvent::Error { error, .. } => {
                            return Err(anyhow!("{}", error));
                        }
                        _ => {
                            return Ok(platform_event);
                        }
                    }
                }
            }
            
            // Sleep before next poll
            smol::Timer::after(Duration::from_millis(100)).await;
        }
    }
}
```

- [ ] **Step 8: Run tests to verify they pass**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::runner_tests`
Expected: All 3 tests PASS

- [ ] **Step 9: Commit**

```bash
git add mobile/src/cli/commands/platform/runner.rs mobile/src/cli/commands/platform/tests.rs
git commit -m "feat(cli): implement PlatformCliRunner with TDD

- ViewModel wrapper for CLI commands
- Execute command with 60s timeout and 100ms polling
- Handle success and error events
- Tests passing (3 tests)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 4: List and Show Commands (TDD)

**Files:**
- Create: `mobile/src/cli/commands/platform/list.rs`
- Modify: `mobile/src/cli/commands/platform/tests.rs`

**Interfaces:**
- Consumes: `format_steps()`, `format_drawer_content()` from helpers
- Produces: `execute_platform_list() -> Result<()>`
- Produces: `execute_platform_show(name: String) -> Result<()>`

- [ ] **Step 1: Write test for list command with no platforms**

Add to `mobile/src/cli/commands/platform/tests.rs`:

```rust
#[cfg(test)]
mod list_tests {
    use super::*;
    use crate::cli::commands::platform::list::*;

    #[test]
    fn test_list_empty() {
        // Create empty config
        let config = crate::config::AppConfig {
            platforms: vec![],
            ..Default::default()
        };
        
        // This is a display test - just verify it compiles
        // Real test would check output, but that requires mocking println
        let result = format_platform_list(&config);
        assert!(result.contains("No platforms configured"));
    }

    #[test]
    fn test_list_with_platforms() {
        let config = crate::config::AppConfig {
            platforms: vec![
                mock_platform_connected(),
                mock_platform_no_vm(),
            ],
            ..Default::default()
        };
        
        let result = format_platform_list(&config);
        assert!(result.contains("test-gcp"));
        assert!(result.contains("GCP"));
        assert!(result.contains("✓"));
        assert!(result.contains("→"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::list_tests`
Expected: FAIL with "cannot find function `format_platform_list`"

- [ ] **Step 3: Create list.rs with format_platform_list**

Create `mobile/src/cli/commands/platform/list.rs`:

```rust
//! List and show commands for platforms

use crate::config::AppConfig;
use crate::cli::commands::platform::helpers::*;
use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Get config file path
fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
        .ok_or_else(|| anyhow!("Failed to get project directories"))?;
    Ok(proj_dirs.config_dir().join("config.yml"))
}

/// Load application config
fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path()?;
    Ok(AppConfig::load_or_default(&config_path))
}

/// Format platform list for display
pub fn format_platform_list(config: &AppConfig) -> String {
    if config.platforms.is_empty() {
        return "No platforms configured\n\nAdd a platform with: dure platform add <name> <type>".to_string();
    }
    
    let mut output = String::new();
    output.push_str("Platform Status:\n");
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!("{:<20} {:<8} {}\n", "Name", "Type", "Steps"));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    for platform in &config.platforms {
        let steps = format_steps(platform);
        output.push_str(&format!("{:<20} {:<8} {}\n", 
            platform.name,
            platform.platform_type.to_uppercase(),
            steps.chars().take(50).collect::<String>()
        ));
    }
    
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    output.push_str(&format!("\nSteps: Connected → Project → VM → Firewall → SSH\n"));
    output.push_str(&format!("\nTotal platforms: {}\n", config.platforms.len()));
    output.push_str("\nUse 'dure platform <name>' to see details and available actions.\n");
    
    output
}

/// Execute platform list command
pub fn execute_platform_list() -> Result<()> {
    let config = load_config()?;
    let output = format_platform_list(&config);
    println!("{}", output);
    Ok(())
}

/// Execute platform show command
pub fn execute_platform_show(name: String) -> Result<()> {
    let config = load_config()?;
    
    let platform = config.platforms.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| {
            let available: Vec<_> = config.platforms.iter().map(|p| &p.name).collect();
            anyhow!(
                "Platform '{}' not found\n\nAvailable platforms:\n{}\n\nRun 'dure platform' to list all platforms.",
                name,
                available.iter().map(|n| format!("  • {}", n)).collect::<Vec<_>>().join("\n")
            )
        })?;
    
    println!("Platform: {}", platform.name);
    println!("Type: {}", platform.platform_type.to_uppercase());
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    println!("Connection Steps:");
    let steps = format_steps(platform);
    for line in steps.lines() {
        println!("  {}", line);
    }
    println!();
    
    println!("Details:");
    let details = format_drawer_content(platform);
    for line in details.lines() {
        println!("  {}", line);
    }
    println!();
    
    println!("Available Actions:");
    println!("  refresh   - Refresh platform data");
    if platform.vms.is_empty() {
        println!("  addvm     - Add a new VM");
    } else {
        println!("  addvm     - Add a new VM (disabled: VM already exists)");
    }
    println!("  firewall  - Update firewall rules");
    if !platform.vms.is_empty() {
        println!("  restart   - Restart VM");
        println!("  delvm     - Delete VM");
    }
    if platform.gcp_selected_project_id.is_some() {
        println!("  billing   - Show billing information");
    }
    println!("  delete    - Delete platform");
    println!("\nRun: dure platform {} <action>", name);
    
    Ok(())
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::list_tests`
Expected: All 2 tests PASS

- [ ] **Step 5: Write test for show command**

Add to `mobile/src/cli/commands/platform/tests.rs` in `list_tests` mod:

```rust
#[test]
fn test_show_platform_not_found() {
    let config = crate::config::AppConfig {
        platforms: vec![mock_platform_connected()],
        ..Default::default()
    };
    
    let result = format_platform_show(&config, "nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[test]
fn test_show_platform_found() {
    let config = crate::config::AppConfig {
        platforms: vec![mock_platform_connected()],
        ..Default::default()
    };
    
    let result = format_platform_show(&config, "test-gcp");
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("test-gcp"));
    assert!(output.contains("test@example.com"));
    assert!(output.contains("Available Actions"));
}
```

- [ ] **Step 6: Add format_platform_show helper**

Add to `mobile/src/cli/commands/platform/list.rs` before `execute_platform_show`:

```rust
/// Format platform show output
pub fn format_platform_show(config: &AppConfig, name: &str) -> Result<String> {
    let platform = config.platforms.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| {
            let available: Vec<_> = config.platforms.iter().map(|p| &p.name).collect();
            anyhow!(
                "Platform '{}' not found\n\nAvailable platforms:\n{}",
                name,
                available.iter().map(|n| format!("  • {}", n)).collect::<Vec<_>>().join("\n")
            )
        })?;
    
    let mut output = String::new();
    output.push_str(&format!("Platform: {}\n", platform.name));
    output.push_str(&format!("Type: {}\n", platform.platform_type.to_uppercase()));
    output.push_str("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\n");
    
    output.push_str("Connection Steps:\n");
    let steps = format_steps(platform);
    for step in steps.split("→") {
        output.push_str(&format!("  {}\n", step.trim()));
    }
    output.push('\n');
    
    output.push_str("Details:\n");
    let details = format_drawer_content(platform);
    for line in details.lines() {
        output.push_str(&format!("  {}\n", line));
    }
    output.push('\n');
    
    output.push_str("Available Actions:\n");
    output.push_str("  refresh   - Refresh platform data\n");
    if platform.vms.is_empty() {
        output.push_str("  addvm     - Add a new VM\n");
    } else {
        output.push_str("  addvm     - Add a new VM (disabled: VM already exists)\n");
    }
    output.push_str("  firewall  - Update firewall rules\n");
    if !platform.vms.is_empty() {
        output.push_str("  restart   - Restart VM\n");
        output.push_str("  delvm     - Delete VM\n");
    }
    if platform.gcp_selected_project_id.is_some() {
        output.push_str("  billing   - Show billing information\n");
    }
    output.push_str("  delete    - Delete platform\n");
    output.push_str(&format!("\nRun: dure platform {} <action>\n", name));
    
    Ok(output)
}
```

Update `execute_platform_show`:

```rust
/// Execute platform show command
pub fn execute_platform_show(name: String) -> Result<()> {
    let config = load_config()?;
    let output = format_platform_show(&config, &name)?;
    println!("{}", output);
    Ok(())
}
```

- [ ] **Step 7: Run tests to verify they pass**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::list_tests`
Expected: All 4 tests PASS

- [ ] **Step 8: Commit**

```bash
git add mobile/src/cli/commands/platform/list.rs mobile/src/cli/commands/platform/tests.rs
git commit -m "feat(cli): implement list and show commands with TDD

- List all platforms with steps summary
- Show detailed platform info and available actions
- Format helpers with table layout
- Tests passing (4 tests)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 5: Firewall Command (TDD)

**Files:**
- Create: `mobile/src/cli/commands/platform/firewall.rs`
- Modify: `mobile/src/cli/commands/platform/tests.rs`

**Interfaces:**
- Consumes: `PlatformCliRunner::execute_command()` from runner
- Consumes: `validate_platform_ready()` from helpers
- Produces: `execute_firewall_command(name: String, ip: Option<String>) -> Result<()>`

- [ ] **Step 1: Write test for firewall with explicit IP**

Add to `mobile/src/cli/commands/platform/tests.rs`:

```rust
#[cfg(test)]
mod firewall_tests {
    use super::*;

    #[smol::test]
    async fn test_firewall_with_explicit_ip() {
        let mut runner = MockPlatformRunner::new();
        runner.expect_response(PlatformEvent::FirewallUpdated {
            platform_name: "test-gcp".to_string(),
            whitelisted_ip: "203.0.113.42".to_string(),
        });
        
        let platform = mock_platform_connected();
        let result = crate::cli::commands::platform::firewall::execute_firewall_inner(
            &mut runner,
            &platform,
            Some("203.0.113.42".to_string())
        ).await;
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_firewall_validation_not_connected() {
        let platform = mock_platform_disconnected();
        let result = crate::cli::commands::platform::helpers::validate_platform_ready(
            &platform,
            "firewall"
        );
        
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::firewall_tests`
Expected: FAIL with "cannot find `firewall` module"

- [ ] **Step 3: Create firewall.rs**

Create `mobile/src/cli/commands/platform/firewall.rs`:

```rust
//! Firewall command implementation

use crate::config::AppConfig;
use crate::cli::commands::platform::helpers::*;
use crate::cli::commands::platform::runner::PlatformCliRunner;
use crate::viewmodel::platform::{PlatformCommand, PlatformEvent};
use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Get config file path
fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
        .ok_or_else(|| anyhow!("Failed to get project directories"))?;
    Ok(proj_dirs.config_dir().join("config.yml"))
}

/// Load application config
fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path()?;
    Ok(AppConfig::load_or_default(&config_path))
}

/// Auto-detect current IP address
async fn get_current_ip(ip_flag: Option<String>) -> Result<String> {
    if let Some(ip) = ip_flag {
        return Ok(ip);
    }
    
    // Try ipify API
    match ureq::get("https://api.ipify.org").call() {
        Ok(resp) => Ok(resp.into_string()?),
        Err(e) => {
            Err(anyhow!(
                "Failed to auto-detect IP: {}\n\
                 Use --ip <address> to specify manually",
                e
            ))
        }
    }
}

/// Execute firewall command (inner function for testing)
#[cfg(test)]
pub async fn execute_firewall_inner(
    runner: &mut crate::cli::commands::platform::tests::MockPlatformRunner,
    platform: &crate::config::CloudPlatformConfig,
    ip: Option<String>,
) -> Result<()> {
    let allow_ip = get_current_ip(ip).await?;
    
    let event = runner.execute_command(PlatformCommand::UpdateFirewall {
        platform_name: platform.name.clone(),
        allow_ip: allow_ip.clone(),
    }).await?;
    
    if let PlatformEvent::FirewallUpdated { whitelisted_ip, .. } = event {
        println!("✓ Updated firewall rules");
        println!("✓ Whitelisted IP: {}", whitelisted_ip);
        Ok(())
    } else {
        Err(anyhow!("Unexpected event: {:?}", event))
    }
}

/// Execute firewall command
pub fn execute_firewall_command(name: String, ip: Option<String>) -> Result<()> {
    let config = load_config()?;
    
    let platform = config.platforms.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;
    
    // Validate platform is ready
    validate_platform_ready(platform, "firewall")?;
    
    smol::block_on(async {
        let mut runner = PlatformCliRunner::new();
        let allow_ip = get_current_ip(ip).await?;
        
        println!("✓ Detected current IP: {}", allow_ip);
        
        let event = runner.execute_command(PlatformCommand::UpdateFirewall {
            platform_name: platform.name.clone(),
            allow_ip: allow_ip.clone(),
        }).await?;
        
        if let PlatformEvent::FirewallUpdated { whitelisted_ip, .. } = event {
            println!("✓ Updated firewall rules for project '{}'", 
                platform.gcp_selected_project_id.as_deref().unwrap_or("unknown"));
            println!("✓ Whitelisted IP: {}", whitelisted_ip);
            Ok(())
        } else {
            Err(anyhow!("Unexpected event: {:?}", event))
        }
    })
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::firewall_tests`
Expected: All 2 tests PASS

- [ ] **Step 5: Commit**

```bash
git add mobile/src/cli/commands/platform/firewall.rs mobile/src/cli/commands/platform/tests.rs
git commit -m "feat(cli): implement firewall command with TDD

- Auto-detect current IP via ipify API
- Update firewall rules via ViewModel
- Validation for connected platform
- Tests passing (2 tests)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 6: VM Commands (TDD)

**Files:**
- Create: `mobile/src/cli/commands/platform/vm.rs`
- Modify: `mobile/src/cli/commands/platform/tests.rs`

**Interfaces:**
- Consumes: `PlatformCliRunner::execute_command()` from runner
- Consumes: `validate_platform_ready()` from helpers
- Produces: `execute_addvm_command(name: String, vm_name: Option<String>, zone: Option<String>, machine_type: Option<String>) -> Result<()>`
- Produces: `execute_restart_command(name: String, vm: Option<String>) -> Result<()>`
- Produces: `execute_delvm_command(name: String, vm: Option<String>) -> Result<()>`

- [ ] **Step 1: Write test for addvm**

Add to `mobile/src/cli/commands/platform/tests.rs`:

```rust
#[cfg(test)]
mod vm_tests {
    use super::*;

    #[smol::test]
    async fn test_addvm_success() {
        let mut runner = MockPlatformRunner::new();
        runner.expect_response(PlatformEvent::VMCreated {
            platform_name: "test-gcp".to_string(),
            vm_name: "test-vm".to_string(),
            external_ip: "203.0.113.50".to_string(),
        });
        
        let platform = mock_platform_no_vm();
        let result = crate::cli::commands::platform::vm::execute_addvm_inner(
            &mut runner,
            &platform,
            "test-vm".to_string(),
            "us-central1-a".to_string(),
            "e2-micro".to_string(),
        ).await;
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_select_vm_single() {
        let platform = mock_platform_connected();
        let result = crate::cli::commands::platform::vm::select_vm(&platform, None);
        
        assert!(result.is_ok());
        let (name, zone) = result.unwrap();
        assert_eq!(name, "test-vm");
        assert_eq!(zone, "us-central1-a");
    }

    #[test]
    fn test_select_vm_none() {
        let platform = mock_platform_no_vm();
        let result = crate::cli::commands::platform::vm::select_vm(&platform, None);
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No VMs found"));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::vm_tests`
Expected: FAIL with "cannot find `vm` module"

- [ ] **Step 3: Create vm.rs**

Create `mobile/src/cli/commands/platform/vm.rs`:

```rust
//! VM operation commands

use crate::config::{AppConfig, CloudPlatformConfig};
use crate::cli::commands::platform::helpers::*;
use crate::cli::commands::platform::runner::PlatformCliRunner;
use crate::viewmodel::platform::{PlatformCommand, PlatformEvent};
use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Get config file path
fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
        .ok_or_else(|| anyhow!("Failed to get project directories"))?;
    Ok(proj_dirs.config_dir().join("config.yml"))
}

/// Load application config
fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path()?;
    Ok(AppConfig::load_or_default(&config_path))
}

/// Select VM from platform (auto-select if one, error if none)
pub fn select_vm(platform: &CloudPlatformConfig, vm_flag: Option<String>) -> Result<(String, String)> {
    if let Some(vm_name) = vm_flag {
        let vm = platform.vms.iter().find(|v| v.name == vm_name)
            .ok_or_else(|| anyhow!("VM '{}' not found", vm_name))?;
        return Ok((vm.name.clone(), vm.zone.clone()));
    }
    
    match platform.vms.len() {
        0 => Err(anyhow!("No VMs found")),
        1 => {
            let vm = &platform.vms[0];
            Ok((vm.name.clone(), vm.zone.clone()))
        }
        _ => {
            // Multiple VMs - in real CLI would prompt, for now error
            Err(anyhow!(
                "Multiple VMs found. Use --vm <name> to specify:\n{}",
                platform.vms.iter()
                    .map(|v| format!("  • {} ({})", v.name, v.zone))
                    .collect::<Vec<_>>()
                    .join("\n")
            ))
        }
    }
}

/// Execute addvm command (inner function for testing)
#[cfg(test)]
pub async fn execute_addvm_inner(
    runner: &mut crate::cli::commands::platform::tests::MockPlatformRunner,
    platform: &CloudPlatformConfig,
    vm_name: String,
    zone: String,
    machine_type: String,
) -> Result<()> {
    let event = runner.execute_command(PlatformCommand::CreateVM {
        platform_name: platform.name.clone(),
        vm_name: vm_name.clone(),
        zone: zone.clone(),
        machine_type: machine_type.clone(),
    }).await?;
    
    if let PlatformEvent::VMCreated { vm_name, external_ip, .. } = event {
        println!("✓ VM created successfully");
        println!("  Name: {}", vm_name);
        println!("  Zone: {}", zone);
        println!("  External IP: {}", external_ip);
        Ok(())
    } else {
        Err(anyhow!("Unexpected event: {:?}", event))
    }
}

/// Execute addvm command
pub fn execute_addvm_command(
    name: String,
    vm_name_flag: Option<String>,
    zone_flag: Option<String>,
    machine_type_flag: Option<String>,
) -> Result<()> {
    let config = load_config()?;
    
    let platform = config.platforms.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;
    
    // Validate platform is ready
    validate_platform_ready(platform, "addvm")?;
    
    // Check if VM already exists
    if !platform.vms.is_empty() {
        return Err(anyhow!(
            "Platform '{}' already has a VM: {}\n\n\
             To create a new VM, first delete the existing one:\n  \
             dure platform {} delvm",
            platform.name,
            platform.vms[0].name,
            platform.name
        ));
    }
    
    // Get VM parameters (use defaults or prompt)
    let vm_name = vm_name_flag.ok_or_else(|| 
        anyhow!("VM name required. Use --vm-name <name>")
    )?;
    let zone = zone_flag.unwrap_or_else(|| "us-central1-a".to_string());
    let machine_type = machine_type_flag.unwrap_or_else(|| "e2-micro".to_string());
    
    println!("Creating VM...");
    println!("  Name: {}", vm_name);
    println!("  Zone: {}", zone);
    println!("  Machine Type: {}", machine_type);
    println!();
    
    smol::block_on(async {
        let mut runner = PlatformCliRunner::new();
        
        let event = runner.execute_command(PlatformCommand::CreateVM {
            platform_name: platform.name.clone(),
            vm_name: vm_name.clone(),
            zone: zone.clone(),
            machine_type: machine_type.clone(),
        }).await?;
        
        if let PlatformEvent::VMCreated { vm_name, external_ip, .. } = event {
            println!("✓ VM created successfully");
            println!("  Name: {}", vm_name);
            println!("  Zone: {}", zone);
            println!("  External IP: {}", external_ip);
            Ok(())
        } else {
            Err(anyhow!("Unexpected event: {:?}", event))
        }
    })
}

/// Execute restart command
pub fn execute_restart_command(name: String, vm_flag: Option<String>) -> Result<()> {
    let config = load_config()?;
    
    let platform = config.platforms.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;
    
    // Validate platform is ready
    validate_platform_ready(platform, "restart")?;
    
    // Select VM
    let (vm_name, zone) = select_vm(platform, vm_flag)?;
    
    println!("Restarting VM '{}'...", vm_name);
    
    smol::block_on(async {
        let mut runner = PlatformCliRunner::new();
        
        let event = runner.execute_command(PlatformCommand::RestartVM {
            platform_name: platform.name.clone(),
            vm_name: vm_name.clone(),
            zone: zone.clone(),
        }).await?;
        
        if let PlatformEvent::VMRestarted { vm_name, .. } = event {
            println!("✓ VM '{}' restarted successfully", vm_name);
            Ok(())
        } else {
            Err(anyhow!("Unexpected event: {:?}", event))
        }
    })
}

/// Execute delvm command
pub fn execute_delvm_command(name: String, vm_flag: Option<String>) -> Result<()> {
    let config = load_config()?;
    
    let platform = config.platforms.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;
    
    // Validate platform is ready
    validate_platform_ready(platform, "delvm")?;
    
    // Select VM
    let (vm_name, zone) = select_vm(platform, vm_flag)?;
    
    println!("⚠️  Delete VM '{}'? This cannot be undone.", vm_name);
    println!("Type 'yes' to confirm: ");
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if input.trim() != "yes" {
        println!("Cancelled");
        return Ok(());
    }
    
    println!("Deleting VM '{}'...", vm_name);
    
    smol::block_on(async {
        let mut runner = PlatformCliRunner::new();
        
        let event = runner.execute_command(PlatformCommand::DeleteVM {
            platform_name: platform.name.clone(),
            vm_name: vm_name.clone(),
            zone: zone.clone(),
        }).await?;
        
        if let PlatformEvent::VMDeleted { vm_name, .. } = event {
            println!("✓ VM '{}' deleted successfully", vm_name);
            Ok(())
        } else {
            Err(anyhow!("Unexpected event: {:?}", event))
        }
    })
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::vm_tests`
Expected: All 3 tests PASS

- [ ] **Step 5: Commit**

```bash
git add mobile/src/cli/commands/platform/vm.rs mobile/src/cli/commands/platform/tests.rs
git commit -m "feat(cli): implement VM commands with TDD

- Add VM with defaults (zone: us-central1-a, type: e2-micro)
- Restart VM with auto-selection
- Delete VM with confirmation
- Tests passing (3 tests)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 7: Billing Command (TDD)

**Files:**
- Create: `mobile/src/cli/commands/platform/billing.rs`
- Modify: `mobile/src/cli/commands/platform/tests.rs`

**Interfaces:**
- Consumes: `PlatformCliRunner::execute_command()` from runner
- Consumes: `validate_platform_ready()` from helpers
- Produces: `execute_billing_command(name: String) -> Result<()>`

- [ ] **Step 1: Write test for billing**

Add to `mobile/src/cli/commands/platform/tests.rs`:

```rust
#[cfg(test)]
mod billing_tests {
    use super::*;
    use crate::calc::gcp_rest::BillingRecord;

    #[smol::test]
    async fn test_billing_success() {
        let mut runner = MockPlatformRunner::new();
        runner.expect_response(PlatformEvent::BillingFetched {
            platform_name: "test-gcp".to_string(),
            records: vec![
                BillingRecord {
                    month: "2026-07".to_string(),
                    cost: 12.45,
                },
                BillingRecord {
                    month: "2026-06".to_string(),
                    cost: 11.89,
                },
                BillingRecord {
                    month: "2026-05".to_string(),
                    cost: 13.20,
                },
            ],
        });
        
        let platform = mock_platform_connected();
        let result = crate::cli::commands::platform::billing::execute_billing_inner(
            &mut runner,
            &platform,
        ).await;
        
        assert!(result.is_ok());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::billing_tests`
Expected: FAIL with "cannot find `billing` module"

- [ ] **Step 3: Create billing.rs**

Create `mobile/src/cli/commands/platform/billing.rs`:

```rust
//! Billing command implementation

use crate::config::AppConfig;
use crate::cli::commands::platform::helpers::*;
use crate::cli::commands::platform::runner::PlatformCliRunner;
use crate::viewmodel::platform::{PlatformCommand, PlatformEvent};
use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Get config file path
fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
        .ok_or_else(|| anyhow!("Failed to get project directories"))?;
    Ok(proj_dirs.config_dir().join("config.yml"))
}

/// Load application config
fn load_config() -> Result<AppConfig> {
    let config_path = get_config_path()?;
    Ok(AppConfig::load_or_default(&config_path))
}

/// Execute billing command (inner function for testing)
#[cfg(test)]
pub async fn execute_billing_inner(
    runner: &mut crate::cli::commands::platform::tests::MockPlatformRunner,
    platform: &crate::config::CloudPlatformConfig,
) -> Result<()> {
    // For testing, use placeholder values
    let project_id = platform.gcp_selected_project_id.as_ref()
        .ok_or_else(|| anyhow!("No project selected"))?;
    
    let event = runner.execute_command(PlatformCommand::FetchBilling {
        platform_name: platform.name.clone(),
        project_id: project_id.clone(),
        dataset: "billing_export".to_string(),
        table: "gcp_billing_export".to_string(),
    }).await?;
    
    if let PlatformEvent::BillingFetched { records, .. } = event {
        println!("Billing Summary (Last 3 Months):");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("{:<12} {}", "Month", "Cost (USD)");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        let mut total = 0.0;
        for record in &records {
            println!("{:<12} ${:.2}", record.month, record.cost);
            total += record.cost;
        }
        
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("{:<12} ${:.2}", "Total:", total);
        
        Ok(())
    } else {
        Err(anyhow!("Unexpected event: {:?}", event))
    }
}

/// Execute billing command
pub fn execute_billing_command(name: String) -> Result<()> {
    let config = load_config()?;
    
    let platform = config.platforms.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;
    
    // Validate platform is ready
    validate_platform_ready(platform, "billing")?;
    
    // Check billing configuration
    let project_id = platform.gcp_selected_project_id.as_ref()
        .ok_or_else(|| anyhow!("No project selected for platform '{}'", platform.name))?;
    
    // Use hardcoded billing export settings (could be made configurable later)
    let dataset = "billing_export".to_string();
    let table = "gcp_billing_export".to_string();
    
    println!("Fetching billing data...");
    
    smol::block_on(async {
        let mut runner = PlatformCliRunner::new();
        
        let event = runner.execute_command(PlatformCommand::FetchBilling {
            platform_name: platform.name.clone(),
            project_id: project_id.clone(),
            dataset,
            table,
        }).await?;
        
        if let PlatformEvent::BillingFetched { records, .. } = event {
            println!("\nBilling Summary (Last 3 Months):");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("{:<12} {}", "Month", "Cost (USD)");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            
            let mut total = 0.0;
            for record in &records {
                println!("{:<12} ${:.2}", record.month, record.cost);
                total += record.cost;
            }
            
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("{:<12} ${:.2}", "Total:", total);
            
            Ok(())
        } else {
            Err(anyhow!("Unexpected event: {:?}", event))
        }
    })
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --package mobile --lib cli::commands::platform::tests::billing_tests`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add mobile/src/cli/commands/platform/billing.rs mobile/src/cli/commands/platform/tests.rs
git commit -m "feat(cli): implement billing command with TDD

- Fetch billing data for last 3 months
- Display formatted table with monthly costs
- Calculate total cost
- Test passing (1 test)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 8: Command Router and CLI Integration

**Files:**
- Modify: `mobile/src/cli/mod.rs`
- Modify: `mobile/src/cli/commands/platform/mod.rs`

**Interfaces:**
- Consumes: All command execution functions from previous tasks
- Produces: Complete platform CLI command routing

- [ ] **Step 1: Update PlatformCommands enum in cli/mod.rs**

Modify `mobile/src/cli/mod.rs`, replace the `PlatformCommands` enum:

```rust
#[derive(Subcommand)]
pub enum PlatformCommands {
    /// List all platforms with status (default command)
    List,
    
    /// Show platform details and available actions
    Show { 
        name: String 
    },
    
    /// Refresh platform data
    Refresh { 
        name: String 
    },
    
    /// Add a new VM to the platform
    AddVm {
        name: String,
        #[arg(long)]
        vm_name: Option<String>,
        #[arg(long)]
        zone: Option<String>,
        #[arg(long)]
        machine_type: Option<String>,
    },
    
    /// Update firewall rules (whitelist current IP)
    Firewall {
        name: String,
        #[arg(long)]
        ip: Option<String>,
    },
    
    /// Restart VM
    Restart {
        name: String,
        #[arg(long)]
        vm: Option<String>,
    },
    
    /// Delete VM
    DelVm {
        name: String,
        #[arg(long)]
        vm: Option<String>,
    },
    
    /// Show billing information
    Billing { 
        name: String 
    },
    
    /// Delete platform configuration
    Delete { 
        name: String 
    },
    
    // Legacy commands (deprecated but kept for backwards compatibility)
    /// [Deprecated] List all platforms (use 'list' instead)
    #[command(hide = true)]
    Status,
    
    /// [Deprecated] Add a new platform (use GUI or init)
    #[command(hide = true)]
    Add {
        name: String,
        platform_type: String,
    },
    
    /// [Deprecated] Delete a platform (use 'delete' instead)
    #[command(hide = true)]
    Del {
        name: String,
    },
    
    /// Initialize a platform (OAuth, project setup)
    Init {
        name: String,
    },
}
```

- [ ] **Step 2: Update platform command handler in cli/mod.rs**

In `mobile/src/cli/mod.rs`, update the `Commands::Platform` match arm:

```rust
Commands::Platform { command } => match command {
    PlatformCommands::List => {
        commands::platform::list::execute_platform_list()?;
    }
    PlatformCommands::Show { name } => {
        commands::platform::list::execute_platform_show(name)?;
    }
    PlatformCommands::Refresh { name } => {
        commands::platform::execute_refresh_command(name)?;
    }
    PlatformCommands::AddVm { name, vm_name, zone, machine_type } => {
        commands::platform::vm::execute_addvm_command(name, vm_name, zone, machine_type)?;
    }
    PlatformCommands::Firewall { name, ip } => {
        commands::platform::firewall::execute_firewall_command(name, ip)?;
    }
    PlatformCommands::Restart { name, vm } => {
        commands::platform::vm::execute_restart_command(name, vm)?;
    }
    PlatformCommands::DelVm { name, vm } => {
        commands::platform::vm::execute_delvm_command(name, vm)?;
    }
    PlatformCommands::Billing { name } => {
        commands::platform::billing::execute_billing_command(name)?;
    }
    PlatformCommands::Delete { name } => {
        commands::platform::execute_delete_command(name)?;
    }
    // Legacy commands
    PlatformCommands::Status => {
        eprintln!("Warning: 'status' is deprecated, use 'list' instead");
        commands::platform::list::execute_platform_list()?;
    }
    PlatformCommands::Add { name, platform_type } => {
        eprintln!("Warning: 'add' is deprecated");
        commands::platform::execute_platform_add(name, platform_type)?;
    }
    PlatformCommands::Del { name } => {
        eprintln!("Warning: 'del' is deprecated, use 'delete' instead");
        commands::platform::execute_delete_command(name)?;
    }
    PlatformCommands::Init { name } => {
        commands::platform::execute_platform_init(name)?;
    }
},
```

- [ ] **Step 3: Implement refresh and delete in platform/mod.rs**

Modify `mobile/src/cli/commands/platform/mod.rs`:

```rust
//! Platform command implementation with ViewModel integration

pub mod runner;
pub mod list;
pub mod vm;
pub mod firewall;
pub mod billing;
pub mod helpers;

#[cfg(test)]
mod tests;

// Re-export for CLI router
pub use list::{execute_platform_list, execute_platform_show};
pub use vm::{execute_addvm_command, execute_restart_command, execute_delvm_command};
pub use firewall::execute_firewall_command;
pub use billing::execute_billing_command;

use crate::config::AppConfig;
use crate::cli::commands::platform::runner::PlatformCliRunner;
use crate::viewmodel::platform::PlatformCommand;
use anyhow::{anyhow, Result};
use std::path::PathBuf;

/// Get config file path
fn get_config_path() -> Result<PathBuf> {
    let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
        .ok_or_else(|| anyhow!("Failed to get project directories"))?;
    Ok(proj_dirs.config_dir().join("config.yml"))
}

/// Load application config
fn load_config() -> Result<(AppConfig, PathBuf)> {
    let config_path = get_config_path()?;
    let config = AppConfig::load_or_default(&config_path);
    Ok((config, config_path))
}

/// Execute refresh command
pub fn execute_refresh_command(name: String) -> Result<()> {
    let (config, _) = load_config()?;
    
    let platform = config.platforms.iter()
        .find(|p| p.name == name)
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;
    
    println!("Refreshing platform '{}'...", platform.name);
    
    smol::block_on(async {
        let mut runner = PlatformCliRunner::new();
        
        // Send RefreshAll command
        let _ = runner.execute_command(PlatformCommand::RefreshAll).await?;
        
        println!("✓ Platform data refreshed");
        println!("\nRun 'dure platform {}' to see updated status", name);
        
        Ok(())
    })
}

/// Execute delete platform command
pub fn execute_delete_command(name: String) -> Result<()> {
    let (mut config, config_path) = load_config()?;
    
    let platform_idx = config.platforms.iter()
        .position(|p| p.name == name)
        .ok_or_else(|| anyhow!("Platform '{}' not found", name))?;
    
    let platform = &config.platforms[platform_idx];
    
    println!("⚠️  Delete platform '{}'?", name);
    println!("  Type: {}", platform.platform_type);
    println!("  VMs: {}", platform.vms.len());
    println!("  Project: {}", platform.gcp_selected_project_id.as_deref().unwrap_or("none"));
    print!("Type 'yes' to confirm: ");
    std::io::Write::flush(&mut std::io::stdout())?;
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    
    if input.trim() != "yes" {
        println!("Cancelled");
        return Ok(());
    }
    
    // Remove from config
    config.platforms.remove(platform_idx);
    
    // Save config
    config.save(&config_path)?;
    
    println!("✓ Platform '{}' deleted successfully", name);
    
    Ok(())
}

/// Execute platform add command (legacy)
pub fn execute_platform_add(name: String, platform_type: String) -> Result<()> {
    // Keep legacy implementation for backwards compatibility
    crate::cli::commands::platform_legacy::execute_platform_add(name, platform_type)
}

/// Execute platform init command (legacy)
pub fn execute_platform_init(name: String) -> Result<()> {
    // Keep legacy implementation for backwards compatibility
    crate::cli::commands::platform_legacy::execute_platform_init(name)
}
```

- [ ] **Step 4: Rename old platform.rs to platform_legacy.rs**

```bash
mv mobile/src/cli/commands/platform.rs mobile/src/cli/commands/platform_legacy.rs
```

Update `mobile/src/cli/commands/mod.rs`:

```rust
pub mod platform;
mod platform_legacy;
```

- [ ] **Step 5: Verify compilation**

Run: `cargo check --all-targets`
Expected: Success

- [ ] **Step 6: Test CLI commands manually**

Run: `cargo run --bin dure-desktop -- platform`
Expected: List of platforms displayed

- [ ] **Step 7: Commit**

```bash
git add mobile/src/cli/mod.rs mobile/src/cli/commands/platform/mod.rs mobile/src/cli/commands/platform_legacy.rs mobile/src/cli/commands/mod.rs
git commit -m "feat(cli): integrate platform commands with CLI router

- Update PlatformCommands enum with new commands
- Add command routing in CLI handler
- Implement refresh and delete commands
- Keep legacy commands for backwards compatibility
- All commands integrated and working

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 9: Documentation and Final Testing

**Files:**
- Modify: `docs/QUICK_REFERENCE.md` (if exists)
- Create: Test manual with real GCP account

**Interfaces:**
- None (documentation task)

- [ ] **Step 1: Add platform CLI examples to documentation**

If `docs/QUICK_REFERENCE.md` exists, add section:

```markdown
## Platform CLI Commands

### List Platforms
```bash
dure platform
```

### Show Platform Details
```bash
dure platform my-gcp
```

### Refresh Platform Data
```bash
dure platform my-gcp refresh
```

### Update Firewall (Auto-detect IP)
```bash
dure platform my-gcp firewall
```

### Update Firewall (Explicit IP)
```bash
dure platform my-gcp firewall --ip 203.0.113.42
```

### Add VM (Interactive)
```bash
dure platform my-gcp addvm --vm-name my-vm
```

### Add VM (Full Explicit)
```bash
dure platform my-gcp addvm --vm-name my-vm --zone us-west1-a --machine-type e2-small
```

### Restart VM
```bash
dure platform my-gcp restart
```

### Delete VM
```bash
dure platform my-gcp delvm
```

### Show Billing
```bash
dure platform my-gcp billing
```

### Delete Platform
```bash
dure platform my-gcp delete
```
```

- [ ] **Step 2: Run all tests**

Run: `cargo test --package mobile --lib cli::commands::platform`
Expected: All tests PASS

Count: Should be ~19 tests total (helpers: 9, runner: 3, list: 4, firewall: 2, vm: 3, billing: 1)

- [ ] **Step 3: Manual testing checklist**

Test with real GCP account:
- [ ] `dure platform` lists platforms correctly
- [ ] `dure platform {name}` shows details and actions
- [ ] `dure platform {name} refresh` updates data
- [ ] `dure platform {name} firewall` auto-detects IP and updates
- [ ] `dure platform {name} addvm --vm-name test` creates VM
- [ ] `dure platform {name} restart` restarts VM
- [ ] `dure platform {name} billing` shows billing data
- [ ] `dure platform {name} delvm` deletes VM with confirmation
- [ ] `dure platform {name} delete` deletes platform with confirmation
- [ ] Error messages are helpful (test disconnected platform, no project, etc.)

- [ ] **Step 4: Create manual test report**

Create `docs/superpowers/test-reports/2026-07-05-platform-cli-manual-test.md`:

```markdown
# Platform CLI Manual Test Report

**Date:** 2026-07-05
**Tester:** [Your name]
**Environment:** [Linux/macOS/Windows, GCP account details]

## Test Results

### List Command
- [ ] `dure platform` - PASS/FAIL
- Notes:

### Show Command
- [ ] `dure platform {name}` - PASS/FAIL
- Notes:

### Refresh Command
- [ ] `dure platform {name} refresh` - PASS/FAIL
- Notes:

### Firewall Command
- [ ] `dure platform {name} firewall` - PASS/FAIL
- Notes:

### Add VM Command
- [ ] `dure platform {name} addvm` - PASS/FAIL
- Notes:

### Restart Command
- [ ] `dure platform {name} restart` - PASS/FAIL
- Notes:

### Delete VM Command
- [ ] `dure platform {name} delvm` - PASS/FAIL
- Notes:

### Billing Command
- [ ] `dure platform {name} billing` - PASS/FAIL
- Notes:

### Delete Platform Command
- [ ] `dure platform {name} delete` - PASS/FAIL
- Notes:

### Error Handling
- [ ] Not connected error - PASS/FAIL
- [ ] No project selected error - PASS/FAIL
- [ ] Platform not found error - PASS/FAIL
- Notes:

## Summary
- Total tests: 12
- Passed: X
- Failed: X
- Issues found: [List any bugs or UX issues]
```

- [ ] **Step 5: Final commit**

```bash
git add docs/
git commit -m "docs: add platform CLI documentation and test report

- Add CLI command examples to QUICK_REFERENCE
- Create manual test report template
- All automated tests passing (19 tests)
- Ready for manual testing with real GCP account

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Success Criteria

- [ ] All 19 automated tests passing
- [ ] All commands compile and run without errors
- [ ] CLI output matches design mockups (formatting, colors, etc.)
- [ ] Smart defaults work correctly (IP detection, zone, machine type)
- [ ] Error messages are helpful and actionable
- [ ] Documentation updated with examples
- [ ] Manual testing completed with real GCP account
- [ ] No regressions in existing CLI commands (legacy commands still work)

## Completion

Total tasks: 9
Total steps: ~80
Estimated time: ~10 hours

After completing all tasks, update the design spec status to "Implemented" and close any related issues.
