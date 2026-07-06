# SSH Tab Drawer Redesign - Design Specification

**Date:** 2026-07-05  
**Status:** Approved  
**Branch:** `feat/mvvm-refactor`  
**Approach:** Clean Rewrite with Unified Data Model

## Executive Summary

Redesign the SSH tab to use an expandable drawer table (matching Platform tab UX) with service management capabilities. The implementation follows the MVVM pattern with actor-based concurrency, showing comprehensive Linux system status and placeholder sections for future ansible/docker/dure-wss management.

### Scope

**Phase 1 (This Design):**
- Reimplement SSH tab UI with data_table + drawer
- Add platform relationship (SSH host → Platform)
- Implement Linux status display (uptime, IP, memory, disk, load, processes)
- Add placeholder sections for ansible, docker, dure-wss
- ViewModel/Actor methods for service operations
- Dynamic operation buttons based on service state

**Future Phases:**
- Ansible management (daemon, roles, lifecycle)
- Docker management (daemon, containers, lifecycle)
- Dure-WSS management (service, lifecycle)

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Data Models](#data-models)
3. [ViewModel/Actor Layer](#viewmodelactor-layer)
4. [UI Layer](#ui-layer)
5. [Data Flow & Error Handling](#data-flow--error-handling)
6. [Testing Strategy](#testing-strategy)
7. [Implementation Notes](#implementation-notes)
8. [Migration Path](#migration-path)

---

## Architecture Overview

### System Structure

```
┌─────────────────────────────────────────────────────────────┐
│                      SSH Tab UI Layer                        │
│  ┌────────────────────────────────────────────────────┐     │
│  │ SshTab::ui()                                       │     │
│  │  • Renders data_table with drawers                │     │
│  │  • Processes ViewModel events                     │     │
│  │  • Handles user actions → ViewModel commands      │     │
│  └────────────────────────────────────────────────────┘     │
│                           ▲                                   │
│                           │ SshRowData                        │
│  ┌────────────────────────▼──────────────────────────┐     │
│  │ load_rows()                                        │     │
│  │  • Reads config + platform data                   │     │
│  │  • Builds SshRowData for each host                │     │
│  └────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
                           │
                           │ Commands
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                  ViewModel Layer                             │
│  ┌────────────────────────────────────────────────────┐     │
│  │ ViewModel::get_linux_status(host)                 │     │
│  │ ViewModel::install_docker(host)                   │     │
│  │ ViewModel::get_docker_status(host)                │     │
│  │ ViewModel::install_ansible(host)                  │     │
│  └────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
                           │
                           │ SshCommand
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    SshActor (async)                          │
│  ┌────────────────────────────────────────────────────┐     │
│  │ Commands:                                          │     │
│  │  • GetLinuxStatus                                  │     │
│  │  • InstallDocker / GetDockerStatus                │     │
│  │  • InstallAnsible / GetAnsibleStatus              │     │
│  │  • InstallDureWss / GetDureWssStatus              │     │
│  └────────────────────────────────────────────────────┘     │
│                           ▲                                   │
│                           │ Events                            │
│  ┌────────────────────────▼──────────────────────────┐     │
│  │ Events:                                            │     │
│  │  • LinuxStatusRetrieved                           │     │
│  │  • DockerInstalled / DockerStatusRetrieved        │     │
│  │  • ServiceError                                    │     │
│  └────────────────────────────────────────────────────┘     │
└─────────────────────────────────────────────────────────────┘
                           │
                           │ SSH + shell commands
                           ▼
┌─────────────────────────────────────────────────────────────┐
│              calc::ssh (Business Logic)                      │
│  • execute_command(host, cmd) → Result<String>              │
│  • parse_linux_status(output) → LinuxStatus                 │
│  • detect_service_installed(host, service) → bool           │
└─────────────────────────────────────────────────────────────┘
```

### Key Design Principles

1. **MVVM Pattern**: UI → ViewModel → Actor → calc layer
2. **Event-Driven Updates**: Actor emits events, UI polls and updates
3. **Data Model Separation**: `SshRowData` for display, `SshHostConfig` for persistence
4. **Async by Default**: All SSH/service operations run in actor background threads
5. **Platform Integration**: SSH hosts store optional `platform_name` linking to Platform tab

### Design Rationale

**Why data_table instead of MaterialSpreadsheet?**
- Enables expandable drawers for detailed information
- Consistent UX with Platform tab
- Better support for complex cell content (buttons, widgets)
- Built-in state management for drawer open/close

**Why MVVM for service operations?**
- Consistency with ongoing MVVM migration (17/30 operations complete)
- Non-blocking UI during long SSH operations
- Centralized error handling through events
- Easy to test actors independently

**Why store platform relationship in config?**
- Fast lookup without network calls
- Survives app restarts
- Simple one-way relationship (SSH → Platform)
- Matches existing config patterns

---

## Data Models

### 1. SshRowData (Display Model)

```rust
/// Display data for SSH table row + drawer
#[derive(Clone, Debug)]
struct SshRowData {
    // Identity
    host: String,              // "root@example.com"
    port: u16,                 // 22
    
    // Platform relationship
    platform_name: Option<String>,  // Some("gcp") or None
    platform_type: Option<String>,  // Some("GCP") or None
    
    // Service status flags (for status column display)
    linux_detected: bool,
    linux_os: Option<String>,       // "debian", "ubuntu", etc.
    ansible_enabled: bool,
    docker_enabled: bool,
    dure_wss_enabled: bool,
    
    // Drawer content data
    linux_status: Option<LinuxStatus>,
    
    // Connection state
    connection_status: ConnectionStatus,  // Connected, Offline, Testing
}

#[derive(Clone, Debug)]
enum ConnectionStatus {
    Connected,
    Offline,
    Testing,
    Unknown,
}
```

**Field Descriptions:**

| Field | Purpose | Source |
|-------|---------|--------|
| `host`, `port` | Identity, shown in table | Config |
| `platform_name`, `platform_type` | Link to Platform tab | Config + Platform config |
| `linux_detected` | Whether Linux status was queried | Runtime (after query) |
| `linux_os` | Detected OS distribution | Runtime (from `uname` or `/etc/os-release`) |
| `*_enabled` flags | Service installation state | Runtime (from service checks) |
| `linux_status` | Detailed system info | Runtime (from SSH commands) |
| `connection_status` | SSH reachability | Runtime (from test_connection) |

### 2. LinuxStatus (System Information)

```rust
#[derive(Clone, Debug)]
struct LinuxStatus {
    uptime: String,              // "up 2 days, 3 hours"
    external_ip: String,         // "113.52.198.120"
    load_average: String,        // "0.15, 0.10, 0.08"
    memory_usage: String,        // "2.1G / 4.0G (52%)"
    disk_usage: String,          // "15G / 50G (30%)"
    top_processes: Vec<String>,  // ["apache2", "mysqld", "sshd"]
}
```

**Data Collection Commands:**

| Field | Command |
|-------|---------|
| `uptime` | `uptime -p` |
| `external_ip` | `curl -s ifconfig.me` |
| `load_average` | `cat /proc/loadavg \| awk '{print $1, $2, $3}'` |
| `memory_usage` | `free -h \| grep Mem \| awk '{print $3 " / " $2}'` |
| `disk_usage` | `df -h / \| tail -1 \| awk '{print $3 " / " $2 " (" $5 ")"}'` |
| `top_processes` | `ps aux --sort=-%mem \| head -6 \| tail -5 \| awk '{print $11}'` |

### 3. Config Updates

**Add to `mobile/src/config.rs`:**

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshHostConfig {
    pub host: String,
    pub port: u16,
    pub password: Option<String>,
    pub private_key_path: Option<String>,
    pub initialized: bool,
    
    // NEW: Platform relationship
    #[serde(default)]
    pub platform_name: Option<String>,  // Links to CloudPlatformConfig.name
}
```

**Migration:** Existing configs without `platform_name` will default to `None` (shown as "manual").

### 4. ViewModel Commands (New)

**Add to `mobile/src/viewmodel/ssh/actor.rs`:**

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

### 5. ViewModel Events (New)

**Add to `mobile/src/viewmodel/ssh/mod.rs`:**

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
        status: LinuxStatus,
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
        roles: Vec<String>,
    },
    AnsibleUninstalled { name: String },
    
    DureWssInstalled { name: String },
    DureWssStatusRetrieved {
        name: String,
        installed: bool,
        running: bool,
    },
    DureWssUninstalled { name: String },
    
    ServiceError {
        name: String,
        service: String,  // "linux", "docker", "ansible", "dure-wss"
        operation: String, // "get_status", "install", "uninstall"
        error: String,
    },
    
    Error { operation: String, error: String },
}
```

---

## ViewModel/Actor Layer

### ViewModel Public API

**Add to `mobile/src/viewmodel/mod.rs`:**

```rust
impl ViewModel {
    // Linux status
    pub fn get_linux_status(&mut self, host: String) -> Result<(), String> {
        self.send_ssh_command(SshCommand::GetLinuxStatus { name: host })
    }
    
    // Docker lifecycle
    pub fn install_docker(&mut self, host: String) -> Result<(), String> {
        self.send_ssh_command(SshCommand::InstallDocker { name: host })
    }
    
    pub fn get_docker_status(&mut self, host: String) -> Result<(), String> {
        self.send_ssh_command(SshCommand::GetDockerStatus { name: host })
    }
    
    pub fn uninstall_docker(&mut self, host: String) -> Result<(), String> {
        self.send_ssh_command(SshCommand::UninstallDocker { name: host })
    }
    
    // Ansible lifecycle
    pub fn install_ansible(&mut self, host: String) -> Result<(), String> {
        self.send_ssh_command(SshCommand::InstallAnsible { name: host })
    }
    
    pub fn get_ansible_status(&mut self, host: String) -> Result<(), String> {
        self.send_ssh_command(SshCommand::GetAnsibleStatus { name: host })
    }
    
    pub fn uninstall_ansible(&mut self, host: String) -> Result<(), String> {
        self.send_ssh_command(SshCommand::UninstallAnsible { name: host })
    }
    
    // Dure-WSS lifecycle
    pub fn install_dure_wss(&mut self, host: String) -> Result<(), String> {
        self.send_ssh_command(SshCommand::InstallDureWss { name: host })
    }
    
    pub fn get_dure_wss_status(&mut self, host: String) -> Result<(), String> {
        self.send_ssh_command(SshCommand::GetDureWssStatus { name: host })
    }
    
    pub fn uninstall_dure_wss(&mut self, host: String) -> Result<(), String> {
        self.send_ssh_command(SshCommand::UninstallDureWss { name: host })
    }
}
```

### Actor Command Handling Pattern

**In `mobile/src/viewmodel/ssh/actor.rs`:**

```rust
async fn handle_get_linux_status(
    name: String,
    tx: Sender<SshEvent>,
) {
    // 1. Load host config
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
    
    // 2. Execute SSH commands via calc layer (async)
    let status_result = runtime::unblock(move || {
        calc::ssh::get_linux_status(&host_config)
    }).await;
    
    // 3. Emit event with result
    match status_result {
        Ok(status) => {
            let _ = tx.send(SshEvent::LinuxStatusRetrieved {
                name,
                status,
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

async fn handle_install_docker(
    name: String,
    tx: Sender<SshEvent>,
) {
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
    
    // Install Docker via convenience script
    let install_result = runtime::unblock(move || {
        calc::ssh::install_docker(&host_config)
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

// Similar handlers for:
// - handle_get_docker_status
// - handle_uninstall_docker
// - handle_install_ansible
// - handle_get_ansible_status
// - handle_uninstall_ansible
// - handle_install_dure_wss
// - handle_get_dure_wss_status
// - handle_uninstall_dure_wss
```

### Calc Layer Functions (New)

**Add to `mobile/src/calc/ssh.rs`:**

```rust
/// Get comprehensive Linux system status
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
    
    let disk = execute_ssh_command(&session, "df -h / | tail -1 | awk '{print $3 \" / \" $2 \" (\" $5 \")\"}'")
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

/// Detect OS distribution
pub fn detect_os(host_config: &SshHostConfig) -> Result<String, String> {
    let session = establish_connection(host_config)?;
    
    // Try /etc/os-release first (modern standard)
    if let Ok(output) = execute_ssh_command(&session, "cat /etc/os-release | grep '^ID=' | cut -d= -f2 | tr -d '\"'") {
        return Ok(output.trim().to_string());
    }
    
    // Fallback to uname
    if let Ok(output) = execute_ssh_command(&session, "uname -s") {
        return Ok(output.trim().to_lowercase());
    }
    
    Ok("unknown".to_string())
}

/// Check if Docker is installed
pub fn check_docker_installed(host_config: &SshHostConfig) -> Result<bool, String> {
    let session = establish_connection(host_config)?;
    let result = execute_ssh_command(&session, "command -v docker");
    Ok(result.is_ok() && !result.unwrap().trim().is_empty())
}

/// Check if Docker daemon is running
pub fn check_docker_running(host_config: &SshHostConfig) -> Result<bool, String> {
    let session = establish_connection(host_config)?;
    let result = execute_ssh_command(&session, "systemctl is-active docker");
    Ok(result.is_ok() && result.unwrap().trim() == "active")
}

/// Install Docker via convenience script
pub fn install_docker(host_config: &SshHostConfig) -> Result<(), String> {
    let session = establish_connection(host_config)?;
    
    // Download and execute Docker install script
    execute_ssh_command(&session, 
        "curl -fsSL https://get.docker.com | sh"
    )?;
    
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
    
    // Remove packages (works on Debian/Ubuntu)
    execute_ssh_command(&session, 
        "apt-get remove -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin"
    )?;
    
    Ok(())
}

// Placeholder functions for future implementation:

pub fn check_ansible_installed(host_config: &SshHostConfig) -> Result<bool, String> {
    let session = establish_connection(host_config)?;
    let result = execute_ssh_command(&session, "command -v ansible");
    Ok(result.is_ok() && !result.unwrap().trim().is_empty())
}

pub fn install_ansible(host_config: &SshHostConfig) -> Result<(), String> {
    // TODO: Implement ansible installation
    Err("Ansible installation not yet implemented".to_string())
}

pub fn uninstall_ansible(host_config: &SshHostConfig) -> Result<(), String> {
    // TODO: Implement ansible uninstallation
    Err("Ansible uninstallation not yet implemented".to_string())
}

pub fn check_dure_wss_installed(host_config: &SshHostConfig) -> Result<bool, String> {
    let session = establish_connection(host_config)?;
    let result = execute_ssh_command(&session, "command -v dure");
    Ok(result.is_ok() && !result.unwrap().trim().is_empty())
}

pub fn install_dure_wss(host_config: &SshHostConfig) -> Result<(), String> {
    // TODO: Implement dure-wss installation
    Err("Dure-WSS installation not yet implemented".to_string())
}

pub fn uninstall_dure_wss(host_config: &SshHostConfig) -> Result<(), String> {
    // TODO: Implement dure-wss uninstallation
    Err("Dure-WSS uninstallation not yet implemented".to_string())
}
```

---

## UI Layer

### SshTab Structure (Updated)

**In `mobile/src/ui_tabs/ssh.rs`:**

```rust
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
pub struct SshTab {
    // Display data
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
    
    // REMOVED: MaterialSpreadsheet, init_promise, test_promise
    // (ViewModel handles these now)
}
```

### Main UI Method

```rust
impl SshTab {
    pub fn ui(&mut self, ui: &mut egui::Ui, mut vm: Option<&mut crate::viewmodel::ViewModel>) {
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

### Event Handling

```rust
impl SshTab {
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
            
            ViewModelEvent::Ssh(SshEvent::LinuxStatusRetrieved { name, status }) => {
                eprintln!("✓ Linux status retrieved for {}", name);
                
                // Update row
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.linux_status = Some(status);
                    row.linux_detected = true;
                }
            }
            
            ViewModelEvent::Ssh(SshEvent::DockerInstalled { name }) => {
                eprintln!("✓ Docker installed on {}", name);
                
                // Update row
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.docker_enabled = true;
                }
            }
            
            ViewModelEvent::Ssh(SshEvent::DockerStatusRetrieved { name, installed, running }) => {
                eprintln!("✓ Docker status for {}: installed={}, running={}", name, installed, running);
                
                // Update row
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.docker_enabled = installed;
                }
            }
            
            ViewModelEvent::Ssh(SshEvent::DockerUninstalled { name }) => {
                eprintln!("✓ Docker uninstalled from {}", name);
                
                // Update row
                if let Some(row) = self.rows.iter_mut().find(|r| r.host == name) {
                    row.docker_enabled = false;
                }
            }
            
            ViewModelEvent::Ssh(SshEvent::ServiceError { name, service, operation, error }) => {
                self.load_error = Some(format!(
                    "Failed to {} {} on {}: {}", operation, service, name, error
                ));
            }
            
            // TODO: Handle ansible and dure-wss events
            _ => {}
        }
    }
}
```

### Load Rows

```rust
impl SshTab {
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

### Table Rendering

```rust
impl SshTab {
    fn render_table(&mut self, ui: &mut egui::Ui, vm: Option<&mut crate::viewmodel::ViewModel>) {
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
            .column("Status", 250.0, false)
            .column("Operations", 300.0, false);
        
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

### Formatting Functions

```rust
fn format_platform(row: &SshRowData) -> String {
    match (&row.platform_name, &row.platform_type) {
        (Some(name), Some(ptype)) => format!("{}({})", name, ptype),
        _ => "manual".to_string(),
    }
}

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

### Dynamic Operations Buttons

```rust
fn render_operations(ui: &mut egui::Ui, row: &SshRowData, idx: usize) {
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
                        .on_hover_text("Install Ansible")
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
                    if ui.add(MaterialButton::outlined("Install Dure-WSS").small()).clicked() {
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

### Action Processing

```rust
impl SshTab {
    fn process_action_triggers(&mut self, ui: &mut egui::Ui, vm: Option<&mut crate::viewmodel::ViewModel>) {
        let Some(vm) = vm else { return };
        
        // Check all possible action IDs
        for (idx, row) in self.rows.iter().enumerate() {
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

### Drawer Content Rendering

```rust
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
        ui.label(format!("  ps: {}", status.top_processes.join(", ")));
    } else {
        ui.colored_label(
            ui.visuals().weak_text_color(),
            "  (status not loaded - click drawer to load)"
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

---

## Data Flow & Error Handling

### Complete Data Flow: "Install Docker"

```
1. USER CLICKS "Install Docker" button
   ↓
2. UI stores action in egui temp data
   ui.data_mut().insert_temp(Id::new("ssh_install_docker_0"), "root@example.com")
   ↓
3. UI reads temp data in process_action_triggers() and calls ViewModel
   vm.install_docker("root@example.com")?;
   ↓
4. ViewModel sends command to Actor
   self.send_ssh_command(SshCommand::InstallDocker { 
       name: "root@example.com".into() 
   })
   ↓
5. Actor receives command in background thread
   ↓
6. Actor calls calc layer (async, with timeout)
   let result = smol::future::or(
       runtime::unblock(|| calc::ssh::install_docker(&host_config)),
       async { 
           Timer::after(Duration::from_secs(300)).await;
           Err("Timeout".into())
       }
   ).await;
   ↓
7. Actor emits event based on result
   Success: SshEvent::DockerInstalled { name: "root@example.com" }
   Failure: SshEvent::ServiceError { 
       name: "root@example.com",
       service: "docker",
       operation: "install",
       error: "..." 
   }
   ↓
8. UI polls events in next frame
   for event in vm.poll_events(ui.ctx()) { ... }
   ↓
9. UI updates row data in handle_event()
   if let Some(row) = self.rows.iter_mut().find(|r| r.host == "root@example.com") {
       row.docker_enabled = true;
   }
   ↓
10. UI re-renders with updated status
    Status column: "✓ linux(debian) ✓ docker"
    Operations: [Docker Status] [Uninstall Docker] buttons now shown
```

### Initial Load Flow

```
1. Tab first rendered (loaded = false)
   ↓
2. load_rows() called
   ↓
3. Reads AppConfig from YAML
   - platforms: Vec<CloudPlatformConfig>
   - ssh_hosts: Vec<SshHostConfig>
   ↓
4. For each SSH host:
   - Resolve platform relationship (if platform_name exists)
   - Build SshRowData with config data
   - Set linux_detected = false (not yet queried)
   - Set service flags = false (not yet queried)
   ↓
5. Table renders with rows
   ↓
6. When user opens drawer (clicks row):
   - Check if row.linux_status.is_none()
   - If yes, show "(status not loaded - click drawer to load)"
   - Could optionally auto-trigger: vm.get_linux_status(row.host)
   ↓
7. User clicks "Refresh" button:
   - Triggers vm.get_linux_status(host)
   ↓
8. LinuxStatusRetrieved event arrives
   - Updates row.linux_status
   - Sets row.linux_detected = true
   - Drawer refreshes with data
```

### Error Handling Patterns

**1. Network/SSH Errors**
```rust
// Actor level - catch and emit error event
match calc::ssh::get_linux_status(&host_config) {
    Ok(status) => {
        let _ = tx.send(SshEvent::LinuxStatusRetrieved { 
            name, 
            status 
        }).await;
    }
    Err(e) => {
        let _ = tx.send(SshEvent::ServiceError {
            name,
            service: "linux".into(),
            operation: "get_status".into(),
            error: format!("SSH connection failed: {}", e),
        }).await;
    }
}
```

**2. UI Display**
```rust
// Global error display (below table)
if let Some(error) = &self.load_error {
    ui.colored_label(egui::Color32::RED, format!("⚠ {}", error));
}

// ViewModel recent errors (from MVVM pattern)
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
}
```

**3. Operation Timeouts**
```rust
// In actor - wrap long operations with timeout
use smol::Timer;
use std::time::Duration;

let result = smol::future::or(
    runtime::unblock(move || calc::ssh::install_docker(&host_config)),
    async {
        Timer::after(Duration::from_secs(300)).await;
        Err("Operation timed out after 5 minutes".to_string())
    }
).await;
```

**4. Config Save Failures**
```rust
// After successful operation, update config (non-fatal if fails)
match app_config.save(&config_path) {
    Ok(_) => {
        eprintln!("✓ Config saved");
    }
    Err(e) => {
        // Operation succeeded, config save failed - warn but don't fail
        eprintln!("⚠ Warning: Config save failed: {}", e);
        // Could show warning banner in UI
    }
}
```

**5. Partial Failures (Resilience)**
```rust
// When querying Linux status, some commands may fail
// Return partial data instead of failing completely
pub fn get_linux_status(host_config: &SshHostConfig) -> Result<LinuxStatus, String> {
    let session = establish_connection(host_config)?; // Only fail if SSH fails
    
    // Individual commands use unwrap_or for defaults
    Ok(LinuxStatus {
        uptime: execute_ssh_command(&session, "uptime -p")
            .unwrap_or_else(|_| "unknown".to_string()),
        external_ip: execute_ssh_command(&session, "curl -s ifconfig.me")
            .unwrap_or_else(|_| "unknown".to_string()),
        // ... etc - never fail due to single command
    })
}
```

**6. State Consistency (Concurrent Operations)**
```rust
// Problem: User deletes host while operation in progress
// Solution: Cancel active operations on host delete

impl ViewModel {
    pub fn delete_ssh_host(&mut self, host: String) -> Result<(), String> {
        // 1. Cancel any pending operations for this host
        self.cancel_operations_for_host(&host);
        
        // 2. Then send delete command
        self.send_ssh_command(SshCommand::DeleteHost { name: host })
    }
}

// Actor checks if host still exists before emitting event
async fn handle_operation_complete(name: String, result: Result<...>, tx: Sender) {
    // Verify host still exists
    if !host_exists_in_config(&name) {
        // Silently drop event - host was deleted
        return;
    }
    
    // Emit event
    let _ = tx.send(event).await;
}
```

---

## Testing Strategy

### Unit Tests

**1. Formatting Functions**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_platform_with_relationship() {
        let row = SshRowData {
            platform_name: Some("gcp".into()),
            platform_type: Some("GCP".into()),
            ..Default::default()
        };
        assert_eq!(format_platform(&row), "gcp(GCP)");
    }
    
    #[test]
    fn test_format_platform_manual() {
        let row = SshRowData {
            platform_name: None,
            platform_type: None,
            ..Default::default()
        };
        assert_eq!(format_platform(&row), "manual");
    }
    
    #[test]
    fn test_format_status_multiple_services() {
        let row = SshRowData {
            linux_detected: true,
            linux_os: Some("debian".into()),
            docker_enabled: true,
            ansible_enabled: false,
            dure_wss_enabled: false,
            ..Default::default()
        };
        assert_eq!(format_status(&row), "✓ linux(debian) ✓ docker");
    }
    
    #[test]
    fn test_format_status_empty() {
        let row = SshRowData {
            linux_detected: false,
            ..Default::default()
        };
        assert_eq!(format_status(&row), "—");
    }
}
```

**2. Calc Layer Tests**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_os_from_os_release() {
        // Mock test - would need SSH mock
        // Verify OS detection from /etc/os-release
    }
    
    #[test]
    fn test_check_docker_installed_true() {
        // Mock SSH session returning "/usr/bin/docker"
        // Verify returns true
    }
    
    #[test]
    fn test_check_docker_installed_false() {
        // Mock SSH session returning ""
        // Verify returns false
    }
    
    #[test]
    fn test_linux_status_partial_failure() {
        // Mock some commands failing
        // Verify still returns LinuxStatus with "unknown" for failed fields
    }
}
```

**3. Actor Event Tests**
```rust
#[cfg(test)]
mod tests {
    use smol_potat as smol;
    
    #[smol::test]
    async fn test_get_linux_status_success() {
        let (cmd_tx, cmd_rx) = channel::unbounded();
        let (event_tx, event_rx) = channel::unbounded();
        
        // Spawn actor
        spawn_ssh_actor(cmd_rx, event_tx);
        
        // Send command
        cmd_tx.send(SshCommand::GetLinuxStatus {
            name: "test@example.com".into()
        }).await.unwrap();
        
        // Receive event (with timeout)
        let event = smol::future::or(
            event_rx.recv(),
            async {
                smol::Timer::after(Duration::from_secs(5)).await;
                Err("timeout".into())
            }
        ).await.unwrap();
        
        match event {
            SshEvent::LinuxStatusRetrieved { name, status } => {
                assert_eq!(name, "test@example.com");
                assert!(!status.uptime.is_empty());
            }
            _ => panic!("Expected LinuxStatusRetrieved event"),
        }
    }
    
    #[smol::test]
    async fn test_install_docker_timeout() {
        // Test that long operations timeout correctly
    }
}
```

### Integration Tests

**1. Platform-SSH Integration**
```rust
#[test]
fn test_platform_ssh_relationship() {
    // 1. Create platform config
    let mut app_config = AppConfig::default();
    app_config.platforms.push(CloudPlatformConfig {
        name: "test-gcp".into(),
        platform_type: "GCP".into(),
        ..Default::default()
    });
    
    // 2. Add SSH host with platform_name
    app_config.ssh_hosts.push(SshHostConfig {
        host: "root@test.com".into(),
        port: 22,
        platform_name: Some("test-gcp".into()),
        ..Default::default()
    });
    
    // 3. Load rows
    let mut tab = SshTab::default();
    // ... simulate load_rows() logic
    
    // 4. Verify relationship
    assert_eq!(tab.rows[0].platform_name, Some("test-gcp".into()));
    assert_eq!(tab.rows[0].platform_type, Some("GCP".into()));
    assert_eq!(format_platform(&tab.rows[0]), "test-gcp(GCP)");
}

#[test]
fn test_manual_ssh_host() {
    // Verify manual hosts show "manual"
    let mut app_config = AppConfig::default();
    app_config.ssh_hosts.push(SshHostConfig {
        host: "root@manual.com".into(),
        port: 22,
        platform_name: None, // Manual host
        ..Default::default()
    });
    
    // ... load and verify shows "manual"
}
```

**2. End-to-End Flow (Manual/Requires Live SSH)**
```rust
#[test]
#[ignore] // Requires live SSH host
fn test_ssh_docker_lifecycle() {
    // 1. Add host
    // 2. Get Linux status
    // 3. Install Docker
    // 4. Verify Docker installed
    // 5. Get Docker status
    // 6. Uninstall Docker
    // 7. Verify Docker removed
    // 8. Delete host
}
```

### Manual Testing Checklist

**UI/UX:**
- [ ] Table renders with 4 columns (Host, Platform, Status, Operations)
- [ ] Drawer expands/collapses smoothly
- [ ] Drawer shows Linux status correctly
- [ ] Placeholder sections show "—"
- [ ] Status column updates after service install
- [ ] Operations buttons show/hide based on service state
- [ ] Progress bars display during long operations
- [ ] Error messages display clearly
- [ ] Add host dialog works
- [ ] Platform relationship displays correctly (platform name vs "manual")

**Service Operations:**
- [ ] Get Linux status retrieves all fields (uptime, IP, memory, disk, load, processes)
- [ ] Linux status shows "unknown" for failed queries (resilience)
- [ ] Install Docker succeeds (takes ~30-60s)
- [ ] Docker status shows correct state
- [ ] Uninstall Docker works
- [ ] Operations buttons switch based on service state
- [ ] Multiple operations can run concurrently
- [ ] Refresh button updates all services

**Error Scenarios:**
- [ ] Network timeout handled gracefully (5min timeout)
- [ ] SSH authentication failure shown
- [ ] Invalid host handled (error message)
- [ ] Service already installed handled
- [ ] Config save failure doesn't break operation
- [ ] Concurrent operations don't conflict
- [ ] Deleting host cancels pending operations

**Platform Integration:**
- [ ] Platform-created SSH hosts link correctly
- [ ] Manual hosts show "manual"
- [ ] Deleting platform doesn't break SSH host (shows "manual" after)
- [ ] Platform name updates propagate

### Performance Testing

**Metrics:**
- **Initial load time:** < 500ms for 20 hosts
- **Linux status query:** < 5s per host
- **Docker install:** < 60s on typical VPS
- **UI responsiveness:** No blocking during operations
- **Memory usage:** < 10MB additional for SSH tab data

**Load Testing:**
- Test with 50+ SSH hosts
- Test with 10 concurrent operations
- Test drawer open/close performance with many rows
- Test scroll performance with large table

### Regression Prevention

**Before Merge:**
1. Verify existing SSH operations still work:
   - [ ] Add host
   - [ ] Delete host
   - [ ] Test connection
   - [ ] Init host

2. Verify MVVM pattern consistency:
   - [ ] All new operations use ViewModel
   - [ ] No direct calc layer calls from UI
   - [ ] Event processing follows standard pattern
   - [ ] Actor handles all async operations

3. Verify no breaking changes:
   - [ ] Config format backward compatible (platform_name defaults to None)
   - [ ] Existing hosts load correctly
   - [ ] No GUI regressions in other tabs
   - [ ] MaterialSpreadsheet removal doesn't break anything

---

## Implementation Notes

### File Changes

**New Files:**
- None (all changes in existing files)

**Modified Files:**
1. `mobile/src/config.rs`
   - Add `platform_name: Option<String>` to `SshHostConfig`

2. `mobile/src/viewmodel/mod.rs`
   - Add service management methods (get_linux_status, install_docker, etc.)

3. `mobile/src/viewmodel/ssh/actor.rs`
   - Add new SshCommand variants
   - Add command handlers for service operations

4. `mobile/src/viewmodel/ssh/mod.rs`
   - Add new SshEvent variants

5. `mobile/src/calc/ssh.rs`
   - Add `get_linux_status()`, `detect_os()`
   - Add `check_docker_installed()`, `install_docker()`, `uninstall_docker()`
   - Add placeholders for ansible and dure-wss functions

6. `mobile/src/ui_tabs/ssh.rs`
   - Complete rewrite (~786 lines → ~1000 lines)
   - Remove MaterialSpreadsheet, use data_table
   - Remove init_promise, test_promise
   - Add SshRowData struct
   - Add render_table(), render_drawer_content(), format_*() functions
   - Add event handling, action processing

### Code Size Estimate

| Component | Estimated Lines |
|-----------|----------------|
| Data models (SshRowData, LinuxStatus) | ~50 |
| ViewModel methods | ~120 |
| Actor command handlers | ~300 |
| Calc layer functions | ~200 |
| UI layer rewrite | ~600 |
| Tests | ~200 |
| **Total** | **~1470 lines** |

**Code Reduction:**
- SSH tab: ~786 lines → ~1000 lines (+214 lines, but +drawer functionality)
- Overall: Net addition due to new features (service management)

### Dependencies

**No new dependencies required.** All functionality uses existing crates:
- `egui_material3::data_table` (already used in Platform tab)
- `smol` runtime (already in use)
- `russh` for SSH (already in use)

### Performance Considerations

1. **SSH Connection Pooling:** Not implemented in this design. Each operation opens/closes connection. Future optimization: maintain connection pool.

2. **Concurrent Operations:** Actor allows concurrent operations. Rate limiting not implemented. Future: add max concurrent ops limit.

3. **Drawer Auto-Load:** Design uses lazy loading (query on user action, not automatic). Alternative: auto-query on drawer open.

4. **Status Caching:** No caching of Linux status. Each "Refresh" re-queries. Future: add TTL-based cache.

### Security Considerations

1. **SSH Credentials:** Stored in config.yml (existing pattern). Future: use system keyring.

2. **Docker Install Script:** Uses official `get.docker.com`. Risk: script compromise. Mitigation: verify HTTPS, consider pinning script version.

3. **Command Injection:** calc layer uses russh with proper escaping. No user input directly in shell commands.

4. **Privilege Escalation:** Operations assume root SSH access. No sudo required.

---

## Migration Path

### Phase 1: Data Models & Config (1 day)
1. Add `platform_name` to SshHostConfig
2. Add SshRowData, LinuxStatus structs
3. Add new SshCommand, SshEvent variants
4. Test config backward compatibility

### Phase 2: Calc Layer (2 days)
1. Implement `get_linux_status()` with resilience
2. Implement `detect_os()`
3. Implement Docker functions (install, check, uninstall)
4. Add placeholders for ansible, dure-wss
5. Add unit tests

### Phase 3: Actor Layer (2 days)
1. Add command handlers for service operations
2. Add timeout wrappers
3. Add event emission logic
4. Add actor tests

### Phase 4: ViewModel API (1 day)
1. Add public methods
2. Wire to actor commands
3. Test command → event flow

### Phase 5: UI Layer (3 days)
1. Implement SshTab rewrite with data_table
2. Implement load_rows() with platform relationship
3. Implement render_table(), render_drawer_content()
4. Implement format_*() functions
5. Implement dynamic operations buttons
6. Implement action processing
7. Implement event handling

### Phase 6: Integration & Testing (2 days)
1. Manual testing of all operations
2. Platform integration testing
3. Error scenario testing
4. Performance testing
5. Regression testing

### Phase 7: Documentation & Cleanup (1 day)
1. Update user documentation
2. Update developer docs
3. Clean up old code
4. Final code review

**Total Estimated Time: 12 days**

### Rollback Plan

If major issues arise:
1. Revert UI layer changes
2. Keep actor/ViewModel additions (useful for future)
3. Restore MaterialSpreadsheet version
4. Config changes are backward compatible (no rollback needed)

---

## Future Enhancements

### Phase 2: Ansible Management
- Full lifecycle: install, configure, roles management
- Show ansible-galaxy roles in drawer
- Role installation from galaxy
- Playbook execution

### Phase 3: Docker Management
- Container list in drawer
- Container start/stop/restart
- Image management
- Docker Compose support

### Phase 4: Dure-WSS Management
- Full lifecycle: install, configure, start/stop
- Show service status in drawer
- Log viewing
- Configuration management

### Phase 5: Advanced Features
- SSH connection pooling for performance
- Status caching with TTL
- Bulk operations (install Docker on all hosts)
- Host grouping (dev, staging, prod)
- Custom SSH commands in drawer
- Terminal emulator in drawer

---

## Conclusion

This design provides a complete specification for reimplementing the SSH tab with expandable drawer functionality, matching the UX of the Platform tab while following MVVM architecture patterns. The implementation is scoped to deliver working Linux status display with placeholder sections for future service management features.

**Key Deliverables:**
- ✅ Data table with expandable drawers
- ✅ Platform relationship (SSH → Platform)
- ✅ Linux status display (uptime, IP, memory, disk, load, processes)
- ✅ Dynamic operations buttons based on service state
- ✅ Placeholder sections for ansible, docker, dure-wss
- ✅ Full MVVM integration with ViewModel/Actor pattern
- ✅ Comprehensive error handling
- ✅ Testing strategy

**Success Criteria:**
- SSH tab UX matches Platform tab (drawer pattern)
- Linux status query completes in < 5s
- UI remains responsive during operations
- No regressions in existing SSH functionality
- Config backward compatible
- All manual tests pass
