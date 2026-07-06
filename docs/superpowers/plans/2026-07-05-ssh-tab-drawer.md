# SSH Tab Drawer Redesign Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reimplement SSH tab with expandable drawer table showing Linux system status, matching Platform tab UX with MVVM architecture.

**Architecture:** Clean rewrite replacing MaterialSpreadsheet with data_table + drawer. ViewModel/Actor pattern for async service operations. Platform relationship links SSH hosts to cloud platforms.

**Tech Stack:** Rust, egui, egui-material3 data_table, smol async runtime, russh, existing ViewModel/Actor pattern

## Global Constraints

- Rust nightly toolchain required
- Follow existing MVVM pattern (UI → ViewModel → Actor → calc)
- Config format must be backward compatible
- No new dependencies (use existing egui-material3, smol, russh)
- All SSH operations async via actor threads
- Phase 1 scope: Linux status working, ansible/docker/dure-wss as placeholders
- Maintain existing SSH operations (add/delete/test/init)

---

## Task 1: Add Platform Relationship to Config

**Files:**
- Modify: `mobile/src/config.rs:13-23` (SshHostConfig struct)
- Test: Manual verification (config loading)

**Interfaces:**
- Produces: `SshHostConfig.platform_name: Option<String>`

- [ ] **Step 1: Add platform_name field to SshHostConfig**

Open `mobile/src/config.rs` and locate `SshHostConfig` struct (around line 13-23). Add the `platform_name` field:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshHostConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub initialized: bool,
    
    // Platform relationship
    #[serde(default)]
    pub platform_name: Option<String>,
}
```

- [ ] **Step 2: Verify backward compatibility**

Run: `cargo +nightly build --bin dure-desktop`
Expected: Compiles successfully. Old configs without `platform_name` will default to `None`.

- [ ] **Step 3: Test config loading**

Create test config at `/tmp/test-ssh-config.yml`:

```yaml
ssh_hosts:
  - host: "root@test.com"
    port: 22
    initialized: false
    # platform_name omitted - should default to None
```

Run desktop app and verify it loads without error.

- [ ] **Step 4: Commit**

```bash
git add mobile/src/config.rs
git commit -m "feat(config): add platform_name to SshHostConfig

Add optional platform_name field to link SSH hosts with Platform tab entries.
Uses #[serde(default)] for backward compatibility with existing configs.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Add Data Models for SSH Tab

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:1-61` (add models before SshTab struct)

**Interfaces:**
- Produces: `SshRowData`, `LinuxStatus`, `ConnectionStatus` structs

- [ ] **Step 1: Add LinuxStatus struct**

Open `mobile/src/ui_tabs/ssh.rs`. After the imports (line ~10), before `SshTab` struct, add:

```rust
/// Linux system status information
#[derive(Clone, Debug, Default)]
struct LinuxStatus {
    uptime: String,
    external_ip: String,
    load_average: String,
    memory_usage: String,
    disk_usage: String,
    top_processes: Vec<String>,
}
```

- [ ] **Step 2: Add ConnectionStatus enum**

```rust
/// SSH connection state
#[derive(Clone, Debug, PartialEq)]
enum ConnectionStatus {
    Connected,
    Offline,
    Testing,
    Unknown,
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        ConnectionStatus::Unknown
    }
}
```

- [ ] **Step 3: Add SshRowData struct**

```rust
/// Display data for SSH table row + drawer
#[derive(Clone, Debug, Default)]
struct SshRowData {
    // Identity
    host: String,
    port: u16,
    
    // Platform relationship
    platform_name: Option<String>,
    platform_type: Option<String>,
    
    // Service status flags
    linux_detected: bool,
    linux_os: Option<String>,
    ansible_enabled: bool,
    docker_enabled: bool,
    dure_wss_enabled: bool,
    
    // Drawer content
    linux_status: Option<LinuxStatus>,
    
    // Connection state
    connection_status: ConnectionStatus,
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo +nightly check --bin dure-desktop`
Expected: Compiles successfully (new structs are unused but valid)

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh): add data models for drawer table

Add SshRowData, LinuxStatus, ConnectionStatus models for new table structure.
These will replace MaterialSpreadsheet rows.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Add Calc Layer - Linux Status Functions

**Files:**
- Modify: `mobile/src/calc/ssh.rs:1-end` (add new functions)
- Test: Manual testing (requires live SSH host)

**Interfaces:**
- Consumes: `SshHostConfig` from config.rs
- Produces: `get_linux_status(config) -> Result<LinuxStatus>`, `detect_os(config) -> Result<String>`

- [ ] **Step 1: Add imports at top of ssh.rs**

Open `mobile/src/calc/ssh.rs`. Add these imports near the top (after existing imports):

```rust
use crate::config::SshHostConfig;
```

- [ ] **Step 2: Add LinuxStatus struct (duplicate for calc layer)**

Add after imports:

```rust
/// Linux system status (calc layer version)
#[derive(Clone, Debug)]
pub struct LinuxStatus {
    pub uptime: String,
    pub external_ip: String,
    pub load_average: String,
    pub memory_usage: String,
    pub disk_usage: String,
    pub top_processes: Vec<String>,
}
```

- [ ] **Step 3: Add detect_os function**

```rust
/// Detect OS distribution via SSH
pub fn detect_os(host_config: &SshHostConfig) -> Result<String, String> {
    let session = establish_connection(host_config)?;
    
    // Try /etc/os-release first (modern standard)
    if let Ok(output) = execute_ssh_command(&session, "cat /etc/os-release | grep '^ID=' | cut -d= -f2 | tr -d '\"'") {
        let os = output.trim().to_string();
        if !os.is_empty() {
            return Ok(os);
        }
    }
    
    // Fallback to uname
    if let Ok(output) = execute_ssh_command(&session, "uname -s") {
        let os = output.trim().to_lowercase();
        if !os.is_empty() {
            return Ok(os);
        }
    }
    
    Ok("unknown".to_string())
}
```

- [ ] **Step 4: Add get_linux_status function**

```rust
/// Get comprehensive Linux system status via SSH
pub fn get_linux_status(host_config: &SshHostConfig) -> Result<LinuxStatus, String> {
    let session = establish_connection(host_config)?;
    
    // Execute multiple commands - use unwrap_or for resilience
    let uptime = execute_ssh_command(&session, "uptime -p")
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();
    
    let external_ip = execute_ssh_command(&session, "curl -s ifconfig.me")
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();
    
    let load = execute_ssh_command(&session, "cat /proc/loadavg | awk '{print $1, $2, $3}'")
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();
    
    let memory = execute_ssh_command(&session, "free -h | grep Mem | awk '{print $3 \" / \" $2}'")
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();
    
    let disk = execute_ssh_command(&session, "df -h / | tail -1 | awk '{print $3 \" / \" $2 \" (\" $5 \")\"}}'")
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();
    
    let processes_output = execute_ssh_command(&session, "ps aux --sort=-%mem | head -6 | tail -5 | awk '{print $11}'")
        .unwrap_or_else(|_| "".to_string());
    
    let top_processes: Vec<String> = processes_output
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    Ok(LinuxStatus {
        uptime,
        external_ip,
        load_average: load,
        memory_usage: memory,
        disk_usage: disk,
        top_processes,
    })
}
```

- [ ] **Step 5: Add Docker status check functions**

```rust
/// Check if Docker is installed via SSH
pub fn check_docker_installed(host_config: &SshHostConfig) -> Result<bool, String> {
    let session = establish_connection(host_config)?;
    let result = execute_ssh_command(&session, "command -v docker");
    Ok(result.is_ok() && !result.unwrap().trim().is_empty())
}

/// Check if Docker daemon is running via SSH
pub fn check_docker_running(host_config: &SshHostConfig) -> Result<bool, String> {
    let session = establish_connection(host_config)?;
    let result = execute_ssh_command(&session, "systemctl is-active docker");
    Ok(result.is_ok() && result.unwrap().trim() == "active")
}
```

- [ ] **Step 6: Add Docker install/uninstall functions**

```rust
/// Install Docker via convenience script
pub fn install_docker(host_config: &SshHostConfig) -> Result<(), String> {
    let session = establish_connection(host_config)?;
    
    // Download and execute Docker install script
    execute_ssh_command(&session, "curl -fsSL https://get.docker.com | sh")?;
    
    // Enable and start Docker service
    execute_ssh_command(&session, "systemctl enable docker")?;
    execute_ssh_command(&session, "systemctl start docker")?;
    
    Ok(())
}

/// Uninstall Docker
pub fn uninstall_docker(host_config: &SshHostConfig) -> Result<(), String> {
    let session = establish_connection(host_config)?;
    
    // Stop and disable service
    let _ = execute_ssh_command(&session, "systemctl stop docker");
    let _ = execute_ssh_command(&session, "systemctl disable docker");
    
    // Remove packages (Debian/Ubuntu)
    execute_ssh_command(&session, 
        "apt-get remove -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin"
    )?;
    
    Ok(())
}
```

- [ ] **Step 7: Add placeholder functions for ansible and dure-wss**

```rust
/// Check if Ansible is installed
pub fn check_ansible_installed(host_config: &SshHostConfig) -> Result<bool, String> {
    let session = establish_connection(host_config)?;
    let result = execute_ssh_command(&session, "command -v ansible");
    Ok(result.is_ok() && !result.unwrap().trim().is_empty())
}

/// Install Ansible (placeholder)
pub fn install_ansible(_host_config: &SshHostConfig) -> Result<(), String> {
    Err("Ansible installation not yet implemented".to_string())
}

/// Uninstall Ansible (placeholder)
pub fn uninstall_ansible(_host_config: &SshHostConfig) -> Result<(), String> {
    Err("Ansible uninstallation not yet implemented".to_string())
}

/// Check if Dure-WSS is installed
pub fn check_dure_wss_installed(host_config: &SshHostConfig) -> Result<bool, String> {
    let session = establish_connection(host_config)?;
    let result = execute_ssh_command(&session, "command -v dure");
    Ok(result.is_ok() && !result.unwrap().trim().is_empty())
}

/// Install Dure-WSS (placeholder)
pub fn install_dure_wss(_host_config: &SshHostConfig) -> Result<(), String> {
    Err("Dure-WSS installation not yet implemented".to_string())
}

/// Uninstall Dure-WSS (placeholder)
pub fn uninstall_dure_wss(_host_config: &SshHostConfig) -> Result<(), String> {
    Err("Dure-WSS uninstallation not yet implemented".to_string())
}
```

- [ ] **Step 8: Verify compilation**

Run: `cargo +nightly check --bin dure-desktop`
Expected: Compiles successfully

- [ ] **Step 9: Commit**

```bash
git add mobile/src/calc/ssh.rs
git commit -m "feat(calc): add Linux status and service check functions

Add SSH-based functions to query Linux system status (uptime, IP, memory, 
disk, load, processes). Add Docker install/check/uninstall. Add placeholders
for ansible and dure-wss.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Add SSH Actor Commands and Events

**Files:**
- Modify: `mobile/src/viewmodel/ssh/actor.rs:1-end` (add commands)
- Modify: `mobile/src/viewmodel/ssh/mod.rs:1-end` (add events)

**Interfaces:**
- Produces: `SshCommand` variants, `SshEvent` variants for service operations

- [ ] **Step 1: Add new SshCommand variants**

Open `mobile/src/viewmodel/ssh/actor.rs`. Locate the `SshCommand` enum and add new variants:

```rust
pub enum SshCommand {
    // Existing commands...
    AddHost { name: String, host: String, port: u16, user: String, key_path: String },
    DeleteHost { name: String },
    TestConnection { name: String },
    InitHost { name: String },
    
    // NEW: Service management
    GetLinuxStatus { name: String },
    
    InstallDocker { name: String },
    GetDockerStatus { name: String },
    UninstallDocker { name: String },
    
    InstallAnsible { name: String },
    GetAnsibleStatus { name: String },
    UninstallAnsible { name: String },
    
    InstallDureWss { name: String },
    GetDureWssStatus { name: String },
    UninstallDureWss { name: String },
}
```

- [ ] **Step 2: Add new SshEvent variants**

Open `mobile/src/viewmodel/ssh/mod.rs`. Locate the `SshEvent` enum and add:

```rust
#[derive(Debug, Clone)]
pub enum SshEvent {
    // Existing events...
    HostAdded { name: String },
    HostDeleted { name: String },
    ConnectionTested { name: String, success: bool, latency_ms: Option<u64> },
    HostInitialized { name: String, success: bool },
    
    // NEW: Service events
    LinuxStatusRetrieved {
        name: String,
        uptime: String,
        external_ip: String,
        load_average: String,
        memory_usage: String,
        disk_usage: String,
        top_processes: Vec<String>,
    },
    
    DockerInstalled { name: String },
    DockerStatusRetrieved {
        name: String,
        installed: bool,
        running: bool,
    },
    DockerUninstalled { name: String },
    
    AnsibleInstalled { name: String },
    AnsibleStatusRetrieved {
        name: String,
        installed: bool,
    },
    AnsibleUninstalled { name: String },
    
    DureWssInstalled { name: String },
    DureWssStatusRetrieved {
        name: String,
        installed: bool,
    },
    DureWssUninstalled { name: String },
    
    ServiceError {
        name: String,
        service: String,
        operation: String,
        error: String,
    },
    
    Error { operation: String, error: String },
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo +nightly check --bin dure-desktop`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add mobile/src/viewmodel/ssh/actor.rs mobile/src/viewmodel/ssh/mod.rs
git commit -m "feat(viewmodel): add SSH service management commands and events

Add SshCommand and SshEvent variants for Linux status, Docker, Ansible, 
and Dure-WSS service operations.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 5: Implement Actor Command Handlers

**Files:**
- Modify: `mobile/src/viewmodel/ssh/actor.rs:handle_command()` (add handlers)

**Interfaces:**
- Consumes: `SshCommand` variants from Task 4
- Consumes: calc layer functions from Task 3
- Produces: `SshEvent` emissions

- [ ] **Step 1: Add helper function to load SSH host config**

In `mobile/src/viewmodel/ssh/actor.rs`, add this helper function:

```rust
/// Load SSH host config from app config
fn load_ssh_host_config(name: &str) -> Result<crate::config::SshHostConfig, String> {
    let proj_dirs = directories::ProjectDirs::from("pe", "nikescar", "dure")
        .ok_or_else(|| "Failed to get project directories".to_string())?;
    let config_path = proj_dirs.config_dir().join("config.yml");
    
    let app_config = crate::config::AppConfig::load_or_default(&config_path);
    
    app_config
        .ssh_hosts
        .into_iter()
        .find(|h| h.host == name)
        .ok_or_else(|| format!("SSH host '{}' not found in config", name))
}
```

- [ ] **Step 2: Add GetLinuxStatus handler**

In the `handle_command()` function (or wherever commands are matched), add:

```rust
SshCommand::GetLinuxStatus { name } => {
    let host_config = match load_ssh_host_config(&name) {
        Ok(cfg) => cfg,
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "linux".into(),
                operation: "get_status".into(),
                error: format!("Failed to load config: {}", e),
            }).await;
            return;
        }
    };
    
    // Execute in blocking thread
    let status_result = crate::viewmodel::runtime::unblock(move || {
        crate::calc::ssh::get_linux_status(&host_config)
    }).await;
    
    match status_result {
        Ok(status) => {
            let _ = tx.send(SshEvent::LinuxStatusRetrieved {
                name,
                uptime: status.uptime,
                external_ip: status.external_ip,
                load_average: status.load_average,
                memory_usage: status.memory_usage,
                disk_usage: status.disk_usage,
                top_processes: status.top_processes,
            }).await;
        }
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "linux".into(),
                operation: "get_status".into(),
                error: e,
            }).await;
        }
    }
}
```

- [ ] **Step 3: Add InstallDocker handler**

```rust
SshCommand::InstallDocker { name } => {
    let host_config = match load_ssh_host_config(&name) {
        Ok(cfg) => cfg,
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "docker".into(),
                operation: "install".into(),
                error: format!("Failed to load config: {}", e),
            }).await;
            return;
        }
    };
    
    let install_result = crate::viewmodel::runtime::unblock(move || {
        crate::calc::ssh::install_docker(&host_config)
    }).await;
    
    match install_result {
        Ok(_) => {
            let _ = tx.send(SshEvent::DockerInstalled { name }).await;
        }
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "docker".into(),
                operation: "install".into(),
                error: e,
            }).await;
        }
    }
}
```

- [ ] **Step 4: Add GetDockerStatus handler**

```rust
SshCommand::GetDockerStatus { name } => {
    let host_config = match load_ssh_host_config(&name) {
        Ok(cfg) => cfg,
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "docker".into(),
                operation: "get_status".into(),
                error: format!("Failed to load config: {}", e),
            }).await;
            return;
        }
    };
    
    let status_result = crate::viewmodel::runtime::unblock(move || {
        let installed = crate::calc::ssh::check_docker_installed(&host_config)?;
        let running = if installed {
            crate::calc::ssh::check_docker_running(&host_config)?
        } else {
            false
        };
        Ok((installed, running))
    }).await;
    
    match status_result {
        Ok((installed, running)) => {
            let _ = tx.send(SshEvent::DockerStatusRetrieved {
                name,
                installed,
                running,
            }).await;
        }
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "docker".into(),
                operation: "get_status".into(),
                error: e,
            }).await;
        }
    }
}
```

- [ ] **Step 5: Add UninstallDocker handler**

```rust
SshCommand::UninstallDocker { name } => {
    let host_config = match load_ssh_host_config(&name) {
        Ok(cfg) => cfg,
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "docker".into(),
                operation: "uninstall".into(),
                error: format!("Failed to load config: {}", e),
            }).await;
            return;
        }
    };
    
    let uninstall_result = crate::viewmodel::runtime::unblock(move || {
        crate::calc::ssh::uninstall_docker(&host_config)
    }).await;
    
    match uninstall_result {
        Ok(_) => {
            let _ = tx.send(SshEvent::DockerUninstalled { name }).await;
        }
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "docker".into(),
                operation: "uninstall".into(),
                error: e,
            }).await;
        }
    }
}
```

- [ ] **Step 6: Add Ansible handlers (call placeholders)**

```rust
SshCommand::InstallAnsible { name } => {
    let host_config = match load_ssh_host_config(&name) {
        Ok(cfg) => cfg,
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "ansible".into(),
                operation: "install".into(),
                error: format!("Failed to load config: {}", e),
            }).await;
            return;
        }
    };
    
    let install_result = crate::viewmodel::runtime::unblock(move || {
        crate::calc::ssh::install_ansible(&host_config)
    }).await;
    
    match install_result {
        Ok(_) => {
            let _ = tx.send(SshEvent::AnsibleInstalled { name }).await;
        }
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "ansible".into(),
                operation: "install".into(),
                error: e,
            }).await;
        }
    }
}

SshCommand::GetAnsibleStatus { name } => {
    let host_config = match load_ssh_host_config(&name) {
        Ok(cfg) => cfg,
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "ansible".into(),
                operation: "get_status".into(),
                error: format!("Failed to load config: {}", e),
            }).await;
            return;
        }
    };
    
    let status_result = crate::viewmodel::runtime::unblock(move || {
        crate::calc::ssh::check_ansible_installed(&host_config)
    }).await;
    
    match status_result {
        Ok(installed) => {
            let _ = tx.send(SshEvent::AnsibleStatusRetrieved {
                name,
                installed,
            }).await;
        }
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "ansible".into(),
                operation: "get_status".into(),
                error: e,
            }).await;
        }
    }
}

SshCommand::UninstallAnsible { name } => {
    let host_config = match load_ssh_host_config(&name) {
        Ok(cfg) => cfg,
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "ansible".into(),
                operation: "uninstall".into(),
                error: format!("Failed to load config: {}", e),
            }).await;
            return;
        }
    };
    
    let uninstall_result = crate::viewmodel::runtime::unblock(move || {
        crate::calc::ssh::uninstall_ansible(&host_config)
    }).await;
    
    match uninstall_result {
        Ok(_) => {
            let _ = tx.send(SshEvent::AnsibleUninstalled { name }).await;
        }
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "ansible".into(),
                operation: "uninstall".into(),
                error: e,
            }).await;
        }
    }
}
```

- [ ] **Step 7: Add Dure-WSS handlers (call placeholders)**

```rust
SshCommand::InstallDureWss { name } => {
    let host_config = match load_ssh_host_config(&name) {
        Ok(cfg) => cfg,
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "dure-wss".into(),
                operation: "install".into(),
                error: format!("Failed to load config: {}", e),
            }).await;
            return;
        }
    };
    
    let install_result = crate::viewmodel::runtime::unblock(move || {
        crate::calc::ssh::install_dure_wss(&host_config)
    }).await;
    
    match install_result {
        Ok(_) => {
            let _ = tx.send(SshEvent::DureWssInstalled { name }).await;
        }
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "dure-wss".into(),
                operation: "install".into(),
                error: e,
            }).await;
        }
    }
}

SshCommand::GetDureWssStatus { name } => {
    let host_config = match load_ssh_host_config(&name) {
        Ok(cfg) => cfg,
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "dure-wss".into(),
                operation: "get_status".into(),
                error: format!("Failed to load config: {}", e),
            }).await;
            return;
        }
    };
    
    let status_result = crate::viewmodel::runtime::unblock(move || {
        crate::calc::ssh::check_dure_wss_installed(&host_config)
    }).await;
    
    match status_result {
        Ok(installed) => {
            let _ = tx.send(SshEvent::DureWssStatusRetrieved {
                name,
                installed,
            }).await;
        }
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "dure-wss".into(),
                operation: "get_status".into(),
                error: e,
            }).await;
        }
    }
}

SshCommand::UninstallDureWss { name } => {
    let host_config = match load_ssh_host_config(&name) {
        Ok(cfg) => cfg,
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "dure-wss".into(),
                operation: "uninstall".into(),
                error: format!("Failed to load config: {}", e),
            }).await;
            return;
        }
    };
    
    let uninstall_result = crate::viewmodel::runtime::unblock(move || {
        crate::calc::ssh::uninstall_dure_wss(&host_config)
    }).await;
    
    match uninstall_result {
        Ok(_) => {
            let _ = tx.send(SshEvent::DureWssUninstalled { name }).await;
        }
        Err(e) => {
            let _ = tx.send(SshEvent::ServiceError {
                name,
                service: "dure-wss".into(),
                operation: "uninstall".into(),
                error: e,
            }).await;
        }
    }
}
```

- [ ] **Step 8: Verify compilation**

Run: `cargo +nightly check --bin dure-desktop`
Expected: Compiles successfully

- [ ] **Step 9: Commit**

```bash
git add mobile/src/viewmodel/ssh/actor.rs
git commit -m "feat(actor): implement SSH service operation handlers

Add command handlers for GetLinuxStatus, InstallDocker, GetDockerStatus,
UninstallDocker, and placeholder handlers for Ansible/Dure-WSS operations.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 6: Add ViewModel Public API Methods

**Files:**
- Modify: `mobile/src/viewmodel/mod.rs:ViewModel impl` (add public methods)

**Interfaces:**
- Consumes: `SshCommand` from actor.rs
- Produces: Public API methods for UI layer

- [ ] **Step 1: Add get_linux_status method**

Open `mobile/src/viewmodel/mod.rs`. In the `impl ViewModel` block, add:

```rust
// Linux status
pub fn get_linux_status(&mut self, host: String) -> Result<(), String> {
    self.send_ssh_command(crate::viewmodel::ssh::actor::SshCommand::GetLinuxStatus { name: host })
}
```

- [ ] **Step 2: Add Docker lifecycle methods**

```rust
// Docker lifecycle
pub fn install_docker(&mut self, host: String) -> Result<(), String> {
    self.send_ssh_command(crate::viewmodel::ssh::actor::SshCommand::InstallDocker { name: host })
}

pub fn get_docker_status(&mut self, host: String) -> Result<(), String> {
    self.send_ssh_command(crate::viewmodel::ssh::actor::SshCommand::GetDockerStatus { name: host })
}

pub fn uninstall_docker(&mut self, host: String) -> Result<(), String> {
    self.send_ssh_command(crate::viewmodel::ssh::actor::SshCommand::UninstallDocker { name: host })
}
```

- [ ] **Step 3: Add Ansible lifecycle methods**

```rust
// Ansible lifecycle
pub fn install_ansible(&mut self, host: String) -> Result<(), String> {
    self.send_ssh_command(crate::viewmodel::ssh::actor::SshCommand::InstallAnsible { name: host })
}

pub fn get_ansible_status(&mut self, host: String) -> Result<(), String> {
    self.send_ssh_command(crate::viewmodel::ssh::actor::SshCommand::GetAnsibleStatus { name: host })
}

pub fn uninstall_ansible(&mut self, host: String) -> Result<(), String> {
    self.send_ssh_command(crate::viewmodel::ssh::actor::SshCommand::UninstallAnsible { name: host })
}
```

- [ ] **Step 4: Add Dure-WSS lifecycle methods**

```rust
// Dure-WSS lifecycle
pub fn install_dure_wss(&mut self, host: String) -> Result<(), String> {
    self.send_ssh_command(crate::viewmodel::ssh::actor::SshCommand::InstallDureWss { name: host })
}

pub fn get_dure_wss_status(&mut self, host: String) -> Result<(), String> {
    self.send_ssh_command(crate::viewmodel::ssh::actor::SshCommand::GetDureWssStatus { name: host })
}

pub fn uninstall_dure_wss(&mut self, host: String) -> Result<(), String> {
    self.send_ssh_command(crate::viewmodel::ssh::actor::SshCommand::UninstallDureWss { name: host })
}
```

- [ ] **Step 5: Verify compilation**

Run: `cargo +nightly check --bin dure-desktop`
Expected: Compiles successfully

- [ ] **Step 6: Commit**

```bash
git add mobile/src/viewmodel/mod.rs
git commit -m "feat(viewmodel): add SSH service management public API

Add ViewModel methods for Linux status, Docker, Ansible, and Dure-WSS
service operations. Clean public API for UI layer.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7: Implement SSH Tab UI - Helper Functions

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:785-end` (add after existing impl)

**Interfaces:**
- Produces: `format_platform()`, `format_status()`, helper functions

- [ ] **Step 1: Add format_platform helper**

At the end of `mobile/src/ui_tabs/ssh.rs` (after the existing `impl SshTab`), add:

```rust
/// Format platform relationship for display
fn format_platform(row: &SshRowData) -> String {
    match (&row.platform_name, &row.platform_type) {
        (Some(name), Some(ptype)) => format!("{}({})", name, ptype),
        _ => "manual".to_string(),
    }
}
```

- [ ] **Step 2: Add format_status helper**

```rust
/// Format status column showing only enabled services
fn format_status(row: &SshRowData) -> String {
    let mut parts = Vec::new();
    
    // Show Linux with OS if available
    if row.linux_detected {
        if let Some(os) = &row.linux_os {
            parts.push(format!("✓ linux({})", os));
        } else {
            parts.push("✓ linux".to_string());
        }
    }
    
    // Show enabled services
    if row.ansible_enabled {
        parts.push("✓ ansible".to_string());
    }
    
    if row.docker_enabled {
        parts.push("✓ docker".to_string());
    }
    
    if row.dure_wss_enabled {
        parts.push("✓ dure-wss".to_string());
    }
    
    if parts.is_empty() {
        "—".to_string()
    } else {
        parts.join(" ")
    }
}
```

- [ ] **Step 3: Add render_drawer_content helper**

```rust
/// Render drawer content with Linux status and service placeholders
fn render_drawer_content(ui: &mut egui::Ui, row: &SshRowData) {
    ui.add_space(8.0);
    
    // Linux status (detailed)
    ui.label(egui::RichText::new("linux:").strong());
    if let Some(status) = &row.linux_status {
        ui.label(format!("  uptime: {}", status.uptime));
        ui.label(format!("  ip: {}", status.external_ip));
        ui.label(format!("  load: {}", status.load_average));
        ui.label(format!("  memory: {}", status.memory_usage));
        ui.label(format!("  disk: {}", status.disk_usage));
        
        let processes = if status.top_processes.is_empty() {
            "none".to_string()
        } else {
            status.top_processes.join(", ")
        };
        ui.label(format!("  ps: {}", processes));
    } else {
        ui.colored_label(
            ui.visuals().weak_text_color(),
            "  (status not loaded - click Refresh to load)"
        );
    }
    
    ui.add_space(4.0);
    
    // Ansible placeholder
    ui.label(egui::RichText::new("ansible:").strong());
    ui.colored_label(ui.visuals().weak_text_color(), "  —");
    
    ui.add_space(4.0);
    
    // Docker placeholder
    ui.label(egui::RichText::new("docker:").strong());
    ui.colored_label(ui.visuals().weak_text_color(), "  —");
    
    ui.add_space(4.0);
    
    // Dure-WSS placeholder
    ui.label(egui::RichText::new("dure-wss:").strong());
    ui.colored_label(ui.visuals().weak_text_color(), "  —");
    
    ui.add_space(4.0);
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo +nightly check --bin dure-desktop`
Expected: Compiles successfully

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh-ui): add helper functions for drawer rendering

Add format_platform, format_status, and render_drawer_content helpers
for the new data_table UI.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 8: Implement SSH Tab UI - Update SshTab Struct

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:11-110` (SshTab struct and Default impl)

**Interfaces:**
- Replaces: Old MaterialSpreadsheet-based fields
- Produces: New SshTab with `rows: Vec<SshRowData>`

- [ ] **Step 1: Update SshTab struct fields**

Open `mobile/src/ui_tabs/ssh.rs`. Locate the `SshTab` struct (around line 11-61). Replace it with:

```rust
/// SSH tab state
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SshTab {
    /// Display data
    #[cfg_attr(feature = "serde", serde(skip))]
    rows: Vec<SshRowData>,
    
    #[cfg_attr(feature = "serde", serde(skip))]
    loaded: bool,
    
    #[cfg_attr(feature = "serde", serde(skip))]
    load_error: Option<String>,
    
    // Add host dialog
    #[cfg_attr(feature = "serde", serde(skip))]
    show_add_dialog: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_host: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_password: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_private_key_path: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_port: String,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_use_password: bool,
    #[cfg_attr(feature = "serde", serde(skip))]
    add_use_private_key: bool,
}
```

- [ ] **Step 2: Update Default impl**

Replace the `Default` impl (around line 63-110) with:

```rust
impl Default for SshTab {
    fn default() -> Self {
        Self {
            rows: Vec::new(),
            loaded: false,
            load_error: None,
            show_add_dialog: false,
            add_host: String::new(),
            add_password: String::new(),
            add_private_key_path: String::new(),
            add_port: "22".to_string(),
            add_use_password: false,
            add_use_private_key: false,
        }
    }
}
```

- [ ] **Step 3: Comment out or remove old functions temporarily**

Comment out these old methods (we'll replace them):
- `load_rows()` - will be rewritten
- `poll_connection_test()` - no longer needed (ViewModel handles)
- `render_init_progress()` - keep for now (still used)
- `render_test_result()` - keep for now (still used)

Just add `/*` and `*/` around methods that reference removed fields (like `spreadsheet`, `test_promise`).

- [ ] **Step 4: Verify compilation**

Run: `cargo +nightly check --bin dure-desktop`
Expected: May have warnings about unused fields, but should compile

- [ ] **Step 5: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "refactor(ssh-ui): update SshTab struct to use SshRowData

Remove MaterialSpreadsheet and promise-based state. Replace with
Vec<SshRowData> for new data_table pattern.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 9: Implement SSH Tab UI - New load_rows

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:load_rows()` (rewrite method)

**Interfaces:**
- Consumes: Config from `load_config()`
- Produces: Populates `self.rows: Vec<SshRowData>`

- [ ] **Step 1: Rewrite load_rows method**

In `mobile/src/ui_tabs/ssh.rs`, find the `impl SshTab` block. Add or replace the `load_rows` method:

```rust
impl SshTab {
    /// Load SSH hosts from config and build row data
    fn load_rows(&mut self) {
        self.rows.clear();
        self.load_error = None;
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            match load_config() {
                Ok((app_config, _)) => {
                    for host_config in &app_config.ssh_hosts {
                        // Resolve platform relationship
                        let (platform_name, platform_type) = if let Some(pname) = &host_config.platform_name {
                            let ptype = app_config.platforms
                                .iter()
                                .find(|p| &p.name == pname)
                                .map(|p| p.platform_type.clone());
                            (Some(pname.clone()), ptype)
                        } else {
                            (None, None)
                        };
                        
                        self.rows.push(SshRowData {
                            host: host_config.host.clone(),
                            port: host_config.port,
                            platform_name,
                            platform_type,
                            linux_detected: false,
                            linux_os: None,
                            ansible_enabled: false,
                            docker_enabled: false,
                            dure_wss_enabled: false,
                            linux_status: None,
                            connection_status: ConnectionStatus::Unknown,
                        });
                    }
                }
                Err(e) => {
                    self.load_error = Some(format!("Failed to load config: {}", e));
                }
            }
        }
        
        #[cfg(target_arch = "wasm32")]
        {
            self.load_error = Some("SSH management not available on WASM".to_string());
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo +nightly check --bin dure-desktop`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh-ui): implement new load_rows with platform relationship

Build SshRowData from config, resolving platform name/type from Platform
tab entries. Initialize service flags to false (queried on demand).

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 10: Implement SSH Tab UI - Event Handling

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:handle_event()` (new method)

**Interfaces:**
- Consumes: `ViewModelEvent::Ssh(SshEvent)` from ViewModel
- Produces: Updates to `self.rows`

- [ ] **Step 1: Add handle_event method**

In `mobile/src/ui_tabs/ssh.rs`, in the `impl SshTab` block, add:

```rust
impl SshTab {
    /// Handle ViewModel events to update UI state
    fn handle_event(&mut self, event: crate::viewmodel::ViewModelEvent) {
        use crate::viewmodel::ViewModelEvent;
        use crate::viewmodel::ssh::SshEvent;
        
        match event {
            ViewModelEvent::Ssh(SshEvent::HostAdded { name }) => {
                eprintln!("✓ SSH host {} added", name);
                self.loaded = false; // Trigger reload
            }
            
            ViewModelEvent::Ssh(SshEvent::HostDeleted { name }) => {
                eprintln!("✓ SSH host {} deleted", name);
                
                // Remove from config
                #[cfg(not(target_arch = "wasm32"))]
                if let Ok((mut app_config, config_path)) = load_config() {
                    app_config.ssh_hosts.retain(|h| h.host != name);
                    let _ = app_config.save(&config_path);
                }
                
                self.loaded = false; // Trigger reload
            }
            
            ViewModelEvent::Ssh(SshEvent::LinuxStatusRetrieved {
                name,
                uptime,
                external_ip,
                load_average,
                memory_usage,
                disk_usage,
                top_processes,
            }) => {
                eprintln!("✓ Linux status retrieved for {}", name);
                
                // Update row
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.linux_status = Some(LinuxStatus {
                        uptime,
                        external_ip,
                        load_average,
                        memory_usage,
                        disk_usage,
                        top_processes,
                    });
                    row.linux_detected = true;
                }
            }
            
            ViewModelEvent::Ssh(SshEvent::DockerInstalled { name }) => {
                eprintln!("✓ Docker installed on {}", name);
                
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.docker_enabled = true;
                }
            }
            
            ViewModelEvent::Ssh(SshEvent::DockerStatusRetrieved { name, installed, running: _ }) => {
                eprintln!("✓ Docker status for {}: installed={}", name, installed);
                
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.docker_enabled = installed;
                }
            }
            
            ViewModelEvent::Ssh(SshEvent::DockerUninstalled { name }) => {
                eprintln!("✓ Docker uninstalled from {}", name);
                
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.docker_enabled = false;
                }
            }
            
            ViewModelEvent::Ssh(SshEvent::AnsibleInstalled { name }) => {
                eprintln!("✓ Ansible installed on {}", name);
                
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.ansible_enabled = true;
                }
            }
            
            ViewModelEvent::Ssh(SshEvent::AnsibleStatusRetrieved { name, installed }) => {
                eprintln!("✓ Ansible status for {}: installed={}", name, installed);
                
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.ansible_enabled = installed;
                }
            }
            
            ViewModelEvent::Ssh(SshEvent::AnsibleUninstalled { name }) => {
                eprintln!("✓ Ansible uninstalled from {}", name);
                
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.ansible_enabled = false;
                }
            }
            
            ViewModelEvent::Ssh(SshEvent::DureWssInstalled { name }) => {
                eprintln!("✓ Dure-WSS installed on {}", name);
                
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.dure_wss_enabled = true;
                }
            }
            
            ViewModelEvent::Ssh(SshEvent::DureWssStatusRetrieved { name, installed }) => {
                eprintln!("✓ Dure-WSS status for {}: installed={}", name, installed);
                
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.dure_wss_enabled = installed;
                }
            }
            
            ViewModelEvent::Ssh(SshEvent::DureWssUninstalled { name }) => {
                eprintln!("✓ Dure-WSS uninstalled from {}", name);
                
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.dure_wss_enabled = false;
                }
            }
            
            ViewModelEvent::Ssh(SshEvent::ServiceError { name, service, operation, error }) => {
                self.load_error = Some(format!(
                    "Failed to {} {} on {}: {}", operation, service, name, error
                ));
            }
            
            ViewModelEvent::Ssh(SshEvent::Error { operation, error }) => {
                self.load_error = Some(format!(
                    "SSH operation '{}' failed: {}", operation, error
                ));
            }
            
            // Keep existing event handlers (ConnectionTested, HostInitialized, etc.)
            _ => {}
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo +nightly check --bin dure-desktop`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh-ui): implement event handling for service operations

Add handle_event method to process ViewModel events and update row state
for Linux status, Docker, Ansible, and Dure-WSS operations.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 11: Implement SSH Tab UI - Operations Buttons

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:render_operations()` (new function)

**Interfaces:**
- Produces: Dynamic operation buttons in table cells

- [ ] **Step 1: Add render_operations function**

In `mobile/src/ui_tabs/ssh.rs`, add this function after the helper functions:

```rust
/// Render dynamic operation buttons based on service state
fn render_operations(ui: &mut egui::Ui, row: &SshRowData, idx: usize) {
    use egui_material3::MaterialButton;
    
    egui::ScrollArea::horizontal()
        .id_salt(format!("operations_scroll_{}", idx))
        .auto_shrink([false, true])
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                ui.style_mut().spacing.button_padding = egui::vec2(6.0, 2.0);
                
                // Refresh - always available
                if ui.add(MaterialButton::outlined("Refresh").small())
                    .on_hover_text("Refresh host status")
                    .clicked()
                {
                    ui.data_mut(|d| d.insert_temp(
                        egui::Id::new(format!("ssh_refresh_{}", idx)),
                        row.host.clone()
                    ));
                }
                
                // Docker operations - dynamic based on state
                if !row.docker_enabled {
                    if ui.add(MaterialButton::outlined("Install Docker").small())
                        .on_hover_text("Install Docker")
                        .clicked()
                    {
                        ui.data_mut(|d| d.insert_temp(
                            egui::Id::new(format!("ssh_install_docker_{}", idx)),
                            row.host.clone()
                        ));
                    }
                } else {
                    if ui.add(MaterialButton::outlined("Docker Status").small())
                        .on_hover_text("Check Docker status")
                        .clicked()
                    {
                        ui.data_mut(|d| d.insert_temp(
                            egui::Id::new(format!("ssh_docker_status_{}", idx)),
                            row.host.clone()
                        ));
                    }
                    if ui.add(MaterialButton::outlined("Uninstall Docker").small())
                        .on_hover_text("Uninstall Docker")
                        .clicked()
                    {
                        ui.data_mut(|d| d.insert_temp(
                            egui::Id::new(format!("ssh_uninstall_docker_{}", idx)),
                            row.host.clone()
                        ));
                    }
                }
                
                // Ansible operations - similar pattern
                if !row.ansible_enabled {
                    if ui.add(MaterialButton::outlined("Install Ansible").small())
                        .on_hover_text("Install Ansible (placeholder)")
                        .clicked()
                    {
                        ui.data_mut(|d| d.insert_temp(
                            egui::Id::new(format!("ssh_install_ansible_{}", idx)),
                            row.host.clone()
                        ));
                    }
                } else {
                    if ui.add(MaterialButton::outlined("Ansible Status").small()).clicked() {
                        ui.data_mut(|d| d.insert_temp(
                            egui::Id::new(format!("ssh_ansible_status_{}", idx)),
                            row.host.clone()
                        ));
                    }
                    if ui.add(MaterialButton::outlined("Uninstall Ansible").small()).clicked() {
                        ui.data_mut(|d| d.insert_temp(
                            egui::Id::new(format!("ssh_uninstall_ansible_{}", idx)),
                            row.host.clone()
                        ));
                    }
                }
                
                // Dure-WSS operations - similar pattern
                if !row.dure_wss_enabled {
                    if ui.add(MaterialButton::outlined("Install Dure-WSS").small())
                        .on_hover_text("Install Dure-WSS (placeholder)")
                        .clicked()
                    {
                        ui.data_mut(|d| d.insert_temp(
                            egui::Id::new(format!("ssh_install_dure_wss_{}", idx)),
                            row.host.clone()
                        ));
                    }
                } else {
                    if ui.add(MaterialButton::outlined("Dure-WSS Status").small()).clicked() {
                        ui.data_mut(|d| d.insert_temp(
                            egui::Id::new(format!("ssh_dure_wss_status_{}", idx)),
                            row.host.clone()
                        ));
                    }
                    if ui.add(MaterialButton::outlined("Uninstall Dure-WSS").small()).clicked() {
                        ui.data_mut(|d| d.insert_temp(
                            egui::Id::new(format!("ssh_uninstall_dure_wss_{}", idx)),
                            row.host.clone()
                        ));
                    }
                }
                
                // Delete - always available
                if ui.add(MaterialButton::outlined("Delete").small())
                    .on_hover_text("Delete SSH host")
                    .clicked()
                {
                    ui.data_mut(|d| d.insert_temp(
                        egui::Id::new(format!("ssh_delete_{}", idx)),
                        row.host.clone()
                    ));
                }
            });
        });
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo +nightly check --bin dure-desktop`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh-ui): add dynamic operations buttons

Implement render_operations with state-based button display. Show Install
when service not enabled, show Status/Uninstall when enabled.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 12: Implement SSH Tab UI - Action Processing

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:process_action_triggers()` (new method)

**Interfaces:**
- Consumes: Temp data from egui, ViewModel from ui() method
- Produces: ViewModel command calls

- [ ] **Step 1: Add process_action_triggers method**

In `mobile/src/ui_tabs/ssh.rs`, in the `impl SshTab` block, add:

```rust
impl SshTab {
    /// Process action triggers from operation buttons
    fn process_action_triggers(&mut self, ui: &mut egui::Ui, vm: Option<&mut crate::viewmodel::ViewModel>) {
        let Some(vm) = vm else { return };
        
        // Check all possible action IDs
        for (idx, _row) in self.rows.iter().enumerate() {
            // Refresh
            if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new(format!("ssh_refresh_{}", idx)))) {
                let _ = vm.get_linux_status(host.clone());
                let _ = vm.get_docker_status(host.clone());
                let _ = vm.get_ansible_status(host.clone());
                let _ = vm.get_dure_wss_status(host);
            }
            
            // Docker operations
            if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new(format!("ssh_install_docker_{}", idx)))) {
                let _ = vm.install_docker(host);
            }
            if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new(format!("ssh_docker_status_{}", idx)))) {
                let _ = vm.get_docker_status(host);
            }
            if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new(format!("ssh_uninstall_docker_{}", idx)))) {
                let _ = vm.uninstall_docker(host);
            }
            
            // Ansible operations
            if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new(format!("ssh_install_ansible_{}", idx)))) {
                let _ = vm.install_ansible(host);
            }
            if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new(format!("ssh_ansible_status_{}", idx)))) {
                let _ = vm.get_ansible_status(host);
            }
            if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new(format!("ssh_uninstall_ansible_{}", idx)))) {
                let _ = vm.uninstall_ansible(host);
            }
            
            // Dure-WSS operations
            if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new(format!("ssh_install_dure_wss_{}", idx)))) {
                let _ = vm.install_dure_wss(host);
            }
            if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new(format!("ssh_dure_wss_status_{}", idx)))) {
                let _ = vm.get_dure_wss_status(host);
            }
            if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new(format!("ssh_uninstall_dure_wss_{}", idx)))) {
                let _ = vm.uninstall_dure_wss(host);
            }
            
            // Delete
            if let Some(host) = ui.data(|d| d.get_temp::<String>(egui::Id::new(format!("ssh_delete_{}", idx)))) {
                let _ = vm.delete_ssh_host(host);
            }
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo +nightly check --bin dure-desktop`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh-ui): add action processing for operation buttons

Implement process_action_triggers to read egui temp data and call
ViewModel methods for all service operations.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 13: Implement SSH Tab UI - Table Rendering

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:render_table()` (new method)

**Interfaces:**
- Consumes: `self.rows`, helper functions from Tasks 7, 11
- Produces: data_table with drawers

- [ ] **Step 1: Add render_table method**

In `mobile/src/ui_tabs/ssh.rs`, in the `impl SshTab` block, add:

```rust
impl SshTab {
    /// Render the SSH hosts table with drawers
    fn render_table(&mut self, ui: &mut egui::Ui, vm: Option<&mut crate::viewmodel::ViewModel>) {
        use egui_material3::data_table;
        
        let table_id = egui::Id::new("ssh_table");
        
        // Initialize drawer state (all closed by default)
        use egui_material3::datatable::DataTableState;
        let state: DataTableState = ui.data_mut(|d| {
            d.get_persisted::<DataTableState>(table_id)
                .unwrap_or_default()
        });
        ui.data_mut(|d| d.insert_persisted(table_id, state));
        
        // Build table
        let mut table = data_table()
            .id(table_id)
            .allow_selection(false)
            .allow_drawer(true)
            .column("Host (Port)", 200.0, false)
            .column("Platform", 150.0, false)
            .column("Status", 300.0, false)
            .column("Operations", 350.0, false);
        
        for (idx, row) in self.rows.iter().enumerate() {
            let row_for_cells = row.clone();
            let row_for_drawer = row.clone();
            let row_for_ops = row.clone();
            
            table = table.row(move |r| {
                r.cell(&format!("{}:{}", row_for_cells.host, row_for_cells.port))
                 .cell(&format_platform(&row_for_cells))
                 .cell(&format_status(&row_for_cells))
                 .widget_cell(move |ui| {
                     render_operations(ui, &row_for_ops, idx);
                 })
                 .drawer(move |ui| {
                     render_drawer_content(ui, &row_for_drawer);
                 })
            });
        }
        
        egui::ScrollArea::vertical().show(ui, |ui| {
            table.show(ui);
        });
        
        // Process action triggers from operations buttons
        self.process_action_triggers(ui, vm);
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo +nightly check --bin dure-desktop`
Expected: Compiles successfully

- [ ] **Step 3: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh-ui): implement data_table rendering with drawers

Add render_table method using egui_material3 data_table. Wire up cells,
drawer content, and action processing.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 14: Implement SSH Tab UI - Main ui() Method

**Files:**
- Modify: `mobile/src/ui_tabs/ssh.rs:ui()` (rewrite method)

**Interfaces:**
- Consumes: ViewModel, all helper methods from previous tasks
- Produces: Complete SSH tab UI

- [ ] **Step 1: Rewrite ui() method**

In `mobile/src/ui_tabs/ssh.rs`, locate the `pub fn ui()` method (around line 130). Replace it with:

```rust
impl SshTab {
    /// Render the SSH tab UI
    pub fn ui(&mut self, ui: &mut egui::Ui, mut vm: Option<&mut crate::viewmodel::ViewModel>) {
        use egui_material3::MaterialButton;
        
        // 1. Process ViewModel events
        if let Some(ref mut vm) = vm {
            let events = vm.poll_events(ui.ctx());
            for event in events {
                self.handle_event(event);
            }
            
            // 2. Show active operations with progress bars
            for (_op_id, progress) in vm.active_operations() {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::ProgressBar::new(progress.progress)
                            .text(format!("{}: {}", progress.operation, progress.status))
                            .desired_width(400.0)
                    );
                });
            }
            
            // 3. Show recent errors
            if let Some(error) = vm.recent_errors()
                .iter()
                .filter(|e| e.actor == "ssh")
                .rev()
                .next()
            {
                ui.colored_label(
                    egui::Color32::from_rgb(255, 100, 100),
                    format!("⚠ Error in {}: {}", error.operation, error.error)
                );
                ui.add_space(4.0);
            }
        }
        
        // 4. Header
        ui.heading("SSH Hosts");
        ui.add_space(4.0);
        ui.label("Manage SSH hosts for remote server deployment and management.");
        ui.add_space(8.0);
        
        // 5. Add Host button
        if ui.add(MaterialButton::filled("Add Host")).clicked() {
            self.show_add_dialog = true;
            self.add_host.clear();
            self.add_password.clear();
            self.add_private_key_path.clear();
            self.add_port = "22".to_string();
            self.add_use_password = false;
            self.add_use_private_key = false;
        }
        ui.add_space(8.0);
        
        // 6. Load rows on demand
        if !self.loaded {
            self.load_rows();
            self.loaded = true;
        }
        
        // 7. Error display
        if let Some(error) = &self.load_error {
            ui.colored_label(egui::Color32::RED, format!("⚠ {}", error));
            ui.add_space(4.0);
        }
        
        // 8. Render table or empty state
        if self.rows.is_empty() {
            ui.label("No SSH hosts configured. Click 'Add Host' to get started.");
        } else {
            self.render_table(ui, vm);
        }
        
        // 9. Dialogs
        if self.show_add_dialog {
            self.render_add_dialog(ui.ctx(), vm.as_deref_mut());
        }
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo +nightly build --bin dure-desktop`
Expected: Compiles successfully. May have warnings about unused methods (render_add_dialog is kept from old code).

- [ ] **Step 3: Test the UI**

Run: `cargo +nightly run --bin dure-desktop`
Navigate to SSH tab. Verify:
- Table renders with columns
- Empty state shows if no hosts
- Add Host button shows dialog (existing functionality)

- [ ] **Step 4: Commit**

```bash
git add mobile/src/ui_tabs/ssh.rs
git commit -m "feat(ssh-ui): rewrite main ui() method with new table

Complete SSH tab UI rewrite. Process ViewModel events, show progress bars,
render data_table with drawers. Maintains Add Host dialog.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 15: Integration Testing and Bug Fixes

**Files:**
- Various files as needed for fixes

**Interfaces:**
- Complete integration testing

- [ ] **Step 1: Test basic table rendering**

Run: `cargo +nightly run --bin dure-desktop`

Manual test checklist:
- [ ] SSH tab renders without crashes
- [ ] Table shows correct columns (Host, Platform, Status, Operations)
- [ ] Existing SSH hosts load (if any configured)
- [ ] Platform column shows "manual" for hosts without platform_name
- [ ] Status column shows "—" initially (no services detected yet)
- [ ] Operations buttons render

- [ ] **Step 2: Test Add Host functionality**

- [ ] Click "Add Host" button
- [ ] Dialog appears (existing functionality)
- [ ] Add a test host
- [ ] Verify it appears in table
- [ ] Verify platform_name defaults to None (shows "manual")

- [ ] **Step 3: Test drawer expansion**

- [ ] Click on a row to expand drawer
- [ ] Verify drawer shows Linux section with "(status not loaded)"
- [ ] Verify ansible, docker, dure-wss sections show "—"
- [ ] Click row again to collapse drawer

- [ ] **Step 4: Test Refresh button**

- [ ] Click "Refresh" button on a row
- [ ] Verify progress bar appears
- [ ] Wait for Linux status query to complete (~5s)
- [ ] Verify drawer updates with actual system info
- [ ] Verify status column updates to "✓ linux(os_name)"

- [ ] **Step 5: Test Docker operations (requires SSH host with Docker)**

- [ ] Click "Install Docker" (this takes 30-60s)
- [ ] Verify progress bar shows
- [ ] Wait for completion
- [ ] Verify buttons change to "Docker Status" / "Uninstall Docker"
- [ ] Verify status column shows "✓ linux(...) ✓ docker"
- [ ] Click "Docker Status" to refresh
- [ ] Click "Uninstall Docker" to test removal

- [ ] **Step 6: Test error scenarios**

- [ ] Add an SSH host with wrong credentials
- [ ] Click Refresh - verify error message displays
- [ ] Click Install Docker - verify error handling

- [ ] **Step 7: Test platform relationship**

Create a test platform in config:
```yaml
platforms:
  - name: "test-gcp"
    platform_type: "GCP"
```

Add SSH host with platform_name:
```yaml
ssh_hosts:
  - host: "root@test.com"
    port: 22
    platform_name: "test-gcp"
```

- [ ] Reload app
- [ ] Verify Platform column shows "test-gcp(GCP)"

- [ ] **Step 8: Fix any bugs found**

Document and fix any issues discovered during testing. Common issues to check:
- Compilation errors
- Runtime panics
- Event processing bugs
- UI layout issues
- Missing error handling

- [ ] **Step 9: Commit bug fixes**

```bash
git add <fixed-files>
git commit -m "fix(ssh-ui): resolve integration test issues

<describe specific fixes>

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 16: Update Documentation

**Files:**
- Modify: `docs/MVVM_MIGRATION_STATUS.md` (update SSH tab status)
- Modify: `docs/PROJECT_SUMMARY.md` (if needed)

**Interfaces:**
- Final documentation updates

- [ ] **Step 1: Update MVVM_MIGRATION_STATUS.md**

Open `docs/MVVM_MIGRATION_STATUS.md`. Update the SSH Tab section (around line 117-128):

```markdown
- ✅ **Task 11**: SSH Tab ViewModel Integration
  - **Status**: Complete - drawer table with service management
  - **File**: `mobile/src/ui_tabs/ssh.rs`
  - ui() accepts `Option<&mut ViewModel>` ✅
  - DureApp passes viewmodel.as_mut() ✅
  - Event processing pattern implemented ✅
  - **Completed Operations** (9 total):
    - ✅ SSH host add/delete/init/test
    - ✅ Linux status retrieval
    - ✅ Docker install/status/uninstall
    - ✅ Ansible/Dure-WSS status checks (placeholders)
  - **UI Changes**:
    - ✅ Replaced MaterialSpreadsheet with data_table + drawer
    - ✅ Platform relationship shows in Platform column
    - ✅ Dynamic operations buttons based on service state
    - ✅ Linux status in drawer (uptime, IP, memory, disk, load, ps)
    - ✅ Placeholder sections for ansible, docker, dure-wss
  - **Remaining**: Full ansible/docker/dure-wss management (future phases)
```

Update overall stats:
```markdown
## ✅ MVVM Architecture Complete!

**Status:** 
- ✅ **Actor Layer**: 100% complete (40/40 operations including new SSH services)
- ✅ **UI Migration**: 60% complete (18/30 operations - added 5 SSH service ops)
```

- [ ] **Step 2: Update commit counts**

Update the commit history section if tracking commit numbers.

- [ ] **Step 3: Commit documentation**

```bash
git add docs/MVVM_MIGRATION_STATUS.md
git commit -m "docs: update MVVM status - SSH tab drawer complete

SSH tab redesigned with data_table + drawer. Linux status working,
Docker operations implemented, ansible/dure-wss placeholders.
Platform relationship added.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 17: Final Cleanup and Polish

**Files:**
- Various files as needed

**Interfaces:**
- Code cleanup and polish

- [ ] **Step 1: Remove dead code**

Check for and remove:
- [ ] Old MaterialSpreadsheet code (if any remains)
- [ ] Old promise-based async code (test_promise, init_promise)
- [ ] Unused imports
- [ ] Commented-out code blocks

- [ ] **Step 2: Run clippy**

Run: `cargo +nightly clippy --bin dure-desktop -- -D warnings`

Fix any warnings found. Common issues:
- Unused variables
- Redundant clones
- Needless borrow
- Missing error handling

- [ ] **Step 3: Format code**

Run: `cargo +nightly fmt`

- [ ] **Step 4: Run final build**

Run: `cargo +nightly build --release --bin dure-desktop`

Verify: Builds successfully with no warnings.

- [ ] **Step 5: Final manual test**

Run the release build and do a quick smoke test:
- [ ] SSH tab loads
- [ ] Add host works
- [ ] Refresh works
- [ ] Drawer expands/collapses
- [ ] Operations buttons work

- [ ] **Step 6: Commit cleanup**

```bash
git add -A
git commit -m "chore: final cleanup for SSH tab drawer redesign

Remove dead code, fix clippy warnings, format code.
SSH tab drawer redesign complete and ready for review.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Success Criteria

At completion, the following should be true:

**Functionality:**
- ✅ SSH tab uses data_table with expandable drawers
- ✅ Platform relationship shows in table (platform_name → "gcp(GCP)" or "manual")
- ✅ Status column shows only enabled services (✓ linux(os) ✓ docker)
- ✅ Operations buttons are dynamic based on service state
- ✅ Linux status displayed in drawer (uptime, IP, memory, disk, load, processes)
- ✅ Docker install/status/uninstall working
- ✅ Ansible/Dure-WSS placeholders in drawer
- ✅ Progress bars show during operations
- ✅ Error messages display clearly
- ✅ Config backward compatible (platform_name defaults to None)

**Code Quality:**
- ✅ MVVM pattern followed (UI → ViewModel → Actor → calc)
- ✅ No compilation errors or warnings
- ✅ Clippy clean
- ✅ Code formatted
- ✅ Documentation updated

**Testing:**
- ✅ All manual tests pass
- ✅ No regressions in existing SSH operations
- ✅ Error scenarios handled gracefully

---

## Notes for Implementer

**Common Pitfalls:**
1. **Actor channel errors**: Make sure `send()` is always `.await`ed
2. **Event matching**: Use exact field names from SshEvent enum
3. **Row finding**: Always use `.find(|r| r.host == name)` to locate correct row
4. **Platform relationship**: Check both platform_name and platforms list exist
5. **Drawer updates**: Remember to set `linux_detected = true` when status arrives

**Performance Tips:**
- Linux status query takes ~5s per host
- Docker install takes 30-60s
- Don't auto-query all hosts on load (lazy load on drawer open or Refresh)

**Testing Requirements:**
- Requires live SSH host for full testing
- Can use Docker container for local testing
- Ansible/Dure-WSS placeholders will show "not implemented" errors (expected)

**Future Extensions:**
- Ansible management (roles, playbooks)
- Docker management (containers, images)
- Dure-WSS management (config, logs)
- SSH connection pooling for performance
- Status caching with TTL
