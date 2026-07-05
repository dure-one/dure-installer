# Platform CLI Redesign - Design Specification

**Date:** 2026-07-05  
**Author:** Claude Code  
**Status:** Design Review  

## Overview

Redesign the `dure platform` CLI commands to match the functionality and user experience of the GUI platform tab (`mobile/src/ui_tabs/platform.rs`). The new CLI will provide a natural, interactive interface for managing cloud platforms (GCP) with smart defaults and helpful error messages.

## Context

### Current State

The current CLI has a minimal command structure:
- `platform status` - List all platforms
- `platform add <name> <type>` - Add platform
- `platform del <name>` - Delete platform  
- `platform init <name>` - Initialize platform (OAuth)

### Problems

1. **Limited functionality** - Missing VM operations, firewall management, billing, etc.
2. **No match with GUI** - Users expect CLI to mirror GUI capabilities
3. **Direct calc layer calls** - Doesn't leverage ViewModel/MVVM architecture
4. **Poor UX** - No progressive disclosure (list → details → action)

### Goals

1. **Feature parity with GUI** - All operations available in GUI should work in CLI
2. **ViewModel integration** - Reuse existing PlatformActor infrastructure
3. **Natural UX** - Progressive disclosure with smart defaults
4. **Actionable errors** - Clear guidance when things fail
5. **TDD implementation** - Write tests first

## Architecture

### Design Principles

**Approach: Minimal Interactive**
- Natural CLI hierarchy: list → details → action
- Smart defaults for common operations (auto-detect IP, default zones)
- Interactive prompts only when needed
- Optional flags to override defaults
- Can add `--non-interactive` mode later if needed

### ViewModel Integration

```
┌─────────────┐
│ CLI Command │
└─────┬───────┘
      │
      ▼
┌─────────────────────┐
│ PlatformCliRunner   │  ← New helper for CLI
│ - Creates ViewModel │
│ - Sends commands    │
│ - Polls for events  │
│ - Formats output    │
└─────┬───────────────┘
      │
      ▼
┌─────────────────────┐
│ ViewModel           │
│ - PlatformActor     │
│ - Event queue       │
└─────┬───────────────┘
      │
      ▼
┌─────────────────────┐
│ PlatformActor       │  ← Existing infrastructure
│ - GCP REST API      │
│ - Config management │
│ - State updates     │
└─────────────────────┘
```

**Flow Pattern:**
```
CLI Command → Create ViewModel → Send PlatformCommand → Wait for Event → Display Result → Exit
```

**Async Runtime:**
- Use `smol::block_on()` to bridge sync CLI with async ViewModel
- Timeout: 60 seconds for operations
- Progress polling: 100ms intervals

## Command Structure

### Command Hierarchy

```
dure platform
  └─ Lists all platforms (name, type, steps summary)

dure platform {name}
  └─ Shows detailed info + available actions menu

dure platform {name} refresh
  └─ Refreshes platform data (re-queries GCP API)

dure platform {name} addvm
  └─ Creates new VM (prompts for name, uses defaults)

dure platform {name} firewall
  └─ Updates firewall (auto-detects IP, adds to whitelist)

dure platform {name} restart
  └─ Restarts VM (auto-selects if one, prompts if multiple)

dure platform {name} delvm
  └─ Deletes VM (shows list if multiple, prompts)

dure platform {name} billing
  └─ Shows billing for last 3 months

dure platform {name} delete
  └─ Deletes platform record (confirms first)
```

### Clap Command Definition

Update `PlatformCommands` enum in `mobile/src/cli/mod.rs`:

```rust
#[derive(Subcommand)]
pub enum PlatformCommands {
    /// List all platforms with status
    #[command(flatten)]
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
}
```

**Note:** Keep existing `Add`, `Del`, `Init` commands for backwards compatibility (mark as deprecated in help text).

## Output Format

### 1. Platform List (`dure platform`)

Displays table matching GUI's data table view with connection steps:

```
Platform Status:
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Name                Type    Steps
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
my-gcp              GCP     ✓ → ✓ → ✓ → ✗ → ✓
another-platform    GCP     ✓ → ✓ → ✗ → ✗ → ✗
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Steps: Connected → Project → VM → Firewall → SSH

Total platforms: 2

Use 'dure platform <name>' to see details and available actions.
```

**Steps Calculation:**
- ✓ Connected: `gcp_oauth_access_token.is_some()`
- ✓ Project: `gcp_selected_project_id.is_some()`
- ✓ VM: `vms.len() > 0`
- ✓ Firewall: Current IP is whitelisted (query GCP API)
- ✓ SSH: VM has `external_ip.is_some()`

### 2. Platform Details (`dure platform {name}`)

Displays detailed info (drawer content) and available actions:

```
Platform: my-gcp
Type: GCP
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Connection Steps:
  ✓ GCP Connected
  ✓ Project Created
  ✓ VM Created
  ✗ Firewall Rules Updated
  ✓ SSH Connected

Details:
  Email: user@example.com (15 projects total)
  └─ Project: my-project-123 (selected)
     └─ VM: my-vm-instance (203.0.113.42)
        • Firewall: ✗ Not whitelisted
        • SSH: ✓ Ready

Available Actions:
  refresh   - Refresh platform data
  addvm     - Add a new VM (disabled: VM already exists)
  firewall  - Update firewall rules
  restart   - Restart VM
  delvm     - Delete VM
  billing   - Show billing information
  delete    - Delete platform

Run: dure platform my-gcp <action>
```

### 3. Action Results

Concise confirmation with relevant details:

**Firewall:**
```
$ dure platform my-gcp firewall
✓ Detected current IP: 203.0.113.42
✓ Updated firewall rules for project 'my-project-123'
✓ Whitelisted IP: 203.0.113.42
```

**Billing:**
```
$ dure platform my-gcp billing
Billing Summary (Last 3 Months):
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Month        Cost (USD)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
2026-07      $12.45
2026-06      $11.89
2026-05      $13.20
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total:       $37.54
```

### 4. Progress Display (Long Operations)

For VM creation, deletion, restart:

```
$ dure platform my-gcp addvm
VM Name (default: dure-vm-1): my-new-vm
Zone (default: us-central1-a): 
Machine Type (default: e2-micro): 

Creating VM...
[25%] Validating configuration...
[50%] Creating VM instance...
[75%] Waiting for external IP...
[100%] VM ready

✓ VM created successfully
  Name: my-new-vm
  Zone: us-central1-a
  External IP: 203.0.113.50
```

## Smart Defaults and Interactive Prompts

### 1. Firewall Command

**Smart Behavior:**
- Auto-detect current IP via `https://api.ipify.org`
- Fallback to prompt if detection fails
- Flag override: `--ip <address>` to specify manually

**Implementation:**
```rust
async fn get_current_ip(ip_flag: Option<String>) -> Result<String> {
    if let Some(ip) = ip_flag {
        return Ok(ip);
    }
    
    // Try ipify API
    match ureq::get("https://api.ipify.org").call() {
        Ok(resp) => Ok(resp.into_string()?),
        Err(_) => {
            // Prompt user
            print!("Enter IP address to whitelist: ");
            io::stdout().flush()?;
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            Ok(input.trim().to_string())
        }
    }
}
```

### 2. Add VM Command

**Defaults:**
- **VM Name**: Prompt (no default) OR use flag `--vm-name`
- **Zone**: `us-central1-a` (most common) OR use flag `--zone`
- **Machine Type**: `e2-micro` (free tier) OR use flag `--machine-type`

**Implementation:**
```rust
let vm_name = match vm_name_flag {
    Some(name) => name,
    None => {
        print!("VM Name: ");
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        input.trim().to_string()
    }
};

let zone = zone_flag.unwrap_or_else(|| "us-central1-a".to_string());
let machine_type = machine_type_flag.unwrap_or_else(|| "e2-micro".to_string());
```

### 3. Restart/Delete VM

**Smart Selection:**
- **Single VM**: Auto-select (no prompt)
- **Multiple VMs**: Show numbered list, prompt for selection
- **Flag override**: `--vm <name>` to specify directly

**Implementation:**
```rust
async fn select_vm(
    platform: &CloudPlatformConfig, 
    vm_flag: Option<String>
) -> Result<(String, String)> {
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
            println!("Select VM:");
            for (idx, vm) in platform.vms.iter().enumerate() {
                println!("  {}. {} ({})", idx + 1, vm.name, vm.zone);
            }
            print!("Enter number: ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let idx: usize = input.trim().parse()? - 1;
            
            let vm = platform.vms.get(idx)
                .ok_or_else(|| anyhow!("Invalid selection"))?;
            Ok((vm.name.clone(), vm.zone.clone()))
        }
    }
}
```

### 4. Billing Command

**Defaults:**
- Use platform's configured `billing_dataset` and `billing_table` from config
- Time range: Last 3 months (hardcoded)
- If not configured: Display helpful error with setup instructions

**Implementation:**
```rust
let dataset = platform.billing_dataset.as_ref()
    .ok_or_else(|| anyhow!(
        "Billing not configured for platform '{}'\n\
         Set up billing export: https://cloud.google.com/billing/docs/how-to/export-data-bigquery",
        platform.name
    ))?;

let table = platform.billing_table.as_ref()
    .ok_or_else(|| anyhow!("Billing table not configured"))?;

// Query last 3 months
let end_date = chrono::Utc::now();
let start_date = end_date - chrono::Duration::days(90);
```

### 5. Delete Platform

**Confirmation Required:**
- Always show platform details (type, VMs, project)
- Require explicit "yes" response (not just "y")
- No default acceptance

**Implementation:**
```rust
println!("⚠️  Delete platform '{}'?", platform_name);
println!("  Type: {}", platform.platform_type);
println!("  VMs: {}", platform.vms.len());
println!("  Project: {}", platform.gcp_selected_project_id.as_deref().unwrap_or("none"));
print!("Type 'yes' to confirm: ");
io::stdout().flush()?;

let mut input = String::new();
io::stdin().read_line(&mut input)?;
if input.trim() != "yes" {
    println!("Cancelled");
    return Ok(());
}
```

## Error Handling

### Error Display Strategy

All errors display with:
1. Clear error message
2. Contextual explanation
3. Actionable next steps

**Format:**
```rust
match result {
    Err(e) => {
        eprintln!("❌ Error: {}", e);
        
        // Add context-specific hints
        if e.to_string().contains("OAuth") {
            eprintln!("\nRun 'dure platform init {}' to reconnect", platform_name);
        } else if e.to_string().contains("quota") {
            eprintln!("\nCheck GCP quotas: https://console.cloud.google.com/iam-admin/quotas");
        }
        
        std::process::exit(1);
    }
    Ok(result) => { /* ... */ }
}
```

### Edge Cases

**1. Platform Not Found**
```
❌ Platform 'my-gcp' not found

Available platforms:
  • another-gcp
  • test-platform

Run 'dure platform' to list all platforms.
```

**2. No OAuth Token (Not Connected)**
```
❌ Platform 'my-gcp' is not connected

Run 'dure platform init my-gcp' to authenticate with Google Cloud.
```

**3. No Project Selected**
```
❌ No project selected for platform 'my-gcp'

Available actions require a GCP project to be selected.
This is typically done during 'dure platform init', but you can reconnect.
```

**4. Firewall Update When Already Whitelisted**
```
✓ IP 203.0.113.42 is already whitelisted in project 'my-project-123'
  No changes needed.
```

**5. Delete VM When No VMs Exist**
```
❌ No VMs found for platform 'my-gcp'

Run 'dure platform my-gcp addvm' to create a VM.
```

**6. Billing Not Configured**
```
❌ Billing export not configured for platform 'my-gcp'

To view billing data, you must export billing to BigQuery:
  1. Visit: https://console.cloud.google.com/billing/export
  2. Enable BigQuery export
  3. Update platform config with dataset and table names

Or run 'dure platform init my-gcp' to reconfigure.
```

**7. Operation Timeout**
```
❌ Operation timed out after 60 seconds

The operation may still be running in the background.
Check GCP Console: https://console.cloud.google.com/compute/instances

If the issue persists, try:
  • Check your network connection
  • Verify GCP API quotas
  • Run 'dure platform my-gcp refresh' to update status
```

**8. VM Already Exists (addvm)**
```
❌ Platform 'my-gcp' already has a VM: my-vm-instance

To create a new VM, first delete the existing one:
  dure platform my-gcp delvm

Note: This platform configuration is designed for single-VM deployments.
For multiple VMs, use the GCP Console or gcloud CLI.
```

### Validation Function

Pre-flight checks before sending commands:

```rust
fn validate_platform_ready(
    platform: &CloudPlatformConfig, 
    operation: &str
) -> Result<()> {
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

## Testing Strategy (TDD)

### Test Structure

Create `mobile/src/cli/commands/platform/tests.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // Unit tests for helper functions
    mod helpers {
        #[test]
        fn test_format_steps() { ... }
        
        #[test]
        fn test_format_drawer_content() { ... }
        
        #[test]
        fn test_validate_platform_ready() { ... }
        
        #[test]
        fn test_select_vm_single() { ... }
        
        #[test]
        fn test_select_vm_multiple() { ... }
    }
    
    // Integration tests with mock ViewModel
    mod integration {
        #[test]
        fn test_platform_list_empty() { ... }
        
        #[test]
        fn test_platform_list_with_platforms() { ... }
        
        #[test]
        fn test_platform_show_details() { ... }
        
        #[smol::test]
        async fn test_firewall_auto_detect_ip() { ... }
        
        #[smol::test]
        async fn test_addvm_with_defaults() { ... }
        
        #[smol::test]
        async fn test_delvm_single_vm() { ... }
        
        #[smol::test]
        async fn test_restart_vm() { ... }
        
        #[smol::test]
        async fn test_billing_display() { ... }
        
        #[test]
        fn test_delete_platform_confirmation() { ... }
    }
    
    // Error case tests
    mod errors {
        #[test]
        fn test_platform_not_found() { ... }
        
        #[test]
        fn test_no_oauth_token() { ... }
        
        #[test]
        fn test_no_project_selected() { ... }
        
        #[test]
        fn test_billing_not_configured() { ... }
        
        #[smol::test]
        async fn test_operation_timeout() { ... }
        
        #[test]
        fn test_vm_already_exists() { ... }
    }
}
```

### Test Fixtures

Mock platform configs for testing:

```rust
fn mock_platform_connected() -> CloudPlatformConfig {
    CloudPlatformConfig {
        name: "test-gcp".to_string(),
        platform_type: "gcp".to_string(),
        gcp_oauth_access_token: Some("mock_token".to_string()),
        gcp_oauth_refresh_token: Some("mock_refresh".to_string()),
        gcp_oauth_token_expiry: Some(chrono::Utc::now() + chrono::Duration::hours(1)),
        gcp_connected_email: Some("test@example.com".to_string()),
        gcp_selected_project_id: Some("test-project-123".to_string()),
        vms: vec![
            VmConfig {
                name: "test-vm".to_string(),
                zone: "us-central1-a".to_string(),
                machine_type: "e2-micro".to_string(),
                external_ip: Some("203.0.113.42".to_string()),
            }
        ],
        ..Default::default()
    }
}

fn mock_platform_no_vm() -> CloudPlatformConfig {
    let mut platform = mock_platform_connected();
    platform.vms.clear();
    platform
}

fn mock_platform_disconnected() -> CloudPlatformConfig {
    CloudPlatformConfig {
        name: "test-gcp".to_string(),
        platform_type: "gcp".to_string(),
        gcp_oauth_access_token: None,
        ..Default::default()
    }
}
```

### Mock ViewModel Runner

Test helper that simulates ViewModel responses:

```rust
#[cfg(test)]
struct MockPlatformRunner {
    responses: VecDeque<PlatformEvent>,
}

#[cfg(test)]
impl MockPlatformRunner {
    fn new() -> Self {
        Self {
            responses: VecDeque::new(),
        }
    }
    
    fn expect_command(&mut self, response: PlatformEvent) {
        self.responses.push_back(response);
    }
    
    async fn execute_command(
        &mut self, 
        _cmd: PlatformCommand
    ) -> Result<PlatformEvent> {
        self.responses.pop_front()
            .ok_or_else(|| anyhow!("No more responses"))
    }
}
```

### Example Test Cases

```rust
#[test]
fn test_format_steps_all_complete() {
    let platform = mock_platform_connected();
    let steps = format_steps(&platform);
    
    assert!(steps.contains("✓"));
    assert!(steps.contains("→"));
}

#[test]
fn test_validate_platform_not_connected() {
    let platform = mock_platform_disconnected();
    let result = validate_platform_ready(&platform, "addvm");
    
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not connected"));
    assert!(err.contains("dure platform init"));
}

#[smol::test]
async fn test_firewall_update_success() {
    let mut runner = MockPlatformRunner::new();
    runner.expect_command(
        PlatformEvent::FirewallUpdated {
            platform_name: "test-gcp".to_string(),
            whitelisted_ip: "203.0.113.42".to_string(),
        }
    );
    
    let result = execute_firewall_command(
        &mut runner, 
        "test-gcp", 
        Some("203.0.113.42".to_string())
    ).await;
    
    assert!(result.is_ok());
}

#[test]
fn test_select_vm_single() {
    let platform = mock_platform_connected();
    let result = select_vm_sync(&platform, None);
    
    assert!(result.is_ok());
    let (name, zone) = result.unwrap();
    assert_eq!(name, "test-vm");
    assert_eq!(zone, "us-central1-a");
}
```

### TDD Implementation Order

1. **Write helper tests** → Implement helpers (format, validation)
2. **Write list command test** → Implement list command
3. **Write show command test** → Implement show command
4. **Write refresh test** → Implement refresh command
5. **Write firewall test** → Implement firewall command
6. **Write addvm test** → Implement addvm command
7. **Write restart test** → Implement restart command
8. **Write delvm test** → Implement delvm command
9. **Write billing test** → Implement billing command
10. **Write delete test** → Implement delete command
11. **Write error tests** → Implement error handling
12. **Manual testing** with real GCP account

### CI Integration

Tests run automatically in existing CI pipeline:

```yaml
- name: Run tests
  run: cargo test --workspace
```

## Implementation Plan

### File Structure

```
mobile/src/cli/
├── mod.rs                        # Update PlatformCommands enum
├── commands/
│   ├── mod.rs                    # Re-export platform module
│   └── platform/
│       ├── mod.rs                # Main command router
│       ├── runner.rs             # PlatformCliRunner (ViewModel wrapper)
│       ├── list.rs               # List and show commands
│       ├── vm.rs                 # VM operations (add, delete, restart)
│       ├── firewall.rs           # Firewall command
│       ├── billing.rs            # Billing command
│       ├── helpers.rs            # Format and validation helpers
│       └── tests.rs              # Test suite
```

### Implementation Steps (TDD)

1. **Setup infrastructure** (30 min)
   - Create file structure
   - Set up PlatformCliRunner skeleton
   - Add test fixtures and mock runner

2. **Helpers (TDD)** (45 min)
   - Write tests for `format_steps`, `format_drawer_content`, `validate_platform_ready`
   - Implement helpers until tests pass

3. **List/Show commands (TDD)** (1 hour)
   - Write tests for platform list and show
   - Implement list and show commands
   - Test with mock config

4. **Refresh command (TDD)** (30 min)
   - Write tests for refresh
   - Implement refresh (sends RefreshAll command)
   - Display updated status

5. **Firewall command (TDD)** (45 min)
   - Write tests for IP auto-detection and firewall update
   - Implement firewall command with smart IP detection
   - Test success and error cases

6. **Add VM command (TDD)** (1 hour)
   - Write tests for VM creation with defaults
   - Implement addvm with prompts and defaults
   - Test progress display

7. **Restart VM command (TDD)** (45 min)
   - Write tests for VM restart (single and multiple)
   - Implement restart with VM selection
   - Test progress display

8. **Delete VM command (TDD)** (45 min)
   - Write tests for VM deletion (single and multiple)
   - Implement delvm with VM selection
   - Test confirmation

9. **Billing command (TDD)** (1 hour)
   - Write tests for billing display
   - Implement billing with 3-month query
   - Test formatting

10. **Delete platform command (TDD)** (45 min)
    - Write tests for platform deletion with confirmation
    - Implement delete with explicit confirmation
    - Test cancellation

11. **Error handling (TDD)** (1 hour)
    - Write tests for all error cases
    - Implement error display with helpful messages
    - Test edge cases

12. **Integration testing** (1 hour)
    - Manual testing with real GCP account
    - Test all commands end-to-end
    - Verify output formatting

13. **Documentation** (30 min)
    - Update CLI help text
    - Add examples to QUICK_REFERENCE.md
    - Update CLAUDE.md if needed

**Total estimated time: ~10 hours**

## Success Criteria

- [ ] All commands work as specified
- [ ] All tests pass (unit + integration)
- [ ] CLI output matches design mockups
- [ ] Smart defaults work correctly
- [ ] Error messages are helpful and actionable
- [ ] Progress display works for long operations
- [ ] No regression in existing CLI commands
- [ ] Manual testing with real GCP account successful
- [ ] CI tests pass

## Future Enhancements (Not in Scope)

- `--non-interactive` flag for scripting
- JSON output format (`--json`)
- Support for multiple VMs per platform
- Firebase and Supabase platform types
- VM regeneration command (currently commented out in GUI)
- Batch operations (e.g., update firewall for all platforms)

## References

- **GUI Implementation**: `mobile/src/ui_tabs/platform.rs`
- **ViewModel Commands**: `mobile/src/viewmodel/platform/commands.rs`
- **ViewModel Events**: `mobile/src/viewmodel/platform/events.rs`
- **Platform Actor**: `mobile/src/viewmodel/platform/actor.rs`
- **Current CLI**: `mobile/src/cli/commands/platform.rs`
